use encoding_rs;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::os::windows::process::CommandExt;
use tauri::Emitter;
// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub duration: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub bitrate: u32,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    pub points: Vec<f32>,
    pub duration: f64,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropOptions {
    pub start_time: f64,
    pub end_time: f64,
    pub output_format: String,
    pub mp3_bitrate: u32,
    pub output_path: Option<String>,
    pub use_ffmpeg: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}

// ============ 格式转换 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    pub output_format: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResult {
    pub output_path: String,
    pub output_size: u64,
}

// ============ 音频压缩 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressOptions {
    pub mode: String,
    pub bitrate: Option<u32>,
    pub quality: Option<String>,
    pub sample_rate: Option<u32>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub output_path: String,
    pub output_size: u64,
    pub original_size: u64,
}

// ============ 音频合并 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOptions {
    pub input_paths: Vec<String>,
    pub output_format: String,
    pub bitrate: u32,
    pub mode: String,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}

// ============ 变速变调 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedChangeOptions {
    pub speed: f64,
    pub keep_pitch: bool,
    pub output_format: String,
    pub bitrate: u32,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedChangeResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}

// ============ 内部函数 ============

/// 快速探测音频元信息（不完整解码，仅读取头部/首帧）
fn probe_audio(path: &str) -> Result<(u32, u16, f64), String> {
    let fmt = guess_format(path);
    if fmt == "wav" {
        // WAV 头可直接读取完整信息
        let reader = hound::WavReader::open(path)
            .map_err(|e| format!("无法读取 WAV 文件: {}", e))?;
        let spec = reader.spec();
        let duration = reader.duration() as f64 / spec.sample_rate as f64;
        return Ok((spec.sample_rate, spec.channels, duration));
    }

    // 非 WAV 格式: 用 symphonia 探测首帧获取参数
    let file = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(file), Default::default());
    let hint = symphonia::core::probe::Hint::new();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .map_err(|e| format!("不支持的音频格式: {}", e))?;

    let track = probed.format.default_track().ok_or("未找到音频轨道")?;
    let params = track.codec_params.clone();
    let sample_rate = params.sample_rate.unwrap_or(44100);
    let channels = params.channels.map(|c| c.count() as u16).unwrap_or(2);

    // MP3 的 n_frames 可能为空，此时 duration = 0（前端用 generate_waveform 更新）
    let duration = params.n_frames.and_then(|n| {
        params.time_base.map(|tb| n as f64 * tb.numer as f64 / tb.denom as f64)
    }).unwrap_or(0.0);

    Ok((sample_rate, channels, duration))
}

/// 解码整个音频文件，返回 (PCM f32 samples, sample_rate, channels)
fn decode_audio_full(path: &str) -> Result<(Vec<f32>, u32, u16), String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("不支持的音频格式，仅支持 MP3/WAV/M4A: {}", e))?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("未找到音频轨道")?;
    let track_id = track.id;
    let codec_params = track.codec_params.clone();
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| format!("解码器初始化失败: {}", e))?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let spec = *decoded.spec();
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(buf.samples());
    }

    if all_samples.is_empty() {
        return Err("音频文件无有效数据".to_string());
    }

    Ok((all_samples, sample_rate, channels))
}

/// 解码音频文件的指定时间区间，返回 PCM f32 samples
fn decode_audio_segment(path: &str, start_time: f64, end_time: f64) -> Result<(Vec<f32>, u32, u16), String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("不支持的音频格式: {}", e))?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("未找到音频轨道")?;
    let track_id = track.id;
    let codec_params = track.codec_params.clone();
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| format!("解码器初始化失败: {}", e))?;

    let start_sample = (start_time * sample_rate as f64) as usize;
    let end_sample = (end_time * sample_rate as f64) as usize;
    let mut sample_count = 0usize;
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let spec = *decoded.spec();
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buf.copy_interleaved_ref(decoded);
        let samples = buf.samples();
        let frame_samples = samples.len() / channels as usize;

        let frame_start = sample_count;
        let frame_end = sample_count + frame_samples;

        if frame_end > start_sample && frame_start < end_sample {
            let clip_start = if frame_start < start_sample { start_sample - frame_start } else { 0 };
            let clip_end = if frame_end > end_sample { end_sample - frame_start } else { frame_samples };
            let clip_start_idx = clip_start * channels as usize;
            let clip_end_idx = clip_end * channels as usize;
            all_samples.extend_from_slice(&samples[clip_start_idx..clip_end_idx]);
        }

        sample_count = frame_end;
        if sample_count >= end_sample {
            break;
        }
    }

    if all_samples.is_empty() {
        return Err("裁剪区间无有效音频数据".to_string());
    }

    Ok((all_samples, sample_rate, channels))
}

