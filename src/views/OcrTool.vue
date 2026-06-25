<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="ocr-tabs">
      <el-tab-pane label="文字识别" name="text">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持 PNG/JPG/WebP/BMP 格式图片</p>
                <p>可通过以下方式输入图片：</p>
                <p>• 点击「上传文件」选择本地图片</p>
                <p>• 使用 Ctrl+V 粘贴剪贴板中的图片</p>
                <p>• 拖拽图片到预览区域</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-tag v-if="isModelLoading" size="small" type="info" effect="plain">模型加载中...</el-tag>
          <el-tag v-else-if="isModelReady" size="small" type="success" effect="plain">模型就绪</el-tag>
          <el-button size="small" plain @click="handleClearResult">清除结果</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">图片输入</span>
            <div class="group-buttons">
              <el-button size="small" type="primary" @click="triggerFileInput">
                批量上传
              </el-button>
              <el-button size="small" @click="handlePaste">
                粘贴剪贴板
              </el-button>
            </div>
          </div>
          <div v-if="batchImages.length > 0" class="action-group">
            <span class="group-label">批量操作</span>
            <div class="group-buttons">
              <el-button
                size="small"
                type="success"
                :disabled="isAllRecognizing || batchImages.length === 0"
                @click="handleBatchRecognize"
              >
                识别全部 ({{ batchImages.length }}张)
              </el-button>
              <el-button size="small" @click="clearBatchImages">
                清空列表
              </el-button>
            </div>
          </div>
        </div>
        <input
          ref="fileInputRef"
          type="file"
          accept="image/png,image/jpeg,image/webp,image/bmp"
          multiple
          style="display: none"
          @change="handleFileSelect"
        />
      </div>
    </div>

    <!-- 批量图片列表 -->
    <div v-if="batchImages.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片列表 ({{ batchImages.length }}/{{ 20 }})</span>
        <div class="card-actions">
          <el-tag v-if="isAllRecognizing" size="small" type="warning">
            识别中 {{ completedCount }}/{{ batchImages.length }}
          </el-tag>
          <el-button size="small" @click="clearBatchImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="thumbnail-grid">
          <div
            v-for="img in batchImages"
            :key="img.id"
            class="thumbnail-item"
            @click="loadBatchImageResult(img)"
            @dblclick="previewBatchImage(img)"
          >
            <img :src="img.thumbnail" class="thumbnail-img" />
            <div class="thumbnail-name">{{ img.name }}</div>
            <div class="thumbnail-status">
              <el-tag v-if="img.status === 'success'" size="small" type="success">成功</el-tag>
              <el-tag v-else-if="img.status === 'recognizing'" size="small" type="warning">识别中</el-tag>
              <el-tag v-else-if="img.status === 'error'" size="small" type="danger">失败</el-tag>
              <el-tag v-else size="small" type="info">待识别</el-tag>
            </div>
            <div class="thumbnail-delete" @click.stop="removeBatchImage(img.id)">
              <el-icon><Close /></el-icon>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 图片预览卡片（单图模式） -->
    <div v-else-if="imagePreview" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片预览</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClearImage">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="image-preview-container" @dblclick="openImagePreview">
          <img 
            :src="imagePreview" 
            alt="预览图片" 
            class="preview-image"
          />
          <div class="preview-overlay">
            <span class="overlay-text">双击查看大图</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 大图预览弹窗 -->
    <el-dialog 
      v-model="showImageModal" 
      title="图片预览" 
      :width="imageModalWidth"
      :close-on-click-modal="true"
      :show-close="true"
    >
      <div class="modal-image-container">
        <img :src="originalImageUrl" alt="大图预览" class="modal-image" />
      </div>
    </el-dialog>

    <!-- 合并结果卡片（批量模式） -->
    <div v-if="isBatchMode && showMergedView && mergedResult" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">合并结果</span>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleCopyMerged">复制全部</el-button>
          <el-button size="small" @click="handleExportMerged">导出合并txt</el-button>
          <el-button size="small" @click="showMergedView = false">切换单图</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="mergedResult"
          type="textarea"
          :rows="12"
          readonly
          class="result-textarea"
        />
      </div>
    </div>

    <!-- 识别结果卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">识别结果</span>
          <el-button
            v-if="isBatchMode && !showMergedView"
            size="small"
            @click="showMergedView = true"
          >
            切换合并
          </el-button>
        </div>
        <div class="card-actions">
          <el-button size="small" :disabled="!resultText" @click="handleCopy">复制</el-button>
          <el-button size="small" :disabled="!resultText" @click="handleCleanText">清理空行</el-button>
          <el-button size="small" :disabled="!resultText" @click="handleExport">导出txt</el-button>
        </div>
      </div>
      <div class="card-body" v-loading="isRecognizing" element-loading-text="正在识别中...">
        <el-input
          v-model="resultText"
          type="textarea"
          :rows="12"
          placeholder="上传图片或粘贴剪贴板后，自动进行OCR识别..."
          readonly
          class="result-textarea"
        />
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- 识别历史卡片 -->
    <div v-if="ocrHistory.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">识别历史</span>
        <el-button size="small" @click="handleClearHistory">清空</el-button>
      </div>
      <div class="card-body">
        <div class="history-list">
          <div
            v-for="(record, idx) in ocrHistory"
            :key="idx"
            class="history-item"
            @click="handleLoadHistory(record)"
          >
            <img v-if="record.thumbnail" :src="record.thumbnail" class="history-thumb" />
            <div class="history-info">
              <div class="history-text">{{ record.text.substring(0, 80) }}{{ record.text.length > 80 ? '...' : '' }}</div>
              <div class="history-time">{{ record.time }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
      </el-tab-pane>

      <el-tab-pane label="表格识别" name="table">
        <div class="tool-container table-tab-container">
          <!-- 操作卡片 -->
          <div class="tool-card sticky-card">
            <div class="card-header">
              <div class="header-left">
                <span class="card-title">操作</span>
                <el-tooltip placement="bottom" effect="dark">
                  <template #content>
                    <div class="tooltip-content">
                      <p>识别图片中的表格，输出CSV格式</p>
                      <p>支持 PNG/JPG/WebP/BMP 格式图片</p>
                      <p>• 点击「上传文件」选择本地图片</p>
                      <p>• 使用 Ctrl+V 粘贴剪贴板中的图片</p>
                    </div>
                  </template>
                  <el-icon class="hint-icon"><QuestionFilled /></el-icon>
                </el-tooltip>
              </div>
              <div class="header-actions">
                <el-tag v-if="isTableRecognizing" size="small" type="info">识别中...</el-tag>
                <el-button size="small" @click="handleClearTable">清除结果</el-button>
              </div>
            </div>
            <div class="card-body">
              <div class="action-grid">
                <div class="action-group">
                  <span class="group-label">图片输入</span>
                  <div class="group-buttons">
                    <el-button size="small" type="primary" @click="triggerTableFileInput">
                      上传文件
                    </el-button>
                    <el-button size="small" @click="handleTablePaste">
                      粘贴剪贴板
                    </el-button>
                  </div>
                </div>
              </div>
              <input
                ref="tableFileInputRef"
                type="file"
                accept="image/png,image/jpeg,image/webp,image/bmp"
                style="display: none"
                @change="handleTableFileSelect"
              />
            </div>
          </div>

          <!-- 图片预览卡片 -->
          <div v-if="tableImagePreview" class="tool-card">
            <div class="card-header">
              <span class="card-title">图片预览</span>
              <div class="card-actions">
                <el-button size="small" @click="handleClearTableImage">移除</el-button>
              </div>
            </div>
            <div class="card-body">
              <div class="image-preview-container">
                <img :src="tableImagePreview" alt="预览图片" class="preview-image" />
              </div>
            </div>
          </div>

          <!-- 识别结果卡片 -->
          <div class="tool-card">
            <div class="card-header">
              <div class="header-left">
                <span class="card-title">识别结果</span>
              </div>
              <div class="card-actions">
                <el-button size="small" :disabled="!tableCsvText" @click="handleCopyTableCsv">复制CSV</el-button>
                <el-button size="small" :disabled="!tableCsvText" @click="handleExportTableCsv">导出CSV</el-button>
              </div>
            </div>
            <div class="card-body" v-loading="isTableRecognizing" element-loading-text="正在识别表格...">
              <el-input
                v-model="tableCsvText"
                type="textarea"
                :rows="8"
                placeholder="上传图片后，自动识别表格并输出CSV..."
                readonly
                class="result-textarea"
              />
              <div v-if="tableError" class="error-message">{{ tableError }}</div>
            </div>
          </div>

          <!-- 表格预览卡片 -->
          <div v-if="tableData.length > 0" class="tool-card">
            <div class="card-header">
              <span class="card-title">表格预览</span>
            </div>
            <div class="card-body">
              <el-table :data="tableDisplayData" border stripe size="small" class="table-preview">
                <el-table-column
                  v-for="(header, idx) in tableHeaders"
                  :key="idx"
                  :prop="'col' + idx"
                  :label="header"
                  min-width="100"
                />
              </el-table>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="Markdown转换" name="markdown">
        <div class="tool-container markdown-tab-container">
          <!-- 操作卡片 -->
          <div class="tool-card sticky-card">
            <div class="card-header">
              <div class="header-left">
                <span class="card-title">操作</span>
                <el-tooltip placement="bottom" effect="dark">
                  <template #content>
                    <div class="tooltip-content">
                      <p>识别图片文字，输出Markdown格式</p>
                      <p>支持 PNG/JPG/WebP/BMP 格式图片</p>
                      <p>• 点击「上传文件」选择本地图片</p>
                      <p>• 使用 Ctrl+V 粘贴剪贴板中的图片</p>
                      <p>• 自动推断标题层级（基于字体大小）</p>
                    </div>
                  </template>
                  <el-icon class="hint-icon"><QuestionFilled /></el-icon>
                </el-tooltip>
              </div>
              <div class="card-actions">
                <el-tag v-if="isMarkdownRecognizing" size="small" type="info">识别中...</el-tag>
                <el-button size="small" @click="handleClearMarkdown">清除结果</el-button>
              </div>
            </div>
            <div class="card-body">
              <div class="action-grid">
                <div class="action-group">
                  <span class="group-label">图片输入</span>
                  <div class="group-buttons">
                    <el-button size="small" type="primary" @click="triggerMarkdownFileInput">
                      上传文件
                    </el-button>
                    <el-button size="small" @click="handleMarkdownPaste">
                      粘贴剪贴板
                    </el-button>
                  </div>
                </div>
              </div>
              <input
                ref="markdownFileInputRef"
                type="file"
                accept="image/png,image/jpeg,image/webp,image/bmp"
                style="display: none"
                @change="handleMarkdownFileSelect"
              />
            </div>
          </div>

          <!-- 图片预览卡片 -->
          <div v-if="markdownImagePreview" class="tool-card">
            <div class="card-header">
              <span class="card-title">图片预览</span>
              <div class="card-actions">
                <el-button size="small" @click="handleClearMarkdownImage">移除</el-button>
              </div>
            </div>
            <div class="card-body">
              <div class="image-preview-container" @dblclick="openMarkdownImagePreview">
                <img :src="markdownImagePreview" alt="预览图片" class="preview-image" />
                <div class="preview-overlay">
                  <span class="overlay-text">双击查看大图</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Markdown大图预览弹窗 -->
          <el-dialog 
            v-model="showMarkdownImageModal" 
            title="图片预览" 
            :width="imageModalWidth"
            :close-on-click-modal="true"
            :show-close="true"
          >
            <div class="modal-image-container">
              <img :src="markdownOriginalImageUrl" alt="大图预览" class="modal-image" />
            </div>
          </el-dialog>

          <!-- Markdown输出卡片 -->
          <div v-if="markdownMdText" class="tool-card">
            <div class="card-header">
              <div class="header-left">
                <span class="card-title">Markdown输出</span>
              </div>
              <div class="card-actions">
                <el-button size="small" @click="handleCopyMarkdown">复制Markdown</el-button>
                <el-button size="small" @click="handleExportMarkdown">导出.md文件</el-button>
              </div>
            </div>
            <div class="card-body" v-loading="isMarkdownRecognizing" element-loading-text="正在转换Markdown...">
              <div class="markdown-output-grid">
                <div class="markdown-source-panel">
                  <div class="panel-label">源码</div>
                  <el-input
                    v-model="markdownMdText"
                    type="textarea"
                    :rows="16"
                    readonly
                    class="markdown-textarea"
                  />
                </div>
                <div class="markdown-preview-panel">
                  <div class="panel-label">预览</div>
                  <div class="markdown-preview" v-html="markdownHtmlPreview"></div>
                </div>
              </div>
              <div v-if="markdownError" class="error-message">{{ markdownError }}</div>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Close } from '@element-plus/icons-vue'
