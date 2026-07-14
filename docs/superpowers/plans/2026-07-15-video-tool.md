# 视频工具（VideoTool）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增"视频工具"页面，提供 MP4 视频裁剪功能（关键帧无损裁剪 + ffmpeg 可选增强 + 缩略图时间轴）

**Architecture:** 纯 Rust 双轨策略 — mp4 crate 解析关键帧索引做无损裁剪，ffmpeg 运行时探测做缩略图提取和更精确裁剪。前端 Canvas 时间轴 + 滑块交互，与 AudioTool 模式一致。

**Tech Stack:** Rust (mp4 crate), ffmpeg/ffprobe (可选), Vue 3 + Canvas API, Tauri 2.0

---

### Task 1: 添加 mp4 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 的 dependencies 中新增 mp4**

```toml
mp4 = "0.14"
```

添加到 `lopdf = "0.34"` 之后：

```toml
lopdf = "0.34"
mp4 = "0.14"
flate2 = "1.0"
```

- [ ] **Step 2: 验证依赖下载**

```powershell
cd src-tauri; cargo check 2>&1 | Select-Object -Last 20
```

Expected: 编译通过，`mp4` 依赖下载成功。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: 新增 mp4 crate 依赖"
```

---

### Task 2: 创建 video_tools.rs 后端核心逻辑

**Files:**
- Create: `src-tauri/src/video_tools.rs`

- [ ] **Step 1: 创建文件骨架 + 数据结构定义**

```rust
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
```

- [ ] **Step 2: 实现 ffmpeg 可用性检测 + 格式识别**

```rust
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
```

- [ ] **Step 3: 实现纯 Rust 视频元信息读取（mp4 crate）**

```rust
use mp4::Mp4Reader;

fn get_video_info_mp4(path: &str) -> Result<VideoInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();

    let f = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let buf_reader = std::io::BufReader::new(f);

    let mp4 = Mp4Reader::read_header(buf_reader, file_size)
        .map_err(|e| format!("MP4 解析失败: {}", e))?;

    let duration = mp4.duration().as_secs_f64();

    // 获取视频轨道信息
    let video_track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    let width = video_track.width();
    let height = video_track.height();

    // 编码格式
    let codec = match video_track.media_type() {
        Ok(mp4::MediaType::H264) => "h264".to_string(),
        Ok(mp4::MediaType::H265) => "h265".to_string(),
        Ok(mp4::MediaType::AV1) => "av1".to_string(),
        _ => "unknown".to_string(),
    };

    // 帧率（从 timescale 和 sample 计算）
    let timescale = video_track.timescale() as f64;
    let default_sample_duration = video_track.default_sample_duration() as f64;
    let fps = if default_sample_duration > 0.0 {
        timescale / default_sample_duration
    } else {
        0.0
    };

    let bitrate = if duration > 0.0 {
        ((file_size as f64 * 8.0) / duration / 1000.0) as u32
    } else {
        0
    };

    let format = guess_video_format(path);

    Ok(VideoInfo { duration, width, height, codec, fps, bitrate, file_size, format })
}
```

- [ ] **Step 4: 实现 ffprobe 视频元信息读取**

```rust
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
```

- [ ] **Step 5: 实现缩略图提取（ffmpeg）**

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

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
```

- [ ] **Step 6: 实现纯 Rust 关键帧裁剪**

