<template>
  <div class="tool-container">
    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="barcode-tabs">
        <el-tab-pane label="生成" name="generate" />
        <el-tab-pane label="批量生成" name="batch" />
      </el-tabs>
    </div>

    <!-- Tab 1: 单个生成 -->
    <div v-if="activeTab === 'generate'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>输入内容生成一维条形码</p>
                <p>不同格式对输入有不同要求，详见格式说明</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="genInput += `{{${$event}}}`" />
          <el-button size="small" @click="genInput = ''; genDataUrl = ''; genError = ''">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
          <el-button size="small" type="primary" @click="handleGenerate">生成</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">格式</div>
            <el-select v-model="genFormat" size="small" style="width: 140px" @change="onFormatChange">
              <el-option v-for="f in FORMATS" :key="f.value" :label="f.label" :value="f.value" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">条宽</div>
            <el-input-number v-model="genWidth" :min="1" :max="5" size="small" style="width: 90px" />
          </div>
          <div class="action-group">
            <div class="group-label">条高</div>
            <el-input-number v-model="genHeight" :min="40" :max="300" :step="10" size="small" style="width: 100px" />
          </div>
          <div class="action-group">
            <div class="group-label">边距</div>
            <el-input-number v-model="genMargin" :min="0" :max="20" size="small" style="width: 80px" />
          </div>
          <div class="action-group">
            <div class="group-label">前景色</div>
            <input type="color" v-model="genFgColor" class="native-color-picker" />
          </div>
          <div class="action-group">
            <div class="group-label">背景色</div>
            <input type="color" v-model="genBgColor" class="native-color-picker" />
          </div>
          <div class="action-group">
            <div class="group-label">显示文字</div>
            <el-switch v-model="genDisplayValue" size="small" />
          </div>
          <div class="action-group" v-if="genDisplayValue">
            <div class="group-label">字号</div>
            <el-input-number v-model="genFontSize" :min="8" :max="30" size="small" style="width: 80px" />
          </div>
        </div>
        <!-- 格式说明 -->
        <div class="format-hint">{{ currentFormatDesc }}</div>
      </div>
    </div>

    <div v-if="activeTab === 'generate'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入内容</span>
      </div>
      <div class="card-body">
        <el-input
          v-model="genInput"
          type="textarea"
          :rows="3"
          :placeholder="inputPlaceholder"
          resize="vertical"
          @keydown.ctrl.enter="handleGenerate"
        />
      </div>
    </div>

    <div v-if="activeTab === 'generate'" class="tool-card">
      <div class="card-header">
        <span class="card-title">条形码</span>
        <div class="card-actions">
          <el-button size="small" @click="copyToClipboard" :disabled="!genDataUrl">复制图片</el-button>
          <el-button size="small" @click="handleDownload" :disabled="!genDataUrl">下载 PNG</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="genDataUrl" class="barcode-result">
          <img :src="genDataUrl" alt="Barcode" class="barcode-image" />
        </div>
        <div v-else-if="genError" class="error-message">{{ genError }}</div>
        <div v-else class="stats-empty">输入内容后点击"生成"或按 Ctrl+Enter</div>
      </div>
    </div>

    <!-- Tab 2: 批量生成 -->
    <div v-if="activeTab === 'batch'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">批量操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>每行一条内容，批量生成条形码</p>
                <p>可打包下载为 ZIP</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="batchInput = ''; batchResults = []">清空</el-button>
          <el-button size="small" @click="uploadCsv">
            <el-icon><Upload /></el-icon>
            <span>导入</span>
          </el-button>
          <el-button size="small" type="primary" @click="handleBatchGenerate" :loading="batchLoading">
            <el-icon class="batch-icon"><MagicStick /></el-icon>
            <span>批量生成</span>
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">格式</div>
            <el-select v-model="genFormat" size="small" style="width: 140px">
              <el-option v-for="f in FORMATS" :key="f.value" :label="f.label" :value="f.value" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">条宽</div>
            <el-input-number v-model="genWidth" :min="1" :max="5" size="small" style="width: 90px" />
          </div>
          <div class="action-group">
            <div class="group-label">条高</div>
            <el-input-number v-model="genHeight" :min="40" :max="300" :step="10" size="small" style="width: 100px" />
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

    <div v-if="activeTab === 'batch'" class="tool-card">
      <div class="card-header">
        <span class="card-title">内容列表</span>
        <div class="card-actions">
          <span class="stat-text">{{ batchLines.length }} 条</span>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="batchInput"
          type="textarea"
          :rows="8"
          placeholder="每行一条内容，例如：&#10;123456789012&#10;HELLO-WORLD&#10;9787111111111"
          resize="vertical"
        />
        <div v-if="batchErrors.length" class="batch-errors">
          <span class="error-title">{{ batchErrors.length }} 条失败：</span>
          <span v-for="e in batchErrors" :key="e.line" class="error-item">第{{ e.line }}行</span>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'batch' && batchResults.length" class="tool-card">
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
          <div v-for="item in batchResults" :key="item.index" class="batch-item">
            <img :src="item.dataUrl" :alt="item.filename" class="batch-barcode" />
            <div class="batch-info">
              <span class="batch-filename" :title="item.text">{{ item.filename }}</span>
              <span class="batch-text" :title="item.text">{{ item.text }}</span>
            </div>
            <div class="batch-actions">
              <el-button size="small" @click="downloadSingle(item)">
                <el-icon><Download /></el-icon>
              </el-button>
              <el-button size="small" @click="copySingle(item)">
                <el-icon><DocumentCopy /></el-icon>
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Upload, MagicStick, Download, DocumentCopy } from '@element-plus/icons-vue'
import JsBarcode from 'jsbarcode'
import { useToolboxStore } from '@/store'
import { saveFileWithDialog } from '@/utils/fileSaver'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