/// PCM f32 → WAV 字节 (i16)
fn encode_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut buf = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut buf, spec)
            .map_err(|e| format!("WAV 编码器初始化失败: {}", e))?;
        for &sample in samples {
            let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(s).map_err(|e| format!("WAV 写入失败: {}", e))?;
        }
        writer.finalize().map_err(|e| format!("WAV 完成失败: {}", e))?;
    }
    Ok(buf.into_inner())
}

/// PCM f32 → MP3 字节
/// IMPORTANT: mp3lame-encoder 0.2.4 API 使用 Builder 模式（非 Encoder::new()）
/// encode 写入输出缓冲区，flush 收尾
fn encode_mp3(samples: &[f32], sample_rate: u32, channels: u16, bitrate: u32) -> Result<Vec<u8>, String> {
    use mp3lame_encoder::{Builder, Bitrate, InterleavedPcm, FlushNoGap};

    let mut builder = Builder::new()
        .ok_or_else(|| "MP3 编码器初始化失败".to_string())?;

    let br = match bitrate {
        128 => Bitrate::Kbps128,
        256 => Bitrate::Kbps256,
        320 => Bitrate::Kbps320,
        _ => Bitrate::Kbps192,
    };
    builder.set_brate(br).map_err(|e| format!("设置比特率失败: {}", e))?;
    builder.set_sample_rate(sample_rate).map_err(|e| format!("设置采样率失败: {}", e))?;
    builder.set_num_channels(channels as u8).map_err(|e| format!("设置声道失败: {}", e))?;

    let mut mp3_enc = builder.build().map_err(|e| format!("构建 MP3 编码器失败: {}", e))?;

    let i16_samples: Vec<i16> = samples.iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let input = InterleavedPcm(&i16_samples);
    let mut mp3_buf: Vec<u8> = Vec::new();
    let reserve_size = mp3lame_encoder::max_required_buffer_size(i16_samples.len());
    mp3_buf.reserve(reserve_size);

    let encoded = mp3_enc.encode(input, mp3_buf.spare_capacity_mut())
        .map_err(|e| format!("MP3 编码失败: {}", e))?;
    unsafe { mp3_buf.set_len(mp3_buf.len().wrapping_add(encoded)); }

    let tail = mp3_enc.flush::<FlushNoGap>(mp3_buf.spare_capacity_mut())
        .map_err(|e| format!("MP3 收尾失败: {}", e))?;
    unsafe { mp3_buf.set_len(mp3_buf.len().wrapping_add(tail)); }

    Ok(mp3_buf)
}

/// 从文件路径推断格式
fn guess_format(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".wav") || lower.ends_with(".wave") {
        "wav".to_string()
    } else if lower.ends_with(".mp3") {
        "mp3".to_string()
    } else if lower.ends_with(".m4a") || lower.ends_with(".mp4") {
        "m4a".to_string()
    } else {
        "unknown".to_string()
    }
}

// ============ ffmpeg 辅助 ============

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn ffmpeg_available() -> bool {
    crate::video_tools::ensure_ffmpeg_in_path();
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
    crate::video_tools::ensure_ffmpeg_in_path();
    std::process::Command::new("ffprobe")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_audio_info_via_ffprobe(path: &str) -> Result<AudioInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();
    let format = guess_format(path);

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

    let stream = &json["streams"][0];
    let sample_rate = stream["sample_rate"].as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(44100);
    let channels = stream["channels"].as_u64().unwrap_or(2) as u16;
    let bitrate = format_info["bit_rate"].as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .map(|b| b / 1000)
        .unwrap_or(0);

    Ok(AudioInfo { duration, sample_rate, channels, format, bitrate, file_size })
}

fn crop_via_ffmpeg(app_handle: &tauri::AppHandle, path: &str, options: &CropOptions) -> Result<CropResult, String> {
    let duration = options.end_time - options.start_time;
    let ext = &options.output_format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_cropped.{}", input_stem, ext))
    };

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 10.0 }));

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-ss").arg(format!("{:.3}", options.start_time))
        .arg("-t").arg(format!("{:.3}", duration))
        .arg("-i").arg(path)
        .creation_flags(CREATE_NO_WINDOW);

    match options.output_format.as_str() {
        "mp3" => {
            cmd.arg("-acodec").arg("libmp3lame")
                .arg("-b:a").arg(format!("{}k", options.mp3_bitrate));
        }
        "wav" => {
            cmd.arg("-acodec").arg("pcm_s16le");
        }
        _ => return Err("不支持的输出格式".to_string()),
    };

    cmd.arg(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 30.0 }));

    let output = cmd.output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg 裁剪失败: {}", stderr));
    }

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
    })
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn check_ffmpeg() -> bool {
    ffmpeg_available() && ffprobe_available()
}

