<template>
  <div class="tool-container">
    <!-- 1. 搜索配置卡片（sticky） -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">全文搜索</span>
        <div class="card-actions">
          <el-dropdown trigger="click" @command="applyHistoryItem">
            <el-button size="small">
              搜索历史<el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-for="(item, idx) in searchHistory"
                  :key="idx"
                  :command="item"
                  class="history-item"
                >
                  <div class="history-title">
                    <span class="history-query">{{ item.query }}</span>
                    <span class="history-mode">{{ item.mode === 'content' ? '内容' : '文件名' }}</span>
                  </div>
                  <div class="history-path">{{ item.path }}</div>
                </el-dropdown-item>
                <el-dropdown-item v-if="searchHistory.length === 0" disabled>
                  暂无搜索历史
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group action-group-flex-2">
            <div class="group-label">搜索路径</div>
            <el-input
              v-model="searchPath"
              placeholder="选择或输入要搜索的目录路径"
              size="small"
              clearable
            >
              <template #append>
                <el-button size="small" @click="selectFolder">浏览</el-button>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">模式</div>
            <el-radio-group v-model="opts.mode" size="small">
              <el-radio-button value="filename">文件名</el-radio-button>
              <el-radio-button value="content">内容</el-radio-button>
            </el-radio-group>
          </div>
        </div>
        <div class="action-grid action-grid-mt-sm">
          <div class="action-group action-group-flex-2">
            <div class="group-label">搜索词（正则）</div>
            <el-input
              v-model="opts.query"
              placeholder="例如 \d{4}-\d{2}-\d{2} 或 TODO"
              size="small"
              clearable
              @keyup.enter="startSearch"
            />
          </div>
          <div class="action-group">
            <div class="group-label">扩展名</div>
            <el-input
              v-model="extFilterText"
              placeholder="ts,js 或 !exe,dll"
              size="small"
            />
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!searchPath || !opts.query || searching"
                :loading="searching"
                @click="startSearch"
              >
                搜索
              </el-button>
            </div>
          </div>
        </div>
        <div class="action-grid action-grid-mt-sm">
          <el-checkbox v-model="opts.caseSensitive">区分大小写</el-checkbox>
          <el-checkbox v-model="opts.includeHidden">包含隐藏</el-checkbox>
          <div class="action-group" v-if="opts.mode === 'content'">
            <div class="group-label">内容最大文件</div>
            <el-input-number
              v-model="maxContentMb"
              :min="1"
              :max="500"
              size="small"
              controls-position="right"
              style="width: 110px"
            />
            <span class="hint">MB</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 2. 进度卡片 -->
    <div v-if="searching" class="tool-card">
      <div class="card-header">
        <span class="card-title">搜索中</span>
        <el-button size="small" type="danger" @click="cancelSearch">取消</el-button>
      </div>
      <div class="card-body">
        <div class="progress-info">
          <div>当前: {{ progress?.currentPath || '准备中...' }}</div>
          <div>
            已扫描 {{ progress?.filesScanned || 0 }} 文件 |
            命中 {{ progress?.matchesFound || 0 }} |
            耗时 {{ formatDuration(elapsedMs) }}
          </div>
        </div>
        <el-progress
          :percentage="100"
          :show-text="false"
          :stroke-width="14"
          stripe
          status="success"
          :indeterminate="true"
        />
      </div>
    </div>

    <!-- 3. 错误卡片 -->
    <div v-if="searchError" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ searchError }}</div>
      </div>
    </div>

    <!-- 4. 结果卡片 -->
    <div v-if="completed && summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">结果</span>
        <div class="card-actions">
          <span class="summary-text">
            {{ summary.matchesFound }} 命中 |
            {{ summary.totalFiles }} 文件 |
            耗时 {{ formatDuration(summary.durationMs) }}
            <span v-if="summary.truncated" class="warn-text">
              (已达上限 1000，结果截断)
            </span>
            <span v-if="summary.skippedCount > 0" class="warn-text">
              (跳过 {{ summary.skippedCount }} 个文件)
            </span>
          </span>
        </div>
      </div>
      <div class="card-body">
        <el-table
          :data="resultItems"
          stripe
          size="small"
          @row-dblclick="locateInExplorer"
        >
          <el-table-column label="文件" min-width="300">
            <template #default="{ row }">
              <div class="file-name"><strong>{{ row.name }}</strong></div>
              <div class="file-path">{{ row.path }}</div>
              <div class="file-meta">
                {{ formatBytes(row.sizeBytes) }} · {{ formatTime(row.modifiedMs) }}
              </div>
            </template>
          </el-table-column>
          <el-table-column
            v-if="opts.mode === 'content'"
            label="命中行"
            min-width="400"
          >
            <template #default="{ row }">
              <div
                v-for="ml in row.matchedLines"
                :key="ml.lineNumber"
                class="match-line"
              >
                <span class="line-no">L{{ ml.lineNumber }}:</span>
                <span class="line-text" v-html="highlightLine(ml)"></span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-button size="small" link @click="locateInExplorer(row)">
                定位
              </el-button>
            </template>
          </el-table-column>
        </el-table>
        <el-pagination
          v-if="totalResults > pageSize"
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="totalResults"
          layout="prev, pager, next, total"
          @current-change="loadResults"
          class="pagination-right"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { ArrowDown } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import {
  fileSearchStart,
  fileSearchCancel,
  fileSearchGetResults,
  fileSearchClear,
} from '@/utils/fileSearcherClient'
import type {
  SearchOptions,
  SearchProgress,
  SearchSummary,
  SearchResultItem,
  MatchedLine,
} from '@/utils/fileSearcherTypes'

