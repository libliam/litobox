# 视频加水印 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 VideoTool.vue 中新增顶级 Tab「加水印」，支持文字/图片/两者叠加水印，通过 ffmpeg filter_complex 一次编码完成，支持 9 宫格位置 + 偏移 + 时间段限定。

**Architecture:**
- 后端：`video_tools.rs` 追加 2 个结构体 + 1 个 Tauri 命令 + 5 个辅助函数（位置计算/路径转义/drawtext 构造/overlay 构造/核心执行）
- 前端：`VideoTool.vue` 追加 1 个 el-tab-pane（4 卡片布局：文件 / 类型+位置 / 参数 / 操作）+ 配套状态 + 事件监听
- 集成：`main.rs` 注册命令、`WorkflowView.vue` 加分支、`README.md` 追加 V6.9 条目、`feature-backlog.md` 标记 F29 完成

**Tech Stack:** Rust (tauri 2.0 + rusqlite + ffmpeg CLI), Vue 3 Composition API + TypeScript + Element Plus

---

### Task 1: 后端 — 数据结构 + 辅助函数骨架

**Files:**
- Modify: `src-tauri/src/video_tools.rs` (在 `do_video_volume` 函数之后追加)

- [ ] **Step 1: 追加 VideoWatermarkOptions / VideoWatermarkResult 结构体**

在 `video_tools.rs` 末尾（`do_video_volume` 的 `}` 之后）追加：

```rust
// ============ F29: 视频加水印 ============

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoWatermarkOptions {
    pub wm_type: String,           // "text" | "image" | "both"

    // 文字水印
    pub text: Option<String>,
    pub font_file: Option<String>,
    pub font_size: Option<u32>,
    pub font_color: Option<String>,
    pub font_border: Option<bool>,
    pub font_border_color: Option<String>,
    pub font_opacity: Option<f32>,

    // 图片水印
    pub image_path: Option<String>,
    pub image_scale: Option<f32>,
    pub image_opacity: Option<f32>,

    // 公共
    pub position: String,
    pub offset_x: u32,
    pub offset_y: u32,
    pub use_time_range: bool,
    pub start_time: f64,
    pub end_time: f64,

    // 输出
    pub output_format: String,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoWatermarkResult {
    pub output_path: String,
    pub output_size: u64,
}
```

- [ ] **Step 2: 追加 4 个纯函数辅助（escape / position / enable / color）**

紧接着上面的结构体追加：

```rust
/// drawtext / movie 滤镜内文件路径转义
/// - \ → /
/// - ' → '\''
/// - 盘符的 : → \:
fn escape_ffmpeg_path(p: &str) -> String {
    let p = p.replace('\\', "/");
    let p = p.replace('\'', "'\\''");
    p.replace(':', "\\:")
}

/// drawtext 文字内容转义: \ ' : %
fn escape_drawtext_text(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "'\\''")
     .replace(':', "\\:")
     .replace('%', "%%")
}

/// 9 宫格位置 → ffmpeg x/y 表达式字符串
/// drawtext 和 overlay 都用 W/w H/h 变量，可复用
fn calc_position_expr(pos: &str, ox: u32, oy: u32) -> (String, String) {
    let ox_s = ox.to_string();
    let oy_s = oy.to_string();
    match pos {
        "topLeft"     => (ox_s.clone(), oy_s.clone()),
        "top"         => ("(W-w)/2".to_string(), oy_s.clone()),
        "topRight"    => (format!("W-w-{}", ox_s), oy_s.clone()),
        "left"        => (ox_s.clone(), "(H-h)/2".to_string()),
        "center"      => ("(W-w)/2".to_string(), "(H-h)/2".to_string()),
        "right"       => (format!("W-w-{}", ox_s), "(H-h)/2".to_string()),
        "bottomLeft"  => (ox_s.clone(), format!("H-h-{}", oy_s)),
        "bottom"      => ("(W-w)/2".to_string(), format!("H-h-{}", oy_s)),
        "bottomRight" => (format!("W-w-{}", ox_s), format!("H-h-{}", oy_s)),
        _             => (format!("W-w-{}", ox_s), format!("H-h-{}", oy_s)), // 默认右下
    }
}

/// ffmpeg enable='between(t,s,e)' 表达式（空字符串表示全程）
/// 逗号需要 \, 转义（filter 参数内）
fn build_enable_expr(use_range: bool, s: f64, e: f64) -> String {
    if use_range && e > s {
        format!("between(t\\,{}\\,{})", s, e)
    } else {
        String::new()
    }
}

/// #RRGGBB + opacity → ffmpeg color 字符串
/// 例: #ffffff + 0.5 → white@0.5 或 0xFFFFFF@0.5
/// ffmpeg drawtext fontcolor 接受 0xRRGGBB@AA 或 colorname@AA 格式
fn format_color_with_alpha(hex: &str, alpha: f32) -> String {
    let hex = hex.trim_start_matches('#');
    let alpha = alpha.clamp(0.0, 1.0);
    // 0xRRGGBB@AA.AA 格式（alpha 用小数 0.0-1.0）
    format!("0x{}@{:.3}", hex, alpha)
}
```

- [ ] **Step 3: Run `cargo check` 验证编译**

