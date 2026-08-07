<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="qr-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 二维码生成 -->
      <el-tab-pane label="生成" name="generate">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入文本或 URL，生成二维码图片</p>
                    <p>支持配置尺寸、边距、前景色/背景色</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('generate')">清空</el-button>
              <el-button size="small" @click="handlePaste('generate')">粘贴</el-button>
              <el-button size="small" type="primary" @click="handleGenerate">生成</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">尺寸</div>
                <el-input-number v-model="qrSize" :min="100" :max="1000" :step="50" size="small" style="width: 100px" />
              </div>
              <div class="action-group">
                <div class="group-label">边距</div>
                <el-input-number v-model="qrMargin" :min="0" :max="10" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">前景色</div>
                <input type="color" v-model="qrFgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">背景色</div>
                <input type="color" v-model="qrBgColor" class="native-color-picker" />
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入内容</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('generate')">清空</el-button>
              <el-button size="small" @click="handlePaste('generate')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.generate.input"
              type="textarea"
              :rows="4"
              placeholder="输入文本或 URL..."
              resize="vertical"
            />
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">二维码</span>
            <el-button size="small" @click="handleDownloadQr" :disabled="!qrDataUrl">下载 PNG</el-button>
          </div>
          <div class="card-body">
            <div v-if="qrDataUrl" class="qr-result">
              <img :src="qrDataUrl" alt="QR Code" class="qr-image" />
            </div>
            <div v-else-if="generateError" class="error-message">{{ generateError }}</div>
            <div v-else class="stats-empty">
              点击"生成"按钮生成二维码
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 批量生成 -->
      <el-tab-pane label="批量生成" name="batch">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">批量操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>每行一条文本，支持 CSV/文本列表</p>
                    <p>可批量生成并打包下载</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('batch')">清空</el-button>
              <el-button size="small" @click="handlePaste('batch')">粘贴</el-button>
              <el-button size="small" type="primary" @click="handleBatchGenerate" :loading="batchLoading">
                <el-icon class="batch-icon"><MagicStick /></el-icon>
                <span>批量生成</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">尺寸</div>
                <el-input-number v-model="batchSize" :min="100" :max="1000" :step="50" size="small" style="width: 100px" />
              </div>
              <div class="action-group">
                <div class="group-label">边距</div>
                <el-input-number v-model="batchMargin" :min="0" :max="10" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">前景色</div>
                <input type="color" v-model="batchFgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">背景色</div>
                <input type="color" v-model="batchBgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">命名</div>
                <el-select v-model="batchNameMode" size="small" style="width: 140px">
                  <el-option label="序号 (001, 002...)" value="indexed" />
                  <el-option label="文本前16字符" value="text" />
                </el-select>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">文本列表</span>
            <div class="card-actions">
              <span class="stat-text">{{ batchLines.length }} 条</span>
              <el-button size="small" @click="uploadCsv">
                <el-icon><Upload /></el-icon>
                <span>导入</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.batch.input"
              type="textarea"
              :rows="8"
              placeholder="每行一条文本或 URL，例如：&#10;https://example.com&#10;Hello World&#10;SELECT * FROM users"
              resize="vertical"
            />
            <div v-if="batchErrors.length" class="batch-errors">
              <span class="error-title">{{ batchErrors.length }} 条失败：</span>
              <span v-for="(e, i) in batchErrors" :key="i" class="error-item">第{{ e.line }}行</span>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card" v-if="batchResults.length">
          <div class="card-header">
            <span class="card-title">生成结果（{{ batchResults.length }} 条）</span>
            <div class="card-actions">
              <el-button size="small" @click="downloadAllZip">
                <el-icon><Download /></el-icon>
                <span>打包下载 (ZIP)</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="batch-grid">
              <div v-for="(item, i) in batchResults" :key="i" class="batch-item">
                <img :src="item.dataUrl" :alt="item.filename" class="batch-qr" />
                <div class="batch-info">
                  <span class="batch-filename" :title="item.text">{{ item.filename }}</span>
                  <span class="batch-text" :title="item.text">{{ item.text }}</span>
                </div>
                <div class="batch-actions">
                  <el-button size="small" @click="downloadSingle(item)">
                    <el-icon><Download /></el-icon>
                    <span>下载</span>
                  </el-button>
                  <el-button size="small" @click="copyDataUrl(item)">
                    <el-icon><DocumentCopy /></el-icon>
                    <span>复制</span>
                  </el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2 (实际顺序 Tab 3): 二维码解码 -->
      <el-tab-pane label="解码" name="decode">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>上传二维码图片或从剪贴板粘贴，解析二维码内容</p>
                    <p>支持 PNG/JPG/WebP 格式</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('decode')">清空</el-button>
              <el-button size="small" @click="handleCopy('decode')">复制</el-button>
            </div>
          </div>
        </div>

        <!-- 上传卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">上传二维码</span>
          </div>
          <div class="card-body">
            <div class="upload-area" @click="triggerFileInput" @drop.prevent="handleDrop" @dragover.prevent>
              <input ref="fileInput" type="file" accept="image/*" class="file-input" @change="handleFileChange" />
              <div class="upload-content">
                <el-icon class="upload-icon"><Upload /></el-icon>
                <p>点击上传或拖拽图片到此处</p>
                <p class="upload-hint">支持 PNG / JPG / WebP</p>
              </div>
            </div>
            <div v-if="decodePreview" class="decode-preview">
              <img :src="decodePreview" alt="Preview" class="preview-image" />
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">解析结果</span>
            <div class="card-actions">
              <el-button size="small" @click="handleCopy('decode')">复制</el-button>
              <el-button size="small" type="success" :disabled="!tabState.decode.output" @click="handleSaveDecodeEdit">保存修改</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.decode.output"
              type="textarea"
              :rows="6"
              resize="vertical"
              :class="{ 'error': tabState.decode.isError }"
            />
            <div v-if="tabState.decode.error" class="error-message">
              {{ tabState.decode.error }}
            </div>
            <div v-if="!tabState.decode.output && !tabState.decode.error" class="stats-empty">
              上传图片后自动解析
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Upload, MagicStick, Download, DocumentCopy } from '@element-plus/icons-vue'
import QRCode from 'qrcode'
import jsQR from 'jsqr'
import { useToolboxStore } from '@/store'
import { saveFileWithDialog } from '@/utils/fileSaver'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('generate')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  generate: { input: '', output: '', error: '', isError: false },
  decode: { input: '', output: '', error: '', isError: false },
  batch: { input: '', output: '', error: '', isError: false }
})

