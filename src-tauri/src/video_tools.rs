use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::os::windows::process::CommandExt;
use tauri::Emitter;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub fps: f64,
    pub bitrate: u32,
    pub file_size: u64,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailResult {
    pub images: Vec<String>,
    pub timestamps: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCropOptions {
    pub start_time: f64,
    pub end_time: f64,
    pub use_ffmpeg: bool,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
    pub actual_start: Option<f64>,
    pub actual_end: Option<f64>,
}

// ============ 格式识别 + 工具检测 ============

fn guess_video_format(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".mp4") || lower.ends_with(".m4v") {
        "mp4".to_string()
    } else if lower.ends_with(".mkv") {
        "mkv".to_string()
    } else if lower.ends_with(".avi") {
        "avi".to_string()
    } else if lower.ends_with(".mov") {
        "mov".to_string()
    } else if lower.ends_with(".webm") {
        "webm".to_string()
    } else {
        "unknown".to_string()
    }
}

fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ffprobe_available() -> bool {
    std::process::Command::new("ffprobe")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ============ 纯 Rust 视频元信息读取（mp4 crate） ============

fn get_video_info_mp4(path: &str) -> Result<VideoInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();

    let f = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let buf_reader = std::io::BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(buf_reader, file_size)
        .map_err(|e| format!("MP4 解析失败: {}", e))?;

    let duration = mp4.duration().as_secs_f64();

    // 获取视频轨道信息
    let video_track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    let width = video_track.width() as u32;
    let height = video_track.height() as u32;

    // 编码格式
    let codec = match video_track.media_type() {
        Ok(mp4::MediaType::H264) => "h264".to_string(),
        Ok(mp4::MediaType::H265) => "h265".to_string(),
        Ok(mp4::MediaType::VP9) => "vp9".to_string(),
        _ => "unknown".to_string(),
    };

    // 帧率（从 timescale 和 default_sample_duration 计算）
    let timescale = video_track.timescale() as f64;
    let default_sample_duration = video_track.default_sample_duration as f64;
    let fps = if default_sample_duration > 0.0 {
        timescale / default_sample_duration
    } else {
        // fallback: 使用 track 自带的 frame_rate 方法
        video_track.frame_rate()
    };

    let bitrate = if duration > 0.0 {
        ((file_size as f64 * 8.0) / duration / 1000.0) as u32
    } else {
        0
    };

    let format = guess_video_format(path);

    Ok(VideoInfo { duration, width, height, codec, fps, bitrate, file_size, format })
}

// ============ ffprobe 视频元信息读取 ============

fn get_video_info_ffprobe(path: &str) -> Result<VideoInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();
    let format = guess_video_format(path);

    let output = std::process::Command::new("ffprobe")
        .args(&["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", path])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffprobe 执行失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("ffprobe 输出解析失败: {}", e))?;

    let format_info = &json["format"];
    let duration = format_info["duration"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let bitrate = format_info["bit_rate"].as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .map(|b| b / 1000)
        .unwrap_or(0);

    // 找视频流
    let streams = json["streams"].as_array()
        .ok_or("ffprobe 输出无 streams")?;
    let video_stream = streams.iter()
        .find(|s| s["codec_type"].as_str() == Some("video"))
        .ok_or("未找到视频流")?;

    let width = video_stream["width"].as_u64().unwrap_or(0) as u32;
    let height = video_stream["height"].as_u64().unwrap_or(0) as u32;
    let codec = video_stream["codec_name"].as_str().unwrap_or("unknown").to_string();

    // 帧率：优先 r_frame_rate，其次 avg_frame_rate
    let fps_str = video_stream["r_frame_rate"].as_str()
        .or_else(|| video_stream["avg_frame_rate"].as_str())
        .unwrap_or("0/1");
    let fps = if let Some((num, den)) = fps_str.split_once('/') {
        let n = num.parse::<f64>().unwrap_or(0.0);
        let d = den.parse::<f64>().unwrap_or(1.0);
        if d > 0.0 { n / d } else { 0.0 }
    } else {
        0.0
    };

    Ok(VideoInfo { duration, width, height, codec, fps, bitrate, file_size, format })
}

// ============ 缩略图提取（ffmpeg） ============

