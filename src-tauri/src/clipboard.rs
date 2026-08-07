use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, RgbaImage};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter};

use crate::db;

static MONITORING: AtomicBool = AtomicBool::new(true);
static LAST_SEQ: AtomicU32 = AtomicU32::new(0);
static LAST_TEXT_HASH: Mutex<String> = Mutex::new(String::new());
static LAST_IMAGE_HASH: Mutex<String> = Mutex::new(String::new());
static LAST_FILES_HASH: Mutex<String> = Mutex::new(String::new());

#[derive(Clone, Serialize)]
pub struct ClipboardEntry {
    pub entry_type: String, // "text", "image", "files"
    pub text: String,       // text content / image file path / JSON file paths
    pub meta: String,       // JSON metadata
    pub timestamp: u64,
}

/// 获取剪贴板图片缓存目录
fn get_clipboard_image_dir() -> Result<std::path::PathBuf, String> {
    let app_dir = dirs::config_dir()
        .ok_or("无法获取应用数据目录")?;
    let img_dir = app_dir.join("com.dev.toolbox").join("clipboard_images");
    std::fs::create_dir_all(&img_dir).map_err(|e| e.to_string())?;
    Ok(img_dir)
}

#[tauri::command]
pub fn start_clipboard_monitor(app: AppHandle) {
    MONITORING.store(true, Ordering::SeqCst);

    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };

        loop {
            if !MONITORING.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }

            // ponytail: 用 GetClipboardSequenceNumber 检测变更，避免每轮都读取内容
            let seq = get_clipboard_sequence_number();
            if seq == 0 || seq == LAST_SEQ.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }

            LAST_SEQ.store(seq, Ordering::SeqCst);

            // 按优先级检测：文件 > 图片 > 文本
            // 复制文件时剪贴板可能同时有文本（路径），优先取文件
            if let Some(files) = read_clipboard_files() {
                let hash = hash_string(&files.join(","));
                if hash != *LAST_FILES_HASH.lock().unwrap() {
                    *LAST_FILES_HASH.lock().unwrap() = hash.clone();
                    // 同步更新 text hash 防止文件路径的文本也被重复记录
                    *LAST_TEXT_HASH.lock().unwrap() = hash_string(&files.join("\n"));

                    let meta = format!("{{\"count\":{}}}", files.len());
                    let content = serde_json::to_string(&files).unwrap_or_default();

                    let _ = db::db_add_clipboard_record(
                        content.clone(),
                        "files".to_string(),
                        meta.clone(),
                    );

                    let _ = app.emit("clipboard://new-entry", ClipboardEntry {
                        entry_type: "files".to_string(),
                        text: content,
                        meta,
                        timestamp: now_ms(),
                    });
                }
                thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }

            if let Some(img_data) = read_clipboard_image_for_monitor() {
                let hash = hash_string(&img_data.base64_png[..64.min(img_data.base64_png.len())]);
                if hash != *LAST_IMAGE_HASH.lock().unwrap() {
                    *LAST_IMAGE_HASH.lock().unwrap() = hash.clone();
                    // 同步更新 text hash
                    *LAST_TEXT_HASH.lock().unwrap() = hash.clone();

                    // 保存图片到临时文件
                    let img_dir = match get_clipboard_image_dir() {
                        Ok(d) => d,
                        Err(_) => {
                            thread::sleep(std::time::Duration::from_millis(500));
                            continue;
                        }
                    };
                    let filename = format!("clip_{}.png", now_ms());
                    let img_path = img_dir.join(&filename);
                    let bytes = STANDARD.decode(&img_data.base64_png).unwrap_or_default();
                    if std::fs::write(&img_path, &bytes).is_ok() {
                        let path_str = img_path.to_string_lossy().to_string();
                        let meta = format!(
                            "{{\"width\":{},\"height\":{},\"size_bytes\":{}}}",
                            img_data.width, img_data.height, img_data.size_bytes
                        );

                        let _ = db::db_add_clipboard_record(
                            path_str.clone(),
                            "image".to_string(),
                            meta.clone(),
                        );

                        let _ = app.emit("clipboard://new-entry", ClipboardEntry {
                            entry_type: "image".to_string(),
                            text: path_str,
                            meta,
                            timestamp: now_ms(),
                        });
                    }
                }
                thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }

            // 文本检测
            let text = clipboard.get_text().unwrap_or_default();
            if !text.is_empty() {
                let hash = hash_string(&text);
                if hash != *LAST_TEXT_HASH.lock().unwrap() {
                    *LAST_TEXT_HASH.lock().unwrap() = hash;

                    let _ = db::db_add_clipboard_record(
                        text.clone(),
                        "text".to_string(),
                        "{}".to_string(),
                    );

                    let _ = app.emit("clipboard://new-entry", ClipboardEntry {
                        entry_type: "text".to_string(),
                        text,
                        meta: "{}".to_string(),
                        timestamp: now_ms(),
                    });
                }
            }

            thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// 读取剪贴板中的文件路径列表 (CF_HDROP)