// ============ 状态 ============
type SearchState = 'idle' | 'searching' | 'completed' | 'failed' | 'cancelled'
const state = ref<SearchState>('idle')
const searching = computed(() => state.value === 'searching')
const completed = computed(() => state.value === 'completed')

const searchPath = ref('')
const opts = reactive<SearchOptions>({
  mode: 'filename',
  query: '',
  caseSensitive: false,
  extensions: [],
  excludeExtensions: [],
  includeHidden: false,
  maxContentFileBytes: 10 * 1024 * 1024,
})
const extFilterText = ref('')
const maxContentMb = ref(10)

const searchId = ref('')
const progress = ref<SearchProgress | null>(null)
const summary = ref<SearchSummary | null>(null)
const searchError = ref('')
const elapsedMs = ref(0)
const resultItems = ref<SearchResultItem[]>([])
const totalResults = ref(0)
const currentPage = ref(1)
const pageSize = 100

let timerId: ReturnType<typeof setInterval> | null = null
let unlistenFns: UnlistenFn[] = []
const startTime = ref(0)

// ============ 工具函数 ============
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  return `${m}m${s % 60}s`
}

function formatTime(ms: number): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function highlightLine(ml: MatchedLine): string {
  // 反向应用 matchRanges 以避免偏移变化（先替换后面的）
  const sorted = [...ml.matchRanges].sort((a, b) => b[0] - a[0])
  let result = ''
  // 转成数组操作字符
  // ponytail: escapeHtml 后偏移会变，这里简化为对原始 lineText 做标记再 escape
  // 改用：对原始文本按 ranges 反向插入 <mark> 标签，再整体 escape（标记不转义）
  const origChars = Array.from(ml.lineText)
  for (const [start, end] of sorted) {
    origChars.splice(end, 0, ...Array.from('</mark>'))
    origChars.splice(start, 0, ...Array.from('<mark>'))
  }
  // 对非标签部分 escape：简单方案是先 join 再 escape，但会转义 <mark>
  // 正确方案：分段 escape
  result = origChars.join('')
  // escape 非标签文本（保留 <mark></mark>），裸 < 单独 escape 防 XSS
  result = result.replace(/(<mark>|<\/mark>)|([^<]+)|(<)/g, (_, tag, text, lt) => {
    if (tag) return tag
    if (lt) return '&lt;'
    return escapeHtml(text)
  })
  return result
}

