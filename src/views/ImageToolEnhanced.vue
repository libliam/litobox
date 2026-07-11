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

    <!-- Tab 2: 模板拼图 -->
    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectMergeFiles">选择图片</el-button>
          <el-button v-if="mergeImages.length" size="small" @click="clearMergeImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="mergeImages.length" class="merge-file-list">
          <div v-for="(f, i) in mergeImages" :key="i" class="merge-file-item">
            <img :src="f.thumb" class="merge-thumb" />
            <span class="file-name">{{ f.name }}</span>
            <span class="file-size">{{ formatBytes(f.size) }}</span>
            <el-button size="small" text type="danger" @click="removeMergeImage(i)">移除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">选择图片开始拼图</div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && availableTemplates.length" class="tool-card">
      <div class="card-header"><span class="card-title">拼图模板</span></div>
      <div class="card-body">
        <div class="template-grid">
          <div
            v-for="tpl in availableTemplates"
            :key="tpl.id"
            class="template-item"
            :class="{ 'template-active': currentTemplate?.id === tpl.id }"
            @click="selectTemplate(tpl)"
          >
            <div class="template-preview" :style="templatePreviewStyle(tpl)">
              <div
                v-for="(slot, si) in tpl.grid"
                :key="si"
                class="template-slot"
                :style="slotStyle(slot)"
              ></div>
            </div>
            <span class="template-name">{{ tpl.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && currentTemplate" class="tool-card">
      <div class="card-header"><span class="card-title">拼图预览</span></div>
      <div class="card-body">
        <div class="merge-preview-grid" :style="mergeGridStyle">
          <div
            v-for="(slot, si) in currentTemplate.grid"
            :key="si"
            class="merge-slot"
            :class="{
              'merge-slot-selected': selectedSlot === si,
              'merge-slot-empty': slotMap[si] === null,
            }"
            :style="slotStyle(slot)"
            @click="onSlotClick(si)"
            @mousedown="onSlotMouseDown($event, si)"
            @mousemove="onSlotMouseMove($event, si)"
            @mouseup="onSlotMouseUp"
            @mouseleave="onSlotMouseUp"
          >
            <img
              v-if="slotMap[si] !== null && mergeImages[slotMap[si]!]"
              :src="mergeImages[slotMap[si]!].thumb"
              class="merge-slot-img"
              :style="{ objectPosition: slotImgPosition(si) }"
            />
            <span v-else class="merge-slot-placeholder">点击交换</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && currentTemplate" class="tool-card">
      <div class="card-header"><span class="card-title">输出设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">背景色</span>
            <el-color-picker v-model="mergeBgColor" size="small" show-alpha />
            <el-button size="small" @click="mergeBgColor = ''" style="margin-left: 8px">透明</el-button>
          </div>
          <div class="action-group">
            <span class="group-label">间距</span>
            <el-input-number v-model="mergeGap" :min="0" :max="100" size="small" controls-position="right" style="width: 90px" />
            <span style="font-size: 12px; color: var(--text-secondary)">px</span>
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="!hasFilledSlots" :loading="mergeLoading" @click="handleTemplateMerge">生成拼图</el-button>
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
import { ref, computed } from 'vue'
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

// ============ Tab 2: 模板拼图 ============

// 模板定义
interface TemplateSlot {
  colStart: number
  colEnd: number
  rowStart: number
  rowEnd: number
}

interface Template {
  id: string
  name: string
  count: number
  grid: TemplateSlot[]
}

const TEMPLATES: Template[] = [
  { id: 'h2', name: '左右2列', count: 2, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'v2', name: '上下2行', count: 2, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l2', name: '左大右小', count: 2, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'h3', name: '三等分', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 't3', name: '上1下2', count: 3, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l3', name: '左1右2', count: 3, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'g4', name: '四宫格', count: 4, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l4', name: '左大右3', count: 4, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 3, rowEnd: 4 },
  ]},
  { id: 'h5', name: '五宫格', count: 5, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'g6', name: '六宫格', count: 6, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
]

interface MergeImage {
  path: string
  name: string
  size: number
  thumb: string
}

const mergeImages = ref<MergeImage[]>([])
const slotMap = ref<(number | null)[]>([])
const currentTemplate = ref<Template | null>(null)
const mergeBgColor = ref('#ffffff')
const mergeGap = ref(4)
const mergeResult = ref<{ base64: string; width: number; height: number } | null>(null)
const mergeLoading = ref(false)

// 槽位内图片的平移偏移（像素），用于调整裁剪区域
interface SlotOffset { x: number; y: number }
const slotOffsets = ref<SlotOffset[]>([])

// 点击交换状态
const selectedSlot = ref<number | null>(null)

// 槽位内平移状态
const panState = ref<{ slotIndex: number; startX: number; startY: number; moved: boolean } | null>(null)
const PAN_THRESHOLD = 5 // 超过此像素视为拖拽而非点击

const mergeResultUrl = computed(() =>
  mergeResult.value ? 'data:image/png;base64,' + mergeResult.value.base64 : ''
)

// 根据图片数量过滤可用模板（优先显示图片数完全匹配的）
const availableTemplates = computed(() => {
  const n = mergeImages.value.length
  const exact = TEMPLATES.filter(t => t.count === n)
  if (exact.length > 0) return exact
  return TEMPLATES.filter(t => t.count <= n)
})

// 至少有 1 个槽位有图片
const hasFilledSlots = computed(() =>
  slotMap.value.some(s => s !== null)
)

// 模板预览图（缩略版 CSS Grid）
const templatePreviewStyle = (tpl: Template) => {
  const cols = Math.max(...tpl.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...tpl.grid.map(s => s.rowEnd)) - 1
  return {
    gridTemplateColumns: `repeat(${cols}, 1fr)`,
    gridTemplateRows: `repeat(${rows}, 1fr)`,
  }
}

// 拼图预览 Grid（固定 1200x800 比例）
const mergeGridStyle = computed(() => {
  if (!currentTemplate.value) return {}
  const cols = Math.max(...currentTemplate.value.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...currentTemplate.value.grid.map(s => s.rowEnd)) - 1
  return {
    gridTemplateColumns: `repeat(${cols}, 1fr)`,
    gridTemplateRows: `repeat(${rows}, 1fr)`,
    aspectRatio: '1200 / 800',
    gap: mergeGap.value + 'px',
  }
})

const slotStyle = (slot: TemplateSlot) => ({
  gridColumn: `${slot.colStart} / ${slot.colEnd}`,
  gridRow: `${slot.rowStart} / ${slot.rowEnd}`,
})

// 选择模板
const selectTemplate = (tpl: Template) => {
  currentTemplate.value = tpl
  const newSlots: (number | null)[] = new Array(tpl.grid.length).fill(null)

  // 如果已有槽位映射，保留它；否则按图片顺序填充
  const hasExisting = slotMap.value.some(s => s !== null)
  if (hasExisting) {
    for (let i = 0; i < Math.min(tpl.grid.length, slotMap.value.length); i++) {
      newSlots[i] = slotMap.value[i]
    }
  } else {
    for (let i = 0; i < Math.min(tpl.grid.length, mergeImages.value.length); i++) {
      newSlots[i] = i
    }
  }
  slotMap.value = newSlots
}

// 选择图片
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

    let thumb = ''
    try {
      // 使用原图而非缩略图，避免预览模糊
      const fullBase64 = await invoke<string>('read_file_base64', { filePath: path })
      thumb = 'data:image/png;base64,' + fullBase64
    } catch { /* ignore */ }

    mergeImages.value.push({ path, name, size, thumb })
  }

  // 自动选择首个匹配模板
  if (!currentTemplate.value && availableTemplates.value.length > 0) {
    selectTemplate(availableTemplates.value[0])
  } else if (currentTemplate.value) {
    // 已有模板：刷新 slotMap（selectTemplate 会自动按图片顺序填充）
    selectTemplate(currentTemplate.value)
  }

  error.value = ''
}