#[cfg(target_os = "windows")]
fn read_clipboard_files() -> Option<Vec<String>> {
    use windows_sys::Win32::System::DataExchange::*;
    use windows_sys::Win32::UI::Shell::DragQueryFileW;

    const CF_HDROP: u32 = 15;

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }

        let result = (|| {
            let handle = GetClipboardData(CF_HDROP);
            if handle.is_null() {
                return None;
            }

            let count = DragQueryFileW(handle as _, 0xFFFFFFFF, std::ptr::null_mut(), 0);
            if count == 0 {
                return None;
            }

            let mut files = Vec::with_capacity(count as usize);
            for i in 0..count {
                let len = DragQueryFileW(handle as _, i, std::ptr::null_mut(), 0);
                if len == 0 {
                    continue;
                }
                let mut buf = vec![0u16; (len + 1) as usize];
                DragQueryFileW(handle as _, i, buf.as_mut_ptr(), (len + 1) as u32);
                if let Ok(s) = String::from_utf16(&buf[..len as usize]) {
                    files.push(s);
                }
            }

            if files.is_empty() { None } else { Some(files) }
        })();

        CloseClipboard();
        result
    }
}

#[cfg(not(target_os = "windows"))]
fn read_clipboard_files() -> Option<Vec<String>> {
    None
}

/// 监控用：读取剪贴板图片（轻量，不写 DB）
fn read_clipboard_image_for_monitor() -> Option<ClipboardImageData> {
    clipboard_get_image().ok().flatten()
}

/// 获取剪贴板序列号（变更检测）
#[cfg(target_os = "windows")]
fn get_clipboard_sequence_number() -> u32 {
    use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() }
}

#[cfg(not(target_os = "windows"))]
fn get_clipboard_sequence_number() -> u32 {
    0
}

#[tauri::command]
pub fn stop_clipboard_monitor() {
    MONITORING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn is_monitoring() -> bool {
    MONITORING.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct ClipboardImageData {
    pub width: u32,
    pub height: u32,
    pub base64_png: String,
    pub size_bytes: u32,
}

/// 从剪贴板读取图片，返回图片信息和 PNG base64 数据
#[tauri::command]
pub fn clipboard_get_image() -> Result<Option<ClipboardImageData>, String> {
    debug_log!("[clipboard_get_image] 开始读取剪贴板图片");

    // 先尝试 arboard（标准路径）
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            debug_log!("[clipboard_get_image] arboard 创建失败: {}, 尝试 Win32 API", e);
            return read_clipboard_image_win32();
        }
    };

    match clipboard.get_image() {
        Ok(img_data) => {
            let width = img_data.width as u32;
            let height = img_data.height as u32;
            debug_log!("[clipboard_get_image] arboard 成功, 尺寸: {}x{}", width, height);

            let rgba_img = RgbaImage::from_raw(width, height, img_data.bytes.into_owned())
                .ok_or_else(|| "图片数据构造失败".to_string())?;

            encode_png(rgba_img, width, height)
        }
        Err(arboard::Error::ContentNotAvailable) => {
            debug_log!("[clipboard_get_image] 剪贴板中没有图片");
            Ok(None)
        }
        Err(e) => {
            // arboard 失败（如浏览器复制的 PNG 格式），回退到 Win32 API
            debug_log!("[clipboard_get_image] arboard 失败: {}, 回退到 Win32 API", e);
            read_clipboard_image_win32()
        }
    }
}

