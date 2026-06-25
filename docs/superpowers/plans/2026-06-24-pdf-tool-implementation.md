# PDF 工具集 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 PDF 工具集页面，支持 PDF 转图片、图片转 PDF、PDF 文本提取、PDF 合并/拆分 四个功能。

**Architecture:** 纯前端实现，基于 `pdf-lib`（PDF 创建/合并/拆分）和 `pdfjs-dist`（PDF 渲染/文本提取）。采用 Tab 分栏模式，与 ImageTool 保持一致的交互结构。Worker 文件通过复制到 `public/` 目录提供。

**Tech Stack:** Vue 3 + TypeScript + Element Plus + pdf-lib + pdfjs-dist

---

### Task 1: 安装依赖并配置 Worker

**Files:**
- Modify: `d:\work\trae_use\desktop-tools\package.json`
- Create: `d:\work\trae_use\desktop-tools\public\pdf.worker.min.mjs` (copy from node_modules)

- [ ] **Step 1: 安装 pdf-lib 和 pdfjs-dist**

```bash
cd d:\work\trae_use\desktop-tools
npm install pdf-lib pdfjs-dist
```

Expected: Both packages installed successfully. pdf-lib ~1.17.x, pdfjs-dist ~4.x.

- [ ] **Step 2: 复制 pdfjs worker 文件到 public 目录**

```bash
cd d:\work\trae_use\desktop-tools
copy node_modules\pdfjs-dist\build\pdf.worker.min.mjs public\pdf.worker.min.mjs
```

Expected: File copied successfully. This is needed for pdfjs-dist worker setup.

- [ ] **Step 3: 验证依赖安装**

```bash
cd d:\work\trae_use\desktop-tools
npm ls pdf-lib pdfjs-dist
```

Expected: Both packages listed in dependency tree.

- [ ] **Step 4: 提交**

```bash
git add package.json package-lock.json public/pdf.worker.min.mjs
git commit -m "feat: add pdf-lib and pdfjs-dist dependencies for PDF tool"
```

---

### Task 2: 编写 pdfUtils.ts 核心工具函数

**Files:**
- Create: `d:\work\trae_use\desktop-tools\src\utils\pdfUtils.ts`

- [ ] **Step 1: 编写 pdfUtils.ts**

创建 `src/utils/pdfUtils.ts`，包含以下函数：