// 删除图片
const removeMergeImage = (index: number) => {
  mergeImages.value.splice(index, 1)
  // 从 slotMap 中移除，并重新映射
  slotMap.value = slotMap.value.map(s => {
    if (s === null) return null
    if (s === index) return null
    return s > index ? s - 1 : s
  })
  // 检查模板是否仍可用
  if (currentTemplate.value && currentTemplate.value.count > mergeImages.value.length) {
    currentTemplate.value = null
    slotMap.value = []
  }
}

// 清空
const clearMergeImages = () => {
  mergeImages.value = []
  slotMap.value = []
  currentTemplate.value = null
  mergeResult.value = null
  error.value = ''
}

// 拖拽状态（已废弃，改用点击交换）
const selectedSlot = ref<number | null>(null)

// 点击槽位：选中 → 交换（如果发生了拖拽则不交换）
const onSlotClick = (slotIndex: number) => {
  if (panState.value?.moved) {
    panState.value.moved = false
    return
  }
  if (selectedSlot.value === null) {
    selectedSlot.value = slotIndex
  } else if (selectedSlot.value === slotIndex) {
    selectedSlot.value = null
  } else {
    const from = selectedSlot.value
    const temp = slotMap.value[from]
    slotMap.value[from] = slotMap.value[slotIndex]
    slotMap.value[slotIndex] = temp
    // 交换偏移
    const offFrom = slotOffsets.value[from] || { x: 0, y: 0 }
    const offTo = slotOffsets.value[slotIndex] || { x: 0, y: 0 }
    slotOffsets.value[from] = offTo
    slotOffsets.value[slotIndex] = offFrom
    selectedSlot.value = null
  }
}

// 平移：鼠标按下
const onSlotMouseDown = (e: MouseEvent, slotIndex: number) => {
  if (slotMap.value[slotIndex] === null) return
  panState.value = { slotIndex, startX: e.clientX, startY: e.clientY, moved: false }
}

