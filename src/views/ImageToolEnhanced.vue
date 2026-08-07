<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="image-tabs">
        <el-tab-pane label="批量压缩/转换" name="compress" />
        <el-tab-pane label="尺寸缩放" name="resize" />
        <el-tab-pane label="图片转Base64" name="base64" />
        <el-tab-pane label="加水印" name="watermark" />
        <el-tab-pane label="图片拼图" name="merge" />
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
        <div class="merge-canvas-container">
          <canvas
          ref="mergeCanvas"
          class="merge-canvas"
          :width="canvasWidth"
          :height="canvasHeight"
          :style="mergeCanvasStyle"
          @click="onCanvasClick"
          @mousedown="onCanvasMouseDown"
          @mousemove="onCanvasMouseMove"
          @mouseup="onCanvasMouseUp"
          @mouseleave="onCanvasMouseUp"
          @wheel="onCanvasWheel"
        />
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && currentTemplate" class="tool-card">
      <div class="card-header"><span class="card-title">输出设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">画布尺寸</span>
            <el-input-number v-model="canvasWidth" :min="100" :max="8192" size="small" controls-position="right" style="width: 100px" placeholder="宽" />
            <span>×</span>
            <el-input-number v-model="canvasHeight" :min="100" :max="8192" size="small" controls-position="right" style="width: 100px" placeholder="高" />
            <span style="font-size: 12px; color: var(--text-secondary)">px</span>
          </div>
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
          <el-button size="small" type="primary" :disabled="!hasFilledSlots" @click="downloadMergeResult">下载结果</el-button>
          <el-button size="small" @click="resetAllSlots">重置所有</el-button>
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

    <!-- 单图上传（尺寸缩放/Base64 共用） -->
    <div v-if="activeTab === 'resize' || activeTab === 'base64'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerSingleFileInput">上传文件</el-button>
          <el-button v-if="singleImageFile" size="small" @click="handleClearSingleImage">移除</el-button>
        </div>
      </div>
      <div
        class="card-body upload-area"
        :class="{ 'drag-over': isSingleDragging }"
        @dragover="handleSingleDragOver"
        @dragleave="handleSingleDragLeave"
        @drop="handleSingleDrop"
      >
        <input
          ref="singleFileInputRef"
          type="file"
          accept="image/png,image/jpeg,image/webp,image/bmp"
          style="display: none"
          @change="handleSingleFileSelect"
        />
        <div v-if="singleImageFile" class="image-info">
          <span class="info-name">{{ singleImageFile.name }}</span>
          <span class="info-size">{{ formatBytes(singleImageInfo?.size || 0) }}</span>
          <span class="info-dimensions">{{ singleImageInfo?.width }}×{{ singleImageInfo?.height }}</span>
        </div>
        <div v-else class="upload-hint">
          点击「上传文件」或拖拽图片到此处
        </div>
      </div>
    </div>

    <!-- Tab 2: 尺寸缩放 -->
    <div v-if="activeTab === 'resize'" class="tool-card">
      <div class="card-header">
        <span class="card-title">尺寸缩放</span>
      </div>
      <div class="card-body">
        <div class="resize-controls">
          <div class="input-row">
            <label>宽度 (px)</label>
            <input
              type="number"
              v-model.number="resizeWidth"
              :disabled="lockAspect && !resizeTarget"
              class="resize-input"
              @input="handleWidthChange"
            />
          </div>
          <div class="input-row">
            <label>高度 (px)</label>
            <input
              type="number"
              v-model.number="resizeHeight"
              :disabled="lockAspect && !resizeTarget"
              class="resize-input"
              @input="handleHeightChange"
            />
          </div>
          <div class="input-row">
            <label>百分比 (%)</label>
            <input
              type="number"
              v-model.number="resizePercent"
              class="resize-input"
              @input="handlePercentChange"
            />
          </div>
          <div class="lock-row">
            <el-checkbox v-model="lockAspect">等比例缩放</el-checkbox>
          </div>
        </div>
        <div class="action-grid">
          <div class="action-group">
            <el-button size="small" type="primary" :disabled="!singleImageFile" @click="handleResize">
              开始缩放
            </el-button>
            <el-button size="small" :disabled="!resizedBlob" @click="handleDownloadResized">
              下载缩放图
            </el-button>
          </div>
        </div>
        <div v-if="resizedBlob" class="result-info">
          <span>原尺寸: {{ resizeOrigWidth }}×{{ resizeOrigHeight }}</span>
          <span class="arrow">→</span>
          <span>新尺寸: {{ resizeWidth }}×{{ resizeHeight }}</span>
        </div>
        <div v-if="resizeError" class="error-message">{{ resizeError }}</div>
      </div>
    </div>

    <!-- Tab 3: 图片转Base64 -->
    <div v-if="activeTab === 'base64'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Base64 输出</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!base64Result" @click="handleCopyBase64">复制</el-button>
          <el-button size="small" :disabled="!base64Result" @click="handleDownloadBase64">下载.txt</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button size="small" type="primary" :disabled="!singleImageFile" @click="handleToBase64">
              生成Base64
            </el-button>
          </div>
        </div>
        <div v-if="base64Result" class="base64-info">
          <span>大小: {{ formatBytes(base64Result.length) }}</span>
        </div>
        <el-input
          v-model="base64Result"
          type="textarea"
          :rows="10"
          placeholder="生成Base64后在此显示..."
          readonly
        />
        <div v-if="base64Error" class="error-message">{{ base64Error }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { saveFileWithDialog } from '@/utils/fileSaver'
import * as imageUtils from '@/utils/imageUtils'

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
  longImage?: boolean  // 长图模式：不裁剪，按原图比例拼接
}

