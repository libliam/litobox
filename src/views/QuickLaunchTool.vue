<template>
  <div class="tool-container">
    <!-- 索引状态提示 -->
    <div v-if="showIndexPrompt" class="tool-card">
      <div class="card-body">
        <div class="index-prompt">
          <span class="prompt-text">首次使用需要建立文件名索引，之后搜索即时返回</span>
          <el-button type="primary" size="small" :loading="isIndexing" @click="startIndex">
            {{ isIndexing ? '索引中...' : '开始建立索引' }}
          </el-button>
        </div>
      </div>
    </div>

    <!-- 索引进度 -->
    <div v-if="isIndexing && indexProgress" class="tool-card">
      <div class="card-body">
        <div class="index-progress">
          <div class="progress-text">
            已扫描 {{ indexProgress.filesScanned }} 个文件
            <span v-if="indexProgress.currentDrive" class="progress-drive">{{ indexProgress.currentDrive }}</span>
          </div>
          <el-progress :percentage="100" :stroke-width="6" :indeterminate="true" :duration="3" />
          <div v-if="indexProgress.currentPath" class="progress-path">{{ indexProgress.currentPath }}</div>
        </div>
      </div>
    </div>

    <!-- 搜索卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">快速启动</span>
        <div class="card-actions">
          <VariablePicker @select="(v: string) => { query = v; onQueryInput() }" />
          <el-tooltip content="重建索引" placement="top">
            <el-button size="small" :icon="Refresh" :loading="isIndexing" @click="handleRebuild" />
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="search-input-wrapper">
          <el-input
            ref="inputRef"
            v-model="query"
            placeholder="搜索文件名…"
            size="large"
            clearable
            :disabled="isIndexing"
            @input="onQueryInput"
            @keydown="handleKeydown"
          />
        </div>
      </div>
    </div>

    <!-- 搜索结果 -->
    <div v-if="results.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">搜索结果 ({{ results.length }})</span>
      </div>
      <div class="result-list">
        <div
          v-for="(item, idx) in results"
          :key="item.id"
          class="result-row"
          :class="{ selected: selectedIndex === idx }"
          @click="openFile(item.path)"
          @mouseenter="selectedIndex = idx"
        >
          <span class="result-icon">{{ getFileIcon(item.extension) }}</span>
          <div class="result-info">
            <span class="result-name">{{ item.name }}</span>
            <span class="result-path">{{ item.path }}</span>
          </div>
          <div class="result-meta">
            <span class="result-size">{{ formatSize(item.sizeBytes) }}</span>
            <span class="result-date">{{ formatDate(item.modifiedAt) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="searched" class="tool-card">
      <div class="card-body">
        <el-empty :description="query ? '未找到匹配的文件' : '输入文件名开始搜索'" />
      </div>
    </div>

    <!-- 索引状态栏 -->
    <div v-if="indexStatus && indexStatus.drives.length > 0" class="index-status-bar">
      <span v-for="d in indexStatus.drives" :key="d.drive" class="drive-status">
        {{ d.drive }}
        <span :class="statusClass(d.status)">{{ statusLabel(d.status) }}</span>
        ({{ d.fileCount }} 文件)
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  type QuickLaunchResult,
  type IndexStatus,
  type QLIndexProgress,
  qlSearch,
  qlIndexStatus,
  qlBuildIndex,
  qlRebuildIndex,
  qlOpenFile,
} from '@/utils/quickLaunchClient'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const query = ref('')
const results = ref<QuickLaunchResult[]>([])
const selectedIndex = ref(0)
const searched = ref(false)
const isIndexing = ref(false)
const showIndexPrompt = ref(false)
const indexProgress = ref<QLIndexProgress | null>(null)
const indexStatus = ref<IndexStatus | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | null = null
let unlistens: UnlistenFn[] = []
let lastRecordedQuery = ''

onMounted(async () => {
  await refreshIndexStatus()
  unlistens.push(await listen<QLIndexProgress>('ql-index-progress', (e) => {
    const p = e.payload
    indexProgress.value = p
    if (p.status === 'completed') {
      isIndexing.value = false
      showIndexPrompt.value = false
      ElMessage.success('索引构建完成')
      refreshIndexStatus()
    } else if (p.status === 'cancelled') {
      isIndexing.value = false
      ElMessage.info('索引已取消')
    } else if (p.status === 'failed') {
      isIndexing.value = false
      ElMessage.error(p.error || '索引构建失败')
    }
  }))

  if (store.pendingHistoryRestore?.tool === 'quickLaunch') {
    const data = store.pendingHistoryRestore
    query.value = data.input
    await doSearch()
    store.clearHistoryRestore()
  }
})

onUnmounted(() => {
  unlistens.forEach(fn => fn())
  if (searchTimer) clearTimeout(searchTimer)
})

async function refreshIndexStatus() {
  try {
    const status = await qlIndexStatus()
    indexStatus.value = status
    isIndexing.value = status.isBuilding
    const allReady = status.drives.length > 0 && status.drives.every(d => d.status === 'ready')
    showIndexPrompt.value = !allReady && !status.isBuilding
  } catch {
    showIndexPrompt.value = true
  }
}

async function startIndex() {
  isIndexing.value = true
  showIndexPrompt.value = false
  try {
    await qlBuildIndex()
  } catch (e) {
    isIndexing.value = false
    ElMessage.error(String(e))
  }
}

async function handleRebuild() {
  isIndexing.value = true
  results.value = []
  searched.value = false
  try {
    await qlRebuildIndex()
  } catch (e) {
    isIndexing.value = false
    ElMessage.error(String(e))
  }
}

function onQueryInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
}

