<template>
  <div class="tool-container">
    <!-- 顶部操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">图片对比</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 选择两张图片进行像素级对比（UI 还原/版本差异比对）</p>
                <p>• 三种模式：滑动对比 / 差异高亮 / 半透明叠加</p>
                <p>• 差异高亮模式支持调节容差与查看差异率</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" :loading="loadingA" @click="pickImage('A')">
            {{ fileNameA || '选择图片 A' }}
          </el-button>
          <el-button size="small" type="primary" plain :loading="loadingB" @click="pickImage('B')">
            {{ fileNameB || '选择图片 B' }}
          </el-button>
          <el-button v-if="imgA || imgB" size="small" @click="clearAll">清除</el-button>
        </div>
      </div>
      <div class="card-body" v-if="fileNameA || fileNameB">
        <div class="file-info">
          <span v-if="fileNameA" class="file-tag tag-a">A：{{ fileNameA }}</span>
          <span v-if="fileNameB" class="file-tag tag-b">B：{{ fileNameB }}</span>
          <span v-if="sizeInfo" class="size-info">{{ sizeInfo.width }} × {{ sizeInfo.height }}</span>
        </div>
      </div>
    </div>

    <!-- 模式与控制 -->
    <div v-if="imgA && imgB" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">对比模式</span>
        </div>
        <div class="card-actions">
          <el-radio-group v-model="mode" size="small">
            <el-radio-button value="slider">滑动对比</el-radio-button>
            <el-radio-button value="diff">差异高亮</el-radio-button>
            <el-radio-button value="overlay">叠加对比</el-radio-button>
          </el-radio-group>
        </div>
      </div>
      <div class="card-body">
        <!-- 差异模式控制 -->
        <div v-if="mode === 'diff'" class="ctrl-row">
          <span class="ctrl-label">容差（每通道）</span>
          <el-slider v-model="tolerance" :min="0" :max="128" :step="1" style="flex: 1" @change="computeDiff" />
          <span class="ctrl-value">{{ tolerance }}</span>
          <div class="diff-stats">
            <span class="diff-pct" :class="{ 'diff-ok': diffRatio !== null && diffRatio < 0.05 }">
              {{ diffRatio === null ? '--' : (diffRatio * 100).toFixed(2) + '%' }}
            </span>
            <span class="diff-label">差异率</span>
          </div>
        </div>
        <!-- 叠加模式控制 -->
        <div v-else-if="mode === 'overlay'" class="ctrl-row">
          <span class="ctrl-label">B 图透明度</span>
          <el-slider v-model="overlayAlpha" :min="10" :max="100" :step="5" style="flex: 1" />
          <span class="ctrl-value">{{ overlayAlpha }}%</span>
        </div>
        <!-- 滑动模式提示 -->
        <div v-else class="ctrl-row">
          <span class="ctrl-hint">拖动分割线左右查看两张图，左侧为 B，右侧为 A</span>
        </div>
      </div>
    </div>

    <!-- 对比展示区 -->
    <div v-if="imgA && imgB" class="tool-card">
      <div class="card-body compare-body">
        <!-- 滑动对比 -->
        <div
          v-if="mode === 'slider'"
          ref="compareBox"
          class="compare-box"
          :style="{ aspectRatio: aspectRatio }"
          @mousedown="onSliderDown"
          @mousemove="onSliderMove"
          @mouseup="onSliderUp"
          @mouseleave="onSliderUp"
        >
          <img :src="dataUrlA" class="base-img" draggable="false" />
          <div class="overlay-wrap" :style="{ clipPath: 'inset(0 ' + (100 - sliderPct) + '% 0 0)' }">
            <img :src="dataUrlB" class="base-img" draggable="false" />
          </div>
          <div class="slider-line" :style="{ left: sliderPct + '%' }">
            <div class="slider-handle">⇔</div>
          </div>
        </div>
        <!-- 差异高亮 -->
        <div v-else-if="mode === 'diff'" class="canvas-wrap">
          <canvas ref="diffCanvas" class="compare-canvas" />
        </div>
        <!-- 叠加对比 -->
        <div v-else class="canvas-wrap">
          <canvas ref="overlayCanvas" class="compare-canvas" />
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="!imgA && !imgB" class="tool-card">
      <div class="card-body empty-state">
        <p class="empty-icon">🖼️</p>
        <p>选择两张图片开始对比（支持 PNG / JPG / WebP / BMP / GIF）</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import {
  loadImageFromUrl, drawScaledImage, unionSize, compareImages, selfCheck,
} from '@/utils/imageCompareUtils'

