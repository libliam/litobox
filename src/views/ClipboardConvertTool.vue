<template>
  <div class="tool-container">
    <!-- 大操作区：从剪贴板读 + 图片/文本切换 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">剪贴板转换</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>一键读取剪贴板内容，自动识别类型并显示支持的转换选项</p>
                <p>• 图片 → Base64 / DataURL / &lt;img&gt; 标签</p>
                <p>• 文本(HTML/Tab分隔/配置) → 多种格式互转</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="readClipboard">
            <el-icon><DocumentCopy /></el-icon> 从剪贴板读取
          </el-button>
          <el-button size="small" @click="clearAll">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="source-info">
          <div class="source-badge" :class="`type-${sourceType}`">
            {{ SOURCE_LABEL[sourceType] }}
          </div>
          <span v-if="sourceInfo" class="source-meta">{{ sourceInfo }}</span>
        </div>
      </div>
    </div>

    <!-- 图片预览（当剪贴板是图片时） -->
    <div v-if="sourceType === 'image'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片预览</span>
        <div class="card-actions">
          <VariablePicker @select="insertVariable($event, 'imgOut')" />
          <el-button size="small" @click="copy(imgOutput)" :disabled="!imgOutput">复制结果</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="preview-row">
          <div class="preview-img-wrap">
            <img v-if="lastDataUrl" :src="lastDataUrl" class="preview-img" />
            <div v-else class="empty-img">（无图片）</div>
          </div>
          <div class="img-actions">
            <div class="action-group" style="--group-color: #10b981">
              <div class="group-label">Base64</div>
              <div class="group-buttons">
                <el-button size="small" @click="imgToBase64(false)">纯Base64</el-button>
                <el-button size="small" @click="imgToBase64(true)">DataURL</el-button>
                <el-button size="small" @click="imgToImgTag">&lt;img&gt;标签</el-button>
                <el-button size="small" @click="imgToEncodedUrl">URL编码DataURL</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="output-wrap" style="margin-top: 16px;">
          <div class="card-subtitle">输出</div>
          <el-input
            v-model="imgOutput"
            type="textarea"
            :rows="8"
            readonly
            placeholder="点上面的按钮生成..."
            resize="vertical"
            :class="{ error: isError }"
          />
          <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
        </div>
      </div>
    </div>

    <!-- 文本转换（当剪贴板是文本 / 或手动输入时） -->
    <div v-if="sourceType === 'text' || sourceType === 'unknown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入文本</span>
        <div class="card-actions">
          <VariablePicker @select="insertVariable($event, 'textIn')" />
          <el-button size="small" @click="textInput = ''">清空</el-button>
          <el-button size="small" @click="pasteToTextInput">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="textInput"
          type="textarea"
          :rows="5"
          placeholder="HTML / Tab分隔表格 / JSON / YAML / TOML / INI / Properties ..."
          resize="vertical"
        />
        <div v-if="detectedHint" class="detected-hint">
          <el-icon><MagicStick /></el-icon> 自动识别：{{ detectedHint }}
        </div>
      </div>
    </div>

    <!-- 文本转换操作组（多卡片独立样式） -->
    <div v-if="sourceType === 'text' || sourceType === 'unknown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">转换</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="--group-color: #8b5cf6">
            <div class="group-label">HTML → Markdown</div>
            <div class="group-buttons">
              <el-button size="small" type="primary" @click="doHtmlToMd">转换</el-button>
              <el-button size="small" @click="doHtmlToMdAndCopy">转换并复制</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #00d4ff">
            <div class="group-label">表格（Tab分隔）→</div>
            <div class="group-buttons">
              <el-button size="small" @click="doTsvTo('md')">Markdown</el-button>
              <el-button size="small" @click="doTsvTo('csv')">CSV</el-button>
              <el-button size="small" @click="doTsvTo('json')">JSON数组</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #10b981">
            <div class="group-label">配置格式互转</div>
            <div class="group-buttons" style="flex-direction: column; align-items: stretch;">
              <!-- 交换与下拉框同一行（功能不同，不与转换按钮并排） -->
              <div class="format-pickers">
                <el-select v-model="srcFmt" size="small" placeholder="源格式">
                  <el-option v-for="o in fmtOptions" :key="o.v" :label="o.l" :value="o.v" />
                </el-select>
                <span class="fmt-arrow">→</span>
                <el-select v-model="dstFmt" size="small" placeholder="目标格式">
                  <el-option v-for="o in fmtOptions" :key="o.v" :label="o.l" :value="o.v" />
                </el-select>
                <el-button size="small" @click="swapFmt">交换</el-button>
              </div>
              <div class="fmt-btns">
                <el-button size="small" type="primary" @click="doConfigConvert">转换</el-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 文本输出区 -->
    <div v-if="sourceType === 'text' || sourceType === 'unknown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-button size="small" @click="textOutputToInput" :disabled="!textOutput">转到输入</el-button>
          <el-button size="small" type="primary" @click="copy(textOutput)" :disabled="!textOutput">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="textOutput"
          type="textarea"
          :rows="10"
          readonly
          resize="vertical"
          :class="{ error: isError }"
        />
        <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