#[tauri::command]
pub async fn get_audio_info(path: String, use_ffmpeg: bool) -> Result<AudioInfo, String> {
    tauri::async_runtime::spawn_blocking(move || do_get_audio_info(&path, use_ffmpeg))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_get_audio_info(path: &str, use_ffmpeg: bool) -> Result<AudioInfo, String> {
    if use_ffmpeg {
        return get_audio_info_via_ffprobe(path);
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();

    let format = guess_format(path);
    if format == "unknown" {
        return Err("不支持的音频格式，仅支持 MP3/WAV/M4A".to_string());
    }

    let (sample_rate, channels, duration) = probe_audio(path)?;

    let bitrate = if duration > 0.0 {
        ((file_size as f64 * 8.0) / duration / 1000.0) as u32
    } else {
        0
    };

    Ok(AudioInfo { duration, sample_rate, channels, format, bitrate, file_size })
}

#[tauri::command]
pub async fn generate_waveform(path: String) -> Result<WaveformData, String> {
    tauri::async_runtime::spawn_blocking(move || do_generate_waveform(&path))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_generate_waveform(path: &str) -> Result<WaveformData, String> {
    let (samples, sample_rate, _channels) = decode_audio_full(path)?;
    let channels = _channels as usize;
    let total_samples = samples.len() / channels;
    let duration = total_samples as f64 / sample_rate as f64;

    const TARGET_POINTS: usize = 2000;
    let step = (total_samples.max(1) as f64 / TARGET_POINTS as f64).max(1.0);
    let mut points = Vec::with_capacity(TARGET_POINTS);

    for i in 0..TARGET_POINTS {
        let start_idx = (i as f64 * step) as usize * channels;
        let end_idx = ((i as f64 + 1.0) * step) as usize * channels;
        let end_idx = end_idx.min(samples.len());

        if start_idx >= end_idx {
            points.push(0.0);
            continue;
        }

        let mut max_abs = 0.0f32;
        for j in (start_idx..end_idx).step_by(channels) {
            let v = samples[j].abs();
            if v > max_abs { max_abs = v; }
        }
        points.push(max_abs);
    }

    Ok(WaveformData { points, duration, sample_rate })
}

#[tauri::command]
pub async fn audio_crop(
    app_handle: tauri::AppHandle,
    path: String,
    options: CropOptions,
) -> Result<CropResult, String> {
    tauri::async_runtime::spawn_blocking(move || do_audio_crop(app_handle, &path, &options))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_audio_crop(app_handle: tauri::AppHandle, path: &str, options: &CropOptions) -> Result<CropResult, String> {
    if options.start_time < 0.0 || options.end_time <= options.start_time {
        return Err("起止时间非法".to_string());
    }
    let duration = options.end_time - options.start_time;
    if duration < 0.1 {
        return Err("裁剪区间不能小于 0.1 秒".to_string());
    }

    if options.use_ffmpeg {
        return crop_via_ffmpeg(&app_handle, path, options);
    }

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 10.0 }));

    let (samples, sample_rate, channels) = decode_audio_segment(path, options.start_time, options.end_time)?;

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 50.0 }));

    let output_bytes = match options.output_format.as_str() {
        "mp3" => encode_mp3(&samples, sample_rate, channels, options.mp3_bitrate)?,
        "wav" => encode_wav(&samples, sample_rate, channels)?,
        _ => return Err("不支持的输出格式".to_string()),
    };

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 80.0 }));

    // 确定输出路径：用户指定 > 源文件目录
    let ext = &options.output_format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_cropped.{}", input_stem, ext))
    };

    std::fs::write(&output_path, &output_bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let _ = app_handle.emit("audio-crop-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = output_bytes.len() as u64;

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
    })
}

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[tauri::command]
pub async fn get_audio_preview(
    path: String,
    start: f64,
    end: f64,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || do_get_audio_preview(&path, start, end))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_get_audio_preview(path: &str, start: f64, end: f64) -> Result<String, String> {
    let duration = end - start;
    if duration < 0.1 {
        return Err("预览区间不能小于 0.1 秒".to_string());
    }

    let (samples, sample_rate, channels) = decode_audio_segment(path, start, end)?;
    let wav_bytes = encode_wav(&samples, sample_rate, channels)?;
    Ok(BASE64.encode(&wav_bytes))
}

// ============ 格式转换实现 ============

