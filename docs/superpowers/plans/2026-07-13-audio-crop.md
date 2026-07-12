# 音频裁剪工具 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现纯 Rust 音频裁剪工具，支持 MP3/WAV 单段裁剪、波形可视化、滑块拖拽选择、实时预览播放。

**Architecture:** 后端新增 `audio_tools.rs` 模块（symphonia 解码 + hound/lame 编码），前端新增 `AudioTool.vue` 单页面组件（Canvas 波形 + Web Audio API 预览）。所有命令使用 `async fn` + `spawn_blocking` 模式。

**Tech Stack:** symphonia 0.5, hound 3.5, mp3lame-encoder 0.5 (Rust), Canvas 2D + Web Audio API (Vue 3)

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | 添加音频依赖 |
| `src-tauri/src/audio_tools.rs` | 新增 | 音频解码/编码/波形生成/裁剪 |
| `src-tauri/src/main.rs` | 修改 | 注册模块和命令 |
| `src/views/AudioTool.vue` | 新增 | 前端页面 |
| `src/store/index.ts` | 修改 | TOOL_LIST 添加条目 |
| `src/App.vue` | 修改 | 注册组件映射 |

---

### Task 1: 添加 Rust 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 修改 Cargo.toml 添加音频依赖**

在 `[dependencies]` 段落末尾添加：

```toml
symphonia = { version = "0.5", features = ["mp3", "wav"], default-features = false }
symphonia-core = "0.5"
hound = "3.5"
mp3lame-encoder = "0.5"
```

- [ ] **Step 2: 验证依赖下载**

```powershell
cd src-tauri; cargo check
```

预期：下载依赖，无编译错误（仅可能有未使用代码的 warning）。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: 添加音频处理依赖 (symphonia, hound, mp3lame-encoder)"
```

---

### Task 2: 创建 audio_tools.rs - 结构体和解码辅助函数

**Files:**
- Create: `src-tauri/src/audio_tools.rs`

- [ ] **Step 1: 创建文件，定义结构体和基础 helper**

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::io::Cursor;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}

// ============ 内部函数 ============

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
        .map_err(|e| format!("不支持的音频格式，仅支持 MP3/WAV: {}", e))?;

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

        // 帧与目标区间有交集
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
fn encode_mp3(samples: &[f32], sample_rate: u32, channels: u16, bitrate: u32) -> Result<Vec<u8>, String> {
    use mp3lame_encoder::{Encoder, Bitrate, MonoOrStereo, InterleavedPcm};

    let mut enc = Encoder::new()
        .map_err(|e| format!("MP3 编码器初始化失败: {}", e))?;

    let br = match bitrate {
        128 => Bitrate::Kbps128,
        256 => Bitrate::Kbps256,
        320 => Bitrate::Kbps320,
        _ => Bitrate::Kbps192,
    };
    enc.set_bitrate(br).map_err(|e| format!("设置比特率失败: {}", e))?;
    enc.set_sample_rate(sample_rate).map_err(|e| format!("设置采样率失败: {}", e))?;
    let stereo = if channels == 2 { MonoOrStereo::Stereo } else { MonoOrStereo::Mono };
    enc.set_channels(stereo).map_err(|e| format!("设置声道失败: {}", e))?;

    let mut mp3_enc = enc.build().map_err(|e| format!("构建 MP3 编码器失败: {}", e))?;

    // 转换为 i16
    let i16_samples: Vec<i16> = samples.iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let input = InterleavedPcm::from(&i16_samples);
    let mut mp3_buf = mp3_enc.encode(input)
        .map_err(|e| format!("MP3 编码失败: {}", e))?;

    let tail = mp3_enc.encode_finish()
        .map_err(|e| format!("MP3 收尾失败: {}", e))?;
    mp3_buf.extend_from_slice(&tail);

    Ok(mp3_buf)
}

/// 从文件路径推断格式
fn guess_format(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".wav") || lower.ends_with(".wave") {
        "wav".to_string()
    } else if lower.ends_with(".mp3") {
        "mp3".to_string()
    } else {
        "unknown".to_string()
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/src/audio_tools.rs
git commit -m "feat: 创建 audio_tools 模块骨架 - 结构体与解码辅助函数"
```

---

### Task 3: 实现 Tauri 命令

**Files:**
- Modify: `src-tauri/src/audio_tools.rs`

- [ ] **Step 1: 追加 get_audio_info 命令**

在 `audio_tools.rs` 末尾追加：

