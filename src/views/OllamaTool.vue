<template>
  <div class="tool-container">
    <!-- 服务状态条 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Ollama 服务</span>
        </div>
        <div class="card-actions">
          <div class="status-badge" :class="serviceStatus">
            <span class="status-dot"></span>
            <span>{{ serviceText }}</span>
            <span v-if="ollamaVersion" class="version">v{{ ollamaVersion }}</span>
          </div>
          <el-input
            v-model="hostInput"
            size="small"
            placeholder="http://localhost:11434"
            style="width: 200px; margin-left: 12px"
            @keyup.enter="checkService"
          />
          <el-button type="primary" size="small" :loading="checking" @click="checkService">检测</el-button>
        </div>
      </div>
      <div v-if="!serviceOnline" class="card-body">
        <el-alert
          title="未检测到 Ollama 服务"
          type="warning"
          :closable="false"
          show-icon
        >
          <template #default>
            <p>请先安装并启动 Ollama，安装后默认运行在 <code>http://localhost:11434</code></p>
            <p>安装方式：</p>
            <ul class="install-tips">
              <li>1. 访问 <a href="https://ollama.com/download" target="_blank">https://ollama.com/download</a> 下载安装包</li>
              <li>2. 或使用命令：<code>winget install Ollama.Ollama</code></li>
              <li>3. 安装完成后启动 Ollama，再点击"检测"按钮</li>
            </ul>
          </template>
        </el-alert>
      </div>
    </div>

    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="ollama-tabs">
        <el-tab-pane label="模型管理" name="models" />
        <el-tab-pane label="运行监控" name="running" />
        <el-tab-pane label="对话测试" name="chat" />
      </el-tabs>
    </div>

    <!-- Tab 1: 模型管理 -->
    <div v-if="activeTab === 'models'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">模型列表</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>已下载到本地的模型</template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" :loading="loadingModels" @click="loadModels">刷新</el-button>
        </div>
      </div>
      <div class="card-body">
        <!-- 下载新模型 -->
        <div class="action-grid" style="margin-bottom: 16px">
          <div class="action-group">
            <div class="group-label">拉取新模型</div>
            <div class="group-buttons">
              <el-input
                v-model="pullName"
                size="small"
                placeholder="模型名称，如 llama3.1:8b"
                style="width: 260px"
                :disabled="pulling"
              />
              <el-button
                type="primary"
                size="small"
                :loading="pulling"
                :disabled="!pullName.trim()"
                @click="pullModel"
              >拉取</el-button>
            </div>
          </div>
        </div>

        <!-- 拉取进度 -->
        <div v-if="pulling" class="pull-progress">
          <div class="pull-status">{{ pullStatusText }}</div>
          <el-progress
            v-if="pullTotal > 0"
            :percentage="pullPercent"
            :stroke-width="6"
            :status="pullStatus === 'success' ? 'success' : ''"
          />
        </div>

        <!-- 模型列表 -->
        <el-table :data="models" size="small" stripe border style="width: 100%">
          <el-table-column prop="name" label="模型名称" min-width="200" />
          <el-table-column label="大小" width="120">
            <template #default="{ row }">{{ formatSize(row.size) }}</template>
          </el-table-column>
          <el-table-column label="参数" width="100">
            <template #default="{ row }">{{ row.details?.parameter_size || '-' }}</template>
          </el-table-column>
          <el-table-column label="家族" width="120">
            <template #default="{ row }">{{ row.details?.family || '-' }}</template>
          </el-table-column>
          <el-table-column prop="modified_at" label="修改时间" width="170" />
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button size="small" @click="showDetail(row)">详情</el-button>
              <el-button size="small" type="danger" @click="deleteModel(row)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>

        <el-empty v-if="!loadingModels && models.length === 0" description="暂无模型，拉取一个开始使用" />
      </div>
    </div>

    <!-- Tab 2: 运行监控 -->
    <div v-if="activeTab === 'running'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">运行中的模型</span>
        </div>
        <div class="card-actions">
          <el-button size="small" :loading="loadingPs" @click="loadPs">刷新</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="runningModels" size="small" stripe border style="width: 100%">
          <el-table-column prop="name" label="模型名称" min-width="200" />
          <el-table-column label="占用内存" width="140">
            <template #default="{ row }">{{ formatSize(row.size) }}</template>
          </el-table-column>
          <el-table-column label="占用显存" width="140">
            <template #default="{ row }">{{ formatSize(row.size_vram) }}</template>
          </el-table-column>
          <el-table-column prop="expires_at" label="过期时间" width="200" />
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" type="warning" @click="stopModel(row)">停止</el-button>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="!loadingPs && runningModels.length === 0" description="没有运行中的模型" />
      </div>
    </div>

    <!-- Tab 3: 对话测试 -->
    <div v-if="activeTab === 'chat'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">对话测试</span>
        </div>
        <div class="card-actions">
          <el-select v-model="chatModel" size="small" placeholder="选择模型" style="width: 220px" :disabled="chatting">
            <el-option
              v-for="m in models"
              :key="m.name"
              :label="m.name"
              :value="m.name"
            />
          </el-select>
          <el-button size="small" @click="loadModels" :disabled="chatting">刷新模型</el-button>
          <el-button size="small" type="danger" plain @click="clearChat" :disabled="chatting">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="chat-container">
          <div class="chat-messages" ref="chatMessagesRef">
            <div v-for="(msg, i) in chatMessages" :key="i" class="chat-message" :class="msg.role">
              <div class="msg-role">{{ msg.role === 'user' ? '我' : 'AI' }}</div>
              <div class="msg-content">{{ msg.content }}</div>
            </div>
            <div v-if="chatting" class="chat-message assistant">
              <div class="msg-role">AI</div>
              <div class="msg-content streaming">{{ streamingContent }}<span class="cursor">▋</span></div>
            </div>
          </div>
          <div class="chat-input-area">
            <el-input
              v-model="chatInput"
              type="textarea"
              :rows="2"
              placeholder="输入消息，Ctrl+Enter 发送"
              :disabled="chatting || !chatModel"
              @keydown.ctrl.enter="sendChat"
            />
            <el-button
              type="primary"
              :loading="chatting"
              :disabled="!chatInput.trim() || !chatModel || chatting"
              @click="sendChat"
            >发送</el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 模型详情弹窗 -->
    <el-dialog v-model="detailVisible" title="模型详情" width="640px">
      <pre class="detail-pre">{{ modelDetail }}</pre>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const activeTab = ref('models')