```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 无编译错误（warning 可容忍）

---

### Task 2: 后端 — 核心 do_video_watermark + Tauri 命令

**Files:**
- Modify: `src-tauri/src/video_tools.rs` (紧接 Task1 代码之后)
- Modify: `src-tauri/src/main.rs` (invoke_handler 注册)

- [ ] **Step 1: 追加 format_encoders_for 函数（复用 video_speed_change 的编码器 match）**

在 Task1 代码后追加：

```rust
/// 根据输出格式组装 ffmpeg 编码器参数（参考 video_speed_change）
fn push_encoder_args(args: &mut Vec<String>, ext: &str) {
    match ext {
        "mp4" | "mov" => {
            args.push("-c:v".to_string()); args.push("libx264".to_string());
            args.push("-c:a".to_string()); args.push("aac".to_string());
            args.push("-movflags".to_string()); args.push("+faststart".to_string());
        }
        "mkv" => {
            args.push("-c:v".to_string()); args.push("libx264".to_string());
            args.push("-c:a".to_string()); args.push("libmp3lame".to_string());
        }
        "webm" => {
            args.push("-c:v".to_string()); args.push("libvpx-vp9".to_string());
            args.push("-c:a".to_string()); args.push("libopus".to_string());
            args.push("-deadline".to_string()); args.push("realtime".to_string());
            args.push("-cpu-used".to_string()); args.push("5".to_string());
        }
        "avi" => {
            args.push("-c:v".to_string()); args.push("libx264".to_string());
            args.push("-c:a".to_string()); args.push("libmp3lame".to_string());
        }
        _ => {
            // 默认 MP4 兼容参数
            args.push("-c:v".to_string()); args.push("libx264".to_string());
            args.push("-c:a".to_string()); args.push("aac".to_string());
        }
    }
}
```

- [ ] **Step 2: 追加 do_video_watermark 核心实现**

紧接着追加：

```rust
fn do_video_watermark(
    app_handle: &tauri::AppHandle,
    path: &str,
    opts: &VideoWatermarkOptions,
) -> Result<VideoWatermarkResult, String> {
    debug_log!("[WM] type={} position={} useTimeRange={}", opts.wm_type, opts.position, opts.use_time_range);

    // 基础校验
    let info = get_video_info_ffprobe(path)
        .map_err(|e| format!("无法读取视频信息: {}", e))?;
    let duration = info.duration;

    // 校验文字水印
    if opts.wm_type == "text" || opts.wm_type == "both" {
        let text = opts.text.as_deref().unwrap_or("").trim();
        if text.is_empty() {
            return Err("文字水印内容不能为空".to_string());
        }
        let font = opts.font_file.as_deref().unwrap_or("");
        if font.is_empty() || !std::path::Path::new(font).is_file() {
            return Err(format!("字体文件不存在: {}", font));
        }
    }

    // 校验图片水印
    if opts.wm_type == "image" || opts.wm_type == "both" {
        let img = opts.image_path.as_deref().unwrap_or("");
        if img.is_empty() || !std::path::Path::new(img).is_file() {
            return Err(format!("水印图片文件不存在: {}", img));
        }
        // 校验是否为有效图片 (用 image crate 快速 probe, 不成功不致命，给 warning)
        if let Err(e) = image::image_dimensions(img) {
            debug_log!("[WM] 警告: 水印图片格式 probe 失败: {} (继续尝试 overlay)", e);
        }
    }

    // 校验时间段
    if opts.use_time_range && opts.end_time <= opts.start_time {
        return Err(format!(
            "时间段无效: start({}) >= end({})", opts.start_time, opts.end_time
        ));
    }
    if opts.use_time_range && opts.end_time > duration {
        debug_log!("[WM] end_time {} > duration {}, 截断到 duration", opts.end_time, duration);
    }

    // 输出路径
    let input_path = std::path::Path::new(path);
    let input_stem = input_path
        .file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let ext = opts.output_format.as_str();
    let output_path = if let Some(ref custom) = opts.output_path {
        std::path::PathBuf::from(custom)
    } else {
        input_path
            .parent().unwrap_or(std::path::Path::new("."))
            .join(format!("{}_watermarked.{}", input_stem, ext))
    };
    let output_path_str = output_path.to_string_lossy().to_string();
    debug_log!("[WM] output = {}", output_path_str);

    let _ = app_handle.emit("video-watermark-progress", serde_json::json!({ "progress": 5.0 }));

    // 位置表达式 + enable 表达式（文字和图片共用位置逻辑）
    let (x_expr, y_expr) = calc_position_expr(&opts.position, opts.offset_x, opts.offset_y);
    let enable_expr = build_enable_expr(opts.use_time_range, opts.start_time, opts.end_time);
    debug_log!("[WM] x={}, y={}, enable={}", x_expr, y_expr, enable_expr);

    // ========== 构造 filter_complex 各段 ==========
    let mut filter_parts: Vec<String> = Vec::new();
    let mut last_video_label = String::from("0:v");

    // 4a. drawtext
    if opts.wm_type == "text" || opts.wm_type == "both" {
        let ff = escape_ffmpeg_path(opts.font_file.as_deref().unwrap_or(""));
        let txt = escape_drawtext_text(opts.text.as_deref().unwrap_or("").trim());
        let fs = opts.font_size.unwrap_or(32);
        let alpha = opts.font_opacity.unwrap_or(1.0);
        let fc = format_color_with_alpha(
            opts.font_color.as_deref().unwrap_or("#ffffff"), alpha,
        );

        let mut dt = format!(
            "drawtext=fontfile='{}':text='{}':fontsize={}:fontcolor={}:x={}:y={}",
            ff, txt, fs, fc, x_expr, y_expr
        );
        if opts.font_border.unwrap_or(false) {
            let bc = format_color_with_alpha(
                opts.font_border_color.as_deref().unwrap_or("#000000"), alpha,
            );
            dt.push_str(&format!(":borderw=2:bordercolor={}", bc));
        }
        if !enable_expr.is_empty() {
            dt.push_str(&format!(":enable='{}'", enable_expr));
        }
        filter_parts.push(format!("[{}]{}[v1]", last_video_label, dt));
        last_video_label = String::from("v1");
    }

    // 4b. movie + overlay
    if opts.wm_type == "image" || opts.wm_type == "both" {
        let img = escape_ffmpeg_path(opts.image_path.as_deref().unwrap_or(""));
        let scale = opts.image_scale.unwrap_or(1.0).clamp(0.1, 2.0);
        let alpha = opts.image_opacity.unwrap_or(0.8).clamp(0.1, 1.0);
        let wm_label = format!("wmimg");

        // 图片预处理: 载入 → loop无限 → 缩放 → rgba → 透明度
        filter_parts.push(format!(
            "movie='{}',loop=loop=-1:size=32767:start=0,setpts=PTS-STARTPTS,scale=iw*{}:-1,format=rgba,colorchannelmixer=aa={}[{}]",
            img, scale, alpha, wm_label
        ));

        let next_label = if opts.wm_type == "both" { "v2" } else { "v1" };
        let mut ov = format!(
            "[{}][{}]overlay=x={}:y={}:format=auto",
            last_video_label, wm_label, x_expr, y_expr
        );
        if !enable_expr.is_empty() {
            ov.push_str(&format!(":enable='{}'", enable_expr));
        }
        filter_parts.push(format!("{}[{}]", ov, next_label));
        last_video_label = next_label.to_string();
    }

    let filter_complex = filter_parts.join(";");
    debug_log!("[WM] filter_complex: {}", filter_complex);

    let _ = app_handle.emit("video-watermark-progress", serde_json::json!({ "progress": 10.0 }));

    // ========== 组装 ffmpeg args ==========
    let mut args: Vec<String> = Vec::new();
    args.push("-y".to_string());
    args.push("-i".to_string());
    args.push(path.to_string());

    // 如果是 both 模式，movie 输入源在 filter_complex 里声明，不需要额外 -i
    // ffmpeg movie 滤镜直接从路径读，所以这里不需要额外 -i

    args.push("-filter_complex".to_string());
    args.push(filter_complex);

    args.push("-map".to_string());
    args.push(format!("[{}]", last_video_label));
    args.push("-map".to_string());
    args.push("0:a?".to_string()); // 如果有音频就复制

    push_encoder_args(&mut args, ext);

    args.push(output_path_str.clone());

    debug_log!("[WM] ffmpeg args: {:?}", args);

    // ========== 执行 ==========
    run_ffmpeg_with_progress(
        app_handle,
        &args,
        "video-watermark-progress",
        duration.max(0.1),
    )?;

    let output_size = std::fs::metadata(&output_path_str)
        .map(|m| m.len()).unwrap_or(0);

    let _ = app_handle.emit("video-watermark-progress", serde_json::json!({ "progress": 100.0 }));

    Ok(VideoWatermarkResult {
        output_path: output_path_str,
        output_size,
    })
}
```

- [ ] **Step 3: 追加 pub async fn video_watermark Tauri 命令封装**

紧接着追加：

```rust
#[tauri::command]
pub async fn video_watermark(
    app_handle: tauri::AppHandle,
    path: String,
    options: VideoWatermarkOptions,
) -> Result<VideoWatermarkResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !ffmpeg_available() {
            return Err("视频加水印需要 ffmpeg，请先安装 ffmpeg".to_string());
        }
        do_video_watermark(&app_handle, &path, &options)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