```rust
// ============ Tauri 命令 ============

#[tauri::command]
pub async fn get_audio_info(path: String) -> Result<AudioInfo, String> {
    tauri::async_runtime::spawn_blocking(move || do_get_audio_info(&path))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_get_audio_info(path: &str) -> Result<AudioInfo, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;
    let file_size = metadata.len();

    let format = guess_format(path);
    if format == "unknown" {
        return Err("不支持的音频格式，仅支持 MP3/WAV".to_string());
    }

    // 解码获取采样率和声道
    let (samples, sample_rate, channels) = decode_audio_full(path)?;
    let total_samples = samples.len() / channels as usize;
    let duration = total_samples as f64 / sample_rate as f64;

    let bitrate = if duration > 0.0 {
        ((file_size as f64 * 8.0) / duration / 1000.0) as u32
    } else {
        0
    };

    Ok(AudioInfo { duration, sample_rate, channels, format, bitrate, file_size })
}
```

- [ ] **Step 2: 追加 generate_waveform 命令**

```rust
#[tauri::command]
pub async fn generate_waveform(path: String) -> Result<WaveformData, String> {
    tauri::async_runtime::spawn_blocking(move || do_generate_waveform(&path))
        .await
        .map_err(|e| format!("任务执行失败: {}", e))?
}

fn do_generate_waveform(path: &str) -> Result<WaveformData, String> {
    let (samples, sample_rate, channels) = decode_audio_full(path)?;
    let total_samples = samples.len() / channels as usize;
    let duration = total_samples as f64 / sample_rate as f64;

    // 降采样到 ~2000 个点
    const TARGET_POINTS: usize = 2000;
    let step = (total_samples.max(1) as f64 / TARGET_POINTS as f64).max(1.0);
    let mut points = Vec::with_capacity(TARGET_POINTS);

    for i in 0..TARGET_POINTS {
        let start_idx = (i as f64 * step) as usize * channels as usize;
        let end_idx = ((i as f64 + 1.0) * step) as usize * channels as usize;
        let end_idx = end_idx.min(samples.len());

        if start_idx >= end_idx {
            points.push(0.0);
            continue;
        }

        // 取该区间内最大绝对值
        let mut max_abs = 0.0f32;
        for j in (start_idx..end_idx).step_by(channels as usize) {
            let v = samples[j].abs();
            if v > max_abs { max_abs = v; }
        }
        points.push(max_abs);
    }

    Ok(WaveformData { points, duration, sample_rate })
}
```

- [ ] **Step 3: 追加 audio_crop 命令**

```rust
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

fn do_audio_crop(
    app_handle: tauri::AppHandle,
    path: &str,
    options: &CropOptions,
) -> Result<CropResult, String> {
    if options.start_time < 0.0 || options.end_time <= options.start_time {
        return Err("起止时间非法".to_string());
    }
    let duration = options.end_time - options.start_time;
    if duration < 0.1 {
        return Err("裁剪区间不能小于 0.1 秒".to_string());
    }

    let (samples, sample_rate, channels) = decode_audio_segment(path, options.start_time, options.end_time)?;

    let output_bytes = match options.output_format.as_str() {
        "mp3" => encode_mp3(&samples, sample_rate, channels, options.mp3_bitrate)?,
        "wav" => encode_wav(&samples, sample_rate, channels)?,
        _ => return Err("不支持的输出格式".to_string()),
    };

    // 写入临时文件
    let ext = &options.output_format;
    let input_stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let (dir, prefix) = std::path::Path::new(path)
        .parent()
        .map(|p| (p, input_stem))
        .unwrap_or_else(|| (std::path::Path::new("."), input_stem));

    let output_path = dir.join(format!("{}_cropped.{}", prefix, ext));

    std::fs::write(&output_path, &output_bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let output_size = output_bytes.len() as u64;

    Ok(CropResult {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        duration,
    })
}
```

- [ ] **Step 4: 追加 get_audio_preview 命令**

```rust
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
    if duration < 0.1 || duration > 60.0 {
        return Err("预览区间需在 0.1-60 秒之间".to_string());
    }

    let (samples, sample_rate, channels) = decode_audio_segment(path, start, end)?;
    let wav_bytes = encode_wav(&samples, sample_rate, channels)?;
    Ok(BASE64.encode(&wav_bytes))
}
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/audio_tools.rs
git commit -m "feat: 实现音频 Tauri 命令 (info/waveform/crop/preview)"
```

---

### Task 4: 注册模块和命令到 main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 添加 mod 声明**

