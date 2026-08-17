<template>
  <div class="tool-container">
    <!-- 输入卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Mermaid 代码</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>输入 Mermaid 图表语法，实时渲染预览</p>
                <p>支持流程图、时序图、甘特图、类图、状态图等</p>
                <p>点击「模板」快速插入常用图表示例</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">模板</div>
            <el-select
              v-model="templateType"
              size="small"
              style="width: 140px"
              placeholder="插入模板"
              @change="handleTemplateChange"
            >
              <el-option
                v-for="t in MERMAID_TEMPLATES"
                :key="t.value"
                :label="t.label"
                :value="t.value"
              />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">主题</div>
            <el-select
              v-model="chartTheme"
              size="small"
              style="width: 160px"
              @change="handleStyleChange"
            >
              <el-option
                v-for="t in MERMAID_THEME_OPTIONS"
                :key="t.value"
                :label="t.label"
                :value="t.value"
              />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">字号</div>
            <el-select v-model="fontSize" size="small" style="width: 100px" @change="handleStyleChange">
              <el-option v-for="s in [12, 14, 16, 18, 20]" :key="s" :label="`${s}px`" :value="s" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">渲染</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :loading="rendering" @click="handleRender">
                <el-icon><VideoPlay /></el-icon>
                <span>渲染</span>
              </el-button>
            </div>
          </div>
        </div>

        <div class="editor-section">
          <div class="editor-wrapper">
            <textarea
              v-model="inputCode"
              class="mermaid-input"
              spellcheck="false"
              placeholder="flowchart TD&#10;  A[开始] --> B{判断}&#10;  B -- 是 --> C[执行]&#10;  B -- 否 --> D[结束]"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 预览卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">图表预览</span>
          <span v-if="renderTime" class="stat-text">渲染耗时 {{ renderTime }}ms</span>
        </div>
        <div class="card-actions">
          <div class="preview-ctrl">
            <span class="ctrl-label">背景</span>
            <el-select v-model="bgMode" size="small" style="width: 110px">
              <el-option v-for="b in BG_OPTIONS" :key="b.value" :label="b.label" :value="b.value" />
            </el-select>
          </div>
          <div class="preview-ctrl">
            <span class="ctrl-label">缩放</span>
            <el-input-number
              v-model="zoomPct"
              :min="25"
              :max="200"
              :step="25"
              size="small"
              style="width: 100px"
            />
          </div>
          <el-button size="small" :disabled="!renderedSvg" @click="handleExportSvg">
            <el-icon><Download /></el-icon>
            <span>导出 SVG</span>
          </el-button>
          <el-button size="small" type="primary" :disabled="!renderedSvg" @click="handleExportPng">
            <el-icon><Download /></el-icon>
            <span>导出 PNG</span>
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="error" class="mermaid-error">
          <el-icon><WarningFilled /></el-icon>
          <div class="mermaid-error-text">
            <p class="mermaid-error-title">渲染失败</p>
            <p class="mermaid-error-detail">{{ error }}</p>
          </div>
        </div>
        <div v-else-if="renderedSvg" class="preview-shell" :class="bgClass">
          <div class="preview-zoom" :style="{ zoom: zoomPct / 100 }">
            <div class="mermaid-preview" v-html="renderedSvg"></div>
          </div>
        </div>
        <div v-else class="empty-hint">
          <el-icon :size="48"><Share /></el-icon>
          <p>输入 Mermaid 代码后自动渲染，或点击「渲染」按钮</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, VideoPlay, Download, WarningFilled, Share } from '@element-plus/icons-vue'
import VariablePicker from '@/components/VariablePicker.vue'
import { useToolboxStore } from '@/store'
import type { HistoryRestoreState } from '@/store'
import { renderMermaid, fixExportSvg, MERMAID_TEMPLATES, MERMAID_THEME_OPTIONS, MERMAID_EXPORT_FONT } from '@/utils/mermaidUtils'
import { saveFileWithDialog } from '@/utils/fileSaver'

const store = useToolboxStore()

const inputCode = ref('')
const templateType = ref('')
const renderedSvg = ref('')
const error = ref('')
const rendering = ref(false)
const renderTime = ref<number | null>(null)
const chartTheme = ref('auto')
const fontSize = ref(16)
const bgMode = ref('follow')
const zoomPct = ref(100)

/** 预览背景模式 */
const BG_OPTIONS = [
  { label: '跟随主题', value: 'follow' },
  { label: '白色', value: 'white' },
  { label: '透明', value: 'transparent' },
]
const bgClass = computed(() => `bg-${bgMode.value}`)

/** 解析导出 PNG 的背景色 */
const pngBgColor = (): string | null => {
  switch (bgMode.value) {
    case 'white':
      return '#ffffff'
    case 'transparent':
      return null
    default:
      // 跟随应用主题
      return document.documentElement.classList.contains('light') ? '#ffffff' : '#0a0e17'
  }
}

