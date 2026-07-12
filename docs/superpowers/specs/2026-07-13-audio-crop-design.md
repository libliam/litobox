# 音频裁剪工具 - 设计文档

**日期**: 2026-07-13
**版本**: V5.0
**类型**: 新增功能

---

## 1. 背景

LitoBox 当前缺少音频处理能力。用户需要一个轻量级的音频裁剪工具，用于快速截取音频片段（如铃声音频提取、会议录音片段截取等），无需安装 Audacity 等重型软件。

**目标**：纯 Rust 实现音频裁剪工具，支持 MP3/WAV 格式，提供波形可视化 + 滑块拖拽 + 实时预览的交互体验。

---

## 2. 方案概述

**核心思路**：纯 Rust 后端（symphonia 解码 + LAME/hound 编码）+ 前端 Canvas 波形渲染 + Web Audio API 预览。

**技术栈**：
- 解码：`symphonia`（纯 Rust，支持 MP3/WAV）
- MP3 编码：`mp3lame-encoder`（LAME 纯 Rust 绑定）
- WAV 编码：`hound`（纯 Rust）
- 波形渲染：前端 Canvas 2D
- 音频预览：前端 Web Audio API（`AudioContext.decodeAudioData()`）

**体积增量**：~3-5 MB（symphonia ~2MB, lame-sys ~1.5MB, hound ~0.3MB）

---

## 3. 功能范围

### 3.1 包含

- 打开 MP3/WAV 音频文件，显示文件信息（时长/采样率/声道/比特率）
- 波形可视化展示（Canvas 渲染，~2000 个数据点）
- 滑块拖拽设定起止时间（秒级精度，0.1 秒步进）
- 起止时间输入框与滑块双向绑定
- 实时预览选中片段（后端解码 PCM → 前端 Web Audio API 播放）
- 单段裁剪导出（可选 MP3/WAV 输出格式）
- 裁剪进度反馈（大文件）

### 3.2 不包含

- 多段裁剪 + 拼接
- 淡入淡出效果
- 音量调节
- 条形码/OGG/FLAC/AAC 等其他格式
- 工作流集成
- 操作历史记录

---

## 4. 后端设计

### 4.1 依赖

```toml
symphonia = { version = "0.5", features = ["mp3", "wav"], default-features = false }
symphonia-core = "0.5"
hound = "3.5"
mp3lame-encoder = "0.5"
```

### 4.2 模块结构

新增 `src-tauri/src/audio_tools.rs`：

```rust
pub struct AudioInfo {
    pub duration: f64,       // 秒
    pub sample_rate: u32,    // Hz
    pub channels: u16,       // 1=单声道, 2=立体声
    pub format: String,      // "mp3" / "wav"
    pub bitrate: u32,        // kbps
    pub file_size: u64,      // 字节
}

pub struct CropOptions {
    pub start_time: f64,     // 秒
    pub end_time: f64,       // 秒
    pub output_format: String, // "mp3" / "wav"
    pub mp3_bitrate: u32,    // 默认 192
}

pub struct CropResult {
    pub output_path: String,
    pub output_size: u64,
    pub duration: f64,
}
```

### 4.3 Tauri 命令

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_audio_info` | `path: String` | `AudioInfo` | 读取文件元信息 |
| `generate_waveform` | `path: String` | `Vec<f32>` | 生成波形数据点（~2000 点） |
| `audio_crop` | `path: String, options: CropOptions` | `CropResult` | 执行裁剪编码 |
| `get_audio_preview` | `path: String, start: f64, end: f64` | `Vec<u8>` | 返回 PCM 数据供前端播放 |
| `save_audio_file` | `data: Vec<u8>, suggested_name: String` | `String` | 保存文件到用户指定路径 |

### 4.4 裁剪流程

```
audio_crop:
1. 打开文件，symphonia 创建 reader
2. 通过 duration 计算起始帧偏移，跳转到起始位置
3. 从起始帧开始解码直到结束时间，收集 PCM samples
4. 根据 output_format 编码：
   - MP3: mp3lame-encoder 编码（默认 192kbps CBR）
   - WAV: hound 写入 PCM 原始数据
5. 写入临时文件，返回路径
```

### 4.5 线程安全

所有命令使用 `async fn` + `tauri::async_runtime::spawn_blocking`，与现有图片处理架构一致。

### 4.6 进度事件

裁剪大文件时通过 Tauri 事件上报进度：

```
事件: audio-crop-progress
payload: { current: f64, total: f64 }  // 秒
```

---

## 5. 前端设计

### 5.1 页面结构

单页面布局（非多 Tab），按操作流程从上到下：

```
┌─ tool-card: 选择音频文件 ──────────────────┐
│  [打开文件]  📁 filename.mp3  (时长/格式...) │
└───────────────────────────────────────────┘