// ============ 状态 ============
const imgA = ref<HTMLImageElement | null>(null)
const imgB = ref<HTMLImageElement | null>(null)
const fileNameA = ref('')
const fileNameB = ref('')
const loadingA = ref(false)
const loadingB = ref(false)
const mode = ref<'slider' | 'diff' | 'overlay'>('slider')
const tolerance = ref(32)
const overlayAlpha = ref(50)
const diffRatio = ref<number | null>(null)
const sizeInfo = ref<{ width: number; height: number } | null>(null)

// 滑动对比状态
const sliderPct = ref(50)
const dragging = ref(false)
const compareBox = ref<HTMLDivElement | null>(null)

// 画布
const diffCanvas = ref<HTMLCanvasElement | null>(null)
const overlayCanvas = ref<HTMLCanvasElement | null>(null)

// 统一尺寸后的 dataURL（A / B / 差异合成）
const dataUrlA = ref('')
const dataUrlB = ref('')

// ============ 工具 ============
const mimeOf = (path: string): string => {
  const ext = path.split('.').pop()?.toLowerCase()
  const map: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg',
    webp: 'image/webp', bmp: 'image/bmp', gif: 'image/gif', svg: 'image/svg+xml', ico: 'image/x-icon',
  }
  return map[ext || ''] || 'image/png'
}

const basename = (path: string): string => path.split(/[\\/]/).pop() || path

// ============ 选择图片 ============
const pickImage = async (which: 'A' | 'B') => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    multiple: false,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (typeof selected !== 'string' || !selected) return
  if (which === 'A') loadingA.value = true
  else loadingB.value = true
  try {
    const base64 = await invoke<string>('read_file_base64', { filePath: selected })
    const img = await loadImageFromUrl(`data:${mimeOf(selected)};base64,${base64}`)
    if (which === 'A') {
      imgA.value = img
      fileNameA.value = basename(selected)
    } else {
      imgB.value = img
      fileNameB.value = basename(selected)
    }
    await alignAndRender()
  } catch (e: any) {
    ElMessage.error('图片加载失败: ' + (e.message || e))
  } finally {
    loadingA.value = false
    loadingB.value = false
  }
}

// ============ 统一尺寸并渲染 ============
let alignedA: ImageData | null = null
let alignedB: ImageData | null = null

const alignAndRender = async () => {
  if (!imgA.value || !imgB.value) return
  const sa = drawScaledImage(imgA.value)
  const sb = drawScaledImage(imgB.value)
  const union = unionSize(sa.imgData.width, sa.imgData.height, sb.imgData.width, sb.imgData.height)
  sizeInfo.value = union

  // 在统一画布上对齐绘制（左上对齐，透明补白）
  const canvasA = document.createElement('canvas')
  canvasA.width = union.width
  canvasA.height = union.height
  const ctxA = canvasA.getContext('2d')!
  ctxA.drawImage(sa.canvas, 0, 0)
  alignedA = ctxA.getImageData(0, 0, union.width, union.height)

  const canvasB = document.createElement('canvas')
  canvasB.width = union.width
  canvasB.height = union.height
  const ctxB = canvasB.getContext('2d')!
  ctxB.drawImage(sb.canvas, 0, 0)
  alignedB = ctxB.getImageData(0, 0, union.width, union.height)

  dataUrlA.value = canvasA.toDataURL('image/png')
  dataUrlB.value = canvasB.toDataURL('image/png')

  computeDiff()
  await nextTick()
  renderDiffCanvas()
  renderOverlayCanvas()
}

// ============ 差异计算与渲染 ============
const computeDiff = () => {
  const a = alignedA
  const b = alignedB
  if (!a || !b) return
  const { result } = compareImages(a, b, tolerance.value)
  diffRatio.value = result.diffRatio
}

const renderDiffCanvas = () => {
  const a = alignedA
  const b = alignedB
  if (!a || !b || !diffCanvas.value) return
  const canvas = diffCanvas.value
  canvas.width = a.width
  canvas.height = a.height
  const ctx = canvas.getContext('2d')!
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.putImageData(a, 0, 0)
  const { diffData } = compareImages(a, b, tolerance.value)
  ctx.putImageData(diffData, 0, 0)
}

const renderOverlayCanvas = () => {
  const a = alignedA
  const b = alignedB
  if (!a || !b || !overlayCanvas.value) return
  const canvas = overlayCanvas.value
  canvas.width = a.width
  canvas.height = a.height
  const ctx = canvas.getContext('2d')!
  // putImageData 不受 globalAlpha 影响，需先转成 canvas 再用 drawImage 合成
  const toCanvas = (imgData: ImageData): HTMLCanvasElement => {
    const c = document.createElement('canvas')
    c.width = imgData.width
    c.height = imgData.height
    c.getContext('2d')!.putImageData(imgData, 0, 0)
    return c
  }
  const ca = toCanvas(a)
  const cb = toCanvas(b)
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.globalAlpha = 1
  ctx.drawImage(ca, 0, 0)
  ctx.globalAlpha = overlayAlpha.value / 100
  ctx.drawImage(cb, 0, 0)
  ctx.globalAlpha = 1
}

