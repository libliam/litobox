use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::os::windows::process::CommandExt;
use tauri::Emitter;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

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

// ponytail: 测试 exe 是否可以执行（带 -version 参数）
fn test_exe(path: &str) -> bool {
    std::process::Command::new(path)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ponytail: dev 模式下 PATH 可能不完整，搜索常见安装路径作为回退
// 找到后把目录加入进程 PATH，后续所有 Command::new("ffmpeg") 自动生效
use std::sync::OnceLock;
static FFMPEG_PATH_INITIALIZED: OnceLock<bool> = OnceLock::new();

/// 递归搜索目录下 depth 层内的 ffmpeg.exe，返回 bin 目录路径
fn search_ffmpeg_in_dir(base: &std::path::Path, depth: u32) -> Option<String> {
    if depth == 0 { return None; }
    let exe = base.join("ffmpeg.exe");
    if exe.exists() {
        return Some(base.to_string_lossy().to_string());
    }
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(found) = search_ffmpeg_in_dir(&p, depth - 1) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub(crate) fn ensure_ffmpeg_in_path() {
    FFMPEG_PATH_INITIALIZED.get_or_init(|| {
        debug_log!("[ffmpeg] 开始检测 ffmpeg...");
        if test_exe("ffmpeg") {
            debug_log!("[ffmpeg] PATH 中找到 ffmpeg");
            return true;
        }
        debug_log!("[ffmpeg] PATH 中未找到，开始搜索常见安装路径...");

        let mut search_dirs: Vec<String> = Vec::new();

        // 优先检查当前 exe 所在目录（用户可将 ffmpeg 放在 litobox 同目录）
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let ffmpeg_exe = exe_dir.join("ffmpeg.exe");
                debug_log!("[ffmpeg] 检查当前目录: {}", ffmpeg_exe.display());
                if ffmpeg_exe.exists() {
                    search_dirs.push(exe_dir.to_string_lossy().to_string());
                    debug_log!("[ffmpeg] 当前目录找到 ffmpeg");
                }
            }
        }

        search_dirs.push(r"C:\ffmpeg\bin".to_string());
        search_dirs.push(r"C:\Program Files\ffmpeg\bin".to_string());

        // winget 安装路径: %LOCALAPPDATA%\Microsoft\WinGet\Packages\*
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let winget_base = std::path::PathBuf::from(&local)
                .join("Microsoft").join("WinGet").join("Packages");
            debug_log!("[ffmpeg] 搜索 winget 目录: {}", winget_base.display());
            if winget_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&winget_base) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if !p.is_dir() { continue; }
                        debug_log!("[ffmpeg] 搜索 winget 包: {}", p.display());
                        // 递归搜索包目录下最多 3 层，适配 ffmpeg-X.Y.Z-full_build\bin 结构
                        if let Some(bin_dir) = search_ffmpeg_in_dir(&p, 3) {
                            debug_log!("[ffmpeg] winget 包中找到: {}", bin_dir);
                            search_dirs.push(bin_dir);
                        }
                    }
                }
            }
        }

        // scoop shims
        if let Ok(home) = std::env::var("USERPROFILE") {
            let scoop = std::path::PathBuf::from(&home).join("scoop").join("shims");
            if scoop.join("ffmpeg.exe").exists() {
                search_dirs.push(scoop.to_string_lossy().to_string());
            }
        }

        debug_log!("[ffmpeg] 搜索目录列表: {:?}", search_dirs);

        for dir in &search_dirs {
            let exe = std::path::PathBuf::from(dir).join("ffmpeg.exe");
            debug_log!("[ffmpeg] 尝试: {}", exe.display());
            if test_exe(&exe.to_string_lossy()) {
                let current = std::env::var("PATH").unwrap_or_default();
                std::env::set_var("PATH", format!("{};{}", dir, current));
                debug_log!("[ffmpeg] 成功! 已将 {} 添加到 PATH", dir);
                return true;
            }
        }
        debug_log!("[ffmpeg] 未找到 ffmpeg");
        false
    });
}

fn ffmpeg_available() -> bool {
    ensure_ffmpeg_in_path();
    test_exe("ffmpeg")
}