const TEMPLATES: Template[] = [
  // === 2 张 ===
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
  { id: 't2', name: '上大下小', count: 2, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
  ]},
  // === 3 张 ===
  { id: 'h3', name: '三等分', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'v3', name: '三行', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
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
  { id: 'b3', name: '上2下1', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 3, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'r3', name: '左2右1', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 3 },
  ]},
  // === 4 张 ===
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
  { id: 't4', name: '上1下3', count: 4, grid: [
    { colStart: 1, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'c4', name: '中间大', count: 4, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 4, rowStart: 1, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 4, rowStart: 3, rowEnd: 4 },
  ]},
  // === 5 张 ===
  { id: 'h5', name: '五宫格', count: 5, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 't5', name: '上1下4', count: 5, grid: [
    { colStart: 1, colEnd: 5, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
    { colStart: 4, colEnd: 5, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l5', name: '左大右4', count: 5, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 5 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 3, rowEnd: 4 },
    { colStart: 3, colEnd: 4, rowStart: 4, rowEnd: 5 },
  ]},
  // === 6 张 ===
  { id: 'g6', name: '六宫格', count: 6, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'v6', name: '六行', count: 6, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
    { colStart: 1, colEnd: 2, rowStart: 4, rowEnd: 5 },
    { colStart: 1, colEnd: 2, rowStart: 5, rowEnd: 6 },
    { colStart: 1, colEnd: 2, rowStart: 6, rowEnd: 7 },
  ]},
  { id: 'h6', name: '六列', count: 6, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 4, colEnd: 5, rowStart: 1, rowEnd: 2 },
    { colStart: 5, colEnd: 6, rowStart: 1, rowEnd: 2 },
    { colStart: 6, colEnd: 7, rowStart: 1, rowEnd: 2 },
  ]},
  // === 长图拼接（纵向） ===
  { id: 'long2', name: '长图2张', count: 2, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'long3', name: '长图3张', count: 3, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
  ]},
  { id: 'long4', name: '长图4张', count: 4, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
    { colStart: 1, colEnd: 2, rowStart: 4, rowEnd: 5 },
  ]},
  { id: 'long5', name: '长图5张', count: 5, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
    { colStart: 1, colEnd: 2, rowStart: 4, rowEnd: 5 },
    { colStart: 1, colEnd: 2, rowStart: 5, rowEnd: 6 },
  ]},
  { id: 'long6', name: '长图6张', count: 6, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 1, colEnd: 2, rowStart: 3, rowEnd: 4 },
    { colStart: 1, colEnd: 2, rowStart: 4, rowEnd: 5 },
    { colStart: 1, colEnd: 2, rowStart: 5, rowEnd: 6 },
    { colStart: 1, colEnd: 2, rowStart: 6, rowEnd: 7 },
  ]},
  // === 长图拼接（横向） ===
  { id: 'wide2', name: '宽图2张', count: 2, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'wide3', name: '宽图3张', count: 3, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'wide4', name: '宽图4张', count: 4, longImage: true, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 4, colEnd: 5, rowStart: 1, rowEnd: 2 },
  ]},
]

