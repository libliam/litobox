use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, RgbaImage};
use serde::Serialize;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter};

static MONITORING: AtomicBool = AtomicBool::new(true);
static LAST_TEXT: Mutex<String> = Mutex::new(String::new());

#[derive(Clone, Serialize)]
pub struct ClipboardEntry {
    pub text: String,
    pub timestamp: u64,
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

            let text = clipboard.get_text().unwrap_or_default();

            if !text.is_empty() {
                let last = LAST_TEXT.lock().unwrap();
                if text != *last {
                    drop(last);
                    *LAST_TEXT.lock().unwrap() = text.clone();

                    let entry = ClipboardEntry {
                        text: text.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    let _ = app.emit("clipboard://new-entry", entry);
                }
            }

            thread::sleep(std::time::Duration::from_millis(500));
        }
    });
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
