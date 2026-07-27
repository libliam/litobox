# 音频工具 ffmpeg 扩展功能 Spec

> **版本**: V5.4  
> **日期**: 2026-07-16  
> **状态**: ✅ 已完成

---

## 1. 功能概述

在现有「音频裁剪」基础上，新增 4 个 Tab，全部基于 ffmpeg 实现（运行时探测，无 ffmpeg 则降级提示）：

| Tab | 功能 | 核心能力 |
|-----|------|---------|
| 格式转换 | 音频格式互转 | MP3/WAV/M4A/AAC/FLAC/OGG 互转 |
| 音频压缩 | 调整比特率/采样率压缩体积 | 可选目标比特率或质量等级 |
| 音频合并 | 多文件拼接为一个 | 顺序拖拽排列，同格式 concat / 跨格式转码合并 |
| 变速变调 | 调整播放速度/音调 | atempo 滤镜，0.5x~4x 变速 |

---

## 2. 通用设计原则

### 2.1 ffmpeg 依赖策略
- **运行时探测**：`check_ffmpeg()` 检测系统是否安装 ffmpeg
- **ffmpeg 模式**：所有新功能仅 ffmpeg 模式可用，无 ffmpeg 则显示提示横幅
- **复用现有架构**：复用 `video_tools::ensure_ffmpeg_in_path()` 和 `CREATE_NO_WINDOW` 常量

### 2.2 UI 规范
- **Tab 栏置顶**：`.tool-card.sticky-card` + 自定义 class `audio-tool-tabs`
- **卡片式布局**：每个操作区用 `.tool-card`，包含 `.card-header` + `.card-body`
- **按钮 loading**：`:loading="isProcessing"`，不用 ElLoading 全屏锁
- **进度事件**：`audio-xxx-progress` 事件 + `el-progress` 组件
- **错误提示**：`error.value` + `.error-message` 样式

### 2.3 后端命令规范
- **async + spawn_blocking**：所有耗时操作必须后台执行
- **debug_log!**：关键逻辑分支添加日志
- **路径处理**：canonicalize 后去掉 `\\?\` 前缀
- **进度上报**：`app_handle.emit("audio-xxx-progress", { progress: N })`

---

## 3. 功能详细设计

### 3.1 格式转换 Tab

**功能**：将音频文件转换为其他格式

**UI 布局**：
```
┌─ 选择音频文件 ─────────────────────────┐
│ [打开文件]                              │
│ 文件名.mp3 | 03:45 | MP3 | 44100Hz | 立体声 | 192kbps │
└─────────────────────────────────────────┘

┌─ 转换设置 ─────────────────────────────┐
│ 输出格式: [MP3 ▼]                       │
│ 质量设置: [192 kbps ▼] (仅 MP3/AAC)     │
│ 采样率: [原始 ▼] (44100/48000/96000)    │
│ 声道: [原始 ▼] (单声道/立体声)          │
└─────────────────────────────────────────┘

┌─ 操作 ─────────────────────────────────┐
│ [转换并导出] [重置]                     │
│ ████████████░░░░ 75%                    │
└─────────────────────────────────────────┘
```

**后端命令**：`audio_convert`
```rust
pub struct ConvertOptions {
    pub output_format: String,  // mp3/wav/m4a/aac/flac/ogg
    pub bitrate: Option<u32>,   // kbps (仅 MP3/AAC)
    pub sample_rate: Option<u32>, // Hz
    pub channels: Option<u16>,  // 1=单声道, 2=立体声
    pub output_path: Option<String>,
}

pub async fn audio_convert(
    app_handle: tauri::AppHandle,
    path: String,
    options: ConvertOptions,
) -> Result<ConvertResult, String>
```

**ffmpeg 命令示例**：
```bash
# MP3 192kbps
ffmpeg -i input.wav -acodec libmp3lame -b:a 192k output.mp3

# AAC 256kbps
ffmpeg -i input.m4a -acodec aac -b:a 256k output.m4a

# FLAC 无损
ffmpeg -i input.wav -acodec flac output.flac

# OGG 192kbps
ffmpeg -i input.mp3 -acodec libvorbis -b:a 192k output.ogg

# WAV PCM
ffmpeg -i input.mp3 -acodec pcm_s16le output.wav
```

---

### 3.2 音频压缩 Tab

**功能**：调整比特率/采样率压缩音频体积

**UI 布局**：
```
┌─ 选择音频文件 ─────────────────────────┐
│ [打开文件]                              │
│ 文件名.mp3 | 03:45 | 5.2 MB | 192kbps  │
└─────────────────────────────────────────┘