fn extract_thumbnails_ffmpeg(path: &str, count: u32, max_width: u32) -> Result<ThumbnailResult, String> {
    // 先用 ffprobe 获取时长
    let duration = get_video_info_ffprobe(path)?.duration;
    if duration <= 0.0 {
        return Err("无法获取视频时长".to_string());
    }

    let temp_dir = std::env::temp_dir().join("litobox_video_thumbnails");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let mut images: Vec<String> = Vec::with_capacity(count as usize);
    let mut timestamps: Vec<f64> = Vec::with_capacity(count as usize);

    for i in 0..count {
        let ts = duration * (i as f64 + 0.5) / count as f64; // 取每段中间位置
        let out_path = temp_dir.join(format!("thumb_{}.jpg", i));

        let status = std::process::Command::new("ffmpeg")
            .args(&[
                "-y",
                "-ss", &format!("{:.3}", ts),
                "-i", path,
                "-vframes", "1",
                "-vf", &format!("scale={}:-1", max_width),
                &out_path.to_string_lossy(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("ffmpeg 缩略图提取失败: {}", e))?;

        if status.success() {
            let data = std::fs::read(&out_path).unwrap_or_default();
            images.push(BASE64.encode(&data));
            timestamps.push(ts);
        }
        // 忽略单个缩略图失败，继续处理下一张
        let _ = std::fs::remove_file(&out_path);
    }

    // 清理临时目录
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(ThumbnailResult { images, timestamps })
}

// ============ 纯 Rust 关键帧裁剪 ============

fn do_video_crop_keyframe(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &VideoCropOptions,
) -> Result<CropResult, String> {
    if options.start_time < 0.0 || options.end_time <= options.start_time {
        return Err("起止时间非法".to_string());
    }
    let duration = options.end_time - options.start_time;
    if duration < 0.1 {
        return Err("裁剪区间不能小于 0.1 秒".to_string());
    }

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 5.0 }));

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();
    let f = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let buf_reader = std::io::BufReader::new(f);

    let mp4 = mp4::Mp4Reader::read_header(buf_reader, file_size)
        .map_err(|e| format!("MP4 解析失败: {}", e))?;

    let video_track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    let timescale = video_track.timescale() as f64;
    let total_duration = mp4.duration().as_secs_f64();

    // 获取关键帧索引（stss）
    let keyframes: Vec<u32> = video_track.trak.mdia.minf.stbl.stss
        .as_ref()
        .map(|stss| {
            let mut samples: Vec<u32> = stss.entries.clone();
            samples.sort();
            samples
        })
        .unwrap_or_default();

    if keyframes.is_empty() {
        return Err("未找到关键帧索引，无法进行无损裁剪。请安装 ffmpeg 以获得完整支持".to_string());
    }

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 15.0 }));

    // 将时间转为 sample 编号
    let sample_count = video_track.sample_count();
    let default_sample_dur = video_track.default_sample_duration as f64;
    let sample_duration = if default_sample_dur > 0.0 {
        default_sample_dur / timescale
    } else {
        total_duration / sample_count.max(1) as f64
    };

    let start_sample = ((options.start_time / sample_duration) as u32).min(sample_count);
    let end_sample = ((options.end_time / sample_duration) as u32).min(sample_count);

    // 找到起止时间对应的最近关键帧（sample 编号从 1 开始）
    let actual_start_sample = keyframes.iter()
        .filter(|&&k| k <= start_sample + 1)
        .last()
        .copied()
        .unwrap_or(keyframes[0]);

    let actual_end_sample = keyframes.iter()
        .filter(|&&k| k >= end_sample)
        .next()
        .copied()
        .unwrap_or(*keyframes.last().unwrap());

    if actual_start_sample >= actual_end_sample {
        return Err("裁剪区间内无关键帧，请扩大区间或使用 ffmpeg".to_string());
    }

    let actual_start = actual_start_sample as f64 * sample_duration;
    let actual_end = actual_end_sample as f64 * sample_duration;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 30.0 }));

    // 确定输出路径
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_cropped.mp4", input_stem))
    };

    // 读取源文件数据
    let src_data = std::fs::read(path)
        .map_err(|e| format!("读取源文件失败: {}", e))?;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 50.0 }));

    // 重新解析以获取 sample 偏移信息
    let cursor = std::io::Cursor::new(&src_data);
    let buf_reader = std::io::BufReader::new(cursor);
    let mp4 = mp4::Mp4Reader::read_header(buf_reader, src_data.len() as u64)
        .map_err(|e| format!("二次解析 MP4 失败: {}", e))?;

    let track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    // 获取 sample 大小表
    let stsz = &track.trak.mdia.minf.stbl.stsz;
    let sample_sizes: Vec<u32> = if stsz.sample_size > 0 {
        vec![stsz.sample_size; stsz.sample_count as usize]
    } else {
        stsz.sample_sizes.clone()
    };

    // 构建 sample → byte offset 映射
    let sample_offsets = build_sample_offsets(track, &sample_sizes)?;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 70.0 }));

    // 收集裁剪范围内所有 sample 的数据
    // sample 编号从 1 开始，数组索引从 0 开始
    let start_idx = (actual_start_sample as usize).saturating_sub(1);
    let end_idx = (actual_end_sample as usize).saturating_sub(1).min(sample_offsets.len().saturating_sub(1));

    let start_offset = sample_offsets.get(start_idx).copied().unwrap_or(0);
    let end_offset = if end_idx + 1 < sample_offsets.len() {
        sample_offsets[end_idx + 1]
    } else {
        src_data.len() as u64
    };

    // 复制 mdat 前的内容（ftyp + moov 等头部）+ 裁剪后的 mdat 数据
    let mdat_start = sample_offsets.first().copied().unwrap_or(0);
    let head_data = &src_data[..mdat_start as usize];
    let clip_data = &src_data[start_offset as usize..end_offset as usize];

    let mut output_data = Vec::with_capacity(head_data.len() + clip_data.len());
    output_data.extend_from_slice(head_data);
    output_data.extend_from_slice(clip_data);

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 90.0 }));

    std::fs::write(&output_path, &output_data)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let output_size = output_data.len() as u64;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration: actual_end - actual_start,
        actual_start: Some(actual_start),
        actual_end: Some(actual_end),
    })
}