在 `src-tauri/src/main.rs` 的 `mod` 声明区域末尾添加：

```rust
mod audio_tools;
```

- [ ] **Step 2: 注册命令**

在 `invoke_handler` 的 `generate_handler![...]` 末尾（`]` 之前）添加：

```rust
            audio_tools::get_audio_info,
            audio_tools::generate_waveform,
            audio_tools::audio_crop,
            audio_tools::get_audio_preview,
```

- [ ] **Step 3: 验证编译**

```powershell
cd src-tauri; cargo check
```

预期：编译通过。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: 注册 audio_tools 模块和 Tauri 命令"
```

---

### Task 5: 创建 AudioTool.vue 前端页面

**Files:**
- Create: `src/views/AudioTool.vue`

- [ ] **Step 1: 创建基础模板和脚本**

```vue
<template>
  <div class="tool-container">
    <!-- 文件选择 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">选择音频文件</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" @click="openFile" :loading="isLoadingInfo">
              打开文件
            </el-button>
          </div>
        </div>
        <div v-if="filePath" class="audio-file-info">
          <span class="file-name">{{ fileName }}</span>
          <span class="file-detail" v-if="audioInfo">
            {{ formatDuration(audioInfo.duration) }} | {{ audioInfo.format.toUpperCase() }} |
            {{ audioInfo.sample_rate }}Hz | {{ audioInfo.channels === 2 ? '立体声' : '单声道' }} |
            {{ audioInfo.bitrate }}kbps
          </span>
        </div>
      </div>
    </div>

    <!-- 波形预览 -->
    <div v-if="waveformData.points.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">波形预览</span>
      </div>
      <div class="card-body">
        <div class="waveform-container" ref="waveformContainer">
          <canvas ref="canvasRef" class="waveform-canvas" @mousedown="onCanvasMouseDown"></canvas>
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
        <div class="waveform-labels">
          <span>{{ formatTime(startTime) }}</span>
          <span>{{ formatTime(endTime) }}</span>
        </div>
        <div class="action-grid" style="margin-top: 8px">
          <div class="action-group">
            <el-button size="small" @click="togglePreview" :type="isPreviewing ? 'danger' : 'default'">
              {{ isPreviewing ? '⏹ 停止' : '▶ 预览选中区域' }}
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 裁剪设置 -->
    <div v-if="waveformData.points.length > 0" class="tool-card">
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
              :max="waveformData.duration"
              :step="0.1"
              :precision="1"
              size="small"
              style="width: 140px"
            />
            <span class="unit-text">秒</span>
          </div>
          <div class="action-group">
            <div class="group-label">输出格式</div>
            <el-select v-model="outputFormat" size="small" style="width: 100px">
              <el-option label="MP3" value="mp3" />
              <el-option label="WAV" value="wav" />
            </el-select>
          </div>
          <div class="action-group" v-if="outputFormat === 'mp3'">
            <div class="group-label">比特率</div>
            <el-select v-model="mp3Bitrate" size="small" style="width: 120px">
              <el-option label="128 kbps" :value="128" />
              <el-option label="192 kbps" :value="192" />
              <el-option label="256 kbps" :value="256" />
              <el-option label="320 kbps" :value="320" />
            </el-select>
          </div>
        </div>
        <div class="segment-info" v-if="audioInfo">
          片段时长: {{ formatDuration(segmentDuration) }}
        </div>
      </div>
    </div>

    <!-- 操作 -->
    <div v-if="waveformData.points.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" @click="cropAudio" :loading="isProcessing" :disabled="!isRangeValid">
              裁剪并导出
            </el-button>
            <el-button size="small" @click="resetForm">重置</el-button>
          </div>
        </div>
        <el-progress v-if="isProcessing" :percentage="cropProgress" :stroke-width="6" style="margin-top: 12px" />
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'

// ============ 类型定义 ============
interface AudioInfo {
  duration: number
  sample_rate: number
  channels: number
  format: string
  bitrate: number
  file_size: number
}

interface WaveformData {
  points: number[]
  duration: number
  sample_rate: number
}

interface CropResult {
  output_path: string
  output_size: number
  duration: number
}

// ============ 状态 ============
const filePath = ref('')
const fileName = ref('')
const audioInfo = ref<AudioInfo | null>(null)
const waveformData = ref<WaveformData>({ points: [], duration: 0, sample_rate: 44100 })
const startTime = ref(0)
const endTime = ref(0)
const outputFormat = ref<'mp3' | 'wav'>('mp3')
const mp3Bitrate = ref(192)
const isProcessing = ref(false)
const isLoadingInfo = ref(false)
const isPreviewing = ref(false)
const cropProgress = ref(0)
const error = ref('')

