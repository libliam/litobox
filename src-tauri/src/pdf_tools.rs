use std::fs;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use flate2::read::ZlibDecoder;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, RgbImage, GrayImage, RgbaImage};
use image::imageops::FilterType;
use lopdf::{Document, Object, Stream, Dictionary};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct CompressResult {
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
}

static GS_AVAILABLE: AtomicBool = AtomicBool::new(false);
static GS_CHECKED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn detect_ghostscript() -> bool {
    if GS_CHECKED.load(Ordering::Relaxed) {
        return GS_AVAILABLE.load(Ordering::Relaxed);
    }
    let available = Command::new("where")
        .arg("gswin64c.exe")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("where")
            .arg("gswin32c.exe")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    GS_AVAILABLE.store(available, Ordering::Relaxed);
    GS_CHECKED.store(true, Ordering::Relaxed);
    available
}

#[tauri::command]
pub fn get_pdf_page_count(file_path: String) -> Result<u32, String> {
    let doc = Document::load(&file_path).map_err(|e| format!("PDF 加载失败: {}", e))?;
    Ok(doc.get_pages().len() as u32)
}

#[tauri::command]
pub async fn compress_pdf(
    file_path: String,
    level: u8,
    gs_available: bool,
    target_dpi: Option<f64>,
    jpeg_quality: Option<u8>,
) -> Result<CompressResult, String> {
    let original_size = fs::metadata(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?
        .len();

    let result = tauri::async_runtime::spawn_blocking(move || {
        do_compress_pdf(&file_path, level, gs_available, target_dpi, jpeg_quality)
    })
    .await
    .map_err(|e| format!("压缩线程异常: {}", e))??;

    Ok(CompressResult {
        compressed_size: fs::metadata(&result.output_path)
            .map_err(|e| format!("读取输出文件失败: {}", e))?
            .len(),
        original_size,
        ..result
    })
}

fn do_compress_pdf(
    file_path: &str,
    level: u8,
    gs_available: bool,
    custom_dpi: Option<f64>,
    custom_quality: Option<u8>,
) -> Result<CompressResult, String> {
    let mut doc = Document::load(file_path)
        .map_err(|e| format!("PDF 加载失败: {}", e))?;

    // 检查是否加密
    if doc.is_encrypted() {
        return Err("不支持加密 PDF".into());
    }

    // 压缩参数：自定义覆盖预设
    let (target_dpi, jpeg_quality) = match (custom_dpi, custom_quality) {
        (Some(dpi), Some(q)) => (dpi, q),
        _ => match level {
            1 => (150.0, 92u8),  // 快速：高质量，轻微压缩
            2 => (150.0, 75u8),  // 标准：均衡质量
            _ => (72.0, 50u8),   // 极限：低质量，最大压缩
        },
    };

    // 遍历所有页面，处理图片 XObject
    let page_map = doc.get_pages();
    let page_ids: Vec<u32> = page_map.keys().copied().collect();
    for page_id in &page_ids {
        if let Some(object_id) = page_map.get(page_id) {
            if let Ok(page_obj) = doc.get_object(*object_id) {
                if let Ok(page_dict) = page_obj.as_dict() {
                    let resources = page_dict.get(b"Resources").ok().cloned().unwrap_or(Object::Null);
                    if resources.as_dict().is_ok() {
                        process_page_resources(&mut doc, &resources, target_dpi, jpeg_quality, level, custom_dpi)?;
                    }
                }
            }
        }
    }

    // 元数据清理
    if level >= 1 {
        doc.trailer.remove(b"MarkInfo");
    }
    if level >= 2 {
        doc.trailer.remove(b"Info");
    }

    // 保存临时文件
    let temp_dir = std::env::temp_dir();
    let file_stem = PathBuf::from(file_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let temp_output = temp_dir.join(format!("{}_compressed.pdf", file_stem));
    doc.save(&temp_output).map_err(|e| format!("保存 PDF 失败: {}", e))?;

    // 极限压缩：尝试 Ghostscript
    if level == 3 && gs_available {
        let gs_output = temp_dir.join(format!("{}_gs_compressed.pdf", file_stem));
        let gs_result = Command::new("gswin64c.exe")
            .args([
                "-sDEVICE=pdfwrite",
                "-dPDFSETTINGS=/screen",
                "-dNOPAUSE",
                "-dQUIET",
                "-dBATCH",
                &format!("-sOutputFile={}", gs_output.to_string_lossy()),
                &temp_output.to_string_lossy().replace('\\', "/"),
            ])
            .creation_flags(0x08000000)
            .output();

        if let Ok(output) = gs_result {
            if output.status.success() && gs_output.exists() {
                let rust_size = fs::metadata(&temp_output).map(|m| m.len()).unwrap_or(0);
                let gs_size = fs::metadata(&gs_output).map(|m| m.len()).unwrap_or(0);
                if gs_size < rust_size {
                    let _ = fs::remove_file(&temp_output);
                    let _ = fs::rename(&gs_output, &temp_output);
                } else {
                    let _ = fs::remove_file(&gs_output);
                }
            }
        }
    }

    Ok(CompressResult {
        output_path: temp_output.to_string_lossy().to_string(),
        original_size: 0,
        compressed_size: 0,
    })
}

fn process_page_resources(
    doc: &mut Document,
    resources: &lopdf::Object,
    target_dpi: f64,
    jpeg_quality: u8,
    level: u8,
    custom_dpi: Option<f64>,
) -> Result<(), String> {
    let dict = match resources.as_dict() {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };

    if let Ok(xobjects) = dict.get(b"XObject") {
        if let Ok(xobj_dict) = xobjects.as_dict() {
            for (_, obj_id) in xobj_dict.iter() {
                if let Ok(obj_id) = obj_id.as_reference() {
                    if let Ok(obj) = doc.get_object_mut(obj_id) {
                        if let Ok(stream) = obj.as_stream() {
                            let sdict = &stream.dict;
                            if sdict.get(b"Subtype").ok().map(|o| o.as_name().ok()) == Some(Some(b"Image")) {
                                let width = sdict.get(b"Width").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0) as u32;
                                let height = sdict.get(b"Height").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0) as u32;
                                if width == 0 || height == 0 {
                                    continue;
                                }

                                let raw_data = stream.content.clone();
                                let bpc = sdict.get(b"BitsPerComponent").ok().and_then(|o| o.as_i64().ok()).unwrap_or(8) as u8;
                                let color_space = sdict.get(b"ColorSpace").ok().and_then(|o| o.as_name().ok().map(|n| String::from_utf8_lossy(n).to_string())).unwrap_or_else(|| "Unknown".into());
                                let filter = sdict.get(b"Filter").ok().and_then(|o| o.as_name().ok().map(|n| String::from_utf8_lossy(n).to_string())).unwrap_or_else(|| "None".into());

                                eprintln!("[PDF压缩] 图片: {}x{}, BPC={}, ColorSpace={}, Filter={}, 数据大小={} bytes",
                                    width, height, bpc, color_space, filter, raw_data.len());

                                // 尝试构建 DynamicImage
                                let img = build_image_from_pdf(&raw_data, width, height, sdict, bpc);

                                if let Some(img) = img {
                                    // 计算目标像素尺寸：预设档位用固定缩放比，自定义 DPI 用公式
                                    let scale_factor = if custom_dpi.is_some() {
                                        // 自定义模式：DPI 越低缩越狠，300=不缩，72≈25%
                                        (target_dpi / 300.0).clamp(0.15, 1.0)
                                    } else {
                                        // 预设模式：只对超大图片缩放，普通图片只做 JPEG 重编码（质量降低）
                                        let max_dim = width.max(height);
                                        match level {
                                            1 => {
                                                // 快速：>4000px 缩到 85%，其余不缩
                                                if max_dim > 4000 { 0.85 } else { 1.0 }
                                            }
                                            2 => {
                                                // 标准：>3000px 缩到 75%，其余不缩
                                                if max_dim > 3000 { 0.75 } else { 1.0 }
                                            }
                                            _ => {
                                                // 极限：>800px 缩到 50%，其余不缩
                                                if max_dim > 800 { 0.50 } else { 1.0 }
                                            }
                                        }
                                    };

                                    let (new_w, new_h) = if scale_factor < 1.0 {
                                        let nw = (width as f64 * scale_factor).round() as u32;
                                        let nh = (height as f64 * scale_factor).round() as u32;
                                        (nw.max(1), nh.max(1))
                                    } else {
                                        (width, height)
                                    };

                                    let resized = if (new_w, new_h) != (width, height) {
                                        img.resize_exact(new_w, new_h, FilterType::Lanczos3)
                                    } else {
                                        img
                                    };

                                    let mut buf = Vec::new();
                                    let mut encoder = JpegEncoder::new_with_quality(&mut buf, jpeg_quality);
                                    if encoder.encode_image(&resized).is_ok() {
                                        let mut new_dict = Dictionary::new();
                                        new_dict.set("Type", Object::Name(b"XObject".to_vec()));
                                        new_dict.set("Subtype", Object::Name(b"Image".to_vec()));
                                        new_dict.set("Width", Object::Integer(new_w as i64));
                                        new_dict.set("Height", Object::Integer(new_h as i64));
                                        new_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
                                        new_dict.set("BitsPerComponent", Object::Integer(8));
                                        new_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                                        let mut new_stream = Stream::new(new_dict, buf);
                                        let _ = new_stream.compress();
                                        *obj = Object::Stream(new_stream);
                                        eprintln!("[PDF压缩] 图片已压缩: {}x{} -> {}x{}, JPEG质量={}", width, height, new_w, new_h, jpeg_quality);
                                    }
                                } else {
                                    eprintln!("[PDF压缩] 图片解码失败，跳过: {}x{} ColorSpace={}", width, height, color_space);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// 从 PDF 图片流构建 DynamicImage
/// PDF 图片可能是：
/// 1. DCTDecode（JPEG）— 直接解码
/// 2. FlateDecode（zlib）— 解压后是原始像素
/// 3. 无 Filter — 原始像素
fn build_image_from_pdf(
    raw_data: &[u8],
    width: u32,
    height: u32,
    dict: &Dictionary,
    bpc: u8,
) -> Option<DynamicImage> {
    // 优先尝试作为完整图片文件解码（DCTDecode 通常是 JPEG）
    if let Ok(img) = image::load_from_memory(raw_data) {
        return Some(img);
    }

    // 尝试指定 JPEG 格式解码
    if let Ok(img) = image::load_from_memory_with_format(raw_data, image::ImageFormat::Jpeg) {
        return Some(img);
    }

    // 尝试指定 PNG 格式解码
    if let Ok(img) = image::load_from_memory_with_format(raw_data, image::ImageFormat::Png) {
        return Some(img);
    }

    // 尝试 FlateDecode 解压（zlib）
    let decompressed = if let Ok(filter) = dict.get(b"Filter") {
        if let Ok(name) = filter.as_name() {
            if name == b"FlateDecode" {
                let mut decoder = ZlibDecoder::new(raw_data);
                let mut buf = Vec::new();
                if decoder.read_to_end(&mut buf).is_ok() {
                    Some(buf)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let pixel_data = decompressed.as_deref().unwrap_or(raw_data);

    // 原始像素数据：根据 ColorSpace 和 BitsPerComponent 构建
    let color_space = dict.get(b"ColorSpace").ok().and_then(|o| o.as_name().ok());
    let channels = match color_space {
        Some(b"DeviceGray") => 1u8,
        Some(b"DeviceRGB") => 3u8,
        Some(b"DeviceCMYK") => return None, // CMYK 暂不处理
        _ => 3u8, // 默认 RGB
    };

    if bpc != 8 {
        return None; // 只处理 8 位
    }

    let expected_len = (width * height * channels as u32) as usize;
    if pixel_data.len() < expected_len {
        return None; // 数据不足
    }

    let pixel_data = &pixel_data[..expected_len];

    match channels {
        1 => {
            let img = GrayImage::from_raw(width, height, pixel_data.to_vec())?;
            Some(DynamicImage::ImageLuma8(img))
        }
        3 => {
            let img = RgbImage::from_raw(width, height, pixel_data.to_vec())?;
            Some(DynamicImage::ImageRgb8(img))
        }
        4 => {
            let img = RgbaImage::from_raw(width, height, pixel_data.to_vec())?;
            Some(DynamicImage::ImageRgba8(img))
        }
        _ => None,
    }
}

#[tauri::command]
pub fn save_temp_file(data: String, filename: String) -> Result<String, String> {
    let bytes = STANDARD
        .decode(&data)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(&filename);
    fs::write(&temp_path, &bytes)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    Ok(temp_path.to_string_lossy().to_string())
}