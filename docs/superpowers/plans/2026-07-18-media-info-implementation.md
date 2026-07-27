# 媒体信息查看器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建一个独立的媒体信息查看工具，使用 ffprobe 深度分析音视频文件，展示所有可提取的元数据和技术参数。

**Architecture:** 后端新增 `media_info.rs` 模块，调用 ffprobe 获取完整 JSON 输出，解析为结构化数据返回前端。前端创建 `MediaInfoTool.vue` 页面，以卡片形式展示容器信息、视频流、音频流、字幕流、元数据、章节等，并提供原始 JSON 视图。

**Tech Stack:** Tauri 2.0 (Rust), Vue 3 + TypeScript, ffprobe, serde_json

**Spec:** `docs/superpowers/specs/2026-07-18-media-info-design.md`

---

## File Structure

```
src-tauri/src/
  ├── media_info.rs (新增) - 后端命令实现，ffprobe 调用与数据解析
  
src/views/
  ├── MediaInfoTool.vue (新增) - 前端页面，信息展示与交互

src/
  ├── App.vue (修改) - 导入组件并注册到 toolComponentMap
  ├── store/index.ts (修改) - 添加 mediaInfo 菜单项
  ├── views/WorkflowView.vue (修改) - 添加 mediaInfo 工作流分支
```

---

## Task 1: 后端数据结构定义

**Files:**
- Create: `src-tauri/src/media_info.rs`