fn ffprobe_available() -> bool {
    ensure_ffmpeg_in_path();
    test_exe("ffprobe")
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
// ponytail: 使用 Mp4Writer 重建 MP4 容器，让 crate 自动构建正确的 moov/stts/stsz/stco 等表
// 之前的"头部复制 + mdat 裁剪"策略不重写 moov 元数据，导致播放器无法正确解析

/// 获取 sample 在文件中的字节偏移（sample_id 从 1 开始）
fn get_sample_offset(track: &mp4::Mp4Track, sample_id: u32) -> Result<u64, String> {
    let stsc = &track.trak.mdia.minf.stbl.stsc;
    let stco = track.trak.mdia.minf.stbl.stco.as_ref()
        .ok_or("未找到 stco box")?;

    // 找到 sample_id 所在的 stsc entry
    let mut stsc_idx = 0usize;
    for (i, entry) in stsc.entries.iter().enumerate() {
        if sample_id >= entry.first_sample {
            stsc_idx = i;
        }
    }
    let stsc_entry = &stsc.entries[stsc_idx];
    let samples_per_chunk = stsc_entry.samples_per_chunk;
    let first_sample = stsc_entry.first_sample;
    let first_chunk = stsc_entry.first_chunk;

    let chunk_id = first_chunk + (sample_id - first_sample) / samples_per_chunk;
    let chunk_offset = stco.entries.get(chunk_id as usize - 1).copied()
        .ok_or("chunk offset 超出范围")? as u64;

    let first_sample_in_chunk = sample_id - (sample_id - first_sample) % samples_per_chunk;
    let mut offset_in_chunk: u64 = 0;
    let stsz = &track.trak.mdia.minf.stbl.stsz;
    for i in first_sample_in_chunk..sample_id {
        let size = if stsz.sample_size > 0 {
            stsz.sample_size
        } else {
            *stsz.sample_sizes.get(i as usize - 1).ok_or("sample size 超出范围")?
        };
        offset_in_chunk += size as u64;
    }

    Ok(chunk_offset + offset_in_chunk)
}

/// 获取 sample 大小（sample_id 从 1 开始）
fn get_sample_size(track: &mp4::Mp4Track, sample_id: u32) -> Result<u32, String> {
    let stsz = &track.trak.mdia.minf.stbl.stsz;
    if stsz.sample_size > 0 {
        Ok(stsz.sample_size)
    } else {
        Ok(*stsz.sample_sizes.get(sample_id as usize - 1).ok_or("sample size 超出范围")?)
    }
}

/// 获取 sample 时长（stts 表，sample_id 从 1 开始）
fn get_sample_duration(track: &mp4::Mp4Track, sample_id: u32) -> u32 {
    let stts = &track.trak.mdia.minf.stbl.stts;
    let mut count: u32 = 1;
    for entry in &stts.entries {
        if sample_id < count + entry.sample_count {
            return entry.sample_delta;
        }
        count += entry.sample_count;
    }
    // fallback
    track.default_sample_duration
}

/// 判断 sample 是否为关键帧（sample_id 从 1 开始）
fn is_sync_sample(track: &mp4::Mp4Track, sample_id: u32) -> bool {
    if let Some(ref stss) = track.trak.mdia.minf.stbl.stss {
        stss.entries.binary_search(&sample_id).is_ok()
    } else {
        true // 没有 stss 表，所有 sample 都是关键帧
    }
}

/// 根据时间戳计算 sample 范围（返回 start_sample 和 end_sample，sample_id 从 1 开始）
fn get_sample_range_by_time(track: &mp4::Mp4Track, start_time_secs: f64, end_time_secs: f64) -> (u32, u32) {
    let stts = &track.trak.mdia.minf.stbl.stts;
    let timescale = track.timescale() as f64;
    let sample_count = track.sample_count();
    
    let start_time_ticks = (start_time_secs * timescale) as u64;
    let end_time_ticks = (end_time_secs * timescale) as u64;
    
    let mut current_time: u64 = 0;
    let mut sample_id: u32 = 1;
    let mut start_sample: u32 = 1;
    let mut end_sample: u32 = sample_count;
    
    for entry in &stts.entries {
        let entry_duration = entry.sample_delta as u64;
        let entry_samples = entry.sample_count;
        
        for _ in 0..entry_samples {
            let sample_start_time = current_time;
            let sample_end_time = current_time + entry_duration;
            
            if sample_start_time <= start_time_ticks && start_time_ticks < sample_end_time {
                start_sample = sample_id;
            }
            
            if sample_start_time <= end_time_ticks && end_time_ticks <= sample_end_time {
                end_sample = sample_id;
                return (start_sample, end_sample);
            }
            
            current_time = sample_end_time;
            sample_id += 1;
        }
    }
    
    (start_sample, end_sample)
}

/// 调整所有包含指定位置的父盒的 size 字段
/// 从 pos 位置向前搜索，找到所有包含该位置的父盒（mp4a, stsd, stbl, minf, mdia, trak, moov），
/// 并调整它们的 size 字段增加 size_diff
fn adjust_parent_box_sizes(data: &mut Vec<u8>, pos: usize, size_diff: i64) {
    // 需要调整的父盒签名（按嵌套顺序，从内到外）
    // mp4a 必须包含在内，因为它直接包含 ESDS
    let parent_signatures: &[&[u8; 4]] = &[
        b"mp4a", b"stsd", b"stbl", b"minf", b"mdia", b"trak", b"moov"
    ];
    
    let mut current_pos = pos;
    
    // 从内向外逐层调整父盒
    for sig in parent_signatures {
        // 从 current_pos 向前搜索该签名
        if let Some(sig_pos) = data[..current_pos].windows(4).rposition(|w| w == *sig) {
            // 验证这是有效的盒（size 字段在 sig_pos - 4）
            if sig_pos >= 4 {
                let box_size_offset = sig_pos - 4;
                let old_size = u32::from_be_bytes([
                    data[box_size_offset],
                    data[box_size_offset + 1],
                    data[box_size_offset + 2],
                    data[box_size_offset + 3],
                ]);
                
                let new_size = (old_size as i64 + size_diff) as u32;
                let new_size_bytes = new_size.to_be_bytes();
                
                data[box_size_offset] = new_size_bytes[0];
                data[box_size_offset + 1] = new_size_bytes[1];
                data[box_size_offset + 2] = new_size_bytes[2];
                data[box_size_offset + 3] = new_size_bytes[3];
                
                debug_log!("调整父盒 {} size: {} → {} (offset={})", 
                    String::from_utf8_lossy(*sig), old_size, new_size, box_size_offset);
                
                // 更新 current_pos 为该父盒的开头，继续向上找
                current_pos = box_size_offset;
            }
        } else {
            debug_log!("未找到父盒 {}", String::from_utf8_lossy(*sig));
            break; // 找不到该父盒，停止向上搜索
        }
    }
}

/// 在 ESDS 盒内精准修复 SLConfigDescriptor: size=0x00→0x01, data=0x00→0x02
/// 只搜索 ESDS 盒内部（data 数组从 esds_pos 开始，长度为 out_box_size），避免误匹配 mdat 数据
fn fix_esds_slconfig(data: &mut [u8], esds_pos: usize, out_box_size: usize) {
    let esds_start = esds_pos + 8; // 跳过 8 字节 box header（4 size + 4 type）
    let esds_end = (esds_pos + out_box_size).min(data.len());
    let esds_data = &data[esds_start..esds_end];

    debug_log!("ESDS 修复: esds_start={}, esds_end={}, esds_data.len()={}", esds_start, esds_end, esds_data.len());
    debug_log!("ESDS 修复前字节: {:02X?}", esds_data);

    // 在 ESDS 盒数据内搜索 SLConfigDescriptor: tag=0x06, size=0x00, data=0x00
    // 字节序列: 06 00 00 → 06 01 02
    if let Some(rel_pos) = esds_data.windows(3).position(|w| w == [0x06, 0x00, 0x00]) {
        let abs_pos = esds_start + rel_pos;
        data[abs_pos] = 0x06;
        data[abs_pos + 1] = 0x01;
        data[abs_pos + 2] = 0x02;
        debug_log!("SLConfigDescriptor 修复成功: rel_pos={}, abs_pos={}", rel_pos, abs_pos);
        debug_log!("ESDS 修复后字节: {:02X?}", &data[esds_start..esds_end]);
    } else {
        debug_log!("SLConfigDescriptor 未找到 06 00 00 序列");
    }
}

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

    // 读取源文件并解析
    let file_size = std::fs::metadata(path).map(|m| m.len()).map_err(|e| format!("无法读取文件: {}", e))?;
    let f = std::fs::File::open(path).map_err(|e| format!("无法打开文件: {}", e))?;
    let buf_reader = std::io::BufReader::new(f);
    let mp4 = mp4::Mp4Reader::read_header(buf_reader, file_size)
        .map_err(|e| format!("MP4 解析失败: {}", e))?;

    let video_track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    let timescale = video_track.timescale();
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
    let sample_duration_secs = if default_sample_dur > 0.0 {
        default_sample_dur / timescale as f64
    } else {
        total_duration / sample_count.max(1) as f64
    };

    let start_sample = ((options.start_time / sample_duration_secs) as u32).min(sample_count);
    let end_sample = ((options.end_time / sample_duration_secs) as u32).min(sample_count);

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

    let actual_start_secs = actual_start_sample as f64 * sample_duration_secs;
    let actual_end_secs = actual_end_sample as f64 * sample_duration_secs;

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

    // ponytail: 用 Cursor<Vec<u8>> 缓冲输出，写入完成后用源文件的原始 ESDS 盒替换
    // mp4 crate 的 Mp4Writer 重建 ESDS 盒时使用了硬编码默认值（SLConfigDescriptor size=0、
    // object_type_indication=0x40 等），与源文件不完全一致，导致 Windows Media Player 无法识别 AAC 音频
    let mut buf = std::io::Cursor::new(Vec::new());

    // 读取源文件的原始 ESDS 盒字节，用于后续替换
    let src_esds_bytes = {
        let mut src = std::fs::File::open(path)
            .map_err(|e| format!("打开源文件失败: {}", e))?;
        let mut file_data = Vec::new();
        std::io::Read::read_to_end(&mut src, &mut file_data)
            .map_err(|e| format!("读取源文件失败: {}", e))?;
        // 搜索 "esds" 签名 (0x65 0x73 0x64 0x73)
        file_data.windows(4).position(|w| w == [0x65, 0x73, 0x64, 0x73])
            .and_then(|pos| {
                if pos < 4 { return None; }
                let box_size = u32::from_be_bytes([file_data[pos-4], file_data[pos-3], file_data[pos-2], file_data[pos-1]]);
                if box_size < 8 || pos + box_size as usize > file_data.len() { return None; }
                Some(file_data[pos-4..pos-4 + box_size as usize].to_vec())
            })
    };

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 10.0 }));

    let config = mp4::Mp4Config {
        major_brand: mp4.ftyp.major_brand,
        minor_version: mp4.ftyp.minor_version,
        compatible_brands: mp4.ftyp.compatible_brands.clone(),
        timescale,
    };

    let mut writer = mp4::Mp4Writer::write_start(&mut buf, &config)
        .map_err(|e| format!("创建写入器失败: {}", e))?;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 50.0 }));

    // 打开源文件用于读取 sample 数据（复用同一个文件句柄）
    let mut src_file = std::fs::File::open(path)
        .map_err(|e| format!("打开源文件失败: {}", e))?;

    // ponytail: writer.add_track() 按 push 顺序分配 track_id 1, 2, 3...
    // 源 mp4.tracks() 是 HashMap，遍历顺序不确定，必须用 writer 实际分配的 track_id
    // 否则音频轨道可能先被遍历（源 track_id=2），writer 分配 audio=1，
    // 但代码用 track_id=2 写 sample → 数据写进 writer 分配给 video 的轨道 → 杂音+时长错误
    let mut writer_track_id: u32 = 1;

    // 处理每个轨道（视频 + 音频）
    for track in mp4.tracks().values() {
        let track_type = track.track_type().map_err(|e| format!("获取轨道类型失败: {}", e))?;

        // 构建 TrackConfig
        let media_conf = match track.media_type().map_err(|e| format!("获取媒体类型失败: {}", e))? {
            mp4::MediaType::H264 => {
                let avc1 = track.trak.mdia.minf.stbl.stsd.avc1.as_ref()
                    .ok_or("未找到 avc1 box")?;
                let sps = avc1.avcc.sequence_parameter_sets.get(0)
                    .ok_or("未找到 SPS")?;
                let pps = avc1.avcc.picture_parameter_sets.get(0)
                    .ok_or("未找到 PPS")?;
                mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
                    width: track.width(),
                    height: track.height(),
                    seq_param_set: sps.bytes.to_vec(),
                    pic_param_set: pps.bytes.to_vec(),
                })
            }
            mp4::MediaType::H265 => {
                mp4::MediaConfig::HevcConfig(mp4::HevcConfig {
                    width: track.width(),
                    height: track.height(),
                })
            }
            mp4::MediaType::AAC => {
                let mp4a = track.trak.mdia.minf.stbl.stsd.mp4a.as_ref()
                    .ok_or("未找到 mp4a box")?;
                let esds = mp4a.esds.as_ref()
                    .ok_or("未找到 esds box")?;
                let dec_config = &esds.es_desc.dec_config;
                let profile = mp4::AudioObjectType::try_from(dec_config.dec_specific.profile)
                    .map_err(|e| format!("音频 profile 转换失败: {}", e))?;
                let freq_index = mp4::SampleFreqIndex::try_from(dec_config.dec_specific.freq_index)
                    .map_err(|e| format!("频率索引转换失败: {}", e))?;
                let chan_conf = mp4::ChannelConfig::try_from(dec_config.dec_specific.chan_conf)
                    .map_err(|e| format!("声道配置转换失败: {}", e))?;
                mp4::MediaConfig::AacConfig(mp4::AacConfig {
                    bitrate: dec_config.avg_bitrate,
                    profile,
                    freq_index,
                    chan_conf,
                })
            }
            _ => continue,
        };

        let track_config = mp4::TrackConfig {
            track_type,
            timescale: track.timescale(),
            language: track.language().to_string(),
            media_conf,
        };

        writer.add_track(&track_config)
            .map_err(|e| format!("添加轨道失败: {}", e))?;

        // 使用 writer 实际分配的 track_id，而不是源文件的 track_id
        let track_id = writer_track_id;
        writer_track_id += 1;

        // 计算该轨道的裁剪 sample 范围
        let (clip_start, clip_end) = if track_type == mp4::TrackType::Video {
            (actual_start_sample, actual_end_sample)
        } else {
            // 音频轨道：根据实际裁剪时间计算 sample 范围
            let audio_ts = track.timescale() as f64;
            let audio_count = track.sample_count();
            let audio_dur = if track.default_sample_duration > 0 {
                track.default_sample_duration as f64 / audio_ts
            } else {
                total_duration / audio_count.max(1) as f64
            };
            let s = ((actual_start_secs / audio_dur) as u32).min(audio_count).max(1);
            let e = ((actual_end_secs / audio_dur) as u32).min(audio_count);
            (s, e)
        };

        // 逐个读取并写入 sample
        let mut total_samples_written = 0u32;
        let mut total_duration_ticks = 0u64;
        for sample_id in clip_start..=clip_end {
            let offset = get_sample_offset(track, sample_id)?;
            let size = get_sample_size(track, sample_id)?;
            let dur = get_sample_duration(track, sample_id);
            let is_sync = is_sync_sample(track, sample_id);

            use std::io::{Read, Seek, SeekFrom};
            src_file.seek(SeekFrom::Start(offset))
                .map_err(|e| format!("seek 失败: {}", e))?;
            let mut buf = vec![0u8; size as usize];
            src_file.read_exact(&mut buf)
                .map_err(|e| format!("读取 sample {} 数据失败: {}", sample_id, e))?;

            let sample = mp4::Mp4Sample {
                start_time: 0, // Mp4Writer 会自动计算
                duration: dur,
                rendering_offset: 0,
                is_sync,
                bytes: mp4::Bytes::from(buf),
            };

            writer.write_sample(track_id, &sample)
                .map_err(|e| format!("写入 sample {} 失败: {}", sample_id, e))?;
            
            total_samples_written += 1;
            total_duration_ticks += dur as u64;
        }
        
        let track_type_str = if track_type == mp4::TrackType::Video { "video" } else { "audio" };
        let track_timescale = track.timescale() as f64;
        let track_duration_secs = total_duration_ticks as f64 / track_timescale;
        debug_log!("轨道 {} ({}): 写入 sample {}-{}, 共 {} 个, 总时长 {} ticks = {:.3}s, timescale={}",
            track.track_id(), track_type_str, clip_start, clip_end, total_samples_written,
            total_duration_ticks, track_duration_secs, track.timescale());
        
        // 音频轨道额外调试信息
        if track_type == mp4::TrackType::Audio {
            debug_log!("音频轨道详情: sample_count={}, timescale={}, default_sample_duration={}",
                track.sample_count(), track.timescale(), track.default_sample_duration);
            // 检查前 5 个 sample 的详细信息
            let check_count = 5.min(clip_end - clip_start + 1);
            for i in 0..check_count {
                let sample_id = clip_start + i;
                let offset = get_sample_offset(track, sample_id).unwrap_or(0);
                let size = get_sample_size(track, sample_id).unwrap_or(0);
                let dur = get_sample_duration(track, sample_id);
                let is_sync = is_sync_sample(track, sample_id);
                debug_log!("音频 sample[{}]: id={}, offset={}, size={}, dur={}, sync={}",
                    i, sample_id, offset, size, dur, is_sync);
            }
        }
    }

    writer.write_end()
        .map_err(|e| format!("完成写入失败: {}", e))?;

    let mut data = buf.into_inner();

    // ponytail: 用源文件的完整 ESDS 盒替换 mp4 crate 重建的 ESDS 盒
    // mp4 crate 的 ESDS 盒使用紧凑编码，丢失了源文件的 DecoderSpecificInfo 扩展数据，
    // 导致 AAC 解码器无法正确初始化。必须用源文件原始字节整体替换。
    //
    // 策略：替换 ESDS 盒内容，同时调整所有父盒（stsd/stbl/minf/mdia/trak/moov）的 size 字段，
    // 并在 data 中插入差量字节以保持文件结构完整。
    if let Some(esds_pos) = data.windows(4).position(|w| w == [0x65, 0x73, 0x64, 0x73]) {
        debug_log!("ESDS 盒位置: esds_pos={}", esds_pos);
        if esds_pos >= 4 {
            let out_box_size = u32::from_be_bytes([data[esds_pos-4], data[esds_pos-3], data[esds_pos-2], data[esds_pos-1]]) as usize;

            if let Some(ref esds_bytes) = src_esds_bytes {
                let src_box_size = esds_bytes.len();
                debug_log!("ESDS 盒大小: out_box_size={}, src_box_size={}", out_box_size, src_box_size);
                
                if src_box_size == out_box_size {
                    // 大小一致，直接替换
                    data[esds_pos-4..esds_pos-4 + out_box_size].copy_from_slice(esds_bytes);
                } else {
                    // 大小不一致，用源文件完整 ESDS 盒（含 size 字段）整体替换
                    let size_diff = src_box_size as i64 - out_box_size as i64;
                    debug_log!("ESDS 大小差异: size_diff={}", size_diff);
                    
                    // 替换整个 ESDS 盒（size + type + data），保持盒结构完整
                    data.splice(esds_pos - 4..esds_pos - 4 + out_box_size, esds_bytes.iter().copied());
                    
                    // 调整所有父盒的 size 字段（mp4a, stsd, stbl, minf, mdia, trak, moov）
                    adjust_parent_box_sizes(&mut data, esds_pos - 4, size_diff);
                }
            } else {
                debug_log!("ESDS 替换: 无源文件 ESDS 数据，执行精准修复");
                fix_esds_slconfig(&mut data, esds_pos, out_box_size);
            }
        }
    } else {
        debug_log!("ESDS 盒未找到");
    }

    // 修复 mp4a 盒的 samplesize 字段：设为 1 使 Windows 计算 nBlockAlign=1
    // 从 esds_pos 向前搜索最近的 mp4a 签名，根据 size 字段验证后计算 samplesize 精确位置
    // mp4a 盒结构: 4字节size + 4字节type + 28字节AudioSampleEntry，samplesize 在偏移 26 处
    if let Some(esds_pos) = data.windows(4).position(|w| w == [0x65, 0x73, 0x64, 0x73]) {
        // 从 esds_pos 向前搜索最近的 mp4a 签名
        if let Some(mp4a_pos) = data[..esds_pos].windows(4).rposition(|w| w == [0x6D, 0x70, 0x34, 0x61]) {
            // 验证 mp4a 盒 size 字段（4 字节大端）
            let mp4a_size = u32::from_be_bytes([data[mp4a_pos-4], data[mp4a_pos-3], data[mp4a_pos-2], data[mp4a_pos-1]]);
            debug_log!("mp4a 盒位置: mp4a_pos={}, mp4a_size={}", mp4a_pos, mp4a_size);
            
            // 合理的 mp4a 盒大小：至少 36 字节（8+28），不超过 1000 字节
            if mp4a_size >= 36 && mp4a_size <= 1000 {
                let samplesize_offset = mp4a_pos + 26;
                if samplesize_offset + 2 <= data.len() {
                    let old_samplesize = u16::from_be_bytes([data[samplesize_offset], data[samplesize_offset + 1]]);
                    data[samplesize_offset] = 0;
                    data[samplesize_offset + 1] = 1;
                    debug_log!("mp4a samplesize 修改: {} → 1", old_samplesize);
                }
            } else {
                debug_log!("mp4a 盒大小异常，跳过 samplesize 修改");
            }
        } else {
            debug_log!("未找到 mp4a 签名");
        }
    }

    std::fs::write(&output_path, &data)
        .map_err(|e| format!("写入输出文件失败: {}", e))?;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = data.len() as u64;

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration: actual_end_secs - actual_start_secs,
        actual_start: Some(actual_start_secs),
        actual_end: Some(actual_end_secs),
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
        // ponytail: 320px 宽度让缩略图在 180px 高的 canvas 中更清晰
        extract_thumbnails_ffmpeg(&path, count, 320)
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