// ============ 持久化（搜索历史，最近 5 条） ============
const HISTORY_KEY = 'litobox.fileSearcher.history'
const MAX_HISTORY = 5

interface SearchHistoryItem {
  path: string
  mode: string
  query: string
  caseSensitive: boolean
  extFilterText: string
  includeHidden: boolean
  maxContentMb: number
}

const searchHistory = ref<SearchHistoryItem[]>([])

function loadHistory() {
  const raw = localStorage.getItem(HISTORY_KEY)
  if (raw) {
    try {
      searchHistory.value = JSON.parse(raw)
    } catch {
      searchHistory.value = []
    }
  }
}

function saveHistory() {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(searchHistory.value))
}

function pushHistory() {
  const item: SearchHistoryItem = {
    path: searchPath.value,
    mode: opts.mode,
    query: opts.query,
    caseSensitive: opts.caseSensitive,
    extFilterText: extFilterText.value,
    includeHidden: opts.includeHidden,
    maxContentMb: maxContentMb.value,
  }
  const idx = searchHistory.value.findIndex(
    (h) =>
      h.path === item.path &&
      h.mode === item.mode &&
      h.query === item.query &&
      h.extFilterText === item.extFilterText
  )
  if (idx === 0) return
  if (idx > 0) {
    searchHistory.value.splice(idx, 1)
  }
  searchHistory.value.unshift(item)
  if (searchHistory.value.length > MAX_HISTORY) {
    searchHistory.value = searchHistory.value.slice(0, MAX_HISTORY)
  }
  saveHistory()
}

function applyHistoryItem(item: SearchHistoryItem) {
  searchPath.value = item.path
  opts.mode = item.mode as 'filename' | 'content'
  opts.query = item.query
  opts.caseSensitive = item.caseSensitive
  extFilterText.value = item.extFilterText
  opts.includeHidden = item.includeHidden
  maxContentMb.value = item.maxContentMb
  ElMessage.success('已加载搜索历史')
}

// ============ 扩展名解析 ============
function parseExtFilter(text: string): { inc: string[]; exc: string[] } {
  const tokens = text
    .split(',')
    .map((s) => s.trim().replace(/^\.+/, '').toLowerCase())
    .filter((s) => s.length > 0)
  if (tokens.length === 0) return { inc: [], exc: [] }
  const isExclude = tokens.some((t) => t.startsWith('!'))
  if (isExclude) {
    return { inc: [], exc: tokens.map((t) => t.replace(/^!/, '')) }
  }
  return { inc: tokens, exc: [] }
}

// ============ 文件夹选择 ============
async function selectFolder() {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    searchPath.value = selected as string
  }
}

// ============ 搜索流程 ============
async function startSearch() {
  if (!searchPath.value || !opts.query) return

  // 扩展名过滤解析
  const { inc, exc } = parseExtFilter(extFilterText.value)
  if (inc.length > 0 && exc.length > 0) {
    ElMessage.warning('扩展名不能同时包含和排除，请只用一种模式')
    return
  }
  opts.extensions = inc
  opts.excludeExtensions = exc
  opts.maxContentFileBytes = maxContentMb.value * 1024 * 1024

  // 重置状态
  searchError.value = ''
  summary.value = null
  resultItems.value = []
  totalResults.value = 0
  currentPage.value = 1
  elapsedMs.value = 0
  progress.value = null

  // 保存到搜索历史
  pushHistory()

  try {
    const id = await fileSearchStart(searchPath.value, opts)
    searchId.value = id
    state.value = 'searching'
    startTime.value = Date.now()
    startTimer()
  } catch (e: any) {
    searchError.value = String(e)
    state.value = 'failed'
  }
}