// 容差变化时同步刷新差异画布
watch(tolerance, () => {
  renderDiffCanvas()
})
watch(overlayAlpha, () => {
  renderOverlayCanvas()
})

// 切换模式时 canvas 刚挂载，需要重新绘制
watch(mode, async (val) => {
  const a = alignedA
  const b = alignedB
  if (!a || !b) return
  await nextTick()
  if (val === 'diff') renderDiffCanvas()
  else if (val === 'overlay') renderOverlayCanvas()
})

// ============ 滑动对比交互 ============
const onSliderDown = (e: MouseEvent) => {
  dragging.value = true
  updateSlider(e)
}
const onSliderMove = (e: MouseEvent) => {
  if (dragging.value) updateSlider(e)
}
const onSliderUp = () => {
  dragging.value = false
}
const updateSlider = (e: MouseEvent) => {
  const box = compareBox.value
  if (!box) return
  const rect = box.getBoundingClientRect()
  sliderPct.value = Math.min(100, Math.max(0, ((e.clientX - rect.left) / rect.width) * 100))
}

// ============ 清除 ============
const clearAll = () => {
  imgA.value = null
  imgB.value = null
  fileNameA.value = ''
  fileNameB.value = ''
  diffRatio.value = null
  sizeInfo.value = null
  alignedA = null
  alignedB = null
  dataUrlA.value = ''
  dataUrlB.value = ''
  sliderPct.value = 50
}

// ============ 布局 ============
const aspectRatio = computed(() => {
  if (!sizeInfo.value) return '1'
  return `${sizeInfo.value.width} / ${sizeInfo.value.height}`
})

// ============ 初始化自检 ============
const init = () => {
  const errors = selfCheck()
  if (errors.length) {
    console.error('[ImageCompareTool] selfCheck 失败:', errors)
  }
}
init()
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
  flex-wrap: wrap;
  gap: 8px;
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.file-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.file-tag {
  font-size: 12px;
  padding: 3px 10px;
  border-radius: 10px;
  font-family: 'JetBrains Mono', Consolas, monospace;
}
.tag-a { color: #00d4ff; background: rgba(0, 212, 255, 0.1); border: 1px solid rgba(0, 212, 255, 0.3); }
.tag-b { color: #f59e0b; background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.3); }
.size-info { color: var(--text-muted); font-size: 12px; font-family: 'JetBrains Mono', Consolas, monospace; }

/* 控制行 */
.ctrl-row {
  display: flex;
  align-items: center;
  gap: 16px;
}
.ctrl-label { font-size: 13px; color: var(--text-secondary); white-space: nowrap; }
.ctrl-value {
  min-width: 40px;
  text-align: right;
  font-size: 13px;
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', Consolas, monospace;
}
.ctrl-hint { font-size: 12px; color: var(--text-muted); }
.diff-stats {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-left: 12px;
  padding-left: 12px;
  border-left: 1px solid var(--border-color);
}
.diff-pct {
  font-size: 20px;
  font-weight: 700;
  font-family: 'JetBrains Mono', Consolas, monospace;
  color: #ef4444;
}
.diff-pct.diff-ok { color: #10b981; }
.diff-label { font-size: 12px; color: var(--text-muted); }

/* 对比展示 */
.compare-body {
  padding: 20px;
}
.compare-box {
  position: relative;
  width: 100%;
  max-height: 70vh;
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: repeating-conic-gradient(#1a1a1a 0% 25%, #222 0% 50%) 50% / 16px 16px;
  cursor: col-resize;
  user-select: none;
}
.base-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
  display: block;
  pointer-events: none;
}
.overlay-wrap {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 100%;
  overflow: hidden;
}
.slider-line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background: rgba(0, 212, 255, 0.9);
  transform: translateX(-1px);
  pointer-events: none;
}
.slider-handle {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: rgba(0, 212, 255, 0.9);
  color: #0a0f1a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  font-weight: 700;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  pointer-events: none;
}

.canvas-wrap {
  display: flex;
  justify-content: center;
}
.compare-canvas {
  max-width: 100%;
  max-height: 70vh;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: repeating-conic-gradient(#1a1a1a 0% 25%, #222 0% 50%) 50% / 16px 16px;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 48px 20px;
}
.empty-icon { font-size: 40px; margin-bottom: 12px; }
.empty-state p { color: var(--text-muted); font-size: 13px; margin: 4px 0; }
</style>