// ===== 服务状态 =====
const hostInput = ref('http://localhost:11434')
const checking = ref(false)
const ollamaVersion = ref('')
const serviceStatus = ref<'unknown' | 'online' | 'offline'>('unknown')
const serviceOnline = computed(() => serviceStatus.value === 'online')
const serviceText = computed(() => {
  switch (serviceStatus.value) {
    case 'online': return '运行中'
    case 'offline': return '未连接'
    default: return '检测中'
  }
})

const checkService = async () => {
  checking.value = true
  serviceStatus.value = 'unknown'
  try {
    const res = await invoke<{ version: string }>('ollama_check', { host: hostInput.value || undefined })
    ollamaVersion.value = res.version
    serviceStatus.value = 'online'
    loadModels()
  } catch (e) {
    ollamaVersion.value = ''
    serviceStatus.value = 'offline'
  } finally {
    checking.value = false
  }
}

// ===== 模型列表 =====
const loadingModels = ref(false)
const models = ref<any[]>([])

const loadModels = async () => {
  if (!serviceOnline.value) return
  loadingModels.value = true
  try {
    const res = await invoke<{ models: any[] }>('ollama_list', { host: hostInput.value || undefined })
    models.value = res.models
  } catch (e) {
    ElMessage.error('加载模型列表失败: ' + String(e))
  } finally {
    loadingModels.value = false
  }
}

