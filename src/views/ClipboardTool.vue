<template>
  <div class="tool-container">
    <!-- 标题卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">剪贴板历史</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>自动记录剪贴板的文本、图片和文件路径</p>
                <p>• 最多保存 1000 条记录</p>
                <p>• 图片缓存在本地，关闭监听后不再自动保存</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-switch
            v-model="isMonitoring"
            :active-text="isMonitoring ? '监听中' : '已停止'"
            @change="handleToggleMonitor"
          />
          <el-button size="small" @click="handleClear">清空历史</el-button>
        </div>
      </div>
    </div>

    <!-- 筛选 + 搜索 -->
    <div class="tool-card">
      <div class="card-body">
        <div class="filter-bar">
          <div class="type-filters">
            <button
              v-for="f in filterOptions"
              :key="f.value"
              :class="['filter-btn', { active: filterType === f.value }]"
              @click="setFilter(f.value)"
            >
              {{ f.label }}
              <span v-if="f.count > 0" class="filter-count">{{ f.count }}</span>
            </button>
          </div>
          <div class="search-box">
            <input
              v-model="searchQuery"
              type="text"
              class="search-input"
              placeholder="搜索..."
              @input="handleSearch()"
            />
            <span class="record-count">共 {{ filteredRecords.length }} 条</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 记录列表 -->
    <div class="tool-card">
      <div class="card-body">
        <div v-if="filteredRecords.length === 0" class="empty-state">
          {{ searchQuery ? '未找到匹配的记录' : '暂无剪贴板记录，开启监听后自动保存' }}
        </div>
        <div v-else class="record-list">
          <div
            v-for="record in filteredRecords"
            :key="record.id"
            class="record-item"
            :class="`record-${record.type}`"
          >
            <!-- 文本 -->
            <template v-if="record.type === 'text'">
              <div class="record-content">
                <pre class="record-text">{{ record.text }}</pre>
                <span class="record-time">{{ formatTime(record.timestamp) }}</span>
              </div>
              <div class="record-actions">
                <el-button size="small" @click="handleCopyText(record.text)">复制</el-button>
                <el-button size="small" type="danger" @click="handleDelete(record)">删除</el-button>
              </div>
            </template>

            <!-- 图片 -->
            <template v-else-if="record.type === 'image'">
              <div class="record-content image-content">
                <div class="image-thumb" @click="openViewer(record)">
                  <img v-if="thumbnails[record.id!]" :src="thumbnails[record.id!]" alt="缩略图" />
                  <div v-else class="thumb-placeholder">加载中</div>
                </div>
                <div class="image-meta">
                  <span class="meta-line">{{ imgMeta(record).w }} × {{ imgMeta(record).h }} px</span>
                  <span class="meta-line">{{ imgMeta(record).size }}</span>
                  <span class="record-time">{{ formatTime(record.timestamp) }}</span>
                </div>
              </div>
              <div class="record-actions">
                <el-button size="small" @click="handleCopyImage(record)">复制图片</el-button>
                <el-button size="small" @click="handleSaveImage(record)">保存</el-button>
                <el-button size="small" type="danger" @click="handleDelete(record)">删除</el-button>
              </div>
            </template>

            <!-- 文件 -->
            <template v-else-if="record.type === 'files'">
              <div class="record-content">
                <div class="files-header">
                  <span class="files-icon">📎</span>
                  <span class="files-count">{{ filesMeta(record).count }} 个文件</span>
                </div>
                <pre class="record-text files-list">{{ filesMeta(record).preview }}</pre>
                <span class="record-time">{{ formatTime(record.timestamp) }}</span>
              </div>
              <div class="record-actions">
                <el-button size="small" @click="handleCopyFiles(record)">复制路径</el-button>
                <el-button size="small" type="danger" @click="handleDelete(record)">删除</el-button>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 大图查看器 -->
    <el-dialog
      v-model="viewerOpen"
      title="图片查看"
      width="90%"
      top="5vh"
      :close-on-click-modal="true"
      class="clipboard-viewer-dialog"
    >
      <div class="viewer-container">
        <img v-if="viewerImage" :src="viewerImage" class="viewer-img" alt="大图" />
      </div>
      <template #footer>
        <el-button @click="viewerOpen = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { saveFileWithDialog } from '@/utils/fileSaver'
import * as db from '@/utils/dbClient'

// ============ 状态 ============
const MAX_RECORDS = 1000

const isMonitoring = ref(true)
const records = ref<db.ClipboardRecord[]>([])
const searchQuery = ref('')
const filterType = ref('')
let unlisten: UnlistenFn | null = null
let searchTimer: ReturnType<typeof setTimeout> | null = null

const thumbnails = reactive<Record<number, string>>({})
const viewerOpen = ref(false)
const viewerImage = ref('')

const filterOptions = computed(() => [
  { value: '', label: '全部', count: records.value.length },
  { value: 'text', label: '文本', count: records.value.filter(r => r.type === 'text').length },
  { value: 'image', label: '图片', count: records.value.filter(r => r.type === 'image').length },
  { value: 'files', label: '文件', count: records.value.filter(r => r.type === 'files').length },
])

