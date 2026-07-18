# 媒体信息查看器设计规格

**版本**: 1.0  
**日期**: 2026-07-18  
**状态**: 待实现

---

## 1. 概述

### 1.1 目标
创建一个独立的媒体信息查看工具，使用 ffprobe 深度分析音视频文件，展示所有可提取的元数据和技术参数。

### 1.2 核心价值
- **排查编码问题**：快速查看视频编码、色彩空间、比特率等关键参数
- **确认媒体参数**：验证文件分辨率、帧率、音频采样率等
- **查看隐藏信息**：提取元数据、章节信息、多语言音轨等

### 1.3 技术约束
- 依赖系统安装的 ffmpeg/ffprobe（运行时检测，不内嵌）
- 纯本地离线运行，无网络请求
- 遵循 LitoBox 轻量级原则（不新增依赖）

---

## 2. 架构设计

### 2.1 独立工具页面
- **位置**：侧边栏 `utility` 分类，紧跟 `videoTool` 之后
- **菜单项**：`mediaInfo` - 媒体信息
- **页面文件**：`src/views/MediaInfoTool.vue`
- **后端模块**：`src-tauri/src/media_info.rs`

### 2.2 数据流
```
用户选择文件
  ↓
前端 invoke('get_media_info', { path })
  ↓
后端调用 ffprobe -v quiet -print_format json -show_format -show_streams -show_chapters
  ↓
解析 JSON，提取结构化数据
  ↓
返回 { structured, raw }
  ↓
前端展示结构化卡片 + 可切换的原始 JSON 视图
```

### 2.3 文件变更清单
1. **新增** `src-tauri/src/media_info.rs` - 后端命令实现
2. **新增** `src/views/MediaInfoTool.vue` - 前端页面
3. **修改** `src-tauri/src/main.rs` - 注册 `get_media_info` 命令
4. **修改** `src/store/index.ts` - 添加 `mediaInfo` 菜单项
5. **修改** `src/router/index.ts` - 添加路由（如需要）
6. **修改** `src/views/WorkflowView.vue` - 工作流集成

---

## 3. 后端数据结构

### 3.1 返回结构
```rust
pub struct MediaInfoResult {
    pub structured: StructuredMediaInfo,
    pub raw: String,  // ffprobe 原始 JSON
}

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
```

### 3.2 容器格式信息
```rust
pub struct FormatInfo {
    pub format_name: String,           // "mov,mp4,m4a,3gp,3g2,mj2"
    pub format_long_name: String,      // "QuickTime / MOV"
    pub duration: f64,                 // 秒
    pub bitrate: u64,                  // bps
    pub stream_count: u32,
    pub nb_programs: u32,
}
```

### 3.3 视频流信息
```rust
pub struct VideoStreamInfo {
    pub index: u32,
    pub codec_name: String,            // h264, hevc, vp9
    pub codec_long_name: String,       // "H.264 / AVC / MPEG-4 AVC"
    pub profile: String,               // High, Main, Baseline
    pub level: Option<i64>,            // 41, 51
    pub width: u32,
    pub height: u32,
    pub coded_width: u32,
    pub coded_height: u32,
    pub display_aspect_ratio: String,  // "16:9"
    pub sample_aspect_ratio: String,   // "1:1"
    pub pix_fmt: String,               // yuv420p, yuv420p10le
    pub color_space: String,           // bt709, bt2020nc
    pub color_primaries: String,       // bt709, bt2020
    pub color_transfer: String,        // bt709, smpte2084(HDR10)
    pub color_range: String,           // tv, pc
    pub chroma_location: String,       // left, center
    pub field_order: String,           // progressive, tt
    pub fps: f64,                      // r_frame_rate
    pub avg_fps: f64,                  // avg_frame_rate
    pub bitrate: u64,                  // bps
    pub bit_depth: Option<u32>,        // 8, 10, 12
    pub duration: f64,
    pub nb_frames: u64,
    pub disposition: Vec<String>,      // default, forced
    pub tags: Vec<KeyValue>,           // 流级元数据
    pub extra: Vec<KeyValue>,          // 其他未归类字段
}
```