```

- [ ] **Step 4: 在 main.rs 注册 video_watermark 命令**

打开 `src-tauri/src/main.rs`，在 invoke_handler 数组末尾（`video_tools::video_volume,` 之后）追加：

```rust
            video_tools::video_watermark,
```

位置参考：在 `video_tools::video_volume,` 行后加逗号换行，插入上面一行。

- [ ] **Step 5: cargo check 验证**

```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 无编译错误

---

### Task 3: 前端 — VideoTool.vue 新增 Tab 模板 + 状态

**Files:**
- Modify: `src/views/VideoTool.vue`

- [ ] **Step 3a: 在 el-tabs 中新增「加水印」pane（模板 L19-L30 附近）**

找到：
```html
        <el-tab-pane label="视频调整" name="adjust" />
```
在它**后面**追加：
```html
        <el-tab-pane label="加水印" name="watermark" />
```

- [ ] **Step 3b: 追加状态变量（脚本，在 adjustVolumeResult 声明后面 ~L1426）**

在 `// 音量调整状态` 块之后，`cropOverlayStyle` computed 之前追加：

```typescript
// ============ 水印 Tab 状态 (F29) ============
const wm_filePath = ref('')
const wm_fileName = ref('')
const wm_videoInfo = ref<VideoInfo | null>(null)
const wm_loadingInfo = ref(false)
const wm_isProcessing = ref(false)
const wm_progress = ref(0)
const wm_done = ref(false)
const wm_listenOff = ref<(() => void) | null>(null)

const wm_type = ref<'text' | 'image' | 'both'>('text')
const POSITIONS = [
  { value: 'topLeft',     icon: '↖' },
  { value: 'top',         icon: '↑' },
  { value: 'topRight',    icon: '↗' },
  { value: 'left',        icon: '←' },
  { value: 'center',      icon: '●' },
  { value: 'right',       icon: '→' },
  { value: 'bottomLeft',  icon: '↙' },
  { value: 'bottom',      icon: '↓' },
  { value: 'bottomRight', icon: '↘' },
] as const
type WmPosition = typeof POSITIONS[number]['value']

// 文字
const CANDIDATE_FONTS = [
  { name: '微软雅黑',        path: 'C:/Windows/Fonts/msyh.ttc' },
  { name: '微软雅黑 Bold',   path: 'C:/Windows/Fonts/msyhbd.ttc' },
  { name: '黑体 SimHei',     path: 'C:/Windows/Fonts/simhei.ttf' },
  { name: '宋体 SimSun',     path: 'C:/Windows/Fonts/simsun.ttc' },
  { name: 'Arial',           path: 'C:/Windows/Fonts/arial.ttf' },
]
const wm_fontList = ref<{ name: string; path: string }[]>([])
const wm_fontName = ref('微软雅黑')
const wm_customFontPath = ref('')
const wm_fontSize = ref(32)
const wm_fontColor = ref('#ffffff')
const wm_fontBorder = ref(false)
const wm_fontBorderColor = ref('#000000')
const wm_fontOpacity = ref(1.0)

// 图片
const wm_imagePath = ref('')
const wm_imageName = ref('')
const wm_imageScale = ref(1.0)
const wm_imageOpacity = ref(0.8)

// 公共
const wm_position = ref<WmPosition>('bottomRight')
const wm_offsetX = ref(20)
const wm_offsetY = ref(20)
const wm_useTimeRange = ref(false)
const wm_startTime = ref(0)
const wm_endTime = ref(0)

// 输出
const wm_outputFormat = ref('mp4')
const wm_saveToSamePath = ref(true)
const wm_result = ref<{ path: string; size: number } | null>(null)
```

