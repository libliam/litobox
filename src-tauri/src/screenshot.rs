//! D2: 截图工具后端命令
//!
//! 架构（lazy）：
//! 1. `screenshot_capture_fullscreen` 用 GDI 捕获虚拟屏幕全屏帧 → 返回 PNG base64
//! 2. 前端拿到底图后自己在全屏 Canvas 上做选框、标注、裁剪、马赛克
//! 3. 处理完成后前端把最终 PNG base64 回传给：
//!    - `screenshot_write_clipboard_image`：arboard 写图片到剪贴板
//!    - `screenshot_save_file`：保存到指定路径（或默认 图片/LitoBox/ 目录）
//!
//! 简化：不新建额外窗口，前端直接用全屏主窗口（maximized + frameless 路由方式）做截图层

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use arboard::Clipboard;
use image::{ImageBuffer, RgbaImage, Rgba, ImageFormat};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

// ============ 公共宏和常量（项目已有惯例） ============
#[cfg(target_os = "windows")]
const _CREATE_NO_WINDOW: u32 = 0x08000000;

#[allow(unused_macros)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ==================== GDI 全屏捕获 ====================

#[derive(Debug, Clone, Serialize)]
pub struct CaptureResult {
    pub base64: String,
    pub width: u32,
    pub height: u32,
    pub total_width: i32,
    pub total_height: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[cfg(windows)]
fn do_capture_fullscreen() -> Result<CaptureResult, String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        GetDIBits, GetObjectW, ReleaseDC, SelectObject, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP, HDC, SRCCOPY,
        HGDIOBJ,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetDesktopWindow, GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    };
    use std::ffi::c_void;

    // 1. 虚拟屏幕尺寸
    let dx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let dy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let dw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let dh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    debug_log!("[screenshot] virtual screen: x={}, y={}, w={}, h={}", dx, dy, dw, dh);

    if dw <= 0 || dh <= 0 {
        return Err("无法获取虚拟屏幕尺寸".to_string());
    }

    // 2. DC 准备
    let desktop = unsafe { GetDesktopWindow() };
    let hdc_screen = unsafe {
        windows_sys::Win32::Graphics::Gdi::GetDC(desktop)
    };
    if hdc_screen == 0 as HDC {
        return Err("GetDC(Desktop) failed".to_string());
    }
    let hdc_mem = unsafe { CreateCompatibleDC(hdc_screen) };
    if hdc_mem == 0 as HDC {
        unsafe { ReleaseDC(desktop, hdc_screen); }
        return Err("CreateCompatibleDC failed".to_string());
    }
    let w = dw as u32;
    let h = dh as u32;
    let hbmp: HBITMAP = unsafe {
        CreateCompatibleBitmap(hdc_screen, dw, dh) as HBITMAP
    };
    if hbmp == 0 as HBITMAP {
        unsafe { DeleteDC(hdc_mem); ReleaseDC(desktop, hdc_screen); }
        return Err("CreateCompatibleBitmap failed".to_string());
    }
    unsafe { SelectObject(hdc_mem, hbmp as HGDIOBJ); }

    // 3. BitBlt 复制
    let ok = unsafe {
        BitBlt(
            hdc_mem,
            0, 0, dw, dh,
            hdc_screen,
            dx, dy,
            SRCCOPY | CAPTUREBLT,
        )
    };
    if ok == 0 {
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        unsafe { DeleteObject(hbmp as HGDIOBJ); DeleteDC(hdc_mem); ReleaseDC(desktop, hdc_screen); }
        return Err(format!("BitBlt failed, err={}", err));
    }

    // 4. 从 HBITMAP 读取像素
    let mut bmp: BITMAP = unsafe { std::mem::zeroed() };
    let got = unsafe {
        GetObjectW(
            hbmp as HGDIOBJ,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bmp as *mut _ as *mut c_void,
        )
    };
    if got == 0 {
        unsafe { DeleteObject(hbmp as HGDIOBJ); DeleteDC(hdc_mem); ReleaseDC(desktop, hdc_screen); }
        return Err("GetObjectW on HBITMAP failed".to_string());
    }
    let stride = (((bmp.bmWidth as i64) * 32 + 31) / 32 * 4) as i32;
    let buf_size = (stride as i64) * (bmp.bmHeight as i64);
    let mut buf: Vec<u8> = vec![0u8; buf_size.max(0) as usize];
    let mut bi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = bmp.bmWidth;
    bi.bmiHeader.biHeight = -bmp.bmHeight; // top-down
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;

    let lines = unsafe {
        GetDIBits(
            hdc_mem, hbmp,
            0, h,
            buf.as_mut_ptr() as *mut c_void,
            &mut bi, DIB_RGB_COLORS,
        )
    };
    debug_log!("[screenshot] GetDIBits lines={}, stride={}, w*h={}", lines, stride, w * h);

    unsafe {
        DeleteObject(hbmp as HGDIOBJ);
        DeleteDC(hdc_mem);
        ReleaseDC(desktop, hdc_screen);
    }

    if lines == 0 {
        return Err("GetDIBits returned 0 lines".to_string());
    }

    // 5. BGRA → RGBA
    let mut img: RgbaImage = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src_off = (y as i32) * stride + (x as i32) * 4;
            let s = src_off as usize;
            let b = buf[s];
            let g = buf[s + 1];
            let r = buf[s + 2];
            let a = buf[s + 3];
            img.put_pixel(x, y, Rgba([r, g, b, if a == 0 { 255 } else { a }]));
        }
    }

    // 6. PNG 编码
    let mut png_bytes: Vec<u8> = Vec::new();
    {
        let mut cur = Cursor::new(&mut png_bytes);
        img.write_to(&mut cur, ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {}", e))?;
    }
    let b64 = BASE64.encode(&png_bytes);
    debug_log!("[screenshot] capture ok: {}x{}, png={}b, b64={}", w, h, png_bytes.len(), b64.len());

    Ok(CaptureResult {
        base64: b64,
        width: w,
        height: h,
        total_width: dw,
        total_height: dh,
        offset_x: dx,
        offset_y: dy,
    })
}