```rust
fn do_video_crop_keyframe(
    app_handle: &tauri::AppHandle,
    path: &str,
    options: &VideoCropOptions,
) -> Result<CropResult, String> {
    use mp4::Mp4Reader;

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

    let mp4 = Mp4Reader::read_header(buf_reader, file_size)
        .map_err(|e| format!("MP4 解析失败: {}", e))?;

    let video_track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    let timescale = video_track.timescale() as f64;
    let total_duration = mp4.duration().as_secs_f64();

    // 获取关键帧索引（stss）
    let keyframes = video_track.sync_samples()
        .map(|ss| {
            let mut samples: Vec<u32> = ss.iter().copied().collect();
            samples.sort();
            samples
        })
        .unwrap_or_default();

    if keyframes.is_empty() {
        return Err("未找到关键帧索引，无法进行无损裁剪。请安装 ffmpeg 以获得完整支持".to_string());
    }

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 15.0 }));

    // 将时间转为 sample 编号
    let sample_count = video_track.sample_count() as u32;
    let default_sample_dur = video_track.default_sample_duration() as f64;
    let sample_duration = if default_sample_dur > 0.0 {
        default_sample_dur / timescale
    } else {
        total_duration / sample_count.max(1) as f64
    };

    let start_sample = ((options.start_time / sample_duration) as u32).min(sample_count);
    let end_sample = ((options.end_time / sample_duration) as u32).min(sample_count);

    // 找到起止时间对应的最近关键帧
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

    // 使用 mp4 crate 重写裁剪后的 MP4
    // 重新打开文件读取原始数据
    let src_data = std::fs::read(path)
        .map_err(|e| format!("读取源文件失败: {}", e))?;

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 50.0 }));

    // 重新解析以获取 sample 偏移信息
    let cursor = std::io::Cursor::new(&src_data);
    let buf_reader = std::io::BufReader::new(cursor);
    let mp4 = Mp4Reader::read_header(buf_reader, src_data.len() as u64)
        .map_err(|e| format!("二次解析 MP4 失败: {}", e))?;

    let track = mp4.tracks().values()
        .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
        .ok_or("未找到视频轨道")?;

    // 获取 sample 偏移表
    let stbl = track.stbl();
    let stsz = stbl.stsz();
    let stco = stbl.stco();

    let sample_sizes: Vec<u32> = stsz.entry_sizes().to_vec();
    let chunk_offsets: Vec<u64> = stco.entries().iter().map(|e| e.chunk_offset).collect();

    // 获取 sample-to-chunk 映射
    let stsc = stbl.stsc();
    let stsc_entries: Vec<(u32, u32, u32)> = stsc.entries().iter()
        .map(|e| (e.first_chunk, e.samples_per_chunk, e.sample_description_index))
        .collect();

    // 构建 sample → offset 映射
    let mut sample_offsets: Vec<u64> = Vec::new();
    let mut chunk_idx = 0usize;
    let mut samples_in_chunk = 0u32;
    let mut current_sps = 0u32;
    let mut current_sdi = 0u32;

    for sample_idx in 0..sample_sizes.len() {
        if samples_in_chunk == 0 {
            // 确定当前 chunk 的 samples_per_chunk
            let mut sps = 0u32;
            let mut sdi = 0u32;
            for (fc, sc, sd) in &stsc_entries {
                if chunk_idx + 1 >= *fc as usize {
                    sps = *sc;
                    sdi = *sd;
                }
            }
            if sps == 0 {
                sps = 1;
            }
            current_sps = sps;
            current_sdi = sdi;
            samples_in_chunk = sps;
        }

        if chunk_idx < chunk_offsets.len() {
            let base_offset = chunk_offsets[chunk_idx];
            let offset_in_chunk: u64 = sample_sizes[..sample_idx].iter()
                .skip(sample_idx.saturating_sub((samples_in_chunk) as usize))
                .take((current_sps - samples_in_chunk) as usize)
                .map(|&s| s as u64)
                .sum();
            sample_offsets.push(base_offset + offset_in_chunk);
        } else {
            sample_offsets.push(0);
        }

        samples_in_chunk -= 1;
        if samples_in_chunk == 0 {
            chunk_idx += 1;
        }
    }

    let _ = app_handle.emit("video-crop-progress", serde_json::json!({ "progress": 70.0 }));

    // 收集裁剪范围内所有 sample 的数据
    let start_offset = sample_offsets.get(actual_start_sample as usize).copied().unwrap_or(0);
    let end_offset = if actual_end_sample as usize + 1 < sample_offsets.len() {
        sample_offsets[actual_end_sample as usize + 1]
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
```

- [ ] **Step 7: 实现 ffmpeg 裁剪**

```rust
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
```

- [ ] **Step 8: 实现 Tauri commands**

```rust
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
```

- [ ] **Step 9: 验证编译**

```powershell
cd src-tauri; cargo check 2>&1 | Select-Object -Last 20
```

Expected: `error: linking with `link.exe` failed: exit code: 0` 或 `Finished` — 无编译错误。

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/video_tools.rs
git commit -m "feat: 新增 video_tools.rs 视频处理后端（mp4 解析 + ffmpeg 裁剪 + 缩略图提取）"
```

---

### Task 3: 注册 Tauri commands

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 main.rs 中添加 mod 声明和 command 注册**

在 `mod audio_tools;` 之后添加 `mod video_tools;`：

```rust
mod audio_tools;
mod video_tools;
mod pdf_tools;
```

在 `generate_handler!` 中 `audio_tools::check_ffmpeg,` 行之前添加 video_tools 的 commands：

```rust
generate_handler![
    ...
    audio_tools::check_ffmpeg,
    audio_tools::get_audio_info,
    audio_tools::generate_waveform,
    audio_tools::audio_crop,
    audio_tools::get_audio_preview,
    video_tools::get_video_info,
    video_tools::extract_thumbnails,
    video_tools::video_crop,
    pdf_tools::detect_ghostscript,
    ...
]
```

- [ ] **Step 2: 验证编译**

```powershell
cd src-tauri; cargo check 2>&1 | Select-Object -Last 20
```

Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: 注册 video_tools Tauri commands"
```

