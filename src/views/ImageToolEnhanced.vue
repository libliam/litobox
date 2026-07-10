<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="image-tabs">
        <el-tab-pane label="批量压缩/转换" name="compress" />
        <el-tab-pane label="图片拼图" name="merge" />
        <el-tab-pane label="加水印" name="watermark" />
        <el-tab-pane label="调色板提取" name="palette" />
      </el-tabs>
    </div>

    <!-- Tab 1: 批量压缩/格式转换 -->
    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectCompressFiles">选择图片</el-button>
          <el-button v-if="compressFiles.length" size="small" @click="clearCompressFiles">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="compressFiles.length" class="file-list">
          <div v-for="(f, i) in compressFiles" :key="i" class="file-item">
            <span class="file-name">{{ f.name }}</span>
            <span class="file-size">{{ formatBytes(f.size) }}</span>
          </div>
        </div>
        <div v-else class="upload-hint">选择多张图片进行批量压缩或格式转换</div>
      </div>
    </div>
    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header"><span class="card-title">压缩设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">目标格式</span>
            <div class="group-buttons">
              <el-button size="small" :type="compressFormat === 'jpg' ? 'primary' : ''" @click="compressFormat = 'jpg'">JPG</el-button>
              <el-button size="small" :type="compressFormat === 'png' ? 'primary' : ''" @click="compressFormat = 'png'">PNG</el-button>
              <el-button size="small" :type="compressFormat === 'webp' ? 'primary' : ''" @click="compressFormat = 'webp'">WebP</el-button>
            </div>
          </div>
          <div v-if="compressFormat === 'jpg'" class="action-group">
            <span class="group-label">质量: {{ compressQuality }}%</span>
            <el-slider v-model="compressQuality" :min="10" :max="100" style="width: 160px" />
          </div>
          <div class="action-group">
            <span class="group-label">最大尺寸</span>
            <el-input-number v-model="maxWidth" :min="0" :max="8192" size="small" placeholder="宽" controls-position="right" style="width: 100px" />
            <span>×</span>
            <el-input-number v-model="maxHeight" :min="0" :max="8192" size="small" placeholder="高" controls-position="right" style="width: 100px" />
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="!compressFiles.length" :loading="compressLoading" @click="handleCompress">开始转换</el-button>
          <el-button size="small" :disabled="!compressResults.length" @click="downloadAllCompressResults">下载全部</el-button>
        </div>
        <div v-if="compressResults.length" class="result-list">
          <div v-for="(r, i) in compressResults" :key="i" class="result-item">
            <span class="result-name">{{ r.name }}</span>
            <span class="result-size">{{ formatBytes(r.original_size) }} → {{ formatBytes(r.compressed_size) }}</span>
            <span class="result-ratio">{{ r.ratio }}%</span>
            <el-button size="small" text type="primary" @click="downloadCompressResult(r)">下载</el-button>
          </div>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 2: 自由画布拼图 -->
    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片操作</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectMergeFiles">选择图片</el-button>
          <el-button v-if="canvasImages.length" size="small" @click="clearCanvasImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="canvasImages.length" class="merge-file-list">
          <div v-for="(f, i) in canvasImages" :key="f.id" class="merge-file-item">
            <img v-if="f.thumb" :src="f.thumb" class="merge-thumb" />
            <span class="file-name">{{ f.name }}</span>
            <span class="file-size">{{ formatBytes(f.size) }}</span>
            <el-button size="small" text type="danger" @click="removeCanvasImage(i)">移除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">选择图片添加到画布</div>
      </div>
    </div>

    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header"><span class="card-title">画布</span></div>
      <div class="card-body">
        <canvas ref="fabricCanvasRef" class="fabric-canvas"></canvas>
      </div>
    </div>

    <!-- 右键菜单（放在最外层避免被裁剪） -->
    <div v-if="contextMenuVisible" class="canvas-context-menu" :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }">
      <div class="context-menu-item" @click="bringToFront">置顶</div>
      <div class="context-menu-item" @click="sendToBack">置底</div>
    </div>

    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header"><span class="card-title">输出设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">画布尺寸</span>
            <div class="group-buttons">
              <el-button size="small" :type="canvasSizeMode === 'auto' ? 'primary' : ''" @click="canvasSizeMode = 'auto'">自动适应</el-button>
              <el-button size="small" :type="canvasSizeMode === 'manual' ? 'primary' : ''" @click="canvasSizeMode = 'manual'">手动指定</el-button>
            </div>
          </div>
          <div v-if="canvasSizeMode === 'manual'" class="action-group">
            <span class="group-label">尺寸 (px)</span>
            <el-input-number v-model="manualCanvasWidth" :min="100" :max="8000" size="small" placeholder="宽" controls-position="right" style="width: 100px" />
            <span>×</span>
            <el-input-number v-model="manualCanvasHeight" :min="100" :max="8000" size="small" placeholder="高" controls-position="right" style="width: 100px" />
          </div>
          <div class="action-group">
            <span class="group-label">背景色</span>
            <el-color-picker v-model="mergeBgColor" size="small" show-alpha />
            <el-button size="small" @click="mergeBgColor = ''" style="margin-left: 8px">透明</el-button>
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="canvasImages.length === 0" :loading="mergeLoading" @click="handleCanvasMerge">生成拼图</el-button>
          <el-button size="small" :disabled="!mergeResult" @click="downloadMergeResult">下载结果</el-button>
        </div>
        <div v-if="mergeResult" class="preview-area">
          <img :src="mergeResultUrl" class="merge-preview" />
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 3: 加水印 -->
    <div v-if="activeTab === 'watermark'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectWatermarkFile">选择图片</el-button>
          <el-button v-if="watermarkFile" size="small" @click="clearWatermarkFile">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="watermarkFile" class="image-info">
          <span class="info-name">{{ watermarkFile.name }}</span>
          <span class="info-size">{{ formatBytes(watermarkFile.size) }}</span>
        </div>
        <div v-else class="upload-hint">选择一张图片添加水印</div>
      </div>
    </div>
    <div v-if="activeTab === 'watermark'" class="tool-card">
      <div class="card-header"><span class="card-title">水印设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">水印文字</span>
            <el-input v-model="watermarkText" size="small" placeholder="水印文字" style="width: 200px" />
          </div>
          <div class="action-group">
            <span class="group-label">位置</span>
            <el-select v-model="watermarkPos" size="small" style="width: 120px">
              <el-option label="左上" value="topLeft" />
              <el-option label="右上" value="topRight" />
              <el-option label="居中" value="center" />
              <el-option label="左下" value="bottomLeft" />
              <el-option label="右下" value="bottomRight" />
            </el-select>
          </div>
          <div class="action-group">
            <span class="group-label">字体大小</span>
            <el-input-number v-model="watermarkFontSize" :min="8" :max="200" size="small" style="width: 80px" />
          </div>
          <div class="action-group">
            <span class="group-label">透明度</span>
            <el-slider v-model="watermarkOpacity" :min="0.1" :max="1" :step="0.1" style="width: 120px" />
          </div>
          <div class="action-group">
            <span class="group-label">颜色</span>
            <el-color-picker v-model="watermarkColor" size="small" />
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="!watermarkFile || !watermarkText" :loading="watermarkLoading" @click="handleWatermark">添加水印</el-button>
          <el-button size="small" :disabled="!watermarkResult" @click="downloadWatermarkResult">下载结果</el-button>
        </div>
        <div v-if="watermarkResult" class="preview-area">
          <img :src="watermarkResultUrl" class="merge-preview" />
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 4: 调色板提取 -->
    <div v-if="activeTab === 'palette'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectPaletteFile">选择图片</el-button>
          <el-button v-if="paletteFile" size="small" @click="clearPaletteFile">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="paletteFile" class="image-info">
          <span class="info-name">{{ paletteFile.name }}</span>
          <span class="info-size">{{ formatBytes(paletteFile.size) }}</span>
        </div>
        <div v-else class="upload-hint">选择图片提取主色调</div>
      </div>
    </div>
    <div v-if="activeTab === 'palette'" class="tool-card">
      <div class="card-header"><span class="card-title">提取设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">颜色数量</span>
            <el-input-number v-model="paletteCount" :min="2" :max="32" size="small" style="width: 80px" />
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="!paletteFile" :loading="paletteLoading" @click="handlePalette">提取调色板</el-button>
          <el-button size="small" :disabled="!paletteColors.length" @click="copyPaletteCss">复制 CSS 变量</el-button>
        </div>
        <div v-if="paletteColors.length" class="palette-grid">
          <div v-for="(c, i) in paletteColors" :key="i" class="palette-item" @click="copyPaletteColor(c.hex)">
            <div class="palette-swatch" :style="{ background: c.hex }" />
            <div class="palette-info">
              <span class="palette-hex">{{ c.hex }}</span>
              <span class="palette-rgb">rgb({{ c.rgb.join(',') }})</span>
              <span class="palette-ratio">{{ (c.ratio * 100).toFixed(1) }}%</span>
            </div>
          </div>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { saveFileWithDialog } from '@/utils/fileSaver'