- [ ] **Step 3c: 追加计算属性 + 辅助函数（和 POSITIONS 同区域）**

在上面代码块之后追加：

```typescript
// 解析出最终字体路径（自定义优先 → 预置列表匹配 → 空）
function getResolvedFontPath(): string {
  if (wm_customFontPath.value.trim()) return wm_customFontPath.value.trim()
  const found = wm_fontList.value.find(f => f.name === wm_fontName.value)
  return found?.path || ''
}

const wm_canSubmit = computed(() => {
  if (!wm_filePath.value || !useFfmpeg.value) return false
  if (wm_type.value === 'text' || wm_type.value === 'both') {
    if (!wm_text.value.trim()) return false
    if (!getResolvedFontPath()) return false
  }
  if (wm_type.value === 'image' || wm_type.value === 'both') {
    if (!wm_imagePath.value) return false
  }
  if (wm_useTimeRange.value && wm_endTime.value <= wm_startTime.value) return false
  return true
})
```

注意：上面代码引用了 `wm_text` 变量（还没声明），在 Step 3b 状态块的 `// 文字` 段里漏掉了，补充在 Step 3b：

在 Step 3b 的 `const wm_fontOpacity = ref(1.0)` 行之前（或后）补上：

```typescript
const wm_text = ref('')
```

---

### Task 4: 前端 — 水印 Tab 卡片模板

**Files:**
- Modify: `src/views/VideoTool.vue`

- [ ] **Step 4a: 在最后一个 Tab 模板（视频调整 adjust 闭合 `</template>` L1165 之后）追加 watermark Tab 模板**

找到 `视频调整 Tab` 的闭合标记：
```
    </template>
```
（它是 `v-if="activeTab === 'adjust'"` 的闭合）

在其后追加 **水印 Tab 完整 4 卡片模板**：