// ============ 生成 Tab ============
const qrSize = ref(300)
const qrMargin = ref(2)
const qrFgColor = ref('#000000')
const qrBgColor = ref('#ffffff')
const qrDataUrl = ref('')
const generateError = ref('')

const handleGenerate = async () => {
  const input = tabState.generate.input.trim()
  if (!input) {
    ElMessage.warning('请输入内容')
    return
  }

  try {
    qrDataUrl.value = await QRCode.toDataURL(input, {
      width: qrSize.value,
      margin: qrMargin.value,
      color: {
        dark: qrFgColor.value,
        light: qrBgColor.value
      }
    })
    generateError.value = ''
    store.addHistory({ tool: 'qr', action: 'generate', inputPreview: input.slice(0, 30), outputPreview: '二维码已生成', inputFull: input, outputFull: qrDataUrl.value })
    ElMessage.success('二维码生成成功')
  } catch (e: any) {
    qrDataUrl.value = ''
    generateError.value = '生成失败: ' + (e.message || '未知错误')
    ElMessage.error('生成失败')
  }
}

const handleDownloadQr = async () => {
  if (!qrDataUrl.value) {
    ElMessage.warning('没有可下载的内容')
    return
  }
  const response = await fetch(qrDataUrl.value)
  const blob = await response.blob()
  await saveFileWithDialog(blob, 'qrcode.png', 'png')
}

// ============ 解码 Tab ============
const fileInput = ref<HTMLInputElement | null>(null)
const decodePreview = ref('')

const triggerFileInput = () => {
  fileInput.value?.click()
}

const handleFileChange = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) {
    processImageFile(file)
  }
}