fn convert_via_ffmpeg(app_handle: &tauri::AppHandle, path: &str, options: &ConvertOptions) -> Result<ConvertResult, String> {
    // 检查文件存在
    if !std::path::Path::new(path).exists() {
        return Err("输入文件不存在".to_string());
    }

    let ext = &options.output_format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_converted.{}", input_stem, ext))
    };

    let _ = app_handle.emit("audio-convert-progress", serde_json::json!({ "progress": 10.0 }));

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i").arg(path)
        .creation_flags(CREATE_NO_WINDOW);

    // 编码器选择
    match options.output_format.as_str() {
        "mp3" => {
            cmd.arg("-acodec").arg("libmp3lame");
            if let Some(br) = options.bitrate {
                cmd.arg("-b:a").arg(format!("{}k", br));
            }
        }
        "wav" => {
            cmd.arg("-acodec").arg("pcm_s16le");
        }
        "m4a" | "aac" => {
            cmd.arg("-acodec").arg("aac");
            if let Some(br) = options.bitrate {
                cmd.arg("-b:a").arg(format!("{}k", br));
            }
        }
        "flac" => {
            cmd.arg("-acodec").arg("flac");
            // FLAC 是无损格式，不支持比特率参数
        }
        "ogg" => {
            cmd.arg("-acodec").arg("libvorbis");
            // OGG Vorbis 使用质量参数更合适
            if let Some(br) = options.bitrate {
                cmd.arg("-b:a").arg(format!("{}k", br));
            }
        }
        _ => return Err("不支持的输出格式".to_string()),
    }

    // 采样率
    if let Some(sr) = options.sample_rate {
        cmd.arg("-ar").arg(sr.to_string());
    }

    // 声道
    if let Some(ch) = options.channels {
        cmd.arg("-ac").arg(ch.to_string());
    }

    cmd.arg(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-convert-progress", serde_json::json!({ "progress": 30.0 }));

    let output = cmd.output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 提取关键错误信息
        let error_msg = stderr.lines()
            .find(|line| line.contains("Error") || line.contains("error"))
            .unwrap_or(&stderr);
        return Err(format!("转换失败: {}", error_msg));
    }

    let _ = app_handle.emit("audio-convert-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    Ok(ConvertResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
    })
}

#[tauri::command]
pub async fn audio_convert(
    app_handle: tauri::AppHandle,
    path: String,
    options: ConvertOptions,
) -> Result<ConvertResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::video_tools::ensure_ffmpeg_in_path();
        convert_via_ffmpeg(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

// ============ 音频压缩实现 ============

fn compress_via_ffmpeg(app_handle: &tauri::AppHandle, path: &str, options: &CompressOptions) -> Result<CompressResult, String> {
    // 检查文件存在
    if !std::path::Path::new(path).exists() {
        return Err("输入文件不存在".to_string());
    }

    let format = guess_format(path);
    let ext = &format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_compressed.{}", input_stem, ext))
    };

    let original_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let _ = app_handle.emit("audio-compress-progress", serde_json::json!({ "progress": 10.0 }));

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i").arg(path)
        .creation_flags(CREATE_NO_WINDOW);

    // 根据模式设置参数
    match options.mode.as_str() {
        "bitrate" => {
            if let Some(br) = options.bitrate {
                match format.as_str() {
                    "mp3" => {
                        cmd.arg("-acodec").arg("libmp3lame");
                        cmd.arg("-b:a").arg(format!("{}k", br));
                    }
                    "m4a" | "aac" => {
                        cmd.arg("-acodec").arg("aac");
                        cmd.arg("-b:a").arg(format!("{}k", br));
                    }
                    "ogg" => {
                        cmd.arg("-acodec").arg("libvorbis");
                        cmd.arg("-b:a").arg(format!("{}k", br));
                    }
                    _ => {
                        cmd.arg("-b:a").arg(format!("{}k", br));
                    }
                }
            }
        }
        "quality" => {
            let quality = options.quality.as_deref().unwrap_or("medium");
            match format.as_str() {
                "mp3" => {
                    cmd.arg("-acodec").arg("libmp3lame");
                    // VBR 质量等级: 0=最好, 9=最差
                    let q = match quality {
                        "low" => "7",
                        "medium" => "5",
                        "high" => "2",
                        _ => "5",
                    };
                    cmd.arg("-q:a").arg(q);
                }
                _ => {
                    // 其他格式用比特率映射
                    let br = match quality {
                        "low" => 64,
                        "medium" => 128,
                        "high" => 192,
                        _ => 128,
                    };
                    cmd.arg("-b:a").arg(format!("{}k", br));
                }
            }
        }
        _ => return Err("不支持的压缩模式".to_string()),
    }

    // 采样率
    if let Some(sr) = options.sample_rate {
        cmd.arg("-ar").arg(sr.to_string());
    }

    cmd.arg(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-compress-progress", serde_json::json!({ "progress": 30.0 }));

    let output = cmd.output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = stderr.lines()
            .find(|line| line.contains("Error") || line.contains("error"))
            .unwrap_or(&stderr);
        return Err(format!("压缩失败: {}", error_msg));
    }

    let _ = app_handle.emit("audio-compress-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    Ok(CompressResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        original_size,
    })
}

