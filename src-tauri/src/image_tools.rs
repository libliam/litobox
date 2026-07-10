use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use image::imageops::{self, FilterType};
use imageproc::drawing;
// rotate_about_center/Interpolation removed — no longer needed after migrating to template merge
use ab_glyph::{FontVec, PxScale, Font as _};
use std::io::Cursor;
use std::collections::HashMap;

// ============ 通用工具 ============

fn image_to_base64_png(img: &DynamicImage) -> Result<String, String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| format!("PNG 编码失败: {}", e))?;
    Ok(STANDARD.encode(&buf))
}

fn parse_color(hex: &str) -> Result<Rgba<u8>, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("颜色格式错误，需为 #RRGGBB".into());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "颜色解析失败")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "颜色解析失败")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "颜色解析失败")?;
    Ok(Rgba([r, g, b, 255]))
}

// ============ B6: 图片批量压缩/格式转换 ============

#[derive(serde::Serialize)]
pub struct FileInfo {
    pub size: u32,
    pub width: u32,
    pub height: u32,
}

/// 获取文件信息（仅大小，不解码图片，避免大文件卡死）
#[tauri::command]
pub fn get_file_info(file_path: String) -> Result<FileInfo, String> {
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| format!("获取文件信息失败: {}", e))?;
    let size = metadata.len() as u32;

    // 仅获取尺寸，不读取文件内容（避免大文件阻塞）
    // 尺寸信息在压缩时自然获取，此处不需要
    Ok(FileInfo {
        size,
        width: 0,
        height: 0,
    })
}

/// 获取图片缩略图（最大 100px，用于预览）
#[tauri::command]
pub fn get_thumbnail(file_path: String) -> Result<String, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("无法解码图片: {}", e))?;

    // 缩放到最大 100px
    let thumb = img.thumbnail(100, 100);
    let mut buf = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("缩略图编码失败: {}", e))?;
    Ok(STANDARD.encode(&buf))
}

#[derive(serde::Serialize)]
pub struct CompressResult {
    pub original_size: u32,
    pub compressed_size: u32,
    pub base64: String,
    pub format: String,
}

#[tauri::command]
pub async fn image_compress(
    file_path: String,
    quality: u8,
    format: String,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<CompressResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_compress(file_path, quality, format, max_width, max_height)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_image_compress(
    file_path: String,
    quality: u8,
    format: String,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<CompressResult, String> {
    let original_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let original_size = original_bytes.len() as u32;

    let mut img = image::load_from_memory(&original_bytes)
        .map_err(|e| format!("无法解码图片: {}", e))?;

    // 尺寸调整
    if let (Some(mw), Some(mh)) = (max_width, max_height) {
        let (w, h) = img.dimensions();
        if w > mw || h > mh {
            img = img.resize(mw, mh, FilterType::Lanczos3);
        }
    }

    // 编码为目标格式
    let img_format = match format.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::WebP,
        "png" => ImageFormat::Png,
        "bmp" => ImageFormat::Bmp,
        _ => return Err(format!("不支持的格式: {}", format)),
    };

    let mut buf = Vec::new();

    if format == "jpg" || format == "jpeg" {
        let mut cursor = Cursor::new(&mut buf);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
            .encode(img.as_bytes(), img.width(), img.height(), img.color().into())
            .map_err(|e| format!("JPEG 编码失败: {}", e))?;
    } else {
        let mut cursor = Cursor::new(&mut buf);
        img.write_to(&mut cursor, img_format)
            .map_err(|e| format!("编码失败: {}", e))?;
    }

    let compressed_size = buf.len() as u32;
    let base64 = STANDARD.encode(&buf);

    Ok(CompressResult {
        original_size,
        compressed_size,
        base64,
        format,
    })
}

// ============ F2: 图片拼接/长图合并 ============

#[derive(serde::Serialize)]
pub struct MergeResult {
    pub base64: String,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn image_merge(
    file_paths: Vec<String>,
    direction: String,
    gap: u32,
    bg_color: String,
    alignment: String,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_merge(file_paths, direction, gap, bg_color, alignment)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_image_merge(
    file_paths: Vec<String>,
    direction: String,
    gap: u32,
    bg_color: String,
    alignment: String,
) -> Result<MergeResult, String> {
    if file_paths.is_empty() {
        return Err("请至少选择一张图片".into());
    }

    let bg = parse_color(&bg_color)?;
    let mut images: Vec<DynamicImage> = Vec::new();
    for path in &file_paths {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("读取文件失败 ({}): {}", path, e))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("无法解码图片 ({}): {}", path, e))?;
        images.push(img);
    }

    let is_horizontal = direction == "horizontal";

    // 计算画布尺寸
    let max_cross = if is_horizontal {
        images.iter().map(|i| i.height()).max().unwrap_or(0)
    } else {
        images.iter().map(|i| i.width()).max().unwrap_or(0)
    };

    let total_main: u32 = images.iter()
        .map(|_| max_cross + gap)
        .sum::<u32>()
        - if images.len() > 0 { gap } else { 0 };

    let (canvas_w, canvas_h) = if is_horizontal {
        (total_main, max_cross)
    } else {
        (max_cross, total_main)
    };

    let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, bg);

    let mut offset = 0u32;
    for img in &images {
        let (iw, ih) = img.dimensions();
        let (x, y) = if is_horizontal {
            let cross_pos = match alignment.as_str() {
                "center" => (max_cross.saturating_sub(ih)) / 2,
                "end" => max_cross.saturating_sub(ih),
                _ => 0,
            };
            (offset, cross_pos)
        } else {
            let cross_pos = match alignment.as_str() {
                "center" => (max_cross.saturating_sub(iw)) / 2,
                "end" => max_cross.saturating_sub(iw),
                _ => 0,
            };
            (cross_pos, offset)
        };

        imageops::overlay(&mut canvas, img, x as i64, y as i64);
        offset += if is_horizontal { iw } else { ih } + gap;
    }

    let merged = DynamicImage::ImageRgba8(canvas);
    let base64 = image_to_base64_png(&merged)?;

    Ok(MergeResult {
        base64,
        width: canvas_w,
        height: canvas_h,
    })
}