/// 编码 RGBA 图片为 PNG，返回 ClipboardImageData
fn encode_png(rgba_img: RgbaImage, width: u32, height: u32) -> Result<Option<ClipboardImageData>, String> {
    let mut png_buf: Vec<u8> = Vec::new();
    image::DynamicImage::ImageRgba8(rgba_img)
        .write_to(&mut Cursor::new(&mut png_buf), ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;

    let size_bytes = png_buf.len() as u32;
    let base64_png = STANDARD.encode(&png_buf);
    debug_log!("[clipboard_get_image] PNG 编码完成, {} bytes", size_bytes);

    Ok(Some(ClipboardImageData { width, height, base64_png, size_bytes }))
}

// ============ Win32 API 回退：支持 PNG / DIB / DIBv5 格式 ============

#[cfg(target_os = "windows")]
fn read_clipboard_image_win32() -> Result<Option<ClipboardImageData>, String> {
    use windows_sys::Win32::System::DataExchange::*;
    use windows_sys::Win32::System::Memory::*;

    debug_log!("[clipboard_get_image] 使用 Win32 API 读取剪贴板");

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("无法打开剪贴板".to_string());
        }

        let result = (|| {
            // 1. 尝试 "PNG" 注册格式（浏览器复制的图片通常有这个）
            if let Some(data) = try_read_format(b"PNG\0") {
                debug_log!("[clipboard_get_image] 从 PNG 格式读取成功");
                return decode_image_bytes(&data);
            }

            // 2. 尝试 CF_DIBV5 (17)
            if let Some(data) = try_read_cf(17) {
                debug_log!("[clipboard_get_image] 从 CF_DIBV5 读取, {} bytes", data.len());
                return dib_to_image(&data);
            }

            // 3. 尝试 CF_DIB (8)
            if let Some(data) = try_read_cf(8) {
                debug_log!("[clipboard_get_image] 从 CF_DIB 读取, {} bytes", data.len());
                return dib_to_image(&data);
            }

            debug_log!("[clipboard_get_image] 剪贴板中未找到图片数据");
            Ok(None)
        })();

        CloseClipboard();
        result
    }
}

#[cfg(target_os = "windows")]
unsafe fn try_read_format(name: &[u8]) -> Option<Vec<u8>> {
    use windows_sys::Win32::System::DataExchange::*;
    use windows_sys::Win32::System::Memory::*;

    let format_id = RegisterClipboardFormatA(name.as_ptr());
    if format_id == 0 {
        return None;
    }

    let handle = GetClipboardData(format_id);
    if handle.is_null() {
        return None;
    }

    let ptr = GlobalLock(handle);
    if ptr.is_null() {
        return None;
    }

    let size = GlobalSize(handle);
    let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();

    GlobalUnlock(handle);
    Some(bytes)
}

#[cfg(target_os = "windows")]
unsafe fn try_read_cf(format: u32) -> Option<Vec<u8>> {
    use windows_sys::Win32::System::DataExchange::*;
    use windows_sys::Win32::System::Memory::*;

    let handle = GetClipboardData(format);
    if handle.is_null() {
        return None;
    }

    let ptr = GlobalLock(handle);
    if ptr.is_null() {
        return None;
    }

    let size = GlobalSize(handle);
    let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();

    GlobalUnlock(handle);
    Some(bytes)
}

/// 直接解码 PNG/JPEG 字节
#[cfg(target_os = "windows")]
fn decode_image_bytes(data: &[u8]) -> Result<Option<ClipboardImageData>, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| format!("图片解码失败: {}", e))?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    encode_png(rgba, w, h)
}