import { recognizeImage, cleanText, exportAsTxt, destroyOcr, batchRecognize, getMergedResult, type BatchImage, recognizeTable, toCsv, recognizeMarkdown, exportAsMd } from '@/utils/ocrUtils'
import { saveFileWithDialog } from '@/utils/fileSaver'
import { useToolboxStore } from '@/store'

interface OcrHistoryRecord {
  thumbnail: string
  originalUrl: string
  text: string
  time: string
}

const store = useToolboxStore()
const activeTab = ref('text')
const fileInputRef = ref<HTMLInputElement | null>(null)
const imagePreview = ref<string>('')
const originalImageUrl = ref<string>('')
const resultText = ref('')
const error = ref('')
const isModelLoading = ref(false)
const isModelReady = ref(false)
const isRecognizing = ref(false)
const ocrHistory = reactive<OcrHistoryRecord[]>([])
const showImageModal = ref(false)
const imageModalWidth = ref('80%')

// 批量图片状态
const batchImages = ref<BatchImage[]>([])
const isBatchMode = computed(() => batchImages.value.length > 1)
const showMergedView = ref(true)
const mergedResult = computed(() => getMergedResult(batchImages.value))
const completedCount = computed(() =>
  batchImages.value.filter(i => i.status === 'success' || i.status === 'error').length
)
const isAllRecognizing = computed(() =>
  batchImages.value.some(i => i.status === 'recognizing')
)