const activeTab = ref('compress')
const error = ref('')

// ============ 通用 ============
const formatBytes = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1048576).toFixed(2) + ' MB'
}

const base64ToBlob = (base64: string, mime = 'image/png') => {
  const byteChars = atob(base64)
  const byteNumbers = new Uint8Array(byteChars.length)
  for (let i = 0; i < byteChars.length; i++) byteNumbers[i] = byteChars.charCodeAt(i)
  return new Blob([byteNumbers], { type: mime })
}

// ============ Tab 1: 批量压缩/转换 ============
interface CompressFile {
  name: string
  path: string
  size: number
}

interface CompressResultItem {
  name: string
  original_size: number
  compressed_size: number
  ratio: string
  base64: string
  format: string
}

const compressFiles = ref<CompressFile[]>([])
const compressFormat = ref('jpg')
const compressQuality = ref(80)
const maxWidth = ref(0)
const maxHeight = ref(0)
const compressResults = ref<CompressResultItem[]>([])
const compressLoading = ref(false)

const selectCompressFiles = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  compressFiles.value = paths.map(p => ({
    name: p.split(/[/\\]/).pop() || '',
    path: p,
    size: 0,
  }))
  // 获取文件大小和尺寸
  for (const f of compressFiles.value) {
    try {
      const info = await invoke<{ size: number; width: number; height: number }>('get_file_info', {
        filePath: f.path,
      })
      f.size = info.size
    } catch (e) {
      console.warn('获取文件信息失败:', f.name, e)
    }
  }
  error.value = ''
}