- [ ] **Step 1: 创建 media_info.rs 文件，定义基础数据结构**

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过（结构体定义无语法错误）

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/media_info.rs
git commit -m "feat(media-info): 定义后端数据结构"
```

---

## Task 2: 后端 ffprobe 调用与 JSON 解析

**Files:**
- Modify: `src-tauri/src/media_info.rs`

- [ ] **Step 1: 添加 ffprobe 调用函数**

在 `media_info.rs` 末尾添加：

```rust
use std::process::Command;
use serde_json::Value;

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
        "duration_ts", "bit_rate", "nb_read_frames", "nb_read_packets",
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/media_info.rs
git commit -m "feat(media-info): 实现 ffprobe 调用与 JSON 解析"
```

---

## Task 3: 注册后端命令

**Files:**
- Modify: `src-tauri/src/main.rs:1-19` (模块声明)
- Modify: `src-tauri/src/main.rs:29-175` (命令注册)

- [ ] **Step 1: 在 main.rs 顶部添加模块声明**

在第 18 行（`mod pdf_tools;` 之后）添加：

```rust
mod media_info;
```

- [ ] **Step 2: 在 invoke_handler 中注册命令**

在 `invoke_handler` 的最后一行（如 `pdf_tools::xxx` 之后）添加：

```rust
media_info::get_media_info,
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(media-info): 注册 get_media_info 命令"
```

---

## Task 4: 前端页面创建

**Files:**
- Create: `src/views/MediaInfoTool.vue`

- [ ] **Step 1: 创建 MediaInfoTool.vue 基础结构**

```vue
<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': ffmpegAvailable, 'ffmpeg-missing': !ffmpegAvailable }" v-if="ffmpegChecked">
      <template v-if="ffmpegAvailable">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，所有功能可用
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        此功能需要 ffmpeg，请先安装。
        <span class="ffmpeg-tip">
          安装命令：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
        </span>
      </template>
    </div>

    <!-- 文件选择卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">选择媒体文件</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" @click="selectFile" :loading="isLoading" :disabled="!ffmpegAvailable">
              选择文件
            </el-button>
            <el-button size="small" @click="clearInfo" v-if="mediaInfo">
              清除
            </el-button>
          </div>
        </div>
        <div v-if="mediaInfo" class="file-info">
          <span class="file-name">{{ mediaInfo.structured.file_name }}</span>
          <span class="file-detail">{{ formatFileSize(mediaInfo.structured.file_size) }}</span>
        </div>
      </div>
    </div>

    <!-- 错误信息 -->
    <div v-if="errorMessage" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ errorMessage }}</div>
      </div>
    </div>

    <!-- 容器信息 -->
    <div v-if="mediaInfo" class="tool-card">
      <div class="card-header">
        <span class="card-title">容器信息</span>
        <div class="card-actions">
          <el-button size="small" @click="copyFormatInfo">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">格式名称</span>
            <span class="info-value">{{ mediaInfo.structured.format.format_long_name || mediaInfo.structured.format.format_name }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(mediaInfo.structured.format.duration) }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">文件大小</span>
            <span class="info-value">{{ formatFileSize(mediaInfo.structured.file_size) }} ({{ mediaInfo.structured.file_size.toLocaleString() }} bytes)</span>
          </div>
          <div class="info-row">
            <span class="info-key">总比特率</span>
            <span class="info-value">{{ (mediaInfo.structured.format.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row">
            <span class="info-key">流数量</span>
            <span class="info-value">{{ mediaInfo.structured.format.stream_count }} (视频×{{ mediaInfo.structured.video_streams.length }} + 音频×{{ mediaInfo.structured.audio_streams.length }} + 字幕×{{ mediaInfo.structured.subtitle_streams.length }})</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 视频流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.video_streams" :key="'video-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">视频流 #{{ stream.index }}</span>
        <div class="card-actions">
          <el-button size="small" @click="copyVideoStreamInfo(stream)">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}{{ stream.profile ? ` (${stream.profile}${stream.level ? `, Level ${stream.level}` : ''})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">分辨率</span>
            <span class="info-value">{{ stream.width }}×{{ stream.height }}{{ stream.display_aspect_ratio ? ` (${stream.display_aspect_ratio})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">帧率</span>
            <span class="info-value">{{ stream.fps.toFixed(3) }} fps</span>
          </div>
          <div class="info-row">
            <span class="info-key">像素格式</span>
            <span class="info-value">{{ stream.pix_fmt }}{{ stream.bit_depth ? ` (${stream.bit_depth} bit)` : '' }}</span>
          </div>
          <div class="info-row" v-if="stream.color_space || stream.color_primaries || stream.color_transfer">
            <span class="info-key">色彩空间</span>
            <span class="info-value">{{ stream.color_space || '未知' }} / {{ stream.color_primaries || '未知' }} / {{ stream.color_transfer || '未知' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">比特率</span>
            <span class="info-value">{{ (stream.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row" v-if="stream.nb_frames > 0">
            <span class="info-key">帧数</span>
            <span class="info-value">{{ stream.nb_frames.toLocaleString() }}</span>
          </div>
          <div class="info-row" v-if="stream.duration > 0">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(stream.duration) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'vtag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'vext-' + eidx">
            <span class="info-key">{{ ext.key }}</span>
            <span class="info-value">{{ ext.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 音频流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.audio_streams" :key="'audio-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">音频流 #{{ stream.index }}</span>
        <div class="card-actions">
          <el-button size="small" @click="copyAudioStreamInfo(stream)">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}{{ stream.profile ? ` (${stream.profile})` : '' }}</span>
          </div>
          <div class="info-row">
            <span class="info-key">采样率</span>
            <span class="info-value">{{ stream.sample_rate.toLocaleString() }} Hz</span>
          </div>
          <div class="info-row">
            <span class="info-key">声道</span>
            <span class="info-value">{{ stream.channel_layout || (stream.channels === 2 ? '立体声' : stream.channels === 1 ? '单声道' : `${stream.channels} 声道`) }}</span>
          </div>
          <div class="info-row" v-if="stream.bit_depth">
            <span class="info-key">位深度</span>
            <span class="info-value">{{ stream.bit_depth }} bit</span>
          </div>
          <div class="info-row">
            <span class="info-key">比特率</span>
            <span class="info-value">{{ (stream.bitrate / 1000).toFixed(0) }} kbps</span>
          </div>
          <div class="info-row" v-if="stream.duration > 0">
            <span class="info-key">时长</span>
            <span class="info-value">{{ formatDuration(stream.duration) }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'atag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
          <div class="info-row" v-for="(ext, eidx) in stream.extra" :key="'aext-' + eidx">
            <span class="info-key">{{ ext.key }}</span>
            <span class="info-value">{{ ext.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 字幕流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.subtitle_streams" :key="'sub-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">字幕流 #{{ stream.index }}</span>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_long_name || stream.codec_name }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'stag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 其他流信息 -->
    <div v-for="(stream, idx) in mediaInfo?.structured.other_streams" :key="'other-' + idx" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ stream.codec_type }} 流 #{{ stream.index }}</span>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row">
            <span class="info-key">编解码器</span>
            <span class="info-value">{{ stream.codec_name }}</span>
          </div>
          <div class="info-row" v-for="(tag, tidx) in stream.tags" :key="'otag-' + tidx">
            <span class="info-key">{{ tag.key }}</span>
            <span class="info-value">{{ tag.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 元数据 -->
    <div v-if="mediaInfo && mediaInfo.structured.metadata.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">元数据</span>
        <div class="card-actions">
          <el-button size="small" @click="copyMetadata">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="info-grid">
          <div class="info-row" v-for="(meta, idx) in mediaInfo.structured.metadata" :key="'meta-' + idx">
            <span class="info-key">{{ meta.key }}</span>
            <span class="info-value">{{ meta.value }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 章节信息 -->
    <div v-if="mediaInfo && mediaInfo.structured.chapters.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">章节</span>
      </div>
      <div class="card-body">
        <div class="chapter-list">
          <div v-for="(chap, idx) in mediaInfo.structured.chapters" :key="'chap-' + idx" class="chapter-item">
            <span class="chapter-time">{{ formatDuration(chap.start_time) }} - {{ formatDuration(chap.end_time) }}</span>
            <span class="chapter-title">{{ chap.title || `章节 ${idx + 1}` }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 原始 JSON -->
    <div v-if="mediaInfo" class="tool-card">
      <div class="card-header">
        <span class="card-title">原始 JSON</span>
        <div class="card-actions">
          <el-button size="small" @click="toggleRawJson">
            {{ rawJsonVisible ? '收起' : '展开' }}
          </el-button>
          <el-button size="small" @click="copyRawJson">复制</el-button>
        </div>
      </div>
      <div class="card-body" v-if="rawJsonVisible">
        <pre class="raw-json">{{ mediaInfo.raw }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

interface KeyValue {
  key: string
  value: string
}

interface FormatInfo {
  format_name: string
  format_long_name: string
  duration: number
  bitrate: number
  stream_count: number
  nb_programs: number
}

interface VideoStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  profile: string
  level: number | null
  width: number
  height: number
  coded_width: number
  coded_height: number
  display_aspect_ratio: string
  sample_aspect_ratio: string
  pix_fmt: string
  color_space: string
  color_primaries: string
  color_transfer: string
  color_range: string
  chroma_location: string
  field_order: string
  fps: number
  avg_fps: number
  bitrate: number
  bit_depth: number | null
  duration: number
  nb_frames: number
  disposition: string[]
  tags: KeyValue[]
  extra: KeyValue[]
}

interface AudioStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  profile: string
  sample_rate: number
  channels: number
  channel_layout: string
  sample_fmt: string
  bit_depth: number | null
  bitrate: number
  duration: number
  nb_frames: number
  disposition: string[]
  tags: KeyValue[]
  extra: KeyValue[]
}

interface SubtitleStreamInfo {
  index: number
  codec_name: string
  codec_long_name: string
  tags: KeyValue[]
}

interface OtherStreamInfo {
  index: number
  codec_type: string
  codec_name: string
  tags: KeyValue[]
}

interface ChapterInfo {
  id: number
  start_time: number
  end_time: number
  title: string
  tags: KeyValue[]
}

interface StructuredMediaInfo {
  file_path: string
  file_name: string
  file_size: number
  format: FormatInfo
  video_streams: VideoStreamInfo[]
  audio_streams: AudioStreamInfo[]
  subtitle_streams: SubtitleStreamInfo[]
  other_streams: OtherStreamInfo[]
  metadata: KeyValue[]
  chapters: ChapterInfo[]
}

interface MediaInfoResult {
  structured: StructuredMediaInfo
  raw: string
}

const ffmpegChecked = ref(false)
const ffmpegAvailable = ref(false)
const isLoading = ref(false)
const mediaInfo = ref<MediaInfoResult | null>(null)
const errorMessage = ref('')
const rawJsonVisible = ref(false)

onMounted(async () => {
  await checkFfmpeg()
})

async function checkFfmpeg() {
  try {
    const available = await invoke<boolean>('check_ffmpeg_available')
    ffmpegAvailable.value = available
  } catch (e) {
    ffmpegAvailable.value = false
  } finally {
    ffmpegChecked.value = true
  }
}

async function selectFile() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: '媒体文件',
      extensions: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', 'wmv', 'mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'm4v']
    }]
  })
  
  if (!selected) return
  
  await loadMediaInfo(selected as string)
}

async function loadMediaInfo(path: string) {
  isLoading.value = true
  errorMessage.value = ''
  mediaInfo.value = null
  
  try {
    const result = await invoke<MediaInfoResult>('get_media_info', { path })
    mediaInfo.value = result
    
    // 记录历史
    const format = result.structured.format.format_name.split(',')[0].toUpperCase()
    const videoInfo = result.structured.video_streams[0]
    const resolution = videoInfo ? `${videoInfo.width}x${videoInfo.height}` : ''
    const codec = videoInfo ? videoInfo.codec_name.toUpperCase() : ''
    const duration = formatDuration(result.structured.format.duration)
    
    store.addHistory({
      tool: 'mediaInfo',
      action: '查看媒体信息',
      inputPreview: result.structured.file_name.slice(0, 50),
      outputPreview: `${format} | ${resolution} | ${codec} | ${duration}`,
      inputFull: path,
      outputFull: JSON.stringify(result.structured, null, 2),
    })
    
    ElMessage.success('媒体信息加载成功')
  } catch (e) {
    errorMessage.value = String(e)
    ElMessage.error('加载失败')
  } finally {
    isLoading.value = false
  }
}

function clearInfo() {
  mediaInfo.value = null
  errorMessage.value = ''
  rawJsonVisible.value = false
}

function toggleRawJson() {
  rawJsonVisible.value = !rawJsonVisible.value
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

function formatDuration(seconds: number): string {
  if (seconds <= 0) return '00:00:00'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 1000)
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(3, '0')}`
}

function formatStreamInfo(stream: any, type: string): string {
  const lines: string[] = []
  lines.push(`类型: ${type}`)
  lines.push(`索引: #${stream.index}`)
  lines.push(`编解码器: ${stream.codec_long_name || stream.codec_name}`)
  
  if (type === '视频流') {
    lines.push(`分辨率: ${stream.width}×${stream.height}`)
    lines.push(`帧率: ${stream.fps.toFixed(3)} fps`)
    lines.push(`像素格式: ${stream.pix_fmt}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  } else if (type === '音频流') {
    lines.push(`采样率: ${stream.sample_rate} Hz`)
    lines.push(`声道: ${stream.channel_layout || stream.channels}`)
    lines.push(`比特率: ${(stream.bitrate / 1000).toFixed(0)} kbps`)
  }
  
  if (stream.profile) lines.push(`Profile: ${stream.profile}`)
  if (stream.tags && stream.tags.length > 0) {
    lines.push('标签:')
    stream.tags.forEach((t: KeyValue) => lines.push(`  ${t.key}: ${t.value}`))
  }
  if (stream.extra && stream.extra.length > 0) {
    lines.push('其他:')
    stream.extra.forEach((t: KeyValue) => lines.push(`  ${t.key}: ${t.value}`))
  }
  
  return lines.join('\n')
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch (e) {
    ElMessage.error('复制失败')
  }
}

function copyFormatInfo() {
  if (!mediaInfo.value) return
  const f = mediaInfo.value.structured.format
  const text = `格式: ${f.format_long_name || f.format_name}
时长: ${formatDuration(f.duration)}
大小: ${formatFileSize(mediaInfo.value.structured.file_size)}
比特率: ${(f.bitrate / 1000).toFixed(0)} kbps
流数量: ${f.stream_count}`
  copyToClipboard(text)
}

function copyVideoStreamInfo(stream: VideoStreamInfo) {
  copyToClipboard(formatStreamInfo(stream, '视频流'))
}

function copyAudioStreamInfo(stream: AudioStreamInfo) {
  copyToClipboard(formatStreamInfo(stream, '音频流'))
}

function copyMetadata() {
  if (!mediaInfo.value) return
  const lines = mediaInfo.value.structured.metadata.map(m => `${m.key}: ${m.value}`)
  copyToClipboard(lines.join('\n'))
}

function copyRawJson() {
  if (!mediaInfo.value) return
  copyToClipboard(mediaInfo.value.raw)
}
</script>

<style scoped>
.ffmpeg-banner {
  padding: 12px 16px;
  border-radius: 6px;
  margin-bottom: 16px;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ffmpeg-detected {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid var(--accent-green, #10b981);
  color: var(--accent-green, #10b981);
}

.ffmpeg-missing {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red, #ef4444);
  color: var(--accent-red, #ef4444);
}

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-tip {
  margin-left: auto;
  font-size: 13px;
}

.ffmpeg-cmd {
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.file-info {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-name {
  font-weight: 600;
  color: var(--text-primary, #e2e8f0);
}

.file-detail {
  color: var(--text-secondary, #94a3b8);
  font-size: 13px;
}

.error-message {
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red, #ef4444);
  border-radius: 4px;
  color: var(--accent-red, #ef4444);
  font-size: 13px;
  line-height: 1.5;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.info-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-key {
  color: var(--text-secondary, #94a3b8);
  font-size: 12px;
  font-weight: 500;
}

.info-value {
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
  word-break: break-all;
}

.chapter-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chapter-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input, #0d1520);
  border-radius: 4px;
}

.chapter-time {
  color: var(--accent-cyan, #00d4ff);
  font-family: 'Courier New', monospace;
  font-size: 13px;
  min-width: 180px;
}

.chapter-title {
  color: var(--text-primary, #e2e8f0);
  font-size: 14px;
}

.raw-json {
  background: var(--bg-input, #0d1520);
  padding: 16px;
  border-radius: 4px;
  color: var(--text-primary, #e2e8f0);
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 500px;
  overflow-y: auto;
}
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/MediaInfoTool.vue
git commit -m "feat(media-info): 创建前端页面"
```

---

## Task 5: 注册前端路由与菜单

**Files:**
- Modify: `src/App.vue:1-100` (组件导入与映射)
- Modify: `src/store/index.ts:45-94` (TOOL_LIST)

- [ ] **Step 1: 在 App.vue 中导入组件**

在第 79 行（`import VideoTool from '@/views/VideoTool.vue'`）之后添加：

```typescript
import MediaInfoTool from '@/views/MediaInfoTool.vue'
```

- [ ] **Step 2: 在 toolComponentMap 中注册组件**

在 `toolComponentMap` 对象中（`videoTool: VideoTool,` 之后）添加：

```typescript
mediaInfo: MediaInfoTool,
```

- [ ] **Step 3: 在 TOOL_LIST 中添加菜单项**

在 `src/store/index.ts` 的 `TOOL_LIST` 数组中，`videoTool` 条目之后添加：

```typescript
{ id: 'mediaInfo', name: '媒体信息', icon: 'ℹ️', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`, description: '查看音视频文件的详细信息（编解码器、分辨率、比特率、元数据等）', keywords: ['媒体', '信息', 'ffprobe', '视频', '音频', 'metadata'], category: 'utility' },
```

- [ ] **Step 4: 验证前端编译**

Run: `npm run build`
Expected: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src/App.vue src/store/index.ts
git commit -m "feat(media-info): 注册前端路由与菜单"
```

---

## Task 6: 工作流集成

**Files:**
- Modify: `src/views/WorkflowView.vue:697-716` (executeStep 函数)

- [ ] **Step 1: 在 executeStep 中添加 mediaInfo 分支**

在 `WorkflowView.vue` 的 `executeStep` 函数中，`case 'calculator':` 之前添加：

```typescript
case 'mediaInfo': {
  const result = await invoke<any>('get_media_info', { path: input })
  return JSON.stringify(result.structured, null, 2)
}
```

- [ ] **Step 2: 验证编译**

Run: `npm run build`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/views/WorkflowView.vue
git commit -m "feat(media-info): 集成工作流"
```

---

## Task 7: 手动测试与验证

- [ ] **Step 1: 启动开发服务器**

Run: `npm run tauri dev`

- [ ] **Step 2: 测试视频文件**

1. 在侧边栏点击「媒体信息」
2. 点击「选择文件」，选择一个 MP4 视频文件
3. 验证显示：容器信息、视频流、音频流、元数据
4. 点击「复制」按钮，验证剪贴板内容
5. 点击「展开」原始 JSON，验证 JSON 格式正确

- [ ] **Step 3: 测试音频文件**

1. 点击「清除」
2. 选择一个 MP3 或 FLAC 文件
3. 验证显示：容器信息、音频流、元数据（无视频流卡片）

- [ ] **Step 4: 测试错误场景**

1. 选择一个非媒体文件（如 .txt）
2. 验证显示错误信息

- [ ] **Step 5: 测试工作流**

1. 打开工作流页面
2. 添加一个步骤，工具选择「媒体信息」
3. 输入一个视频文件路径
4. 执行工作流，验证输出为 JSON 格式

- [ ] **Step 6: 提交最终版本**

```bash
git add .
git commit -m "feat(media-info): 完成媒体信息查看器功能"
```

---

## 完成标准

- [ ] 后端 `get_media_info` 命令可正常调用
- [ ] 前端页面正确显示所有媒体信息
- [ ] 复制功能正常工作
- [ ] 原始 JSON 视图可展开/收起
- [ ] 工作流集成正常
- [ ] 历史记录正确保存
- [ ] 错误处理完善（无 ffmpeg、文件不存在、非媒体文件）

---

**计划结束**