// ============ 视频转码 (F16) ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTranscodeOptions {
    pub output_format: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeResult {
    pub output_path: String,
    pub output_size: u64,
    pub input_size: u64,
    pub duration: f64,
}

#[tauri::command]
pub async fn video_transcode(
    app_handle: tauri::AppHandle,
    path: String,
    options: VideoTranscodeOptions,
) -> Result<TranscodeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            return Err("视频转码需要 ffmpeg，请先安装 ffmpeg".to_string());
        }
        do_video_transcode(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_video_transcode(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &VideoTranscodeOptions,
) -> Result<TranscodeResult, String> {
    let input_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // 获取输入视频时长
    let duration = get_video_info_ffprobe(path)?.duration;

    let input_stem = std::path::Path::new(path)
        .file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let ext = &options.output_format;
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent().unwrap_or(std::path::Path::new("."))
            .join(format!("{}_transcoded.{}", input_stem, ext))
    };

    let _ = app_handle.emit("video-transcode-progress", serde_json::json!({ "progress": 5.0 }));

    let mut args: Vec<String> = vec![
        "-y".to_string(),
        "-i".to_string(), path.to_string(),
    ];

    // 视频编码器
    args.push("-c:v".to_string());
    args.push(options.video_codec.clone());

    // 音频编码器
    args.push("-c:a".to_string());
    args.push(options.audio_codec.clone());

    // 分辨率
    if let (Some(w), Some(h)) = (options.width, options.height) {
        args.push("-vf".to_string());
        args.push(format!("scale={}:{}", w, h));
    }

    // 帧率
    if let Some(fps) = options.fps {
        args.push("-r".to_string());
        args.push(format!("{:.1}", fps));
    }

    // 视频比特率
    if let Some(ref bitrate) = options.video_bitrate {
        args.push("-b:v".to_string());
        args.push(bitrate.clone());
    }

    // 音频比特率
    if let Some(ref bitrate) = options.audio_bitrate {
        args.push("-b:a".to_string());
        args.push(bitrate.clone());
    }

    args.push(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("video-transcode-progress", serde_json::json!({ "progress": 20.0 }));

    debug_log!("ffmpeg 转码: {:?}", args);

    let output = std::process::Command::new("ffmpeg")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    let _ = app_handle.emit("video-transcode-progress", serde_json::json!({ "progress": 90.0 }));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let preview: String = stderr.chars().take(300).collect();
        return Err(format!("ffmpeg 转码失败: {}", preview));
    }

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    let _ = app_handle.emit("video-transcode-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(TranscodeResult { output_path: output_path.to_string_lossy().to_string(), output_size, input_size, duration })
}

// ============ 音频提取 (F17) ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioExtractOptions {
    pub output_format: String,
    pub audio_codec: String,
    pub bitrate: Option<String>,
    pub quality: Option<u32>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioExtractResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}