```typescript
import * as pdfjsLib from 'pdfjs-dist'
import { PDFDocument } from 'pdf-lib'

// 配置 pdfjs worker（模块加载时执行一次）
pdfjsLib.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.mjs'

// ============ 类型定义 ============

export interface PdfPageInfo {
  pageNum: number
  canvas: HTMLCanvasElement
}

export interface ImageToPdfOptions {
  pageSize: 'a4' | 'a3' | 'auto'
  orientation: 'portrait' | 'landscape' | 'auto'
  quality: number
}

// ============ PDF 转图片 ============

export async function loadPdfDocument(data: Uint8Array) {
  const loadingTask = pdfjsLib.getDocument({ data })
  return loadingTask.promise
}

export async function renderPdfPageToCanvas(
  pdfDocument: any,
  pageNum: number,
  dpi: number
): Promise<HTMLCanvasElement> {
  const page = await pdfDocument.getPage(pageNum)
  const viewport = page.getViewport({ scale: dpi / 72 })

  const canvas = document.createElement('canvas')
  canvas.width = viewport.width
  canvas.height = viewport.height

  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('Canvas context not available')

  await page.render({ canvasContext: ctx, viewport }).promise
  return canvas
}

export async function pdfToImages(
  pdfFile: File,
  dpi: number = 150
): Promise<Blob[]> {
  const buffer = await pdfFile.arrayBuffer()
  const pdfDocument = await loadPdfDocument(new Uint8Array(buffer))
  const totalPages = pdfDocument.numPages

  const blobs: Blob[] = []
  for (let i = 1; i <= totalPages; i++) {
    const canvas = await renderPdfPageToCanvas(pdfDocument, i, dpi)
    const blob = await new Promise<Blob | null>((resolve) =>
      canvas.toBlob(resolve, 'image/png')
    )
    if (!blob) throw new Error(`第 ${i} 页渲染失败`)
    blobs.push(blob)
  }

  return blobs
}

// ============ PDF 文本提取 ============

export async function extractPdfText(pdfFile: File): Promise<string> {
  const buffer = await pdfFile.arrayBuffer()
  const pdfDocument = await loadPdfDocument(new Uint8Array(buffer))
  const totalPages = pdfDocument.numPages

  let text = ''
  for (let i = 1; i <= totalPages; i++) {
    const page = await pdfDocument.getPage(i)
    const content = await page.getTextContent()
    const pageText = content.items
      .map((item: any) => item.str)
      .join('')
    text += pageText + '\n'
  }

  return text.trim()
}

// ============ 图片转 PDF ============

export async function imagesToPdf(
  imageFiles: File[],
  options: ImageToPdfOptions = { pageSize: 'auto', orientation: 'auto', quality: 0.92 }
): Promise<Blob> {
  const pdfDoc = await PDFDocument.create()

  for (const file of imageFiles) {
    const bytes = await file.arrayBuffer()
    const img = await loadImage(file)

    let pageWidth: number
    let pageHeight: number

    if (options.pageSize === 'a4') {
      pageWidth = 595.28
      pageHeight = 841.89
    } else if (options.pageSize === 'a3') {
      pageWidth = 841.89
      pageHeight = 1190.55
    } else {
      pageWidth = img.width
      pageHeight = img.height
    }

    if (options.orientation === 'landscape' ||
        (options.orientation === 'auto' && img.width > img.height)) {
      ;[pageWidth, pageHeight] = [pageHeight, pageWidth]
    }

    const page = pdfDoc.addPage([pageWidth, pageHeight])

    let embeddedImage
    if (file.type === 'image/png') {
      embeddedImage = await pdfDoc.embedPng(new Uint8Array(bytes))
    } else {
      embeddedImage = await pdfDoc.embedJpg(new Uint8Array(bytes))
    }

    const scale = Math.min(pageWidth / embeddedImage.width, pageHeight / embeddedImage.height)
    const drawWidth = embeddedImage.width * scale
    const drawHeight = embeddedImage.height * scale

    page.drawImage(embeddedImage, {
      x: (pageWidth - drawWidth) / 2,
      y: (pageHeight - drawHeight) / 2,
      width: drawWidth,
      height: drawHeight,
    })
  }

  const pdfBytes = await pdfDoc.save()
  return new Blob([pdfBytes], { type: 'application/pdf' })
}

function loadImage(file: File): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = URL.createObjectURL(file)
  })
}

// ============ PDF 合并/拆分 ============

export function parsePageRange(range: string, totalPages: number): number[] {
  const trimmed = range.trim().toLowerCase()

  if (trimmed === 'all') {
    return Array.from({ length: totalPages }, (_, i) => i + 1)
  }

  if (trimmed === 'odd') {
    return Array.from({ length: totalPages }, (_, i) => i + 1).filter(n => n % 2 === 1)
  }

  if (trimmed === 'even') {
    return Array.from({ length: totalPages }, (_, i) => i + 1).filter(n => n % 2 === 0)
  }

  const pages = new Set<number>()
  const parts = trimmed.split(',')

  for (const part of parts) {
    const p = part.trim()
    if (p.includes('-')) {
      const [start, end] = p.split('-').map(Number)
      if (isNaN(start) || isNaN(end) || start < 1 || end > totalPages || start > end) {
        throw new Error(`无效的页码范围: ${p}`)
      }
      for (let i = start; i <= end; i++) {
        pages.add(i)
      }
    } else {
      const num = Number(p)
      if (isNaN(num) || num < 1 || num > totalPages) {
        throw new Error(`无效的页码: ${p}`)
      }
      pages.add(num)
    }
  }

  return Array.from(pages).sort((a, b) => a - b)
}

export async function mergePdf(
  pdfFiles: File[],
  pageRanges: string[] = []
): Promise<Blob> {
  const mergedPdf = await PDFDocument.create()

  for (let i = 0; i < pdfFiles.length; i++) {
    const file = pdfFiles[i]
    const bytes = await file.arrayBuffer()
    const srcPdf = await PDFDocument.load(new Uint8Array(bytes))
    const totalPages = srcPdf.getPageCount()

    const range = pageRanges[i] || 'all'
    const pageNumbers = parsePageRange(range, totalPages)
    const indices = pageNumbers.map(n => n - 1)

    const copiedPages = await mergedPdf.copyPages(srcPdf, indices)
    for (const page of copiedPages) {
      mergedPdf.addPage(page)
    }
  }

  const pdfBytes = await mergedPdf.save()
  return new Blob([pdfBytes], { type: 'application/pdf' })
}

// ============ 辅助函数 ============

export function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}
```

