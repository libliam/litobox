<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作历史</span>
        <div class="card-actions">
          <el-input
            v-model="searchQuery"
            placeholder="搜索工具名、操作、内容..."
            size="small"
            clearable
            style="width: 200px;"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
          <el-button size="small" @click="handleRefresh">刷新</el-button>
          <el-button size="small" type="danger" @click="handleClear">清空历史</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="filteredHistory.length === 0" class="empty-state">
          {{ searchQuery ? '未找到匹配的历史记录' : '暂无操作历史' }}
        </div>
        <div v-else class="history-list">
          <div
            v-for="(record, index) in filteredHistory"
            :key="index"
            class="history-item"
          >
            <div class="history-header">
              <div class="history-meta">
                <span class="history-tool">{{ getToolName(record.tool) }}</span>
                <span class="history-action">{{ record.action }}</span>
              </div>
              <span class="history-time">{{ formatTime(record.timestamp) }}</span>
            </div>
            <div class="history-preview">
              <div class="preview-row">
                <span class="preview-label">输入</span>
                <code class="preview-text">{{ record.inputPreview }}</code>
              </div>
              <div class="preview-row">
                <span class="preview-label">输出</span>
                <code class="preview-text">{{ record.outputPreview }}</code>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const history = computed(() => store.history)

// 搜索关键词
const searchQuery = ref('')

// 过滤后的历史记录
const filteredHistory = computed(() => {
  if (!searchQuery.value.trim()) {
    return history.value
  }
  const query = searchQuery.value.toLowerCase()
  return history.value.filter(record => {
    return (
      getToolName(record.tool).toLowerCase().includes(query) ||
      record.action.toLowerCase().includes(query) ||
      record.inputPreview.toLowerCase().includes(query) ||
      record.outputPreview.toLowerCase().includes(query)
    )
  })
})

const getToolName = (tool: string): string => {
  const names: Record<string, string> = {
    json: 'JSON工具',
    string: '字符串工具',
    encode: '编码工具',
    regex: '正则工具',
    base: '进制转换',
    uuid: 'UUID生成',
    mockData: '随机数据',
    ocr: 'OCR识别',
    cron: 'Cron表达式',
    css: 'CSS工具',
    jwt: 'JWT解析',
    wordCount: '字数统计',
    color: '颜色工具',
    http: 'HTTP请求',
    sql: 'SQL工具',
    time: '时间工具',
    url: 'URL工具',
    markdown: 'Markdown工具'
  }
  return names[tool] || tool
}

const formatTime = (timestamp: string): string => {
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

const handleRefresh = () => {
  ElMessage.success('已刷新')
}

const handleClear = async () => {
  try {
    await ElMessageBox.confirm('确定要清空所有历史记录吗？', '确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    store.clearHistory()
    ElMessage.success('历史已清空')
  } catch {
    // 用户取消
  }
}
</script>

<style scoped>
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}
.card-body {
  padding: 20px;
}

.empty-state {
  color: var(--text-muted);
  text-align: center;
  padding: 40px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.history-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.3s;
}
.history-item:hover {
  border-color: var(--accent-cyan);
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.history-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}
.history-tool {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-cyan);
}
.history-action {
  font-size: 12px;
  color: var(--text-secondary);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 8px;
  border-radius: 3px;
}
.history-time {
  font-size: 11px;
  color: var(--text-muted);
}

.history-preview {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.preview-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.preview-label {
  font-size: 11px;
  color: var(--text-muted);
  min-width: 30px;
}
.preview-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--text-secondary);
  word-break: break-all;
}
</style>