// ============ 计算属性 ============
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// ============ Canvas ============
const canvasRef = ref<HTMLCanvasElement | null>(null)
const waveformContainer = ref<HTMLDivElement | null>(null)

function drawWaveform() {
  const canvas = canvasRef.value
  if (!canvas || !waveformData.value.points.length) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.scale(dpr, dpr)
  const width = rect.width
  const height = rect.height
  const data = waveformData.value.points
  const n = data.length
  const dur = waveformData.value.duration

  // 背景
  const style = getComputedStyle(document.documentElement)
  const bgColor = style.getPropertyValue('--bg-input').trim() || '#0d1520'
  const primaryColor = style.getPropertyValue('--accent-cyan').trim() || '#00d4ff'
  const secondaryColor = style.getPropertyValue('--text-secondary').trim() || '#94a3b8'

  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, width, height)

  const startX = (startTime.value / dur) * width
  const endX = (endTime.value / dur) * width
  const barWidth = width / n
  const midY = height / 2

  for (let i = 0; i < n; i++) {
    const x = i * barWidth
    const barHeight = data[i] * midY * 0.9

    if (x >= startX && x <= endX) {
      ctx.fillStyle = primaryColor
    } else {
      ctx.fillStyle = secondaryColor + '66'
    }

    ctx.fillRect(x, midY - barHeight / 2, Math.max(barWidth, 1), barHeight || 1)
  }

  // 选中区域高亮覆盖
  ctx.fillStyle = primaryColor + '1A'
  ctx.fillRect(startX, 0, endX - startX, height)
}

function timeToPercent(time: number): number {
  if (waveformData.value.duration <= 0) return 0
  return (time / waveformData.value.duration) * 100
}

function percentToTime(percent: number): number {
  return Math.round((percent / 100) * waveformData.value.duration * 10) / 10
}

// ============ 滑块拖拽 ============
let draggingSlider: 'start' | 'end' | null = null

function onSliderMouseDown(e: MouseEvent, slider: 'start' | 'end') {
  draggingSlider = slider
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!draggingSlider || !waveformContainer.value) return
  const rect = waveformContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  if (draggingSlider === 'start') {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawWaveform()
}

function onMouseUp() {
  draggingSlider = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown(e: MouseEvent) {
  if (!waveformContainer.value) return
  const rect = waveformContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  // 点击靠近哪个滑块就移动哪个
  const startDist = Math.abs(time - startTime.value)
  const endDist = Math.abs(time - endTime.value)

  if (startDist <= endDist) {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawWaveform()
}

// ============ 音频预览 ============
let audioCtx: AudioContext | null = null
let audioSource: AudioBufferSourceNode | null = null

async function togglePreview() {
  if (isPreviewing.value) {
    stopPreview()
    return
  }
  await previewAudio()
}

async function previewAudio() {
  try {
    error.value = ''
    const base64Wav: string = await invoke('get_audio_preview', {
      path: filePath.value,
      start: startTime.value,
      end: endTime.value,
    })

    const binaryStr = atob(base64Wav)
    const bytes = new Uint8Array(binaryStr.length)
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i)
    }

    if (!audioCtx) {
      audioCtx = new AudioContext()
    }
    await audioCtx.resume()

    const audioBuffer = await audioCtx.decodeAudioData(bytes.buffer.slice(0))
    audioSource = audioCtx.createBufferSource()
    audioSource.buffer = audioBuffer
    audioSource.connect(audioCtx.destination)
    audioSource.onended = () => { isPreviewing.value = false }
    audioSource.start()
    isPreviewing.value = true
  } catch (e: any) {
    error.value = '预览播放失败: ' + (typeof e === 'string' ? e : e.message || e)
    isPreviewing.value = false
  }
}

function stopPreview() {
  if (audioSource) {
    try { audioSource.stop() } catch (_) { /* 忽略已停止错误 */ }
    audioSource = null
  }
  isPreviewing.value = false
}

// ============ 文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav'] }],
      multiple: false,
    })
    if (!selected) return

    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''

    isLoadingInfo.value = true
    const info: AudioInfo = await invoke('get_audio_info', { path: filePath.value })
    audioInfo.value = info

    const wf: WaveformData = await invoke('generate_waveform', { path: filePath.value })
    waveformData.value = wf
    // 用实际解码时长更新
    audioInfo.value.duration = wf.duration

    startTime.value = 0
    endTime.value = wf.duration
    await nextTick()
    drawWaveform()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetForm()
  } finally {
    isLoadingInfo.value = false
  }
}