interface MergeImage {
  path: string
  name: string
  size: number
  thumb: string
  img?: HTMLImageElement  // 预加载的 Image 对象
}

const mergeImages = ref<MergeImage[]>([])
const slotMap = ref<(number | null)[]>([])
const currentTemplate = ref<Template | null>(null)
const mergeBgColor = ref('#ffffff')
const mergeGap = ref(4)

// 画布尺寸（可自定义）
const canvasWidth = ref(1200)
const canvasHeight = ref(800)

// Canvas 相关
const mergeCanvas = ref<HTMLCanvasElement | null>(null)
const mergeCanvasStyle = computed(() => ({
  width: '100%',
  maxWidth: canvasWidth.value + 'px',
  aspectRatio: `${canvasWidth.value} / ${canvasHeight.value}`,
  cursor: 'crosshair',
}))

// 槽位内图片的平移偏移（像素，基于槽位尺寸）
interface SlotOffset { x: number; y: number }
const slotOffsets = ref<SlotOffset[]>([])

// 槽位内图片的缩放比例（1.0 = 原图填满槽位）
const slotScales = ref<number[]>([])

// 点击交换状态
const selectedSlot = ref<number | null>(null)

// 槽位内平移状态
const panState = ref<{ slotIndex: number; startX: number; startY: number; moved: boolean } | null>(null)
const PAN_THRESHOLD = 5

// 根据图片数量过滤可用模板
const availableTemplates = computed(() => {
  const n = mergeImages.value.length
  const exact = TEMPLATES.filter(t => t.count === n)
  if (exact.length > 0) return exact
  return TEMPLATES.filter(t => t.count <= n)
})

const hasFilledSlots = computed(() =>
  slotMap.value.some(s => s !== null)
)

// 模板预览图样式
const templatePreviewStyle = (tpl: Template) => {
  const cols = Math.max(...tpl.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...tpl.grid.map(s => s.rowEnd)) - 1
  return {
    gridTemplateColumns: `repeat(${cols}, 1fr)`,
    gridTemplateRows: `repeat(${rows}, 1fr)`,
  }
}

// 槽位样式
const slotStyle = (slot: TemplateSlot) => ({
  gridColumn: `${slot.colStart} / ${slot.colEnd}`,
  gridRow: `${slot.rowStart} / ${slot.rowEnd}`,
})

// 计算每个槽位的像素坐标（基于 1200x800 画布）
const getSlotRect = (si: number) => {
  if (!currentTemplate.value) return null
  const tpl = currentTemplate.value
  const cols = Math.max(...tpl.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...tpl.grid.map(s => s.rowEnd)) - 1
  const gap = mergeGap.value
  const slotW = (canvasWidth.value - gap * (cols - 1)) / cols
  const slotH = (canvasHeight.value - gap * (rows - 1)) / rows
  const slot = tpl.grid[si]
  const col = slot.colStart - 1
  const row = slot.rowStart - 1
  const colSpan = slot.colEnd - slot.colStart
  const rowSpan = slot.rowEnd - slot.rowStart
  return {
    x: col * (slotW + gap),
    y: row * (slotH + gap),
    w: slotW * colSpan + gap * (colSpan - 1),
    h: slotH * rowSpan + gap * (rowSpan - 1),
  }
}

