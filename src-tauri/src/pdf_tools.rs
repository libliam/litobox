use std::fs;
use std::io::Cursor;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageFormat};
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
) -> Result<CompressResult, String> {
    let original_size = fs::metadata(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?
        .len();

    let result = tauri::async_runtime::spawn_blocking(move || {
        do_compress_pdf(&file_path, level, gs_available)
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

fn do_compress_pdf(file_path: &str, level: u8, gs_available: bool) -> Result<CompressResult, String> {
    let mut doc = Document::load(file_path)
        .map_err(|e| format!("PDF 加载失败: {}", e))?;

    // 检查是否加密
    if doc.is_encrypted() {
        return Err("不支持加密 PDF".into());
    }

    // 压缩参数
    let (target_dpi, _jpeg_quality) = match level {
        1 => (150.0, 85u8),
        2 => (150.0, 70u8),
        _ => (72.0, 50u8),
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
                        process_page_resources(&mut doc, &resources, target_dpi)?;
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
                                let (width, height) = (
                                    sdict.get(b"Width").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0),
                                    sdict.get(b"Height").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0),
                                );
                                if width > 0 && height > 0 {
                                    let raw_data = stream.content.clone();
                                    if let Ok(img) = image::load_from_memory(&raw_data) {
                                        let new_img = resize_image(&img, width as u32, height as u32, target_dpi);
                                        let mut buf = Vec::new();
                                        if new_img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg).is_ok() {
                                            let mut new_dict = Dictionary::new();
                                            new_dict.set("Type", Object::Name(b"XObject".to_vec()));
                                            new_dict.set("Subtype", Object::Name(b"Image".to_vec()));
                                            new_dict.set("Width", Object::Integer(width));
                                            new_dict.set("Height", Object::Integer(height));
                                            new_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
                                            new_dict.set("BitsPerComponent", Object::Integer(8));
                                            new_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                                            let mut new_stream = Stream::new(new_dict, buf);
                                            let _ = new_stream.compress();
                                            *obj = Object::Stream(new_stream);
                                        }
                                    }
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

fn resize_image(img: &DynamicImage, orig_width: u32, orig_height: u32, target_dpi: f64) -> DynamicImage {
    let assumed_dpi = 300.0;
    if target_dpi >= assumed_dpi {
        return img.clone();
    }
    let scale = target_dpi / assumed_dpi;
    let new_w = (orig_width as f64 * scale) as u32;
    let new_h = (orig_height as f64 * scale) as u32;
    if new_w < 1 || new_h < 1 {
        return img.clone();
    }
    img.resize_exact(new_w, new_h, FilterType::Lanczos3)
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