<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="inputValue"
          placeholder="输入数值..."
          size="default"
          @input="handleConvert"
        >
          <template #prepend>
            <el-select v-model="fromBase" style="width: 90px" @change="handleConvert">
              <el-option label="BIN" :value="2" />
              <el-option label="OCT" :value="8" />
              <el-option label="DEC" :value="10" />
              <el-option label="HEX" :value="16" />
            </el-select>
          </template>
        </el-input>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">转换结果</span>
        <el-button size="small" @click="handleCopyAll">复制全部</el-button>
      </div>
      <div class="card-body">
        <div v-if="errorMessage" class="error-message">
          {{ errorMessage }}
        </div>
        <div v-else class="result-grid">
          <div class="result-item">
            <span class="result-label">二进制 (BIN)</span>
            <code class="result-value">{{ result.binary || '-' }}</code>
            <el-button size="small" text @click="handleCopy(result.binary || '')">复制</el-button>
          </div>
          <div class="result-item">
            <span class="result-label">八进制 (OCT)</span>
            <code class="result-value">{{ result.octal || '-' }}</code>
            <el-button size="small" text @click="handleCopy(result.octal || '')">复制</el-button>
          </div>
          <div class="result-item">
            <span class="result-label">十进制 (DEC)</span>
            <code class="result-value">{{ result.decimal || '-' }}</code>
            <el-button size="small" text @click="handleCopy(result.decimal || '')">复制</el-button>
          </div>
          <div class="result-item">
            <span class="result-label">十六进制 (HEX)</span>
            <code class="result-value">{{ result.hexadecimal || '-' }}</code>
            <el-button size="small" text @click="handleCopy(result.hexadecimal || '')">复制</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { convertBase } from '@/utils/baseConverter'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const inputValue = ref('')
const fromBase = ref<2 | 8 | 10 | 16>(10)
const errorMessage = ref('')
const result = reactive({
  binary: '',
  octal: '',
  decimal: '',
  hexadecimal: ''
})

const handleConvert = () => {
  if (!inputValue.value.trim()) {
    errorMessage.value = ''
    result.binary = ''
    result.octal = ''
    result.decimal = ''
    result.hexadecimal = ''
    return
  }

  const conversion = convertBase(inputValue.value, fromBase.value)
  if (!conversion.success) {
    errorMessage.value = conversion.error || '转换失败'
    result.binary = ''
    result.octal = ''
    result.decimal = ''
    result.hexadecimal = ''
    return
  }

  errorMessage.value = ''
  result.binary = conversion.binary || ''
  result.octal = conversion.octal || ''
  result.decimal = conversion.decimal || ''
  result.hexadecimal = conversion.hexadecimal || ''

  store.addHistory({
    tool: 'base',
    action: 'convert',
    inputPreview: `${inputValue.value} (base ${fromBase.value})`,
    outputPreview: `DEC: ${result.decimal}`
  })
}

const handleClear = () => {
  inputValue.value = ''
  errorMessage.value = ''
  result.binary = ''
  result.octal = ''
  result.decimal = ''
  result.hexadecimal = ''
}

const handleCopy = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyAll = async () => {
  const allResults = `BIN: ${result.binary}\nOCT: ${result.octal}\nDEC: ${result.decimal}\nHEX: ${result.hexadecimal}`
  try {
    await navigator.clipboard.writeText(allResults)
    ElMessage.success('已复制全部')
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

.result-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.3s;
}
.result-item:hover {
  border-color: var(--accent-cyan);
}
.result-label {
  font-size: 12px;
  color: var(--accent-cyan);
  font-weight: 600;
  min-width: 100px;
  text-transform: uppercase;
  letter-spacing: 1px;
}
.result-value {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}
</style>