// ============ 格式定义 ============
const FORMATS = [
  { value: 'CODE128', label: 'CODE128', desc: '支持任意 ASCII 字符（字母/数字/符号），最通用的条码格式' },
  { value: 'EAN13', label: 'EAN-13', desc: '13 位数字（末位为校验码，12 位时自动补算），商品条码国际标准' },
  { value: 'EAN8', label: 'EAN-8', desc: '8 位数字（末位为校验码，7 位时自动补算），小商品条码' },
  { value: 'UPC', label: 'UPC-A', desc: '12 位数字（末位为校验码，11 位时自动补算），北美商品条码' },
  { value: 'CODE39', label: 'CODE39', desc: '大写字母 + 数字 + 特殊符号（- . $ / + % 空格），工业/军事用' },
  { value: 'ITF14', label: 'ITF-14', desc: '14 位数字（末位为校验码，13 位时自动补算），物流包装条码' },
  { value: 'codabar', label: 'Codabar', desc: '数字 + 特殊符号（- $ : / . +），图书馆/医疗用' },
  { value: 'MSI', label: 'MSI', desc: '数字，仓库/货架管理用' },
  { value: 'pharmacode', label: 'Pharmacode', desc: '3~131070 之间的整数，药品包装专用' },
] as const

// ============ Tab 状态 ============
const activeTab = ref('generate')

// 还原期间禁止写历史
let isRestoring = false
let restoreTimer: ReturnType<typeof setTimeout> | null = null
const blockHistory = () => {
  isRestoring = true
  if (restoreTimer) clearTimeout(restoreTimer)
  restoreTimer = setTimeout(() => { isRestoring = false }, 500)
}

// ============ 生成 Tab ============
const genInput = ref('')
const genFormat = ref<string>('CODE128')
const genWidth = ref(2)
const genHeight = ref(100)
const genMargin = ref(10)
const genFgColor = ref('#000000')
const genBgColor = ref('#ffffff')
const genDisplayValue = ref(true)
const genFontSize = ref(20)
const genDataUrl = ref('')
const genError = ref('')

const currentFormatDesc = computed(() => {
  const f = FORMATS.find(f => f.value === genFormat.value)
  return f ? f.desc : ''
})

const inputPlaceholder = computed(() => {
  switch (genFormat.value) {
    case 'CODE128': return '任意文本，如 ABC123 或 https://example.com'
    case 'EAN13': return '12 或 13 位数字，如 690123456789'
    case 'EAN8': return '7 或 8 位数字，如 6901234'
    case 'UPC': return '11 或 12 位数字，如 036000291452'
    case 'CODE39': return '大写字母+数字+符号，如 HELLO-123'
    case 'ITF14': return '13 或 14 位数字，如 10012345678902'
    case 'codabar': return '数字+符号，如 A12345B'
    case 'MSI': return '数字，如 1234567'
    case 'pharmacode': return '3~131070 的整数，如 1234'
    default: return '输入内容'
  }
})

// 切换格式时清空已生成结果（不同格式校验不同）
const onFormatChange = () => {
  genDataUrl.value = ''
  genError.value = ''
}

/** 核心：用 JsBarcode 渲染到 canvas，返回 dataURL */
const renderBarcode = (text: string): string => {
  const canvas = document.createElement('canvas')
  JsBarcode(canvas, text, {
    format: genFormat.value,
    width: genWidth.value,
    height: genHeight.value,
    margin: genMargin.value,
    background: genBgColor.value,
    lineColor: genFgColor.value,
    displayValue: genDisplayValue.value,
    fontSize: genFontSize.value,
    textAlign: 'center',
    textPosition: 'bottom',
    textMargin: 2,
  })
  return canvas.toDataURL('image/png')
}