async function doSearch() {
  if (!query.value.trim()) {
    results.value = []
    searched.value = false
    return
  }
  try {
    results.value = await qlSearch(query.value.trim())
    searched.value = true
    selectedIndex.value = 0
    if (query.value.trim() && query.value.trim() !== lastRecordedQuery) {
      lastRecordedQuery = query.value.trim()
      store.addHistory({
        tool: 'quickLaunch',
        action: '文件名搜索',
        inputPreview: query.value.trim().slice(0, 50),
        outputPreview: `${results.value.length} 条结果`.slice(0, 50),
        inputFull: query.value.trim(),
        outputFull: JSON.stringify(results.value.map(r => ({ name: r.name, path: r.path }))),
      })
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = results.value[selectedIndex.value]
    if (item) openFile(item.path)
  }
}

async function openFile(path: string) {
  try {
    await qlOpenFile(path)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function getFileIcon(ext: string): string {
  const icons: Record<string, string> = {
    txt: '📄', md: '📝', pdf: '📕',
    doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
    ppt: '📙', pptx: '📙',
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', webp: '🖼️', bmp: '🖼️', svg: '🖼️',
    mp3: '🎵', wav: '🎵', flac: '🎵', m4a: '🎵', ogg: '🎵',
    mp4: '🎬', avi: '🎬', mkv: '🎬', mov: '🎬', webm: '🎬',
    zip: '📦', rar: '📦', '7z': '📦', gz: '📦',
    exe: '⚙️', dll: '⚙️', msi: '⚙️',
    js: '🟨', ts: '🟦', py: '🐍', rs: '🦀', go: '🔵',
    json: '📋', xml: '📋', yaml: '📋', yml: '📋', toml: '📋',
    html: '🌐', css: '🎨',
  }
  return icons[ext.toLowerCase()] || '📄'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`
}

function formatDate(ts: number): string {
  if (!ts) return ''
  return new Date(ts * 1000).toLocaleDateString('zh-CN', {
    month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
}

function statusClass(s: string): string {
  return s === 'ready' ? 'status-ready' : s === 'indexing' ? 'status-indexing' : s === 'pending' ? 'status-pending' : 'status-failed'
}

function statusLabel(s: string): string {
  return s === 'ready' ? '✓' : s === 'indexing' ? '⟳' : s === 'pending' ? '⚠' : '✗'
}
</script>

<style scoped>
.tool-container {
  padding: 20px;
  height: 100vh;
  overflow-y: auto;
  background: var(--bg-primary);
}

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

.card-body { padding: 16px 20px; }

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input-wrapper {
  max-width: 600px;
}

.index-prompt {
  display: flex;
  align-items: center;
  gap: 12px;
}

.prompt-text {
  color: var(--text-secondary);
  font-size: 13px;
}

.index-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-text {
  font-size: 12px;
  color: var(--text-secondary);
}

.progress-drive {
  margin-left: 8px;
  font-weight: 600;
}

.progress-path {
  font-size: 11px;
  color: var(--text-muted);
  word-break: break-all;
}

.result-list {
  padding: 0;
}

.result-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  cursor: pointer;
  transition: background 0.12s;
  border-bottom: 1px solid var(--border-color);
}
.result-row:last-child { border-bottom: none; }
.result-row:hover,
.result-row.selected {
  background: var(--bg-secondary);
}

.result-icon {
  flex-shrink: 0;
  font-size: 20px;
  width: 28px;
  text-align: center;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  display: block;
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.3;
}

.result-path {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  flex-shrink: 0;
  text-align: right;
  font-size: 11px;
  color: var(--text-muted);
}

.result-size { display: block; }
.result-date { display: block; margin-top: 2px; }

.index-status-bar {
  display: flex;
  gap: 16px;
  padding: 8px 0;
  font-size: 12px;
  color: var(--text-muted);
}

.drive-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-ready { color: var(--accent-green); }
.status-indexing { color: var(--accent-cyan); }
.status-pending { color: var(--accent-orange); }
.status-failed { color: var(--accent-red); }
</style>
