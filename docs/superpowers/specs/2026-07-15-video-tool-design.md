# 视频工具（VideoTool）设计文档

> 版本: V5.2 | 日期: 2026-07-15 | 状态: 设计中

## 1. 概述

新增"视频工具"页面，提供视频裁剪（起止裁剪/片段提取）功能。采用纯 Rust（mp4 crate 关键帧无损裁剪）+ ffmpeg（缩略图 + 帧精确裁剪）双轨策略，与音频工具策略一致。

**来源:** [feature-backlog.md](../../superpowers/plans/feature-backlog.md) F13

## 2. 核心功能

### 2.1 视频裁剪

- 用户选择视频文件，设置起止时间区间，导出裁剪后的视频
- 双轨实现：
  - **纯 Rust 路径**（无 ffmpeg）：MP4 关键帧无损裁剪，不重编码，速度快，精度受关键帧间距限制（1-10 秒）
  - **ffmpeg 路径**（有 ffmpeg）：`-c copy` 关键帧裁剪，无损、速度快，精度优于纯 Rust（ffmpeg 自动处理关键帧对齐）
- 仅输出 MP4 格式，不做格式转换
- 输出文件默认保存在源文件同目录，命名为 `{原文件名}_cropped.mp4`

### 2.2 缩略图时间轴

- 有 ffmpeg 时：从视频均匀抽取 20 帧缩略图，Canvas 绘制时间轴，拖拽滑块选裁剪区间
- 无 ffmpeg 时：降级为纯文本时间轴（一条线 + 滑块 + 时间刻度），功能不变仅视觉降级
- 选中区域高亮（青色半透明覆盖）

### 2.3 视频元信息

- 显示：时长、分辨率、编码格式、帧率、比特率、文件大小
- 纯 Rust 通过 mp4 crate 解析 MOOV atom 获取
- ffmpeg 路径通过 ffprobe 获取（更准确）

## 3. 架构

```
┌─────────────────────────────────────────────────────┐
│  VideoTool.vue（前端）                                │
│  ├─ ffmpeg 状态横幅（复用 AudioTool 模式）              │
│  ├─ 文件选择区                                        │
│  ├─ 缩略图时间轴 / 纯文本时间轴（有/无 ffmpeg 切换）     │
│  ├─ 裁剪设置区（起止时间、实际裁剪区间提示）              │
│  └─ 操作区（裁剪按钮 + 进度条）                         │
├─────────────────────────────────────────────────────┤
│  Tauri Commands（Rust 后端）                          │
│  ├─ check_ffmpeg()          → 复用已有                │
│  ├─ get_video_info()        → 元信息（mp4 crate）     │
│  ├─ extract_thumbnails()    → ffmpeg 抽帧缩略图       │
│  ├─ video_crop()            → 裁剪入口                 │
│  │   ├─ do_video_crop_ffmpeg()    → ffmpeg 帧精确     │
│  │   └─ do_video_crop_keyframe()  → mp4 关键帧无损    │
│  └─ save_temp_file()        → 复用已有（pdf_tools）    │
├─────────────────────────────────────────────────────┤
│  依赖                                                │
│  ├─ mp4 crate（新增 ~1 MB）                          │
│  └─ ffmpeg/ffprobe（可选，运行时探测）                  │
└─────────────────────────────────────────────────────┘
```

## 4. 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | 新增 `mp4` 依赖 |
| `src-tauri/src/video_tools.rs` | 新增 | 视频处理核心逻辑 |
| `src-tauri/src/main.rs` | 修改 | 注册 `mod video_tools` 和 commands |
| `src/views/VideoTool.vue` | 新增 | 前端页面 |
| `src/store/index.ts` | 修改 | TOOL_LIST 新增"视频工具"条目 |
| `src/router/index.ts` | 修改 | 新增路由 `/video` |

## 5. Rust 后端设计

### 5.1 数据结构