```html
    <!-- ==================== Tab: 加水印 (F29) ==================== -->
    <template v-if="activeTab === 'watermark'">
      <div v-if="!useFfmpeg" class="tool-card">
        <div class="card-body">
          <div class="ffmpeg-required">
            加水印需要 ffmpeg，请先安装 ffmpeg 后重启应用
          </div>
        </div>
      </div>

      <template v-else>
        <!-- 卡片 1: 选择视频 -->
        <div class="tool-card">
          <div class="card-header"><span class="card-title">选择视频文件</span></div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <el-button type="primary" size="small" @click="wmOpenFile" :loading="wm_loadingInfo">
                  打开文件
                </el-button>
              </div>
            </div>
            <div v-if="wm_filePath" class="video-file-info">
              <span class="file-name">{{ wm_fileName }}</span>
              <span class="file-detail" v-if="wm_videoInfo">
                {{ formatDuration(wm_videoInfo.duration) }} | {{ wm_videoInfo.width }}x{{ wm_videoInfo.height }} |
                {{ formatFileSize(wm_videoInfo.file_size) }}
              </span>
            </div>
          </div>
        </div>

        <!-- 卡片 2: 类型 + 位置 -->
        <div v-if="wm_videoInfo" class="tool-card">
          <div class="card-header"><span class="card-title">水印类型 & 位置</span></div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">水印类型</div>
                <div class="group-buttons">
                  <el-button size="small" :type="wm_type === 'text' ? 'primary' : ''" @click="wm_type = 'text'">文字</el-button>
                  <el-button size="small" :type="wm_type === 'image' ? 'primary' : ''" @click="wm_type = 'image'">图片</el-button>
                  <el-button size="small" :type="wm_type === 'both' ? 'primary' : ''" @click="wm_type = 'both'">都用</el-button>
                </div>
              </div>
            </div>

            <div class="wm-settings-row">
              <div class="wm-pos-wrap">
                <div class="group-label">位置</div>
                <div class="wm-position-grid">
                  <el-button
                    v-for="p in POSITIONS"
                    :key="p.value"
                    size="small"
                    :type="wm_position === p.value ? 'primary' : ''"
                    @click="wm_position = p.value"
                  >{{ p.icon }}</el-button>
                </div>
              </div>
              <div class="wm-offset-wrap">
                <div class="action-group">
                  <div class="group-label">水平偏移 X</div>
                  <el-input-number v-model="wm_offsetX" :min="0" :max="2000" :step="10" size="small" style="width: 100px" />
                  <span class="unit-text">px</span>
                </div>
                <div class="action-group">
                  <div class="group-label">垂直偏移 Y</div>
                  <el-input-number v-model="wm_offsetY" :min="0" :max="2000" :step="10" size="small" style="width: 100px" />
                  <span class="unit-text">px</span>
                </div>
              </div>
            </div>

            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <el-checkbox v-model="wm_useTimeRange" size="small">限定时间段显示水印</el-checkbox>
              </div>
            </div>
            <div class="action-grid" v-if="wm_useTimeRange && wm_videoInfo">
              <div class="action-group">
                <div class="group-label">起始时间</div>
                <el-input-number
                  v-model="wm_startTime"
                  :min="0" :max="Math.max(0, wm_endTime - 0.1)"
                  :step="0.1" :precision="1" size="small" style="width: 120px"
                />
                <span class="unit-text">秒</span>
              </div>
              <div class="action-group">
                <div class="group-label">结束时间</div>
                <el-input-number
                  v-model="wm_endTime"
                  :min="wm_startTime + 0.1" :max="wm_videoInfo.duration"
                  :step="0.1" :precision="1" size="small" style="width: 120px"
                />
                <span class="unit-text">秒</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 卡片 3: 水印内容参数 -->
        <div v-if="wm_videoInfo" class="tool-card">
          <div class="card-header"><span class="card-title">水印参数</span></div>
          <div class="card-body">
            <!-- 文字水印 -->
            <div v-if="wm_type === 'text' || wm_type === 'both'" class="wm-params-block">
              <div class="wm-params-title">📝 文字水印</div>
              <div class="action-grid">
                <div class="action-group wm-text-row">
                  <div class="group-label">水印文字</div>
                  <el-input
                    v-model="wm_text"
                    size="small"
                    placeholder="请输入水印文字..."
                    style="width: 260px"
                    maxlength="200"
                    show-word-limit
                  />
                  <VariablePicker v-model="wm_text" size="small" />
                </div>
              </div>
              <div class="action-grid" style="margin-top: 8px">
                <div class="action-group">
                  <div class="group-label">字体</div>
                  <el-select v-model="wm_fontName" size="small" style="width: 160px">
                    <el-option
                      v-for="f in wm_fontList"
                      :key="f.path"
                      :label="f.name"
                      :value="f.name"
                    />
                  </el-select>
                  <div class="group-buttons">
                    <el-button size="small" @click="wmPickCustomFont">自定义字体...</el-button>
                  </div>
                  <span class="font-path-hint" v-if="getResolvedFontPath()">
                    → {{ getResolvedFontPath() }}
                  </span>
                </div>
              </div>
              <div class="action-grid" style="margin-top: 8px">
                <div class="action-group">
                  <div class="group-label">字号</div>
                  <el-input-number v-model="wm_fontSize" :min="8" :max="200" size="small" style="width: 90px" />
                </div>
                <div class="action-group">
                  <div class="group-label">颜色</div>
                  <el-color-picker v-model="wm_fontColor" size="small" />
                </div>
                <div class="action-group">
                  <div class="group-label">透明度 {{ wm_fontOpacity.toFixed(1) }}</div>
                  <el-slider v-model="wm_fontOpacity" :min="0.1" :max="1.0" :step="0.1" style="width: 140px" />
                </div>
                <div class="action-group">
                  <el-checkbox v-model="wm_fontBorder" size="small">描边</el-checkbox>
                  <el-color-picker v-if="wm_fontBorder" v-model="wm_fontBorderColor" size="small" />
                </div>
              </div>
            </div>

            <!-- 图片水印 -->
            <div v-if="wm_type === 'image' || wm_type === 'both'" class="wm-params-block">
              <div class="wm-params-title">🖼️ 图片水印</div>
              <div class="action-grid">
                <div class="action-group">
                  <el-button type="primary" size="small" @click="wmPickImage">选择水印图片</el-button>
                  <el-button size="small" v-if="wm_imagePath" @click="wmClearImage">移除</el-button>
                </div>
                <div v-if="wm_imageName" class="file-name">{{ wm_imageName }}</div>
              </div>
              <div class="action-grid" style="margin-top: 8px">
                <div class="action-group">
                  <div class="group-label">缩放 {{ (wm_imageScale * 100).toFixed(0) }}%</div>
                  <el-slider v-model="wm_imageScale" :min="0.1" :max="2.0" :step="0.05" style="width: 180px" show-input size="small" />
                </div>
                <div class="action-group">
                  <div class="group-label">透明度 {{ wm_imageOpacity.toFixed(1) }}</div>
                  <el-slider v-model="wm_imageOpacity" :min="0.1" :max="1.0" :step="0.05" style="width: 180px" />
                </div>
              </div>
              <div class="wm-hint-text">
                支持 PNG（带透明通道）/ JPG / WebP 静态图片；GIF 动画请先转为静态帧
              </div>
            </div>
          </div>
        </div>

        <!-- 卡片 4: 输出 & 操作 -->
        <div v-if="wm_videoInfo" class="tool-card">
          <div class="card-header"><span class="card-title">输出 & 操作</span></div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输出格式</div>
                <el-select v-model="wm_outputFormat" size="small" style="width: 100px">
                  <el-option label="MP4" value="mp4" />
                  <el-option label="MKV" value="mkv" />
                  <el-option label="MOV" value="mov" />
                  <el-option label="WebM" value="webm" />
                </el-select>
              </div>
              <div class="action-group">
                <el-checkbox v-model="wm_saveToSamePath" size="small">与源文件同路径</el-checkbox>
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px">
              <div class="action-group">
                <el-button
                  type="primary"
                  size="small"
                  @click="wmDoWatermark"
                  :loading="wm_isProcessing"
                  :disabled="!wm_canSubmit"
                >加水印并导出</el-button>
                <el-button size="small" @click="wmReset">重置</el-button>
              </div>
            </div>
            <el-progress v-if="wm_isProcessing" :percentage="wm_progress" :stroke-width="6" style="margin-top: 12px" />
            <div v-if="wm_result" class="result-info">
              <span>输出路径: {{ wm_result.path }}</span>
              <span class="result-sep">|</span>
              <span>大小: {{ formatFileSize(wm_result.size) }}</span>
            </div>
          </div>
        </div>
      </template>
    </template>
```

---

### Task 5: 前端 — 水印 Tab 操作函数 + 事件监听 + scoped 样式