// ============ 模板插入 ============
const handleTemplateChange = (val: string) => {
  const t = MERMAID_TEMPLATES.find(t => t.value === val)
  if (!t) return
  inputCode.value = t.code
  templateType.value = ''
  // 插入模板后立即渲染
  handleRender()
}

// ============ 渲染 ============
const doRender = async () => {
  const code = inputCode.value.trim()
  if (!code) {
    renderedSvg.value = ''
    error.value = ''
    renderTime.value = null
    return
  }
  rendering.value = true
  const start = performance.now()
  try {
    renderedSvg.value = await renderMermaid(code, { theme: chartTheme.value, fontSize: fontSize.value })
    error.value = ''
    renderTime.value = Math.round(performance.now() - start)
  } catch (e: any) {
    renderedSvg.value = ''
    renderTime.value = null
    error.value = extractError(e?.message || String(e))
  } finally {
    rendering.value = false
  }
}

// mermaid 错误信息包含解析错误对象，提取可读的 message
const extractError = (raw: string): string => {
  try {
    // 形如: ...{"message":"...","hash":"..."}...
    const m = raw.match(/\{"message":"([^"]+)"\}/)
    if (m) return m[1]
  } catch {}
  return raw.slice(0, 300)
}

const handleRender = () => {
  doRender()
}

// 主题/字号变更后重新渲染
const handleStyleChange = () => {
  if (inputCode.value.trim()) doRender()
}

// ============ 自动渲染（输入防抖 300ms） ============
let autoTimer: ReturnType<typeof setTimeout> | null = null
let isRestoringFromHistory = false
watch(inputCode, () => {
  if (isRestoringFromHistory) return
  if (autoTimer) clearTimeout(autoTimer)
  autoTimer = setTimeout(() => {
    doRender()
  }, 300)
})

// ============ 主题变化重新渲染（auto 跟随应用主题） ============
watch(() => store.config.theme, () => {
  if (chartTheme.value === 'auto' && renderedSvg.value) doRender()
})
// ============ 历史还原 ============
const restoreFromHistory = (data: HistoryRestoreState) => {
  isRestoringFromHistory = true
  inputCode.value = data.input
  if (data.output) {
    // 还原输出为 SVG 字符串时直接展示；否则触发重新渲染
    if (data.output.includes('<svg')) {
      renderedSvg.value = data.output
      error.value = ''
    } else {
      doRender()
    }
  } else {
    doRender()
  }
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
  setTimeout(() => {
    isRestoringFromHistory = false
  }, 500)
}

onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'mermaid') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  } else {
    // 首次进入自动载入流程图模板并渲染，
    // 避免 inputCode 为空导致导出按钮 disabled、点击无反应
    const t = MERMAID_TEMPLATES.find(t => t.value === 'flowchart')
    if (t) {
      inputCode.value = t.code
      doRender()
    }
  }
})

onUnmounted(() => {
  if (autoTimer) clearTimeout(autoTimer)
})

// ============ 通用操作 ============
const handleClear = () => {
  inputCode.value = ''
  renderedSvg.value = ''
  error.value = ''
  renderTime.value = null
}

const handlePaste = async () => {
  try {
    inputCode.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  inputCode.value = value
}

// ============ 导出 ============
// 导出专用渲染：htmlLabels: false 产出纯 SVG（无 foreignObject），
// 任意 SVG 查看器/本工具都能正常预览，且栅格化不被 canvas 判定为 tainted；
// 明确字体栈保证栅格化字体与布局测量字体一致，避免文字溢出节点框。
// 返回前经 fixExportSvg 后处理：补齐根 svg 实际尺寸（viewBox 像素值）、
// 给缺失 text-anchor 的节点标签补居中锚点，否则独立打开/栅格化时文字水平偏右。
const renderExportSvg = async (): Promise<string> => {
  const code = inputCode.value.trim()
  if (!code) throw new Error('没有可导出的图表内容')
  const svg = await renderMermaid(code, {
    theme: chartTheme.value,
    fontSize: fontSize.value,
    htmlLabels: false,
    fontFamily: MERMAID_EXPORT_FONT,
  })
  return fixExportSvg(svg)
}

const handleExportSvg = async () => {
  if (!inputCode.value.trim()) {
    ElMessage.warning('请先输入 Mermaid 代码')
    return
  }
  try {
    const exportSvg = await renderExportSvg()
    const blob = new Blob([exportSvg], { type: 'image/svg+xml;charset=utf-8' })
    const name = `mermaid_${Date.now()}.svg`
    const savedPath = await saveFileWithDialog(blob, name, 'svg')
    if (savedPath) {
      store.addHistory({
        tool: 'mermaid',
        action: '导出 SVG',
        inputPreview: inputCode.value.slice(0, 50),
        outputPreview: 'SVG 图表已导出',
        inputFull: inputCode.value,
        outputFull: exportSvg,
      })
    }
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e.message || e))
  }
}