```rust
struct VideoInfo {
    duration: f64,        // 时长（秒）
    width: u32,           // 宽度（像素）
    height: u32,          // 高度（像素）
    codec: String,        // 编码格式（h264/h265/av1）
    fps: f64,             // 帧率
    bitrate: u32,         // 比特率（kbps）
    file_size: u64,       // 文件大小（字节）
    format: String,       // 容器格式（mp4/mkv/avi）
}

struct ThumbnailOptions {
    path: String,         // 视频路径
    count: u32,           // 缩略图数量（默认 20）
    max_width: u32,       // 缩略图最大宽度（默认 160px）
}

struct ThumbnailResult {
    images: Vec<String>,  // base64 编码的缩略图列表
    timestamps: Vec<f64>, // 每张缩略图对应的时间点
}

struct VideoCropOptions {
    start_time: f64,      // 起始时间（秒）
    end_time: f64,        // 结束时间（秒）
    use_ffmpeg: bool,     // 是否使用 ffmpeg
    output_path: Option<String>, // 自定义输出路径
}

struct CropResult {
    output_path: String,
    output_size: u64,
    duration: f64,
    actual_start: Option<f64>,  // 纯 Rust 路径的实际裁剪起点（关键帧对齐后）
    actual_end: Option<f64>,    // 纯 Rust 路径的实际裁剪终点
}
```

### 5.2 Tauri Commands

| Command | 功能 | 纯 Rust 降级 |
|---------|------|-------------|
| `get_video_info(path)` | 读取视频元信息 | mp4 crate 解析 MOOV atom |
| `extract_thumbnails(path, count)` | 抽 N 帧缩略图 | 无降级（返回空数组，前端切换纯文本时间轴） |
| `video_crop(path, options)` | 裁剪视频 | mp4 关键帧无损裁剪 |

### 5.3 关键帧裁剪逻辑（mp4 crate）

```
1. 解析 MP4 文件 → 读取 MOOV → track → stss（关键帧索引表）
2. 将起止时间转为 sample 索引
3. 找到 start_time 之前最近的关键帧 → 作为实际裁剪起点
4. 找到 end_time 之后最近的关键帧 → 作为实际裁剪终点
5. 重写 MP4：复制 ftyp/moov 头 + 裁剪后的 mdat 数据
6. 更新 moov 中的 duration 和 sample table
```

**精度说明：** 纯 Rust 路径裁剪点对齐到最近关键帧，实际区间可能比用户指定区间稍大（最大误差 = GOP 长度，通常 1-10 秒）。CropResult 中的 `actual_start`/`actual_end` 返回实际裁剪区间。

**格式限制：** 纯 Rust 路径仅支持 MP4 容器。ffmpeg 路径支持所有 ffmpeg 兼容格式。

### 5.4 ffmpeg 裁剪逻辑

```
ffmpeg -y -ss {start} -t {duration} -i {input} -c copy {output}
```

使用 `-c copy` 关键帧模式，无损、速度快。ffmpeg 自动处理关键帧对齐，精度优于纯 Rust 手动解析。

### 5.5 缩略图提取（ffmpeg）

```
ffmpeg -ss {timestamp} -i {input} -vframes 1 -vf scale={max_width}:-1 {output}.jpg
```

均匀抽取 N 帧，每帧返回 base64 编码的 JPEG。

## 6. 前端设计

### 6.1 页面布局

```
┌──────────────────────────────────────────────┐
│  ffmpeg 状态横幅                              │
│  🚀 ffmpeg 已启用 / 💡 未检测到 ffmpeg         │
├──────────────────────────────────────────────┤
│  Tab 栏（sticky）                             │
│  [ 视频裁剪 ]                                 │
├──────────────────────────────────────────────┤
│  选择视频文件                                 │
│  [ 打开文件 ]                                 │
│  文件名  |  时长 | 分辨率 | 编码 | 帧率 | 大小  │
├──────────────────────────────────────────────┤
│  缩略图时间轴（有 ffmpeg）                     │
│  ┌──┬──┬──┬──┬──┬──┬──┬──┐                  │
│  │  │  │  │  │  │  │  │  │  ← 缩略图条带      │
│  └──┴──┴──┴──┴──┴──┴──┴──┘                  │
│  ●══════════════════════●                    │  ← 滑块 + 选中区域
│  00:05.0                          01:42.0    │
│                                              │
│  或 纯文本时间轴（无 ffmpeg）                   │
│  ────────────────────────────────────────    │
│  ●══════════════════════●                    │
│  00:05.0                          01:42.0    │
├──────────────────────────────────────────────┤
│  裁剪设置                                     │
│  起始时间 [00:05.0] 秒    结束时间 [01:42.0] 秒 │
│  片段时长: 01:37.0                            │
│  实际裁剪区间（关键帧对齐）: 00:04.2 - 01:44.8  │  ← 仅纯 Rust 路径显示
│  ☑ 与源文件相同路径                            │
├──────────────────────────────────────────────┤
│  操作                                        │
│  [ 裁剪并导出 ]  [ 重置 ]                      │
│  ████████████████░░░░ 80%                    │
└──────────────────────────────────────────────┘
```