#[tauri::command]
pub async fn audio_extract(
    app_handle: tauri::AppHandle,
    path: String,
    options: AudioExtractOptions,
) -> Result<AudioExtractResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            return Err("音频提取需要 ffmpeg，请先安装 ffmpeg".to_string());
        }
        do_audio_extract(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_audio_extract(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &AudioExtractOptions,
) -> Result<AudioExtractResult, String> {
    let duration = get_video_info_ffprobe(path)?.duration;

    let input_stem = std::path::Path::new(path)
        .file_stem().and_then(|s| s.to_str()).unwrap_or("audio");
    let ext = &options.output_format;
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent().unwrap_or(std::path::Path::new("."))
            .join(format!("{}_audio.{}", input_stem, ext))
    };

    let _ = app_handle.emit("audio-extract-progress", serde_json::json!({ "progress": 5.0 }));

    let mut args: Vec<String> = vec![
        "-y".to_string(),
        "-i".to_string(), path.to_string(),
        "-vn".to_string(), // 去除视频流
        "-c:a".to_string(), options.audio_codec.clone(),
    ];

    if let Some(ref bitrate) = options.bitrate {
        args.push("-b:a".to_string());
        args.push(bitrate.clone());
    }

    // WAV 格式特殊处理
    if options.output_format == "wav" {
        args.push("-acodec".to_string());
        args.push("pcm_s16le".to_string());
    }

    args.push(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-extract-progress", serde_json::json!({ "progress": 20.0 }));

    debug_log!("ffmpeg 音频提取: {:?}", args);

    let output = std::process::Command::new("ffmpeg")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    let _ = app_handle.emit("audio-extract-progress", serde_json::json!({ "progress": 90.0 }));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let preview: String = stderr.chars().take(300).collect();
        return Err(format!("ffmpeg 音频提取失败: {}", preview));
    }

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    let _ = app_handle.emit("audio-extract-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(AudioExtractResult { output_path: output_path.to_string_lossy().to_string(), output_size, duration })
}

// ============ 视频压缩 (F18) ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCompressOptions {
    pub crf: u32,
    pub preset: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub width: Option<u32>,
    pub keep_resolution: bool,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub output_path: String,
    pub output_size: u64,
    pub input_size: u64,
    pub compression_ratio: f64,
    pub duration: f64,
}

