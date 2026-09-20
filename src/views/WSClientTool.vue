<template>
  <div class="tool-container ws-container">
    <!-- 连接参数 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">WebSocket</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>原生 WebSocket 客户端，支持 ws:// 和 wss://</p>
                <p>可配置子协议和自定义请求头（连接时附加）</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="action-bar">
          <div class="conn-status">
            <span class="status-dot" :class="statusClass"></span>
            <span class="status-text">{{ statusText }}</span>
          </div>
          <el-button
            :type="connected ? 'danger' : 'primary'"
            size="small"
            @click="toggleConnection"
            :loading="connecting"
          >
            {{ connected ? '断开' : '连接' }}
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="request-bar">
          <el-input
            v-model="url"
            placeholder="ws://localhost:8080/ws 或 wss://api.example.com/socket"
            size="default"
            @keyup.enter="handleConnect"
          />
        </div>
        <div class="config-grid">
          <div class="config-group">
            <div class="group-label">子协议</div>
            <el-input
              v-model="protocolsInput"
              size="small"
              placeholder="多个用逗号分隔，可选"
              style="width: 100%"
            />
          </div>
          <div class="config-group">
            <div class="group-label">重连策略</div>
            <el-switch
              v-model="autoReconnect"
              active-text="自动重连"
              inactive-text="手动"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 自定义 Headers（仅部分浏览器支持，作展示用） -->
    <div class="tool-card" v-if="showHeaders">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">自定义 Headers</span>
          <span class="header-note">注：浏览器 WebSocket API 不支持自定义 Header，此处预留备用</span>
        </div>
        <el-button size="small" @click="addHeader">+ 添加</el-button>
      </div>
      <div class="card-body">
        <div v-for="(h, idx) in headers" :key="idx" class="header-row">
          <el-input v-model="h.key" placeholder="Header" size="small" style="width: 200px" />
          <el-input v-model="h.value" placeholder="Value" size="small" style="flex: 1; margin-left: 8px" />
          <el-button size="small" type="danger" :icon="Delete" circle @click="removeHeader(idx)" style="margin-left: 8px" />
        </div>
        <div v-if="headers.length === 0" class="empty-hint">暂无 Header</div>
      </div>
    </div>

    <!-- 消息发送 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">发送消息</span>
        <div class="action-bar">
          <el-select v-model="msgTemplate" size="small" placeholder="快速模板" clearable style="width: 160px" @change="onTemplateChange">
            <el-option label="ping" value="ping" />
            <el-option label='{"type":"hello"}' value='{"type":"hello"}' />
            <el-option label='{"action":"subscribe"}' value='{"action":"subscribe"}' />
          </el-select>
          <el-button size="small" @click="sendMessage" :disabled="!connected">发送</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="messageInput"
          type="textarea"
          :rows="6"
          placeholder="输入要发送的消息内容，支持文本和 JSON"
          resize="vertical"
        />
        <div class="action-grid" style="margin-top: 8px">
          <div class="action-group">
            <div class="group-label">快捷</div>
            <div class="group-buttons">
              <el-button size="small" @click="messageInput = ''">清空</el-button>
              <el-button size="small" @click="messageInput = lastMessage" :disabled="!lastMessage">上一条</el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">格式化</div>
            <div class="group-buttons">
              <el-button size="small" @click="formatJson" :disabled="!messageInput.trim() || !isValidJson">格式化 JSON</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 消息日志 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">消息日志</span>
          <span class="header-count">{{ logs.length }} 条</span>
        </div>
        <div class="action-bar">
          <el-checkbox v-model="autoScroll" size="small">自动滚动</el-checkbox>
          <el-button size="small" @click="clearLogs">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div ref="logContainer" class="log-container">
          <div
            v-for="(log, idx) in logs"
            :key="idx"
            class="log-item"
            :class="'log-' + log.type"
          >
            <span class="log-time">{{ log.time }}</span>
            <span class="log-dir" :class="'dir-' + log.dir">{{ log.dir === 'send' ? '→ 发送' : log.dir === 'recv' ? '← 接收' : '● 系统' }}</span>
            <span class="log-content" :title="log.content">{{ log.content }}</span>
          </div>
          <div v-if="logs.length === 0" class="empty-hint">暂无消息，连接后开始收发</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Delete, QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

// ponytail: store 暂未使用，保留以备后续扩展
void useToolboxStore()

// ============ 状态 ============
let ws: WebSocket | null = null
const url = ref('ws://localhost:8080/ws')
const protocolsInput = ref('')
const autoReconnect = ref(false)
const showHeaders = ref(false)

const headers = ref<{ key: string; value: string }[]>([
  { key: 'Authorization', value: '' }
])

const connected = ref(false)
const connecting = ref(false)
const lastMessage = ref('')

// 发送区
const messageInput = ref('')
const msgTemplate = ref('')

// 日志
interface LogItem {
  type: 'info' | 'error' | 'msg'
  dir: 'send' | 'recv' | 'sys'
  time: string
  content: string
}
const logs = ref<LogItem[]>([])
const autoScroll = ref(true)
const logContainer = ref<HTMLElement | null>(null)

let reconnectTimer: ReturnType<typeof setTimeout> | null = null

// ============ 计算属性 ============
const statusText = computed(() => {
  if (connecting.value) return '连接中...'
  if (connected.value) return '已连接'
  return '未连接'
})

const statusClass = computed(() => ({
  connected: connected.value,
  connecting: connecting.value
}))

const isValidJson = computed(() => {
  try { JSON.parse(messageInput.value); return true } catch { return false }
})

// ============ 工具函数 ============
const now = () => {
  const d = new Date()
  return d.toTimeString().slice(0, 8) + '.' + String(d.getMilliseconds()).padStart(3, '0')
}