// ponytail: watch 用于监听 pendingHistoryRestore（历史记录还原）

import { ElMessage } from 'element-plus'
import { QuestionFilled, DocumentCopy, MagicStick } from '@element-plus/icons-vue'
import {
  blobToPureBase64,
  blobToDataUrl,
  dataUrlToImgTag,
  htmlToMarkdown,
  tsvToMarkdown,
  tsvToCsv,
  tsvToJson,
  textFormatConvert,
  selfCheck,
} from '@/utils/clipboardConvert'
import type { ConfigFormat } from '@/utils/xmlYamlUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const TOOL_NAME = 'clipboardConvert'

type SourceType = 'unknown' | 'text' | 'image'
const SOURCE_LABEL: Record<SourceType, string> = {
  unknown: '未读取',
  text: '文本',
  image: '图片',
}

// 状态
const sourceType = ref<SourceType>('unknown')
const sourceInfo = ref('')
const isError = ref(false)
const errorMessage = ref('')

// 图片数据
const lastImageBlob = ref<Blob | null>(null)
const lastDataUrl = ref('')
const imgOutput = ref('')

// 文本数据
const textInput = ref('')
const textOutput = ref('')

// 配置互转
type Fmt = ConfigFormat
const fmtOptions: { v: Fmt; l: string }[] = [
  { v: 'json', l: 'JSON' },
  { v: 'yaml', l: 'YAML' },
  { v: 'toml', l: 'TOML' },
  { v: 'ini', l: 'INI' },
  { v: 'properties', l: 'Java Properties' },
]
const srcFmt = ref<Fmt>('json')
const dstFmt = ref<Fmt>('toml')

// 自动识别提示
const detectedHint = computed(() => {
  const t = textInput.value.trim()
  if (!t) return ''
  if (/^\s*</.test(t) && /<[a-zA-Z][\s\S]*?>/g.test(t)) return 'HTML（建议转 Markdown）'
  // 2 行以上且每行包含 Tab
  const lines = t.split(/\r?\n/)
  if (lines.length >= 2 && lines.slice(0, 3).every(l => l.includes('\t'))) {
    return 'Tab 分隔表格（建议转 Markdown/CSV/JSON）'
  }
  const first = t[0]
  if (first === '{' || first === '[') return 'JSON'
  if (/^\w+:/.test(t) && (t.includes('\n- ') || t.includes('\n  '))) return 'YAML'
  if (t.includes(' = ') && /^\[[\w.]+\]/.test(t)) return 'INI'
  if (/^[A-Za-z_][\w.]*\s*[=:]\s*[^=\n]+$/.test(t.split('\n')[0] || '') && !t.includes('{')) {
    return 'Java Properties'
  }
  return ''
})

