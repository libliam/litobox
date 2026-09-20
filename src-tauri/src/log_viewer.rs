//!
//! 日志实时查看器
//!
//! 打开日志文件后，后台线程轮询文件大小变化，
//! 将新增内容通过事件 `log-viewer://new-lines` 推送给前端。
//! 文件被截断（日志轮转）时发送 `log-viewer://truncated` 事件。
//!
//! 不引入 notify 等新依赖，使用 std::thread 轮询（300ms）。

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::file_encoding::TextEncoding;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
struct ViewerState {
    path: String,
    offset: u64,
    encoding: TextEncoding,
}

static STATE: Mutex<Option<ViewerState>> = Mutex::new(None);

#[derive(Serialize, Clone)]
struct LogEventPayload {
    content: String,
}

// ponytail: 单次最多读取 500KB，超大日志只看尾部避免撑爆内存
const TAIL_MAX_BYTES: u64 = 500 * 1024;

/// 解码字节：先试 UTF-8，失败回退 GBK
fn decode_bytes(bytes: &[u8]) -> (String, TextEncoding) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), TextEncoding::Utf8),
        Err(_) => {
            let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
            (decoded.into_owned(), TextEncoding::Gbk)
        }
    }
}

/// 读取文件尾部，返回 (内容, 编码, 文件大小)
fn read_tail(path: &str, max_bytes: u64) -> Result<(String, TextEncoding, u64), String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();

    let start = if file_size > max_bytes {
        file_size - max_bytes
    } else {
        0
    };

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // 非文件开头时，跳过第一行不完整内容（从换行符后开始）
    if start > 0 {
        if let Some(nl_pos) = buffer.iter().position(|&b| b == b'\n') {
            buffer = buffer[nl_pos + 1..].to_vec();
        }
    }

    let (text, encoding) = decode_bytes(&buffer);
    Ok((text, encoding, file_size))
}

/// 从指定偏移量读取到文件末尾，返回 (内容, 新文件大小)
fn read_from_offset(path: &str, offset: u64) -> Result<(String, u64), String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    let file_size = offset + buffer.len() as u64;
    let (text, _) = decode_bytes(&buffer);
    Ok((text, file_size))
}

/// 打开日志文件，返回尾部内容并启动后台监控线程
#[tauri::command]
pub fn log_viewer_open(app: AppHandle, file_path: String) -> Result<String, String> {
    debug_log!("[log_viewer] open: {}", file_path);

    let (content, encoding, file_size) = read_tail(&file_path, TAIL_MAX_BYTES)?;

    *STATE.lock().unwrap() = Some(ViewerState {
        path: file_path.clone(),
        offset: file_size,
        encoding,
    });

    RUNNING.store(true, Ordering::SeqCst);

    let app_clone = app.clone();
    thread::spawn(move || {
        loop {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(300));

            let state = match STATE.lock().unwrap().as_ref() {
                Some(s) => s.clone(),
                None => continue,
            };

            let metadata = match std::fs::metadata(&state.path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let current_size = metadata.len();

            if current_size == state.offset {
                continue;
            }

            if current_size < state.offset {
                // 文件被截断/轮转，重新读取尾部
                match read_tail(&state.path, TAIL_MAX_BYTES) {
                    Ok((content, enc, size)) => {
                        let _ = app_clone.emit(
                            "log-viewer://truncated",
                            LogEventPayload { content },
                        );
                        let mut s = STATE.lock().unwrap();
                        if let Some(st) = s.as_mut() {
                            st.offset = size;
                            st.encoding = enc;
                        }
                    }
                    Err(e) => debug_log!("[log_viewer] truncate read error: {}", e),
                }
            } else {
                // 读取新增内容
                match read_from_offset(&state.path, state.offset) {
                    Ok((content, size)) => {
                        let _ = app_clone.emit(
                            "log-viewer://new-lines",
                            LogEventPayload { content },
                        );
                        let mut s = STATE.lock().unwrap();
                        if let Some(st) = s.as_mut() {
                            st.offset = size;
                        }
                    }
                    Err(e) => debug_log!("[log_viewer] incremental read error: {}", e),
                }
            }
        }
        debug_log!("[log_viewer] watch thread exited");
    });

    Ok(content)
}

/// 停止监控
#[tauri::command]
pub fn log_viewer_close() -> Result<(), String> {
    debug_log!("[log_viewer] close");
    RUNNING.store(false, Ordering::SeqCst);
    *STATE.lock().unwrap() = None;
    Ok(())
}

/// 重新加载文件（手动刷新，读取尾部）
#[tauri::command]
pub fn log_viewer_reload(file_path: String) -> Result<String, String> {
    debug_log!("[log_viewer] reload: {}", file_path);
    let (content, encoding, file_size) = read_tail(&file_path, TAIL_MAX_BYTES)?;
    let mut s = STATE.lock().unwrap();
    if let Some(st) = s.as_mut() {
        st.offset = file_size;
        st.encoding = encoding;
    }
    Ok(content)
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bytes_utf8() {
        let bytes = "hello 世界".as_bytes();
        let (text, enc) = decode_bytes(bytes);
        assert_eq!(text, "hello 世界");
        assert_eq!(enc, TextEncoding::Utf8);
    }

    #[test]
    fn test_decode_bytes_gbk() {
        // "中文" 的 GBK 编码
        let bytes = vec![0xD6, 0xD0, 0xCE, 0xC4];
        let (text, enc) = decode_bytes(&bytes);
        assert_eq!(text, "中文");
        assert_eq!(enc, TextEncoding::Gbk);
    }

    #[test]
    fn test_decode_bytes_empty() {
        let (text, enc) = decode_bytes(&[]);
        assert_eq!(text, "");
        assert_eq!(enc, TextEncoding::Utf8);
    }

    #[test]
    fn test_read_tail_truncates_to_max() {
        // 创建一个临时文件，内容超过 max_bytes，验证返回内容不超过限制
        let tmp = std::env::temp_dir().join("litobox_log_test.log");
        let content = "a".repeat(200 * 1024); // 200KB
        std::fs::write(&tmp, &content).unwrap();

        let (text, _enc, size) = read_tail(tmp.to_str().unwrap(), 100 * 1024).unwrap();
        assert_eq!(size, 200 * 1024);
        // 读取了约 100KB（可能略少，因为跳过首行）
        assert!(text.len() <= 100 * 1024 + 1);
        assert!(text.len() > 50 * 1024);

        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_read_tail_small_file() {
        let tmp = std::env::temp_dir().join("litobox_log_small.log");
        std::fs::write(&tmp, "line1\nline2\nline3").unwrap();

        let (text, _enc, size) = read_tail(tmp.to_str().unwrap(), 100 * 1024).unwrap();
        assert_eq!(size, 17);
        assert_eq!(text, "line1\nline2\nline3");

        std::fs::remove_file(&tmp).ok();
    }
}
