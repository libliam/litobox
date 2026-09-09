<template>
  <div class="tool-container">
    <!-- 文件选择栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">日志实时查看</span>
        <div class="card-actions">
          <el-input
            v-model="filePath"
            placeholder="选择或拖入日志文件..."
            readonly
            style="width: 340px"
            @dragover.prevent
            @drop="handleDrop"
          >
            <template #append>
              <el-button @click="pickFile">选择</el-button>
            </template>
          </el-input>
          <el-button
            v-if="filePath && !isWatching"
            type="primary"
            size="small"
            @click="startWatching"
          >开始监控</el-button>
          <el-button
            v-if="filePath && isWatching"
            type="warning"
            size="small"
            @click="stopWatching"
          >停止监控</el-button>
        </div>
      </div>
    </div>

    <!-- 工具栏 -->
    <div v-if="filePath" class="tool-card">
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">过滤</div>
            <div class="filter-row">
              <el-input
                v-model="filterKeyword"
                size="small"
                placeholder="多关键词用空格或逗号分隔"
                clearable
                style="width: 200px"
              >
                <template #suffix>
                  <el-icon v-if="isProcessing" class="is-loading"><Loading /></el-icon>
                </template>
              </el-input>
              <el-radio-group v-model="filterMode" size="small">
                <el-radio-button value="and">与</el-radio-button>
                <el-radio-button value="or">或</el-radio-button>
              </el-radio-group>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">高亮</div>
            <el-input
              v-model="highlightKeyword"
              size="small"
              placeholder="多关键词用空格或逗号分隔"
              clearable
              style="width: 220px"
            >
              <template #suffix>
                <el-icon v-if="isProcessing" class="is-loading"><Loading /></el-icon>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">视图</div>
            <div class="group-buttons">
              <el-select v-model="logTheme" size="small" style="width: 110px">
                <el-option label="跟随系统" value="auto" />
                <el-option label="深色" value="dark" />
                <el-option label="浅色" value="light" />
                <el-option label="护眼" value="eye" />
              </el-select>
            </div>
          </div>
          <div class="action-group switch-group">
            <label class="switch-label">
              <span>自动滚动</span>
              <el-switch v-model="autoScroll" size="small" />
            </label>
            <label class="switch-label">
              <span>区分大小写</span>
              <el-switch v-model="caseSensitive" size="small" />
            </label>
            <label class="switch-label">
              <span>自动换行</span>
              <el-switch v-model="wordWrap" size="small" />
            </label>
          </div>
          <div class="action-group">
            <div class="group-label">操作</div>
            <div class="group-buttons">
              <el-button size="small" @click="scrollToBottom">到底部</el-button>
              <el-button size="small" @click="clearDisplay">清空</el-button>
              <el-button size="small" @click="reloadFile" :loading="reloading">重新加载</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 日志显示区 -->
    <div v-if="filePath" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ fileName }}</span>
        <div class="card-actions">
          <span class="log-meta">{{ filteredLines.length }} / {{ lines.length }} 行</span>
          <el-tag v-if="isWatching" type="success" size="small">监控中</el-tag>
          <el-tag v-else type="info" size="small">已停止</el-tag>
        </div>
      </div>
      <div class="card-body">
        <div
          ref="logContainerRef"
          class="log-container"
          :style="logStyle"
          @scroll="onScroll"
          @dragover.prevent
          @drop="handleDrop"
        >
          <div class="log-scroll-spacer" :style="{ height: totalHeight + 'px' }">
            <div class="log-viewport" :style="{ transform: `translateY(${offsetY}px)` }">
              <template v-for="line in visibleLines" :key="line.num">
                <div class="log-line">
                  <span class="line-number">{{ line.num }}</span>
                  <span class="line-text" v-html="line.html"></span>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="!filePath" class="tool-card">
      <div class="card-body">
        <el-empty description="选择或拖入日志文件开始查看">
          <el-button type="primary" @click="pickFile">选择文件</el-button>
        </el-empty>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { Loading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const filePath = ref('')
const lines = ref<{ num: number; text: string }[]>([])
const isWatching = ref(false)
const reloading = ref(false)
const logContainerRef = ref<HTMLElement>()

const filterKeyword = ref('')
const highlightKeyword = ref('')
const autoScroll = ref(true)
const caseSensitive = ref(false)
const wordWrap = ref(true)
const filterMode = ref<'and' | 'or'>('or')
const logTheme = ref<'auto' | 'dark' | 'light' | 'eye'>('auto')

// 防抖生效值（输入停止 400ms 后才更新，避免实时过滤卡顿）
const effectiveFilterKeyword = ref('')
const effectiveHighlightKeyword = ref('')
const isProcessing = ref(false)
let filterTimer: ReturnType<typeof setTimeout> | null = null
let highlightTimer: ReturnType<typeof setTimeout> | null = null

const DEBOUNCE_MS = 400

watch(filterKeyword, (val) => {
  if (filterTimer) clearTimeout(filterTimer)
  isProcessing.value = true
  filterTimer = setTimeout(() => {
    effectiveFilterKeyword.value = val
    isProcessing.value = false
  }, DEBOUNCE_MS)
})

watch(highlightKeyword, (val) => {
  if (highlightTimer) clearTimeout(highlightTimer)
  isProcessing.value = true
  highlightTimer = setTimeout(() => {
    effectiveHighlightKeyword.value = val
    isProcessing.value = false
  }, DEBOUNCE_MS)
})

let unlistenNew: UnlistenFn | null = null
let unlistenTruncated: UnlistenFn | null = null
let lineCounter = 0

// 最多保留行数，防止内存无限增长
const MAX_LINES = 50000

// 高亮色板：每个关键词分配一个不同颜色
const HIGHLIGHT_COLORS = [
  '#fde68a', // 黄
  '#67e8f9', // 青
  '#f9a8d4', // 粉
  '#86efac', // 绿
  '#fdba74', // 橙
  '#c4b5fd', // 紫
  '#fca5a5', // 红
  '#93c5fd', // 蓝
]

// 日志区主题样式（跟随系统 / 深色 / 浅色 / 护眼）
const logStyle = computed(() => {
  const wrap = wordWrap.value
  const base = {
    whiteSpace: wrap ? ('pre-wrap' as const) : ('pre' as const),
    overflowX: wrap ? ('hidden' as const) : ('auto' as const),
    wordBreak: wrap ? ('break-all' as const) : ('normal' as const),
  }
  if (logTheme.value === 'dark') {
    return {
      ...base,
      background: '#1e1e2e',
      color: '#cdd6f4',
      '--ln-color': '#6c7086',
      '--ln-border': '#313244',
      '--mark-bg': '#f9e2af',
      '--mark-color': '#1e1e2e',
    } as Record<string, string>
  }
  if (logTheme.value === 'light') {
    return {
      ...base,
      background: '#f8fafc',
      color: '#0f172a',
      '--ln-color': '#94a3b8',
      '--ln-border': '#e2e8f0',
      '--mark-bg': '#fde68a',
      '--mark-color': '#0f172a',
    } as Record<string, string>
  }
  if (logTheme.value === 'eye') {
    // 绿豆沙护眼背景
    return {
      ...base,
      background: '#C7EDCC',
      color: '#1a3a2a',
      '--ln-color': '#5a8a6a',
      '--ln-border': '#a8d8b0',
      '--mark-bg': '#f9e2af',
      '--mark-color': '#1a3a2a',
    } as Record<string, string>
  }
  // auto：跟随系统主题
  return {
    ...base,
    background: 'var(--bg-input)',
    color: 'var(--text-primary)',
    '--ln-color': 'var(--text-muted)',
    '--ln-border': 'var(--border-color)',
    '--mark-bg': '#fde68a',
    '--mark-color': '#0f172a',
  } as Record<string, string>
})

const fileName = computed(() => {
  if (!filePath.value) return ''
  const parts = filePath.value.split(/[\\/]/)
  return parts[parts.length - 1] || filePath.value
})

// 解析多关键词：按空格、逗号（中英文）分隔
function parseKeywords(s: string): string[] {
  return s.split(/[\s,，]+/).map(k => k.trim()).filter(Boolean)
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

// 多关键词高亮，每个关键词用不同颜色
function highlightText(text: string, keywords: string[]): string {
  if (!keywords.length) return escapeHtml(text)
  const flags = caseSensitive.value ? 'g' : 'gi'
  const escaped = keywords.map(k => k.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
  const re = new RegExp(escaped.join('|'), flags)
  const parts: string[] = []
  let lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = re.exec(text)) !== null) {
    parts.push(escapeHtml(text.slice(lastIndex, match.index)))
    const matchedText = match[0]
    // 找到匹配的关键词索引（用于分配颜色）
    let kwIdx = 0
    for (let i = 0; i < keywords.length; i++) {
      const kw = keywords[i]
      if (caseSensitive.value ? kw === matchedText : kw.toLowerCase() === matchedText.toLowerCase()) {
        kwIdx = i
        break
      }
    }
    parts.push(
      `<mark style="background:${HIGHLIGHT_COLORS[kwIdx % HIGHLIGHT_COLORS.length]};color:#1e1e2e;border-radius:2px;padding:0 2px">${escapeHtml(matchedText)}</mark>`
    )
    lastIndex = match.index + matchedText.length
    if (matchedText === '') re.lastIndex++ // 防止空匹配死循环
  }
  parts.push(escapeHtml(text.slice(lastIndex)))
  return parts.join('')
}

// 过滤后的行（不做高亮，高亮只在可视行上做以提升性能）
const filteredLines = computed(() => {
  const filterKeywords = parseKeywords(effectiveFilterKeyword.value)

  let result = lines.value

  // 多关键词过滤：与 / 或
  if (filterKeywords.length) {
    const cs = caseSensitive.value
    const kws = cs ? filterKeywords : filterKeywords.map(k => k.toLowerCase())
    result = result.filter(l => {
      const text = cs ? l.text : l.text.toLowerCase()
      if (filterMode.value === 'and') {
        return kws.every(k => text.includes(k))
      }
      return kws.some(k => text.includes(k))
    })
  }

  return result
})

// 需要高亮的关键词 = 过滤词 + 高亮词（去重，保持顺序）
const allHighlightKeywords = computed(() => {
  const seen = new Set<string>()
  const result: string[] = []
  const cs = caseSensitive.value
  for (const k of [...parseKeywords(effectiveFilterKeyword.value), ...parseKeywords(effectiveHighlightKeyword.value)]) {
    const key = cs ? k : k.toLowerCase()
    if (!seen.has(key)) {
      seen.add(key)
      result.push(k)
    }
  }
  return result
})

// ============ 虚拟滚动 ============
const scrollTop = ref(0)
const ROW_HEIGHT = 20      // 单行高度（px），对应 font-size 13 * line-height 1.5 ≈ 19.5
const BUFFER = 30          // 上下缓冲行数
const CHAR_WIDTH = 7.2     // 等宽字体单字符宽度估算（px）

// 估算单行高度（换行模式下一行可能折成多行）
function estimateLineHeight(text: string): number {
  if (!wordWrap.value) return ROW_HEIGHT
  const containerWidth = logContainerRef.value?.clientWidth ?? 800
  const usableWidth = Math.max(100, containerWidth - 80) // 减去行号列宽度
  const charsPerLine = Math.max(1, Math.floor(usableWidth / CHAR_WIDTH))
  const wrappedLines = Math.max(1, Math.ceil((text.length || 1) / charsPerLine))
  return wrappedLines * ROW_HEIGHT
}

// 每行累计高度（前缀和），仅换行模式下需要
const lineHeights = computed<number[] | null>(() => {
  if (!wordWrap.value) return null
  const prefix = [0]
  let sum = 0
  for (const l of filteredLines.value) {
    sum += estimateLineHeight(l.text)
    prefix.push(sum)
  }
  return prefix
})

const totalHeight = computed(() => {
  const lh = lineHeights.value
  if (lh) return lh[lh.length - 1]
  return filteredLines.value.length * ROW_HEIGHT
})

// 可视行范围
const visibleRange = computed(() => {
  const scroll = scrollTop.value
  const lh = lineHeights.value
  if (lh) {
    // 换行模式：通过前缀和二分查找起始行
    let lo = 0, hi = lh.length - 1
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1
      if (lh[mid] <= scroll) lo = mid
      else hi = mid - 1
    }
    const start = Math.max(0, lo - BUFFER)
    const viewBottom = scroll + 600 + BUFFER * ROW_HEIGHT
    let end = start
    while (end < filteredLines.value.length && lh[end + 1] < viewBottom) end++
    return { start, end: Math.min(filteredLines.value.length, end + BUFFER) }
  }
  // 不换行模式：固定行高
  const start = Math.max(0, Math.floor(scroll / ROW_HEIGHT) - BUFFER)
  const count = Math.ceil(600 / ROW_HEIGHT) + BUFFER * 2
  return { start, end: Math.min(filteredLines.value.length, start + count) }
})