// 平移：鼠标移动
const onSlotMouseMove = (e: MouseEvent, slotIndex: number) => {
  if (!panState.value || panState.value.slotIndex !== slotIndex) return
  const dx = e.clientX - panState.value.startX
  const dy = e.clientY - panState.value.startY
  if (Math.abs(dx) > PAN_THRESHOLD || Math.abs(dy) > PAN_THRESHOLD) {
    panState.value.moved = true
  }
  if (panState.value.moved) {
    const off = slotOffsets.value[slotIndex] || { x: 0, y: 0 }
    off.x = (off.x || 0) + dx
    off.y = (off.y || 0) + dy
    slotOffsets.value[slotIndex] = off
    panState.value.startX = e.clientX
    panState.value.startY = e.clientY
  }
}

// 平移：鼠标松开
const onSlotMouseUp = () => {
  panState.value = null
}

// 图片在槽位内的 object-position（控制裁剪区域）
const slotImgPosition = (si: number) => {
  const off = slotOffsets.value[si]
  if (!off || (!off.x && !off.y)) return 'center'
  // 使用像素值：50% 居中 + 偏移
  return `calc(50% + ${off.x}px) calc(50% + ${off.y}px)`
}

// 拖拽开始（已废弃，保留引用避免 TS 报错）
const onDragStart = (slotIndex: number) => {
  selectedSlot.value = slotIndex
}

// 拖拽结束
const onDragEnd = () => {
  dragFromSlot.value = null
  dragOverSlot.value = null
}

// 拖拽经过
const onDragOver = (slotIndex: number) => {
  dragOverSlot.value = slotIndex
}

// 拖拽离开
const onDragLeave = () => {
  dragOverSlot.value = null
}

// 放置
const onDrop = (targetSlot: number) => {
  if (dragFromSlot.value === null) return
  const from = dragFromSlot.value
  // 交换两个槽位的图片
  const temp = slotMap.value[from]
  slotMap.value[from] = slotMap.value[targetSlot]
  slotMap.value[targetSlot] = temp
  dragFromSlot.value = null
  dragOverSlot.value = null
}

// 生成拼图
const handleTemplateMerge = async () => {
  if (!currentTemplate.value || !hasFilledSlots.value) return
  error.value = ''
  mergeLoading.value = true

  try {
    const cols = Math.max(...currentTemplate.value.grid.map(s => s.colEnd)) - 1
    const rows = Math.max(...currentTemplate.value.grid.map(s => s.rowEnd)) - 1
    const canvasWidth = 1200
    const canvasHeight = 800
    const gap = mergeGap.value

    const slotWidth = (canvasWidth - gap * (cols - 1)) / cols
    const slotHeight = (canvasHeight - gap * (rows - 1)) / rows

    const images: { file_path: string; x: number; y: number; width: number; height: number }[] = []

    currentTemplate.value.grid.forEach((slot, si) => {
      const imgIdx = slotMap.value[si]
      if (imgIdx === null) return
      const img = mergeImages.value[imgIdx]
      if (!img) return

      const col = slot.colStart - 1
      const row = slot.rowStart - 1
      const colSpan = slot.colEnd - slot.colStart
      const rowSpan = slot.rowEnd - slot.rowStart

      images.push({
        file_path: img.path,
        x: Math.round(col * (slotWidth + gap)),
        y: Math.round(row * (slotHeight + gap)),
        width: Math.round(slotWidth * colSpan + gap * (colSpan - 1)),
        height: Math.round(slotHeight * rowSpan + gap * (rowSpan - 1)),
      })
    })

    const result = await invoke<{ base64: string; width: number; height: number }>('image_template_merge', {
      images,
      canvasWidth,
      canvasHeight,
      bgColor: mergeBgColor.value,
      gap,
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

/* 模板选择器 */
.template-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.template-item {
  cursor: pointer;
  padding: 6px;
  border: 2px solid transparent;
  border-radius: 6px;
  transition: border-color 0.2s;
  text-align: center;
}
.template-item:hover {
  border-color: var(--color-accent);
}
.template-active {
  border-color: var(--color-accent);
  background: rgba(0, 255, 255, 0.05);
}
.template-preview {
  display: grid;
  width: 80px;
  height: 60px;
  gap: 2px;
  margin-bottom: 4px;
}
.template-slot {
  background: var(--border-color);
  border-radius: 2px;
}
.template-name {
  font-size: 11px;
  color: var(--text-secondary);
}

/* 拼图预览 */
.merge-preview-grid {
  display: grid;
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
  background: var(--bg-input);
  border-radius: 4px;
  overflow: hidden;
}
.merge-slot {
  position: relative;
  background: var(--border-color);
  border: 2px dashed transparent;
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  min-height: 60px;
  transition: border-color 0.2s;
  cursor: pointer;
  user-select: none;
}
.merge-slot-selected {
  border-color: var(--color-accent);
  box-shadow: 0 0 8px var(--color-accent);
}
.merge-slot-empty {
  border-style: dashed;
}
.merge-slot-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
}
.merge-slot-placeholder {
  font-size: 12px;
  color: var(--text-secondary);
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