┌─ 压缩设置 ─────────────────────────────┐
│ 压缩模式: [目标比特率 ▼]                │
│   - 目标比特率: [128 kbps ▼]            │
│   - 或质量等级: [中等 ▼] (低/中/高)     │
│ 采样率: [原始 ▼] (保持/44100/22050)     │
│ 预估大小: ~2.6 MB (原始 5.2 MB)         │
└─────────────────────────────────────────┘

┌─ 操作 ─────────────────────────────────┐
│ [压缩并导出] [重置]                     │
│ ████████████░░░░ 75%                    │
└─────────────────────────────────────────┘
```

**后端命令**：`audio_compress`
```rust
pub struct CompressOptions {
    pub mode: String,  // "bitrate" 或 "quality"
    pub bitrate: Option<u32>,  // kbps
    pub quality: Option<String>,  // "low"/"medium"/"high"
    pub sample_rate: Option<u32>,
    pub output_path: Option<String>,
}

pub async fn audio_compress(
    app_handle: tauri::AppHandle,
    path: String,
    options: CompressOptions,
) -> Result<CompressResult, String>
```

**质量等级映射**：
- 低：64 kbps (MP3) / 48 kbps (AAC)
- 中：128 kbps (MP3) / 96 kbps (AAC)
- 高：192 kbps (MP3) / 128 kbps (AAC)

**ffmpeg 命令示例**：
```bash
# 目标比特率
ffmpeg -i input.mp3 -acodec libmp3lame -b:a 128k output.mp3

# 质量等级 (VBR)
ffmpeg -i input.mp3 -acodec libmp3lame -q:a 5 output.mp3  # 中等质量
```

---

### 3.3 音频合并 Tab

**功能**：多个音频文件拼接为一个

**UI 布局**：
```
┌─ 添加音频文件 ─────────────────────────┐
│ [添加文件] [清空列表]                   │
│                                         │
│ 文件列表 (拖拽排序):                    │
│ 1. 🎵 歌曲1.mp3 (03:45) [删除]         │
│ 2. 🎵 歌曲2.wav (02:30) [删除]         │
│ 3. 🎵 歌曲3.m4a (04:15) [删除]         │
│                                         │
│ 总时长: 10:30                           │
└─────────────────────────────────────────┘

┌─ 合并设置 ─────────────────────────────┐
│ 输出格式: [MP3 ▼]                       │
│ 比特率: [192 kbps ▼]                    │
│ 合并模式: [自动 ▼]                      │
│   - 自动: 同格式 concat，跨格式转码     │
│   - 强制转码: 统一转码后合并            │
└─────────────────────────────────────────┘

┌─ 操作 ─────────────────────────────────┐
│ [合并并导出] [重置]                     │
│ ████████████░░░░ 75%                    │
└─────────────────────────────────────────┘
```

**后端命令**：`audio_merge`
```rust
pub struct MergeOptions {
    pub input_paths: Vec<String>,
    pub output_format: String,
    pub bitrate: u32,
    pub mode: String,  // "auto" 或 "force_transcode"
    pub output_path: Option<String>,
}

pub async fn audio_merge(
    app_handle: tauri::AppHandle,
    options: MergeOptions,
) -> Result<MergeResult, String>
```

**ffmpeg 命令示例**：
```bash
# 同格式 concat (无损快速)
ffmpeg -f concat -safe 0 -i filelist.txt -c copy output.mp3

# 跨格式转码合并
ffmpeg -i input1.mp3 -i input2.wav -filter_complex "[0:a][1:a]concat=n=2:v=0:a=1[out]" -map "[out]" output.mp3
```

**filelist.txt 格式**：
```
file 'path/to/audio1.mp3'
file 'path/to/audio2.mp3'
```

---

### 3.4 变速变调 Tab

**功能**：调整音频播放速度（可选保持音调）

**UI 布局**：
```
┌─ 选择音频文件 ─────────────────────────┐
│ [打开文件]                              │
│ 文件名.mp3 | 03:45 | MP3 | 192kbps     │
└─────────────────────────────────────────┘

┌─ 变速设置 ─────────────────────────────┐
│ 播放速度: [1.0x ▼] (0.5x~4.0x)         │
│ 保持音调: [✓] (atempo vs asetrate)      │
│                                         │
│ 预览: [▶ 播放 10 秒]                   │
│                                         │
│ 输出时长: 03:45 → 01:52 (2.0x)          │
└─────────────────────────────────────────┘

┌─ 操作 ─────────────────────────────────┐
│ [导出并保存] [重置]                     │
│ ████████████░░░░ 75%                    │
└─────────────────────────────────────────┘
```

**后端命令**：`audio_speed_change`
```rust
pub struct SpeedChangeOptions {
    pub speed: f64,  // 0.5~4.0
    pub keep_pitch: bool,  // true=atempo, false=asetrate
    pub output_format: String,
    pub bitrate: u32,
    pub output_path: Option<String>,
}

