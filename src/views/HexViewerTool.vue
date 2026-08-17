<template>
  <div class="tool-container">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">十六进制查看器</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>以十六进制方式查看任意二进制文件</p>
                <p>• 支持拖拽文件到此处</p>
                <p>• 左侧为十六进制字节，右侧为 ASCII 预览</p>
                <p>• 每行显示 16 字节</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-button size="small" type="primary" @click="triggerFileInput">选择文件</el-button>
          <el-button v-if="hexData" size="small" @click="handleClear">清空</el-button>
        </div>
      </div>
    </div>

    <!-- 文件信息 -->
    <div v-if="hexData" class="tool-card">
      <div class="card-body">
        <div class="file-info">
          <span class="info-name">{{ hexData.fileName }}</span>
          <span class="info-size">{{ formatFileSize(hexData.fileSize) }}</span>
          <span class="info-bytes">共 {{ hexData.totalBytes }} 字节</span>
        </div>
      </div>
    </div>

    <!-- 十六进制内容 -->
    <div v-if="hexData" class="tool-card hex-card">
      <div class="card-header">
        <span class="card-title">十六进制内容</span>
        <div class="card-actions">
          <span class="page-info">第 {{ currentPage }} / {{ totalPages }} 页</span>
          <el-button size="small" :disabled="currentPage <= 1" @click="prevPage">上一页</el-button>
          <el-button size="small" :disabled="currentPage >= totalPages" @click="nextPage">下一页</el-button>
        </div>
      </div>
      <div class="card-body hex-body">
        <div class="hex-content">
          <div v-for="(row, idx) in currentPageRows" :key="idx" class="hex-row">
            <span class="hex-offset">{{ formatOffset(row.offset) }}</span>
            <span class="hex-bytes">
              <span
                v-for="(byte, bIdx) in row.bytes"
                :key="bIdx"
                class="hex-byte"
                :class="{ 'byte-separator': (bIdx + 1) % 8 === 0 && bIdx < 15 }"
              >{{ byte }}</span>
            </span>
            <span class="hex-ascii">{{ row.ascii }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="!hexData" class="tool-card">
      <div
        class="card-body drop-zone"
        :class="{ 'drag-over': isDragging }"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <input
          ref="fileInputRef"
          type="file"
          style="display: none"
          @change="handleFileSelect"
        />
        <div class="drop-hint">
          <div class="drop-icon"></div>
          <p>点击「选择文件」或拖拽文件到此处</p>
          <p class="drop-sub">支持任意二进制文件</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'

interface HexRow {
  offset: number
  bytes: string[]
  ascii: string
}

interface HexData {
  fileName: string
  fileSize: number
  totalBytes: number
  rows: HexRow[]
}

const fileInputRef = ref<HTMLInputElement | null>(null)
const hexData = ref<HexData | null>(null)
const isDragging = ref(false)
const currentPage = ref(1)
const ROWS_PER_PAGE = 100
const BYTES_PER_ROW = 16

const totalPages = computed(() => {
  if (!hexData.value) return 0
  return Math.ceil(hexData.value.rows.length / ROWS_PER_PAGE)
})

const currentPageRows = computed(() => {
  if (!hexData.value) return []
  const start = (currentPage.value - 1) * ROWS_PER_PAGE
  return hexData.value.rows.slice(start, start + ROWS_PER_PAGE)
})

const triggerFileInput = () => {
  fileInputRef.value?.click()
}

const handleFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await loadFile(file)
  input.value = ''
}

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = true
}

const handleDragLeave = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
}

const handleDrop = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return
  loadFile(files[0])
}

const loadFile = async (file: File) => {
  const maxSize = 10 * 1024 * 1024 // 10MB
  if (file.size > maxSize) {
    ElMessage.warning(`文件过大，建议小于 ${formatFileSize(maxSize)}`)
    return
  }

  try {
    const buffer = await file.arrayBuffer()
    const bytes = new Uint8Array(buffer)
    const rows: HexRow[] = []

    for (let offset = 0; offset < bytes.length; offset += BYTES_PER_ROW) {
      const chunk = bytes.slice(offset, offset + BYTES_PER_ROW)
      const hexBytes: string[] = []
      let ascii = ''

      for (let i = 0; i < BYTES_PER_ROW; i++) {
        if (i < chunk.length) {
          const byte = chunk[i]
          hexBytes.push(byte.toString(16).padStart(2, '0').toUpperCase())
          ascii += byte >= 32 && byte <= 126 ? String.fromCharCode(byte) : '.'
        } else {
          hexBytes.push('  ')
          ascii += ' '
        }
      }

      rows.push({ offset, bytes: hexBytes, ascii })
    }

    hexData.value = {
      fileName: file.name,
      fileSize: file.size,
      totalBytes: bytes.length,
      rows
    }
    currentPage.value = 1
  } catch {
    ElMessage.error('文件读取失败')
  }
}

const handleClear = () => {
  hexData.value = null
  currentPage.value = 1
}

const prevPage = () => {
  if (currentPage.value > 1) currentPage.value--
}

const nextPage = () => {
  if (currentPage.value < totalPages.value) currentPage.value++
}

const formatOffset = (offset: number): string => {
  return offset.toString(16).toUpperCase().padStart(8, '0')
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
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

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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
.card-body { padding: 16px 20px; }

.header-left { display: flex; align-items: center; gap: 8px; }
.header-actions { display: flex; align-items: center; gap: 12px; }

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

/* 文件信息 */
.file-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}
.info-name { color: var(--text-primary); font-weight: 500; }
.info-size, .info-bytes { color: var(--text-muted); }

/* 十六进制内容 */
.hex-card { max-height: calc(100vh - 200px); }
.hex-body { padding: 0; overflow: hidden; }

.hex-content {
  max-height: calc(100vh - 300px);
  overflow-y: auto;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.8;
}

.hex-row {
  display: flex;
  align-items: center;
  padding: 2px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  transition: background 0.15s;
}
.hex-row:hover { background: rgba(0, 212, 255, 0.05); }

.hex-offset {
  color: var(--text-muted);
  min-width: 80px;
  user-select: none;
}

.hex-bytes {
  flex: 1;
  display: inline-flex;
  gap: 2px;
  min-width: 300px;
}

.hex-byte {
  display: inline-block;
  width: 22px;
  text-align: center;
  color: var(--accent-cyan);
}
.hex-byte.byte-separator {
  margin-right: 8px;
}

.hex-ascii {
  color: var(--text-secondary);
  min-width: 160px;
  letter-spacing: 1px;
  user-select: text;
}

.page-info {
  font-size: 12px;
  color: var(--text-muted);
}

/* 拖拽区域 */
.drop-zone {
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}
.drop-zone.drag-over {
  background: rgba(0, 212, 255, 0.05);
}

.drop-hint {
  text-align: center;
  color: var(--text-muted);
}
.drop-icon {
  font-size: 48px;
  margin-bottom: 12px;
}
.drop-hint p { margin: 4px 0; }
.drop-sub { font-size: 12px; color: var(--text-secondary); }
</style>
