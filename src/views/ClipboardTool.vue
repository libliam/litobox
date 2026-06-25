<template>
  <div class="tool-container">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">剪贴板历史</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>自动记录系统剪贴板的文本内容</p>
                <p>• 最多保存 1000 条记录</p>
                <p>• 总大小限制 3MB</p>
                <p>• 关闭监听后不再自动保存</p>
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

    <!-- 搜索框 -->
    <div class="tool-card">
      <div class="card-body">
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索剪贴板内容..."
          />
          <span class="record-count">共 {{ filteredRecords.length }} 条</span>
        </div>
      </div>
    </div>

    <!-- 历史记录列表 -->
    <div class="tool-card">
      <div class="card-body">
        <div v-if="filteredRecords.length === 0" class="empty-state">
          {{ searchQuery ? '未找到匹配的记录' : '暂无剪贴板记录，开启监听后自动保存' }}
        </div>
        <div v-else class="record-list">
          <div
            v-for="(record, idx) in filteredRecords"
            :key="idx"
            class="record-item"
          >
            <div class="record-content">
              <pre class="record-text">{{ record.text }}</pre>
              <span class="record-time">{{ formatTime(record.timestamp) }}</span>
            </div>
            <div class="record-actions">
              <el-button size="small" @click="handleCopy(record.text)">复制</el-button>
              <el-button size="small" type="danger" @click="handleDelete(idx)">删除</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

interface ClipboardRecord {
  text: string
  timestamp: number
}

const STORAGE_KEY = 'clipboard_history'
const MAX_RECORDS = 1000
const MAX_STORAGE_SIZE = 3 * 1024 * 1024

const isMonitoring = ref(true)
const records = ref<ClipboardRecord[]>([])
const searchQuery = ref('')
let unlisten: UnlistenFn | null = null

const filteredRecords = computed(() => {
  if (!searchQuery.value.trim()) return records.value
  const query = searchQuery.value.toLowerCase()
  return records.value.filter(r => r.text.toLowerCase().includes(query))
})

const loadRecords = () => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) {
      records.value = JSON.parse(saved)
    }
  } catch {
    records.value = []
  }
}

const saveRecords = (newRecords: ClipboardRecord[]) => {
  let data = newRecords

  // 按条数裁剪
  while (data.length > MAX_RECORDS) {
    data = data.slice(0, -1)
  }

  // 按大小裁剪
  const json = JSON.stringify(data)
  const size = new Blob([json]).size
  if (size > MAX_STORAGE_SIZE) {
    while (new Blob([JSON.stringify(data)]).size > MAX_STORAGE_SIZE && data.length > 0) {
      data = data.slice(0, -1)
    }
  }

  records.value = data
  localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
}

const addRecord = (text: string) => {
  // 去重：如果与最新一条相同则跳过
  if (records.value.length > 0 && records.value[0].text === text) return

  const newRecords = [{ text, timestamp: Date.now() }, ...records.value]
  saveRecords(newRecords)
}

const handleToggleMonitor = async () => {
  if (isMonitoring.value) {
    await invoke('start_clipboard_monitor')
    ElMessage.success('已开始监听剪贴板')
  } else {
    await invoke('stop_clipboard_monitor')
    ElMessage.warning('已停止监听')
  }
}

const handleCopy = async (text: string) => {
  try {
    await invoke('copy_to_clipboard', { text })
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleDelete = (idx: number) => {
  const filtered = filteredRecords.value
  const record = filtered[idx]
  const realIdx = records.value.indexOf(record)
  if (realIdx > -1) {
    records.value.splice(realIdx, 1)
    saveRecords([...records.value])
  }
}

const handleClear = async () => {
  try {
    await ElMessageBox.confirm('确定要清空所有剪贴板记录吗？', '确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    records.value = []
    localStorage.removeItem(STORAGE_KEY)
    ElMessage.success('已清空')
  } catch {
    // 用户取消
  }
}

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`

  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

onMounted(async () => {
  loadRecords()
  
  // 启动监听
  try {
    await invoke('start_clipboard_monitor')
  } catch {
    // 可能已经启动过
  }

  // 监听新条目
  unlisten = await listen('clipboard://new-entry', (event) => {
    const payload = event.payload as { text: string }
    if (payload?.text) {
      addRecord(payload.text)
    }
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
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

/* 搜索框 */
.search-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.search-input {
  flex: 1;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 8px 12px;
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
</style>