// 工具函数
function setError(msg: string) {
  isError.value = !!msg
  errorMessage.value = msg
}
function clearError() { isError.value = false; errorMessage.value = '' }

async function copy(s: string) {
  if (s == null || s === '') return
  try {
    await navigator.clipboard.writeText(s)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败，请检查剪贴板权限')
  }
}

// 从剪贴板读取（核心）
async function readClipboard() {
  clearError()
  try {
    if (!navigator.clipboard || !navigator.clipboard.read) {
      throw new Error('当前浏览器/环境不支持 Clipboard API')
    }
    const items = await navigator.clipboard.read()
    // 优先处理图片
    for (const item of items) {
      const imageType = item.types.find(t => t.startsWith('image/'))
      if (imageType) {
        const blob = await item.getType(imageType)
        lastImageBlob.value = blob
        lastDataUrl.value = await blobToDataUrl(blob)
        sourceType.value = 'image'
        sourceInfo.value = `${imageType} · ${formatSize(blob.size)}`
        imgOutput.value = ''
        store.addHistory({
          tool: TOOL_NAME,
          action: '读取剪贴板图片',
          inputPreview: `${imageType} · ${formatSize(blob.size)}`,
          outputPreview: '(预览)',
          inputFull: imageType,
          outputFull: `size=${blob.size}`,
        })
        return
      }
    }
    // 回退到 readText（有些文本类型不通过 read 返回）
    const txt = await navigator.clipboard.readText()
    if (txt !== '') {
      // 先检测是否是图片 DataURL / Base64 文本
      const maybeImgBlob = tryParseImageText(txt)
      if (maybeImgBlob) {
        lastImageBlob.value = maybeImgBlob
        lastDataUrl.value = await blobToDataUrl(maybeImgBlob)
        sourceType.value = 'image'
        sourceInfo.value = `${maybeImgBlob.type || 'image/*'} · ${formatSize(maybeImgBlob.size)}（自 Base64 解析）`
        imgOutput.value = ''
        store.addHistory({
          tool: TOOL_NAME,
          action: '解析剪贴板 Base64 图片',
          inputPreview: `${maybeImgBlob.type || 'image/*'} · ${formatSize(maybeImgBlob.size)}`,
          outputPreview: '(预览)',
          inputFull: txt.slice(0, 200),
          outputFull: `size=${maybeImgBlob.size}`,
        })
        return
      }
      sourceType.value = 'text'
      sourceInfo.value = `${txt.length} 字符`
      textInput.value = txt
      store.addHistory({
        tool: TOOL_NAME,
        action: '读取剪贴板文本',
        inputPreview: previewText(txt),
        outputPreview: '',
        inputFull: txt,
        outputFull: '',
      })
      return
    }
    throw new Error('剪贴板为空或暂不支持的内容类型')
  } catch (e: any) {
    setError(e.message || '读取剪贴板失败')
    sourceType.value = 'unknown'
    sourceInfo.value = ''
  }
}

// 文本直接粘贴到输入框（Clipboard API 失败时的后备）
async function pasteToTextInput() {
  try {
    const txt = await navigator.clipboard.readText()
    if (txt == null || txt === '') {
      ElMessage.warning('剪贴板没有文本内容')
      return
    }
    textInput.value = txt
    sourceType.value = 'text'
    sourceInfo.value = `${txt.length} 字符（手动粘贴）`
  } catch (e: any) {
    ElMessage.error('粘贴失败：' + (e.message || ''))
  }
}

function clearAll() {
  sourceType.value = 'unknown'
  sourceInfo.value = ''
  lastImageBlob.value = null
  lastDataUrl.value = ''
  imgOutput.value = ''
  textInput.value = ''
  textOutput.value = ''
  clearError()
}