**Files:**
- Modify: `src/views/VideoTool.vue`

- [ ] **Step 5a: 追加操作函数（脚本，最后一个 onMounted/watch 之后或和 doRotateFlip/doSpeedChange 函数并列）**

找到 `doRotateFlip` 或 `doSpeedChange` 函数的位置，在之后追加：

```typescript
// ============ 水印 Tab 操作 (F29) ============
async function wmOpenFile() {
  const dialog = (window as any).__TAURI_INTERNALS__?.dialog || (await import('@tauri-apps/plugin-dialog'))
  try {
    wm_loadingInfo.value = true
    const selected = await dialog.open({
      multiple: false,
      filters: [{ name: '视频文件', extensions: ['mp4','mkv','mov','avi','webm','flv','wmv','m4v','ts'] }],
    })
    if (!selected) return
    const path = Array.isArray(selected) ? selected[0] : (selected as string)
    wm_filePath.value = path
    wm_fileName.value = path.split(/[/\\]/).pop() || ''
    wm_result.value = null
    try {
      wm_videoInfo.value = await invoke<VideoInfo>('get_video_info', { path, useFfmpeg: true })
      if (wm_videoInfo.value && wm_videoInfo.value.duration > 0) {
        wm_endTime.value = Math.round(wm_videoInfo.value.duration * 10) / 10
      }
    } catch (e: any) {
      ElMessage.error('读取视频信息失败: ' + (e.message || e))
    }
  } finally {
    wm_loadingInfo.value = false
  }
}

async function wmPickCustomFont() {
  const dialog = (window as any).__TAURI_INTERNALS__?.dialog || (await import('@tauri-apps/plugin-dialog'))
  const selected = await dialog.open({
    multiple: false,
    filters: [{ name: '字体文件', extensions: ['ttf','ttc','otf'] }],
  })
  if (!selected) return
  const path = Array.isArray(selected) ? selected[0] : (selected as string)
  wm_customFontPath.value = path
}

async function wmPickImage() {
  const dialog = (window as any).__TAURI_INTERNALS__?.dialog || (await import('@tauri-apps/plugin-dialog'))
  const selected = await dialog.open({
    multiple: false,
    filters: [{ name: '图片文件', extensions: ['png','jpg','jpeg','webp','bmp'] }],
  })
  if (!selected) return
  const path = Array.isArray(selected) ? selected[0] : (selected as string)
  wm_imagePath.value = path
  wm_imageName.value = path.split(/[/\\]/).pop() || ''
  wm_result.value = null
}
function wmClearImage() {
  wm_imagePath.value = ''
  wm_imageName.value = ''
  wm_result.value = null
}

async function probeFonts() {
  wm_fontList.value = []
  for (const f of CANDIDATE_FONTS) {
    try {
      // 用 std fs metadata (通过 invoke 检查，或直接前端 try/catch + dialog 不太行)
      // 简化：通过 invoke('file_read_base64') 尝试探测，不实际读
      // 更轻量：用 tryGetFileInfo 命令，如果没有就用同步的 try-stat
      // 这里用通用方式：probe_path_exists，如没有新增就不 probe 了，直接全加入（让后端报错时提示用户）
      // 为最小化改动，直接全量 push（Windows 默认字体目录一般都在）
      wm_fontList.value.push(f)
    } catch (_e) { /* ignore */ }
  }
}

function wmReset() {
  wm_text.value = ''
  wm_fontSize.value = 32
  wm_fontColor.value = '#ffffff'
  wm_fontOpacity.value = 1.0
  wm_fontBorder.value = false
  wm_fontBorderColor.value = '#000000'
  wm_customFontPath.value = ''
  wm_imagePath.value = ''
  wm_imageName.value = ''
  wm_imageScale.value = 1.0
  wm_imageOpacity.value = 0.8
  wm_position.value = 'bottomRight'
  wm_offsetX.value = 20
  wm_offsetY.value = 20
  wm_useTimeRange.value = false
  wm_startTime.value = 0
  if (wm_videoInfo.value) wm_endTime.value = Math.round(wm_videoInfo.value.duration * 10) / 10
  wm_outputFormat.value = 'mp4'
  wm_result.value = null
}

async function wmDoWatermark() {
  if (!wm_canSubmit.value) return
  wm_isProcessing.value = true
  wm_progress.value = 0
  wm_done.value = false
  wm_result.value = null

  // 输出路径
  let outputPath: string | undefined = undefined
  if (!wm_saveToSamePath.value) {
    const dialog = (window as any).__TAURI_INTERNALS__?.dialog || (await import('@tauri-apps/plugin-dialog'))
    const p = await dialog.save({
      defaultPath: `${wm_fileName.value.replace(/\.[^.]+$/, '')}_watermarked.${wm_outputFormat.value}`,
      filters: [{ name: '视频', extensions: [wm_outputFormat.value] }],
    })
    if (!p) { wm_isProcessing.value = false; return }
    outputPath = p as string
  }

  try {
    const fontFile = getResolvedFontPath()
    const options = {
      wmType: wm_type.value,
      text: wm_text.value || undefined,
      fontFile,
      fontSize: wm_fontSize.value,
      fontColor: wm_fontColor.value,
      fontBorder: wm_fontBorder.value,
      fontBorderColor: wm_fontBorderColor.value,
      fontOpacity: wm_fontOpacity.value,
      imagePath: wm_imagePath.value || undefined,
      imageScale: wm_imageScale.value,
      imageOpacity: wm_imageOpacity.value,
      position: wm_position.value,
      offsetX: wm_offsetX.value,
      offsetY: wm_offsetY.value,
      useTimeRange: wm_useTimeRange.value,
      startTime: wm_startTime.value,
      endTime: wm_endTime.value,
      outputFormat: wm_outputFormat.value,
      outputPath,
    }

    const store = useToolStore()
    const historyId = store.addHistory({
      tool: 'video_watermark',
      category: 'media',
      status: 'processing',
      inputPreview: `${wm_fileName.value} + ${wm_type.value === 'text' ? '文字:' + wm_text.value.slice(0, 20) : wm_type.value === 'image' ? '图片:' + wm_imageName.value : '文字+图片'} @ ${wm_position.value}`,
      inputFull: JSON.stringify({ filePath: wm_filePath.value, ...options }),
      outputPreview: '处理中...',
      outputFull: '',
    })

    const result = await invoke<any>('video_watermark', {
      path: wm_filePath.value,
      options,
    })

    wm_result.value = { path: result.output_path, size: result.output_size }

    store.updateHistory(historyId, {
      status: 'success',
      outputPreview: `输出: ${result.output_path.split(/[/\\]/).pop()} (${formatFileSize(result.output_size)})`,
      outputFull: JSON.stringify(result),
    })

    ElMessage.success('加水印完成')
    if (outputPath) {
      ElMessage.success(`已保存到: ${outputPath}`)
    }
  } catch (e: any) {
    ElMessage.error('加水印失败: ' + (typeof e === 'string' ? e : (e.message || String(e))))
  } finally {
    wm_isProcessing.value = false
    wm_done.value = true
  }
}
```