const clearCompressFiles = () => {
  compressFiles.value = []
  compressResults.value = []
  error.value = ''
}

const handleCompress = async () => {
  if (!compressFiles.value.length) return
  error.value = ''
  compressResults.value = []
  compressLoading.value = true
  try {
    for (const f of compressFiles.value) {
      const result = await invoke<{
        original_size: number; compressed_size: number; base64: string; format: string
      }>('image_compress', {
        filePath: f.path,
        quality: compressFormat.value === 'jpg' ? compressQuality.value : 100,
        format: compressFormat.value,
        maxWidth: maxWidth.value > 0 ? maxWidth.value : null,
        maxHeight: maxHeight.value > 0 ? maxHeight.value : null,
      })
      compressResults.value.push({
        name: f.name,
        original_size: result.original_size,
        compressed_size: result.compressed_size,
        ratio: ((result.compressed_size / Math.max(result.original_size, 1)) * 100).toFixed(1),
        base64: result.base64,
        format: result.format,
      })
    }
    ElMessage.success(`转换完成，共 ${compressResults.value.length} 张`)
  } catch (e: any) {
    error.value = e
  } finally {
    compressLoading.value = false
  }
}

const downloadCompressResult = async (r: CompressResultItem) => {
  const ext = r.format === 'jpeg' ? 'jpg' : r.format
  const mime = `image/${ext === 'jpg' ? 'jpeg' : ext}`
  const blob = base64ToBlob(r.base64, mime)
  const name = r.name.replace(/\.[^.]+$/, '') || 'image'
  await saveFileWithDialog(blob, `${name}_compressed.${ext}`, ext)
}