#[tauri::command]
pub async fn video_compress(
    app_handle: tauri::AppHandle,
    path: String,
    options: VideoCompressOptions,
) -> Result<CompressResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            return Err("视频压缩需要 ffmpeg，请先安装 ffmpeg".to_string());
        }
        do_video_compress(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_video_compress(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &VideoCompressOptions,
) -> Result<CompressResult, String> {
    let input_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let duration = get_video_info_ffprobe(path)?.duration;

    let input_stem = std::path::Path::new(path)
        .file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent().unwrap_or(std::path::Path::new("."))
            .join(format!("{}_compressed.mp4", input_stem))
    };

    let _ = app_handle.emit("video-compress-progress", serde_json::json!({ "progress": 5.0 }));

    let mut args: Vec<String> = vec![
        "-y".to_string(),
        "-i".to_string(), path.to_string(),
        "-c:v".to_string(), options.video_codec.clone(),
        "-crf".to_string(), options.crf.to_string(),
        "-preset".to_string(), options.preset.clone(),
        "-c:a".to_string(), options.audio_codec.clone(),
    ];

    // 缩放分辨率
    if !options.keep_resolution {
        if let Some(w) = options.width {
            args.push("-vf".to_string());
            args.push(format!("scale={}:-1", w));
        }
    }

    args.push(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("video-compress-progress", serde_json::json!({ "progress": 20.0 }));

    debug_log!("ffmpeg 压缩: {:?}", args);

    let output = std::process::Command::new("ffmpeg")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    let _ = app_handle.emit("video-compress-progress", serde_json::json!({ "progress": 90.0 }));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let preview: String = stderr.chars().take(300).collect();
        return Err(format!("ffmpeg 压缩失败: {}", preview));
    }

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
    let compression_ratio = if input_size > 0 {
        (output_size as f64 / input_size as f64 * 100.0).round()
    } else {
        100.0
    };

    let _ = app_handle.emit("video-compress-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(CompressResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        input_size,
        compression_ratio,
        duration,
    })
}