#[tauri::command]
pub async fn audio_compress(
    app_handle: tauri::AppHandle,
    path: String,
    options: CompressOptions,
) -> Result<CompressResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::video_tools::ensure_ffmpeg_in_path();
        compress_via_ffmpeg(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

// ============ 音频合并实现 ============

fn merge_via_ffmpeg(app_handle: &tauri::AppHandle, options: &MergeOptions) -> Result<MergeResult, String> {
    if options.input_paths.is_empty() {
        return Err("至少需要两个音频文件".to_string());
    }

    // 检查所有文件存在
    for path in &options.input_paths {
        if !std::path::Path::new(path).exists() {
            return Err(format!("输入文件不存在: {}", path));
        }
    }

    let ext = &options.output_format;
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        let first_stem = std::path::Path::new(&options.input_paths[0])
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        std::path::Path::new(&options.input_paths[0])
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_merged.{}", first_stem, ext))
    };

    let _ = app_handle.emit("audio-merge-progress", serde_json::json!({ "progress": 10.0 }));

    // 创建临时 filelist.txt
    let temp_dir = std::env::temp_dir();
    let filelist_path = temp_dir.join(format!("litobox_merge_{}.txt", std::process::id()));

    let mut filelist_content = String::new();
    for path in &options.input_paths {
        // Windows 路径转换为 ffmpeg 兼容格式（正斜杠）
        let normalized_path = path.replace('\\', "/");
        filelist_content.push_str(&format!("file '{}'\n", normalized_path));
    }
    std::fs::write(&filelist_path, &filelist_content)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")
        .creation_flags(CREATE_NO_WINDOW);

    match options.mode.as_str() {
        "auto" => {
            // 检查是否所有文件格式相同
            let formats: Vec<String> = options.input_paths.iter()
                .map(|p| guess_format(p))
                .collect();
            let all_same = formats.iter().all(|f| f == &formats[0]) && formats[0] == *ext;

            if all_same {
                // 同格式 concat (快速)
                cmd.arg("-f").arg("concat")
                    .arg("-safe").arg("0")
                    .arg("-i").arg(&filelist_path)
                    .arg("-c").arg("copy");
            } else {
                // 跨格式转码合并
                let inputs: Vec<String> = options.input_paths.iter()
                    .map(|p| format!("-i {}", p))
                    .collect();
                cmd.args(inputs.join(" ").split_whitespace());

                let filter = format!("[0:a]{}", (1..options.input_paths.len())
                    .map(|i| format!("[{}:a]", i))
                    .collect::<Vec<_>>()
                    .join(""));
                let filter_complex = format!("{}concat=n={}:v=0:a=1[out]", filter, options.input_paths.len());
                cmd.arg("-filter_complex").arg(filter_complex)
                    .arg("-map").arg("[out]");
            }
        }
        "force_transcode" => {
            // 强制转码合并
            let inputs: Vec<String> = options.input_paths.iter()
                .map(|p| format!("-i {}", p))
                .collect();
            cmd.args(inputs.join(" ").split_whitespace());

            let filter_complex = format!("{}concat=n={}:v=0:a=1[out]",
                (0..options.input_paths.len())
                    .map(|i| format!("[{}:a]", i))
                    .collect::<Vec<_>>()
                    .join(""),
                options.input_paths.len()
            );
            cmd.arg("-filter_complex").arg(filter_complex)
                .arg("-map").arg("[out]");
        }
        _ => return Err("不支持的合并模式".to_string()),
    }

    // 输出编码器
    match options.output_format.as_str() {
        "mp3" => {
            if options.mode == "force_transcode" || options.mode == "auto" {
                cmd.arg("-acodec").arg("libmp3lame");
                cmd.arg("-b:a").arg(format!("{}k", options.bitrate));
            }
        }
        "wav" => {
            if options.mode == "force_transcode" || options.mode == "auto" {
                cmd.arg("-acodec").arg("pcm_s16le");
            }
        }
        "m4a" | "aac" => {
            if options.mode == "force_transcode" || options.mode == "auto" {
                cmd.arg("-acodec").arg("aac");
                cmd.arg("-b:a").arg(format!("{}k", options.bitrate));
            }
        }
        _ => {}
    }

    cmd.arg(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-merge-progress", serde_json::json!({ "progress": 30.0 }));

    let output = cmd.output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    // 清理临时文件
    let _ = std::fs::remove_file(&filelist_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg 合并失败: {}", stderr));
    }

    let _ = app_handle.emit("audio-merge-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    // 获取合并后时长
    let duration = if let Ok(info) = get_audio_info_via_ffprobe(&output_path.to_string_lossy()) {
        info.duration
    } else {
        0.0
    };

    Ok(MergeResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
    })
}

#[tauri::command]
pub async fn audio_merge(
    app_handle: tauri::AppHandle,
    options: MergeOptions,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::video_tools::ensure_ffmpeg_in_path();
        merge_via_ffmpeg(&app_handle, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

// ============ 变速变调实现 ============

fn speed_change_via_ffmpeg(app_handle: &tauri::AppHandle, path: &str, options: &SpeedChangeOptions) -> Result<SpeedChangeResult, String> {
    if options.speed < 0.5 || options.speed > 4.0 {
        return Err("速度必须在 0.5x 到 4.0x 之间".to_string());
    }

    // 检查文件存在
    if !std::path::Path::new(path).exists() {
        return Err("输入文件不存在".to_string());
    }

    let ext = &options.output_format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(format!("{}_{}x.{}", input_stem, options.speed, ext))
    };

    let _ = app_handle.emit("audio-speed-progress", serde_json::json!({ "progress": 10.0 }));

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i").arg(path)
        .creation_flags(CREATE_NO_WINDOW);

    // 构建滤镜
    let filter = if options.keep_pitch {
        // atempo 只支持 0.5~2.0，超出需链式
        if options.speed >= 0.5 && options.speed <= 2.0 {
            format!("atempo={}", options.speed)
        } else if options.speed > 2.0 && options.speed <= 4.0 {
            // 2.0x~4.0x = atempo=2.0,atempo=X
            let half = options.speed / 2.0;
            format!("atempo=2.0,atempo={}", half)
        } else {
            // 0.25x~0.5x = atempo=0.5,atempo=X
            let double = options.speed * 2.0;
            format!("atempo=0.5,atempo={}", double)
        }
    } else {
        // 不保持音调 (asetrate + aresample)
        // 需要获取原始采样率
        let info = get_audio_info_via_ffprobe(path)?;
        let new_rate = (info.sample_rate as f64 * options.speed) as u32;
        format!("asetrate={},aresample={}", new_rate, info.sample_rate)
    };

    cmd.arg("-filter:a").arg(filter);

    // 输出编码器
    match options.output_format.as_str() {
        "mp3" => {
            cmd.arg("-acodec").arg("libmp3lame");
            cmd.arg("-b:a").arg(format!("{}k", options.bitrate));
        }
        "wav" => {
            cmd.arg("-acodec").arg("pcm_s16le");
        }
        "m4a" | "aac" => {
            cmd.arg("-acodec").arg("aac");
            cmd.arg("-b:a").arg(format!("{}k", options.bitrate));
        }
        "flac" => {
            cmd.arg("-acodec").arg("flac");
            // FLAC 是无损格式，不支持比特率参数
        }
        "ogg" => {
            cmd.arg("-acodec").arg("libvorbis");
            cmd.arg("-b:a").arg(format!("{}k", options.bitrate));
        }
        _ => {}
    }

    cmd.arg(output_path.to_string_lossy().to_string());

    let _ = app_handle.emit("audio-speed-progress", serde_json::json!({ "progress": 30.0 }));

    let output = cmd.output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg 变速失败: {}", stderr));
    }

    let _ = app_handle.emit("audio-speed-progress", serde_json::json!({ "progress": 100.0 }));

    let output_size = std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    // 获取新时长
    let duration = if let Ok(info) = get_audio_info_via_ffprobe(&output_path.to_string_lossy()) {
        info.duration
    } else {
        0.0
    };

    Ok(SpeedChangeResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
    })
}