// ============ 模板拼图（槽位坐标渲染） ============

#[derive(serde::Deserialize)]
pub struct MergeSlotInput {
    pub file_path: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn image_template_merge(
    images: Vec<MergeSlotInput>,
    canvas_width: u32,
    canvas_height: u32,
    bg_color: String,
    gap: u32,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_template_merge(images, canvas_width, canvas_height, bg_color, gap)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_image_template_merge(
    images: Vec<MergeSlotInput>,
    canvas_width: u32,
    canvas_height: u32,
    bg_color: String,
    _gap: u32,  // ponytail: 间距由前端计算好坐标传入，此参数保留供未来扩展
) -> Result<MergeResult, String> {
    if images.is_empty() {
        return Err("请至少选择一张图片".into());
    }

    let is_transparent = bg_color.is_empty() || bg_color == "transparent";
    let bg = if is_transparent {
        Rgba([0, 0, 0, 0])
    } else {
        parse_color(&bg_color)?
    };

    let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, bg);

    for img_input in &images {
        let bytes = std::fs::read(&img_input.file_path)
            .map_err(|e| format!("读取文件失败 ({}): {}", img_input.file_path, e))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("无法解码图片 ({}): {}", img_input.file_path, e))?;

        // cover 模式：等比缩放填满槽位，居中裁剪
        let slot_w = img_input.width;
        let slot_h = img_input.height;
        let img_w = img.width();
        let img_h = img.height();

        let scale = (slot_w as f64 / img_w as f64).max(slot_h as f64 / img_h as f64);
        let scaled_w = (img_w as f64 * scale) as u32;
        let scaled_h = (img_h as f64 * scale) as u32;

        let resized = img.resize(scaled_w, scaled_h, FilterType::Lanczos3);

        // 居中裁剪到槽位尺寸
        let crop_x = (scaled_w.saturating_sub(slot_w) / 2) as u32;
        let crop_y = (scaled_h.saturating_sub(slot_h) / 2) as u32;
        let cropped = resized.crop_imm(crop_x, crop_y, slot_w, slot_h);

        imageops::overlay(&mut canvas, &cropped, img_input.x as i64, img_input.y as i64);
    }

    let merged = DynamicImage::ImageRgba8(canvas);
    let base64 = image_to_base64_png(&merged)?;

