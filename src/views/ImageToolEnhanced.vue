<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="image-tabs">
        <el-tab-pane label="批量压缩/转换" name="compress" />
        <el-tab-pane label="图片拼接" name="merge" />
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

    <!-- Tab 2: 图片拼接 -->
    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectMergeFiles">选择图片</el-button>
          <el-button v-if="mergeFiles.length" size="small" @click="clearMergeFiles">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="mergeFiles.length" class="merge-file-list">
          <div v-for="(f, i) in mergeFiles" :key="i" class="merge-file-item">
            <img v-if="f.thumb" :src="f.thumb" class="merge-thumb" />
            <span class="file-name">{{ f.name }}</span>
            <span class="file-size">{{ formatBytes(f.size) }}</span>
            <el-button size="small" text :disabled="i === 0" @click="moveMergeFile(i, -1)">↑</el-button>
            <el-button size="small" text :disabled="i === mergeFiles.length - 1" @click="moveMergeFile(i, 1)">↓</el-button>
            <el-button size="small" text type="danger" @click="removeMergeFile(i)">移除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">选择多张图片进行拼接</div>
      </div>
    </div>
    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header"><span class="card-title">拼接设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">方向</span>
            <div class="group-buttons">
              <el-button size="small" :type="mergeDirection === 'vertical' ? 'primary' : ''" @click="mergeDirection = 'vertical'">纵向</el-button>
              <el-button size="small" :type="mergeDirection === 'horizontal' ? 'primary' : ''" @click="mergeDirection = 'horizontal'">横向</el-button>
            </div>
          </div>
          <div class="action-group">
            <span class="group-label">对齐</span>
            <div class="group-buttons">
              <el-button size="small" :type="mergeAlign === 'start' ? 'primary' : ''" @click="mergeAlign = 'start'">居左/上</el-button>
              <el-button size="small" :type="mergeAlign === 'center' ? 'primary' : ''" @click="mergeAlign = 'center'">居中</el-button>
              <el-button size="small" :type="mergeAlign === 'end' ? 'primary' : ''" @click="mergeAlign = 'end'">居右/下</el-button>
            </div>
          </div>
          <div class="action-group">
            <span class="group-label">间距 (px)</span>
            <el-input-number v-model="mergeGap" :min="0" :max="200" size="small" style="width: 90px" />
          </div>
          <div class="action-group">
            <span class="group-label">背景色</span>
            <el-color-picker v-model="mergeBgColor" size="small" />
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="mergeFiles.length < 2" :loading="mergeLoading" @click="handleMerge">开始拼接</el-button>
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

// ============ Tab 2: 图片拼接 ============
interface MergeFile {
  name: string
  path: string
  size: number
  thumb?: string
}

const mergeFiles = ref<MergeFile[]>([])
const mergeDirection = ref('vertical')
const mergeAlign = ref('start')
const mergeGap = ref(0)
const mergeBgColor = ref('#000000')
const mergeResult = ref<{ base64: string; width: number; height: number } | null>(null)
const mergeLoading = ref(false)

const mergeResultUrl = computed(() =>
  mergeResult.value ? 'data:image/png;base64,' + mergeResult.value.base64 : ''
)

const selectMergeFiles = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  mergeFiles.value = paths.map(p => ({
    name: p.split(/[/\\]/).pop() || '',
    path: p,
    size: 0,
  }))
  // 获取文件大小和缩略图
  for (const f of mergeFiles.value) {
    try {
      const info = await invoke<{ size: number }>('get_file_info', { filePath: f.path })
      f.size = info.size
    } catch { /* ignore */ }
    try {
      const thumbBase64 = await invoke<string>('get_thumbnail', { filePath: f.path })
      f.thumb = 'data:image/jpeg;base64,' + thumbBase64
    } catch { /* ignore */ }
  }
  error.value = ''
}

const clearMergeFiles = () => {
  mergeFiles.value = []
  mergeResult.value = null
  error.value = ''
}

const removeMergeFile = (i: number) => {
  mergeFiles.value.splice(i, 1)
  mergeResult.value = null
}

const moveMergeFile = (i: number, dir: number) => {
  const j = i + dir
  if (j < 0 || j >= mergeFiles.value.length) return
  const tmp = mergeFiles.value[i]
  mergeFiles.value[i] = mergeFiles.value[j]
  mergeFiles.value[j] = tmp
  mergeResult.value = null
}

const handleMerge = async () => {
  if (mergeFiles.value.length < 2) return
  error.value = ''
  mergeLoading.value = true
  try {
    mergeResult.value = await invoke('image_merge', {
      filePaths: mergeFiles.value.map(f => f.path),
      direction: mergeDirection.value,
      gap: mergeGap.value,
      bgColor: mergeBgColor.value,
      alignment: mergeAlign.value,
    })
    ElMessage.success('拼接完成')
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