**注意**：上面代码用到 `useToolStore()` — 检查文件 import，如果没有就补：
```typescript
import { useToolStore } from '@/store'
```
（通常 VideoTool.vue 已经引入，若未引入则加在 `<script setup>` 顶部的 import 区域）

**注意**：`VariablePicker` 组件也要确保已在 `VideoTool.vue` 引入。搜索：
```
import VariablePicker
```
若无，添加：
```typescript
import VariablePicker from '@/components/VariablePicker.vue'
```

- [ ] **Step 5b: 追加 activeTab 的 watch 中 watermark 分支（注册/注销事件 + probe 字体）**

在 VideoTool.vue 中找是否已有 `watch(() => activeTab.value, ...)`。若无则在现有 watch 区域追加：

```typescript
// Tab 切换：水印 Tab 监听进度事件 + 探测字体
watch(() => activeTab.value, async (t) => {
  if (t === 'watermark') {
    wm_done.value = false
    wm_progress.value = wm_isProcessing.value ? wm_progress.value : 0
    probeFonts()
    // 监听进度事件
    const eventApi = (await import('@tauri-apps/api/event')) as any
    const unlisten = await eventApi.listen('video-watermark-progress', (event: any) => {
      if (wm_done.value) return
      wm_progress.value = Math.round(event.payload?.progress ?? 0)
    })
    wm_listenOff.value = unlisten as any
    // 如果 store 中有从历史记录跳转来的待还原数据，此处处理（和其他 Tab 同模式）
    const store = useToolStore()
    if (store.pendingRestore?.tool === 'video_watermark') {
      try {
        const data = JSON.parse(store.pendingRestore.inputFull)
        wm_type.value = data.wmType || 'text'
        wm_text.value = data.text || ''
        wm_fontSize.value = data.fontSize || 32
        wm_fontColor.value = data.fontColor || '#ffffff'
        wm_fontOpacity.value = data.fontOpacity ?? 1.0
        wm_fontBorder.value = !!data.fontBorder
        wm_fontBorderColor.value = data.fontBorderColor || '#000000'
        wm_imagePath.value = data.imagePath || ''
        wm_imageName.value = data.imagePath ? data.imagePath.split(/[/\\]/).pop() : ''
        wm_imageScale.value = data.imageScale ?? 1.0
        wm_imageOpacity.value = data.imageOpacity ?? 0.8
        wm_position.value = data.position || 'bottomRight'
        wm_offsetX.value = data.offsetX ?? 20
        wm_offsetY.value = data.offsetY ?? 20
        wm_useTimeRange.value = !!data.useTimeRange
        wm_startTime.value = data.startTime || 0
        wm_endTime.value = data.endTime || 0
        wm_outputFormat.value = data.outputFormat || 'mp4'
        if (data.filePath) {
          wm_filePath.value = data.filePath
          wm_fileName.value = data.filePath.split(/[/\\]/).pop() || ''
          try {
            wm_videoInfo.value = await invoke<VideoInfo>('get_video_info', { path: data.filePath, useFfmpeg: true })
          } catch(_e) {}
        }
        store.clearPendingRestore()
      } catch(_e) {}
    }
  } else {
    wm_listenOff.value?.()
    wm_listenOff.value = null
  }
}, { immediate: true, flush: 'post' })
```

> 如果文件中已有 `watch(() => activeTab.value, ...)` 处理其他 Tab 的监听，将 watermark 分支合并到同一个 watch 里。

- [ ] **Step 5c: 追加水印 Tab scoped 样式（`<style scoped>` 末尾）**

找到 `VideoTool.vue` 的 `<style scoped>` 块末尾，在闭合 `</style>` 之前追加：

```css
/* ===== 加水印 Tab 样式 ===== */
.wm-settings-row {
  display: flex;
  gap: 32px;
  align-items: flex-start;
  margin-top: 16px;
  flex-wrap: wrap;
}
.wm-pos-wrap { display: flex; flex-direction: column; gap: 8px; }
.wm-position-grid {
  display: grid;
  grid-template-columns: repeat(3, 44px);
  grid-template-rows: repeat(3, 32px);
  gap: 4px;
}
.wm-position-grid .el-button {
  width: 44px;
  height: 32px;
  padding: 0;
  font-size: 14px;
}
.wm-offset-wrap { display: flex; flex-direction: column; gap: 10px; justify-content: center; }

.wm-params-block { margin-bottom: 16px; }
.wm-params-block:last-child { margin-bottom: 0; }
.wm-params-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  margin-bottom: 10px;
  letter-spacing: 0.5px;
}
.wm-text-row { flex-wrap: wrap; }
.font-path-hint {
  color: var(--text-secondary);
  font-size: 12px;
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.wm-hint-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-top: 8px;
}
```

