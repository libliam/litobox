<template>
  <div class="tool-container">
    <!-- ffmpeg 状态横幅 -->
    <div class="ffmpeg-banner" :class="{ 'ffmpeg-detected': useFfmpeg, 'ffmpeg-missing': !useFfmpeg }" v-if="ffmpegChecked">
      <template v-if="useFfmpeg">
        <span class="ffmpeg-icon">🚀</span> ffmpeg 已启用，可显示缩略图时间轴，裁剪更精确
      </template>
      <template v-else>
        <span class="ffmpeg-icon">💡</span>
        未检测到 ffmpeg，缩略图不可用，裁剪精度受关键帧限制。
        <span class="ffmpeg-tip">
          安装 ffmpeg 可启用缩略图和高精度裁剪：
          <code class="ffmpeg-cmd">winget install ffmpeg</code>
        </span>
      </template>
    </div>

    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="video-tool-tabs">
        <el-tab-pane label="视频裁剪" name="crop" />
      </el-tabs>
    </div>

    <!-- Tab: 视频裁剪 -->
    <template v-if="activeTab === 'crop'">
      <!-- 文件选择 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择视频文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="openFile" :loading="isLoadingInfo">
                打开文件
              </el-button>
            </div>
          </div>
          <div v-if="filePath" class="video-file-info">
            <span class="file-name">{{ fileName }}</span>
            <span class="file-detail" v-if="videoInfo">
              {{ formatDuration(videoInfo.duration) }} | {{ videoInfo.width }}x{{ videoInfo.height }} |
              {{ videoInfo.codec.toUpperCase() }} | {{ videoInfo.fps.toFixed(1) }}fps |
              {{ videoInfo.bitrate }}kbps | {{ formatFileSize(videoInfo.file_size) }}
            </span>
          </div>
        </div>
      </div>

      <!-- 时间轴 -->
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">时间轴</span>
        </div>
        <div class="card-body">
          <div class="timeline-container" ref="timelineContainer" @contextmenu.prevent>
            <canvas ref="canvasRef" class="timeline-canvas" @mousedown="onCanvasMouseDown" @contextmenu.prevent></canvas>
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
          <div class="timeline-labels">
            <span>{{ formatTime(startTime) }}</span>
            <span>{{ formatTime(endTime) }}</span>
          </div>
        </div>
      </div>

      <!-- 裁剪设置 -->
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
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
                :max="videoInfo.duration"
                :step="0.1"
                :precision="1"
                size="small"
                style="width: 140px"
              />
              <span class="unit-text">秒</span>
            </div>
          </div>
          <div class="segment-info" v-if="videoInfo">
            片段时长: {{ formatDuration(segmentDuration) }}
          </div>
          <div v-if="actualRange" class="keyframe-hint">
            实际裁剪区间（关键帧对齐）: {{ formatTime(actualRange.start) }} - {{ formatTime(actualRange.end) }}
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
      <div v-if="videoInfo && videoInfo.duration > 0" class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button type="primary" size="small" @click="cropVideo" :loading="isProcessing" :disabled="!isRangeValid">
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
interface VideoInfo {
  duration: number
  width: number
  height: number
  codec: string
  fps: number
  bitrate: number
  file_size: number
  format: string
}

interface ThumbnailResult {
  images: string[]
  timestamps: number[]
}

interface CropResult {
  output_path: string
  output_size: number
  duration: number
  actual_start: number | null
  actual_end: number | null
}

// ============ Tab 状态 ============
const activeTab = ref('crop')

// ============ 状态 ============
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

// ============ 计算属性 ============
const segmentDuration = computed(() => endTime.value - startTime.value)
const isRangeValid = computed(() => startTime.value < endTime.value && segmentDuration.value >= 0.1)

// ============ Canvas 时间轴 ============
const canvasRef = ref<HTMLCanvasElement | null>(null)
const timelineContainer = ref<HTMLDivElement | null>(null)

function drawTimeline() {
  const canvas = canvasRef.value
  if (!canvas || !videoInfo.value) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  ctx.scale(dpr, dpr)
  const width = rect.width
  const height = rect.height
  const dur = videoInfo.value.duration

  const style = getComputedStyle(document.documentElement)
  const bgColor = style.getPropertyValue('--bg-input').trim() || '#0d1520'
  const primaryColor = style.getPropertyValue('--accent-cyan').trim() || '#00d4ff'
  const secondaryColor = style.getPropertyValue('--text-secondary').trim() || '#94a3b8'
  ctx.fillStyle = bgColor
  ctx.fillRect(0, 0, width, height)

  const startX = (startTime.value / dur) * width
  const endX = (endTime.value / dur) * width

  // 绘制缩略图
  if (thumbnails.value.images.length > 0) {
    const n = thumbnails.value.images.length
    const thumbWidth = width / n
    const imgPromises: Promise<void>[] = []

    for (let i = 0; i < n; i++) {
      const x = i * thumbWidth
      const img = new Image()
      const promise = new Promise<void>((resolve) => {
        img.onload = () => {
          const h = (thumbWidth / img.width) * img.height
          const y = (height - h) / 2
          ctx.drawImage(img, x, y, thumbWidth, h)
          resolve()
        }
        img.onerror = () => resolve()
      })
      img.src = 'data:image/jpeg;base64,' + thumbnails.value.images[i]
      imgPromises.push(promise)
    }

    Promise.all(imgPromises).then(() => {
      // 绘制选中区域
      drawSelectionOverlay(ctx, startX, endX, width, height, primaryColor)
    })
  } else {
    // 纯文本时间轴：时间刻度线
    ctx.strokeStyle = secondaryColor + '66'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()

    // 刻度标记
    const tickCount = 10
    for (let i = 0; i <= tickCount; i++) {
      const x = (i / tickCount) * width
      const tickHeight = i % 2 === 0 ? 12 : 6
      ctx.strokeStyle = secondaryColor + '88'
      ctx.beginPath()
      ctx.moveTo(x, height / 2 - tickHeight)
      ctx.lineTo(x, height / 2 + tickHeight)
      ctx.stroke()
    }

    drawSelectionOverlay(ctx, startX, endX, width, height, primaryColor)
  }
}