// 表格识别状态
const tableFileInputRef = ref<HTMLInputElement | null>(null)
const tableImagePreview = ref<string>('')
const tableCsvText = ref('')
const tableError = ref('')
const isTableRecognizing = ref(false)
const tableData = ref<string[][]>([])

// 表格显示数据
const tableHeaders = computed(() => {
  if (tableData.value.length === 0) return []
  const colCount = tableData.value[0].length
  return Array.from({ length: colCount }, (_, i) => `列${i + 1}`)
})

const tableDisplayData = computed(() => {
  return tableData.value.map((row, idx) => {
    const obj: Record<string, string> = { id: idx.toString() }
    row.forEach((cell, colIdx) => {
      obj['col' + colIdx] = cell
    })
    return obj
  })
})

// Markdown转换状态
const markdownFileInputRef = ref<HTMLInputElement | null>(null)
const markdownImagePreview = ref<string>('')
const markdownOriginalImageUrl = ref<string>('')
const markdownMdText = ref('')
const markdownError = ref('')
const isMarkdownRecognizing = ref(false)
const showMarkdownImageModal = ref(false)

// Markdown HTML预览（简单转换）
const markdownHtmlPreview = computed(() => {
  if (!markdownMdText.value) return ''
  return markdownMdText.value
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^#### (.+)$/gm, '<h4>$1</h4>')
    .replace(/\n\n/g, '</p><p>')
    .replace(/\n/g, '<br>')
    .replace(/^(?!<[h1-6])/gm, '<p>')
    .replace(/(?<!<\/[h1-6]>)$/gm, '</p>')
    .replace(/<p><\/p>/g, '')
})

