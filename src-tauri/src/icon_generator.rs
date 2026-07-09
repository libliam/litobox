use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, imageops::FilterType};
use std::io::Cursor;

/// 单个尺寸的图标生成结果
#[derive(serde::Serialize, Clone)]
pub struct IconPreview {
    pub size: u32,
    pub base64: String, // PNG base64
}

/// 图标生成结果
#[derive(serde::Serialize)]
pub struct IconResult {
    pub previews: Vec<IconPreview>,
    pub ico_base64: String, // 多尺寸 ICO 文件 base64
}

/// 读取文件并返回 base64（用于前端预览本地文件）
#[tauri::command]
pub fn read_file_base64(file_path: String) -> Result<String, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    Ok(STANDARD.encode(&bytes))
}

/// 生成多尺寸图标（PNG + ICO）
#[tauri::command]
pub fn generate_icon(file_path: String, sizes: Vec<u32>) -> Result<IconResult, String> {
    // 读取并解码图片
    let img = image::open(&file_path)
        .map_err(|e| format!("无法读取图片: {}", e))?;

    let mut previews: Vec<IconPreview> = Vec::new();
    let mut png_data_list: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in &sizes {
        let resized = img.resize_exact(size, size, FilterType::Lanczos3);

        // 编码为 PNG
        let mut png_bytes = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败 ({}x{}): {}", size, size, e))?;

        let base64 = STANDARD.encode(&png_bytes);
        previews.push(IconPreview { size, base64 });
        png_data_list.push((size, png_bytes));
    }

    // 生成多尺寸 ICO
    let ico_bytes = build_ico(&png_data_list)?;
    let ico_base64 = STANDARD.encode(&ico_bytes);

    Ok(IconResult {
        previews,
        ico_base64,
    })
}

/// 构建多尺寸 ICO 文件（PNG 编码，兼容 Vista+）
fn build_ico(pngs: &[(u32, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let count = pngs.len();
    if count == 0 {
        return Err("没有可生成的图标尺寸".into());
    }
    if count > u16::MAX as usize {
        return Err(format!("图标数量超过上限 {}", u16::MAX));
    }

    let mut ico = Vec::new();

    // ICO Header (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type: 1=ICO
    ico.extend_from_slice(&(count as u16).to_le_bytes()); // count

    // 计算总偏移
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