### 6.2 交互逻辑

- **缩略图时间轴**：Canvas 绘制，拖拽滑块选区间，点击时间轴定位滑块，选区高亮
- **纯文本时间轴**：与缩略图版同一套交互逻辑，Canvas 不绘制缩略图，仅绘制时间轴 + 选中区域
- **裁剪模式切换**：根据 `useFfmpeg` 自动切换，无需用户操作
- **实际裁剪区间提示**：纯 Rust 路径裁剪后，CropResult 返回 `actual_start`/`actual_end`，前端展示关键帧对齐后的实际起止时间

### 6.3 与 AudioTool 的关键差异

| 特性 | AudioTool | VideoTool |
|------|-----------|-----------|
| 可视化 | 波形图（纯 Rust 生成） | 缩略图（需 ffmpeg） |
| 预览 | 音频片段播放 | 无（视频预览太复杂，不做） |
| 输出格式 | MP3/WAV | 仅 MP4（同格式输出） |
| 精度提示 | 无 | 关键帧对齐提示 |

### 6.4 状态管理

```typescript
const activeTab = ref('crop')
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
```

## 7. 数据流

```
用户点击 [裁剪并导出]
  │
  ├─ useFfmpeg = true
  │   └─ ffmpeg -ss {start} -t {duration} -i {input} -c copy {output}
  │       ├─ 进度: 解析 stderr time= 上报前端
  │       └─ 完成: 返回 CropResult
  │
  └─ useFfmpeg = false
      └─ do_video_crop_keyframe()
          ├─ 解析 stss 找到关键帧位置
          ├─ 计算实际裁剪区间（对齐到最近关键帧）
          ├─ 重写 MP4 容器
          ├─ 进度: 解析/写入/完成 三个阶段上报
          └─ 完成: 返回 CropResult + actual_start/actual_end
```

## 8. 错误处理

| 场景 | 处理 |
|------|------|
| 非 MP4 格式 + 无 ffmpeg | 提示"仅支持 MP4 格式，安装 ffmpeg 可支持更多格式" |
| 文件损坏 / 解析失败 | mp4 crate 解析错误 → 前端展示具体错误信息 |
| 裁剪区间 < 0.1 秒 | 拒绝，提示"裁剪区间不能小于 0.1 秒" |
| 关键帧裁剪区间为空 | 提示"裁剪区间内无关键帧，请扩大区间或使用 ffmpeg" |
| 磁盘空间不足 | std::fs::write 错误 → 展示具体错误 |
| 输出路径已存在 | 覆盖写入（与音频工具一致） |
| ffmpeg 执行失败 | 展示 stderr 前 200 字符 |

## 9. 进度事件

- `video-crop-progress` 事件，payload: `{ progress: 0-100 }`
- 纯 Rust 路径：解析 10% → 写入 50% → 完成 100%
- ffmpeg 路径：解析 stderr 中的 `time=` 字段计算进度百分比

## 10. 测试策略

| 层 | 内容 |
|----|------|
| Rust 单元测试 | `mp4` 解析关键帧提取、时间→sample 转换 |
| 手动验证 | MP4 文件裁剪（关键帧 / ffmpeg 两种路径）、无 ffmpeg 降级、缩略图提取、大文件处理 |

## 11. 版本计划

- 版本号: **5.2.0**（新增侧边栏菜单项，minor 版本升级）
- 体积增量: ~1-2 MB（mp4 crate）
- 新依赖: `mp4` crate