const downloadAllCompressResults = async () => {
  for (const r of compressResults.value) {
    await downloadCompressResult(r)
  }
  ElMessage.success('全部下载完成')
}

// ============ Tab 2: 自由画布拼图 ============
interface CanvasImage {
  id: string
  path: string
  name: string
  size: number
  thumb?: string
  left: number
  top: number
  scaleX: number
  scaleY: number
  angle: number
}

const fabricCanvasRef = ref<HTMLCanvasElement>()
const fabricCanvas = ref<fabric.Canvas>()
const canvasImages = ref<CanvasImage[]>([])
const canvasSizeMode = ref<'auto' | 'manual'>('auto')
const manualCanvasWidth = ref(800)
const manualCanvasHeight = ref(600)
const mergeBgColor = ref('#ffffff')
const mergeResult = ref<{ base64: string; width: number; height: number } | null>(null)
const mergeLoading = ref(false)

// 右键菜单
const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })
const contextMenuTargetId = ref<string | null>(null)

const mergeResultUrl = computed(() =>
  mergeResult.value ? 'data:image/png;base64,' + mergeResult.value.base64 : ''
)

// 对齐线
const alignLines = ref<fabric.Line[]>([])
const SNAP_THRESHOLD = 5

// 初始化画布
const initCanvas = () => {
  if (fabricCanvasRef.value && !fabricCanvas.value) {
    fabricCanvas.value = new fabric.Canvas(fabricCanvasRef.value, {
      width: 800,
      height: 600,
      backgroundColor: 'transparent',
      selection: true,
    })

    // 绘制棋盘格背景
    drawCheckerboard(fabricCanvas.value)

    // 监听对象修改事件，同步状态
    fabricCanvas.value.on('object:modified', (e: any) => {
      if (e.target) {
        const obj = e.target as fabric.FabricImage
        const id = (obj as any).customId
        const imgData = canvasImages.value.find(img => img.id === id)
        if (imgData) {
          imgData.left = obj.left || 0
          imgData.top = obj.top || 0
          imgData.scaleX = obj.scaleX || 1
          imgData.scaleY = obj.scaleY || 1
          imgData.angle = obj.angle || 0
        }
      }
    })

    // 右键菜单
    fabricCanvas.value.on('contextmenu', (e: any) => {
      if (e.target) {
        e.e.preventDefault()
        contextMenuTargetId.value = (e.target as any).customId || null
        contextMenuPos.value = { x: e.e.clientX, y: e.e.clientY }
        contextMenuVisible.value = true
      }
    })

    // 吸附对齐 - 拖拽时
    fabricCanvas.value.on('object:moving', (e: any) => {
      if (!e.target) return
      const obj = e.target
      const canvas = fabricCanvas.value!
      const center = obj.getCenterPoint()
      const bounds = obj.getBoundingRect()

      // 清除旧对齐线
      clearAlignLines()

      // 与其他对象对齐
      canvas.getObjects().forEach((other: any) => {
        if (other === obj || !other.getBoundingRect) return
        const otherBounds = other.getBoundingRect()
        const otherCenter = other.getCenterPoint()

        // 水平对齐
        if (Math.abs(center.x - otherCenter.x) < SNAP_THRESHOLD) {
          obj.set({ left: otherCenter.x - obj.width! * obj.scaleX! / 2 })
          addAlignLine(canvas, otherCenter.x, 0, otherCenter.x, canvas.height!)
        }
        // 垂直对齐
        if (Math.abs(center.y - otherCenter.y) < SNAP_THRESHOLD) {
          obj.set({ top: otherCenter.y - obj.height! * obj.scaleY! / 2 })
          addAlignLine(canvas, 0, otherCenter.y, canvas.width!, otherCenter.y)
        }
        // 左边缘对齐
        if (Math.abs(bounds.left - otherBounds.left) < SNAP_THRESHOLD) {
          obj.set({ left: otherBounds.left })
          addAlignLine(canvas, otherBounds.left, 0, otherBounds.left, canvas.height!)
        }
        // 右边缘对齐
        if (Math.abs(bounds.left + bounds.width - otherBounds.left - otherBounds.width) < SNAP_THRESHOLD) {
          obj.set({ left: otherBounds.left + otherBounds.width - bounds.width })
          addAlignLine(canvas, otherBounds.left + otherBounds.width, 0, otherBounds.left + otherBounds.width, canvas.height!)
        }
        // 上边缘对齐
        if (Math.abs(bounds.top - otherBounds.top) < SNAP_THRESHOLD) {
          obj.set({ top: otherBounds.top })
          addAlignLine(canvas, 0, otherBounds.top, canvas.width!, otherBounds.top)
        }
        // 下边缘对齐
        if (Math.abs(bounds.top + bounds.height - otherBounds.top - otherBounds.height) < SNAP_THRESHOLD) {
          obj.set({ top: otherBounds.top + otherBounds.height - bounds.height })
          addAlignLine(canvas, 0, otherBounds.top + otherBounds.height, canvas.width!, otherBounds.top + otherBounds.height)
        }
      })

      // 与画布中心对齐
      if (Math.abs(center.x - canvas.width! / 2) < SNAP_THRESHOLD) {
        obj.set({ left: canvas.width! / 2 - obj.width! * obj.scaleX! / 2 })
        addAlignLine(canvas, canvas.width! / 2, 0, canvas.width! / 2, canvas.height!)
      }
      if (Math.abs(center.y - canvas.height! / 2) < SNAP_THRESHOLD) {
        obj.set({ top: canvas.height! / 2 - obj.height! * obj.scaleY! / 2 })
        addAlignLine(canvas, 0, canvas.height! / 2, canvas.width!, canvas.height! / 2)
      }

      canvas.renderAll()
    })

    // 拖拽结束清除对齐线
    fabricCanvas.value.on('object:mouseup', () => {
      clearAlignLines()
    })

    // 画布初始化后，把已有的图片加上去
    for (const imgData of canvasImages.value) {
      addImageToCanvas(imgData)
    }
  }
}