// 绘制 Canvas
const drawCanvas = () => {
  const canvas = mergeCanvas.value
  if (!canvas || !currentTemplate.value) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 设置画布尺寸（只设置一次，避免重置）
  if (canvas.width !== canvasWidth.value || canvas.height !== canvasHeight.value) {
    canvas.width = canvasWidth.value
    canvas.height = canvasHeight.value
  }

  // 背景
  const bg = mergeBgColor.value
  if (bg && bg !== 'transparent' && bg !== '') {
    ctx.fillStyle = bg
    ctx.fillRect(0, 0, canvasWidth.value, canvasHeight.value)
  } else {
    // 透明背景：绘制棋盘格
    const checkerSize = 20
    for (let y = 0; y < canvasHeight.value; y += checkerSize) {
      for (let x = 0; x < canvasWidth.value; x += checkerSize) {
        ctx.fillStyle = ((x / checkerSize + y / checkerSize) % 2 === 0) ? '#ffffff' : '#e0e0e0'
        ctx.fillRect(x, y, checkerSize, checkerSize)
      }
    }
  }

  // 绘制每个槽位的图片
  const isLongImage = currentTemplate.value.longImage
  currentTemplate.value.grid.forEach((_slot, si) => {
    const imgIdx = slotMap.value[si]
    if (imgIdx === null) return
    const mergeImg = mergeImages.value[imgIdx]
    if (!mergeImg) return

    const rect = getSlotRect(si)
    if (!rect) return

    const img = mergeImg.img
    if (!img) return

    const userScale = slotScales.value[si] || 1.0
    const offsetX = slotOffsets.value[si]?.x || 0
    const offsetY = slotOffsets.value[si]?.y || 0

    // 长图模式：contain（不裁剪，完整显示）
    // 普通模式：cover（填满槽位，裁剪溢出）
    const baseScale = isLongImage
      ? Math.min(rect.w / img.width, rect.h / img.height)
      : Math.max(rect.w / img.width, rect.h / img.height)
    const scale = baseScale * userScale
    const scaledW = img.width * scale
    const scaledH = img.height * scale

    if (isLongImage) {
      // contain 模式：居中放置，不裁剪
      const drawX = rect.x + (rect.w - scaledW) / 2 + offsetX
      const drawY = rect.y + (rect.h - scaledH) / 2 + offsetY
      ctx.drawImage(img, drawX, drawY, scaledW, scaledH)
    } else {
      // cover 模式：裁剪溢出部分
      // 偏移直接在画布坐标系中应用
      const centerX = rect.x + rect.w / 2 + offsetX
      const centerY = rect.y + rect.h / 2 + offsetY
      const drawX = centerX - scaledW / 2
      const drawY = centerY - scaledH / 2

      ctx.save()
      ctx.beginPath()
      ctx.rect(rect.x, rect.y, rect.w, rect.h)
      ctx.clip()

      ctx.drawImage(img, drawX, drawY, scaledW, scaledH)

      ctx.restore()
    }
  })
}

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
  // 确保偏移和缩放数组长度匹配
  while (slotOffsets.value.length < tpl.grid.length) slotOffsets.value.push({ x: 0, y: 0 })
  while (slotScales.value.length < tpl.grid.length) slotScales.value.push(1.0)
  nextTick(() => drawCanvas())
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
      const fullBase64 = await invoke<string>('read_file_base64', { filePath: path })
      thumb = 'data:image/png;base64,' + fullBase64
    } catch { /* ignore */ }

    // 预加载 Image 对象用于 Canvas 绘制（使用 base64 data URL）
    let img: HTMLImageElement | undefined
    try {
      img = await new Promise<HTMLImageElement>((resolve, reject) => {
        const el = new Image()
        el.onload = () => resolve(el)
        el.onerror = reject
        el.src = thumb
      })
    } catch { /* ignore */ }

    mergeImages.value.push({ path, name, size, thumb, img })
  }

  // 自动选择首个匹配模板
  if (!currentTemplate.value && availableTemplates.value.length > 0) {
    selectTemplate(availableTemplates.value[0])
  } else if (currentTemplate.value) {
    selectTemplate(currentTemplate.value)
  }

  error.value = ''
}