const handleDrop = (e: DragEvent) => {
  const file = e.dataTransfer?.files[0]
  if (file && file.type.startsWith('image/')) {
    processImageFile(file)
  }
}

const processImageFile = (file: File) => {
  const reader = new FileReader()
  reader.onload = (e) => {
    const dataUrl = e.target?.result as string
    decodePreview.value = dataUrl
    decodeQrFromImage(dataUrl)
  }
  reader.readAsDataURL(file)
}

const decodeQrFromImage = (dataUrl: string) => {
  const img = new Image()
  img.onload = () => {
    const canvas = document.createElement('canvas')
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      tabState.decode.error = '无法创建画布上下文'
      tabState.decode.isError = true
      return
    }
    ctx.drawImage(img, 0, 0)
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
    const code = jsQR(imageData.data, imageData.width, imageData.height)

    if (code && code.data) {
      tabState.decode.output = code.data
      tabState.decode.error = ''
      tabState.decode.isError = false
      store.addHistory({ tool: 'qr', action: 'decode', inputPreview: '图片', outputPreview: code.data.slice(0, 30), inputFull: '[图片]', outputFull: code.data })
      ElMessage.success('解析成功')
    } else {
      tabState.decode.output = ''
      tabState.decode.error = '未检测到二维码，请确保图片清晰且包含标准二维码'
      tabState.decode.isError = true
      ElMessage.warning('未检测到二维码')
    }
  }
  img.src = dataUrl
}

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'generate') {
    qrDataUrl.value = ''
    generateError.value = ''
  }
  if (tab === 'decode') {
    decodePreview.value = ''
  }
  if (tab === 'batch') {
    batchResults.value = []
    batchErrors.value = []
  }
}