    Ok(MergeResult {
        base64,
        width: canvas_width,
        height: canvas_height,
    })
}

// ============ F3: 图片加水印 ============

#[derive(serde::Serialize)]
pub struct WatermarkResult {
    pub base64: String,
}

fn load_system_font() -> Result<FontVec, String> {
    // 尝试多个 Windows 系统字体路径
    let font_paths = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyhbd.ttc",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\calibri.ttf",
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(font) = FontVec::try_from_vec(data) {
                return Ok(font);
            }
        }
    }
    Err("未找到系统字体，请确保系统安装了中文字体".into())
}

#[tauri::command]
pub async fn image_watermark(
    file_path: String,
    text: String,
    position: String,
    opacity: f32,
    font_size: f32,
    color: String,
) -> Result<WatermarkResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_watermark(file_path, text, position, opacity, font_size, color)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_image_watermark(
    file_path: String,
    text: String,
    position: String,
    opacity: f32,
    font_size: f32,
    color: String,
) -> Result<WatermarkResult, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let mut img = image::load_from_memory(&bytes)
        .map_err(|e| format!("无法解码图片: {}", e))?;
    let (w, h) = img.dimensions();

    let text_color = parse_color(&color)?;
    let font = load_system_font()?;

    let scale = PxScale::from(font_size);
    let text_color_alpha = Rgba([
        text_color[0],
        text_color[1],
        text_color[2],
        (opacity * 255.0) as u8,
    ]);

    // 测量文字尺寸
    let scale_factor = scale.y / font.units_per_em().unwrap_or(2048.0) as f32;
    let text_w = text.chars().map(|c| {
        let glyph_id = font.glyph_id(c);
        font.h_advance_unscaled(glyph_id) * scale_factor
    }).sum::<f32>() as u32;
    let text_h = ((font.ascent_unscaled() - font.descent_unscaled()) * scale_factor) as u32;

    let padding = 20u32;
    let (x, y) = match position.as_str() {
        "topLeft" => (padding as i32, padding as i32 + text_h as i32 / 2),
        "topRight" => (w.saturating_sub(text_w + padding) as i32, padding as i32 + text_h as i32 / 2),
        "bottomLeft" => (padding as i32, h.saturating_sub(padding + text_h / 2) as i32),
        "bottomRight" => (w.saturating_sub(text_w + padding) as i32, h.saturating_sub(padding + text_h / 2) as i32),
        "center" | _ => ((w.saturating_sub(text_w)) as i32 / 2, (h.saturating_sub(text_h)) as i32 / 2),
    };

    drawing::draw_text_mut(&mut img, text_color_alpha, x, y, scale, &font, &text);

    let base64 = image_to_base64_png(&DynamicImage::ImageRgba8(img.to_rgba8()))?;
    Ok(WatermarkResult { base64 })
}

// ============ F5: 图片调色板提取 ============

#[derive(serde::Serialize, Clone)]
pub struct PaletteColor {
    pub hex: String,
    pub rgb: [u8; 3],
    pub ratio: f32,
}

#[derive(serde::Serialize)]
pub struct PaletteResult {
    pub colors: Vec<PaletteColor>,
}

