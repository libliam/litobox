<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">URL</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.urlEncode)">URL编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.urlDecode)">URL解码</el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">Base64</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.base64Encode)">Base64编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.base64Decode)">Base64解码</el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">时间戳</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleTimestampToDatetime">时间戳 → 时间</el-button>
              <el-button size="small" @click="handleDatetimeToTimestamp">时间 → 时间戳</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="inputValue"
          type="textarea"
          :rows="6"
          placeholder="请输入内容..."
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
          :rows="6"
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
        <span class="card-title">时间戳选项</span>
      </div>
      <div class="card-body">
        <el-radio-group v-model="timestampMode" size="small">
          <el-radio-button label="ms">毫秒级</el-radio-button>
          <el-radio-button label="s">秒级</el-radio-button>
        </el-radio-group>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import * as encodeUtils from '@/utils/encodeUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const timestampMode = ref<'ms' | 's'>('ms')

const handleEncode = (encodeFn: (text: string) => string) => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  
  const result = encodeFn(inputValue.value)
  outputValue.value = result
  errorMessage.value = ''
  isError.value = false
  
  store.addHistory({
    tool: 'encode',
    action: 'encode',
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50)
  })
  ElMessage.success('处理完成')
}

const handleTimestampToDatetime = () => {
  const timestamp = Number(inputValue.value)
  if (isNaN(timestamp)) {
    errorMessage.value = '请输入有效的时间戳'
    isError.value = true
    return
  }
  
  outputValue.value = encodeUtils.timestampToDatetime(timestamp, timestampMode.value === 'ms')
  errorMessage.value = ''
  isError.value = false
  ElMessage.success('转换完成')
}

const handleDatetimeToTimestamp = () => {
  const result = encodeUtils.datetimeToTimestamp(inputValue.value, timestampMode.value === 'ms')
  if (typeof result === 'string') {
    errorMessage.value = result
    isError.value = true
  } else {
    outputValue.value = String(result)
    errorMessage.value = ''
    isError.value = false
    ElMessage.success('转换完成')
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

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(outputValue.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
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
  align-items: center;
  gap: 6px;
}
.card-body {
  padding: 16px 20px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: flex-end;
}
.action-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.group-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}
.group-buttons {
  display: flex;
  gap: 8px;
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