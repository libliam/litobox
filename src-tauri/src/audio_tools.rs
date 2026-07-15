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