const handleGenerate = () => {
  const input = genInput.value.trim()
  if (!input) {
    ElMessage.warning('请输入内容')
    return
  }
  try {
    genDataUrl.value = renderBarcode(input)
    genError.value = ''
    if (!isRestoring) {
      store.addHistory({
        tool: 'barcode',
        action: '生成',
        inputPreview: `${genFormat.value} | ${input.slice(0, 40)}`,
        outputPreview: '条形码已生成',
        inputFull: `${genFormat.value}\n${input}`,
        outputFull: genDataUrl.value,
      })
    }
    ElMessage.success('生成成功')
  } catch (e: any) {
    genDataUrl.value = ''
    genError.value = '生成失败: ' + (e.message || '请检查输入是否符合格式要求')
  }
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    genInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleDownload = async () => {
  if (!genDataUrl.value) return
  const response = await fetch(genDataUrl.value)
  const blob = await response.blob()
  await saveFileWithDialog(blob, `barcode_${genFormat.value.toLowerCase()}.png`, 'png')
}

const copyToClipboard = async () => {
  if (!genDataUrl.value) return
  try {
    const response = await fetch(genDataUrl.value)
    const blob = await response.blob()
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      ElMessage.success('已复制到剪贴板')
    } else {
      ElMessage.warning('当前环境不支持图片剪贴板')
    }
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 批量生成 Tab ============
interface BatchResult {
  index: number
  text: string
  dataUrl: string
  filename: string
}

const batchInput = ref('')
const batchNameMode = ref('indexed')
const batchResults = ref<BatchResult[]>([])
const batchErrors = ref<{ line: number; text: string }[]>([])
const batchLoading = ref(false)

const batchLines = computed(() => {
  return batchInput.value.split('\n').map(l => l.trim()).filter(l => l.length > 0)
})

const sanitizeFilename = (s: string) => s.replace(/[<>:"/\\|?*]/g, '_').slice(0, 60)

const handleBatchGenerate = async () => {
  const lines = batchLines.value
  if (lines.length === 0) {
    ElMessage.warning('请输入至少一行内容')
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
      const dataUrl = renderBarcode(text)
      const filename = batchNameMode.value === 'indexed'
        ? String(i + 1).padStart(3, '0') + '.png'
        : sanitizeFilename(text) + '.png'
      batchResults.value.push({ index: i, text, dataUrl, filename })
    } catch {
      batchErrors.value.push({ line: i + 1, text })
    }
  }

  batchLoading.value = false
  const ok = batchResults.value.length
  const fail = batchErrors.value.length
  if (ok > 0) {
    ElMessage.success(`完成：成功 ${ok} 条${fail > 0 ? `，失败 ${fail} 条` : ''}`)
    store.addHistory({
      tool: 'barcode',
      action: '批量生成',
      inputPreview: `${genFormat.value} | ${ok} 条`,
      outputPreview: `${ok} 张条形码`,
      inputFull: `${genFormat.value}\n${lines.join('\n')}`,
      outputFull: `批量生成 ${ok} 张条形码（${genFormat.value}）`,
    })
  } else {
    ElMessage.error('全部生成失败，请检查输入是否符合格式要求')
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
      batchInput.value = lines.join('\n')
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

const copySingle = async (item: BatchResult) => {
  try {
    const response = await fetch(item.dataUrl)
    const blob = await response.blob()
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      ElMessage.success('已复制')
    } else {
      ElMessage.warning('当前环境不支持图片剪贴板')
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
    await saveFileWithDialog(blob, `barcode_batch_${batchResults.value.length}.zip`, 'zip')
  } catch (e: any) {
    ElMessage.error('打包失败：' + (e.message || '未知错误'))
  }
}

// ============ 历史还原 ============
watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== 'barcode') return
  blockHistory()
  const action = restore.action
  if (action === '生成') {
    activeTab.value = 'generate'
    // inputFull 格式: "FORMAT\n内容"
    const parts = (restore.input || '').split('\n')
    if (parts.length >= 2) {
      genFormat.value = parts[0]
      genInput.value = parts.slice(1).join('\n')
    } else {
      genInput.value = restore.input || ''
    }
    if (genInput.value) handleGenerate()
  } else if (action === '批量生成') {
    activeTab.value = 'batch'
    const parts = (restore.input || '').split('\n')
    if (parts.length >= 2) {
      genFormat.value = parts[0]
      batchInput.value = parts.slice(1).join('\n')
    } else {
      batchInput.value = restore.input || ''
    }
    if (batchInput.value) handleBatchGenerate()
  }
  store.clearHistoryRestore()
})
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.barcode-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
html.light .barcode-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.barcode-tabs :deep(.el-tabs__nav-wrap) { padding-left: 4px; }
.barcode-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}
.barcode-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.barcode-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.barcode-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }

/* ===== 格式说明 ===== */
.format-hint {
  margin-top: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-left: 3px solid var(--accent-cyan);
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
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

/* ===== 提示图标 ===== */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.header-left { display: flex; align-items: center; gap: 8px; }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

/* ===== 条形码结果 ===== */
.barcode-result {
  display: flex;
  justify-content: center;
  padding: 20px 0;
}
.barcode-image {
  max-width: 100%;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: #fff;
}

/* ===== 错误提示 ===== */
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
.batch-icon { margin-right: 4px; }
.batch-errors {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}
.batch-errors .error-title { color: var(--accent-red); font-size: 12px; }
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
.batch-barcode {
  width: 100%;
  max-width: 160px;
  height: 100px;
  object-fit: contain;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: #fff;
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
.batch-actions { display: flex; gap: 4px; }
</style>
