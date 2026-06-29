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
          <el-button size="small" @click="handleExport">导出备份</el-button>
          <el-button size="small" @click="triggerImport">导入恢复</el-button>
          <el-button size="small" type="danger" @click="handleClear">清空历史</el-button>
          <input ref="importInput" type="file" accept=".json" style="display:none" @change="handleImportFile" />
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
            @dblclick="handleJumpToTool(record)"
            :title="'双击跳转到对应工具'"
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
                <code class="preview-text">{{ formatPreview(record.inputPreview) }}</code>
              </div>
              <div class="preview-row">
                <span class="preview-label">输出</span>
                <code class="preview-text">{{ formatPreview(record.outputPreview) }}</code>
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
import { ElMessage, ElMessageBox, ElLoading } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { Base64 } from 'js-base64'
import { useToolboxStore, TOOL_LIST } from '@/store'
import * as db from '@/utils/dbClient'

const store = useToolboxStore()
const history = computed(() => store.history)
const importInput = ref<HTMLInputElement | null>(null)

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

const LARGE_TEXT_THRESHOLD = 10240

const formatPreview = (text: string): string => {
  if (!text) return ''
  if (text.length > LARGE_TEXT_THRESHOLD) {
    return '[大文本 · 双击查看]'
  }
  return text
}

const handleJumpToTool = async (record: any) => {
  // 检查工具是否在导航列表中
  if (!TOOL_LIST.find(t => t.id === record.tool)) {
    ElMessage.warning('该工具当前不可用')
    return
  }

  const loading = ElLoading.service({
    lock: true,
    text: '正在加载历史数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    // 获取完整数据
    const detail = await db.getHistoryDetail(record.id)

    // 如果没有完整数据，提示用户
    if (!detail || (!detail.input_full && !detail.output_full)) {
      loading.close()
      ElMessage.warning('该历史记录没有完整数据（旧记录只保存了预览），无法还原')
      return
    }

    let options: Record<string, any> = {}
    if (detail?.options_json) {
      try {
        options = JSON.parse(detail.options_json)
      } catch {
        // options_json 解析失败，忽略 options 还原
        options = {}
      }
    }

    store.triggerHistoryRestore({
      tool: record.tool,
      input: detail.input_full || '',
      output: detail.output_full || '',
      options,
      timestamp: record.timestamp,
    })

    // 切换页面
    store.activeTool = record.tool
    ElMessage.success('已加载历史记录，输入和输出已填充')
  } catch (e: any) {
    ElMessage.error('加载失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
}

// 导出备份
const handleExport = async () => {
  const loading = ElLoading.service({
    lock: true,
    text: '正在导出数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    const data = await db.exportAll()
    const date = new Date().toISOString().slice(0, 10).replace(/-/g, '')
    const filename = `litobox-backup-${date}.json`
    // ponytail: 使用 Tauri 保存对话框让用户选择路径
    const result = await db.saveFileWithDialog(Base64.encode(data), filename, 'json')
    if (result !== 'cancelled') {
      ElMessage.success('导出成功')
    }
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
}

// 触发文件选择
const triggerImport = () => {
  importInput.value?.click()
}

// 处理导入文件
const handleImportFile = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  try {
    await ElMessageBox.confirm(
      '导入将覆盖现有数据，确定继续？',
      '确认导入',
      { type: 'warning' }
    )
  } catch {
    input.value = ''
    return
  }

  const loading = ElLoading.service({
    lock: true,
    text: '正在导入数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    const text = await file.text()
    await db.importAll(text)
    ElMessage.success('导入成功，页面将刷新')
    setTimeout(() => window.location.reload(), 1000)
  } catch (e: any) {
    ElMessage.error('导入失败: ' + (e.message || e))
  } finally {
    loading.close()
    input.value = ''
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
  cursor: pointer;
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