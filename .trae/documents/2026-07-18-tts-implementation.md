# F11 文字转语音（TTS）实现方案

## 上下文

在 AudioTool 中新增"文字转语音"Tab，调用 Windows SAPI 本地语音引擎将文字转为语音文件（WAV），无需网络、零额外依赖。这是当前音频类唯一的纯待办功能。

## 技术方案

**方案选择：PowerShell + SAPI**

- Windows 自带 `System.Speech.Synthesis.SpeechSynthesizer`，零额外依赖
- 复用项目已有的 `run_powershell()` 模式（`system_info.rs:306-322`）：`CREATE_NO_WINDOW` + `encoding_rs::GBK.decode()`
- 输出 WAV 文件，后续可扩展为 MP3 转换

## 涉及文件（3 个文件）

### 1. `src-tauri/src/audio_tools.rs` — 新增 TTS 后台命令

**新增数据结构：**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsVoice {
    pub name: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOptions {
    pub text: String,
    pub voice_name: Option<String>,
    pub rate: i32,        // -10 ~ 10, 默认 0
    pub volume: i32,      // 0 ~ 100, 默认 100
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub output_path: String,
    pub output_size: u64,
}
```

**新增命令：**

1. `list_tts_voices` — 同步命令（快速），PowerShell 列举已安装语音
2. `tts_generate` — `async fn` + `spawn_blocking`（耗时操作），PowerShell 调用 SAPI 生成 WAV

**PowerShell 脚本模板：**

```powershell
# 列举语音
Add-Type -AssemblyName System.Speech
[System.Speech.Synthesis.SpeechSynthesizer]::new().GetInstalledVoices() | 
  ForEach-Object { @{ name=$_.VoiceInfo.Name; language=$_.VoiceInfo.Culture.DisplayName } } | 
  ConvertTo-Json

# 生成语音
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$s.Rate = {rate}
$s.Volume = {volume}
$s.SetOutputToWaveFile('{output_path}')
$s.Speak('{text}')
$s.Dispose()
```

> ⚠️ 注意：PowerShell 脚本中文本可能有单引号，需要转义（`'` → `''`）

### 2. `src-tauri/src/main.rs` — 注册命令

在 `audio_tools` 命令注册区域（约 L148-167）新增：
```rust
audio_tools::list_tts_voices,
audio_tools::tts_generate,
```

### 3. `src/views/AudioTool.vue` — 新增 Tab "文字转语音"

**Tab 栏新增：**（约 L26）
```html
<el-tab-pane label="文字转语音" name="tts" />
```

**Tab 内容模板：**（在 `</template>` 之前，变速变调 Tab 之后）
- 输入区卡片：文本输入框（el-input type="textarea"，多行），含清空/粘贴按钮
- 设置区卡片：语音选择（下拉）、语速滑块（-10~10）、音量滑块（0~100）
- 操作区卡片：生成按钮（`:loading`）、下载按钮
- 预览区：生成的音频播放器（`<audio>` 标签）

**Script 新增：**
- `ttsState` reactive 对象（text, voiceName, voices列表, rate, volume, isProcessing, resultPath）
- `loadVoices()` 函数 — onMounted 时调用
- `generateTts()` 函数 — 调用 `invoke('tts_generate', ...)`，参考 `convertAudio()` 模式
- 类型定义：`TtsVoice`, `TtsResult`

## 关键设计决策

1. **输出格式固定为 WAV**：SAPI 原生输出 WAV，后续可扩展 MP3 转换（复用现有 `audio_convert`），但首版保持简单
2. **语音列表 onMounted 加载**：进入页面时自动加载可用语音，避免每次生成时重复查询
3. **文本单引号转义**：PowerShell 中文本用单引号包裹，内部单引号需转义为 `''`
4. **耗时操作用 `spawn_blocking`**：长文本 TTS 生成可能耗时数秒，必须放后台线程避免 UI 卡死
5. **不使用 ffmpeg**：TTS 是纯 SAPI 功能，不依赖 ffmpeg

## 验证方式

1. `cargo check` 确认 Rust 编译通过
2. `npm run tauri dev` 启动开发服务器
3. 进入音频工具 → 文字转语音 Tab
4. 输入一段中文文本，选择语音，调整语速/音量
5. 点击"生成语音"，等待完成
6. 点击生成的音频链接试听，确认语音正常
7. 切换不同语音测试（中文语音如 Microsoft Huihui）