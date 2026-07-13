<template>
  <div class="tool-container">
    <!-- ffmpeg 状态提示 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': useFfmpeg, 'ffmpeg-missing': !useFfmpeg }" v-if="ffmpegChecked">
      <template v-if="useFfmpeg">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，处理速度更快、音频信息更准确
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，当前使用内置引擎（功能完整，速度较慢）。
        <span class="ffmpeg-tip">
          安装 ffmpeg 可加速处理：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
          <a href="https://www.wikihow.com/Install-FFmpeg-on-Windows" target="_blank" class="ffmpeg-link">详细教程</a>
        </span>
      </template>
    </div>

    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="audio-tool-tabs">
        <el-tab-pane label="音频裁剪" name="crop" />
      </el-tabs>
    </div>

    <!-- ====== Tab: 音频裁剪 ====== -->
    <template v-if="activeTab === 'crop'">
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
          <div class="waveform-container" ref="waveformContainer" @contextmenu.prevent>
            <canvas ref="canvasRef" class="waveform-canvas" @mousedown="onCanvasMouseDown" @contextmenu.prevent></canvas>
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
              <el-button size="small" @click="togglePreview" :type="isPreviewing ? 'danger' : 'default'" :loading="isPreviewLoading">
                {{ isPreviewing ? (isPreviewLoading ? '加载中…' : '⏹ 停止') : '▶ 预览选中区域' }}
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
          <div class="action-grid" style="margin-top: 8px">
            <div class="action-group">
              <el-checkbox v-model="saveToSamePath" size="small">
                与源文件相同路径
              </el-checkbox>
            </div>
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
    </template>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
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

// ============ Tab 状态 ============
const activeTab = ref('crop')

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
const saveToSamePath = ref(true)
const cropProgress = ref(0)
const useFfmpeg = ref(false)
const ffmpegChecked = ref(false)
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

function onSliderMouseDown(_e: MouseEvent, slider: 'start' | 'end') {
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
let previewAbortFlag = false
const isPreviewLoading = ref(false)

async function togglePreview() {
  if (isPreviewing.value) {
    stopPreview()
    return
  }
  previewAudio()
}

async function previewAudio() {
  // 立即标记为预览中，按钮变为"停止"
  isPreviewing.value = true
  isPreviewLoading.value = true
  previewAbortFlag = false
  error.value = ''
  // 停止之前的播放
  if (audioSource) {
    try { audioSource.stop() } catch (_) { /* 忽略 */ }
    audioSource.disconnect()
    audioSource = null
  }

  try {
    const base64Wav: string = await invoke('get_audio_preview', {
      path: filePath.value,
      start: startTime.value,
      end: endTime.value,
    })

    // 加载期间被取消
    if (previewAbortFlag) return

    const binaryStr = atob(base64Wav)
    const bytes = new Uint8Array(binaryStr.length)
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i)
    }

    if (!audioCtx) {
      audioCtx = new AudioContext()
    }
    await audioCtx.resume()

    if (previewAbortFlag) return

    const audioBuffer = await audioCtx.decodeAudioData(bytes.buffer.slice(0))
    audioSource = audioCtx.createBufferSource()
    audioSource.buffer = audioBuffer
    audioSource.connect(audioCtx.destination)
    audioSource.onended = () => { isPreviewing.value = false }
    audioSource.start()
    isPreviewLoading.value = false
  } catch (e: any) {
    if (!previewAbortFlag) {
      error.value = '预览播放失败: ' + (typeof e === 'string' ? e : e.message || e)
    }
    isPreviewing.value = false
    isPreviewLoading.value = false
  }
}

function stopPreview() {
  previewAbortFlag = true
  if (audioSource) {
    try { audioSource.stop() } catch (_) { /* 忽略已停止错误 */ }
    audioSource.disconnect()
    audioSource = null
  }
  isPreviewing.value = false
  isPreviewLoading.value = false
}

// ============ 文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'm4a'] }],
      multiple: false,
    })
    if (!selected) return

    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''

    isLoadingInfo.value = true
    const info: AudioInfo = await invoke('get_audio_info', { path: filePath.value, useFfmpeg: useFfmpeg.value })
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

    // 监听进度事件
    const unlisten = await listen<{ progress: number }>('audio-crop-progress', (event) => {
      cropProgress.value = Math.round(event.payload.progress)
    })

    // 确定输出路径
    let outputPath: string | null = null
    if (!saveToSamePath.value) {
      const defaultName = fileName.value.replace(/\.[^.]+$/, '') + '_cropped.' + outputFormat.value
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: '音频文件', extensions: [outputFormat.value] }],
      })
      if (!outputPath) {
        unlisten()
        isProcessing.value = false
        return // 用户取消
      }
    }

    const result: CropResult = await invoke('audio_crop', {
      path: filePath.value,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        output_format: outputFormat.value,
        mp3_bitrate: mp3Bitrate.value,
        output_path: outputPath,
        use_ffmpeg: useFfmpeg.value,
      },
    })

    unlisten()
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

onMounted(async () => {
  // 检测 ffmpeg
  try {
    useFfmpeg.value = await invoke('check_ffmpeg')
  } catch { /* 忽略 */ }
  ffmpegChecked.value = true

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
/* ===== Tab 样式 ===== */
.audio-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .audio-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.audio-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.audio-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.audio-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.audio-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.audio-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
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

/* ffmpeg 状态横幅 */
.ffmpeg-banner {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.ffmpeg-banner.ffmpeg-detected {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--accent-green);
}

.ffmpeg-banner.ffmpeg-missing {
  background: rgba(59, 130, 246, 0.12);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: var(--accent-blue);
}

.ffmpeg-icon {
  font-size: 16px;
}

.ffmpeg-link {
  color: var(--accent-cyan);
  margin-left: 4px;
}

.ffmpeg-link:hover {
  text-decoration: underline;
}

.ffmpeg-tip {
  margin-left: 8px;
}

.ffmpeg-cmd {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--accent-orange);
  user-select: all;
}
</style>