async function cropAudio() {
  if (!isRangeValid.value) {
    ElMessage.warning('请设置有效的裁剪区间')
    return
  }

  try {
    error.value = ''
    isProcessing.value = true
    cropProgress.value = 0

    const result: CropResult = await invoke('audio_crop', {
      path: filePath.value,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        output_format: outputFormat.value,
        mp3_bitrate: mp3Bitrate.value,
      },
    })

    cropProgress.value = 100
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
  audioInfo.value = null
  waveformData.value = { points: [], duration: 0, sample_rate: 44100 }
  startTime.value = 0
  endTime.value = 0
  error.value = ''
  stopPreview()
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

// ============ 响应式 ============
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => drawWaveform())
    resizeObserver.observe(canvasRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  stopPreview()
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

watch([startTime, endTime], () => drawWaveform())
</script>

<style scoped>
.audio-file-info {
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

.waveform-container {
  position: relative;
  width: 100%;
  height: 200px;
  cursor: pointer;
}

.waveform-canvas {
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

.waveform-labels {
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
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/AudioTool.vue
git commit -m "feat: 创建音频裁剪前端页面 (波形Canvas + 滑块 + 预览)"
```

---

### Task 6: 注册到 TOOL_LIST 和 App.vue

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 在 TOOL_LIST 中添加条目**

在 `src/store/index.ts` 的 `TOOL_LIST` 数组中，找到 imageToolEnhanced 条目之后（大约第 75 行），追加：

```typescript
  { id: 'audioTool', name: '音频裁剪', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>`, description: '音频裁剪，支持 MP3/WAV 格式，波形可视化、实时预览', keywords: ['音频', '裁剪', 'mp3', 'wav', '波形', 'audio'], category: 'utility' },
```

- [ ] **Step 2: 在 App.vue 中注册组件**

在 `src/App.vue` 的 import 区域末尾添加：

```typescript
import AudioTool from '@/views/AudioTool.vue'
```

在 `componentMap` 对象中添加：

```typescript
  audioTool: AudioTool,
```

- [ ] **Step 3: 验证构建**

```powershell
npm run build
```

预期：构建成功。

- [ ] **Step 4: 提交**

```bash
git add src/store/index.ts src/App.vue
git commit -m "feat: 注册音频裁剪工具到侧边栏和路由"
```

---

### Task 7: 端到端验证

**Files:** 无（手动验证）

- [ ] **Step 1: 启动开发服务器**

```powershell
npm run tauri dev
```

- [ ] **Step 2: 验证核心功能**

执行以下测试用例：

1. 打开 MP3 文件 → 验证信息显示（时长/采样率/声道/比特率）
2. 波形图渲染正确，与音频内容匹配
3. 滑块拖拽，起止时间同步更新
4. 输入框手动修改时间，滑块位置同步更新
5. 预览播放选中片段，声音正确
6. 裁剪导出 MP3 → 验证输出文件可播放
7. 裁剪导出 WAV → 验证输出文件可播放
8. 边界裁剪（起始=0、结束=时长）→ 验证输出完整
9. 格式转换（MP3→WAV, WAV→MP3）→ 验证输出正确
10. 不支持格式打开 → 验证错误提示
11. 非法时间输入 → 验证前端拦截

- [ ] **Step 3: 确认构建**

```powershell
npm run build
```

预期：构建通过，无错误。

---

## 测试要点清单

- [ ] 打开标准 MP3 文件，验证信息显示正确（时长/采样率/声道）
- [ ] 打开 WAV 文件，验证信息显示正确
- [ ] 波形图渲染正确，与音频内容匹配
- [ ] 滑块拖拽，起止时间同步更新
- [ ] 输入框手动修改时间，滑块位置同步更新
- [ ] 预览播放选中片段，声音正确
- [ ] 裁剪导出 MP3，验证输出文件可播放
- [ ] 裁剪导出 WAV，验证输出文件可播放
- [ ] 边界裁剪（起始=0、结束=时长），验证输出完整
- [ ] 短片段裁剪（0.5 秒），验证输出正确
- [ ] 格式转换（MP3→WAV, WAV→MP3），验证输出正确
- [ ] 不支持格式打开，验证错误提示
- [ ] 非法时间输入，验证前端拦截
- [ ] `npm run build` 构建通过