// SVG → PNG 栅格化：导出前以 renderExportSvg() 重新渲染一版纯 SVG
// （htmlLabels: false + 明确字体），避免 foreignObject 污染 canvas 无法导出、
// 以及字体宽度不一致导致的文字溢出节点框。
const svgToPngBlob = async (): Promise<Blob> => {
  const exportSvg = await renderExportSvg()
  return new Promise((resolve, reject) => {
    const svgBlob = new Blob([exportSvg], { type: 'image/svg+xml;charset=utf-8' })
    const url = URL.createObjectURL(svgBlob)

    const img = new Image()
    img.onload = () => {
      // mermaid SVG 自带 viewBox/width/height，直接按 2x 放大导出保证清晰度
      let w = img.naturalWidth || 1200
      let h = img.naturalHeight || 800
      // 大图限制导出尺寸上限，避免 canvas 超限
      const MAX = 8000
      const scale = Math.min(2, MAX / Math.max(w, h))
      w = Math.round(w * scale)
      h = Math.round(h * scale)

      const canvas = document.createElement('canvas')
      canvas.width = w
      canvas.height = h
      const ctx = canvas.getContext('2d')!
      // 背景与预览区所选模式保持一致
      const bg = pngBgColor()
      if (bg) {
        ctx.fillStyle = bg
        ctx.fillRect(0, 0, w, h)
      }
      ctx.drawImage(img, 0, 0, w, h)
      URL.revokeObjectURL(url)

      canvas.toBlob((blob) => {
        if (blob) resolve(blob)
        else reject(new Error('PNG 生成失败'))
      }, 'image/png')
    }
    img.onerror = () => {
      URL.revokeObjectURL(url)
      reject(new Error('SVG 转 PNG 失败'))
    }
    img.src = url
  })
}

const handleExportPng = async () => {
  if (!inputCode.value.trim()) {
    ElMessage.warning('请先输入 Mermaid 代码')
    return
  }
  try {
    const blob = await svgToPngBlob()
    const name = `mermaid_${Date.now()}.png`
    const savedPath = await saveFileWithDialog(blob, name, 'png')
    if (savedPath) {
      store.addHistory({
        tool: 'mermaid',
        action: '导出 PNG',
        inputPreview: inputCode.value.slice(0, 50),
        outputPreview: 'PNG 图表已导出',
        inputFull: inputCode.value,
        outputFull: renderedSvg.value,
      })
    }
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e.message || e))
  }
}
</script>

<style scoped>
/* ===== 代码输入 ===== */
.editor-section {
  margin-top: 12px;
}
.editor-wrapper {
  position: relative;
}
.mermaid-input {
  width: 100%;
  min-height: 200px;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', Consolas, 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  transition: border-color 0.2s;
}
.mermaid-input:focus {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 0 2px rgba(0, 212, 255, 0.15);
}

/* ===== 预览区 ===== */
.preview-shell {
  overflow: auto;
  max-height: 600px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
  background: var(--bg-input);
}
/* 背景模式 */
.preview-shell.bg-white {
  background: #ffffff;
}
.preview-shell.bg-transparent {
  /* 透明棋盘格，便于观察导出后的透明背景 */
  background-image: linear-gradient(45deg, rgba(128, 128, 128, 0.12) 25%, transparent 25%, transparent 75%, rgba(128, 128, 128, 0.12) 75%),
    linear-gradient(45deg, rgba(128, 128, 128, 0.12) 25%, transparent 25%, transparent 75%, rgba(128, 128, 128, 0.12) 75%);
  background-size: 16px 16px;
  background-position: 0 0, 8px 8px;
}
.preview-zoom {
  transform-origin: top left;
}
.mermaid-preview :deep(svg) {
  max-width: 100%;
  height: auto;
}
.mermaid-preview :deep(.er.relationshipLine) {
  stroke: currentColor;
}

/* ===== 预览工具条控件 ===== */
.preview-ctrl {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-right: 12px;
}
.ctrl-label {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

/* ===== 错误提示（红色边框 + 发光） ===== */
.mermaid-error {
  display: flex;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid var(--accent-red);
  border-radius: 8px;
  box-shadow: 0 0 8px rgba(239, 68, 68, 0.15);
  background: var(--bg-input);
  color: var(--accent-red);
}
.mermaid-error .el-icon {
  font-size: 18px;
  margin-top: 2px;
}
.mermaid-error-text {
  font-size: 13px;
  line-height: 1.5;
}
.mermaid-error-title {
  font-weight: 600;
  margin-bottom: 4px;
}
.mermaid-error-detail {
  color: var(--text-secondary);
  word-break: break-all;
  white-space: pre-wrap;
}

/* ===== 空状态 ===== */
.empty-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 0;
  color: var(--text-secondary);
  gap: 12px;
}

/* ===== 统计文本 ===== */
.stat-text {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 8px;
}
</style>
