<template>
  <div class="tool-container">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="pdf-tabs">
        <el-tab-pane label="PDF转图片" name="pdfToImages" />
        <el-tab-pane label="图片转PDF" name="imagesToPdf" />
        <el-tab-pane label="PDF文本提取" name="textExtract" />
        <el-tab-pane label="PDF合并/拆分" name="mergeSplit" />
      </el-tabs>
    </div>

    <!-- Tab 1: PDF转图片 -->
    <div v-if="activeTab === 'pdfToImages'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">PDF 输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>上传 PDF 文件，逐页导出为 PNG 图片</p>
                <p>DPI 越高，图片越清晰但速度越慢</p>
                <p>72 DPI: 快速预览 | 150 DPI: 标准 | 300 DPI: 高清</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerPdfInput">上传 PDF</el-button>
          <el-button v-if="pdfFile" size="small" @click="handleClearPdf">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="pdfInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handlePdfFileSelect"
        />
        <div v-if="pdfFile" class="file-info">
          <span class="file-name">{{ pdfFile.name }}</span>
          <span class="file-size">{{ formatFileSize(pdfFile.size) }}</span>
          <span v-if="pdfPageCount" class="file-pages">{{ pdfPageCount }} 页</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <div v-if="activeTab === 'pdfToImages'" class="tool-card">
      <div class="card-header">
        <span class="card-title">转换设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">DPI</div>
            <el-select v-model="dpi" size="small" style="width: 120px">
              <el-option label="72 (快速)" :value="72" />
              <el-option label="150 (标准)" :value="150" />
              <el-option label="300 (高清)" :value="300" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :disabled="!pdfFile" @click="handlePdfToImages">
                开始转换
              </el-button>
              <el-button size="small" :disabled="!imageBlobs.length" @click="handleDownloadAllImages">
                全部下载
              </el-button>
              <el-button
                size="small"
                type="success"
                :disabled="!imageBlobs.length || isOcrRunning"
                :loading="isOcrRunning"
                @click="handleOcrAll"
              >
                OCR 识别
              </el-button>
              <el-button
                size="small"
                :disabled="!imageBlobs.length"
                @click="handleJumpToOcr"
              >
                跳转到OCR
              </el-button>
            </div>
          </div>
        </div>
        <div v-if="imageBlobs.length" class="result-info">
          <span>共 {{ imageBlobs.length }} 页</span>
          <span>总大小: {{ formatFileSize(imageBlobs.reduce((sum, b) => sum + b.size, 0)) }}</span>
        </div>
        <div v-if="imageBlobs.length" class="image-preview-grid">
          <div v-for="(blob, idx) in imageBlobs" :key="idx" class="image-preview-item">
            <img :src="getImageUrl(blob)" :alt="`第 ${idx + 1} 页`" />
            <div class="image-label">第 {{ idx + 1 }} 页</div>
            <el-button size="small" @click="handleDownloadSingleImage(blob, idx + 1)">下载</el-button>
          </div>
        </div>

        <!-- OCR 结果 -->
        <div v-if="ocrResults.length > 0" class="ocr-result-section">
          <el-divider />
          <div class="ocr-result-header">
            <span class="ocr-result-title">OCR 识别结果 ({{ ocrResults.length }} 页)</span>
            <div class="ocr-actions">
              <el-button size="small" @click="handleCopyOcrResult">复制全部</el-button>
              <el-button size="small" @click="handleExportOcrResult">导出 .txt</el-button>
              <el-button size="small" @click="handleClearOcrResult">清除</el-button>
            </div>
          </div>
          <el-input
            :model-value="ocrFullText"
            type="textarea"
            :rows="10"
            readonly
            class="ocr-textarea"
          />
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 2: 图片转PDF -->
    <div v-if="activeTab === 'imagesToPdf'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerImageInput">添加图片</el-button>
          <el-button v-if="imageFiles.length" size="small" @click="handleClearImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="imageInputRef"
          type="file"
          accept="image/png,image/jpeg"
          multiple
          style="display: none"
          @change="handleImageFileSelect"
        />
        <div v-if="imageFiles.length" class="image-list">
          <div v-for="(file, idx) in imageFiles" :key="idx" class="image-list-item">
            <span class="image-list-index">{{ idx + 1 }}</span>
            <span class="image-list-name">{{ file.name }}</span>
            <span class="image-list-size">{{ formatFileSize(file.size) }}</span>
            <el-button size="small" type="danger" link @click="handleRemoveImage(idx)">删除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">点击「添加图片」选择图片（支持多选）</div>
      </div>
    </div>

    <div v-if="activeTab === 'imagesToPdf'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">页面尺寸</div>
            <el-select v-model="imageToPdfOptions.pageSize" size="small" style="width: 120px">
              <el-option label="跟随图片" value="auto" />
              <el-option label="A4" value="a4" />
              <el-option label="A3" value="a3" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">方向</div>
            <el-select v-model="imageToPdfOptions.orientation" size="small" style="width: 120px">
              <el-option label="自动" value="auto" />
              <el-option label="纵向" value="portrait" />
              <el-option label="横向" value="landscape" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :disabled="!imageFiles.length" @click="handleImagesToPdf">
                生成 PDF
              </el-button>
            </div>
          </div>
        </div>
        <div v-if="generatedPdfBlob" class="result-info">
          <span>PDF 大小: {{ formatFileSize(generatedPdfBlob.size) }}</span>
          <el-button size="small" @click="handleDownloadGeneratedPdf">下载 PDF</el-button>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 3: PDF文本提取 -->
    <div v-if="activeTab === 'textExtract'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerExtractInput">上传 PDF</el-button>
          <el-button v-if="extractPdfFile" size="small" @click="handleClearExtractPdf">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="extractInputRef"
          type="file"
          accept=".pdf"
          style="display: none"
          @change="handleExtractPdfSelect"
        />
        <div v-if="extractPdfFile" class="file-info">
          <span class="file-name">{{ extractPdfFile.name }}</span>
          <span class="file-size">{{ formatFileSize(extractPdfFile.size) }}</span>
        </div>
        <div v-else class="upload-hint">点击「上传 PDF」选择文件</div>
      </div>
    </div>

    <div v-if="activeTab === 'textExtract'" class="tool-card">
      <div class="card-header">
        <span class="card-title">提取结果</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!extractedText" @click="handleCopyExtractedText">复制</el-button>
          <el-button size="small" :disabled="!extractedText" @click="handleDownloadExtractedText">下载 .txt</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" size="small" :disabled="!extractPdfFile" @click="handleExtractText">
              提取文本
            </el-button>
          </div>
        </div>
        <div v-if="extractedText" class="text-info">
          <span>字符数: {{ extractedText.length }}</span>
        </div>
        <el-input
          v-model="extractedText"
          type="textarea"
          :rows="12"
          placeholder="提取的文本将在此处显示..."
          readonly
        />
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 4: PDF合并/拆分 -->
    <div v-if="activeTab === 'mergeSplit'" class="tool-card">
      <div class="card-header">
        <span class="card-title">PDF 文件列表</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerMergeInput">添加 PDF</el-button>
          <el-button v-if="mergePdfFiles.length" size="small" @click="handleClearMergePdfs">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="mergeInputRef"
          type="file"
          accept=".pdf"
          multiple
          style="display: none"
          @change="handleMergePdfSelect"
        />
        <div v-if="mergePdfFiles.length" class="merge-file-list">
          <div v-for="(file, idx) in mergePdfFiles" :key="idx" class="merge-file-item">
            <span class="merge-file-index">{{ idx + 1 }}</span>
            <span class="merge-file-name">{{ file.name }}</span>
            <span class="merge-file-size">{{ formatFileSize(file.size) }}</span>
            <el-input
              v-model="mergePageRanges[idx]"
              size="small"
              placeholder="页码范围 (如: 1-3,5)"
              style="width: 160px"
            />
            <el-button size="small" type="danger" link @click="handleRemoveMergePdf(idx)">删除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">点击「添加 PDF」选择文件（支持多选）</div>
      </div>
    </div>

    <div v-if="activeTab === 'mergeSplit'" class="tool-card">
      <div class="card-header">
        <span class="card-title">快捷操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">快捷</div>
            <div class="group-buttons">
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('all')">
                合并所有
              </el-button>
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('odd')">
                提取奇数页
              </el-button>
              <el-button size="small" :disabled="!mergePdfFiles.length" @click="handleQuickMerge('even')">
                提取偶数页
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">自定义页码</div>
            <el-input
              v-model="customPageRange"
              size="small"
              placeholder="如: 1-3,5,8-10"
              style="width: 200px"
              clearable
            />
            <el-button type="primary" size="small" :disabled="!mergePdfFiles.length || !customPageRange" @click="handleCustomMerge">
              生成 PDF
            </el-button>
          </div>
        </div>
        <div class="page-range-hint">
          支持格式: <code>1-3,5,8-10</code>（连续范围用 <code>-</code>，多个用 <code>,</code> 分隔）
          <span v-if="mergePdfFiles.length === 1 && singleFilePageCount > 0">
            · 当前文件共 {{ singleFilePageCount }} 页
          </span>
        </div>
        <div v-if="mergedPdfBlob" class="result-info">
          <span>PDF 大小: {{ formatFileSize(mergedPdfBlob.size) }}</span>
          <el-button size="small" @click="handleDownloadMergedPdf">下载 PDF</el-button>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { ElMessage, ElLoading } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  pdfToImages,
  extractPdfText,
  imagesToPdf,
  mergePdf,
  loadPdfDocument,
  saveFileWithDialog,
  formatFileSize,
  type ImageToPdfOptions
} from '@/utils/pdfUtils'
import { recognizeImage } from '@/utils/ocrUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const activeTab = ref('pdfToImages')
const error = ref('')