// 删除图片
const removeMergeImage = (index: number) => {
  mergeImages.value.splice(index, 1)
  slotMap.value = slotMap.value.map(s => {
    if (s === null) return null
    if (s === index) return null
    return s > index ? s - 1 : s
  })
  if (currentTemplate.value && currentTemplate.value.count > mergeImages.value.length) {
    currentTemplate.value = null
    slotMap.value = []
  }
  nextTick(() => drawCanvas())
}

// 清空
const clearMergeImages = () => {
  mergeImages.value = []
  slotMap.value = []
  currentTemplate.value = null
  error.value = ''
  nextTick(() => drawCanvas())
}

// 获取点击位置对应的槽位索引
const getSlotAtPoint = (clientX: number, clientY: number): number | null => {
  const canvas = mergeCanvas.value
  if (!canvas || !currentTemplate.value) return null
  const rect = canvas.getBoundingClientRect()
  const scaleX = canvasWidth.value / rect.width
  const scaleY = canvasHeight.value / rect.height
  const x = (clientX - rect.left) * scaleX
  const y = (clientY - rect.top) * scaleY

  for (let si = 0; si < currentTemplate.value.grid.length; si++) {
    const slotRect = getSlotRect(si)
    if (!slotRect) continue
    if (x >= slotRect.x && x <= slotRect.x + slotRect.w &&
        y >= slotRect.y && y <= slotRect.y + slotRect.h) {
      return si
    }
  }
  return null
}

// 点击：选中/交换
const onCanvasClick = (e: MouseEvent) => {
  const si = getSlotAtPoint(e.clientX, e.clientY)
  if (si === null) return

  if (selectedSlot.value === null) {
    selectedSlot.value = si
  } else if (selectedSlot.value === si) {
    selectedSlot.value = null
  } else {
    const from = selectedSlot.value
    const temp = slotMap.value[from]
    slotMap.value[from] = slotMap.value[si]
    slotMap.value[si] = temp
    // 交换偏移和缩放
    const offFrom = slotOffsets.value[from] || { x: 0, y: 0 }
    const offTo = slotOffsets.value[si] || { x: 0, y: 0 }
    slotOffsets.value[from] = offTo
    slotOffsets.value[si] = offFrom
    const scaleFrom = slotScales.value[from] || 1.0
    const scaleTo = slotScales.value[si] || 1.0
    slotScales.value[from] = scaleTo
    slotScales.value[si] = scaleFrom
    selectedSlot.value = null
  }
  nextTick(() => drawCanvas())
}

// 鼠标按下：按住 Alt 开始平移
const onCanvasMouseDown = (e: MouseEvent) => {
  if (!e.altKey) return
  e.preventDefault()
  const si = getSlotAtPoint(e.clientX, e.clientY)
  if (si === null || slotMap.value[si] === null) return
  panState.value = { slotIndex: si, startX: e.clientX, startY: e.clientY, moved: false }
}

// 鼠标移动：平移
const onCanvasMouseMove = (e: MouseEvent) => {
  if (panState.value) {
    const dx = e.clientX - panState.value.startX
    const dy = e.clientY - panState.value.startY
    if (!panState.value.moved && (Math.abs(dx) > PAN_THRESHOLD || Math.abs(dy) > PAN_THRESHOLD)) {
      panState.value.moved = true
    }
    if (panState.value.moved) {
      const canvas = mergeCanvas.value
      if (canvas) {
        const rect = canvas.getBoundingClientRect()
        const scaleX = canvasWidth.value / rect.width
        const scaleY = canvasHeight.value / rect.height
        const off = slotOffsets.value[panState.value.slotIndex] || { x: 0, y: 0 }
        slotOffsets.value[panState.value.slotIndex] = {
          x: (off.x || 0) + dx * scaleX,
          y: (off.y || 0) + dy * scaleY,
        }
      }
      panState.value.startX = e.clientX
      panState.value.startY = e.clientY
      drawCanvas()
    }
  }
}