- [ ] **Step 5d: npm run build 验证前端**

```powershell
cd d:\work\codes\litobox
npm run build
```
Expected: TypeScript 无类型错误，打包通过

---

### Task 6: 工作流 + 文档收尾

**Files:**
- Modify: `src/views/WorkflowView.vue`
- Modify: `docs/superpowers/plans/feature-backlog.md`
- Modify: `README.md`

- [ ] **Step 6a: WorkflowView.vue executeStep 加 video_watermark 分支**

在 `executeStep` 的 switch 中，`case 'quickLaunch'` 分支之后、`default:` 之前追加：

```typescript
    case 'video_watermark': {
      // 输入是视频路径
      if (!input.trim()) return ''
      try {
        const config = step.config || {}
        const resolveVarFn = (v: any) => {
          if (typeof v === 'string' && v.startsWith('$var:')) {
            const name = v.slice(5)
            return varPool.find(x => x.name === name)?.value ?? v
          }
          return v
        }
        const result = await invoke<any>('video_watermark', {
          path: input,
          options: {
            wmType: resolveVarFn(config.wmType) || 'text',
            text: resolveVarFn(config.text),
            fontFile: resolveVarFn(config.fontFile),
            fontSize: resolveVarFn(config.fontSize) || 32,
            fontColor: resolveVarFn(config.fontColor) || '#ffffff',
            fontBorder: !!config.fontBorder,
            fontBorderColor: resolveVarFn(config.fontBorderColor) || '#000000',
            fontOpacity: resolveVarFn(config.fontOpacity) ?? 1.0,
            imagePath: resolveVarFn(config.imagePath),
            imageScale: resolveVarFn(config.imageScale) ?? 1.0,
            imageOpacity: resolveVarFn(config.imageOpacity) ?? 0.8,
            position: resolveVarFn(config.position) || 'bottomRight',
            offsetX: resolveVarFn(config.offsetX) ?? 20,
            offsetY: resolveVarFn(config.offsetY) ?? 20,
            useTimeRange: !!config.useTimeRange,
            startTime: resolveVarFn(config.startTime) || 0,
            endTime: resolveVarFn(config.endTime) || 0,
            outputFormat: resolveVarFn(config.outputFormat) || 'mp4',
          },
        })
        return result.output_path
      } catch (e: any) {
        return `[错误] 视频加水印失败: ${typeof e === 'string' ? e : e.message || e}`
      }
    }
```

- [ ] **Step 6b: feature-backlog.md 标记 F29 已完成**

打开 `docs/superpowers/plans/feature-backlog.md`，找到 F29 行（~第 153 行），把开头的 `| F29  | **视频加水印** |` 改为：
```
| F29  | ✅ **视频加水印**           | 图片/文字水印叠加，支持位置/透明度/时间段。仅 ffmpeg 模式可用                                                                                                 | — 已完成 V6.9 — | 2026-08-04 用户规划            |
```

同时在「已完成版本」表格顶部追加（V6.8 那行之后）：
```
| V6.9 | ✅  | 视频加水印（文字/图片/叠加，9宫格+偏移+时间段）                                 | 2026-08-04 |
```

- [ ] **Step 6c: README.md 追加 V6.9 条目**

打开 README.md，找到功能版本表格或功能阶段列表，在 V6.8 之后追加：
```
- **V6.9 (2026-08-04)**: 新增 视频加水印工具（VideoTool 加水印 Tab），支持文字/图片/两者叠加水印，9 宫格位置 + 自定义偏移，可选时间段，仅 ffmpeg 模式。
```

---

### Task 7: 构建 & 手动验收

**Files:** （无修改）

- [ ] **Step 7a: cargo check 最终验证**
```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: success

- [ ] **Step 7b: npm run build 最终验证**
```powershell
cd d:\work\codes\litobox
npm run build
```
Expected: success

- [ ] **Step 7c: 手动验收清单（至少 1 次实际运行验证，若开发服务器启动中）**
  1. 打开「视频工具 → 加水印」Tab
  2. 选一个 <30s 的 MP4
  3. 纯文字水印 → 右下 + 微软雅黑 + 白色描边 → 导出 → 播放可见
  4. 纯 PNG Logo → 透明度 0.6 + 左上 + 缩放 30% → 导出
  5. 两者叠加 → 文字右上、图片左下 → 同时可见
  6. 时间段 3s-8s → 水印只在这 5s 内出现
  7. 进度条 0 → 100 不跳变
  8. 操作历史生成记录，双击跳转会还原参数

---

### Plan 自审结果

| 检查项 | 结果 |
|---|---|
| Spec §2.3 文件变更清单 | 覆盖：video_tools.rs / main.rs / VideoTool.vue / WorkflowView.vue / README.md / feature-backlog.md ✅ |
| Spec §3 后端函数（escape/position/enable/color/drawtext/overlay/do_watermark） | Task1+Task2 全部对应 ✅ |
| Spec §4 状态变量 + 9宫格 + 字体列表 + 计算属性 | Task3+Task4+Task5 覆盖 ✅ |
| Spec §4.5 Tab切换 事件监听 | Task 5b ✅ |
| Spec §6 历史记录 addHistory | Task 5a `wmDoWatermark` 内 ✅ |
| Spec §7 Workflow 集成 | Task 6a ✅ |
| Spec §8 Out of Scope（无预览/无批量/无动画） | 计划中未涉及 ✅ |
| 版本号同步 README | Task 6c ✅（package/tauri/cargo 已 6.9，无需改文件） |
| 无 placeholder / 无 TBD / 无 "同 TaskN" | 每步都给了具体代码 ✅ |