const offsetY = computed(() => {
  const lh = lineHeights.value
  if (lh) return lh[visibleRange.value.start]
  return visibleRange.value.start * ROW_HEIGHT
})

// 只对可视行做高亮，大幅减少正则处理量
const visibleLines = computed(() => {
  const { start, end } = visibleRange.value
  const kws = allHighlightKeywords.value
  return filteredLines.value.slice(start, end).map(l => ({
    num: l.num,
    html: highlightText(l.text, kws),
  }))
})

function onScroll(e: Event) {
  scrollTop.value = (e.target as HTMLElement).scrollTop
}

function appendText(text: string) {
  if (!text) return
  const newLines = text.split('\n')
  // 处理末尾空行（文件以 \n 结尾时会产生空字符串）
  if (newLines.length > 1 && newLines[newLines.length - 1] === '') {
    newLines.pop()
  }
  for (const line of newLines) {
    lineCounter++
    lines.value.push({ num: lineCounter, text: line })
  }
  // 超过上限时裁剪旧行
  if (lines.value.length > MAX_LINES) {
    lines.value.splice(0, lines.value.length - MAX_LINES)
  }
  if (autoScroll.value) {
    nextTick(() => {
      if (logContainerRef.value) {
        logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
      }
    })
  }
}

async function pickFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: '日志文件', extensions: ['log', 'txt', 'out', 'err'] }, { name: '所有文件', extensions: ['*'] }],
  })
  if (selected) {
    openFile(selected as string)
  }
}