#[cfg(not(windows))]
fn do_capture_fullscreen() -> Result<CaptureResult, String> {
    Err("截图功能仅支持 Windows".to_string())
}

// ==================== 命令：截全屏 ====================

#[tauri::command]
pub async fn screenshot_capture_fullscreen() -> Result<CaptureResult, String> {
    tauri::async_runtime::spawn_blocking(|| do_capture_fullscreen())
        .await
        .map_err(|e| format!("截图任务失败: {}", e))?
}

// ==================== 命令：写图片到剪贴板 ====================

#[tauri::command]
pub async fn screenshot_write_clipboard_image(base64_png: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let trim = base64_png.trim();
        let data = trim.trim_start_matches("data:image/png;base64,");
        let b64_len = data.len();
        debug_log!("[screenshot] clipboard: b64_len={} bytes", b64_len);
        let bytes = BASE64.decode(data)
            .map_err(|e| format!("base64 解码失败: {}", e))?;
        debug_log!("[screenshot] clipboard: decoded {} raw bytes", bytes.len());
        let img = image::load_from_memory_with_format(&bytes, ImageFormat::Png)
            .map_err(|e| format!("PNG 解码失败: {}", e))?
            .to_rgba8();
        let (w, h) = img.dimensions();
        let pixels: Vec<u8> = img.into_raw();
        debug_log!("[screenshot] clipboard: pixel data {}x{}, {} bytes", w, h, pixels.len());

        let mut clip = Clipboard::new().map_err(|e| format!("剪贴板不可用: {}", e))?;
        let img_data = arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: std::borrow::Cow::Owned(pixels),
        };
        clip.set_image(img_data).map_err(|e| format!("写图片剪贴板失败: {}", e))?;
        debug_log!("[screenshot] clipboard set ok: {}x{}", w, h);
        Ok(())
    })
    .await
    .map_err(|e| format!("截图任务失败: {}", e))?
}

// ==================== 命令：保存截图到文件 ====================

#[derive(Debug, Clone, Deserialize)]
pub struct SaveOptions {
    /// 保存目录；为空时用默认：图片/LitoBox/
    pub dir: Option<String>,
    /// 文件名（不含扩展名）；为空时自动加时间戳
    pub filename: Option<String>,
    /// 保留字段（前端自行弹 save dialog 后传入完整 dir + filename）
    pub show_dialog: Option<bool>,
}

fn default_screenshot_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join("Pictures").join("LitoBox")
}

fn timestamp_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    // 人类可读时间戳 (chrono-like): YYYYMMDD_HHMMSS，手动拼
    let secs_total = dur.as_secs();
    // 不用 chrono，采用 secs_since_epoch + 毫秒附加
    let ms = dur.subsec_millis();
    format!("截图_{}_{:03}", secs_total, ms)
}

#[tauri::command]
pub async fn screenshot_save_file(
    app_handle: AppHandle,
    base64_png: String,
    options: Option<SaveOptions>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let trim = base64_png.trim();
        let data = trim.trim_start_matches("data:image/png;base64,");
        let bytes = BASE64.decode(data)
            .map_err(|e| format!("base64 解码失败: {}", e))?;

        let opts = options.unwrap_or(SaveOptions { dir: None, filename: None, show_dialog: None });

        let dir = opts.dir
            .map(PathBuf::from)
            .unwrap_or_else(default_screenshot_dir);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
        let fname = opts.filename.unwrap_or_else(timestamp_name);
        let path = dir.join(format!("{}.png", fname));

        std::fs::write(&path, &bytes)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        let mut display = path.display().to_string();
        if let Some(stripped) = display.strip_prefix(r"\\?\") {
            display = stripped.to_string();
        }
        debug_log!("[screenshot] saved to: {}", display);
        let _ = app_handle.emit("screenshot://saved", serde_json::json!({
            "path": display,
            "size": bytes.len(),
        }));
        Ok(display)
    })
    .await
    .map_err(|e| format!("保存任务失败: {}", e))?
}

// ==================== 命令：获取默认保存目录 ====================

#[tauri::command]
pub fn screenshot_get_default_dir() -> String {
    let p = default_screenshot_dir();
    let s = p.display().to_string();
    s.strip_prefix(r"\\?\").map(|x| x.to_string()).unwrap_or(s)
}