- [ ] **Step 2: 验证 TypeScript 编译**

```bash
cd d:\work\trae_use\desktop-tools
npx vue-tsc --noEmit 2>&1 | Select-String "pdfUtils"
```

Expected: No errors related to pdfUtils.ts.

- [ ] **Step 3: 提交**

```bash
git add src/utils/pdfUtils.ts
git commit -m "feat: add pdfUtils with PDF conversion, text extraction, and merge functions"
```

---

### Task 3: 创建 PdfTool.vue 主页面

**Files:**
- Create: `d:\work\trae_use\desktop-tools\src\views\PdfTool.vue`

- [ ] **Step 1: 创建 PdfTool.vue 页面**

基于 `_ToolTemplate.vue` 模板创建，包含 4 个 Tab：

```vue
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
                <p>• DPI 越高，图片越清晰但速度越慢</p>
                <p>• 72 DPI: 快速预览 | 150 DPI: 标准 | 300 DPI: 高清</p>
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
            <div class="group-label">自定义</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :disabled="!mergePdfFiles.length" @click="handleCustomMerge">
                生成 PDF
              </el-button>
            </div>
          </div>
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
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  pdfToImages,
  extractPdfText,
  imagesToPdf,
  mergePdf,
  downloadBlob,
  formatFileSize,
  type ImageToPdfOptions
} from '@/utils/pdfUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const activeTab = ref('pdfToImages')
const error = ref('')

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
    const { loadPdfDocument } = await import('@/utils/pdfUtils')
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
  try {
    imageBlobs.value = await pdfToImages(pdfFile.value, dpi.value)
    ElMessage.success(`转换完成，共 ${imageBlobs.value.length} 页`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF转图片 (${dpi.value}DPI)`,
      inputPreview: pdfFile.value.name.slice(0, 50),
      outputPreview: `${imageBlobs.value.length} 页`
    })
  } catch (e: any) {
    error.value = e.message || '转换失败'
  }
}

const getImageUrl = (blob: Blob) => URL.createObjectURL(blob)

const handleDownloadSingleImage = (blob: Blob, pageNum: number) => {
  downloadBlob(blob, `page_${pageNum}.png`)
}

const handleDownloadAllImages = () => {
  imageBlobs.value.forEach((blob, idx) => {
    setTimeout(() => downloadBlob(blob, `page_${idx + 1}.png`), idx * 200)
  })
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
  try {
    generatedPdfBlob.value = await imagesToPdf(imageFiles.value, imageToPdfOptions)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: '图片转PDF',
      inputPreview: `${imageFiles.value.length} 张图片`,
      outputPreview: formatFileSize(generatedPdfBlob.value.size)
    })
  } catch (e: any) {
    error.value = e.message || '生成失败'
  }
}

