<template>
  <div class="tool-output">
    <div class="output-header">
      <span class="output-title">输出</span>
      <el-button size="small" @click="handleCopy">复制</el-button>
    </div>
    <el-input
      :model-value="outputValue"
      type="textarea"
      :rows="rows"
      readonly
      resize="vertical"
      :class="{ 'error': isError }"
    />
    <div v-if="errorMessage" class="error-message">
      <span class="error-icon"></span>
      {{ errorMessage }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElMessage } from 'element-plus'

const props = defineProps<{
  outputValue: string
  errorMessage?: string
  isError?: boolean
  rows?: number
}>()

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(props.outputValue)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-output {
  margin-bottom: 16px;
}
.output-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.output-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}
.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.error-icon {
  font-size: 14px;
}
</style>