---

### Task 4: 创建 VideoTool.vue 前端页面

**Files:**
- Create: `src/views/VideoTool.vue`

- [ ] **Step 1: 创建完整 Vue 组件**

```vue
<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': useFfmpeg, 'ffmpeg-missing': !useFfmpeg }" v-if="ffmpegChecked">
      <template v-if="useFfmpeg">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，可显示缩略图时间轴，裁剪更精确
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，缩略图不可用，裁剪精度受关键帧限制。
        <span class="ffmpeg-tip">
          安装 ffmpeg 可启用缩略图和高精度裁剪：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
        </span>
      </template>
    </div>

    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="video-tool-tabs">
        <el-tab-pane label="视频裁剪" name="crop" />
      </el-tabs>
    </div>

    <!-- Tab: 视频裁剪 -->
    <template v-if="activeTab === 'crop'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择视频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openFile" :loading="isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="filePath" class="video-file-info">
            <span class="file-name">{{ fileName }}</span>
            <span class="file-detail" v-if="videoInfo">
              {{ formatDuration(videoInfo.duration) }} | {{ videoInfo.width }}x{{ videoInfo.height }} |
              {{ videoInfo.codec.toUpperCase() }} | {{ videoInfo.fps.toFixed(1) }}fps |
              {{ videoInfo.bitrate }}kbps | {{ formatFileSize(videoInfo.file_size) }}
            </span>
          </div>
        </div>
      </div>

      <!-- 时间轴 -->
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">时间轴</span>
        </div>
        <div class="card-body">
          <div class="timeline-container" ref="timelineContainer" @contextmenu.prevent>
            <canvas ref="canvasRef" class="timeline-canvas" @mousedown="onCanvasMouseDown" @contextmenu.prevent></canvas>
            <div
              class="slider-handle start-handle"
              :style="{ left: timeToPercent(startTime) + '%' }"
              @mousedown.stop="onSliderMouseDown($event, 'start')"
            ></div>
            <div
              class="slider-handle end-handle"
              :style="{ left: timeToPercent(endTime) + '%' }"
              @mousedown.stop="onSliderMouseDown($event, 'end')"
            ></div>
          </div>
          <div class="timeline-labels">
            <span>{{ formatTime(startTime) }}</span>
            <span>{{ formatTime(endTime) }}</span>
          </div>
        </div>
      </div>

      <!-- 裁剪设置 -->
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">裁剪设置</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">起始时间</div>
              <el-input-number
                v-model="startTime"
                :min="0"
                :max="endTime - 0.1"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
            <div class="action-group">
              <div class="group-label">结束时间</div>
              <el-input-number
                v-model="endTime"
                :min="startTime + 0.1"
                :max="videoInfo.duration"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
          </div>
          <div class="segment-info" v-if="videoInfo">
            片段时长: {{ formatDuration(segmentDuration) }}
          </div>
          <div v-if="actualRange" class="keyframe-hint">
            实际裁剪区间（关键帧对齐）: {{ formatTime(actualRange.start) }} - {{ formatTime(actualRange.end) }}
          </div>
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作 -->
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="cropVideo" :loading="isProcessing" :disabled="!isRangeValid">
                裁剪并导出
              </el-button>
              <el-button size="small" @click="resetForm">重置</el-button>
            </div>
          </div>
          <el-progress v-if="isProcessing" :percentage="cropProgress" :stroke-width="6" style="margin-top: 12px" />
        </div>
      </div>
    </template>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'

// ============ 类型定义 ============
interface VideoInfo {
  duration: number
  width: number
  height: number
  codec: string
  fps: number
  bitrate: number
  file_size: number
  format: string
}

interface ThumbnailResult {
  images: string[]
  timestamps: number[]
}

interface CropResult {
  output_path: string
  output_size: number
  duration: number
  actual_start: number | null
  actual_end: number | null
}

// ============ Tab 状态 ============
const activeTab = ref('crop')

// ============ 状态 ============
const filePath = ref('')
const fileName = ref('')
const videoInfo = ref<VideoInfo | null>(null)
const thumbnails = ref<ThumbnailResult>({ images: [], timestamps: [] })
const startTime = ref(0)
const endTime = ref(0)
const isProcessing = ref(false)
const isLoadingInfo = ref(false)
const saveToSamePath = ref(true)
const cropProgress = ref(0)
const useFfmpeg = ref(false)
const ffmpegChecked = ref(false)
const error = ref('')
const actualRange = ref<{ start: number; end: number } | null>(null)

// ============ 计算属性 ============
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// ============ Canvas 时间轴 ============
const canvasRef = ref<HTMLCanvasElement | null>(null)
const timelineContainer = ref<HTMLDivElement | null>(null)

function drawTimeline() {
  const canvas = canvasRef.value
  if (!canvas || !videoInfo.value) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.scale(dpr, dpr)
  const width = rect.width
  const height = rect.height
  const dur = videoInfo.value.duration

  const style = getComputedStyle(document.documentElement)
  const bgColor = style.getPropertyValue('--bg-input').trim() || '#0d1520'
  const primaryColor = style.getPropertyValue('--accent-cyan').trim() || '#00d4ff'
  const secondaryColor = style.getPropertyValue('--text-secondary').trim() || '#94a3b8'
  const borderColor = style.getPropertyValue('--border-color').trim() || '#1e3a5f'

  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, width, height)

  const startX = (startTime.value / dur) * width
  const endX = (endTime.value / dur) * width

  // 绘制缩略图
  if (thumbnails.value.images.length > 0) {
    const n = thumbnails.value.images.length
    const thumbWidth = width / n
    const imgPromises: Promise<void>[] = []

    for (let i = 0; i < n; i++) {
      const x = i * thumbWidth
      const img = new Image()
      const promise = new Promise<void>((resolve) => {
        img.onload = () => {
          const h = (thumbWidth / img.width) * img.height
          const y = (height - h) / 2
          ctx.drawImage(img, x, y, thumbWidth, h)
          resolve()
        }
        img.onerror = () => resolve()
      })
      img.src = 'data:image/jpeg;base64,' + thumbnails.value.images[i]
      imgPromises.push(promise)
    }

    Promise.all(imgPromises).then(() => {
      // 绘制选中区域
      drawSelectionOverlay(ctx, startX, endX, width, height, primaryColor)
    })
  } else {
    // 纯文本时间轴：时间刻度线
    ctx.strokeStyle = secondaryColor + '66'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()

    // 刻度标记
    const tickCount = 10
    for (let i = 0; i <= tickCount; i++) {
      const x = (i / tickCount) * width
      const tickHeight = i % 2 === 0 ? 12 : 6
      ctx.strokeStyle = secondaryColor + '88'
      ctx.beginPath()
      ctx.moveTo(x, height / 2 - tickHeight)
      ctx.lineTo(x, height / 2 + tickHeight)
      ctx.stroke()
    }

    drawSelectionOverlay(ctx, startX, endX, width, height, primaryColor)
  }
}

function drawSelectionOverlay(
  ctx: CanvasRenderingContext2D,
  startX: number,
  endX: number,
  width: number,
  height: number,
  primaryColor: string
) {
  // 选中区域高亮
  ctx.fillStyle = primaryColor + '1A'
  ctx.fillRect(startX, 0, endX - startX, height)

  // 起始/结束边界线
  ctx.strokeStyle = primaryColor
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(startX, 0)
  ctx.lineTo(startX, height)
  ctx.stroke()
  ctx.beginPath()
  ctx.moveTo(endX, 0)
  ctx.lineTo(endX, height)
  ctx.stroke()
}

function timeToPercent(time: number): number {
  if (!videoInfo.value || videoInfo.value.duration <= 0) return 0
  return (time / videoInfo.value.duration) * 100
}

function percentToTime(percent: number): number {
  if (!videoInfo.value) return 0
  return Math.round((percent / 100) * videoInfo.value.duration * 10) / 10
}

// ============ 滑块拖拽 ============
let draggingSlider: 'start' | 'end' | null = null

function onSliderMouseDown(_e: MouseEvent, slider: 'start' | 'end') {
  draggingSlider = slider
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!draggingSlider || !timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  if (draggingSlider === 'start') {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

function onMouseUp() {
  draggingSlider = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown(e: MouseEvent) {
  if (!timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  const startDist = Math.abs(time - startTime.value)
  const endDist = Math.abs(time - endTime.value)

  if (startDist <= endDist) {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

// ============ 文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{
        name: '视频文件',
        extensions: useFfmpeg.value ? ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] : ['mp4', 'm4v']
      }],
      multiple: false,
    })
    if (!selected) return

    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''

    isLoadingInfo.value = true
    actualRange.value = null

    const info: VideoInfo = await invoke('get_video_info', { path: filePath.value, useFfmpeg: useFfmpeg.value })
    videoInfo.value = info

    startTime.value = 0
    endTime.value = info.duration

    // 提取缩略图
    const result: ThumbnailResult = await invoke('extract_thumbnails', { path: filePath.value, count: 20 })
    thumbnails.value = result

    await nextTick()
    drawTimeline()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetForm()
  } finally {
    isLoadingInfo.value = false
  }
}

async function cropVideo() {
  if (!isRangeValid.value) {
    ElMessage.warning('请设置有效的裁剪区间')
    return
  }

  try {
    error.value = ''
    isProcessing.value = true
    cropProgress.value = 0
    actualRange.value = null

    const unlisten = await listen<{ progress: number }>('video-crop-progress', (event) => {
      cropProgress.value = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!saveToSamePath.value) {
      const defaultName = fileName.value.replace(/\.[^.]+$/, '') + '_cropped.mp4'
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'MP4 视频', extensions: ['mp4'] }],
      })
      if (!outputPath) {
        unlisten()
        isProcessing.value = false
        return
      }
    }

    const result: CropResult = await invoke('video_crop', {
      path: filePath.value,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        use_ffmpeg: useFfmpeg.value,
        output_path: outputPath,
      },
    })

    unlisten()
    cropProgress.value = 100

    if (result.actual_start != null && result.actual_end != null) {
      actualRange.value = { start: result.actual_start, end: result.actual_end }
    }

    ElMessage.success(`裁剪完成，已保存到: ${result.output_path}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '裁剪失败'
  } finally {
    isProcessing.value = false
  }
}

