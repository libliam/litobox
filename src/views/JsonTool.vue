<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
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
          :rows="10"
          placeholder="请输入JSON内容..."
          resize="vertical"
        />
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <el-button size="small" @click="handleCopy">复制</el-button>
      </div>
      <div class="card-body">
        <el-input
          :model-value="outputValue"
          type="textarea"
          :rows="10"
          readonly
          resize="vertical"
          :class="{ 'error': isError }"
        />
        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
        </div>
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">缩进设置</span>
      </div>
      <div class="card-body">
        <el-radio-group v-model="indentSize" size="small">
          <el-radio-button :label="2">2空格缩进</el-radio-button>
          <el-radio-button :label="4">4空格缩进</el-radio-button>
        </el-radio-group>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatJson, compressJson, validateJson } from '@/utils/jsonUtils'
import { useToolboxStore, type HistoryRestoreState } from '@/store'
import ToolActions, { type ToolAction } from '@/components/ToolActions.vue'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const indentSize = ref(store.config.jsonIndent)

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
    store.addHistory({
      tool: 'json',
      action: 'format',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50)
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
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
    store.addHistory({
      tool: 'json',
      action: 'compress',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50)
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
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

const handleClear = () => {
  inputValue.value = ''
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
    await navigator.clipboard.writeText(outputValue.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const restoreFromHistory = (data: HistoryRestoreState) => {
  // 填充输入框
  inputValue.value = data.input
  // 填充输出框（不重新执行）
  outputValue.value = data.output
  // 还原配置
  if (data.options?.indentSize !== undefined) {
    indentSize.value = data.options.indentSize
  }
  // 显示提示
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}

onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'json') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})

// 粘贴后自动执行格式化（带防抖）
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(inputValue, (value) => {
  if (!value.trim()) {
    outputValue.value = ''
    errorMessage.value = ''
    isError.value = false
    return
  }
  if (autoExecTimer) clearTimeout(autoExecTimer)
  autoExecTimer = setTimeout(() => {
    handleFormat()
  }, 300)
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
</style>