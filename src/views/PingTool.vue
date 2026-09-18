<template>
  <div class="tool-container fill-height">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="ping-tabs">
        <el-tab-pane label="Ping" name="ping" />
        <el-tab-pane label="Traceroute" name="tracert" />
      </el-tabs>
    </div>

    <!-- Ping Tab -->
    <div v-if="activeTab === 'ping'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Ping 测试</span>
        <el-tooltip placement="top" effect="dark">
          <template #content>
            <div class="tooltip-content">
              <p>向目标主机发送 ICMP 回显请求，测量网络延迟与丢包率</p>
            </div>
          </template>
          <el-icon class="hint-icon"><QuestionFilled /></el-icon>
        </el-tooltip>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">目标</div>
            <el-input
              v-model="pingHost"
              size="small"
              placeholder="主机名或 IP 地址，如 baidu.com"
              style="width: 280px"
              @keyup.enter="startPing"
            />
          </div>
          <div class="action-group">
            <div class="group-label">次数</div>
            <el-input-number v-model="pingCount" size="small" :min="1" :max="100" controls-position="right" style="width: 110px" />
            <div class="group-label" style="margin-left: 12px">超时(ms)</div>
            <el-input-number v-model="pingTimeout" size="small" :min="100" :max="10000" :step="100" controls-position="right" style="width: 110px" />
          </div>
          <div class="action-group">
            <div class="group-label">包大小</div>
            <el-input-number v-model="pingSize" size="small" :min="0" :max="65500" controls-position="right" style="width: 110px" />
            <div class="group-buttons" style="margin-left: 12px">
              <el-button type="primary" size="small" :loading="pingRunning" :disabled="!pingHost.trim()" @click="startPing">
                {{ pingRunning ? 'Ping 中...' : '开始 Ping' }}
              </el-button>
              <el-button type="danger" size="small" :disabled="!pingRunning" @click="stopPing">停止</el-button>
              <el-button size="small" @click="clearPingResults">清空</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Ping 统计卡片 -->
    <div v-if="activeTab === 'ping' && (pingReplies.length || pingStats)" class="tool-card">
      <div class="card-header">
        <span class="card-title">统计信息</span>
      </div>
      <div class="card-body">
        <div class="stats-row">
          <div class="stat-card">
            <span class="stat-number">{{ pingStats ? pingStats.sent : pingReplies.length }}</span>
            <span class="stat-label">已发送</span>
          </div>
          <div class="stat-card">
            <span class="stat-number success">{{ pingStats ? pingStats.received : pingReplies.filter(r => !r.timeout).length }}</span>
            <span class="stat-label">已接收</span>
          </div>
          <div class="stat-card">
            <span class="stat-number danger">{{ pingStats ? pingStats.lost : pingReplies.filter(r => r.timeout).length }}</span>
            <span class="stat-label">丢失</span>
          </div>
          <div class="stat-card">
            <span class="stat-number" :class="lossClass">{{ pingStats ? pingStats.loss_percent.toFixed(1) : calcLossPercent() }}%</span>
            <span class="stat-label">丢包率</span>
          </div>
          <div v-if="pingStats" class="stat-card">
            <span class="stat-number">{{ pingStats.min_ms }}</span>
            <span class="stat-label">最短(ms)</span>
          </div>
          <div v-if="pingStats" class="stat-card">
            <span class="stat-number">{{ pingStats.max_ms }}</span>
            <span class="stat-label">最长(ms)</span>
          </div>
          <div v-if="pingStats" class="stat-card">
            <span class="stat-number accent">{{ pingStats.avg_ms }}</span>
            <span class="stat-label">平均(ms)</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Ping 结果列表 -->
    <div v-if="activeTab === 'ping'" class="tool-card fill-height">
      <div class="card-header">
        <span class="card-title">回复列表 ({{ pingReplies.length }})</span>
      </div>
      <div class="card-body fill-body">
        <div v-if="pingError" class="error-message">{{ pingError }}</div>
        <el-empty v-if="!pingReplies.length && !pingRunning" description="点击「开始 Ping」测试网络连通性" />
        <div v-else class="reply-list">
          <div
            v-for="reply in pingReplies"
            :key="reply.seq"
            class="reply-item"
            :class="{ timeout: reply.timeout }"
          >
            <span class="reply-seq">#{{ reply.seq }}</span>
            <template v-if="reply.timeout">
              <el-tag type="danger" size="small">请求超时</el-tag>
            </template>
            <template v-else>
              <span class="reply-from mono-text">{{ reply.from }}</span>
              <span class="reply-meta">字节={{ reply.bytes }}</span>
              <span class="reply-meta">时间={{ reply.time_ms }}ms</span>
              <span class="reply-meta">TTL={{ reply.ttl }}</span>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- Traceroute Tab -->
    <div v-if="activeTab === 'tracert'" class="tool-card">
      <div class="card-header">
        <span class="card-title">路由跟踪</span>
        <el-tooltip placement="top" effect="dark">
          <template #content>
            <div class="tooltip-content">
              <p>跟踪数据包到达目标主机所经过的路由路径</p>
            </div>
          </template>
          <el-icon class="hint-icon"><QuestionFilled /></el-icon>
        </el-tooltip>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">目标</div>
            <el-input
              v-model="tracertHost"
              size="small"
              placeholder="主机名或 IP 地址，如 baidu.com"
              style="width: 280px"
              @keyup.enter="startTracert"
            />
          </div>
          <div class="action-group">
            <div class="group-label">最大跳数</div>
            <el-input-number v-model="tracertMaxHops" size="small" :min="1" :max="30" controls-position="right" style="width: 110px" />
            <div class="group-label" style="margin-left: 12px">超时(ms)</div>
            <el-input-number v-model="tracertTimeout" size="small" :min="100" :max="10000" :step="100" controls-position="right" style="width: 110px" />
          </div>
          <div class="action-group">
            <div class="group-label">操作</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" :loading="tracertRunning" :disabled="!tracertHost.trim()" @click="startTracert">
                {{ tracertRunning ? '跟踪中...' : '开始跟踪' }}
              </el-button>
              <el-button type="danger" size="small" :disabled="!tracertRunning" @click="stopTracert">停止</el-button>
              <el-button size="small" @click="clearTracertResults">清空</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Traceroute 结果 -->
    <div v-if="activeTab === 'tracert'" class="tool-card fill-height">
      <div class="card-header">
        <span class="card-title">路由路径 ({{ traceHops.length }} 跳)</span>
      </div>
      <div class="card-body fill-body">
        <div v-if="tracertError" class="error-message">{{ tracertError }}</div>
        <el-empty v-if="!traceHops.length && !tracertRunning" description="点击「开始跟踪」查看路由路径" />
        <div v-else class="trace-table">
          <div class="trace-row trace-header">
            <span class="col-hop">跳数</span>
            <span class="col-rtt">延迟1</span>
            <span class="col-rtt">延迟2</span>
            <span class="col-rtt">延迟3</span>
            <span class="col-ip">IP 地址</span>
            <span class="col-host">主机名</span>
          </div>
          <div
            v-for="hop in traceHops"
            :key="hop.hop"
            class="trace-row"
            :class="{ timeout: hop.timeout }"
          >
            <span class="col-hop">{{ hop.hop }}</span>
            <span class="col-rtt" :class="{ danger: hop.rtt1 === '*' }">{{ hop.rtt1 }}</span>
            <span class="col-rtt" :class="{ danger: hop.rtt2 === '*' }">{{ hop.rtt2 }}</span>
            <span class="col-rtt" :class="{ danger: hop.rtt3 === '*' }">{{ hop.rtt3 }}</span>
            <span class="col-ip mono-text">{{ hop.ip || '-' }}</span>
            <span class="col-host">{{ hop.hostname }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { listen } from '@tauri-apps/api/event'
import {
  pingStart,
  pingCancel,
  pingEcho,
  tracertStart,
  tracertCancel,
  type PingReply,
  type PingStats,
  type TraceHop,
} from '@/utils/pingClient'

// ============ Tab 状态 ============
const activeTab = ref('ping')

// ============ Ping 状态 ============
const pingHost = ref('')
const pingCount = ref(4)
const pingTimeout = ref(1000)
const pingSize = ref(32)
const pingRunning = ref(false)
const pingReplies = ref<PingReply[]>([])
const pingStats = ref<PingStats | null>(null)
const pingError = ref('')

// ============ Traceroute 状态 ============
const tracertHost = ref('')
const tracertMaxHops = ref(30)
const tracertTimeout = ref(1000)
const tracertRunning = ref(false)
const traceHops = ref<TraceHop[]>([])
const tracertError = ref('')

// ============ 计算属性 ============
const lossClass = computed(() => {
  if (!pingStats.value) return ''
  if (pingStats.value.loss_percent === 0) return 'success'
  if (pingStats.value.loss_percent < 50) return 'warning'
  return 'danger'
})

const calcLossPercent = () => {
  if (!pingReplies.value.length) return '0.0'
  const lost = pingReplies.value.filter(r => r.timeout).length
  return ((lost / pingReplies.value.length) * 100).toFixed(1)
}

// ============ 事件监听 ============
let unlistenPing: (() => void) | null = null
let unlistenTracert: (() => void) | null = null

onMounted(async () => {
  try {
    unlistenPing = await listen('ping-event', (event: any) => {
      const payload = event.payload
      switch (payload.event) {
        case 'reply':
          pingReplies.value.push(payload.reply)
          break
        case 'stats':
          pingStats.value = payload.stats
          break
        case 'error':
          pingError.value = payload.message
          ElMessage.error(payload.message)
          break
        case 'complete':
          pingRunning.value = false
          break
      }
    })

    unlistenTracert = await listen('tracert-event', (event: any) => {
      const payload = event.payload
      switch (payload.event) {
        case 'hop':
          traceHops.value.push(payload.hop)
          break
        case 'error':
          tracertError.value = payload.message
          ElMessage.error(payload.message)
          break
        case 'complete':
          tracertRunning.value = false
          break
      }
    })
  } catch (e) {
    console.error('[PingTool] 事件监听注册失败:', e)
  }
})

onUnmounted(() => {
  if (unlistenPing) unlistenPing()
  if (unlistenTracert) unlistenTracert()
})

// ============ Ping 操作 ============
const startPing = async () => {
  const host = pingHost.value.trim()
  if (!host) {
    ElMessage.warning('请输入目标主机名或 IP')
    return
  }
  pingReplies.value = []
  pingStats.value = null
  pingError.value = ''
  pingRunning.value = true
  try {
    // 先测试 invoke 链是否正常
    const echo = await pingEcho(host, pingCount.value || 4, pingTimeout.value || 1000, pingSize.value || 32)
    console.log('[PingTool] echo result:', echo)
    await pingStart(host, pingCount.value || 4, pingTimeout.value || 1000, pingSize.value || 32)
  } catch (e: any) {
    const msg = e?.toString?.() || String(e) || '未知错误'
    console.error('[PingTool] startPing error:', e)
    pingError.value = msg
    pingRunning.value = false
    alert('Ping 调用失败: ' + msg)
  }
}

const stopPing = async () => {
  try {
    await pingCancel()
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const clearPingResults = () => {
  pingReplies.value = []
  pingStats.value = null
  pingError.value = ''
}

// ============ Traceroute 操作 ============
const startTracert = async () => {
  if (!tracertHost.value.trim()) {
    ElMessage.warning('请输入目标主机名或 IP')
    return
  }
  traceHops.value = []
  tracertError.value = ''
  tracertRunning.value = true
  try {
    await tracertStart(tracertHost.value.trim(), tracertMaxHops.value, tracertTimeout.value)
  } catch (e: any) {
    tracertError.value = String(e)
    tracertRunning.value = false
    ElMessage.error(String(e))
  }
}

const stopTracert = async () => {
  try {
    await tracertCancel()
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const clearTracertResults = () => {
  traceHops.value = []
  tracertError.value = ''
}
</script>

<style scoped>
.ping-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .ping-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.ping-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.ping-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.ping-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.ping-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.ping-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* 统计卡片 */
.stats-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-radius: 8px;
  min-width: 90px;
}

.stat-number {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
}

.stat-number.success { color: var(--accent-green, #10b981); }
.stat-number.danger { color: var(--accent-red, #ef4444); }
.stat-number.warning { color: var(--accent-orange, #f59e0b); }
.stat-number.accent { color: var(--accent-cyan); }

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

/* Ping 回复列表 */
.fill-body {
  overflow-y: auto;
  max-height: 500px;
}

.reply-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.reply-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  font-size: 13px;
}

.reply-item.timeout {
  background: rgba(239, 68, 68, 0.1);
}

.reply-seq {
  font-weight: 600;
  color: var(--accent-cyan);
  min-width: 32px;
}

.reply-from {
  font-weight: 500;
  color: var(--text-primary);
  min-width: 120px;
}

.reply-meta {
  color: var(--text-secondary);
}

.mono-text {
  font-family: 'Consolas', 'Monaco', monospace;
}

/* Traceroute 表格 */
.trace-table {
  display: flex;
  flex-direction: column;
}

.trace-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  font-size: 13px;
  gap: 8px;
}

.trace-row.trace-header {
  background: var(--bg-secondary);
  font-weight: 600;
  color: var(--text-secondary);
  position: sticky;
  top: 0;
  z-index: 1;
}

.trace-row.timeout {
  background: rgba(239, 68, 68, 0.08);
}

.col-hop { width: 50px; text-align: center; flex-shrink: 0; }
.col-rtt { width: 80px; text-align: center; flex-shrink: 0; }
.col-rtt.danger { color: var(--accent-red, #ef4444); }
.col-ip { flex: 1; min-width: 120px; }
.col-host { flex: 1; min-width: 120px; color: var(--text-secondary); }

.error-message {
  color: var(--accent-red, #ef4444);
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 6px;
  font-size: 13px;
}

.tooltip-content p {
  margin: 0;
  font-size: 12px;
}
</style>