// 加载历史记录
const loadHistory = () => {
  try {
    const saved = localStorage.getItem('ocr_history')
    if (saved) {
      const parsed = JSON.parse(saved)
      ocrHistory.length = 0
      ocrHistory.push(...parsed.slice(0, 10))
    }
  } catch {
    // ignore
  }
}

// 保存历史记录
const saveHistory = (thumbnail: string, originalUrl: string, text: string) => {
  const record: OcrHistoryRecord = {
    thumbnail,
    originalUrl,
    text,
    time: new Date().toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  }
  ocrHistory.unshift(record)
  if (ocrHistory.length > 10) {
    ocrHistory.pop()
  }
  localStorage.setItem('ocr_history', JSON.stringify(ocrHistory))
}

// 触发文件选择
const triggerFileInput = () => {
  fileInputRef.value?.click()
}

// 处理文件选择（支持批量）
const handleFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files || files.length === 0) return

  // 限制最多20张
  const maxFiles = 20
  const selectedFiles = Array.from(files).slice(0, maxFiles)
  if (files.length > maxFiles) {
    ElMessage.warning(`最多支持${maxFiles}张图片，已选择前${maxFiles}张`)
  }

  // 清空之前的批量状态
  batchImages.value = []
  imagePreview.value = ''
  resultText.value = ''
  error.value = ''

  // 处理每张图片
  for (const file of selectedFiles) {
    const thumbnail = await generateThumbnail(file)
    const originalUrl = URL.createObjectURL(file)
    batchImages.value.push({
      id: Date.now().toString() + Math.random().toString(36).substring(2, 9),
      file,
      thumbnail,
      originalUrl,
      name: file.name || `image-${batchImages.value.length + 1}.png`,
      status: 'pending'
    })
  }

  // 如果是单张，直接开始识别
  if (selectedFiles.length === 1) {
    await processImage(selectedFiles[0])
  }

  input.value = ''
}

