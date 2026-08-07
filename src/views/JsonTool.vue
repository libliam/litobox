<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <div class="card-actions">
          <el-radio-group v-model="indentSize" size="small">
            <el-radio-button :label="2">2空格</el-radio-button>
            <el-radio-button :label="4">4空格</el-radio-button>
          </el-radio-group>
        </div>
      </div>
      <div class="card-body">
        <ToolActions :actions="jsonActions" />
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="inputValue"
          type="textarea"
          :rows="5"
          placeholder="请输入JSON内容..."
          resize="vertical"
        />
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ filterPath ? '筛选结果' : '输出' }}</span>
        <div class="card-actions">
          <el-input
            v-model="filterPath"
            size="small"
            placeholder="key路径筛选，如: items 或 data[0]"
            class="filter-input"
            clearable
          />
          <el-button size="small" @click="handleCopy">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          :model-value="currentOutput"
          type="textarea"
          :rows="15"
          readonly
          resize="vertical"
          :class="{ 'error': isError }"
        />
        <div v-if="filterError" class="filter-error">
          {{ filterError }}
        </div>
        <div v-else-if="filteredStats && filterPath" class="filter-stats">
          <span class="filter-hint">筛选结果:</span>
          <span v-if="filteredStats.type === 'array'" class="stat-highlight">{{ filteredStats.arrayLength }} 个元素</span>
          <span v-else-if="filteredStats.type === 'object'" class="stat-highlight">{{ filteredStats.objectKeys?.length }} 个属性</span>
          <span v-else class="stat-highlight">基本类型值</span>
        </div>
        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
        </div>
      </div>
    </div>

    <div class="tool-card" v-if="displayStats">
      <div class="card-header">
        <span class="card-title">统计{{ filterPath && filteredStats ? '（筛选后）' : '' }}</span>
      </div>
      <div class="card-body stats-body">
        <div class="stats-item">
          <span class="stats-label">类型</span>
          <span class="stats-value">{{ typeLabel }}</span>
        </div>
        <div class="stats-item" v-if="displayStats.type === 'array'">
          <span class="stats-label">数组长度</span>
          <span class="stats-value stat-highlight">{{ displayStats.arrayLength }}</span>
        </div>
        <div class="stats-item" v-if="displayStats.type === 'array'">
          <span class="stats-label">嵌套元素总数</span>
          <span class="stats-value">{{ displayStats.totalArrayElements }}</span>
        </div>
        <div class="stats-item" v-if="displayStats.type === 'object'">
          <span class="stats-label">顶级属性数</span>
          <span class="stats-value stat-highlight">{{ displayStats.objectKeys?.length }}</span>
        </div>
        <div class="stats-item" v-if="displayStats.type === 'object'">
          <span class="stats-label">嵌套属性总数</span>
          <span class="stats-value">{{ displayStats.totalObjectKeys }}</span>
        </div>
        <div class="stats-item" v-if="displayStats.type === 'string'">
          <span class="stats-label">字符串长度</span>
          <span class="stats-value">{{ displayStats.stringLength }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatJson, compressJson, validateJson, getJsonStats, filterJsonByPath, type JsonStats } from '@/utils/jsonUtils'
import { useToolboxStore, type HistoryRestoreState } from '@/store'
import ToolActions, { type ToolAction } from '@/components/ToolActions.vue'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const indentSize = ref(store.config.jsonIndent)
const stats = ref<JsonStats | null>(null)
const filterPath = ref('')
const filteredOutput = ref('')
const filteredStats = ref<JsonStats | null>(null)
const filterError = ref('')

const displayStats = computed(() => {
  if (filterPath.value && filteredStats.value) return filteredStats.value
  return stats.value
})

const typeLabel = computed(() => {
  const s = displayStats.value
  if (!s) return ''
  const typeMap: Record<string, string> = {
    array: '数组',
    object: '对象',
    string: '字符串',
    number: '数字',
    boolean: '布尔值',
    null: '空值'
  }
  return typeMap[s.type] || s.type
})