/// 将 DIB (BITMAPINFOHEADER + pixels) 转换为图片
/// 通过构造 BMP 文件头后用 image crate 解码
#[cfg(target_os = "windows")]
fn dib_to_image(dib_data: &[u8]) -> Result<Option<ClipboardImageData>, String> {
    if dib_data.len() < 40 {
        return Ok(None);
    }

    // 读取 BITMAPINFOHEADER 字段
    let header_size = u32::from_le_bytes([dib_data[0], dib_data[1], dib_data[2], dib_data[3]]);
    let bit_count = u16::from_le_bytes([dib_data[14], dib_data[15]]);
    let compression = u32::from_le_bytes([dib_data[16], dib_data[17], dib_data[18], dib_data[19]]);
    let clr_used = u32::from_le_bytes([dib_data[32], dib_data[33], dib_data[34], dib_data[35]]);

    debug_log!("[clipboard_get_image] DIB: header={}bit, compression={}, clr_used={}", bit_count, compression, clr_used);

    // BI_PNG (5): 头后面直接是 PNG 数据
    if compression == 5 {
        let offset = header_size as usize;
        if offset < dib_data.len() {
            return decode_image_bytes(&dib_data[offset..]);
        }
    }

    // BI_JPEG (4): 头后面直接是 JPEG 数据
    if compression == 4 {
        let offset = header_size as usize;
        if offset < dib_data.len() {
            return decode_image_bytes(&dib_data[offset..]);
        }
    }

    // BI_RGB (0) / BI_BITFIELDS (3): 构造 BMP 文件头
    let palette_size = if bit_count <= 8 {
        let colors = if clr_used > 0 { clr_used } else { 1u32 << bit_count };
        colors * 4
    } else {
        0
    };

    let pixel_offset = 14 + header_size + palette_size;
    let file_size = dib_data.len() as u32 + 14;

    let mut bmp_data = Vec::with_capacity(dib_data.len() + 14);
    bmp_data.extend_from_slice(b"BM");
    bmp_data.extend_from_slice(&file_size.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes()); // 保留字段
    bmp_data.extend_from_slice(&pixel_offset.to_le_bytes());
    bmp_data.extend_from_slice(dib_data);

    let img = image::load_from_memory_with_format(&bmp_data, ImageFormat::Bmp)
        .map_err(|e| format!("DIB 解码失败: {}", e))?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    encode_png(rgba, w, h)
}

#[cfg(not(target_os = "windows"))]
fn read_clipboard_image_win32() -> Result<Option<ClipboardImageData>, String> {
    Err("当前平台不支持 Win32 API 回退".to_string())
}

/// 将 base64 PNG 图片写入剪贴板
#[tauri::command]
pub fn clipboard_set_image(base64_png: String) -> Result<(), String> {
    debug_log!("[clipboard_set_image] 开始写入图片到剪贴板, base64 长度: {}", base64_png.len());
    let bytes = STANDARD.decode(&base64_png).map_err(|e| {
        debug_log!("[clipboard_set_image] Base64 解码失败: {}", e);
        format!("Base64 解码失败: {}", e)
    })?;

    let img = image::load_from_memory(&bytes).map_err(|e| {
        debug_log!("[clipboard_set_image] 图片解码失败: {}", e);
        format!("图片解码失败: {}", e)
    })?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let raw_bytes = rgba.into_raw();

    let mut clipboard = Clipboard::new().map_err(|e| {
        debug_log!("[clipboard_set_image] 创建剪贴板实例失败: {}", e);
        e.to_string()
    })?;

    let img_data = arboard::ImageData {
        width: w as usize,
        height: h as usize,
        bytes: std::borrow::Cow::Owned(raw_bytes),
    };

    clipboard.set_image(img_data).map_err(|e| {
        debug_log!("[clipboard_set_image] 写入剪贴板失败: {}", e);
        e.to_string()
    })?;

    debug_log!("[clipboard_set_image] 成功写入剪贴板, 尺寸: {}x{}", w, h);
    Ok(())
}

/// 读取图片文件为 base64（前端展示缩略图用）
#[tauri::command]
pub fn clipboard_read_image_file(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(STANDARD.encode(&bytes))
}

/// 删除指定图片缓存文件
#[tauri::command]
pub fn clipboard_delete_image_file(path: String) -> Result<(), String> {
    if !path.is_empty() {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

/// 清空所有图片缓存
#[tauri::command]
pub fn clipboard_clear_image_cache() -> Result<(), String> {
    let img_dir = get_clipboard_image_dir()?;
    if img_dir.exists() {
        std::fs::remove_dir_all(&img_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&img_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}