// 处理粘贴
const handlePaste = async () => {
  try {
    const items = await navigator.clipboard.read()
    for (const item of items) {
      for (const type of item.types) {
        if (type.startsWith('image/')) {
          const blob = await item.getType(type)
          await processImage(blob)
          return
        }
      }
    }
    ElMessage.warning('剪贴板中没有图片')
  } catch {
    ElMessage.warning('无法读取剪贴板，请确保已复制图片')
  }
}

// 全局粘贴监听
const handleGlobalPaste = (e: ClipboardEvent) => {
  const items = e.clipboardData?.items
  if (!items) return
  for (let i = 0; i < items.length; i++) {
    if (items[i].type.startsWith('image/')) {
      const blob = items[i].getAsFile()
      if (blob) {
        e.preventDefault()
        processImage(blob)
      }
      break
    }
  }
}

// 处理图片
const processImage = async (blob: Blob | File) => {
  error.value = ''

  // 保存原始图片URL
  originalImageUrl.value = URL.createObjectURL(blob)
  // 生成预览
  imagePreview.value = originalImageUrl.value

  // 生成缩略图
  const thumbnail = await generateThumbnail(blob)

  // 懒加载模型
  if (!isModelReady.value) {
    isModelLoading.value = true
    try {
      await recognizeImage(blob)
      isModelReady.value = true
    } catch (e: any) {
      error.value = e.message || '模型加载失败'
      isModelLoading.value = false
      return
    }
    isModelLoading.value = false
  }

  // 执行识别
  isRecognizing.value = true
  try {
    const text = await recognizeImage(blob)
    resultText.value = text

    // 保存历史记录
    saveHistory(thumbnail, originalImageUrl.value, text)

    // 添加到全局历史
    store.addHistory({
      tool: 'ocr',
      action: '文字识别',
      inputPreview: '[图片]',
      outputPreview: text.substring(0, 100)
    })

    ElMessage.success('识别完成')
  } catch (e: any) {
    error.value = e.message || '识别失败'
  } finally {
    isRecognizing.value = false
  }
}

// 批量识别
const handleBatchRecognize = async () => {
  if (batchImages.value.length === 0) return

  error.value = ''

  // 确保模型已加载
  if (!isModelReady.value) {
    isModelLoading.value = true
    try {
      await recognizeImage(batchImages.value[0].file)
      isModelReady.value = true
    } catch (e: any) {
      error.value = e.message || '模型加载失败'
      isModelLoading.value = false
      return
    }
    isModelLoading.value = false
  }

  // 并行识别所有图片
  try {
    await batchRecognize(batchImages.value, (_completed, _total) => {
      // 进度更新，Vue会自动响应
    })

    // 更新全局历史（只记录一次）
    const successImages = batchImages.value.filter(i => i.status === 'success')
    if (successImages.length > 0) {
      store.addHistory({
        tool: 'ocr',
        action: `批量识别(${successImages.length}张)`,
        inputPreview: `[${successImages.length}张图片]`,
        outputPreview: successImages[0].result?.substring(0, 100) || ''
      })
    }

    ElMessage.success(`批量识别完成，成功${successImages.length}张`)
  } catch (e: any) {
    error.value = e.message || '批量识别失败'
  }
}