const formatSize = (bytes: number): string => {
  if (!bytes) return '-'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(i === 0 ? 0 : 2)} ${units[i]}`
}

// ===== 拉取模型 =====
const pullName = ref('')
const pulling = ref(false)
const pullStatus = ref('')
const pullTotal = ref(0)
const pullCompleted = ref(0)
const pullPercent = computed(() =>
  pullTotal.value > 0 ? Math.round((pullCompleted.value / pullTotal.value) * 100) : 0
)
const pullStatusText = computed(() => {
  if (!pullStatus.value) return '准备中...'
  return pullStatus.value
})

let pullUnlisten: UnlistenFn | null = null
let pullDoneUnlisten: UnlistenFn | null = null

const pullModel = async () => {
  if (!pullName.value.trim()) {
    ElMessage.warning('请输入模型名称')
    return
  }
  pulling.value = true
  pullStatus.value = ''
  pullTotal.value = 0
  pullCompleted.value = 0

  pullUnlisten = await listen('ollama-pull-progress', (event) => {
    const payload = event.payload as any
    pullStatus.value = payload.status
    pullTotal.value = payload.total || 0
    pullCompleted.value = payload.completed || 0
  })

  pullDoneUnlisten = await listen('ollama-pull-done', () => {
    pulling.value = false
    pullStatus.value = 'success'
    ElMessage.success(`模型 ${pullName.value} 拉取完成`)
    pullName.value = ''
    loadModels()
    cleanupPullListeners()
  })

  try {
    await invoke('ollama_pull', { host: hostInput.value || undefined, name: pullName.value.trim() })
  } catch (e) {
    pulling.value = false
    ElMessage.error('拉取失败: ' + String(e))
    cleanupPullListeners()
  }
}

const cleanupPullListeners = () => {
  if (pullUnlisten) { pullUnlisten(); pullUnlisten = null }
  if (pullDoneUnlisten) { pullDoneUnlisten(); pullDoneUnlisten = null }
}

// ===== 删除模型 =====
const deleteModel = async (row: any) => {
  try {
    await ElMessageBox.confirm(`确定要删除模型 "${row.name}" 吗？此操作不可恢复。`, '删除确认', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
  } catch { return }

  try {
    await invoke('ollama_delete', { host: hostInput.value || undefined, name: row.name })
    ElMessage.success('模型已删除')
    loadModels()
  } catch (e) {
    ElMessage.error('删除失败: ' + String(e))
  }
}

// ===== 模型详情 =====
const detailVisible = ref(false)
const modelDetail = ref('')

const showDetail = async (row: any) => {
  try {
    const res = await invoke<any>('ollama_show', { host: hostInput.value || undefined, name: row.name })
    modelDetail.value = JSON.stringify(res, null, 2)
    detailVisible.value = true
  } catch (e) {
    ElMessage.error('获取详情失败: ' + String(e))
  }
}

// ===== 运行监控 =====
const loadingPs = ref(false)
const runningModels = ref<any[]>([])

const loadPs = async () => {
  if (!serviceOnline.value) return
  loadingPs.value = true
  try {
    const res = await invoke<{ models: any[] }>('ollama_ps', { host: hostInput.value || undefined })
    runningModels.value = res.models
  } catch (e) {
    ElMessage.error('加载运行状态失败: ' + String(e))
  } finally {
    loadingPs.value = false
  }
}

const stopModel = async (row: any) => {
  try {
    await invoke('ollama_stop', { host: hostInput.value || undefined, name: row.name })
    ElMessage.success('模型已停止')
    loadPs()
  } catch (e) {
    ElMessage.error('停止失败: ' + String(e))
  }
}

// ===== 对话测试 =====
const chatModel = ref('')
const chatInput = ref('')
const chatting = ref(false)
const chatMessages = ref<{ role: string; content: string }[]>([])
const streamingContent = ref('')
const chatMessagesRef = ref<HTMLElement>()

let chatChunkUnlisten: UnlistenFn | null = null
let chatDoneUnlisten: UnlistenFn | null = null

const scrollToBottom = () => {
  nextTick(() => {
    if (chatMessagesRef.value) {
      chatMessagesRef.value.scrollTop = chatMessagesRef.value.scrollHeight
    }
  })
}

const sendChat = async () => {
  if (!chatModel.value) {
    ElMessage.warning('请先选择模型')
    return
  }
  if (!chatInput.value.trim()) return

  chatMessages.value.push({ role: 'user', content: chatInput.value })
  chatInput.value = ''
  chatting.value = true
  streamingContent.value = ''
  scrollToBottom()

  // chatMessages 已包含最新 user 消息，直接传给后端
  const messagesForApi = chatMessages.value

  chatChunkUnlisten = await listen('ollama-chat-chunk', (event) => {
    const payload = event.payload as any
    streamingContent.value += payload.content
    scrollToBottom()
  })

  chatDoneUnlisten = await listen('ollama-chat-done', () => {
    chatMessages.value.push({ role: 'assistant', content: streamingContent.value })
    streamingContent.value = ''
    chatting.value = false
    cleanupChatListeners()
    scrollToBottom()
  })

  try {
    await invoke('ollama_chat', {
      host: hostInput.value || undefined,
      model: chatModel.value,
      messages: messagesForApi,
    })
  } catch (e) {
    chatting.value = false
    streamingContent.value = ''
    ElMessage.error('对话失败: ' + String(e))
    cleanupChatListeners()
  }
}

const cleanupChatListeners = () => {
  if (chatChunkUnlisten) { chatChunkUnlisten(); chatChunkUnlisten = null }
  if (chatDoneUnlisten) { chatDoneUnlisten(); chatDoneUnlisten = null }
}

const clearChat = () => {
  chatMessages.value = []
  streamingContent.value = ''
}

// ===== 生命周期 =====
onMounted(() => {
  checkService()
})

onBeforeUnmount(() => {
  cleanupPullListeners()
  cleanupChatListeners()
})
</script>

<style scoped>
.ollama-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .ollama-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.ollama-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.ollama-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.ollama-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.ollama-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.ollama-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* 服务状态徽章 */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
}

.status-badge.online {
  background: rgba(16, 185, 129, 0.15);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-badge.offline {
  background: rgba(239, 68, 68, 0.15);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.status-badge.unknown {
  background: rgba(148, 163, 184, 0.15);
  color: var(--text-secondary);
  border: 1px solid rgba(148, 163, 184, 0.3);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.status-badge.online .status-dot {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.version {
  font-size: 11px;
  opacity: 0.7;
}

.install-tips {
  margin: 8px 0 0;
  padding-left: 20px;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.8;
}

.install-tips code {
  background: var(--bg-input);
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 12px;
}

/* 拉取进度 */
.pull-progress {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--bg-input);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.pull-status {
  font-size: 13px;
  color: var(--text-primary);
  margin-bottom: 8px;
  font-weight: 500;
}

/* 对话区域 */
.chat-container {
  display: flex;
  flex-direction: column;
  height: 500px;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  background: var(--bg-input);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  margin-bottom: 12px;
}

.chat-message {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.chat-message.user {
  flex-direction: row-reverse;
}

.msg-role {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
}

.chat-message.user .msg-role {
  background: var(--accent-blue);
}

.chat-message.assistant .msg-role {
  background: var(--accent-cyan);
}

.msg-content {
  max-width: 80%;
  padding: 10px 14px;
  background: var(--bg-card);
  border-radius: 10px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  border: 1px solid var(--border-color);
}

.chat-message.user .msg-content {
  background: rgba(59, 130, 246, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
}

.msg-content.streaming {
  color: var(--text-secondary);
}

.cursor {
  animation: blink 1s infinite;
  color: var(--accent-cyan);
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.chat-input-area {
  display: flex;
  gap: 10px;
  align-items: flex-end;
}

.chat-input-area :deep(.el-textarea) {
  flex: 1;
}

.detail-pre {
  background: var(--bg-input);
  padding: 12px;
  border-radius: 6px;
  max-height: 400px;
  overflow: auto;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
}
</style>
