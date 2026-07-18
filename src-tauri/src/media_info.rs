use serde::{Deserialize, Serialize};
use std::process::Command;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfoResult {
    pub structured: StructuredMediaInfo,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredMediaInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub format: FormatInfo,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub subtitle_streams: Vec<SubtitleStreamInfo>,
    pub other_streams: Vec<OtherStreamInfo>,
    pub metadata: Vec<KeyValue>,
    pub chapters: Vec<ChapterInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_name: String,
    pub format_long_name: String,
    pub duration: f64,
    pub bitrate: u64,
    pub stream_count: u32,
    pub nb_programs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub level: Option<i64>,
    pub width: u32,
    pub height: u32,
    pub coded_width: u32,
    pub coded_height: u32,
    pub display_aspect_ratio: String,
    pub sample_aspect_ratio: String,
    pub pix_fmt: String,
    pub color_space: String,
    pub color_primaries: String,
    pub color_transfer: String,
    pub color_range: String,
    pub chroma_location: String,
    pub field_order: String,
    pub fps: f64,
    pub avg_fps: f64,
    pub bitrate: u64,
    pub bit_depth: Option<u32>,
    pub duration: f64,
    pub nb_frames: u64,
    pub disposition: Vec<String>,
    pub tags: Vec<KeyValue>,
    pub extra: Vec<KeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: String,
    pub profile: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub channel_layout: String,
    pub sample_fmt: String,
    pub bit_depth: Option<u32>,
    pub bitrate: u64,
    pub duration: f64,
    pub nb_frames: u64,
    pub disposition: Vec<String>,
    pub tags: Vec<KeyValue>,
    pub extra: Vec<KeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStreamInfo {
    pub index: u32,
    pub codec_name: String,
    pub codec_long_name: String,
    pub tags: Vec<KeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherStreamInfo {
    pub index: u32,
    pub codec_type: String,
    pub codec_name: String,
    pub tags: Vec<KeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterInfo {
    pub id: u64,
    pub start_time: f64,
    pub end_time: f64,
    pub title: String,
    pub tags: Vec<KeyValue>,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[tauri::command]
pub fn get_media_info(path: String) -> Result<MediaInfoResult, String> {
    // 检查文件是否存在
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("无法访问文件: {}", e))?;

    let file_size = metadata.len();
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // 调用 ffprobe
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            "-show_chapters",
            &path
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("ffprobe 执行失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe 错误: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw = stdout.to_string();

    // 解析 JSON
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    // 提取结构化数据
    let structured = parse_media_info(&json, &path, &file_name, file_size)?;

    Ok(MediaInfoResult { structured, raw })
}

fn parse_media_info(
    json: &Value,
    path: &str,
    file_name: &str,
    file_size: u64,
) -> Result<StructuredMediaInfo, String> {
    // 解析 format
    let format_info = parse_format(&json["format"])?;

    // 解析 streams
    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();
    let mut subtitle_streams = Vec::new();
    let mut other_streams = Vec::new();

    if let Some(streams) = json["streams"].as_array() {
        for stream in streams {
            let codec_type = stream["codec_type"].as_str().unwrap_or("unknown");
            match codec_type {
                "video" => video_streams.push(parse_video_stream(stream)?),
                "audio" => audio_streams.push(parse_audio_stream(stream)?),
                "subtitle" => subtitle_streams.push(parse_subtitle_stream(stream)?),
                _ => other_streams.push(parse_other_stream(stream)?),
            }
        }
    }

    // 解析 metadata
    let metadata = parse_tags(&json["format"]["tags"]);

    // 解析 chapters
    let mut chapters = Vec::new();
    if let Some(chaps) = json["chapters"].as_array() {
        for chap in chaps {
            chapters.push(parse_chapter(chap)?);
        }
    }

    Ok(StructuredMediaInfo {
        file_path: path.to_string(),
        file_name: file_name.to_string(),
        file_size,
        format: format_info,
        video_streams,
        audio_streams,
        subtitle_streams,
        other_streams,
        metadata,
        chapters,
    })
}

fn parse_format(format: &Value) -> Result<FormatInfo, String> {
    Ok(FormatInfo {
        format_name: format["format_name"].as_str().unwrap_or("").to_string(),
        format_long_name: format["format_long_name"].as_str().unwrap_or("").to_string(),
        duration: format["duration"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        bitrate: format["bit_rate"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        stream_count: format["nb_streams"].as_u64().unwrap_or(0) as u32,
        nb_programs: format["nb_programs"].as_u64().unwrap_or(0) as u32,
    })
}

fn parse_video_stream(stream: &Value) -> Result<VideoStreamInfo, String> {
    let known_keys = [
        "index", "codec_name", "codec_long_name", "profile", "level",
        "width", "height", "coded_width", "coded_height",
        "display_aspect_ratio", "sample_aspect_ratio", "pix_fmt",
        "color_space", "color_primaries", "color_transfer", "color_range",
        "chroma_location", "field_order", "r_frame_rate", "avg_frame_rate",
        "bit_rate", "bits_per_raw_sample", "duration", "nb_frames",
        "disposition", "tags", "codec_type", "codec_time_base",
        "codec_tag_string", "codec_tag", "start_time", "start_pts",
        "duration_ts", "nb_read_frames", "nb_read_packets",
    ];

    let mut extra = Vec::new();
    if let Some(obj) = stream.as_object() {
        for (key, value) in obj {
            if !known_keys.contains(&key.as_str()) {
                extra.push(KeyValue {
                    key: key.clone(),
                    value: value_to_string(value),
                });
            }
        }
    }

    Ok(VideoStreamInfo {
        index: stream["index"].as_u64().unwrap_or(0) as u32,
        codec_name: stream["codec_name"].as_str().unwrap_or("").to_string(),
        codec_long_name: stream["codec_long_name"].as_str().unwrap_or("").to_string(),
        profile: stream["profile"].as_str().unwrap_or("").to_string(),
        level: stream["level"].as_i64(),
        width: stream["width"].as_u64().unwrap_or(0) as u32,
        height: stream["height"].as_u64().unwrap_or(0) as u32,
        coded_width: stream["coded_width"].as_u64().unwrap_or(0) as u32,
        coded_height: stream["coded_height"].as_u64().unwrap_or(0) as u32,
        display_aspect_ratio: stream["display_aspect_ratio"].as_str().unwrap_or("").to_string(),
        sample_aspect_ratio: stream["sample_aspect_ratio"].as_str().unwrap_or("").to_string(),
        pix_fmt: stream["pix_fmt"].as_str().unwrap_or("").to_string(),
        color_space: stream["color_space"].as_str().unwrap_or("").to_string(),
        color_primaries: stream["color_primaries"].as_str().unwrap_or("").to_string(),
        color_transfer: stream["color_transfer"].as_str().unwrap_or("").to_string(),
        color_range: stream["color_range"].as_str().unwrap_or("").to_string(),
        chroma_location: stream["chroma_location"].as_str().unwrap_or("").to_string(),
        field_order: stream["field_order"].as_str().unwrap_or("").to_string(),
        fps: parse_frame_rate(stream["r_frame_rate"].as_str().unwrap_or("0/1")),
        avg_fps: parse_frame_rate(stream["avg_frame_rate"].as_str().unwrap_or("0/1")),
        bitrate: stream["bit_rate"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bit_depth: stream["bits_per_raw_sample"].as_str()
            .and_then(|s| s.parse().ok()),
        duration: stream["duration"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        nb_frames: stream["nb_frames"].as_u64().unwrap_or(0),
        disposition: parse_disposition(&stream["disposition"]),
        tags: parse_tags(&stream["tags"]),
        extra,
    })
}

fn parse_audio_stream(stream: &Value) -> Result<AudioStreamInfo, String> {
    let known_keys = [
        "index", "codec_name", "codec_long_name", "profile",
        "sample_rate", "channels", "channel_layout", "sample_fmt",
        "bits_per_raw_sample", "bit_rate", "duration", "nb_frames",
        "disposition", "tags", "codec_type", "codec_time_base",
        "codec_tag_string", "codec_tag", "start_time", "start_pts",
        "duration_ts", "nb_read_frames", "nb_read_packets",
    ];

    let mut extra = Vec::new();
    if let Some(obj) = stream.as_object() {
        for (key, value) in obj {
            if !known_keys.contains(&key.as_str()) {
                extra.push(KeyValue {
                    key: key.clone(),
                    value: value_to_string(value),
                });
            }
        }
    }

    Ok(AudioStreamInfo {
        index: stream["index"].as_u64().unwrap_or(0) as u32,
        codec_name: stream["codec_name"].as_str().unwrap_or("").to_string(),
        codec_long_name: stream["codec_long_name"].as_str().unwrap_or("").to_string(),
        profile: stream["profile"].as_str().unwrap_or("").to_string(),
        sample_rate: stream["sample_rate"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        channels: stream["channels"].as_u64().unwrap_or(0) as u32,
        channel_layout: stream["channel_layout"].as_str().unwrap_or("").to_string(),
        sample_fmt: stream["sample_fmt"].as_str().unwrap_or("").to_string(),
        bit_depth: stream["bits_per_raw_sample"].as_str()
            .and_then(|s| s.parse().ok()),
        bitrate: stream["bit_rate"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        duration: stream["duration"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        nb_frames: stream["nb_frames"].as_u64().unwrap_or(0),
        disposition: parse_disposition(&stream["disposition"]),
        tags: parse_tags(&stream["tags"]),
        extra,
    })
}

fn parse_subtitle_stream(stream: &Value) -> Result<SubtitleStreamInfo, String> {
    Ok(SubtitleStreamInfo {
        index: stream["index"].as_u64().unwrap_or(0) as u32,
        codec_name: stream["codec_name"].as_str().unwrap_or("").to_string(),
        codec_long_name: stream["codec_long_name"].as_str().unwrap_or("").to_string(),
        tags: parse_tags(&stream["tags"]),
    })
}

fn parse_other_stream(stream: &Value) -> Result<OtherStreamInfo, String> {
    Ok(OtherStreamInfo {
        index: stream["index"].as_u64().unwrap_or(0) as u32,
        codec_type: stream["codec_type"].as_str().unwrap_or("unknown").to_string(),
        codec_name: stream["codec_name"].as_str().unwrap_or("").to_string(),
        tags: parse_tags(&stream["tags"]),
    })
}

fn parse_chapter(chapter: &Value) -> Result<ChapterInfo, String> {
    let title = chapter["tags"]["title"].as_str().unwrap_or("").to_string();

    Ok(ChapterInfo {
        id: chapter["id"].as_u64().unwrap_or(0),
        start_time: chapter["start_time"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        end_time: chapter["end_time"].as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0),
        title,
        tags: parse_tags(&chapter["tags"]),
    })
}

fn parse_frame_rate(rate_str: &str) -> f64 {
    if let Some((num, den)) = rate_str.split_once('/') {
        let n = num.parse::<f64>().unwrap_or(0.0);
        let d = den.parse::<f64>().unwrap_or(1.0);
        if d > 0.0 { n / d } else { 0.0 }
    } else {
        0.0
    }
}

fn parse_disposition(disp: &Value) -> Vec<String> {
    let mut result = Vec::new();
    if let Some(obj) = disp.as_object() {
        for (key, value) in obj {
            if value.as_i64() == Some(1) {
                result.push(key.clone());
            }
        }
    }
    result
}

fn parse_tags(tags: &Value) -> Vec<KeyValue> {
    let mut result = Vec::new();
    if let Some(obj) = tags.as_object() {
        for (key, value) in obj {
            result.push(KeyValue {
                key: key.clone(),
                value: value_to_string(value),
            });
        }
    }
    result
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "".to_string(),
        _ => value.to_string(),
    }
}