┌─ tool-card: 波形预览 ──────────────────────┐
│  ┌─────────────────────────────────────┐   │
│  │  Canvas 波形图（~2000 数据点）        │   │
│  │  ▲──────────────△                  │   │
│  │  起始滑块         结束滑块            │   │
│  │  00:15.0          02:45.0           │   │
│  └─────────────────────────────────────┘   │
│  [▶ 预览选中区域] [⏹ 停止]                  │
└───────────────────────────────────────────┘

┌─ tool-card: 裁剪设置 ──────────────────────┐
│  起始时间: [00:15.0] 秒                     │
│  结束时间: [02:45.0] 秒                     │
│  输出格式: [MP3 ▼]  MP3比特率: [192kbps ▼]   │
│  (比特率选项: 128/192/256/320 kbps CBR)       │
│  片段时长: 02:30.0                          │
└───────────────────────────────────────────┘

┌─ tool-card: 操作 ──────────────────────────┐
│  [✂ 裁剪并导出]  [↻ 重置]                   │
│  ElProgress 进度条（裁剪时显示）              │
└───────────────────────────────────────────┘
```

### 5.2 组件结构

`src/views/AudioTool.vue`，基于 `_ToolTemplate.vue` 模板：

```typescript
// 状态
const filePath = ref('')
const audioInfo = ref<AudioInfo | null>(null)
const waveformData = ref<number[]>([])
const startTime = ref(0)
const endTime = ref(0)
const outputFormat = ref<'mp3' | 'wav'>('mp3')
const mp3Bitrate = ref(192)
const isProcessing = ref(false)
const isPreviewing = ref(false)
const cropProgress = ref(0)
const error = ref('')

// 计算属性
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// 波形 Canvas
const canvasRef = ref<HTMLCanvasElement>()
function drawWaveform() { /* Canvas 绘制波形 + 选中区域高亮 */ }
function handleSliderDrag() { /* 滑块拖拽逻辑 */ }
function handleCanvasClick() { /* 点击定位 */ }

// 操作
async function openFile() { /* dialog.open → get_audio_info → generate_waveform */ }
async function previewAudio() { /* get_audio_preview → AudioContext.decodeAudioData → play */ }
function stopPreview() { /* AudioContext 停止 */ }
async function cropAudio() { /* audio_crop → save_audio_file */ }
function resetForm() { /* 清空所有状态 */ }
```

### 5.3 波形渲染

- Canvas 宽度自适应，高度固定 200px
- 波形数据点 ~2000 个，等距绘制垂直线段
- 选中区域（`startTime` ~ `endTime`）用 `--accent-cyan` 颜色高亮，未选中区域用 `--text-secondary` 半透明
- 波峰颜色：`--accent-cyan`
- 背景颜色：`--bg-input`
- 两个圆形滑块在波形上，可拖拽调整起止位置
- 拖拽时底部时间实时更新，拖拽步进 0.1 秒

### 5.4 音频预览

- 点击"预览选中区域" → 调用 `get_audio_preview(start, end)` 返回 PCM 字节
- 前端 `AudioContext.decodeAudioData()` 解码 → 播放
- 播放时按钮变为"停止"，点击停止播放
- 必须使用用户手势触发的 AudioContext（浏览器安全策略），按钮点击时 `audioCtx.resume()`

### 5.5 样式规范

- 使用全局 CSS 变量（`--bg-primary`, `--bg-card`, `--text-primary`, `--accent-cyan` 等）
- 使用全局类名（`.tool-card`, `.card-header`, `.card-body`, `.error-message` 等）
- 禁止硬编码颜色和内联样式
- scoped 样式中仅定义页面特有样式

---

## 6. 代码变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | 添加 symphonia, hound, mp3lame-encoder 依赖 |
| `src-tauri/src/audio_tools.rs` | 新增 | 音频处理模块（~400 行） |
| `src-tauri/src/main.rs` | 修改 | 注册 audio_tools 模块和 Tauri 命令 |
| `src/views/AudioTool.vue` | 新增 | 音频裁剪页面（~350 行） |
| `src/store/index.ts` | 修改 | TOOL_LIST 添加 AudioTool 条目（category: 'utility'） |
| `src/App.vue` | 修改 | 注册 AudioTool 组件映射 |

---

## 7. 错误处理

| 场景 | 处理 |
|------|------|
| 不支持的文件格式 | 后端返回错误 "不支持的音频格式，仅支持 MP3/WAV" |
| 文件损坏/解码失败 | symphonia 返回错误 → 前端 `.error-message` 展示 |
| 起止时间非法（起始 >= 结束或超出时长） | 前端校验 + 后端双重校验 |
| 裁剪区间过短（< 0.1 秒） | 前端校验，最小 0.1 秒 |
| 编码失败（磁盘满/权限） | 捕获 IO 错误，前端展示具体原因 |
| 大文件（> 500MB） | 流式解码，进度事件上报；前端显示 ElProgress |
| 预览播放失败 | 捕获错误，提示"预览播放失败" |

---

## 8. 测试要点

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
- [ ] 大文件裁剪（> 100MB），验证进度条显示
- [ ] 不支持格式打开，验证错误提示
- [ ] 非法时间输入，验证前端拦截
- [ ] `npm run build` 构建通过