const addLog = (type: LogItem['type'], dir: LogItem['dir'], content: string) => {
  logs.value.push({ type, dir, time: now(), content })
  if (logs.value.length > 500) logs.value = logs.value.slice(-500)
  if (autoScroll.value) {
    nextTick(() => {
      if (logContainer.value) logContainer.value.scrollTop = logContainer.value.scrollHeight
    })
  }
}

// ============ 连接控制 ============
const parseProtocols = () => {
  const raw = protocolsInput.value.trim()
  if (!raw) return undefined
  return raw.split(',').map(s => s.trim()).filter(Boolean)
}

const handleConnect = async () => {
  if (connected.value) {
    handleDisconnect()
    return
  }
  if (!url.value.trim()) {
    ElMessage.warning('请输入 WebSocket 地址')
    return
  }
  if (!/^wss?:\/\//i.test(url.value.trim())) {
    ElMessage.warning('地址需以 ws:// 或 wss:// 开头')
    return
  }

  connecting.value = true
  const protocols = parseProtocols()

  try {
    ws = protocols ? new WebSocket(url.value.trim(), protocols) : new WebSocket(url.value.trim())
  } catch (e: any) {
    connecting.value = false
    addLog('error', 'sys', `创建失败: ${e.message}`)
    ElMessage.error(e.message)
    return
  }

  ws.onopen = () => {
    connecting.value = false
    connected.value = true
    addLog('info', 'sys', `已连接: ${url.value}`)
  }

  ws.onmessage = (ev) => {
    const content = typeof ev.data === 'string' ? ev.data : `[二进制数据 ${ev.data.size} bytes]`
    addLog('msg', 'recv', content)
  }

  ws.onerror = () => {
    addLog('error', 'sys', '连接出错')
  }

  ws.onclose = (ev) => {
    connecting.value = false
    connected.value = false
    addLog('info', 'sys', `已关闭 (code=${ev.code}${ev.reason ? ', reason=' + ev.reason : ''})`)
    ws = null
    // 自动重连
    if (autoReconnect.value) {
      addLog('info', 'sys', '3 秒后自动重连...')
      reconnectTimer = setTimeout(handleConnect, 3000)
    }
  }
}

const handleDisconnect = () => {
  if (reconnectTimer) clearTimeout(reconnectTimer)
  autoReconnect.value = false
  ws?.close()
  ws = null
}

const toggleConnection = () => {
  if (connected.value || connecting.value) {
    handleDisconnect()
  } else {
    handleConnect()
  }
}

const addHeader = () => headers.value.push({ key: '', value: '' })
const removeHeader = (idx: number) => headers.value.splice(idx, 1)

// ============ 消息发送 ============
const sendMessage = () => {
  if (!connected.value || !ws) {
    ElMessage.warning('未连接')
    return
  }
  const msg = messageInput.value
  if (!msg.trim()) {
    ElMessage.warning('消息内容不能为空')
    return
  }
  try {
    ws.send(msg)
    addLog('msg', 'send', msg)
    lastMessage.value = msg
  } catch (e: any) {
    addLog('error', 'sys', `发送失败: ${e.message}`)
    ElMessage.error(e.message)
  }
}

const formatJson = () => {
  if (!isValidJson.value) {
    ElMessage.warning('不是有效 JSON')
    return
  }
  messageInput.value = JSON.stringify(JSON.parse(messageInput.value), null, 2)
}

const onTemplateChange = () => {
  if (msgTemplate.value) {
    messageInput.value = msgTemplate.value
  }
}

// ============ 日志控制 ============
const clearLogs = () => {
  logs.value = []
}

// ============ 清理 ============
onUnmounted(() => {
  handleDisconnect()
})

// ============ 历史记录 ============
watch([connected, messageInput], () => {
  // 只做状态追踪，不自动存历史
}, { deep: true })
</script>

<style scoped>
.ws-container .request-bar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ws-container .config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-top: 12px;
}

.ws-container .config-group .group-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.ws-container .header-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.ws-container .conn-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-right: 12px;
}

.ws-container .status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-red);
  transition: all 0.3s;
}

.ws-container .status-dot.connected {
  background: var(--accent-green);
  box-shadow: 0 0 8px var(--accent-green);
}

.ws-container .status-dot.connecting {
  background: var(--accent-yellow);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.ws-container .status-text {
  font-size: 13px;
  color: var(--text-secondary);
}

.ws-container .header-note {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 8px;
}

.ws-container .header-count {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 8px;
}

.ws-container .log-container {
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  padding: 8px;
  max-height: 420px;
  overflow-y: auto;
  font-family: 'Consolas', 'Menlo', monospace;
  font-size: 12px;
}

.ws-container .log-item {
  display: flex;
  gap: 8px;
  padding: 4px 6px;
  border-bottom: 1px solid var(--border-color);
  word-break: break-all;
}

.ws-container .log-item:last-child {
  border-bottom: none;
}

.ws-container .log-dir {
  min-width: 54px;
  font-weight: 600;
  font-size: 11px;
}

.ws-container .log-dir.dir-send {
  color: var(--accent-cyan);
}

.ws-container .log-dir.dir-recv {
  color: var(--accent-green);
}

.ws-container .log-dir.dir-sys {
  color: var(--text-secondary);
}

.ws-container .log-content {
  flex: 1;
  color: var(--text-primary);
  white-space: pre-wrap;
}

.ws-container .log-item.log-error .log-content {
  color: var(--accent-red);
}

.ws-container .log-time {
  color: var(--text-secondary);
  font-size: 11px;
  min-width: 110px;
}
</style>