const handleDownloadGeneratedPdf = () => {
  if (!generatedPdfBlob.value) return
  downloadBlob(generatedPdfBlob.value, 'output.pdf')
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
  try {
    extractedText.value = await extractPdfText(extractPdfFile.value)
    ElMessage.success('文本提取完成')
    store.addHistory({
      tool: 'pdf',
      action: 'PDF文本提取',
      inputPreview: extractPdfFile.value.name.slice(0, 50),
      outputPreview: extractedText.value.slice(0, 50)
    })
  } catch (e: any) {
    error.value = e.message || '提取失败'
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

const handleDownloadExtractedText = () => {
  if (!extractedText.value) return
  const blob = new Blob([extractedText.value], { type: 'text/plain' })
  downloadBlob(blob, 'extracted-text.txt')
}

// ============ Tab 4: PDF合并/拆分 ============
const mergeInputRef = ref<HTMLInputElement | null>(null)
const mergePdfFiles = ref<File[]>([])
const mergePageRanges = ref<string[]>([])
const mergedPdfBlob = ref<Blob | null>(null)

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
}

const handleRemoveMergePdf = (idx: number) => {
  mergePdfFiles.value.splice(idx, 1)
  mergePageRanges.value.splice(idx, 1)
  mergedPdfBlob.value = null
}

const handleClearMergePdfs = () => {
  mergePdfFiles.value = []
  mergePageRanges.value = []
  mergedPdfBlob.value = null
  error.value = ''
  if (mergeInputRef.value) mergeInputRef.value.value = ''
}

const handleQuickMerge = async (range: string) => {
  if (!mergePdfFiles.value.length) return
  error.value = ''
  try {
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, mergePdfFiles.value.map(() => range))
    ElMessage.success('PDF 合并完成')
    store.addHistory({
      tool: 'pdf',
      action: `PDF合并 (${range})`,
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size)
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  }
}

const handleCustomMerge = async () => {
  if (!mergePdfFiles.value.length) return
  error.value = ''
  try {
    mergedPdfBlob.value = await mergePdf(mergePdfFiles.value, mergePageRanges.value)
    ElMessage.success('PDF 生成完成')
    store.addHistory({
      tool: 'pdf',
      action: 'PDF自定义合并',
      inputPreview: `${mergePdfFiles.value.length} 个文件`,
      outputPreview: formatFileSize(mergedPdfBlob.value.size)
    })
  } catch (e: any) {
    error.value = e.message || '合并失败'
  }
}

const handleDownloadMergedPdf = () => {
  if (!mergedPdfBlob.value) return
  downloadBlob(mergedPdfBlob.value, 'merged.pdf')
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
</style>
```

- [ ] **Step 2: 验证 TypeScript 编译**

```bash
cd d:\work\trae_use\desktop-tools
npx vue-tsc --noEmit 2>&1 | Select-String -Pattern "PdfTool|pdfUtils" | Select-Object -First 10
```

Expected: No errors related to PdfTool or pdfUtils.

- [ ] **Step 3: 提交**

```bash
git add src/views/PdfTool.vue
git commit -m "feat: add PdfTool.vue with 4 tabs (PDF to images, images to PDF, text extract, merge/split)"
```

---

### Task 4: 集成到应用（store + App.vue + SidebarNav）

**Files:**
- Modify: `d:\work\trae_use\desktop-tools\src\store\index.ts`
- Modify: `d:\work\trae_use\desktop-tools\src\App.vue`

- [ ] **Step 1: 在 store/index.ts 中注册 PDF 工具**

在 `TOOL_LIST` 数组中，在 `csv` 工具后面添加：

```typescript
{ id: 'pdf', name: 'PDF工具', icon: 'PDF', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M9 15v-2a2 2 0 014 0v2"/><path d="M9 13h4"/></svg>`, description: 'PDF转图片、图片转PDF、文本提取、合并拆分', keywords: ['pdf', '转换', '合并', '拆分', '提取'], category: 'utility' },
```

- [ ] **Step 2: 在 App.vue 中导入并渲染 PdfTool**

在 import 区域添加：

```typescript
import PdfTool from '@/views/PdfTool.vue'
```

在路由渲染区域（`HttpTool` 后面）添加：

```vue
<PdfTool v-else-if="activeTool === 'pdf'" />
```

- [ ] **Step 3: 升级版本号**

在 `package.json` 中：

```json
"version": "2.12.0"
```

在 `SidebarNav.vue` 中（搜索 `v2.11`）：

```html
<span class="app-version">v2.12</span>
```

- [ ] **Step 4: 验证构建**

```bash
cd d:\work\trae_use\desktop-tools
npm run build
```

Expected: Build succeeds with no new TypeScript errors.

- [ ] **Step 5: 提交**

```bash
git add src/store/index.ts src/App.vue package.json src/components/SidebarNav.vue
git commit -m "feat: integrate PDF tool into app, bump version to v2.12.0"
```

---

### Task 5: 开发环境验证

**Files:**
- No file changes

- [ ] **Step 1: 启动开发服务器**

```bash
cd d:\work\trae_use\desktop-tools
npm run dev
```

- [ ] **Step 2: 手动验证 4 个 Tab 功能**

1. **PDF转图片**: 上传一个 PDF，选择 150 DPI，点击"开始转换"，确认图片预览和下载正常
2. **图片转PDF**: 上传 2-3 张图片，选择 A4 页面尺寸，点击"生成 PDF"，确认下载正常
3. **PDF文本提取**: 上传一个含文字的 PDF，点击"提取文本"，确认文字提取正常
4. **PDF合并/拆分**: 上传 2 个 PDF，点击"合并所有"，确认下载正常

- [ ] **Step 3: 停止开发服务器**

```bash
# Ctrl+C 或 StopCommand
```

- [ ] **Step 4: 提交最终版本**

```bash
git log --oneline -5
```