// 从批量列表中移除图片
const removeBatchImage = (id: string) => {
  batchImages.value = batchImages.value.filter(img => img.id !== id)
  if (batchImages.value.length === 0) {
    imagePreview.value = ''
    resultText.value = ''
  }
}

// 清空批量列表
const clearBatchImages = () => {
  batchImages.value = []
  imagePreview.value = ''
  resultText.value = ''
  error.value = ''
}

// 加载单张批量图片的结果到主视图
const loadBatchImageResult = (image: BatchImage) => {
  if (image.result) {
    resultText.value = image.result
    imagePreview.value = image.thumbnail
    originalImageUrl.value = image.originalUrl
  }
}

// 预览批量图片大图
const previewBatchImage = (image: BatchImage) => {
  originalImageUrl.value = image.originalUrl
  showImageModal.value = true
}

// 复制合并结果
const handleCopyMerged = async () => {
  try {
    await navigator.clipboard.writeText(mergedResult.value)
    ElMessage.success('已复制全部结果')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 导出合并结果
const handleExportMerged = async () => {
  await exportAsTxt(mergedResult.value, 'ocr-batch-result.txt')
}

// 生成缩略图
const generateThumbnail = (blob: Blob | File): Promise<string> => {
  return new Promise((resolve) => {
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      const size = 60
      canvas.width = size
      canvas.height = size
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        resolve('')
        return
      }
      const ratio = Math.min(size / img.width, size / img.height)
      const w = img.width * ratio
      const h = img.height * ratio
      ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h)
      resolve(canvas.toDataURL('image/jpeg', 0.6))
      URL.revokeObjectURL(img.src)
    }
    img.src = URL.createObjectURL(blob)
  })
}

// 打开图片预览弹窗
const openImagePreview = () => {
  showImageModal.value = true
}

// 清除图片
const handleClearImage = () => {
  imagePreview.value = ''
  showImageModal.value = false
  if (fileInputRef.value) fileInputRef.value.value = ''
}

// 清除结果
const handleClearResult = () => {
  resultText.value = ''
  error.value = ''
}

// 复制
const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(resultText.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 清理空行
const handleCleanText = () => {
  resultText.value = cleanText(resultText.value)
  ElMessage.success('已清理空行')
}

// 导出
const handleExport = async () => {
  await exportAsTxt(resultText.value)
}

// 加载历史记录
const handleLoadHistory = (record: OcrHistoryRecord) => {
  resultText.value = record.text
  imagePreview.value = record.thumbnail
  originalImageUrl.value = record.originalUrl
}

// 清空历史
const handleClearHistory = () => {
  ocrHistory.length = 0
  localStorage.removeItem('ocr_history')
  ElMessage.success('识别历史已清空')
}

// 表格识别相关函数
const triggerTableFileInput = () => {
  tableFileInputRef.value?.click()
}

const handleTableFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await processTableImage(file)
  input.value = ''
}

const handleTablePaste = async () => {
  try {
    const items = await navigator.clipboard.read()
    for (const item of items) {
      for (const type of item.types) {
        if (type.startsWith('image/')) {
          const blob = await item.getType(type)
          await processTableImage(blob)
          return
        }
      }
    }
    ElMessage.warning('剪贴板中没有图片')
  } catch {
    ElMessage.warning('无法读取剪贴板，请确保已复制图片')
  }
}

const processTableImage = async (blob: Blob | File) => {
  tableError.value = ''
  tableImagePreview.value = URL.createObjectURL(blob)
  isTableRecognizing.value = true

  try {
    const table = await recognizeTable(blob)
    tableData.value = table
    tableCsvText.value = toCsv(table)

    // 添加到全局历史
    store.addHistory({
      tool: 'ocr',
      action: '表格识别',
      inputPreview: '[表格图片]',
      outputPreview: tableCsvText.value.substring(0, 100)
    })

    ElMessage.success(`表格识别完成，${table.length}行${table[0]?.length || 0}列`)
  } catch (e: any) {
    tableError.value = e.message || '表格识别失败'
  } finally {
    isTableRecognizing.value = false
  }
}

