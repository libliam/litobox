<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">HTTP 请求</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>通过 Tauri 后端发起 HTTP 请求，绕过浏览器 CORS 限制</p>
                <p>支持 GET/POST/PUT/DELETE/PATCH 方法</p>
                <p>请求体支持 JSON / Form / 纯文本格式</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <el-button size="small" type="primary" @click="handleSend" :loading="loading">发送</el-button>
      </div>
      <div class="card-body">
        <div class="request-bar">
          <el-select v-model="method" size="default" style="width: 120px">
            <el-option label="GET" value="GET" />
            <el-option label="POST" value="POST" />
            <el-option label="PUT" value="PUT" />
            <el-option label="DELETE" value="DELETE" />
            <el-option label="PATCH" value="PATCH" />
            <el-option label="HEAD" value="HEAD" />
            <el-option label="OPTIONS" value="OPTIONS" />
          </el-select>
          <el-input
            v-model="url"
            placeholder="输入请求 URL，如: https://api.example.com/v1/users"
            size="default"
            style="flex: 1; margin-left: 8px"
            @keyup.enter="handleSend"
          />
          <el-input-number v-model="timeoutMs" :min="1000" :max="60000" :step="5000" size="default" style="width: 120px; margin-left: 8px" />
          <span class="timeout-label">ms</span>
        </div>
      </div>
    </div>

    <!-- 请求头 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">请求头</span>
        <el-button size="small" @click="addHeader">+ 添加</el-button>
      </div>
      <div class="card-body">
        <div v-for="(header, idx) in headers" :key="idx" class="header-row">
          <el-input v-model="header.key" placeholder="Header 名称" size="small" style="width: 180px" />
          <el-input v-model="header.value" placeholder="值" size="small" style="flex: 1; margin-left: 8px" />
          <el-button size="small" type="danger" :icon="Delete" circle @click="removeHeader(idx)" style="margin-left: 8px" />
        </div>
        <div v-if="headers.length === 0" class="empty-hint">暂无请求头</div>
      </div>
    </div>

    <!-- 请求体 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">请求体</span>
        <div class="card-actions">
          <el-select v-model="bodyType" size="small" style="width: 120px">
            <el-option label="JSON" value="json" />
            <el-option label="Form" value="form" />
            <el-option label="Text" value="text" />
          </el-select>
          <el-button size="small" @click="handleClearBody">清空</el-button>
          <el-button size="small" @click="handlePasteBody">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="body"
          type="textarea"
          :rows="6"
          placeholder='{"key": "value"}'
          resize="vertical"
        />
      </div>
    </div>

    <!-- 响应 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">响应</span>
        <div class="card-actions">
          <el-tabs v-model="responseTab" size="small" class="response-tabs" @tab-click="">
            <el-tab-pane label="Body" name="body" />
            <el-tab-pane label="Headers" name="headers" />
          </el-tabs>
          <el-button v-if="responseTab === 'body'" size="small" @click="handleCopyResponse" :disabled="!responseBody">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="loading" class="loading-state">
          <el-icon class="is-loading"><Loading /></el-icon>
          <span>请求中...</span>
        </div>
        <div v-else-if="responseStatus">
          <!-- 状态栏 -->
          <div class="response-status-bar">
            <span :class="['status-code', statusClass]">{{ responseStatus }} {{ responseStatusText }}</span>
            <span class="response-meta">
              {{ responseTime }}ms · {{ formatSize(responseSize) }}
            </span>
          </div>
          <!-- Body -->
          <div v-if="responseTab === 'body'">
            <el-input
              :model-value="formattedResponseBody"
              type="textarea"
              :rows="12"
              readonly
              resize="vertical"
              class="response-textarea"
            />
          </div>
          <!-- Headers -->
          <div v-else-if="responseTab === 'headers'">
            <div v-for="(value, key) in responseHeaders" :key="key" class="response-header-row">
              <span class="header-key">{{ key }}</span>
              <span class="header-value">{{ value }}</span>
            </div>
          </div>
        </div>
        <div v-else-if="errorMsg" class="error-message">{{ errorMsg }}</div>
        <div v-else class="stats-empty">发送请求后查看响应</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Delete, Loading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 请求配置 ============
const method = ref('GET')
const url = ref('')
const timeoutMs = ref(30000)
const bodyType = ref('json')
const body = ref('')
const headers = ref<Array<{ key: string; value: string }>>([])
const loading = ref(false)

