# 视频加水印设计规格

**版本**: 1.0
**日期**: 2026-08-04
**状态**: 待实现
**Backlog 编号**: F29
**目标版本**: V6.9.0

---

## 1. 概述

### 1.1 目标
在 `VideoTool.vue` 中新增独立 Tab「加水印」，支持文字水印、图片水印（Logo）、或两者叠加。基于 ffmpeg `drawtext` + `overlay` 滤镜一次编码完成，支持 9 宫格位置 + 自定义偏移、可选限定时间段、实时进度上报。

### 1.2 核心价值
- **版权保护**：快速给视频打上品牌 Logo / 署名
- **水印叠加**：文字 + 图片可组合使用，一次导出
- **零额外依赖**：纯 ffmpeg 滤镜（项目已接入 ffmpeg 可选增强框架），不改 Cargo.toml

### 1.3 技术约束
- **仅 ffmpeg 模式可用**：顶部 banner 已统一检测，未检测到 ffmpeg 时操作按钮禁用
- **一次编码**：文字 + 图片水印通过单条 `filter_complex` 链路组合，不重复编码
- **进度实时性**：复用 `run_ffmpeg_with_progress` 解析 `out_time_us`，避免进度条 0→100 跳变
- **Windows 路径转义**：drawtext/overlay 参数中的文件路径、文字内容必须严格转义（见 3.3 节）
- 严格遵守 AGENTS.md 规则：按钮 loading 不自锁 UI、事件+轮询兜底、历史记录传 `inputFull`/`outputFull`、CSV 导出规范

---

## 2. 架构设计

### 2.1 位置
- **页面文件**：`src/views/VideoTool.vue`（仅修改，新增顶级 Tab）
- **后端模块**：`src-tauri/src/video_tools.rs`（仅追加函数和结构体）
- **分类**：侧边栏 `media` 分类下的「视频工具」内

### 2.2 数据流
```
切换到 watermark Tab (onActivated / watch)
  ↓
检查 ffmpeg 状态 (useFfmpeg)
  └─ 未安装 → 顶部 banner + 按钮 disabled
  ↓
点击「打开文件」 → invoke('get_video_info') → 填充文件信息
  ↓
选择水印类型 text/image/both → 填写参数 (9宫格/偏移/文字/字体/图片路径/缩放/时间段)
  ↓
前端校验 (必填项 + 时间范围合法)
  ↓
invoke('video_watermark', { path, options })
  ↓
后端 spawn_blocking:
  1. 验证输入 (字体文件存在 / 图片可读 / 视频信息)
  2. 根据 position 计算 x:y 表达式
  3. 构造 filter_complex 字符串 (drawtext + overlay 组合)
  4. 调用 run_ffmpeg_with_progress(args, "video-watermark-progress", duration)
  5. 进度事件 → emit
  ↓
前端 listen('video-watermark-progress') + 兜底轮询 (check video_watermark_status? 不需要，进度事件够用)
  ↓
完成: 显示输出路径 + 文件大小 + 记录历史 (addHistory)
```

### 2.3 文件变更清单
1. **修改** `src-tauri/src/video_tools.rs` — 追加结构体 + `video_watermark` 命令 + 5 个辅助函数
2. **修改** `src-tauri/src/main.rs` — 注册 `video_watermark` 命令
3. **修改** `src/views/VideoTool.vue` — 新增顶级 Tab（~600 行）：
   - 模板：4 张卡片（文件选择 / 类型+位置 / 水印参数 / 输出操作）
   - 脚本：状态 + 事件监听 + invoke 调用
   - 样式：9宫格按钮组 + 水印参数面板（scoped）
4. **修改** `src/views/WorkflowView.vue` — `executeStep` 追加 `video_watermark` 分支
5. **修改** `src/store/index.ts` — `TOOL_LIST` 中 video 条目不改（复用现有菜单项）
6. **修改** `package.json` — 版本号 `6.8` → `6.9`
7. **修改** `src-tauri/tauri.conf.json` — 版本号同步
8. **修改** `README.md` — V6.9 功能条目追加「视频加水印」

---

## 3. 后端设计