### 3.4 音频流信息
```rust
pub struct AudioStreamInfo {
    pub index: u32,
    pub codec_name: String,            // aac, mp3, flac
    pub codec_long_name: String,
    pub profile: String,               // LC, HE-AAC
    pub sample_rate: u32,              // 44100, 48000
    pub channels: u32,                 // 1=mono, 2=stereo
    pub channel_layout: String,        // "stereo", "5.1"
    pub sample_fmt: String,            // fltp, s16, s32
    pub bit_depth: Option<u32>,        // bits_per_raw_sample
    pub bitrate: u64,
    pub duration: f64,
    pub nb_frames: u64,
    pub disposition: Vec<String>,
    pub tags: Vec<KeyValue>,
    pub extra: Vec<KeyValue>,
}
```

### 3.5 字幕流与其他流
```rust
pub struct SubtitleStreamInfo {
    pub index: u32,
    pub codec_name: String,            // srt, ass, subrip
    pub codec_long_name: String,
    pub tags: Vec<KeyValue>,
}

pub struct OtherStreamInfo {
    pub index: u32,
    pub codec_type: String,            // data, attachment, unknown
    pub codec_name: String,
    pub tags: Vec<KeyValue>,
}
```

### 3.6 章节与元数据
```rust
pub struct ChapterInfo {
    pub id: u64,
    pub start_time: f64,
    pub end_time: f64,
    pub title: String,
    pub tags: Vec<KeyValue>,
}

pub struct KeyValue {
    pub key: String,
    pub value: String,
}
```

### 3.7 关键设计点
- **ffprobe 命令**：`ffprobe -v quiet -print_format json -show_format -show_streams -show_chapters`
- **extra 字段**：收集所有未在结构体中显式定义的字段，确保不丢失任何信息
- **元数据分层**：
  - `format.tags` → `StructuredMediaInfo.metadata`
  - `stream.tags` → 各流的 `tags` 字段
- **raw 字段**：保存 ffprobe 完整 JSON 原文，供高级用户查看

---

## 4. 前端页面设计

### 4.1 页面结构
基于 `_ToolTemplate.vue` 创建 `MediaInfoTool.vue`，无 Tab（单功能页面）。