function handleDrop(e: DragEvent) {
  e.preventDefault()
  const file = e.dataTransfer?.files?.[0]
  if (file) {
    // Tauri 拖入的 File 对象带有 path 属性
    const path = (file as any).path as string
    if (path) openFile(path)
  }
}

async function openFile(path: string) {
  // 关闭之前的监控
  if (isWatching.value) {
    await stopWatching()
  }
  filePath.value = path
  lines.value = []
  lineCounter = 0

  try {
    const content = await invoke<string>('log_viewer_open', { filePath: path })
    appendText(content)
    isWatching.value = true

    store.addHistory({
      tool: 'logViewer',
      action: '查看日志',
      inputPreview: fileName.value,
      outputPreview: `${lines.value.length} 行`,
      inputFull: path,
      outputFull: content.slice(0, 2000),
    })

    // 监听新增行事件
    unlistenNew = await listen('log-viewer://new-lines', (event) => {
      const payload = event.payload as { content: string }
      appendText(payload.content)
    })

    // 监听文件截断事件
    unlistenTruncated = await listen('log-viewer://truncated', (event) => {
      const payload = event.payload as { content: string }
      lines.value = []
      lineCounter = 0
      appendText(payload.content)
      ElMessage.warning('日志文件已被截断/轮转，已重新加载')
    })
  } catch (e) {
    ElMessage.error('打开文件失败: ' + String(e))
    filePath.value = ''
  }
}