function drawSelectionOverlay(
  ctx: CanvasRenderingContext2D,
  startX: number,
  endX: number,
  _width: number,
  height: number,
  primaryColor: string
) {
  // 选中区域高亮
  ctx.fillStyle = primaryColor + '1A'
  ctx.fillRect(startX, 0, endX - startX, height)

  // 起始/结束边界线
  ctx.strokeStyle = primaryColor
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(startX, 0)
  ctx.lineTo(startX, height)
  ctx.stroke()
  ctx.beginPath()
  ctx.moveTo(endX, 0)
  ctx.lineTo(endX, height)
  ctx.stroke()
}

function timeToPercent(time: number): number {
  if (!videoInfo.value || videoInfo.value.duration <= 0) return 0
  return (time / videoInfo.value.duration) * 100
}

function percentToTime(percent: number): number {
  if (!videoInfo.value) return 0
  return Math.round((percent / 100) * videoInfo.value.duration * 10) / 10
}

// ============ 滑块拖拽 ============
let draggingSlider: 'start' | 'end' | null = null

function onSliderMouseDown(_e: MouseEvent, slider: 'start' | 'end') {
  draggingSlider = slider
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!draggingSlider || !timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  if (draggingSlider === 'start') {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

function onMouseUp() {
  draggingSlider = null
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown(e: MouseEvent) {
  if (!timelineContainer.value || !videoInfo.value) return
  const rect = timelineContainer.value.getBoundingClientRect()
  const percent = ((e.clientX - rect.left) / rect.width) * 100
  const time = percentToTime(Math.max(0, Math.min(100, percent)))

  const startDist = Math.abs(time - startTime.value)
  const endDist = Math.abs(time - endTime.value)

  if (startDist <= endDist) {
    if (time < endTime.value - 0.1) startTime.value = time
  } else {
    if (time > startTime.value + 0.1) endTime.value = time
  }
  drawTimeline()
}

// ============ 文件操作 ============
async function openFile() {
  try {
    error.value = ''
    const selected = await open({
      filters: [{
        name: '视频文件',
        extensions: useFfmpeg.value ? ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v'] : ['mp4', 'm4v']
      }],
      multiple: false,
    })
    if (!selected) return

    filePath.value = selected as string
    fileName.value = (selected as string).split(/[/\\]/).pop() || ''

    isLoadingInfo.value = true
    actualRange.value = null

    const info: VideoInfo = await invoke('get_video_info', { path: filePath.value, useFfmpeg: useFfmpeg.value })
    videoInfo.value = info

    startTime.value = 0
    endTime.value = info.duration

    // 提取缩略图
    const result: ThumbnailResult = await invoke('extract_thumbnails', { path: filePath.value, count: 20 })
    thumbnails.value = result

    await nextTick()
    drawTimeline()
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '加载失败'
    resetForm()
  } finally {
    isLoadingInfo.value = false
  }
}

async function cropVideo() {
  if (!isRangeValid.value) {
    ElMessage.warning('请设置有效的裁剪区间')
    return
  }

  try {
    error.value = ''
    isProcessing.value = true
    cropProgress.value = 0
    actualRange.value = null

    const unlisten = await listen<{ progress: number }>('video-crop-progress', (event) => {
      cropProgress.value = Math.round(event.payload.progress)
    })

    let outputPath: string | null = null
    if (!saveToSamePath.value) {
      const defaultName = fileName.value.replace(/\.[^.]+$/, '') + '_cropped.mp4'
      outputPath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'MP4 视频', extensions: ['mp4'] }],
      })
      if (!outputPath) {
        unlisten()
        isProcessing.value = false
        return
      }
    }

    const result: CropResult = await invoke('video_crop', {
      path: filePath.value,
      options: {
        start_time: startTime.value,
        end_time: endTime.value,
        use_ffmpeg: useFfmpeg.value,
        output_path: outputPath,
      },
    })

    unlisten()
    cropProgress.value = 100

    if (result.actual_start != null && result.actual_end != null) {
      actualRange.value = { start: result.actual_start, end: result.actual_end }
    }

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
  videoInfo.value = null
  thumbnails.value = { images: [], timestamps: [] }
  startTime.value = 0
  endTime.value = 0
  error.value = ''
  actualRange.value = null
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

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
}

// ============ 响应式 ============
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  try {
    useFfmpeg.value = await invoke('check_ffmpeg')
  } catch { /* 忽略 */ }
  ffmpegChecked.value = true

  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => drawTimeline())
    resizeObserver.observe(canvasRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

watch([startTime, endTime], () => drawTimeline())
watch(thumbnails, () => nextTick(() => drawTimeline()), { deep: true })
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.video-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .video-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.video-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.video-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.video-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
.video-file-info {
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

.timeline-container {
  position: relative;
  width: 100%;
  height: 120px;
  cursor: pointer;
}

.timeline-container:has(.slider-handle:active) {
  cursor: col-resize;
}

.timeline-canvas {
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

.timeline-labels {
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

.keyframe-hint {
  margin-top: 4px;
  color: var(--accent-orange);
  font-size: 12px;
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