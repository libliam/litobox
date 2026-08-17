<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 解析 JWT token 的 Header 和 Payload</p>
                <p>• 自动检测过期时间并高亮</p>
                <p>• 支持标准 Base64 和 Base64URL</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleParse">解析</el-button>
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
        <el-input v-model="input" type="textarea" :rows="4" placeholder="请输入 JWT token..." resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">Header</span>
        <el-button v-if="header" size="small" @click="handleCopy(JSON.stringify(header, null, 2))">复制</el-button>
      </div>
      <div class="card-body">
        <pre v-if="header" class="json-output">{{ JSON.stringify(header, null, 2) }}</pre>
        <div v-else class="empty-tip">解析后将在此显示 Header 信息</div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">Payload</span>
        <el-button v-if="payload" size="small" @click="handleCopy(JSON.stringify(payload, null, 2))">复制</el-button>
      </div>
      <div class="card-body">
        <div v-if="payload" class="payload-section">
          <pre class="json-output">{{ JSON.stringify(payload, null, 2) }}</pre>
          <div v-if="expInfo" class="exp-info" :class="{ 'expired': expInfo.isExpired }">
            <span class="exp-label">过期时间:</span>
            <span>{{ expInfo.datetime }}</span>
            <span class="exp-status">{{ expInfo.isExpired ? '已过期' : '有效' }}</span>
          </div>
        </div>
        <div v-else class="empty-tip">解析后将在此显示 Payload 信息</div>
      </div>
    </div>

    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { Base64 } from 'js-base64'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const input = ref('')
const header = ref<Record<string, any> | null>(null)
const payload = ref<Record<string, any> | null>(null)
const error = ref('')

const expInfo = computed(() => {
  if (!payload.value || !payload.value.exp) return null
  const expTimestamp = payload.value.exp * 1000
  const expDate = new Date(expTimestamp)
  const isExpired = Date.now() > expTimestamp
  return {
    datetime: expDate.toLocaleString('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
    }),
    isExpired
  }
})

const base64UrlDecode = (str: string): string => {
  str = str.replace(/-/g, '+').replace(/_/g, '/')
  while (str.length % 4) str += '='
  const bytes = Base64.toUint8Array(str)
  return new TextDecoder().decode(bytes)
}

const handleParse = () => {
  const token = input.value.trim()
  if (!token) {
    ElMessage.warning('请输入 JWT token')
    return
  }

  const parts = token.split('.')
  if (parts.length !== 3) {
    error.value = '无效的 JWT token，应包含 3 个部分（用 . 分隔）'
    header.value = null
    payload.value = null
    ElMessage.error('无效的 JWT token')
    return
  }

  try {
    header.value = JSON.parse(base64UrlDecode(parts[0]))
  } catch {
    error.value = '无法解析 Header 部分'
    header.value = null
    payload.value = null
    ElMessage.error('无法解析 Header')
    return
  }

  try {
    payload.value = JSON.parse(base64UrlDecode(parts[1]))
  } catch {
    error.value = '无法解析 Payload 部分'
    payload.value = null
    ElMessage.error('无法解析 Payload')
    return
  }

  error.value = ''
  store.addHistory({
    tool: 'jwt',
    action: 'JWT解析',
    inputPreview: token.slice(0, 30) + '...',
    outputPreview: JSON.stringify(payload.value).slice(0, 50),
    inputFull: token,
    outputFull: JSON.stringify(payload.value, null, 2),
  })
  ElMessage.success('解析成功')
}

const handleClear = () => {
  input.value = ''
  header.value = null
  payload.value = null
  error.value = ''
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}

const handleCopy = (text: string) => {
  navigator.clipboard.writeText(text)
  ElMessage.success('已复制到剪贴板')
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

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
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

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

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.json-output {
  background: var(--bg-input);
  padding: 12px 16px;
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.payload-section { display: flex; flex-direction: column; gap: 12px; }

.exp-info {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(0, 212, 255, 0.05);
  border: 1px solid var(--border-color);
  font-size: 13px;
}
.exp-info.expired {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--accent-red);
}
.exp-label { color: var(--text-secondary); font-weight: 500; }
.exp-status { margin-left: auto; font-weight: 600; }
.exp-info:not(.expired) .exp-status { color: #22c55e; }
.exp-info.expired .exp-status { color: var(--accent-red); }

.error-message {
  margin-top: 16px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}
</style>
