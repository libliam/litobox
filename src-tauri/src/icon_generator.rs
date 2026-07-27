use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, imageops::FilterType};
use std::io::Cursor;

/// 鍗曚釜灏哄鐨勫浘鏍囩敓鎴愮粨鏋?
#[derive(serde::Serialize, Clone)]
pub struct IconPreview {
    pub size: u32,
    pub base64: String, // PNG base64
}

/// 鍥炬爣鐢熸垚缁撴灉
#[derive(serde::Serialize)]
pub struct IconResult {
    pub previews: Vec<IconPreview>,
    pub ico_base64: String, // 澶氬昂瀵?ICO 鏂囦欢 base64
}

/// 璇诲彇鏂囦欢骞惰繑鍥?base64锛堢敤浜庡墠绔瑙堟湰鍦版枃浠讹級
#[tauri::command]
pub fn read_file_base64(file_path: String) -> Result<String, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("璇诲彇鏂囦欢澶辫触: {}", e))?;
    Ok(STANDARD.encode(&bytes))
}
/// 复制文件到目标路径
#[tauri::command]
pub fn copy_file(from: String, to: String) -> Result<String, String> {
    std::fs::copy(&from, &to)
        .map_err(|e| format!("文件复制失败: {}", e))?;
    Ok(to)
}

/// 鐢熸垚澶氬昂瀵稿浘鏍囷紙PNG + ICO锛?
#[tauri::command]
pub fn generate_icon(file_path: String, sizes: Vec<u32>) -> Result<IconResult, String> {
    // 璇诲彇骞惰В鐮佸浘鐗?
    let img = image::open(&file_path)
        .map_err(|e| format!("鏃犳硶璇诲彇鍥剧墖: {}", e))?;

    let mut previews: Vec<IconPreview> = Vec::new();
    let mut png_data_list: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in &sizes {
        let resized = img.resize_exact(size, size, FilterType::Lanczos3);

        // 缂栫爜涓?PNG
        let mut png_bytes = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .map_err(|e| format!("PNG 缂栫爜澶辫触 ({}x{}): {}", size, size, e))?;

        let base64 = STANDARD.encode(&png_bytes);
        previews.push(IconPreview { size, base64 });
        png_data_list.push((size, png_bytes));
    }

    // 鐢熸垚澶氬昂瀵?ICO
    let ico_bytes = build_ico(&png_data_list)?;
    let ico_base64 = STANDARD.encode(&ico_bytes);

    Ok(IconResult {
        previews,
        ico_base64,
    })
}

/// 鏋勫缓澶氬昂瀵?ICO 鏂囦欢锛圥NG 缂栫爜锛屽吋瀹?Vista+锛?
fn build_ico(pngs: &[(u32, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let count = pngs.len();
    if count == 0 {
        return Err("娌℃湁鍙敓鎴愮殑鍥炬爣灏哄".into());
    }
    if count > u16::MAX as usize {
        return Err(format!("鍥炬爣鏁伴噺瓒呰繃涓婇檺 {}", u16::MAX));
    }

    let mut ico = Vec::new();

    // ICO Header (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type: 1=ICO
    ico.extend_from_slice(&(count as u16).to_le_bytes()); // count

    // 璁＄畻鎬诲亸绉?
    let header_size = 6 + 16 * count;
    let mut offset = header_size as u32;

    // ICO Directory Entries (16 bytes each)
    for (size, data) in pngs {
        let w = if *size >= 256 { 0u8 } else { *size as u8 };
        let h = if *size >= 256 { 0u8 } else { *size as u8 };
        ico.push(w);                              // width
        ico.push(h);                              // height
        ico.push(0u8);                            // color palette count
        ico.push(0u8);                            // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // bpp (32-bit)
        ico.extend_from_slice(&(data.len() as u32).to_le_bytes()); // image size
        ico.extend_from_slice(&offset.to_le_bytes()); // offset
        offset += data.len() as u32;
    }

    // Image Data (raw PNG)
    for (_, data) in pngs {
        ico.extend_from_slice(data);
    }

    Ok(ico)
}