const filteredRecords = computed(() => {
  if (!filterType.value) return records.value
  return records.value.filter(r => r.type === filterType.value)
})

// ============ 数据加载 ============
const loadRecords = async () => {
  try {
    const rows = await db.listClipboardHistory(MAX_RECORDS, 0)
    records.value = rows
    loadThumbnails(rows)
  } catch {
    records.value = []
  }
}

const handleSearch = () => {
  const val = searchQuery.value
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(async () => {
    if (!val.trim()) {
      await loadRecords()
      return
    }
    try {
      const rows = await db.searchClipboardHistory(val, MAX_RECORDS)
      records.value = rows
      loadThumbnails(rows)
    } catch {
      records.value = []
    }
  }, 300)
}

const setFilter = (type: string) => {
  filterType.value = type
}

// ============ 缩略图加载 ============
const loadThumbnails = async (rows: db.ClipboardRecord[]) => {
  for (const row of rows) {
    if (row.type === 'image' && row.id && !thumbnails[row.id]) {
      try {
        const base64 = await invoke<string>('clipboard_read_image_file', { path: row.text })
        thumbnails[row.id] = 'data:image/png;base64,' + base64
      } catch {
        // 文件可能已被删除
      }
    }
  }
}

// ============ 操作 ============
const handleToggleMonitor = async () => {
  if (isMonitoring.value) {
    await invoke('start_clipboard_monitor')
    ElMessage.success('已开始监听剪贴板')
  } else {
    await invoke('stop_clipboard_monitor')
    ElMessage.warning('已停止监听')
  }
}