// ============ 图片转换 ============
async function imgToBase64(includePrefix: boolean) {
  if (!lastImageBlob.value) { ElMessage.warning('先读取剪贴板图片'); return }
  clearError()
  try {
    const pure = await blobToPureBase64(lastImageBlob.value)
    const out = includePrefix ? lastDataUrl.value : pure
    imgOutput.value = out
    store.addHistory({
      tool: TOOL_NAME,
      action: includePrefix ? '图片 → DataURL' : '图片 → 纯 Base64',
      inputPreview: SOURCE_LABEL.image,
      outputPreview: previewText(out),
      inputFull: lastDataUrl.value,
      outputFull: out,
    })
  } catch (e: any) { setError(e.message) }
}

async function imgToImgTag() {
  if (!lastDataUrl.value) { ElMessage.warning('先读取剪贴板图片'); return }
  clearError()
  const out = dataUrlToImgTag(lastDataUrl.value, { alt: 'from-clipboard' })
  imgOutput.value = out
  store.addHistory({
    tool: TOOL_NAME,
    action: '图片 → <img>标签',
    inputPreview: SOURCE_LABEL.image,
    outputPreview: previewText(out),
    inputFull: lastDataUrl.value,
    outputFull: out,
  })
}

async function imgToEncodedUrl() {
  if (!lastDataUrl.value) { ElMessage.warning('先读取剪贴板图片'); return }
  clearError()
  const out = encodeURI(lastDataUrl.value)
  imgOutput.value = out
  ElMessage.info('注意：URL 编码后的 DataURL 体积更大，一般仅用于 CSS url(...) 特殊场景')
  store.addHistory({
    tool: TOOL_NAME,
    action: '图片 → URL编码DataURL',
    inputPreview: SOURCE_LABEL.image,
    outputPreview: previewText(out),
    inputFull: lastDataUrl.value,
    outputFull: out,
  })
}

// ============ 文本转换 ============
function requireTextInput(): string {
  const t = textInput.value
  if (!t.trim()) throw new Error('输入为空')
  return t
}

function doHtmlToMd() {
  clearError()
  try {
    const t = requireTextInput()
    const out = htmlToMarkdown(t)
    textOutput.value = out
    store.addHistory({
      tool: TOOL_NAME,
      action: 'HTML → Markdown',
      inputPreview: previewText(t),
      outputPreview: previewText(out),
      inputFull: t,
      outputFull: out,
    })
  } catch (e: any) { setError(e.message) }
}
function doHtmlToMdAndCopy() {
  doHtmlToMd()
  if (!isError.value && textOutput.value) copy(textOutput.value)
}

function doTsvTo(dst: 'md' | 'csv' | 'json') {
  clearError()
  try {
    const t = requireTextInput()
    let out = ''
    let label = ''
    if (dst === 'md') { out = tsvToMarkdown(t); label = 'TSV → Markdown 表格' }
    if (dst === 'csv') { out = tsvToCsv(t); label = 'TSV → CSV' }
    if (dst === 'json') { out = tsvToJson(t); label = 'TSV → JSON' }
    textOutput.value = out
    store.addHistory({
      tool: TOOL_NAME,
      action: label,
      inputPreview: previewText(t),
      outputPreview: previewText(out),
      inputFull: t,
      outputFull: out,
    })
  } catch (e: any) { setError(e.message) }
}

function swapFmt() {
  const a = srcFmt.value; srcFmt.value = dstFmt.value; dstFmt.value = a
  // 同步互换输入输出
  const t = textInput.value; textInput.value = textOutput.value; textOutput.value = t
}

function doConfigConvert() {
  clearError()
  try {
    const t = requireTextInput()
    const out = textFormatConvert(t, srcFmt.value, dstFmt.value)
    textOutput.value = out
    store.addHistory({
      tool: TOOL_NAME,
      action: `配置互转 ${srcFmt.value.toUpperCase()} → ${dstFmt.value.toUpperCase()}`,
      inputPreview: previewText(t),
      outputPreview: previewText(out),
      inputFull: `${srcFmt.value}\n${t}`,
      outputFull: `${dstFmt.value}\n${out}`,
    })
  } catch (e: any) { setError(e.message) }
}

