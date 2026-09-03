<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <div class="card-actions">
          <el-tooltip content="开启后输入多行数据将逐行转换，输出与输入逐行对应" placement="top">
            <el-checkbox v-model="batchMode" size="small">多行批量</el-checkbox>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="--group-color: #00d4ff">
            <div class="group-label">URL</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.urlEncode)">编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.urlDecode)">解码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.urlDoubleDecode)">双重解码</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #10b981">
            <div class="group-label">Base64</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.base64Encode)">编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.base64Decode)">解码</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #f59e0b">
            <div class="group-label">HTML实体</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.htmlEncode)">编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.htmlDecode)">解码</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #ef4444">
            <div class="group-label">Unicode</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleEncode(encodeUtils.unicodeEncode)">编码</el-button>
              <el-button size="small" @click="handleEncode(encodeUtils.unicodeDecode)">解码</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #64748b">
            <div class="group-label">时间戳</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleTimestampToDatetime">时间戳 → 时间</el-button>
              <el-button size="small" @click="handleDatetimeToTimestamp">时间 → 时间戳</el-button>
            </div>
          </div>
          <div class="action-group" style="--group-color: #8b5cf6">
            <div class="group-label">人民币大写</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleAmountToUpper">数字 → 大写</el-button>
            </div>
          </div>
        </div>
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
          :rows="6"
          placeholder="请输入内容..."
          resize="vertical"
        />
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-button size="small" @click="handleOutputToInput" :disabled="!outputValue">转到输入</el-button>
          <el-button size="small" @click="handleCopy">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="outputValue"
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
    
    <div v-if="showTimestampOptions" class="tool-card">
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
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import * as encodeUtils from '@/utils/encodeUtils'
import { amountToUpper } from '@/utils/mockDataUtils'
import { useToolboxStore, type HistoryRestoreState } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const timestampMode = ref<'ms' | 's'>('ms')
const showTimestampOptions = ref(false)
const batchMode = ref(false)

// 多行批量：按行拆分并过滤空行
const splitLines = () =>
  inputValue.value
    .split('\n')
    .map(l => l.trim())
    .filter(l => l.length > 0)

const handleEncode = (encodeFn: (text: string) => string) => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  const result = batchMode.value
    ? splitLines().map(line => encodeFn(line)).join('\n')
    : encodeFn(inputValue.value)
  outputValue.value = result
  errorMessage.value = ''
  isError.value = result.includes('失败')
  store.addHistory({
    tool: 'encode',
    action: 'encode',
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50),
    inputFull: inputValue.value,
    outputFull: outputValue.value,
    options: { batch: batchMode.value }
  })
  ElMessage.success('处理完成')
}

const handleTimestampToDatetime = () => {
  showTimestampOptions.value = true
  if (batchMode.value) {
    const lines = splitLines()
    if (!lines.length) {
      ElMessage.warning('请输入内容')
      return
    }
    outputValue.value = lines
      .map(line => {
        const t = Number(line)
        if (isNaN(t)) return `无效: ${line}`
        return encodeUtils.timestampToDatetime(t, timestampMode.value === 'ms')
      })
      .join('\n')
    errorMessage.value = ''
    isError.value = false
    store.addHistory({
      tool: 'encode',
      action: 'timestamp_to_datetime',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50),
      inputFull: inputValue.value,
      outputFull: outputValue.value,
      options: { batch: true }
    })
    ElMessage.success('转换完成')
    return
  }
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
  showTimestampOptions.value = true
  if (batchMode.value) {
    const lines = splitLines()
    if (!lines.length) {
      ElMessage.warning('请输入内容')
      return
    }
    outputValue.value = lines
      .map(line => {
        const r = encodeUtils.datetimeToTimestamp(line, timestampMode.value === 'ms')
        return typeof r === 'string' ? `无效: ${line}` : String(r)
      })
      .join('\n')
    errorMessage.value = ''
    isError.value = false
    store.addHistory({
      tool: 'encode',
      action: 'datetime_to_timestamp',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50),
      inputFull: inputValue.value,
      outputFull: outputValue.value,
      options: { batch: true }
    })
    ElMessage.success('转换完成')
    return
  }
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

const handleAmountToUpper = () => {
  const raw = inputValue.value.trim()
  if (!raw) {
    ElMessage.warning('请输入金额')
    return
  }
  if (batchMode.value) {
    const lines = splitLines()
    if (!lines.length) {
      ElMessage.warning('请输入金额')
      return
    }
    outputValue.value = lines
      .map(line => {
        const num = Number(line)
        if (isNaN(num)) return `无效: ${line}`
        return amountToUpper(num)
      })
      .join('\n')
    errorMessage.value = ''
    isError.value = false
    store.addHistory({
      tool: 'encode',
      action: 'amount_to_upper',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50),
      inputFull: inputValue.value,
      outputFull: outputValue.value,
      options: { batch: true }
    })
    ElMessage.success('转换完成')
    return
  }
  const num = Number(raw)
  if (isNaN(num)) {
    errorMessage.value = '请输入有效的数字金额'
    isError.value = true
    return
  }
  outputValue.value = amountToUpper(num)
  errorMessage.value = ''
  isError.value = false
  store.addHistory({
    tool: 'encode',
    action: 'amount_to_upper',
    inputPreview: raw.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50),
    inputFull: raw,
    outputFull: outputValue.value,
    options: {}
  })
  ElMessage.success('转换完成')
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

const handleOutputToInput = () => {
  if (!outputValue.value) {
    ElMessage.warning('输出为空')
    return
  }
  inputValue.value = outputValue.value
  outputValue.value = ''
  errorMessage.value = ''
  isError.value = false
  ElMessage.success('已转到输入')
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
  inputValue.value = data.input
  outputValue.value = data.output
  if (data.options?.timestampMode) {
    timestampMode.value = data.options.timestampMode
  }
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}

onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'encode') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
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
  gap: 12px;
  align-items: stretch;
}
.action-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--group-color, var(--accent-cyan));
  border-radius: 6px;
  min-width: 120px;
}
.group-label {
  font-size: 12px;
  color: var(--group-color, var(--text-secondary));
  font-weight: 600;
  letter-spacing: 0.5px;
}
.group-buttons {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.group-buttons :deep(.el-button) {
  border-color: var(--group-color, var(--border-color));
  color: var(--text-primary);
  background: transparent;
}
.group-buttons :deep(.el-button:hover) {
  border-color: var(--group-color, var(--accent-cyan));
  color: var(--group-color, var(--accent-cyan));
  background: rgba(0, 212, 255, 0.05);
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