pub async fn audio_speed_change(
    app_handle: tauri::AppHandle,
    path: String,
    options: SpeedChangeOptions,
) -> Result<SpeedChangeResult, String>
```

**ffmpeg 命令示例**：
```bash
# 保持音调 (atempo)
ffmpeg -i input.mp3 -filter:a "atempo=2.0" output.mp3

# 不保持音调 (asetrate + aresample)
ffmpeg -i input.mp3 -filter:a "asetrate=44100*2.0,aresample=44100" output.mp3

# 0.5x 慢速 (atempo 只支持 0.5~2.0，需链式)
ffmpeg -i input.mp3 -filter:a "atempo=0.5" output.mp3
```

**atempo 限制处理**：
- atempo 只支持 0.5~2.0
- 超出范围需链式：如 4.0x = atempo=2.0,atempo=2.0
- 0.25x = atempo=0.5,atempo=0.5

---

## 4. 前端实现要点

### 4.1 Tab 状态管理
```typescript
const activeTab = ref('crop')  // crop/convert/compress/merge/speed

// 每个 Tab 独立维护状态
const convertState = reactive({ ... })
const compressState = reactive({ ... })
const mergeState = reactive({ ... })
const speedState = reactive({ ... })
```

### 4.2 文件选择器
```typescript
// 格式转换/压缩/变速：单文件
const selected = await open({
  filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
  multiple: false,
})

// 音频合并：多文件
const selected = await open({
  filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a', 'aac', 'flac', 'ogg'] }],
  multiple: true,
})
```

### 4.3 进度监听
```typescript
const unlisten = await listen<{ progress: number }>('audio-convert-progress', (event) => {
  convertProgress.value = Math.round(event.payload.progress)
})
```

### 4.4 拖拽排序（音频合并）
使用 Element Plus 的 `el-table` + `row-class-name` + 原生拖拽事件实现

---

## 5. 后端实现要点

### 5.1 新增命令注册
在 `main.rs` 中注册：
```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    audio_convert,
    audio_compress,
    audio_merge,
    audio_speed_change,
])
```

### 5.2 ffmpeg 命令构建
- 统一使用 `Command::new("ffmpeg")` + `CREATE_NO_WINDOW`
- 参数构建用 `Vec<String>` 动态拼接
- 错误处理：解析 stderr 输出

### 5.3 进度上报策略
- 开始：10%
- 处理中：50%
- 完成：100%
- ffmpeg 输出解析（可选）：解析 `progress=...` 行

### 5.4 临时文件管理
- 音频合并的 filelist.txt 写入临时目录
- 使用 `std::env::temp_dir()` + 唯一文件名
- 处理完成后删除临时文件

---

## 6. 测试检查清单

### 6.1 格式转换
- [ ] MP3 → WAV
- [ ] WAV → MP3
- [ ] M4A → FLAC
- [ ] MP3 → OGG
- [ ] 采样率转换（44100 → 48000）
- [ ] 声道转换（立体声 → 单声道）

### 6.2 音频压缩
- [ ] 192kbps → 128kbps
- [ ] 质量等级（低/中/高）
- [ ] 采样率降低（44100 → 22050）
- [ ] 文件大小对比

### 6.3 音频合并
- [ ] 2 个 MP3 合并
- [ ] 3 个不同格式合并（MP3+WAV+M4A）
- [ ] 拖拽排序后合并
- [ ] 合并后播放验证无断点

### 6.4 变速变调
- [ ] 2.0x 加速（保持音调）
- [ ] 0.5x 减速（保持音调）
- [ ] 4.0x 加速（链式 atempo）
- [ ] 不保持音调（音调升高）

---

## 7. 版本号更新

- **当前版本**: V5.1
- **新版本**: V5.2
- **更新内容**:
  - 音频工具新增 4 个 Tab：格式转换、音频压缩、音频合并、变速变调
  - 全部基于 ffmpeg 实现，运行时探测
  - 更新 README.md 功能阶段记录

---

## 8. 实现顺序

1. **格式转换**（基础，其他功能复用）
2. **音频压缩**（复用转换逻辑）
3. **变速变调**（独立功能）
4. **音频合并**（最复杂，拖拽排序 + concat）
5. **UI 优化 + 测试**

---

## 9. 已知限制

- **无 ffmpeg 时不可用**：显示提示横幅，引导安装
- **格式支持**：依赖 ffmpeg 编码器，常见格式均支持
- **变速范围**：atempo 限制 0.5~2.0，超出需链式
- **合并性能**：同格式 concat 快速，跨格式转码较慢

---

## 10. 未来扩展（可选）

- 淡入淡出效果（afade 滤镜）
- 音量调整（volume 滤镜）
- 降噪（afftdn 滤镜）
- 均衡器（equalizer 滤镜）
- 音频可视化（频谱图）