async function startWatching() {
  if (!filePath.value) return
  try {
    const content = await invoke<string>('log_viewer_open', { filePath: filePath.value })
    lines.value = []
    lineCounter = 0
    appendText(content)
    isWatching.value = true

    unlistenNew = await listen('log-viewer://new-lines', (event) => {
      const payload = event.payload as { content: string }
      appendText(payload.content)
    })

    unlistenTruncated = await listen('log-viewer://truncated', (event) => {
      const payload = event.payload as { content: string }
      lines.value = []
      lineCounter = 0
      appendText(payload.content)
      ElMessage.warning('日志文件已被截断/轮转，已重新加载')
    })
  } catch (e) {
    ElMessage.error('启动监控失败: ' + String(e))
  }
}

async function stopWatching() {
  try {
    await invoke('log_viewer_close')
  } catch {
    // ignore
  }
  isWatching.value = false
  if (unlistenNew) {
    unlistenNew()
    unlistenNew = null
  }
  if (unlistenTruncated) {
    unlistenTruncated()
    unlistenTruncated = null
  }
}

function clearDisplay() {
  lines.value = []
  lineCounter = 0
}

function scrollToBottom() {
  if (logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

async function reloadFile() {
  if (!filePath.value) return
  reloading.value = true
  try {
    const content = await invoke<string>('log_viewer_reload', { filePath: filePath.value })
    lines.value = []
    lineCounter = 0
    appendText(content)
    ElMessage.success('已重新加载')
  } catch (e) {
    ElMessage.error('重新加载失败: ' + String(e))
  } finally {
    reloading.value = false
  }
}

// 自动滚动开关变化时，如果开启则立即滚到底部
watch(autoScroll, (val) => {
  if (val && logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
})

onUnmounted(() => {
  if (isWatching.value) {
    invoke('log_viewer_close').catch(() => {})
  }
  if (unlistenNew) unlistenNew()
  if (unlistenTruncated) unlistenTruncated()
  if (filterTimer) clearTimeout(filterTimer)
  if (highlightTimer) clearTimeout(highlightTimer)
})
</script>

<style scoped>
.log-container {
  height: 600px;
  overflow: auto;
  position: relative;
  border-radius: 6px;
  padding: 12px;
  font-family: 'Consolas', 'Courier New', 'Cascadia Code', monospace;
  font-size: 13px;
  line-height: 20px;
}

/* 虚拟滚动占位：撑出总高度 */
.log-scroll-spacer {
  position: relative;
  min-width: 100%;
}

/* 可视区：用 transform 偏移到当前滚动位置 */
.log-viewport {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}

.log-line {
  min-height: 20px;
  line-height: 20px;
  white-space: inherit;
  word-break: inherit;
}

.log-line .line-number {
  display: inline-block;
  width: 50px;
  text-align: right;
  color: var(--ln-color);
  user-select: none;
  padding-right: 12px;
  border-right: 1px solid var(--ln-border);
  margin-right: 12px;
  vertical-align: top;
}

.log-line .line-text {
  color: inherit;
}

:deep(.log-line .line-text mark) {
  /* 背景色由内联样式按关键词分配 */
  border-radius: 2px;
  padding: 0 2px;
}

/* 过滤行：输入框 + 与/或切换 */
.filter-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 开关组：文字 + 开关横向排列，清晰标识 */
.switch-group {
  flex-direction: row;
  align-items: center;
  gap: 16px;
}

.switch-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.switch-label span {
  user-select: none;
}

.log-meta {
  font-size: 12px;
  color: var(--text-muted);
  margin-right: 12px;
}
</style>