// ============ 响应 ============
const responseTab = ref('body')
const responseStatus = ref(0)
const responseStatusText = ref('')
const responseHeaders = ref<Record<string, string>>({})
const responseBody = ref('')
const responseTime = ref(0)
const responseSize = ref(0)
const errorMsg = ref('')

const statusClass = computed(() => {
  if (responseStatus.value < 300) return 'status-success'
  if (responseStatus.value < 400) return 'status-redirect'
  if (responseStatus.value < 500) return 'status-client-error'
  return 'status-server-error'
})

const formattedResponseBody = computed(() => {
  if (bodyType.value === 'json' || responseTab.value === 'body') {
    try {
      return JSON.stringify(JSON.parse(responseBody.value), null, 2)
    } catch {
      return responseBody.value
    }
  }
  return responseBody.value
})

// ============ 请求头操作 ============
const addHeader = () => {
  headers.value.push({ key: '', value: '' })
}

const removeHeader = (idx: number) => {
  headers.value.splice(idx, 1)
}

const handleClearBody = () => {
  body.value = ''
}

const handlePasteBody = async () => {
  try {
    const text = await navigator.clipboard.readText()
    body.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

// ============ 发送请求 ============
const handleSend = async () => {
  if (!url.value.trim()) {
    ElMessage.warning('请输入请求 URL')
    return
  }

  loading.value = true
  errorMsg.value = ''
  responseStatus.value = 0

  try {
    const headerMap: Record<string, string> = {}
    for (const h of headers.value) {
      if (h.key.trim()) {
        headerMap[h.key.trim()] = h.value
      }
    }

    const result = await invoke('send_http_request', {
      request: {
        method: method.value,
        url: url.value.trim(),
        headers: headerMap,
        body: body.value || null,
        bodyType: bodyType.value,
        timeoutMs: timeoutMs.value
      }
    }) as any

    responseStatus.value = result.status
    responseStatusText.value = result.status_text
    responseHeaders.value = result.headers
    responseBody.value = result.body
    responseTime.value = result.time_ms
    responseSize.value = result.size_bytes

    ElMessage.success(`请求完成: ${result.status} ${result.status_text}`)
    store.addHistory({
      tool: 'http',
      action: `${method.value} ${url.value.slice(0, 50)}`,
      inputPreview: `${method.value} ${url.value.slice(0, 30)}`,
      outputPreview: `${result.status} ${result.status_text}`
    })
  } catch (e: any) {
    errorMsg.value = e.message || '请求失败'
    ElMessage.error('请求失败')
  } finally {
    loading.value = false
  }
}

// ============ 工具方法 ============
const handleCopyResponse = async () => {
  if (!responseBody.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(responseBody.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<style scoped>
/* ===== 一级卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

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

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}

/* ===== 请求栏 ===== */
.request-bar {
  display: flex;
  align-items: center;
}

.timeout-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 4px;
}

/* ===== 请求头行 ===== */
.header-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.header-row:last-child {
  margin-bottom: 0;
}

.empty-hint {
  text-align: center;
  padding: 12px 0;
  color: var(--text-muted);
  font-size: 13px;
}

/* ===== 响应状态栏 ===== */
.response-status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
}

.status-code {
  font-size: 14px;
  font-weight: 600;
}

.status-success { color: #22c55e; }
.status-redirect { color: #f59e0b; }
.status-client-error { color: #ef4444; }
.status-server-error { color: #dc2626; }

.response-meta {
  font-size: 12px;
  color: var(--text-secondary);
}

/* ===== 响应 Tabs ===== */
.response-tabs {
  margin-right: 8px;
}

.response-tabs :deep(.el-tabs__header) {
  margin: 0;
  border: none;
}

.response-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.response-tabs :deep(.el-tabs__item) {
  font-size: 13px;
  padding: 0 12px;
  height: 28px;
  line-height: 28px;
}

/* ===== 响应头 ===== */
.response-header-row {
  display: flex;
  padding: 6px 0;
  border-bottom: 1px solid var(--border-color);
  font-size: 13px;
}

.response-header-row:last-child {
  border-bottom: none;
}

.header-key {
  color: var(--accent-cyan);
  font-weight: 500;
  min-width: 160px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.header-value {
  color: var(--text-primary);
  word-break: break-all;
  flex: 1;
  margin-left: 12px;
}

/* ===== 响应文本 ===== */
.response-textarea :deep(.el-textarea__inner) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  background: var(--bg-input);
  line-height: 1.6;
}

/* ===== 加载状态 ===== */
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--text-secondary);
}

.loading-state .el-icon {
  font-size: 20px;
}

/* ===== 错误/空状态 ===== */
.error-message {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}
</style>