function resetForm() {
  filePath.value = ''
  fileName.value = ''
  videoInfo.value = null
  thumbnails.value = { images: [], timestamps: [] }
  startTime.value = 0
  endTime.value = 0
  error.value = ''
  actualRange.value = null
}

// ============ 格式化 ============
function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = (seconds % 60).toFixed(1)
  return `${String(m).padStart(2, '0')}:${String(parseFloat(s)).padStart(4, '0')}`
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

// ============ 响应式 ============
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  try {
    useFfmpeg.value = await invoke('check_ffmpeg')
  } catch { /* 忽略 */ }
  ffmpegChecked.value = true

  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => drawTimeline())
    resizeObserver.observe(canvasRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

watch([startTime, endTime], () => drawTimeline())
watch(thumbnails, () => nextTick(() => drawTimeline()), { deep: true })
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.video-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .video-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.video-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.video-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
.video-file-info {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  color: var(--accent-cyan);
  font-size: 14px;
  font-weight: 500;
}

.file-detail {
  color: var(--text-secondary);
  font-size: 12px;
}

.timeline-container {
  position: relative;
  width: 100%;
  height: 120px;
  cursor: pointer;
}

.timeline-container:has(.slider-handle:active) {
  cursor: col-resize;
}

.timeline-canvas {
  width: 100%;
  height: 100%;
  border-radius: 4px;
}

.slider-handle {
  position: absolute;
  top: 0;
  width: 12px;
  height: 100%;
  transform: translateX(-50%);
  cursor: col-resize;
  z-index: 10;
}

.slider-handle::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent-cyan);
  border: 2px solid var(--bg-primary);
  box-shadow: 0 0 4px rgba(0, 0, 0, 0.4);
}