/// 构建 sample → byte offset 映射表
fn build_sample_offsets(track: &mp4::Mp4Track, sample_sizes: &[u32]) -> Result<Vec<u64>, String> {
    let stsc = &track.trak.mdia.minf.stbl.stsc;
    let stco = track.trak.mdia.minf.stbl.stco.as_ref()
        .ok_or("未找到 stco box")?;

    let sample_count = sample_sizes.len();

    // 构建 chunk → (offset, samples_per_chunk) 映射
    let mut chunk_info: Vec<(u64, u32)> = Vec::new();
    let mut stsc_idx = 0usize;

    for chunk_id in 1u32.. {
        // 找到当前 chunk 对应的 stsc entry
        while stsc_idx + 1 < stsc.entries.len()
            && stsc.entries[stsc_idx + 1].first_chunk <= chunk_id
        {
            stsc_idx += 1;
        }
        let spc = stsc.entries[stsc_idx].samples_per_chunk;

        let offset = stco.entries.get(chunk_id as usize - 1).copied();
        match offset {
            Some(off) => chunk_info.push((off as u64, spc)),
            None => break,
        }
    }

    // 构建 sample → offset 映射
    let mut sample_offsets: Vec<u64> = Vec::with_capacity(sample_count);
    let mut sample_idx = 0usize;

    for (chunk_offset, spc) in &chunk_info {
        let mut offset_in_chunk = 0u64;
        for _ in 0..*spc {
            if sample_idx < sample_count {
                sample_offsets.push(chunk_offset + offset_in_chunk);
                offset_in_chunk += sample_sizes[sample_idx] as u64;
                sample_idx += 1;
            } else {
                break;
            }
        }
    }

    Ok(sample_offsets)
}

// ============ ffmpeg 裁剪 ============

fn do_video_crop_ffmpeg(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &VideoCropOptions,
) -> Result<CropResult, String> {
    let duration = options.end_time - options.start_time;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_cropped.mp4", input_stem))
    };

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 5.0 }));

    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-ss", &format!("{:.3}", options.start_time),
            "-t", &format!("{:.3}", duration),
            "-i", path,
            "-c", "copy",
            &output_path.to_string_lossy(),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let preview: String = stderr.chars().take(200).collect();
        return Err(format!("ffmpeg 裁剪失败: {}", preview));
    }

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
        actual_start: None,
        actual_end: None,
    })
}

// ============ Tauri Commands ============

#[tauri::command]
pub async fn get_video_info(path: String, use_ffmpeg: bool) -> Result<VideoInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if use_ffmpeg && ffprobe_available() {
            get_video_info_ffprobe(&path)
        } else {
            let format = guess_video_format(&path);
            if format != "mp4" {
                return Err("仅支持 MP4 格式，安装 ffmpeg 可支持更多格式".to_string());
            }
            get_video_info_mp4(&path)
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn extract_thumbnails(path: String, count: u32) -> Result<ThumbnailResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            // 无 ffmpeg 时返回空结果，前端降级为纯文本时间轴
            return Ok(ThumbnailResult { images: vec![], timestamps: vec![] });
        }
        extract_thumbnails_ffmpeg(&path, count, 160)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

#[tauri::command]
pub async fn video_crop(
    app_handle: tauri::AppHandle,
    path: String,
    options: VideoCropOptions,
) -> Result<CropResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if options.use_ffmpeg && ffmpeg_available() {
            do_video_crop_ffmpeg(&app_handle, &path, &options)
        } else {
            do_video_crop_keyframe(&app_handle, &path, &options)
        }
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}