async function cancelSearch() {
  if (!searchId.value) return
  try {
    await fileSearchCancel(searchId.value)
  } catch (e: any) {
    ElMessage.error('取消失败: ' + String(e))
  }
}

async function loadResults(page: number) {
  if (!searchId.value) return
  const offset = (page - 1) * pageSize
  try {
    const page_data = await fileSearchGetResults(searchId.value, pageSize, offset)
    resultItems.value = page_data.items
    totalResults.value = page_data.total
  } catch (e: any) {
    ElMessage.error('加载结果失败: ' + String(e))
  }
}

async function locateInExplorer(row: SearchResultItem) {
  try {
    await invoke('disk_locate_in_explorer', { path: row.path })
  } catch (e: any) {
    ElMessage.error('定位失败: ' + String(e))
  }
}

// ============ 计时器 ============
function startTimer() {
  stopTimer()
  timerId = setInterval(() => {
    elapsedMs.value = Date.now() - startTime.value
  }, 200)
}
function stopTimer() {
  if (timerId) {
    clearInterval(timerId)
    timerId = null
  }
}

// ============ 事件监听 ============
onMounted(async () => {
  loadHistory()

  unlistenFns.push(
    await listen<SearchProgress>('file-search-progress', (e) => {
      if (e.payload.searchId !== searchId.value) return
      progress.value = e.payload
    })
  )

  unlistenFns.push(
    await listen<{ searchId: string; summary: SearchSummary }>('file-search-complete', async (e) => {
      if (e.payload.searchId !== searchId.value) return
      stopTimer()
      summary.value = e.payload.summary
      elapsedMs.value = e.payload.summary.durationMs
      // 状态查询：判断成功/失败/取消
      if (searchId.value) {
        try {
          const status = await invoke<{ status: string; error?: string }>('file_search_status', { searchId: searchId.value })
          if (status.status === 'failed') {
            state.value = 'failed'
            searchError.value = status.error || '搜索失败'
          } else if (status.status === 'cancelled') {
            state.value = 'cancelled'
          } else {
            state.value = 'completed'
            await loadResults(1)
          }
        } catch {
          state.value = 'completed'
          await loadResults(1)
        }
      }
    })
  )

  unlistenFns.push(
    await listen<{ searchId: string; message: string }>('file-search-warning', (e) => {
      if (e.payload.searchId !== searchId.value) return
      ElMessage.warning(e.payload.message)
    })
  )
})

onUnmounted(() => {
  stopTimer()
  unlistenFns.forEach((fn) => fn())
  unlistenFns = []
  // 释放后端内存
  if (searchId.value) {
    fileSearchCancel(searchId.value).catch(() => {})
    fileSearchClear(searchId.value).catch(() => {})
  }
})
</script>

<style scoped>
.action-group-flex-2 {
  flex: 2;
}
.action-grid-mt-sm {
  margin-top: 8px;
}
.pagination-right {
  margin-top: 12px;
  justify-content: flex-end;
}
.progress-info {
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.progress-info div {
  margin-bottom: 4px;
}
.progress-info div:last-child {
  color: var(--text-primary);
}
.summary-text {
  font-size: 12px;
  color: var(--text-secondary);
}
.warn-text {
  color: var(--accent-orange);
  margin-left: 8px;
}
.file-name {
  font-size: 13px;
}
.file-path {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
  word-break: break-all;
}
.file-meta {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
}
.match-line {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  margin-bottom: 4px;
  word-break: break-all;
}
.match-line :deep(mark) {
  background-color: rgba(245, 158, 11, 0.15);
  color: var(--accent-orange);
  padding: 0 2px;
  border-radius: 2px;
}
.line-no {
  color: var(--text-secondary);
  margin-right: 4px;
}
.hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 4px;
}
.history-item {
  padding: 8px 12px;
  line-height: 1.3;
}
.history-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 13px;
  font-weight: 500;
}
.history-query {
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.history-mode {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
}
.history-path {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 280px;
}
</style>