#[tauri::command]
pub async fn audio_speed_change(
    app_handle: tauri::AppHandle,
    path: String,
    options: SpeedChangeOptions,
) -> Result<SpeedChangeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::video_tools::ensure_ffmpeg_in_path();
        speed_change_via_ffmpeg(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

// ========== TTS 文字转语音 ==========

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("PowerShell 执行失败: {}", e))?;
    if !output.status.success() {
        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        debug_log!("[run_powershell] FAILED - exit code: {:?}", output.status);
        debug_log!("[run_powershell] stderr: {}", stderr);
        debug_log!("[run_powershell] stdout: {}", stdout);
        return Err(format!("PowerShell 错误: {}", stderr));
    }
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsVoice {
    pub name: String,
    pub language: String,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOptions {
    pub text: String,
    pub voice_name: Option<String>,
    pub rate: i32,
    pub volume: i32,
    pub engine: String,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub output_path: String,
    pub output_size: u64,
}

fn escape_ps_string(s: &str) -> String {
    s.replace('\'', "''")
}

fn tts_sapi_generate(text: &str, voice_name: Option<&str>, rate: i32, volume: i32, output_path: &str) -> Result<(), String> {
    if text.contains('\0') {
        return Err("文本中包含非法字符".to_string());
    }
    let escaped_text = escape_ps_string(text);
    let escaped_path = escape_ps_string(output_path);

    let voice_script = if let Some(vn) = voice_name {
        let escaped_vn = escape_ps_string(vn);
        format!(
            "$synth.SelectVoice('{}')\nWrite-Output \"SAPI_VOICE: $($synth.Voice.Name)\"",
            escaped_vn
        )
    } else {
        String::from("Write-Output \"SAPI_VOICE: $($synth.Voice.Name) (default)\"")
    };

    let script = format!(r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.Rate = {rate}
$synth.Volume = {volume}
{voice_script}
$synth.SetOutputToWaveFile('{escaped_path}')
$synth.Speak('{escaped_text}')
$synth.Dispose()
"#, rate = rate, volume = volume, voice_script = voice_script, escaped_path = escaped_path, escaped_text = escaped_text);

    let output = run_powershell(&script)?;
    debug_log!("[tts_sapi] 输出: {}", output.trim());
    Ok(())
}

fn tts_winrt_generate(text: &str, voice_name: Option<&str>, rate: i32, volume: i32, output_path: &str) -> Result<(), String> {
    debug_log!("[tts_winrt] 开始生成, text_len={}, voice={:?}, rate={}, output={}",
        text.len(), voice_name, rate, output_path);

    if text.contains('\0') {
        return Err("文本中包含非法字符".to_string());
    }
    let escaped_text = escape_ps_string(text);
    let escaped_path = escape_ps_string(output_path);

    let rate_norm = (rate as f64) / 10.0;

    let voice_filter = if let Some(vn) = voice_name {
        let escaped_vn = escape_ps_string(vn);
        format!(
            "$voice = $null; foreach ($v in [Windows.Media.SpeechSynthesis.SpeechSynthesizer]::AllVoices) {{ if ($v.DisplayName -eq '{}') {{ $voice = $v; break }} }}",
            escaped_vn
        )
    } else {
        String::from("$voice = $null")
    };

    let script = format!(r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Media.SpeechSynthesis.SpeechSynthesizer, Windows.Media.SpeechSynthesis, ContentType=WindowsRuntime]
Write-Output 'STEP: assemblies loaded'

$synth = New-Object Windows.Media.SpeechSynthesis.SpeechSynthesizer
Write-Output 'STEP: synth created'

# 选择语音
{voice_filter}
if ($voice) {{
    $synth.Voice = $voice
    Write-Output \"STEP: voice selected: $($voice.DisplayName)\"
}} else {{
    Write-Output 'STEP: using default voice'
}}

# 设置语速（WinRT 的 Rate 是相对值 0.5~2.0，1.0 为正常）
$synth.Options.Rate = {rate_norm}
Write-Output \"STEP: rate set to $($synth.Options.Rate)\"

# 合成
$asyncOp = $synth.SynthesizeTextToStreamAsync('{escaped_text}')
Write-Output 'STEP: starting synthesis...'

# 反射调用 AsTask<IAsyncOperation<SpeechSynthesisStream>>
$asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {{ $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }} | Select-Object -First 1)
Write-Output \"STEP: AsTask methods found: $($asTaskGeneric.Count)\"
$closedType = $asTaskGeneric.MakeGenericMethod([Windows.Media.SpeechSynthesis.SpeechSynthesisStream])
$taskResult = $closedType.Invoke($null, @($asyncOp))
Write-Output \"STEP: task type: $($taskResult.GetType().FullName)\"
$stream = $taskResult.Result
Write-Output \"STEP: stream obtained, size: $($stream.Size)\"

# 写入文件：将 WinRT 流转换为 .NET 流后复制
# 注意：扩展方法在 PowerShell 中需显式调用静态类
$netStream = [System.IO.WindowsRuntimeStreamExtensions]::AsStream($stream)
Write-Output \"STEP: netStream created, canRead: $($netStream.CanRead)\"
$fileStream = [System.IO.File]::Create('{escaped_path}')
$netStream.CopyTo($fileStream)
$fileStream.Flush()
$fileStream.Close()
$netStream.Dispose()
$stream.Dispose()
$synth.Dispose()
Write-Output 'STEP: done, file written'
"#, rate_norm = rate_norm, voice_filter = voice_filter, escaped_path = escaped_path, escaped_text = escaped_text);

    debug_log!("[tts_winrt] 执行 PowerShell 脚本");
    let output = run_powershell(&script)?;
    debug_log!("[tts_winrt] 脚本输出: {}", output.trim());

    Ok(())
}

#[tauri::command]
pub async fn list_tts_voices() -> Result<Vec<TtsVoice>, String> {
    let mut all_voices = Vec::new();

    // SAPI 语音
    match list_sapi_voices() {
        Ok(mut v) => all_voices.append(&mut v),
        Err(e) => debug_log!("[list_tts_voices] SAPI 失败: {}", e),
    }

    // WinRT 语音
    match list_winrt_voices() {
        Ok(mut v) => all_voices.append(&mut v),
        Err(e) => debug_log!("[list_tts_voices] WinRT 失败: {}", e),
    }

    Ok(all_voices)
}

fn list_sapi_voices() -> Result<Vec<TtsVoice>, String> {
    let script = r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.GetInstalledVoices() | ForEach-Object {
    @{
        name = $_.VoiceInfo.Name
        language = $_.VoiceInfo.Culture.DisplayName
        engine = 'sapi'
    }
} | ConvertTo-Json -Compress
"#;
    let output = run_powershell(script)?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    let voices: Vec<TtsVoice> = serde_json::from_str(trimmed)
        .map_err(|e| format!("解析 SAPI 语音列表失败: {}", e))?;
    Ok(voices)
}

fn list_winrt_voices() -> Result<Vec<TtsVoice>, String> {
    let script = r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Media.SpeechSynthesis.SpeechSynthesizer, Windows.Media.SpeechSynthesis, ContentType=WindowsRuntime]
$voices = [Windows.Media.SpeechSynthesis.SpeechSynthesizer]::AllVoices
$voices | ForEach-Object {
    @{
        name = $_.DisplayName
        language = $_.Language
        engine = 'winrt'
    }
} | ConvertTo-Json -Compress
"#;
    let output = run_powershell(script)?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    let voices: Vec<TtsVoice> = serde_json::from_str(trimmed)
        .map_err(|e| format!("解析 WinRT 语音列表失败: {}", e))?;
    Ok(voices)
}

