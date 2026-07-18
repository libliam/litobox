use serde::{Deserialize, Serialize};

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