function textOutputToInput() {
  if (!textOutput.value) return
  textInput.value = textOutput.value
  textOutput.value = ''
}

// ============ 变量池 ============
function insertVariable(varName: string, target: 'imgOut' | 'textIn') {
  const tag = `{{${varName}}}`
  if (target === 'textIn') textInput.value += tag
  else imgOutput.value += tag
}

// ============ 通用 ============
function formatSize(n: number): string {
  if (n < 1024) return n + ' B'
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB'
  return (n / 1024 / 1024).toFixed(2) + ' MB'
}
function previewText(s: string, n = 60): string {
  const t = (s || '').replace(/\s+/g, ' ').trim()
  return t.length > n ? t.slice(0, n) + '…' : t
}

/**
 * 尝试把文本识别为图片 DataURL / 纯 Base64
 * 成功返回 Blob，失败返回 null
 */
function tryParseImageText(txt: string): Blob | null {
  const s = (txt || '').trim()
  if (!s) return null
  try {
    // 情况 1：标准 DataURL 前缀 data:image/xxx;base64,
    const prefixMatch = s.match(/^data:(image\/[a-zA-Z0-9.+-]+);base64,/)
    if (prefixMatch) {
      const mime = prefixMatch[1]
      const b64 = s.slice(prefixMatch[0].length)
      const bin = atob(b64)
      const bytes = new Uint8Array(bin.length)
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
      return new Blob([bytes], { type: mime })
    }
    // 情况 2：无前缀的纯 Base64 —— 长度足够且字符合法，尝试解码并用 PNG/JPEG 签名判断
    // 只处理长度 >= 100 的字符串，避免误判普通文本
    if (s.length >= 100 && /^[A-Za-z0-9+/=\s]+$/.test(s)) {
      const compact = s.replace(/\s+/g, '')
      // 标准 base64 字符集
      if (compact.length % 4 !== 0) return null
      try {
        const bin = atob(compact)
        if (bin.length < 8) return null
        // PNG: 89 50 4E 47 (‰PNG)  JPEG: FF D8 FF  GIF: 47 49 46 38  WEBP: RIFF + WEBP  BMP: BM
        const h0 = bin.charCodeAt(0), h1 = bin.charCodeAt(1), h2 = bin.charCodeAt(2), h3 = bin.charCodeAt(3)
        let mime = ''
        if (h0 === 0x89 && h1 === 0x50 && h2 === 0x4E && h3 === 0x47) mime = 'image/png'
        else if (h0 === 0xFF && h1 === 0xD8 && h2 === 0xFF) mime = 'image/jpeg'
        else if (h0 === 0x47 && h1 === 0x49 && h2 === 0x46 && h3 === 0x38) mime = 'image/gif'
        else if (h0 === 0x42 && h1 === 0x4D) mime = 'image/bmp'
        else if (h0 === 0x52 && h1 === 0x49 && h2 === 0x46 && h3 === 0x46
          && bin.length >= 12
          && bin.charCodeAt(8) === 0x57 && bin.charCodeAt(9) === 0x45 && bin.charCodeAt(10) === 0x42 && bin.charCodeAt(11) === 0x50) {
          mime = 'image/webp'
        }
        if (!mime) return null
        const bytes = new Uint8Array(bin.length)
        for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i)
        return new Blob([bytes], { type: mime })
      } catch {
        return null
      }
    }
    return null
  } catch {
    return null
  }
}

// ============ 历史记录还原 ============
let isRestoring = false
let restoreTimer: ReturnType<typeof setTimeout> | null = null
const blockHistory = () => {
  isRestoring = true
  // 读一次变量避免 TS6133（剪贴板转换是手动触发，isRestoring 暂无消费方，保留用于未来可能的自动 watch 扩展）
  void isRestoring
  if (restoreTimer) clearTimeout(restoreTimer)
  restoreTimer = setTimeout(() => { isRestoring = false }, 500)
}

watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== TOOL_NAME) return
  blockHistory()
  const act = restore.action || ''
  const inputStr = restore.input || ''
  const outputStr = restore.output || ''
  // 还原输入，不自动执行
  if (act.includes('图片') && inputStr.startsWith('data:image')) {
    lastDataUrl.value = inputStr
    sourceType.value = 'image'
    sourceInfo.value = '(从历史还原)'
  } else {
    // ponytail: 配置互转在 action 标签里保留了格式前缀，不需要额外拆 input，直接使用
    textInput.value = inputStr
    sourceType.value = 'text'
    sourceInfo.value = '(从历史还原)'
  }
  textOutput.value = outputStr
  store.clearHistoryRestore()
})

// 自检（开发时在 Console 验证通过）
try { selfCheck() } catch { /* dev only */ }

// 粘贴自动读取：监听 paste 事件，聚焦时从剪贴板自动读
let pasteDebounce: number | undefined
const onDocPaste = () => {
  // ponytail: 当用户在应用内按 Ctrl+V 时顺便触发一次剪贴板读取（如果输入区还没填）
  // 避免和原生 paste 抢内容，加 300ms 防抖
  clearTimeout(pasteDebounce)
  pasteDebounce = window.setTimeout(() => {
    if (!textInput.value && sourceType.value !== 'image') {
      navigator.clipboard?.readText?.().then(txt => {
        if (txt && !textInput.value) {
          textInput.value = txt
          sourceType.value = 'text'
          sourceInfo.value = `${txt.length} 字符（Ctrl+V）`
        }
      })
    }
  }, 300)
}
onMounted(() => document.addEventListener('paste', onDocPaste))
onUnmounted(() => document.removeEventListener('paste', onDocPaste))
</script>

<style scoped>
.source-info {
  display: flex;
  align-items: center;
  gap: 12px;
}
.source-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 12px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  text-transform: uppercase;
  border: 1px solid;
}
.source-badge.type-unknown {
  color: var(--text-secondary);
  border-color: var(--border-color);
  background: var(--bg-input);
}
.source-badge.type-text {
  color: #00d4ff;
  border-color: #00d4ff44;
  background: #00d4ff11;
}
.source-badge.type-image {
  color: #10b981;
  border-color: #10b98144;
  background: #10b98111;
}
.source-meta {
  font-size: 13px;
  color: var(--text-secondary);
}

.preview-row {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}
.preview-img-wrap {
  flex: 0 0 220px;
  height: 180px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  overflow: hidden;
}
.preview-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.empty-img {
  color: var(--text-muted);
  font-size: 13px;
}
.img-actions {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detected-hint {
  margin-top: 10px;
  padding: 8px 12px;
  background: #8b5cf611;
  border: 1px solid #8b5cf644;
  color: #a78bfa;
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.format-pickers {
  display: flex;
  flex-wrap: wrap; /* 小窗口下自动换行，不强行挤一行 */
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.format-pickers .el-select { flex: 1; min-width: 100px; }
.fmt-arrow {
  color: var(--accent-cyan);
  font-weight: 700;
}
.fmt-btns {
  display: flex;
  gap: 6px;
}

.card-subtitle {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 8px;
}

/* 彩色分组卡片 */
.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: stretch;
}
.action-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--group-color, var(--accent-cyan));
  border-radius: 6px;
  min-width: 120px;
  flex: 1 1 0; /* 三个功能块平分一行 */
}
.group-label {
  font-size: 12px;
  color: var(--group-color, var(--text-secondary));
  font-weight: 600;
  letter-spacing: 0.5px;
}
.group-buttons {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.group-buttons :deep(.el-button) {
  border-color: var(--group-color, var(--border-color));
  color: var(--text-primary);
  background: transparent;
}
.group-buttons :deep(.el-button:hover) {
  border-color: var(--group-color, var(--accent-cyan));
  color: var(--group-color, var(--accent-cyan));
  background: color-mix(in srgb, var(--group-color, var(--accent-cyan)) 5%, transparent);
}
</style>