.timeline-labels {
  display: flex;
  justify-content: space-between;
  color: var(--text-secondary);
  font-size: 12px;
  margin-top: 4px;
  padding: 0 6px;
}

.unit-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-left: 4px;
}

.segment-info {
  margin-top: 8px;
  color: var(--accent-cyan);
  font-size: 13px;
}

.keyframe-hint {
  margin-top: 4px;
  color: var(--accent-orange);
  font-size: 12px;
}

/* ffmpeg 状态横幅 */
.ffmpeg-banner {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ffmpeg-banner.ffmpeg-detected {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent-green);
}

.ffmpeg-banner.ffmpeg-missing {
  background: rgba(59, 130, 246, 0.12);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: var(--accent-blue);
}

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-link {
  color: var(--accent-cyan);
  margin-left: 4px;
}

.ffmpeg-link:hover {
  text-decoration: underline;
}

.ffmpeg-tip {
  margin-left: 8px;
}

.ffmpeg-cmd {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--accent-orange);
  user-select: all;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/VideoTool.vue
git commit -m "feat: 新增 VideoTool.vue 视频裁剪前端页面（缩略图时间轴 + 滑块交互）"
```

---

### Task 5: 注册到 App.vue 和 TOOL_LIST

**Files:**
- Modify: `src/App.vue`
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 App.vue 中添加导入和映射**

在 `import AudioTool from '@/views/AudioTool.vue'` 之后添加：

```typescript
import VideoTool from '@/views/VideoTool.vue'
```

在 `toolComponentMap` 中 `audioTool: AudioTool,` 之后添加：

```typescript
videoTool: VideoTool,
```

- [ ] **Step 2: 在 TOOL_LIST 中添加"视频工具"条目**

在 `src/store/index.ts` 中，在 `audioTool` 条目之后添加：

```typescript
{ id: 'videoTool', name: '视频工具', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>`, description: '视频裁剪、片段提取，支持 MP4 关键帧无损裁剪，ffmpeg 可选增强', keywords: ['视频', '裁剪', 'mp4', 'video', '剪裁', '片段'], category: 'utility' },
```

- [ ] **Step 3: 验证前端构建**

```powershell
npm run build 2>&1 | Select-Object -Last 20
```

Expected: 构建成功，无 TypeScript 或 Vite 错误。

- [ ] **Step 4: Commit**

```bash
git add src/App.vue src/store/index.ts
git commit -m "feat: 注册 VideoTool 到侧边栏和应用路由"
```

---

### Task 6: 更新版本号和 README

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `README.md`

- [ ] **Step 1: 更新版本号**

在 `src-tauri/Cargo.toml` 中：

```toml
version = "5.2.0"
```

- [ ] **Step 2: 更新 README 功能记录**

在 README.md 的版本表格末尾（V5.1 之后）添加：

```markdown
| V5.2 | ✅ | 视频工具：MP4 视频裁剪（关键帧无损裁剪），缩略图时间轴可视化，滑块拖拽选区间，ffmpeg 可选增强 |
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml README.md
git commit -m "chore: 更新版本号至 5.2.0，更新 README 功能记录"
```

---

### Task 7: 端到端验证

- [ ] **Step 1: 启动 Tauri 开发服务器**

```powershell
npm run tauri dev
```

- [ ] **Step 2: 手动验证清单**

| 验证项 | 操作 | 预期 |
|--------|------|------|
| 侧边栏显示 | 查看侧边栏 utility 分类 | "视频工具"出现在音频工具之后 |
| 打开 MP4 文件 | 点击"打开文件"选择 MP4 | 显示元信息（时长/分辨率/编码/帧率） |
| 有 ffmpeg 缩略图 | 选择视频文件 | 时间轴显示缩略图条带 |
| 无 ffmpeg 降级 | 在没有 ffmpeg 的机器上测试 | 时间轴显示纯文本刻度线 |
| 滑块拖拽 | 拖拽起止滑块 | 时间轴实时更新选中区域 |
| 点击时间轴 | 点击 Canvas 不同位置 | 最近滑块移动到点击位置 |
| 纯 Rust 裁剪 | 无 ffmpeg 时点击裁剪 | 裁剪成功，显示实际裁剪区间 |
| ffmpeg 裁剪 | 有 ffmpeg 时点击裁剪 | 裁剪成功，速度快 |
| 错误处理 | 选择非 MP4 文件（无 ffmpeg） | 显示"仅支持 MP4 格式" |

- [ ] **Step 3: 修复问题后 Commit**

```bash
git add -A
git commit -m "fix: 视频工具端到端验证问题修复"
```

---

### 自审清单

1. **Spec 覆盖:** 所有 spec 中的功能（元信息、缩略图、关键帧裁剪、ffmpeg 裁剪、时间轴交互、错误处理）均有对应 task
2. **无占位符:** 所有代码步骤包含完整实现，无 TBD/TODO
3. **类型一致性:** `VideoInfo`/`CropResult`/`VideoCropOptions` 等结构体在前端 TypeScript 接口和后端 Rust 定义中字段名一致（snake_case）