const currentOutput = computed(() => {
  return filterPath.value ? filteredOutput.value : outputValue.value
})

const jsonActions = computed<ToolAction[]>(() => [
  {
    label: '格式化',
    type: 'primary',
    handler: () => handleFormat()
  },
  {
    label: '压缩',
    type: 'success',
    handler: () => handleCompress()
  },
  {
    label: '校验',
    type: 'warning',
    handler: () => handleValidate()
  }
])

const updateStats = () => {
  if (!inputValue.value.trim()) {
    stats.value = null
    return
  }
  stats.value = getJsonStats(inputValue.value)
}

const handleFormat = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = formatJson(inputValue.value, { indent: indentSize.value })
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    updateStats()
    handleFilter()
    store.addHistory({
      tool: 'json',
      action: 'format',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50),
      inputFull: inputValue.value,
      outputFull: outputValue.value,
      options: { indentSize: indentSize.value }
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
    stats.value = null
  }
}

const handleCompress = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = compressJson(inputValue.value)
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    updateStats()
    handleFilter()
    store.addHistory({
      tool: 'json',
      action: 'compress',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50),
      inputFull: inputValue.value,
      outputFull: outputValue.value,
      options: {}
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
    stats.value = null
  }
}

const handleValidate = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = validateJson(inputValue.value)
  if (result.success) {
    outputValue.value = '✓ JSON格式正确'
    errorMessage.value = ''
    isError.value = false
    ElMessage.success('JSON格式正确')
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
    ElMessage.error('JSON格式错误')
  }
}

const handleFilter = () => {
  if (!filterPath.value.trim()) {
    filteredOutput.value = ''
    filteredStats.value = null
    filterError.value = ''
    return
  }
  if (!inputValue.value.trim()) {
    filterError.value = '请先输入JSON内容'
    filteredOutput.value = ''
    filteredStats.value = null
    return
  }
  const result = filterJsonByPath(inputValue.value, filterPath.value.trim())
  if (result.success) {
    filteredOutput.value = result.data || ''
    filteredStats.value = getJsonStats(filteredOutput.value)
    filterError.value = ''
  } else {
    filteredOutput.value = ''
    filteredStats.value = null
    filterError.value = result.error || '筛选失败'
  }
}

const handleClearFilter = () => {
  filterPath.value = ''
  filteredOutput.value = ''
  filteredStats.value = null
  filterError.value = ''
}

const handleClear = () => {
  inputValue.value = ''
  stats.value = null
  handleClearFilter()
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  inputValue.value = value
}

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(currentOutput.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const restoreFromHistory = (data: HistoryRestoreState) => {
  isRestoringFromHistory = true
  inputValue.value = data.input
  outputValue.value = data.output
  if (data.options?.indentSize !== undefined) {
    indentSize.value = data.options.indentSize
  }
  updateStats()
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
  if (store.pendingHistoryRestore?.tool === 'json') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})

let autoExecTimer: ReturnType<typeof setTimeout> | null = null
let isRestoringFromHistory = false
watch(inputValue, (value) => {
  if (isRestoringFromHistory) return
  if (!value.trim()) {
    outputValue.value = ''
    errorMessage.value = ''
    isError.value = false
    stats.value = null
    handleClearFilter()
    return
  }
  if (autoExecTimer) clearTimeout(autoExecTimer)
  autoExecTimer = setTimeout(() => {
    handleFormat()
  }, 300)
})

watch(filterPath, () => {
  if (filterPath.value) {
    handleFilter()
  } else {
    handleClearFilter()
  }
})
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
  top: -16px;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  margin-top: -16px;
  padding-top: 16px;
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
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
.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}
.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}
.stats-body {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}
.stats-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}
.stats-label {
  color: var(--text-secondary);
}
.stats-value {
  color: var(--text-primary);
  font-weight: 600;
}
.stat-highlight {
  color: var(--accent-cyan);
  font-size: 15px;
}
.filter-input {
  width: 240px;
}
.filter-stats {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 8px;
}
.filter-error {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 8px;
}
.filter-hint {
  margin-right: 8px;
}
</style>