// ============ OCR 识别 ============
const ocrResults = ref<string[]>([])
const isOcrRunning = ref(false)
const ocrFullText = computed(() =>
  ocrResults.value.map((text, idx) => `--- 第 ${idx + 1} 页 ---\n${text}`).join('\n\n')
)

// ============ Tab 1: PDF转图片 ============
const pdfInputRef = ref<HTMLInputElement | null>(null)
const pdfFile = ref<File | null>(null)
const pdfPageCount = ref(0)
const dpi = ref(150)
const imageBlobs = ref<Blob[]>([])

const triggerPdfInput = () => pdfInputRef.value?.click()

const handlePdfFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await loadPdfFile(file)
  input.value = ''
}

const loadPdfFile = async (file: File) => {
  error.value = ''
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    error.value = 'PDF 过大，建议小于 100MB'
    return
  }
  pdfFile.value = file
  imageBlobs.value = []

  try {
    const buffer = await file.arrayBuffer()
    const doc = await loadPdfDocument(new Uint8Array(buffer))
    pdfPageCount.value = doc.numPages
  } catch (e: any) {
    error.value = e.message || 'PDF 加载失败'
  }
}

const handleClearPdf = () => {
  pdfFile.value = null
  pdfPageCount.value = 0
  imageBlobs.value = []
  error.value = ''
  if (pdfInputRef.value) pdfInputRef.value.value = ''
}