// 鼠标松开
const onCanvasMouseUp = () => {
  panState.value = null
}

// 滚轮缩放（需按住 Ctrl）
const onCanvasWheel = (e: WheelEvent) => {
  if (!e.ctrlKey) return
  e.preventDefault()
  const si = getSlotAtPoint(e.clientX, e.clientY)
  if (si === null || slotMap.value[si] === null) return
  const current = slotScales.value[si] || 1.0
  const delta = e.deltaY > 0 ? -0.1 : 0.1
  slotScales.value[si] = Math.max(0.5, Math.min(5.0, current + delta))
  drawCanvas()
}

// 重置所有槽位
const resetAllSlots = () => {
  slotScales.value = slotScales.value.map(() => 1.0)
  slotOffsets.value = slotOffsets.value.map(() => ({ x: 0, y: 0 }))
  drawCanvas()
}

// 下载结果（直接导出 Canvas）
const downloadMergeResult = async () => {
  const canvas = mergeCanvas.value
  if (!canvas) return
  try {
    const dataUrl = canvas.toDataURL('image/png')
    const blob = await fetch(dataUrl).then(r => r.blob())
    await saveFileWithDialog(blob, 'merged.png', 'png')
    ElMessage.success('下载完成')
  } catch (e: any) {
    error.value = e
  }
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

// ============ 单图上传（尺寸缩放/Base64 共用） ============
const singleFileInputRef = ref<HTMLInputElement | null>(null)
const singleImageFile = ref<File | null>(null)
const singleImageInfo = ref<{ size: number; width: number; height: number } | null>(null)
const isSingleDragging = ref(false)

const triggerSingleFileInput = () => singleFileInputRef.value?.click()

const handleSingleFileSelect = async (e: Event) => {
  const target = e.target as HTMLInputElement
  if (!target.files || target.files.length === 0) return
  await setSingleImage(target.files[0])
}

const handleSingleDragOver = (e: DragEvent) => {
  e.preventDefault()
  isSingleDragging.value = true
}

const handleSingleDragLeave = () => {
  isSingleDragging.value = false
}

const handleSingleDrop = async (e: DragEvent) => {
  e.preventDefault()
  isSingleDragging.value = false
  if (e.dataTransfer?.files.length) {
    await setSingleImage(e.dataTransfer.files[0])
  }
}

const setSingleImage = async (file: File) => {
  if (!file.type.startsWith('image/')) {
    ElMessage.error('请选择图片文件')
    return
  }
  singleImageFile.value = file
  singleImageInfo.value = { size: file.size, width: 0, height: 0 }
  try {
    const info = await imageUtils.getImageInfo(file)
    singleImageInfo.value.width = info.width
    singleImageInfo.value.height = info.height
    resizeOrigWidth.value = info.width
    resizeOrigHeight.value = info.height
    resizeWidth.value = info.width
    resizeHeight.value = info.height
    resizePercent.value = 100
  } catch {
    // ignore
  }
  // 清理之前的处理结果
  resizedBlob.value = null
  base64Result.value = ''
  resizeError.value = ''
}

const handleClearSingleImage = () => {
  singleImageFile.value = null
  singleImageInfo.value = null
  resizedBlob.value = null
  base64Result.value = ''
  resizeError.value = ''
}

// ============ Tab 2: 尺寸缩放 ============
const resizeWidth = ref(800)
const resizeHeight = ref(600)
const resizePercent = ref(100)
const lockAspect = ref(true)
const resizeTarget = ref<'width' | 'height' | null>(null)
const resizedBlob = ref<Blob | null>(null)
const resizeOrigWidth = ref(0)
const resizeOrigHeight = ref(0)
const resizeError = ref('')

const handleWidthChange = () => {
  if (lockAspect.value && resizeOrigWidth.value) {
    const ratio = resizeHeight.value / resizeWidth.value
    resizeHeight.value = Math.round(resizeWidth.value * ratio)
  }
  if (resizeOrigWidth.value) {
    resizePercent.value = Math.round((resizeWidth.value / resizeOrigWidth.value) * 100)
  }
}

const handleHeightChange = () => {
  if (lockAspect.value && resizeOrigHeight.value) {
    const ratio = resizeWidth.value / resizeHeight.value
    resizeWidth.value = Math.round(resizeHeight.value * ratio)
  }
  if (resizeOrigHeight.value) {
    resizePercent.value = Math.round((resizeHeight.value / resizeOrigHeight.value) * 100)
  }
}

const handlePercentChange = () => {
  if (!resizeOrigWidth.value) return
  const w = Math.max(1, Math.round((resizePercent.value / 100) * resizeOrigWidth.value))
  const h = Math.max(1, Math.round((resizePercent.value / 100) * resizeOrigHeight.value))
  resizeWidth.value = w
  resizeHeight.value = h
}

const handleResize = async () => {
  if (!singleImageFile.value) return
  resizeError.value = ''
  try {
    const result = await imageUtils.resizeImage(singleImageFile.value, resizeWidth.value, resizeHeight.value)
    resizedBlob.value = result
    ElMessage.success('缩放完成')
  } catch (e: any) {
    resizeError.value = e
    ElMessage.error('缩放失败: ' + e)
  }
}

const handleDownloadResized = () => {
  if (!resizedBlob.value) return
  const url = URL.createObjectURL(resizedBlob.value)
  const a = document.createElement('a')
  a.href = url
  const originalName = singleImageFile.value?.name || 'image'
  const nameWithoutExt = originalName.replace(/\.[^.]+$/, '')
  a.download = `${nameWithoutExt}_${resizeWidth.value}x${resizeHeight.value}.png`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

// ============ Tab 3: 图片转Base64 ============
const base64Result = ref('')
const base64Error = ref('')

const handleToBase64 = async () => {
  if (!singleImageFile.value) return
  base64Error.value = ''
  try {
    const result = await imageUtils.imageToBase64(singleImageFile.value)
    base64Result.value = result
    ElMessage.success('生成成功')
  } catch (e: any) {
    base64Error.value = e
    ElMessage.error('生成失败: ' + e)
  }
}

const handleCopyBase64 = async () => {
  if (!base64Result.value) return
  try {
    await navigator.clipboard.writeText(base64Result.value)
    ElMessage.success('已复制到剪贴板')
  } catch (e: any) {
    ElMessage.error('复制失败: ' + e)
  }
}

const handleDownloadBase64 = () => {
  if (!base64Result.value) return
  const blob = new Blob([base64Result.value], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  const originalName = singleImageFile.value?.name || 'image'
  const nameWithoutExt = originalName.replace(/\.[^.]+$/, '')
  a.download = `${nameWithoutExt}_base64.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
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
.merge-slot-img-wrapper {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  transform-origin: center center;
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

/* ===== 尺寸缩放 & Base64 样式 ===== */
.upload-area {
  min-height: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: background 0.2s, border-color 0.2s;
}

.upload-area.drag-over {
  background: rgba(0, 212, 255, 0.1);
  border: 2px dashed var(--accent-cyan);
}

.resize-controls {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.input-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-row label {
  font-size: 12px;
  color: var(--text-secondary);
}

.resize-input {
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
}

.resize-input:focus {
  border-color: var(--accent-cyan);
}

.resize-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.lock-row {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.result-info {
  margin-top: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.result-info .arrow {
  color: var(--accent-cyan);
  font-weight: bold;
}

.base64-info {
  margin: 8px 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.info-dimensions {
  color: var(--text-secondary);
}
</style>