/// 简单的中位切分算法提取调色板
fn median_cut_palette(img: &DynamicImage, count: u32) -> Vec<PaletteColor> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let total = (w * h) as f32;

    // 收集所有颜色
    let mut color_counts: HashMap<[u8; 3], u32> = HashMap::new();
    for y in 0..h {
        for x in 0..w {
            let p = rgba.get_pixel(x, y);
            if p[3] < 128 { continue; } // 跳过透明像素
            let rgb = [p[0], p[1], p[2]];
            *color_counts.entry(rgb).or_insert(0) += 1;
        }
    }

    if color_counts.is_empty() {
        return vec![];
    }

    // 降采样：将颜色空间量化到 32 级以减少计算量
    let mut buckets: Vec<(Vec<[u8; 3]>, u32)> = Vec::new();
    let mut simplified: HashMap<[u8; 3], u32> = HashMap::new();
    for (rgb, cnt) in &color_counts {
        let simple = [
            (rgb[0] / 8) * 8 + 4,
            (rgb[1] / 8) * 8 + 4,
            (rgb[2] / 8) * 8 + 4,
        ];
        *simplified.entry(simple).or_insert(0) += cnt;
    }

    let all_colors: Vec<[u8; 3]> = simplified.keys().copied().collect();
    let all_counts: Vec<u32> = all_colors.iter().map(|c| simplified[c]).collect();
    let total_simple: u32 = all_counts.iter().sum();

    buckets.push((all_colors, total_simple));

    // 中位切分
    while buckets.len() < count as usize && buckets.len() < color_counts.len() {
        // 找最大的桶
        let mut max_idx = 0;
        let mut max_size = 0u32;
        for (i, (_, s)) in buckets.iter().enumerate() {
            if *s > max_size {
                max_size = *s;
                max_idx = i;
            }
        }
        if max_size == 0 { break; }

        let (colors, total) = buckets.remove(max_idx);
        if colors.len() <= 1 {
            buckets.push((colors, total));
            continue;
        }

        // 找范围最大的通道
        let mut min_r = 255u8; let mut max_r = 0u8;
        let mut min_g = 255u8; let mut max_g = 0u8;
        let mut min_b = 255u8; let mut max_b = 0u8;
        for c in &colors {
            min_r = min_r.min(c[0]); max_r = max_r.max(c[0]);
            min_g = min_g.min(c[1]); max_g = max_g.max(c[1]);
            min_b = min_b.min(c[2]); max_b = max_b.max(c[2]);
        }
        let r_range = max_r - min_r;
        let g_range = max_g - min_g;
        let b_range = max_b - min_b;

        let channel: usize = if r_range >= g_range && r_range >= b_range { 0 }
            else if g_range >= b_range { 1 }
            else { 2 };

        // 按该通道排序
        let mut sorted: Vec<([u8; 3], u32)> = colors.iter().map(|c| (*c, simplified[c])).collect();
        sorted.sort_by_key(|(c, _)| c[channel]);

        // 中位切分
        let mid = sorted.iter().map(|(_, cnt)| *cnt).sum::<u32>() / 2;
        let mut sum = 0u32;
        let mut split = 0;
        for (i, (_, cnt)) in sorted.iter().enumerate() {
            sum += cnt;
            if sum >= mid {
                split = i;
                break;
            }
        }
        split = split.max(1).min(sorted.len() - 1);

        let mut left_colors = Vec::new();
        let mut left_total = 0u32;
        let mut right_colors = Vec::new();
        let mut right_total = 0u32;

        for (c, cnt) in &sorted[..split] {
            left_colors.push(*c);
            left_total += cnt;
        }
        for (c, cnt) in &sorted[split..] {
            right_colors.push(*c);
            right_total += cnt;
        }

        if !left_colors.is_empty() {
            buckets.push((left_colors, left_total));
        }
        if !right_colors.is_empty() {
            buckets.push((right_colors, right_total));
        }
    }

    // 计算每个桶的平均颜色
    let mut result: Vec<PaletteColor> = Vec::new();
    for (colors, cnt) in &buckets {
        if colors.is_empty() { continue; }
        let mut r = 0u64; let mut g = 0u64; let mut b = 0u64;
        let mut total_weight = 0u64;
        for c in colors {
            let weight = simplified[c] as u64;
            r += c[0] as u64 * weight;
            g += c[1] as u64 * weight;
            b += c[2] as u64 * weight;
            total_weight += weight;
        }
        if total_weight == 0 { continue; }
        let rgb = [
            (r / total_weight) as u8,
            (g / total_weight) as u8,
            (b / total_weight) as u8,
        ];
        let hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        result.push(PaletteColor {
            hex,
            rgb,
            ratio: *cnt as f32 / total,
        });
    }

    // 按占比降序排列
    result.sort_by(|a, b| b.ratio.partial_cmp(&a.ratio).unwrap_or(std::cmp::Ordering::Equal));
    result.truncate(count as usize);
    result
}

#[tauri::command]
pub async fn image_palette(
    file_path: String,
    color_count: u32,
) -> Result<PaletteResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_palette(file_path, color_count)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_image_palette(
    file_path: String,
    color_count: u32,
) -> Result<PaletteResult, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("无法解码图片: {}", e))?;

    let colors = median_cut_palette(&img, color_count.clamp(2, 32));
    Ok(PaletteResult { colors })
}