### 3.1 数据结构

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoWatermarkOptions {
    pub wm_type: String,           // "text" | "image" | "both"

    // 文字水印
    pub text: Option<String>,
    pub font_file: Option<String>, // 前端解析后传入的绝对路径
    pub font_size: Option<u32>,    // 默认 32
    pub font_color: Option<String>, // 默认 #ffffff，支持 #RRGGBB / #AARRGGBB
    pub font_border: Option<bool>, // 默认 false
    pub font_border_color: Option<String>,
    pub font_opacity: Option<f32>, // 0.1~1.0，默认 1.0

    // 图片水印
    pub image_path: Option<String>,
    pub image_scale: Option<f32>,  // 0.1~2.0，默认 1.0
    pub image_opacity: Option<f32>, // 0.1~1.0，默认 0.8

    // 公共参数
    pub position: String,          // topLeft/top/topRight/left/center/right/bottomLeft/bottom/bottomRight
    pub offset_x: u32,             // 默认 20
    pub offset_y: u32,             // 默认 20
    pub use_time_range: bool,      // 默认 false
    pub start_time: f64,
    pub end_time: f64,

    // 输出
    pub output_format: String,     // mp4/mkv/mov/webm
    pub output_path: Option<String>, // None 时自动生成
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoWatermarkResult {
    pub output_path: String,
    pub output_size: u64,
}
```

### 3.2 Tauri 命令

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

### 3.3 核心实现（do_video_watermark 伪代码）

```rust
fn do_video_watermark(app, path, opts) -> Result {
    debug_log!("video_watermark start: type={}, position={}", opts.wm_type, opts.position);

    // 1. 基础校验
    let info = get_video_info_ffprobe(path)?;
    let duration = info.duration;
    let (W, H) = (info.width, info.height);
    validate_options(opts, &info)?; // 文字非空 / 图片存在 / 时间段合理

    // 2. 生成输出路径
    let output_path = resolve_output_path(path, opts)?;

    // 3. 计算水印位置表达式 (x_expr, y_expr)
    // 注意：drawtext 的 x/y 接受表达式；overlay 的 x/y 也接受表达式
    // 两者形式完全一致，可复用同一对表达式
    let (x_expr, y_expr) = calc_position_expr(&opts.position, opts.offset_x, opts.offset_y);
    // 例: bottomRight → x_expr = "W-w-20", y_expr = "H-h-20"

    // 4. 构造滤镜链
    let mut filter_parts: Vec<String> = Vec::new();
    let mut last_label = String::from("0:v");

    // 4a. drawtext 部分
    if opts.wm_type == "text" || opts.wm_type == "both" {
        let font_file = escape_ffmpeg_path(opts.font_file.as_deref().unwrap());
        let text = escape_drawtext_text(opts.text.as_deref().unwrap());
        let enable_expr = build_enable_expr(opts.use_time_range, opts.start_time, opts.end_time);
        let alpha = opts.font_opacity.unwrap_or(1.0);
        let fs = opts.font_size.unwrap_or(32);
        let fc = format_color_with_alpha(&opts.font_color.clone().unwrap_or("#ffffff".into()), alpha);

        let mut dt = format!(
            "drawtext=fontfile='{}':text='{}':fontsize={}:fontcolor={}:x={}:y={}",
            font_file, text, fs, fc, x_expr, y_expr
        );
        if opts.font_border.unwrap_or(false) {
            let bc = format_color_with_alpha(&opts.font_border_color.clone().unwrap_or("#000000".into()), alpha);
            dt.push_str(&format!(":borderw=2:bordercolor={}", bc));
        }
        if !enable_expr.is_empty() {
            dt.push_str(&format!(":enable='{}'", enable_expr));
        }
        filter_parts.push(format!("[{}]{}[v1]", last_label, dt));
        last_label = String::from("v1");
    }

    // 4b. overlay 部分 (图片水印)
    if opts.wm_type == "image" || opts.wm_type == "both" {
        let img_path = escape_ffmpeg_path(opts.image_path.as_deref().unwrap());
        let scale = opts.image_scale.unwrap_or(1.0);
        let alpha = opts.image_opacity.unwrap_or(0.8);
        let enable_expr = build_enable_expr(opts.use_time_range, opts.start_time, opts.end_time);

        // 图片输入: 加载 → 缩放 → 转 rgba → 设置透明度
        let img_label = if opts.wm_type == "both" { "wmimg" } else { "1:v" };
        filter_parts.push(format!(
            "movie='{}',loop=0,setpts=PTS-STARTPTS,scale=iw*{}:-1,format=rgba,colorchannelmixer=aa={}[{}]",
            img_path, scale, alpha, img_label
        ));

        let mut ov = format!("[{}][{}]overlay=x={}:y={}", last_label, img_label, x_expr, y_expr);
        if !enable_expr.is_empty() {
            ov.push_str(&format!(":enable='{}'", enable_expr));
        }
        let next_label = if opts.wm_type == "both" { "v2" } else { "v1" };
        filter_parts.push(format!("{}[{}]", ov, next_label));
        last_label = next_label;
    }

    let filter_complex = filter_parts.join(";");
    debug_log!("filter_complex: {}", filter_complex);

    // 5. 组装 ffmpeg args
    let mut args = vec![
        "-y", "-i", path,
        "-filter_complex", &filter_complex,
        "-map", &format!("[{}]", last_label),
        "-map", "0:a?",
        // 根据 output_format 选编码器 (复用 video_speed_change 的 match 逻辑)
        ...encoders_for_format(opts.output_format),
        &output_path
    ];

    app.emit("video-watermark-progress", {progress: 5.0});
    run_ffmpeg_with_progress(app, &args, "video-watermark-progress", duration)?;
    app.emit("video-watermark-progress", {progress: 100.0});

    let size = std::fs::metadata(&output_path)?.len();
    Ok(VideoWatermarkResult { output_path, output_size: size })
}
```

### 3.4 关键转义函数

```rust
/// drawtext / movie 的路径转义:
/// - Windows 反斜杠 \ → 正斜杠 / （ffmpeg 接受）
/// - 单引号 ' → '\'' （在单引号字符串内）
/// - 注意：drawtext 内部分隔符是 :，所以路径的 : 也要特殊处理
///   Windows 盘符 C: 的 : → 用 \: 转义
fn escape_ffmpeg_path(p: &str) -> String {
    let p = p.replace('\\', "/");
    let p = p.replace('\'', "'\\''");
    // drawtext: C:\Windows → C\:/Windows （只转第一个冒号，盘符）
    // 简化做法: 全量 : → \:
    p.replace(':', "\\:")
}

/// drawtext 文字转义:
/// - : → \: （参数分隔符）
/// - ' → '\'' （在 text='...' 内）
/// - % → %% （timecode 变量前缀）
/// - \ → \\
fn escape_drawtext_text(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('\'', "'\\''")
     .replace(':', "\\:")
     .replace('%', "%%")
}

/// enable 表达式 (between)
fn build_enable_expr(use_range: bool, s: f64, e: f64) -> String {
    if use_range && e > s {
        format!("between(t\\,{}\\,{})", s, e)
    } else {
        String::new()
    }
}
```

> **注意**：`enable` 表达式写在 filter 参数里，逗号本身也需要用 `\,` 转义（因为 enable 本身是 `key=value`，value 里的逗号会被解析成 filter 结束符）。

### 3.5 位置表达式（calc_position_expr）

```rust
fn calc_position_expr(pos: &str, ox: u32, oy: u32) -> (String, String) {
    // drawtext 变量: w = text宽度, h = text行高
    // overlay 变量: w = 叠加图宽, h = 叠加图高
    // 两者形式一致，可复用
    let ox = ox.to_string();
    let oy = oy.to_string();
    match pos {
        "topLeft"     => (format!("{}", ox),            format!("{}", oy)),
        "top"         => (format!("(W-w)/2"),           format!("{}", oy)),
        "topRight"    => (format!("W-w-{}", ox),        format!("{}", oy)),
        "left"        => (format!("{}", ox),            format!("(H-h)/2")),
        "center"      => (format!("(W-w)/2"),           format!("(H-h)/2")),
        "right"       => (format!("W-w-{}", ox),        format!("(H-h)/2")),
        "bottomLeft"  => (format!("{}", ox),            format!("H-h-{}", oy)),
        "bottom"      => (format!("(W-w)/2"),           format!("H-h-{}", oy)),
        "bottomRight" => (format!("W-w-{}", ox),        format!("H-h-{}", oy)),
        _             => (format!("W-w-{}", ox),        format!("H-h-{}", oy)), // 默认右下
    }
}
```

---

## 4. 前端设计

### 4.1 状态变量（VideoTool.vue 追加）

```typescript
// ============ 水印 Tab 状态 (F29) ============
const wm_filePath = ref('')
const wm_fileName = ref('')
const wm_videoInfo = ref<VideoInfo | null>(null)
const wm_loadingInfo = ref(false)
const wm_isProcessing = ref(false)
const wm_progress = ref(0)
const wm_done = ref(false)  // 事件+轮询兜底去重 flag
const wm_listenOff = ref<(() => void) | null>(null)

const wm_type = ref<'text' | 'image' | 'both'>('text')

// 文字
const wm_text = ref('')
const wm_fontList = ref<{ name: string; path: string }[]>([])  // 预置字体（probe 后填充）
const wm_fontName = ref('Microsoft YaHei')
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
const wm_position = ref('bottomRight')
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

### 4.2 字体预置 + 探测

进入 Tab 时（watch activeTab === 'watermark'）调用 probe：
```typescript
// 内置候选字体
const CANDIDATE_FONTS = [
  { name: '微软雅黑',        path: 'C:/Windows/Fonts/msyh.ttc' },
  { name: '微软雅黑 Bold',   path: 'C:/Windows/Fonts/msyhbd.ttc' },
  { name: '黑体 (SimHei)',   path: 'C:/Windows/Fonts/simhei.ttf' },
  { name: '宋体 (SimSun)',   path: 'C:/Windows/Fonts/simsun.ttc' },
  { name: 'Arial',           path: 'C:/Windows/Fonts/arial.ttf' },
]
// 新增 backend 命令: probe_font_file(path) -> bool (同步, 极快)
// 或复用: invoke('file_read_meta') 之类，没有的话新增 probe_path_exists
```

### 4.3 9 宫格 UI

```html
<div class="wm-position-grid">
  <el-button
    v-for="p in POSITIONS"
    :key="p.value"
    size="small"
    :type="wm_position === p.value ? 'primary' : ''"
    @click="wm_position = p.value"
  >{{ p.icon }}</el-button>
</div>
```

`POSITIONS` 常量:
```typescript
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
]
```

### 4.4 校验规则（compute 计算属性）

```typescript
const wm_canSubmit = computed(() => {
  if (!wm_filePath.value || !useFfmpeg.value) return false
  if (wm_type.value === 'text' || wm_type.value === 'both') {
    if (!wm_text.value.trim()) return false
    const fontPath = getResolvedFontPath()
    if (!fontPath) return false
  }
  if (wm_type.value === 'image' || wm_type.value === 'both') {
    if (!wm_imagePath.value) return false
  }
  if (wm_useTimeRange.value && wm_endTime.value <= wm_startTime.value) return false
  return true
})
```

### 4.5 进度监听（事件 + watch 兜底）

```typescript
// 切换到 watermark Tab 时注册监听
watch(() => activeTab.value, async (t) => {
  if (t === 'watermark') {
    wm_done.value = false
    // 注册事件
    wm_listenOff.value = await listen('video-watermark-progress', (e: any) => {
      if (wm_done.value) return
      wm_progress.value = Math.round(e.payload.progress)
    })
    // 立即尝试 probe 字体
    probeFonts()
  } else {
    wm_listenOff.value?.()
    wm_listenOff.value = null
  }
}, { immediate: true })
```

### 4.6 工作流集成（WorkflowView.vue executeStep）

```typescript
case 'video_watermark': {
  const filePath = resolveStepInput(step, currentOutput, varPool)
  const wmOpts = {
    wmType: step.config.wmType,
    text: resolveVar(step.config.text, varPool),
    fontFile: resolveVar(step.config.fontFile, varPool),
    ...
  }
  const result = await invoke('video_watermark', { path: filePath, options: wmOpts })
  finalOutput = result.outputPath
  addHistory(...)
  break
}
```

---

## 5. 错误处理矩阵

| # | 场景 | 前端表现 | 后端处理 |
|---|---|---|---|
| E1 | ffmpeg 未装 | 顶部 banner + 所有操作按钮 disabled | 返回 Err("需要 ffmpeg")，双重保险 |
| E2 | 未选视频 | 按钮 disabled + 空卡片提示 | N/A |
| E3 | 文字模式但 text 空 | 按钮 disabled + 输入框红色边框 | N/A |
| E4 | 字体路径不存在 | ElMessage.error("字体文件不存在") | validate_options 中 metadata().is_file() 检查 |
| E5 | 图片水印未选图 | 按钮 disabled | N/A |
| E6 | 图片水印文件非图 | ElMessage.error + 文件选择器限制 accept="image/png,image/jpeg,image/webp" | validate_options 用 image crate probe 格式，非图返回 Err |
| E7 | 时间段 start ≥ end | 滑块 input-number 限制 (max=endTime-0.1) | validate_options 再校验一次 |
| E8 | ffmpeg 执行失败 | ElMessage.error(stderr 前 300 字符) | 见 run_ffmpeg_with_progress，失败时拼接 stderr |
| E9 | 输出磁盘满 / 无权限 | ffmpeg 返回失败 → 同 E8 | 同上 |

---

## 6. 历史记录

```typescript
store.addHistory({
  tool: 'video_watermark',
  category: 'media',
  inputPreview: `${wm_fileName.value} + ${wm_type.value === 'text' ? '文字:' + wm_text.value.slice(0, 20) : wm_type.value === 'image' ? '图片:' + wm_imageName.value : '文字+图片'} @ ${wm_position.value}`,
  inputFull: JSON.stringify({
    filePath: wm_filePath.value,
    wmType: wm_type.value,
    text: wm_text.value,
    imagePath: wm_imagePath.value,
    position: wm_position.value,
    offsetX: wm_offsetX.value,
    offsetY: wm_offsetY.value,
    outputFormat: wm_outputFormat.value,
    // ... 其余参数全量
  }),
  outputPreview: wm_result.value
    ? `输出: ${wm_result.value.path.split(/[/\\]/).pop()} (${formatFileSize(wm_result.value.size)})`
    : '成功',
  outputFull: JSON.stringify(wm_result.value),
  status: wm_isProcessing.value ? 'processing' : 'success',
})
```

---

## 7. 验收标准（完成定义）

1. 打开「视频工具 → 加水印」Tab，ffmpeg 缺失时 banner 提示，按钮不可点
2. 选 10s 以下 MP4 小视频，分别验证：
   - 纯文字水印（右下，微软雅黑，32px，带描边）→ 导出视频播放可见水印
   - 纯图片水印（PNG Logo，透明度 0.6，左上，缩放 0.3）→ 正确叠加
   - 文字 + 图片叠加（两者位置不同）→ 一次导出，同时可见
   - 限定时间段 2s~6s → 前 2s 无水印，2-6s 有，6s 后消失
   - 9 宫格 9 个位置随机测 3 个 → 位置正确
   - 偏移 100px → 距离加大
   - 4 种输出格式（mp4/mkv/mov/webm）→ 各成功 1 次
3. 进度条从 0 → ~50 → 100，不跳变
4. 切换到其他 Tab 再切回，水印参数仍保留（组件级 ref，KeepAlive 缓存）
5. 操作历史页面能找到该条记录，双击跳转回水印 Tab 能还原参数
6. 工作流新建「video_watermark」步骤能执行，结果存入变量池
7. `npm run build` + `cargo check` 通过

---

## 8. 非目标 / Out of Scope

- ❌ 实时预览水印效果（成本高，下一迭代可加）
- ❌ 动画水印（旋转/移动/淡入淡出），本版本仅静态位置
- ❌ 批量多视频加水印，本版本一次一个
- ❌ 字体自动回退（如 msyh.ttc 找不到自动试 simhei.ttf），本版本明确报错让用户选
- ❌ drawtext 中文之外的复杂排版（多行、对齐、阴影），下一迭代