const handleCopyText = async (text: string) => {
  try {
    await invoke('copy_to_clipboard', { text })
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyImage = async (record: db.ClipboardRecord) => {
  try {
    const base64 = await invoke<string>('clipboard_read_image_file', { path: record.text })
    await invoke('clipboard_set_image', { base64Png: base64 })
    ElMessage.success('已复制图片到剪贴板')
  } catch (e: any) {
    ElMessage.error('复制失败: ' + e)
  }
}

const handleSaveImage = async (record: db.ClipboardRecord) => {
  try {
    const base64 = await invoke<string>('clipboard_read_image_file', { path: record.text })
    const blob = base64ToBlob(base64, 'image/png')
    const now = new Date()
    const pad = (n: number) => n.toString().padStart(2, '0')
    const filename = `clipboard_${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.png`
    await saveFileWithDialog(blob, filename, 'png')
  } catch (e: any) {
    ElMessage.error('保存失败: ' + e)
  }
}

const handleCopyFiles = async (record: db.ClipboardRecord) => {
  try {
    const paths: string[] = JSON.parse(record.text)
    await invoke('copy_to_clipboard', { text: paths.join('\r\n') })
    ElMessage.success('已复制文件路径')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleDelete = async (record: db.ClipboardRecord) => {
  if (!record.id) return
  // 图片记录同时删除缓存文件
  if (record.type === 'image' && record.text) {
    try {
      await invoke('clipboard_delete_image_file', { path: record.text })
    } catch {
      // 忽略文件删除失败
    }
  }
  try {
    await db.deleteClipboardRecord(record.id)
  } catch {
    // 忽略
  }
  const idx = records.value.findIndex(r => r.id === record.id)
  if (idx >= 0) records.value.splice(idx, 1)
}

const handleClear = async () => {
  try {
    await ElMessageBox.confirm('确定要清空所有剪贴板记录吗？图片缓存也将被清除。', '确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    records.value = []
    Object.keys(thumbnails).forEach(k => delete thumbnails[Number(k)])
    await db.clearClipboardHistory()
    await invoke('clipboard_clear_image_cache')
    ElMessage.success('已清空')
  } catch {
    // 用户取消
  }
}

// ============ 辅助函数 ============
const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return date.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

const base64ToBlob = (base64: string, mime = 'image/png') => {
  const byteChars = atob(base64)
  const byteNumbers = new Uint8Array(byteChars.length)
  for (let i = 0; i < byteChars.length; i++) byteNumbers[i] = byteChars.charCodeAt(i)
  return new Blob([byteNumbers], { type: mime })
}

const parseMeta = (meta: string): Record<string, any> => {
  try { return JSON.parse(meta) } catch { return {} }
}

const imgMeta = (record: db.ClipboardRecord) => {
  const m = parseMeta(record.meta)
  return {
    w: m.width || '?',
    h: m.height || '?',
    size: formatBytes(m.size_bytes || 0),
  }
}

const filesMeta = (record: db.ClipboardRecord) => {
  const m = parseMeta(record.meta)
  let paths: string[] = []
  try { paths = JSON.parse(record.text) } catch { /* ignore */ }
  const preview = paths.slice(0, 5).join('\n') + (paths.length > 5 ? `\n... 还有 ${paths.length - 5} 个` : '')
  return { count: m.count || paths.length, preview }
}

const formatBytes = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1048576).toFixed(2) + ' MB'
}

const openViewer = async (record: db.ClipboardRecord) => {
  if (record.id && thumbnails[record.id]) {
    viewerImage.value = thumbnails[record.id]
    viewerOpen.value = true
  } else {
    try {
      const base64 = await invoke<string>('clipboard_read_image_file', { path: record.text })
      viewerImage.value = 'data:image/png;base64,' + base64
      viewerOpen.value = true
    } catch (e: any) {
      ElMessage.error('无法加载图片: ' + e)
    }
  }
}

// ============ 测试数据 ============
async function loadTestData() {
  const now = Date.now()
  records.value = [
    { id: 6, text: 'SELECT * FROM users WHERE id = 1;', timestamp: new Date(now - 60000).toISOString(), type: 'text', meta: '{}' },
    { id: 5, text: '[D:\\\\test\\\\report.pdf, D:\\\\test\\\\data.xlsx, D:\\\\docs\\\\readme.md]', timestamp: new Date(now - 120000).toISOString(), type: 'files', meta: '{"count":3}' },
    { id: 4, text: 'C:\\\\Users\\\\test\\\\screenshot_2026_08_07.png', timestamp: new Date(now - 180000).toISOString(), type: 'image', meta: '{"width":1920,"height":1080,"size_bytes":245760}' },
    { id: 3, text: 'npm run build && npm run tauri dev', timestamp: new Date(now - 300000).toISOString(), type: 'text', meta: '{}' },
    { id: 2, text: '[C:\\\\code\\\\main.rs, C:\\\\code\\\\lib.rs]', timestamp: new Date(now - 600000).toISOString(), type: 'files', meta: '{"count":2}' },
    { id: 1, text: 'D:\\\\images\\\\photo.jpg', timestamp: new Date(now - 900000).toISOString(), type: 'image', meta: '{"width":800,"height":600,"size_bytes":125829}' },
  ]

  // 图片记录需要模拟缩略图（生成一个简单的内联 SVG 作为占位）
  for (const r of records.value) {
    if (r.type === 'image' && r.id) {
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#0a1628"/><text x="32" y="36" font-size="11" fill="#00d4ff" text-anchor="middle" font-family="monospace">IMG</text></svg>`
      thumbnails[r.id] = 'data:image/svg+xml,' + encodeURIComponent(svg)
    }
  }
}

// ============ 生命周期 ============
onMounted(async () => {
  // ponytail: 测试模式 - URL加 ?dev_test 注入模拟数据验证 UI 闭环
  const isTest = window.location.search.includes('dev_test')

  if (isTest) {
    await loadTestData()
    return
  }

  await loadRecords()

  try {
    await invoke('start_clipboard_monitor')
  } catch {
    // 可能已经启动过
  }

  unlisten = await listen('clipboard://new-entry', () => {
    // 监控已在后端写 DB，前端只需刷新列表
    loadRecords()
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<style scoped>
.tool-container {
  height: 100vh;
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

/* 筛选栏 */
.filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.type-filters {
  display: flex;
  gap: 6px;
}

.filter-btn {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 14px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 4px;
}
.filter-btn:hover {
  border-color: var(--accent-cyan);
  color: var(--text-primary);
}
.filter-btn.active {
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
  color: var(--bg-primary);
  font-weight: 600;
}

.filter-count {
  font-size: 11px;
  background: rgba(0, 0, 0, 0.3);
  padding: 1px 6px;
  border-radius: 8px;
}
.filter-btn.active .filter-count {
  background: rgba(255, 255, 255, 0.2);
}

.search-box {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 200px;
}
.search-input {
  flex: 1;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 12px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}
.search-input:focus {
  border-color: var(--accent-cyan);
  box-shadow: var(--glow-cyan);
}
.record-count {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

/* 空状态 */
.empty-state {
  color: var(--text-muted);
  text-align: center;
  padding: 40px;
  font-size: 13px;
}

/* 记录列表 */
.record-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.record-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.2s;
}
.record-item:hover { border-color: var(--accent-cyan); }

.record-content { flex: 1; min-width: 0; }
.record-text {
  margin: 0 0 4px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 60px;
  overflow: hidden;
  line-height: 1.5;
}
.record-time {
  font-size: 11px;
  color: var(--text-muted);
}

.record-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

/* 图片记录 */
.image-content {
  display: flex;
  gap: 12px;
  align-items: center;
}
.image-thumb {
  width: 64px;
  height: 64px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  cursor: zoom-in;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: box-shadow 0.2s;
}
.image-thumb:hover {
  box-shadow: 0 0 0 2px var(--accent-cyan);
}
.image-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.thumb-placeholder {
  font-size: 11px;
  color: var(--text-muted);
}
.image-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.meta-line {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 文件记录 */
.files-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}
.files-icon { font-size: 14px; }
.files-count {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-cyan);
}
.files-list {
  max-height: 80px;
}

/* 大图查看器 */
.viewer-container {
  width: 100%;
  max-height: 75vh;
  overflow: auto;
  background: var(--bg-input);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.viewer-img {
  max-width: 100%;
  height: auto;
  display: block;
  border-radius: 4px;
}
.clipboard-viewer-dialog :deep(.el-dialog__body) {
  padding: 16px 20px;
}
</style>