const handlePdfToImages = async () => {
  if (!pdfFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在转换 PDF（${pdfPageCount.value} 页），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    imageBlobs.value = await pdfToImages(pdfFile.value, dpi.value)
    ElMessage.success(`转换完成，共 ${imageBlobs.value.length} 页`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF转图片 (${dpi.value}DPI)`,
      inputPreview: pdfFile.value.name.slice(0, 50),
      outputPreview: `${imageBlobs.value.length} 页`,
      inputFull: pdfFile.value.name,
      outputFull: `${imageBlobs.value.length} 页`,
    })
  } catch (e: any) {
    error.value = e.message || '转换失败'
  } finally {
    loading.close()
  }
}

const getImageUrl = (blob: Blob) => URL.createObjectURL(blob)

const handleDownloadSingleImage = async (blob: Blob, pageNum: number) => {
  await saveFileWithDialog(blob, `page_${pageNum}.png`, 'png')
}

const handleDownloadAllImages = async () => {
  for (let idx = 0; idx < imageBlobs.value.length; idx++) {
    const blob = imageBlobs.value[idx]
    await saveFileWithDialog(blob, `page_${idx + 1}.png`, 'png')
  }
}

// ============ OCR 识别 ============
const handleOcrAll = async () => {
  if (imageBlobs.value.length === 0) return
  error.value = ''
  ocrResults.value = []
  isOcrRunning.value = true

  const loading = ElLoading.service({
    lock: true,
    text: `正在 OCR 识别 ${imageBlobs.value.length} 页...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    for (let idx = 0; idx < imageBlobs.value.length; idx++) {
      const blob = imageBlobs.value[idx]
      const blobFile = new File([blob], `page_${idx + 1}.png`, { type: 'image/png' })
      const text = await recognizeImage(blobFile)
      ocrResults.value.push(text)
    }
    ElMessage.success(`OCR 识别完成，共 ${ocrResults.value.length} 页`)
    store.addHistory({
      tool: 'pdf',
      action: 'PDF转图片+OCR',
      inputPreview: pdfFile.value?.name.slice(0, 50) || '',
      outputPreview: ocrFullText.value.slice(0, 50),
      inputFull: pdfFile.value?.name || '',
      outputFull: ocrFullText.value,
    })
  } catch (e: any) {
    error.value = `OCR 识别失败: ${e.message}`
  } finally {
    isOcrRunning.value = false
    loading.close()
  }
}

const handleCopyOcrResult = async () => {
  if (!ocrFullText.value) return
  try {
    await navigator.clipboard.writeText(ocrFullText.value)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportOcrResult = async () => {
  if (!ocrFullText.value) return
  const blob = new Blob([ocrFullText.value], { type: 'text/plain' })
  await saveFileWithDialog(blob, 'pdf-ocr-result.txt', 'txt')
}

const handleClearOcrResult = () => {
  ocrResults.value = []
}

const handleJumpToOcr = () => {
  if (imageBlobs.value.length === 0) return
  ;(window as any).__pendingOcrBlobs = imageBlobs.value.slice()
  store.activeTool = 'ocr'
}

// ============ Tab 2: 图片转PDF ============
const imageInputRef = ref<HTMLInputElement | null>(null)
const imageFiles = ref<File[]>([])
const imageToPdfOptions = reactive<ImageToPdfOptions>({
  pageSize: 'auto',
  orientation: 'auto',
  quality: 0.92
})
const generatedPdfBlob = ref<Blob | null>(null)

const triggerImageInput = () => imageInputRef.value?.click()

const handleImageFileSelect = (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  imageFiles.value = [...imageFiles.value, ...Array.from(files)]
  generatedPdfBlob.value = null
  input.value = ''
}

const handleRemoveImage = (idx: number) => {
  imageFiles.value.splice(idx, 1)
  generatedPdfBlob.value = null
}

const handleClearImages = () => {
  imageFiles.value = []
  generatedPdfBlob.value = null
  error.value = ''
  if (imageInputRef.value) imageInputRef.value.value = ''
}

const handleImagesToPdf = async () => {
  if (!imageFiles.value.length) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在生成 PDF（${imageFiles.value.length} 张图片），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    generatedPdfBlob.value = await imagesToPdf(imageFiles.value, imageToPdfOptions)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: '图片转PDF',
      inputPreview: `${imageFiles.value.length} 张图片`,
      outputPreview: formatFileSize(generatedPdfBlob.value.size),
      inputFull: imageFiles.value.map(f => f.name).join('\n'),
      outputFull: formatFileSize(generatedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '生成失败'
  } finally {
    loading.close()
  }
}

const handleDownloadGeneratedPdf = async () => {
  if (!generatedPdfBlob.value) return
  await saveFileWithDialog(generatedPdfBlob.value, 'output.pdf', 'pdf')
}

// ============ Tab 3: PDF文本提取 ============
const extractInputRef = ref<HTMLInputElement | null>(null)
const extractPdfFile = ref<File | null>(null)
const extractedText = ref('')

const triggerExtractInput = () => extractInputRef.value?.click()

const handleExtractPdfSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  error.value = ''
  const maxSize = 100 * 1024 * 1024
  if (file.size > maxSize) {
    error.value = 'PDF 过大，建议小于 100MB'
    return
  }
  extractPdfFile.value = file
  extractedText.value = ''
  input.value = ''
}

const handleClearExtractPdf = () => {
  extractPdfFile.value = null
  extractedText.value = ''
  error.value = ''
  if (extractInputRef.value) extractInputRef.value.value = ''
}

const handleExtractText = async () => {
  if (!extractPdfFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: '正在提取 PDF 文本，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    extractedText.value = await extractPdfText(extractPdfFile.value)
    ElMessage.success('文本提取完成')
    store.addHistory({
      tool: 'pdf',
      action: 'PDF文本提取',
      inputPreview: extractPdfFile.value.name.slice(0, 50),
      outputPreview: extractedText.value.slice(0, 50),
      inputFull: extractPdfFile.value.name,
      outputFull: extractedText.value,
    })
  } catch (e: any) {
    error.value = e.message || '提取失败'
  } finally {
    loading.close()
  }
}

const handleCopyExtractedText = async () => {
  if (!extractedText.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(extractedText.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleDownloadExtractedText = async () => {
  if (!extractedText.value) return
  const blob = new Blob([extractedText.value], { type: 'text/plain' })
  await saveFileWithDialog(blob, 'extracted-text.txt', 'txt')
}

// ============ Tab 4: PDF合并/拆分 ============
const mergeInputRef = ref<HTMLInputElement | null>(null)
const mergePdfFiles = ref<File[]>([])
const mergePageRanges = ref<string[]>([])
const customPageRange = ref('')
const mergedPdfBlob = ref<Blob | null>(null)
const singleFilePageCount = ref(0)

const triggerMergeInput = () => mergeInputRef.value?.click()

const handleMergePdfSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  const newFiles = Array.from(files)
  mergePdfFiles.value = [...mergePdfFiles.value, ...newFiles]
  mergePageRanges.value = [...mergePageRanges.value, ...newFiles.map(() => 'all')]
  mergedPdfBlob.value = null
  input.value = ''

  // 单文件时检测总页数，方便用户参考
  if (mergePdfFiles.value.length === 1) {
    try {
      const buffer = await mergePdfFiles.value[0].arrayBuffer()
      const doc = await loadPdfDocument(new Uint8Array(buffer))
      singleFilePageCount.value = doc.numPages
    } catch {
      singleFilePageCount.value = 0
    }
  } else {
    singleFilePageCount.value = 0
  }
}

const handleRemoveMergePdf = async (idx: number) => {
  mergePdfFiles.value.splice(idx, 1)
  mergePageRanges.value.splice(idx, 1)
  mergedPdfBlob.value = null

  // 删除后如果只剩1个文件，重新检测页数
  if (mergePdfFiles.value.length === 1) {
    try {
      const buffer = await mergePdfFiles.value[0].arrayBuffer()
      const doc = await loadPdfDocument(new Uint8Array(buffer))
      singleFilePageCount.value = doc.numPages
    } catch {
      singleFilePageCount.value = 0
    }
  } else {
    singleFilePageCount.value = 0
  }
}

const handleClearMergePdfs = () => {
  mergePdfFiles.value = []
  mergePageRanges.value = []
  customPageRange.value = ''
  mergedPdfBlob.value = null
  singleFilePageCount.value = 0
  error.value = ''
  if (mergeInputRef.value) mergeInputRef.value.value = ''
}

const handleQuickMerge = async (range: string) => {
  if (!mergePdfFiles.value.length) return
  error.value = ''
  customPageRange.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在合并 PDF（${mergePdfFiles.value.length} 个文件），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, mergePdfFiles.value.map(() => range))
    ElMessage.success('PDF 合并完成')
    store.addHistory({
      tool: 'pdf',
      action: `PDF合并 (${range})`,
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size),
      inputFull: mergePdfFiles.value.map(f => f.name).join('\n'),
      outputFull: formatFileSize(mergedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  } finally {
    loading.close()
  }
}

const handleCustomMerge = async () => {
  if (!mergePdfFiles.value.length || !customPageRange.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: `正在生成 PDF（${mergePdfFiles.value.length} 个文件），请稍候...`,
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    // 自定义页码对所有文件统一应用
    const ranges = mergePdfFiles.value.map(() => customPageRange.value)
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, ranges)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: `PDF自定义提取 [${customPageRange.value}]`,
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size),
      inputFull: mergePdfFiles.value.map((f) => `${f.name} [${customPageRange.value}]`).join('\n'),
      outputFull: formatFileSize(mergedPdfBlob.value.size),
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  } finally {
    loading.close()
  }
}

const handleDownloadMergedPdf = async () => {
  if (!mergedPdfBlob.value) return
  await saveFileWithDialog(mergedPdfBlob.value, 'merged.pdf', 'pdf')
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.pdf-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .pdf-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.pdf-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.pdf-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.pdf-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.pdf-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.pdf-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 工具卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

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
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

/* 文件信息 */
.file-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}

.file-name {
  color: var(--text-primary);
  font-weight: 500;
}

.upload-hint {
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
  padding: 20px;
}

/* 结果信息 */
.result-info {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

/* 图片预览网格 */
.image-preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.image-preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px;
  background: var(--bg-input);
  border-radius: 6px;
}

.image-preview-item img {
  width: 100%;
  height: auto;
  border-radius: 4px;
  max-height: 200px;
  object-fit: contain;
}

.image-label {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 图片列表 */
.image-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.image-list-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.image-list-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.image-list-name {
  flex: 1;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-list-size {
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 页码范围提示 */
.page-range-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

.page-range-hint code {
  background: var(--bg-input);
  padding: 1px 5px;
  border-radius: 3px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

/* 合并文件列表 */
.merge-file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.merge-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.merge-file-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.merge-file-name {
  flex: 1;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.merge-file-size {
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 文本信息 */
.text-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
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

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}

/* ===== OCR 结果 ===== */
.ocr-result-section {
  margin-top: 8px;
}

.ocr-result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.ocr-result-title {
  font-size: 13px;
  color: var(--accent-cyan);
  font-weight: 500;
}

.ocr-actions {
  display: flex;
  gap: 6px;
}

.ocr-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
</style>