// 绘制棋盘格背景
const drawCheckerboard = (canvas: fabric.Canvas) => {
  const size = 20
  const w = canvas.width!
  const h = canvas.height!
  for (let y = 0; y < h; y += size) {
    for (let x = 0; x < w; x += size) {
      const color = ((x / size + y / size) % 2 === 0) ? '#e0e0e0' : '#ffffff'
      const rect = new fabric.Rect({
        left: x,
        top: y,
        width: size,
        height: size,
        fill: color,
        selectable: false,
        evented: false,
      })
      canvas.add(rect)
    }
  }
}

// 添加对齐线
const addAlignLine = (canvas: fabric.Canvas, x1: number, y1: number, x2: number, y2: number) => {
  const line = new fabric.Line([x1, y1, x2, y2], {
    stroke: '#ff0000',
    strokeWidth: 1,
    selectable: false,
    evented: false,
    strokeDashArray: [5, 5],
  })
  canvas.add(line)
  alignLines.value.push(line)
}

// 清除对齐线
const clearAlignLines = () => {
  if (!fabricCanvas.value) return
  alignLines.value.forEach(line => fabricCanvas.value!.remove(line))
  alignLines.value = []
}

// 添加图片到画布（使用原图 base64）
const addImageToCanvas = (imgData: CanvasImage) => {
  if (!fabricCanvas.value) return
  // 使用 read_file_base64 读取原图
  invoke<string>('read_file_base64', { filePath: imgData.path }).then((base64) => {
    // 检测图片类型
    const ext = imgData.path.split('.').pop()?.toLowerCase() || 'png'
    const mimeType = ext === 'jpg' || ext === 'jpeg' ? 'image/jpeg' : `image/${ext}`
    const url = `data:${mimeType};base64,${base64}`
    
    return fabric.FabricImage.fromURL(url)
  }).then((img) => {
    // 如果图片太大，缩放到画布可容纳的大小
    const maxW = fabricCanvas.value!.width! * 0.6
    const maxH = fabricCanvas.value!.height! * 0.6
    let scale = 1
    if (img.width! > maxW || img.height! > maxH) {
      scale = Math.min(maxW / img.width!, maxH / img.height!)
    }
    img.set({
      left: imgData.left,
      top: imgData.top,
      scaleX: scale,
      scaleY: scale,
      angle: imgData.angle,
      cornerSize: 12,
      cornerColor: '#00ffff',
      cornerStrokeColor: '#ffffff',
      transparentCorners: false,
      borderColor: '#00ffff',
      borderScaleFactor: 2,
    })
    ;(img as any).customId = imgData.id
    fabricCanvas.value!.add(img)
    fabricCanvas.value!.renderAll()
  }).catch((err) => {
    console.error('加载图片失败:', imgData.name, err)
  })
}