// ============ 视频合并 (F19) ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMergeOptions {
    pub paths: Vec<String>,
    pub output_format: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
    pub file_count: u32,
}

#[tauri::command]
pub async fn video_merge(
    app_handle: tauri::AppHandle,
    options: VideoMergeOptions,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            return Err("视频合并需要 ffmpeg，请先安装 ffmpeg".to_string());
        }
        do_video_merge(&app_handle, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_video_merge(
    app_handle: &tauri::AppHandle,
    options: &VideoMergeOptions,
) -> Result<MergeResult, String> {
    if options.paths.len() < 2 {
        return Err("至少需要 2 个视频文件".to_string());
    }

    let file_count = options.paths.len() as u32;
    let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 5.0 }));

    // 判断是否所有文件格式相同 → 无损合并
    let all_same_format = options.paths.iter().all(|p| {
        let ext = std::path::Path::new(p).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        ext == options.output_format
    });

    let first_path = &options.paths[0];
    let input_stem = std::path::Path::new(first_path)
        .file_stem().and_then(|s| s.to_str()).unwrap_or("merged");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(first_path)
            .parent().unwrap_or(std::path::Path::new("."))
            .join(format!("{}_merged.{}", input_stem, &options.output_format))
    };

    let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 10.0 }));

    if all_same_format {
        // 同格式无损合并（concat demuxer）
        let temp_dir = std::env::temp_dir().join("litobox_video_merge");
        let _ = std::fs::create_dir_all(&temp_dir);
        let filelist_path = temp_dir.join("filelist.txt");

        let filelist: String = options.paths.iter()
            .map(|p| format!("file '{}'", p.replace('\\', "\\\\").replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&filelist_path, &filelist)
            .map_err(|e| format!("写入文件列表失败: {}", e))?;

        let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 20.0 }));

        let output = std::process::Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f", "concat",
                "-safe", "0",
                "-i", &filelist_path.to_string_lossy(),
                "-c", "copy",
                &output_path.to_string_lossy(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

        let _ = std::fs::remove_file(&filelist_path);
        let _ = std::fs::remove_dir_all(&temp_dir);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let preview: String = stderr.chars().take(300).collect();
            return Err(format!("ffmpeg 合并失败: {}", preview));
        }
    } else {
        // 不同格式需转码合并（concat filter）
        let mut args: Vec<String> = vec!["-y".to_string()];
        for p in &options.paths {
            args.push("-i".to_string());
            args.push(p.clone());
        }
        // 构建 filter_complex: concat=n=N:v=1:a=1
        let filter = format!("concat=n={}:v=1:a=1", options.paths.len());
        args.push("-filter_complex".to_string());
        args.push(filter);
        // 编码器
        args.push("-c:v".to_string());
        args.push(options.video_codec.clone());
        args.push("-c:a".to_string());
        args.push(options.audio_codec.clone());
        args.push(output_path.to_string_lossy().to_string());

        let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 20.0 }));

        debug_log!("ffmpeg 合并(转码): {:?}", args);

        let output = std::process::Command::new("ffmpeg")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let preview: String = stderr.chars().take(300).collect();
            return Err(format!("ffmpeg 合并失败: {}", preview));
        }
    }

    let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 90.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
    let duration = get_video_info_ffprobe(&output_path.to_string_lossy())
        .map(|info| info.duration).unwrap_or(0.0);

    let _ = app_handle.emit("video-merge-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(MergeResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
        file_count,
    })
}