#[tauri::command]
pub async fn get_downloads_dir() -> Result<String, String> {
    let script = r#"
$downloads = [Environment]::GetFolderPath('UserProfile') + '\Downloads'
if (Test-Path $downloads) {
    $downloads
} else {
    [Environment]::GetFolderPath('MyDocuments')
}
"#;
    let output = run_powershell(script)?;
    let trimmed = output.trim().to_string();
    if trimmed.is_empty() {
        return Err("无法获取下载目录".to_string());
    }
    Ok(trimmed)
}

#[tauri::command]
pub async fn tts_generate(
    options: TtsOptions,
) -> Result<TtsResult, String> {
    if options.text.trim().is_empty() {
        return Err("请输入要转换的文字".to_string());
    }
    if options.rate < -10 || options.rate > 10 {
        return Err("语速范围 -10 到 10".to_string());
    }
    if options.volume < 0 || options.volume > 100 {
        return Err("音量范围 0 到 100".to_string());
    }

    let output_path = if let Some(ref custom_path) = options.output_path {
        std::path::PathBuf::from(custom_path)
    } else {
        let downloads_dir = get_downloads_dir().await.unwrap_or_else(|_| {
            std::env::temp_dir().to_string_lossy().to_string()
        });
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        std::path::PathBuf::from(&downloads_dir).join(format!("tts_{}.wav", timestamp))
    };

    let output_path_str = output_path.to_string_lossy().to_string();
    let voice_name: Option<String> = options.voice_name.clone();
    let text = options.text.clone();
    let rate = options.rate;
    let volume = options.volume;
    let engine = options.engine.clone();
    let path_clone = output_path_str.clone();

    tauri::async_runtime::spawn_blocking(move || {
        match engine.as_str() {
            "winrt" => {
                match tts_winrt_generate(&text, voice_name.as_deref(), rate, volume, &path_clone) {
                    Ok(_) => {
                        debug_log!("[tts_generate] WinRT 引擎生成成功");
                    }
                    Err(e) => {
                        debug_log!("[tts_generate] WinRT 失败({})，回退到 SAPI 引擎", e);
                        tts_sapi_generate(&text, None, rate, volume, &path_clone)?;
                    }
                }
            }
            _ => {
                tts_sapi_generate(&text, voice_name.as_deref(), rate, volume, &path_clone)?;
            }
        }
        let output_size = std::fs::metadata(&path_clone)
            .map(|m| m.len())
            .map_err(|e| format!("无法读取输出文件: {}", e))?;
        Ok(TtsResult {
            output_path: path_clone,
            output_size,
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}