const handleClearTable = () => {
  tableCsvText.value = ''
  tableError.value = ''
  tableData.value = []
}

const handleClearTableImage = () => {
  tableImagePreview.value = ''
  if (tableFileInputRef.value) tableFileInputRef.value.value = ''
}

const handleCopyTableCsv = async () => {
  try {
    await navigator.clipboard.writeText(tableCsvText.value)
    ElMessage.success('已复制CSV')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportTableCsv = async () => {
  const blob = new Blob([tableCsvText.value], { type: 'text/csv;charset=utf-8' })
  await saveFileWithDialog(blob, 'table-result.csv', 'csv')
}

// Markdown转换相关函数
const triggerMarkdownFileInput = () => {
  markdownFileInputRef.value?.click()
}

const handleMarkdownFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await processMarkdownImage(file)
  input.value = ''
}

const handleMarkdownPaste = async () => {
  try {
    const items = await navigator.clipboard.read()
    for (const item of items) {
      for (const type of item.types) {
        if (type.startsWith('image/')) {
          const blob = await item.getType(type)
          await processMarkdownImage(blob)
          return
        }
      }
    }
    ElMessage.warning('剪贴板中没有图片')
  } catch {
    ElMessage.warning('无法读取剪贴板，请确保已复制图片')
  }
}

const processMarkdownImage = async (blob: Blob | File) => {
  markdownError.value = ''
  markdownOriginalImageUrl.value = URL.createObjectURL(blob)
  markdownImagePreview.value = markdownOriginalImageUrl.value
  isMarkdownRecognizing.value = true

  try {
    const mdText = await recognizeMarkdown(blob)
    markdownMdText.value = mdText

    // 添加到全局历史
    store.addHistory({
      tool: 'ocr',
      action: 'Markdown转换',
      inputPreview: '[Markdown图片]',
      outputPreview: mdText.substring(0, 100)
    })

    ElMessage.success('Markdown转换完成')
  } catch (e: any) {
    markdownError.value = e.message || 'Markdown转换失败'
  } finally {
    isMarkdownRecognizing.value = false
  }
}

const handleClearMarkdown = () => {
  markdownMdText.value = ''
  markdownError.value = ''
}

const handleClearMarkdownImage = () => {
  markdownImagePreview.value = ''
  markdownOriginalImageUrl.value = ''
  showMarkdownImageModal.value = false
  if (markdownFileInputRef.value) markdownFileInputRef.value.value = ''
}

const openMarkdownImagePreview = () => {
  showMarkdownImageModal.value = true
}

const handleCopyMarkdown = async () => {
  try {
    await navigator.clipboard.writeText(markdownMdText.value)
    ElMessage.success('已复制Markdown')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportMarkdown = async () => {
  await exportAsMd(markdownMdText.value, 'markdown-result.md')
}

onMounted(() => {
  loadHistory()
  document.addEventListener('paste', handleGlobalPaste)
})

onUnmounted(() => {
  document.removeEventListener('paste', handleGlobalPaste)
  destroyOcr()
})
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

/* Tab样式 */
.ocr-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.ocr-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

/* 表格识别Tab容器 */
.table-tab-container {
  height: auto;
  overflow: visible;
  padding: 0;
}

/* 工具卡片 */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

/* 置顶卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  margin-bottom: 16px;
}

/* 标题栏 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 8px; }
.card-actions .el-tag {
  height: 28px;
  line-height: 26px;
  padding: 0 10px;
  font-size: 12px;
}
.card-actions .el-button {
  height: 28px;
  padding: 0 12px;
  font-size: 12px;
}
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

/* 提示图标 */
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

/* 操作按钮 */
.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

/* 图片预览 */
.image-preview-container {
  display: flex;
  justify-content: center;
  align-items: center;
  max-height: 300px;
  overflow: hidden;
  border-radius: 4px;
  background: var(--bg-input);
  position: relative;
  cursor: pointer;
}
.preview-image {
  max-width: 100%;
  max-height: 300px;
  object-fit: contain;
}
.preview-image.clickable:hover {
  opacity: 0.9;
}
.preview-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 8px 12px;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.7), transparent);
  opacity: 0;
  transition: opacity 0.3s;
  pointer-events: none;
}
.image-preview-container:hover .preview-overlay {
  opacity: 1;
}
.overlay-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.8);
}