// 监听 Tab 切换，切换到 merge 时初始化画布
watch(() => activeTab.value, (newTab) => {
  if (newTab === 'merge') {
    nextTick(() => {
      initCanvas()
    })
  }
})

onMounted(() => {
  if (activeTab.value === 'merge') {
    nextTick(() => {
      initCanvas()
    })
  }
})

onUnmounted(() => {
  fabricCanvas.value?.dispose()
  fabricCanvas.value = undefined
})

// 点击其他地方关闭右键菜单
const closeContextMenu = () => {
  contextMenuVisible.value = false
}

if (typeof window !== 'undefined') {
  window.addEventListener('click', closeContextMenu)
}

const selectMergeFiles = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]

  for (const path of paths) {
    const name = path.split(/[/\\]/).pop() || ''
    let size = 0
    try {
      const info = await invoke<{ size: number }>('get_file_info', { filePath: path })
      size = info.size
    } catch { /* ignore */ }

    let thumb: string | undefined
    try {
      const thumbBase64 = await invoke<string>('get_thumbnail', { filePath: path })
      thumb = 'data:image/jpeg;base64,' + thumbBase64
    } catch { /* ignore */ }

    const id = `img_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const imgData: CanvasImage = {
      id,
      path,
      name,
      size,
      thumb,
      left: 100 + canvasImages.value.length * 20,
      top: 100 + canvasImages.value.length * 20,
      scaleX: 1,
      scaleY: 1,
      angle: 0,
    }

    canvasImages.value.push(imgData)

    // 添加到 Fabric 画布
    addImageToCanvas(imgData)
  }
  error.value = ''
}

const removeCanvasImage = (index: number) => {
  const imgData = canvasImages.value[index]
  if (!imgData) return

  // 从 Fabric 画布移除
  if (fabricCanvas.value) {
    const objects = fabricCanvas.value.getObjects()
    const fabricObj = objects.find(obj => (obj as any).customId === imgData.id)
    if (fabricObj) {
      fabricCanvas.value.remove(fabricObj)
      fabricCanvas.value.renderAll()
    }
  }

  // 从状态移除
  canvasImages.value.splice(index, 1)
}

const clearCanvasImages = () => {
  if (fabricCanvas.value) {
    fabricCanvas.value.clear()
    fabricCanvas.value.backgroundColor = 'transparent'
    drawCheckerboard(fabricCanvas.value)
    fabricCanvas.value.renderAll()
  }
  canvasImages.value = []
  mergeResult.value = null
  error.value = ''
}

const bringToFront = () => {
  if (!fabricCanvas.value || !contextMenuTargetId.value) return
  const obj = fabricCanvas.value.getObjects().find(o => (o as any).customId === contextMenuTargetId.value)
  if (obj) {
    fabricCanvas.value.bringObjectToFront(obj)
    fabricCanvas.value.renderAll()
  }
  contextMenuVisible.value = false
}

const sendToBack = () => {
  if (!fabricCanvas.value || !contextMenuTargetId.value) return
  const obj = fabricCanvas.value.getObjects().find(o => (o as any).customId === contextMenuTargetId.value)
  if (obj) {
    fabricCanvas.value.sendObjectToBack(obj)
    fabricCanvas.value.renderAll()
  }
  contextMenuVisible.value = false
}

const handleCanvasMerge = async () => {
  if (canvasImages.value.length === 0) return
  error.value = ''
  mergeLoading.value = true

  try {
    const images = canvasImages.value.map(img => ({
      file_path: img.path,
      left: img.left,
      top: img.top,
      scale_x: img.scaleX,
      scale_y: img.scaleY,
      angle: img.angle,
    }))

    const result = await invoke<{ base64: string; width: number; height: number }>('image_canvas_merge', {
      images,
      canvasWidth: canvasSizeMode.value === 'manual' ? manualCanvasWidth.value : null,
      canvasHeight: canvasSizeMode.value === 'manual' ? manualCanvasHeight.value : null,
      bgColor: mergeBgColor.value,
    })

    mergeResult.value = result
    ElMessage.success('拼图生成完成')
  } catch (e: any) {
    error.value = e
  } finally {
    mergeLoading.value = false
  }
}

const downloadMergeResult = async () => {
  if (!mergeResult.value) return
  const blob = base64ToBlob(mergeResult.value.base64)
  await saveFileWithDialog(blob, 'merged.png', 'png')
}

// ============ Tab 3: 加水印 ============
const watermarkFile = ref<{ name: string; path: string; size: number } | null>(null)
const watermarkText = ref('')
const watermarkPos = ref('bottomRight')
const watermarkFontSize = ref(32)
const watermarkOpacity = ref(0.5)
const watermarkColor = ref('#ffffff')
const watermarkResult = ref<string | null>(null)
const watermarkLoading = ref(false)

const watermarkResultUrl = computed(() =>
  watermarkResult.value ? 'data:image/png;base64,' + watermarkResult.value : ''
)

const selectWatermarkFile = async () => {
  const selected = await open({
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] }],
  })
  if (!selected) return
  const path = selected as string
  let size = 0
  try {
    const info = await invoke<{ size: number }>('get_file_info', { filePath: path })
    size = info.size
  } catch { /* ignore */ }
  watermarkFile.value = { name: path.split(/[/\\]/).pop() || '', path, size }
  watermarkResult.value = null
  error.value = ''
}

const clearWatermarkFile = () => {
  watermarkFile.value = null
  watermarkResult.value = null
  error.value = ''
}

const handleWatermark = async () => {
  if (!watermarkFile.value || !watermarkText.value) return
  error.value = ''
  watermarkLoading.value = true
  try {
    const result = await invoke<{ base64: string }>('image_watermark', {
      filePath: watermarkFile.value.path,
      text: watermarkText.value,
      position: watermarkPos.value,
      opacity: watermarkOpacity.value,
      fontSize: watermarkFontSize.value,
      color: watermarkColor.value,
    })
    watermarkResult.value = result.base64
    ElMessage.success('水印添加完成')
  } catch (e: any) {
    error.value = e
  } finally {
    watermarkLoading.value = false
  }
}

const downloadWatermarkResult = async () => {
  if (!watermarkResult.value) return
  const blob = base64ToBlob(watermarkResult.value)
  const name = watermarkFile.value?.name.replace(/\.[^.]+$/, '') || 'watermarked'
  await saveFileWithDialog(blob, `${name}_watermarked.png`, 'png')
}

// ============ Tab 4: 调色板提取 ============
const paletteFile = ref<{ name: string; path: string; size: number } | null>(null)
const paletteCount = ref(8)
const paletteColors = ref<{ hex: string; rgb: [number, number, number]; ratio: number }[]>([])
const paletteLoading = ref(false)

const selectPaletteFile = async () => {
  const selected = await open({
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!selected) return
  const path = selected as string
  let size = 0
  try {
    const info = await invoke<{ size: number }>('get_file_info', { filePath: path })
    size = info.size
  } catch { /* ignore */ }
  paletteFile.value = { name: path.split(/[/\\]/).pop() || '', path, size }
  paletteColors.value = []
  error.value = ''
}

const clearPaletteFile = () => {
  paletteFile.value = null
  paletteColors.value = []
  error.value = ''
}

const handlePalette = async () => {
  if (!paletteFile.value) return
  error.value = ''
  paletteLoading.value = true
  try {
    const result = await invoke<{ colors: { hex: string; rgb: [number, number, number]; ratio: number }[] }>('image_palette', {
      filePath: paletteFile.value.path,
      colorCount: paletteCount.value,
    })
    paletteColors.value = result.colors
    ElMessage.success('提取完成')
  } catch (e: any) {
    error.value = e
  } finally {
    paletteLoading.value = false
  }
}

const copyPaletteColor = async (hex: string) => {
  await navigator.clipboard.writeText(hex)
  ElMessage.success(`已复制 ${hex}`)
}

const copyPaletteCss = async () => {
  const css = paletteColors.value.map((c, i) => `--palette-${i + 1}: ${c.hex};`).join('\n')
  await navigator.clipboard.writeText(css)
  ElMessage.success('已复制 CSS 变量')
}
</script>

<style scoped>
/* ===== Tab 样式（参考 PdfTool） ===== */
.image-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .image-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.image-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.image-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.image-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.image-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.image-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 页面特有样式 ===== */
.upload-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 10px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}
.merge-file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.merge-file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}
.merge-thumb {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  flex-shrink: 0;
}
.file-name {
  flex: 1;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.file-size {
  color: var(--text-secondary);
  white-space: nowrap;
}

.result-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 10px;
  background: rgba(0, 212, 255, 0.05);
  border-radius: 4px;
  font-size: 13px;
}
.result-name {
  flex: 1;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.result-size {
  color: var(--text-secondary);
  white-space: nowrap;
}
.result-ratio {
  color: var(--accent-green);
  font-weight: 600;
  white-space: nowrap;
}

.preview-area {
  margin-top: 12px;
  max-width: 100%;
  overflow: auto;
}
.merge-preview {
  max-width: 100%;
  max-height: 400px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.image-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}
.info-name { color: var(--text-primary); font-weight: 500; }

.palette-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 12px;
}
.palette-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  transition: transform 0.15s;
}
.palette-item:hover {
  transform: scale(1.05);
}
.palette-swatch {
  width: 64px;
  height: 64px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}
.palette-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-top: 4px;
  font-size: 11px;
}
.palette-hex {
  color: var(--text-primary);
  font-weight: 600;
}
.palette-rgb {
  color: var(--text-secondary);
}
.palette-ratio {
  color: var(--accent-cyan);
}

.fabric-canvas {
  border: 1px solid var(--border-color);
  border-radius: 4px;
  display: block;
  margin: 0 auto;
  max-width: 100%;
}

.canvas-context-menu {
  position: fixed;
  z-index: 100;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  padding: 4px 0;
  min-width: 80px;
}
.context-menu-item {
  padding: 6px 16px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
}
.context-menu-item:hover {
  background: var(--bg-input);
  color: var(--accent-cyan);
}

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
}
</style>