### 4.2 布局层次
```
┌─────────────────────────────────────────────────────────────┐
│ ffmpeg 状态横幅（.ffmpeg-banner）                             │
├─────────────────────────────────────────────────────────────┤
│ 文件选择卡片（.tool-card）                                     │
│   [选择文件] [清除]                                           │
│   文件名.mp4 | 1.2 GB                                        │
├─────────────────────────────────────────────────────────────┤
│ 容器信息卡片（.tool-card）                                     │
│   格式名称    MPEG-4 Part 14 (mp4)                           │
│   时长        01:23:45.123                                   │
│   文件大小    1.2 GB (1,234,567,890 bytes)                   │
│   总比特率    2,500 kbps                                     │
│   流数量      3 (视频×1 + 音频×1 + 字幕×1)                    │
├─────────────────────────────────────────────────────────────┤
│ 视频流 #0 卡片（.tool-card）                                  │
│   编解码器     H.264 (High Profile, Level 4.1)               │
│   分辨率       1920×1080 (16:9)                              │
│   帧率         23.976 fps                                    │
│   像素格式     yuv420p (8 bit)                               │
│   色彩空间     BT.709 / BT.709 / BT.709                      │
│   比特率       2,100 kbps                                    │
│   帧数         120,456                                       │
│   ...其他字段以 key-value 列表展示                            │
├─────────────────────────────────────────────────────────────┤
│ 音频流 #1 卡片（.tool-card）                                  │
│   编解码器     AAC (LC Profile)                              │
│   采样率       48,000 Hz                                     │
│   声道         立体声 (stereo)                                │
│   位深度       16 bit                                        │
│   比特率       128 kbps                                      │
│   ...其他字段以 key-value 列表展示                            │
├─────────────────────────────────────────────────────────────┤
│ 字幕流 #2 卡片（如有）                                        │
│ 其他流卡片（如有 data/attachment 等）                          │
├─────────────────────────────────────────────────────────────┤
│ 元数据卡片（.tool-card）                                      │
│   标题         xxx                                           │
│   艺术家       xxx                                           │
│   创建时间     2024-01-01                                    │
│   编码工具     Lavf58.76.100                                 │
│   ...所有 format.tags 以 key-value 展示                      │
├─────────────────────────────────────────────────────────────┤
│ 章节卡片（如有章节，.tool-card）                               │
│   #1  00:00:00 - 00:05:00  开场                              │
│   #2  00:05:00 - 01:20:00  正片                              │
├─────────────────────────────────────────────────────────────┤
│ 原始 JSON 卡片（.tool-card）                                  │
│   [▼ 查看原始 JSON]                        [复制 JSON]       │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ {                                                   │   │
│   │   "format": { ... },                                │   │
│   │   "streams": [ ... ],                               │   │
│   │   "chapters": [ ... ]                               │   │
│   │ }                                                   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 样式规范
- **卡片样式**：使用全局 `.tool-card` + `.card-header` + `.card-body`
- **信息列表**：自定义 `.info-row` class，key 用 `--text-secondary`，value 用 `--text-primary`
- **网格布局**：`.info-grid` 2 列网格，每行 2 个 key-value 对
- **原始 JSON**：`<pre>` + 等宽字体，默认折叠，点击展开
- **颜色**：全部使用 CSS 变量，无硬编码

### 4.4 交互功能
- **文件选择**：点击「选择文件」按钮，调用 Tauri `dialog` API
- **清除**：点击「清除」按钮，重置所有状态
- **复制**：每个卡片右上角「复制」按钮，复制该卡片内容
- **原始 JSON**：点击「查看原始 JSON」展开/折叠，点击「复制 JSON」复制完整 JSON

---

## 5. 错误处理

### 5.1 ffmpeg 未安装
- 页面顶部显示红色横幅：「此功能需要 ffmpeg，请先安装」+ 安装命令
- 不显示文件选择按钮，禁用功能
- 复用 VideoTool 的 ffmpeg 检测逻辑（调用 `check_ffmpeg_available` 命令）

### 5.2 文件解析失败
- 显示 `.error-message` 卡片：「无法解析媒体文件：{具体错误}」
- 错误来源：ffprobe 执行失败、JSON 解析失败、文件不存在等

### 5.3 文件不存在或无权限
- ffprobe 返回错误码，捕获 stderr 输出
- 显示：「文件不存在或无法访问：{path}」

### 5.4 部分信息缺失
- ffprobe 返回的 JSON 中某些字段可能不存在（如纯音频文件无视频流）
- 前端用 `v-if` 条件渲染，缺失的卡片不显示
- 视频流中某些字段（如 `color_space`）可能为空字符串，显示为「未知」或不显示

### 5.5 边界情况
- **纯音频文件**：`video_streams` 为空，不显示视频流卡片
- **纯视频文件（无音频）**：`audio_streams` 为空，不显示音频流卡片
- **多视频流**（如 3D 视频）：显示多个视频流卡片
- **无章节信息**：`chapters` 为空，不显示章节卡片
- **元数据为空**：`metadata` 为空数组，不显示元数据卡片
- **非媒体文件**（如 .txt）：ffprobe 报错，显示错误信息

---

## 6. 工作流集成

### 6.1 工作流步骤
在 `WorkflowView.vue` 的 `executeStep()` 中添加 `mediaInfo` 分支：
- **输入**：文件路径（从变量池或上一步输出获取）
- **输出**：结构化 JSON 字符串（`structured` 部分序列化）
- **复用**：后端 `get_media_info` 命令，不重复实现

### 6.2 历史记录
调用 `store.addHistory()` 记录操作：
- `tool`: `'mediaInfo'`
- `action`: `'查看媒体信息'`
- `inputPreview`: 文件名（截断 50 字符）
- `outputPreview`: 格式摘要（如 `MP4 | 1920x1080 | H.264 | 01:23:45`）
- `inputFull`: 完整文件路径
- `outputFull`: 完整结构化 JSON

---

## 7. 实现要点

### 7.1 后端实现
1. **ffprobe 调用**：
   - 使用 `std::process::Command` 调用 ffprobe
   - 参数：`-v quiet -print_format json -show_format -show_streams -show_chapters`
   - 捕获 stdout（JSON 输出）和 stderr（错误信息）

2. **JSON 解析**：
   - 使用 `serde_json` 解析 ffprobe 输出
   - 提取 `format`、`streams`、`chapters` 三个顶层字段
   - 按 `codec_type` 分类 streams（video/audio/subtitle/other）

3. **字段提取**：
   - 显式提取结构体中定义的字段
   - 剩余字段收集到 `extra` 数组
   - 处理缺失字段（使用 `unwrap_or_default()` 或 `Option`）

4. **错误处理**：
   - ffprobe 不存在：返回「ffmpeg 未安装」错误
   - ffprobe 执行失败：返回 stderr 内容
   - JSON 解析失败：返回解析错误

### 7.2 前端实现
1. **状态管理**：
   - `mediaInfo: MediaInfoResult | null` - 当前分析结果
   - `rawJsonVisible: boolean` - 原始 JSON 是否可见
   - `isLoading: boolean` - 加载状态

2. **条件渲染**：
   - `v-if="mediaInfo"` - 有结果时显示信息卡片
   - `v-if="mediaInfo.video_streams.length > 0"` - 有视频流时显示
   - `v-if="mediaInfo.chapters.length > 0"` - 有章节时显示

3. **复制功能**：
   - 使用 `navigator.clipboard.writeText()`
   - 复制前格式化为可读文本（key-value 格式）

4. **原始 JSON 展示**：
   - 使用 `<pre>` 标签展示
   - 默认折叠，点击按钮展开/收起
   - 提供「复制 JSON」按钮

---

## 8. 测试要点

### 8.1 功能测试
- [ ] 选择视频文件，正确显示所有信息
- [ ] 选择音频文件，不显示视频流卡片
- [ ] 选择带章节的文件，正确显示章节信息
- [ ] 选择无元数据的文件，不显示元数据卡片
- [ ] 点击「查看原始 JSON」，正确展开 JSON
- [ ] 点击「复制」按钮，正确复制内容到剪贴板

### 8.2 错误处理测试
- [ ] 未安装 ffmpeg，显示错误横幅
- [ ] 选择不存在的文件，显示错误信息
- [ ] 选择非媒体文件（如 .txt），显示错误信息
- [ ] ffprobe 执行失败，显示错误信息

### 8.3 边界情况测试
- [ ] 多视频流文件（如 3D 视频），显示多个视频流卡片
- [ ] 多音频流文件（多语言音轨），显示多个音频流卡片
- [ ] 超大文件（>10GB），快速返回结果（ffprobe 不解码）
- [ ] 文件名包含特殊字符，正确处理

---

## 9. 后续优化（可选）

### 9.1 增强功能
- **导出报告**：将媒体信息导出为 Markdown/HTML 报告
- **批量分析**：支持拖入多个文件，批量生成报告
- **对比视图**：并排对比两个文件的媒体信息
- **缩略图预览**：提取视频首帧作为预览图

### 9.2 性能优化
- **大文件优化**：对于超大文件，显示加载进度
- **缓存机制**：对同一文件的分析结果进行缓存（localStorage）

---

## 10. 参考资源

- **ffprobe 文档**：https://ffmpeg.org/ffprobe.html
- **MediaInfo 工具**：https://mediaarea.net/en/MediaInfo
- **现有实现参考**：
  - `src-tauri/src/video_tools.rs` - ffprobe 调用方式
  - `src-tauri/src/audio_tools.rs` - ffprobe 调用方式
  - `src/views/VideoTool.vue` - ffmpeg 检测逻辑
  - `src/views/AudioTool.vue` - 页面布局参考

---

**文档结束**