const handlePaste = async (tab: string) => {
  try {
    const text = await navigator.clipboard.readText()
    tabState[tab].input = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (tab: string) => {
  const text = tabState[tab].output
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleSaveDecodeEdit = () => {
  if (!tabState.decode.output) return
  store.addHistory({
    tool: 'qr',
    action: '二维码解码(已编辑)',
    inputPreview: '图片',
    outputPreview: tabState.decode.output.slice(0, 50),
    inputFull: '[图片]',
    outputFull: tabState.decode.output,
  })
  ElMessage.success('修改已保存')
}

// ============ 批量生成 Tab ============

interface BatchResult {
  text: string
  dataUrl: string
  filename: string
}

interface BatchError {
  line: number
  text: string
  error: string
}

const batchSize = ref(200)
const batchMargin = ref(2)
const batchFgColor = ref('#000000')
const batchBgColor = ref('#ffffff')
const batchNameMode = ref('indexed')
const batchResults = ref<BatchResult[]>([])
const batchErrors = ref<BatchError[]>([])
const batchLoading = ref(false)

const batchLines = computed(() => {
  return tabState.batch.input.split('\n').map(l => l.trim()).filter(l => l.length > 0)
})

const sanitizeFilename = (s: string) => s.replace(/[<>:"/\\|?*]/g, '_').slice(0, 60)

const handleBatchGenerate = async () => {
  const lines = batchLines.value
  if (lines.length === 0) {
    ElMessage.warning('请输入至少一行文本')
    return
  }
  if (lines.length > 500) {
    ElMessage.warning('最多支持 500 条')
    return
  }

  batchLoading.value = true
  batchResults.value = []
  batchErrors.value = []

  for (let i = 0; i < lines.length; i++) {
    const text = lines[i]
    try {
      const dataUrl = await QRCode.toDataURL(text, {
        width: batchSize.value,
        margin: batchMargin.value,
        color: { dark: batchFgColor.value, light: batchBgColor.value }
      })
      const filename = batchNameMode.value === 'indexed'
        ? String(i + 1).padStart(3, '0') + '.png'
        : sanitizeFilename(text) + '.png'

      batchResults.value.push({ text, dataUrl, filename })
    } catch (e: any) {
      batchErrors.value.push({ line: i + 1, text, error: e.message || '生成失败' })
    }
  }

  batchLoading.value = false
  const ok = batchResults.value.length
  const fail = batchErrors.value.length
  if (ok > 0) {
    ElMessage.success(`完成：成功 ${ok} 条${fail > 0 ? `，失败 ${fail} 条` : ''}`)
    store.addHistory({
      tool: 'qr',
      action: '批量生成',
      inputPreview: `${ok} 条文本`,
      outputPreview: `${ok} 张二维码`,
      inputFull: lines.join('\n'),
      outputFull: `批量生成 ${ok} 张二维码`
    })
  } else {
    ElMessage.error('全部生成失败')
  }
}

const uploadCsv = () => {
  const inputEl = document.createElement('input')
  inputEl.type = 'file'
  inputEl.accept = '.csv,.txt'
  inputEl.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      const text = reader.result as string
      const lines = text.split(/\r?\n/).map(l => l.trim()).filter(l => l.length > 0)
      tabState.batch.input = lines.join('\n')
      ElMessage.success(`导入 ${lines.length} 行`)
    }
    reader.readAsText(file)
  }
  inputEl.click()
}

const downloadSingle = async (item: BatchResult) => {
  const response = await fetch(item.dataUrl)
  const blob = await response.blob()
  await saveFileWithDialog(blob, item.filename, 'png')
}

const copyDataUrl = async (item: BatchResult) => {
  try {
    const response = await fetch(item.dataUrl)
    const blob = await response.blob()
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      ElMessage.success('已复制到剪贴板')
    } else {
      ElMessage.warning('当前浏览器不支持图片剪贴板操作')
    }
  } catch {
    ElMessage.error('复制失败')
  }
}

const downloadAllZip = async () => {
  if (batchResults.value.length === 0) return
  try {
    const JSZip = (await import('jszip')).default
    const zip = new JSZip()
    for (const item of batchResults.value) {
      const base64 = item.dataUrl.split(',')[1]
      zip.file(item.filename, base64, { base64: true })
    }
    const blob = await zip.generateAsync({ type: 'blob' })
    await saveFileWithDialog(blob, `qrcode_batch_${batchResults.value.length}.zip`, 'zip')
  } catch (e: any) {
    ElMessage.error('打包失败：' + (e.message || '未知错误'))
  }
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.qr-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .qr-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.qr-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.qr-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.qr-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.qr-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.qr-tool-tabs :deep(.el-tabs__nav-wrap::after) {
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

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 操作按钮 */
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

/* 提示图标 */
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

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
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

/* ===== 颜色选择器 ===== */
.native-color-picker {
  width: 40px;
  height: 32px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px;
  background: var(--bg-input);
}

/* ===== 二维码结果 ===== */
.qr-result {
  display: flex;
  justify-content: center;
  padding: 20px 0;
}

.qr-image {
  max-width: 100%;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

/* ===== 上传区域 ===== */
.upload-area {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s, background 0.3s;
}

.upload-area:hover {
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.05);
}

.file-input {
  display: none;
}

.upload-icon {
  font-size: 48px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.upload-content p {
  color: var(--text-secondary);
  margin: 4px 0;
}

.upload-hint {
  font-size: 12px;
  color: var(--text-muted);
}

/* ===== 解码预览 ===== */
.decode-preview {
  margin-top: 16px;
  text-align: center;
}

.preview-image {
  max-width: 300px;
  max-height: 300px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

/* ===== 空状态 ===== */
.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}

/* ===== 批量生成 ===== */
.stat-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-right: 8px;
}

.batch-icon {
  margin-right: 4px;
}

.batch-errors {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.batch-errors .error-title {
  color: var(--accent-red);
  font-size: 12px;
}

.batch-errors .error-item {
  padding: 2px 6px;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 3px;
  color: var(--accent-red);
  font-size: 12px;
}

.batch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.batch-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  gap: 8px;
}

.batch-qr {
  width: 140px;
  height: 140px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.batch-info {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.batch-filename {
  font-size: 12px;
  color: var(--accent-cyan);
  font-weight: 500;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.batch-text {
  font-size: 11px;
  color: var(--text-secondary);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.batch-actions {
  display: flex;
  gap: 4px;
}
</style>