/* 大图预览弹窗 */
.modal-image-container {
  display: flex;
  justify-content: center;
  align-items: center;
  max-height: 80vh;
  overflow: auto;
}
.modal-image {
  max-width: 100%;
  max-height: 80vh;
  object-fit: contain;
  border-radius: 4px;
}

/* 结果文本 */
.result-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* 错误提示 */
.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

/* 识别历史 */
.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.history-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: border-color 0.3s;
}
.history-item:hover { border-color: var(--accent-cyan); }
.history-thumb {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  object-fit: cover;
  flex-shrink: 0;
}
.history-info { flex: 1; min-width: 0; }
.history-text {
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.history-time {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* 缩略图网格 */
.thumbnail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 12px;
}

.thumbnail-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s;
}
.thumbnail-item:hover {
  border-color: var(--accent-cyan);
}
.thumbnail-item:hover::after {
  content: '双击查看大图';
  position: absolute;
  top: 4px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 10px;
  color: var(--accent-cyan);
  white-space: nowrap;
  pointer-events: none;
}

.thumbnail-img {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
}

.thumbnail-name {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 6px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.thumbnail-status {
  margin-top: 4px;
}

.thumbnail-delete {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 50%;
  cursor: pointer;
  color: #fff;
  font-size: 12px;
  opacity: 0;
  transition: opacity 0.2s;
}
.thumbnail-item:hover .thumbnail-delete {
  opacity: 1;
}

/* 表格预览 */
.table-preview {
  font-size: 13px;
}
:deep(.el-table th) {
  background: var(--bg-input);
  color: var(--accent-cyan);
  font-weight: 600;
}
:deep(.el-table td) {
  color: var(--text-primary);
}
:deep(.el-table--border) {
  border-color: var(--border-color);
}
:deep(.el-table) {
  background: var(--bg-card);
  color: var(--text-primary);
}
:deep(.el-table tr) {
  background: var(--bg-card);
}
:deep(.el-table td) {
  background: var(--bg-card);
  color: var(--text-primary);
}
:deep(.el-table--striped .el-table__body tr.el-table__row--striped td) {
  background: var(--bg-input);
  color: var(--text-primary);
}
:deep(.el-table__body tr:hover > td) {
  background: rgba(0, 212, 255, 0.15) !important;
  color: var(--text-primary) !important;
}
:deep(.el-table__body tr:hover > td .cell) {
  color: var(--text-primary) !important;
}

/* Markdown转换Tab容器 */
.markdown-tab-container {
  height: auto;
  overflow: visible;
  padding: 0;
}

/* Markdown输出网格 */
.markdown-output-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.markdown-source-panel,
.markdown-preview-panel {
  display: flex;
  flex-direction: column;
}

.panel-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.markdown-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
}

/* Markdown预览 */
.markdown-preview {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 16px;
  min-height: 300px;
  max-height: 500px;
  overflow-y: auto;
  line-height: 1.6;
  color: var(--text-primary);
}

.markdown-preview h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--accent-cyan);
  margin: 16px 0 8px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.markdown-preview h2 {
  font-size: 20px;
  font-weight: 600;
  color: var(--accent-cyan);
  margin: 14px 0 6px 0;
}

.markdown-preview h3 {
  font-size: 17px;
  font-weight: 600;
  color: var(--accent-cyan);
  margin: 12px 0 4px 0;
}

.markdown-preview h4 {
  font-size: 15px;
  font-weight: 600;
  color: var(--accent-cyan);
  margin: 10px 0 4px 0;
}

.markdown-preview p {
  margin: 8px 0;
}

.markdown-preview br {
  line-height: 1.6;
}
</style>
