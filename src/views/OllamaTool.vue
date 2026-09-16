<template>
  <div class="tool-container" :class="{ 'chat-mode': activeTab === 'chat' }">
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
    <div v-if="activeTab === 'chat'" class="tool-card chat-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">对话测试</span>
          <el-tag v-if="tools.length > 0" size="small" type="success" effect="plain" style="margin-left: 8px">
            {{ tools.length }} 个工具
          </el-tag>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="toolDialogVisible = true">工具管理</el-button>
          <el-select v-model="chatModel" size="small" placeholder="选择模型" style="width: 220px" :disabled="chatting">
            <el-option
              v-for="m in models"
              :key="m.name"
              :label="m.name"
              :value="m.name"
            />
          </el-select>
          <el-button size="small" @click="loadModels" :disabled="chatting">刷新</el-button>
          <el-button size="small" @click="sessionDialogVisible = true" :disabled="chatting">
            会话{{ sessions.length > 1 ? ` (${sessions.length})` : '' }}
          </el-button>
          <el-button size="small" type="danger" plain @click="clearChat" :disabled="chatting">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="chat-container">
          <div class="chat-messages" ref="chatMessagesRef">
            <template v-for="(msg, i) in chatMessages" :key="i">
              <!-- 工具调用消息 -->
              <div v-if="msg.role === 'tool'" class="chat-message tool">
                <div class="msg-role tool">
                  <el-icon><Tools /></el-icon>
                </div>
                <div class="msg-content tool-msg">
                  <div class="tool-name">🔧 {{ msg.toolName }}</div>
                  <div class="tool-section">
                    <span class="tool-label">参数：</span>
                    <code>{{ JSON.stringify(msg.arguments) }}</code>
                  </div>
                  <div class="tool-section">
                    <span class="tool-label">结果：</span>
                    <pre class="tool-result">{{ msg.result }}</pre>
                  </div>
                </div>
              </div>
              <!-- 普通消息 -->
              <div v-else class="chat-message" :class="msg.role">
                <div class="msg-role">{{ msg.role === 'user' ? '我' : 'AI' }}</div>
                <div class="msg-content">{{ msg.content }}</div>
              </div>
            </template>
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

    <!-- 工具管理弹窗 -->
    <el-dialog v-model="toolDialogVisible" title="Python 工具管理" width="720px" top="8vh">
      <div class="tool-mgr">
        <!-- 工具列表 -->
        <div class="tool-list">
          <div
            v-for="(t, i) in tools"
            :key="i"
            class="tool-item"
            :class="{ active: editingIndex === i }"
            @click="selectTool(i)"
          >
            <div class="tool-item-name">{{ t.name }}</div>
            <div class="tool-item-desc">{{ t.description }}</div>
          </div>
          <div v-if="tools.length === 0" class="empty-tip">暂无工具，点击"加载示例"快速体验，或"新增工具"自定义</div>
        </div>

        <!-- 工具编辑区 -->
        <div class="tool-editor">
          <div class="editor-actions">
            <el-button size="small" type="primary" @click="addTool">新增工具</el-button>
            <el-button size="small" @click="loadExample">加载示例</el-button>
            <el-button
              v-if="editingIndex !== null"
              size="small"
              type="danger"
              @click="deleteTool"
            >删除</el-button>
            <el-button
              v-if="editingIndex !== null"
              size="small"
              @click="testTool"
              :loading="testingTool"
            >测试运行</el-button>
          </div>

          <div v-if="editingIndex !== null" class="editor-form">
            <el-form label-width="80px" size="small">
              <el-form-item label="工具名">
                <el-input v-model="currentTool.name" placeholder="英文，如 calculate" />
              </el-form-item>
              <el-form-item label="描述">
                <el-input
                  v-model="currentTool.description"
                  type="textarea"
                  :rows="2"
                  placeholder="告诉模型这个工具的用途"
                />
              </el-form-item>
              <el-form-item label="参数 Schema">
                <el-input
                  v-model="currentTool.parameters"
                  type="textarea"
                  :rows="4"
                  placeholder='JSON Schema，如 {"type":"object","properties":{"a":{"type":"number"}},"required":["a"]}'
                />
              </el-form-item>
              <el-form-item label="Python 代码">
                <el-input
                  v-model="currentTool.code"
                  type="textarea"
                  :rows="8"
                  placeholder="参数可直接用变量名（如 a）或 params['a']。用 print() 输出结果。"
                />
              </el-form-item>
              <el-form-item>
                <el-alert
                  type="info"
                  :closable="false"
                  title="参数通过 stdin（JSON）传入，已注入全局作用域。代码中直接使用参数名即可，结果通过 print() 返回给模型。"
                />
              </el-form-item>
            </el-form>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 历史会话弹窗 -->
    <el-dialog v-model="sessionDialogVisible" title="历史会话" width="560px">
      <div class="session-actions">
        <el-button size="small" type="primary" @click="newSession()">新建会话</el-button>
        <el-button
          size="small"
          type="danger"
          plain
          :disabled="sessions.length === 0"
          @click="clearAllSessions"
        >清空全部</el-button>
      </div>
      <div class="session-list">
        <div
          v-for="s in sortedSessions"
          :key="s.id"
          class="session-item"
          :class="{ active: s.id === currentSessionId }"
          @click="switchSession(s.id)"
        >
          <div class="session-main">
            <div class="session-title">{{ sessionTitle(s) }}</div>
            <div class="session-meta">
              {{ formatTime(s.updatedAt) }} · {{ s.messages.length }} 条消息
            </div>
          </div>
          <el-button link type="danger" @click.stop="deleteSession(s.id)">删除</el-button>
        </div>
        <div v-if="sessions.length === 0" class="empty-tip">暂无历史会话</div>
      </div>
    </el-dialog>

    <!-- 模型详情弹窗 -->
    <el-dialog v-model="detailVisible" title="模型详情" width="640px">
      <pre class="detail-pre">{{ modelDetail }}</pre>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Tools } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const activeTab = ref('models')

// ===== 工具管理 =====
interface ToolDef {
  name: string
  description: string
  parameters: string // JSON Schema 字符串
  code: string
}

interface ChatMsg {
  role: string
  content: string
  toolName?: string
  arguments?: any
  result?: string
}

const TOOLS_STORAGE_KEY = 'ollama_tools'

const toolDialogVisible = ref(false)
const tools = ref<ToolDef[]>([])
const editingIndex = ref<number | null>(null)
const currentTool = ref<ToolDef>({ name: '', description: '', parameters: '', code: '' })
const testingTool = ref(false)

const loadTools = () => {
  try {
    const saved = localStorage.getItem(TOOLS_STORAGE_KEY)
    if (saved) {
      tools.value = JSON.parse(saved)
    }
  } catch (e) {
    console.error('加载工具失败', e)
  }
}

const saveTools = () => {
  localStorage.setItem(TOOLS_STORAGE_KEY, JSON.stringify(tools.value))
}

const addTool = () => {
  tools.value.push({
    name: '',
    description: '',
    parameters: '{"type":"object","properties":{}}',
    code: '',
  })
  editingIndex.value = tools.value.length - 1
  currentTool.value = tools.value[editingIndex.value]
}

// 示例工具：可直接「测试运行」看效果（测试参数按类型生成，text 为 'test'）
const EXAMPLE_TOOL: ToolDef = {
  name: 'word_count',
  description: '统计一段文本的字符数、词数、行数',
  parameters: '{"type":"object","properties":{"text":{"type":"string","description":"待统计的文本"}},"required":["text"]}',
  code: 'lines = text.splitlines()\nwords = text.split()\nprint(f"字符数: {len(text)}")\nprint(f"词数: {len(words)}")\nprint(f"行数: {len(lines)}")',
}

const loadExample = () => {
  const i = tools.value.findIndex(t => t.name === EXAMPLE_TOOL.name)
  if (i >= 0) {
    selectTool(i)
  } else {
    tools.value.push({ ...EXAMPLE_TOOL })
    selectTool(tools.value.length - 1)
  }
  saveTools()
  ElMessage.success('已加载示例工具，点击「测试运行」查看效果')
}

const selectTool = (i: number) => {
  editingIndex.value = i
  currentTool.value = tools.value[i]
}

const deleteTool = async () => {
  if (editingIndex.value === null) return
  try {
    await ElMessageBox.confirm('确定删除此工具？', '确认', { type: 'warning' })
  } catch { return }
  tools.value.splice(editingIndex.value, 1)
  editingIndex.value = tools.value.length > 0 ? 0 : null
  if (editingIndex.value !== null) {
    currentTool.value = tools.value[editingIndex.value]
  }
  saveTools()
}

const testTool = async () => {
  if (editingIndex.value === null) return
  testingTool.value = true
  try {
    // 解析 parameters 获取必填字段，生成测试参数
    let params: Record<string, any> = {}
    try {
      const schema = JSON.parse(currentTool.value.parameters)
      if (schema.properties) {
        for (const [key, val] of Object.entries<any>(schema.properties)) {
          // 根据类型生成默认测试值
          if (val.type === 'string') params[key] = 'test'
          else if (val.type === 'number' || val.type === 'integer') params[key] = 1
          else if (val.type === 'boolean') params[key] = true
          else params[key] = 'test'
        }
      }
    } catch {
      // schema 解析失败，用空参数
    }
    const result = await invoke<string>('ollama_run_tool', {
      code: currentTool.value.code,
      arguments: params,
    })
    ElMessage.success('执行成功，输出:\n' + result)
  } catch (e) {
    ElMessage.error('执行失败: ' + String(e))
  } finally {
    testingTool.value = false
  }
}

// 关闭弹窗时保存
watch(toolDialogVisible, (val) => {
  if (!val) saveTools()
})

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
const streamingContent = ref('')
const chatMessagesRef = ref<HTMLElement>()

let chatChunkUnlisten: UnlistenFn | null = null
let chatDoneUnlisten: UnlistenFn | null = null
let chatToolUnlisten: UnlistenFn | null = null

// ===== 历史会话（多段对话，最多保留 20 段）=====
interface ChatSession {
  id: string
  title: string
  updatedAt: number
  messages: ChatMsg[]
}

const CHAT_STORAGE_KEY = 'ollama_chat_sessions'
const LEGACY_CHAT_KEY = 'ollama_chat_history'
const MAX_SESSIONS = 20
const MAX_MESSAGES_PER_SESSION = 200

const sessions = ref<ChatSession[]>([])
const currentSessionId = ref('')
const sessionDialogVisible = ref(false)

const currentSession = computed(() => sessions.value.find(s => s.id === currentSessionId.value))

// 用 computed 代理当前会话的消息，原有的 chatMessages.value.push / = [] 写法无需改动
const chatMessages = computed<ChatMsg[]>({
  get: () => currentSession.value?.messages ?? [],
  set: (v) => { if (currentSession.value) currentSession.value.messages = v },
})

const sortedSessions = computed(() => [...sessions.value].sort((a, b) => b.updatedAt - a.updatedAt))

// 标题取该会话第一条用户消息，用户无需手动命名
const sessionTitle = (s: ChatSession) =>
  s.title || s.messages.find(m => m.role === 'user')?.content.slice(0, 20) || '新会话'

const formatTime = (ts: number) => new Date(ts).toLocaleString('zh-CN', { hour12: false })

const genId = () => `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`

const saveChat = () => {
  try {
    const s = currentSession.value
    if (s) s.updatedAt = Date.now()
    // ponytail: 每段最多 200 条、总共最多 20 段；只读不写内存状态，避免 deep watch 自触发
    const data = sortedSessions.value.slice(0, MAX_SESSIONS).map(x => ({
      ...x,
      title: sessionTitle(x),
      messages: x.messages.slice(-MAX_MESSAGES_PER_SESSION),
    }))
    localStorage.setItem(CHAT_STORAGE_KEY, JSON.stringify(data))
  } catch (e) {
    console.error('保存对话记录失败', e)
  }
}

const loadChat = () => {
  try {
    const saved = localStorage.getItem(CHAT_STORAGE_KEY)
    if (saved) {
      const arr = JSON.parse(saved)
      if (Array.isArray(arr)) sessions.value = arr.filter(s => s && Array.isArray(s.messages))
    }
    // 兼容旧版单会话存储：迁移成一段会话后删除旧 key
    if (sessions.value.length === 0) {
      const legacy = localStorage.getItem(LEGACY_CHAT_KEY)
      if (legacy) {
        const msgs = JSON.parse(legacy)
        if (Array.isArray(msgs) && msgs.length > 0) {
          sessions.value = [{ id: genId(), title: '', updatedAt: Date.now(), messages: msgs }]
        }
        localStorage.removeItem(LEGACY_CHAT_KEY)
      }
    }
  } catch (e) {
    console.error('加载对话记录失败', e)
  }
  if (sessions.value.length === 0) newSession(true)
  else currentSessionId.value = sortedSessions.value[0].id
}

const newSession = (silent = false) => {
  const s: ChatSession = { id: genId(), title: '', updatedAt: Date.now(), messages: [] }
  sessions.value.unshift(s)
  if (sessions.value.length > MAX_SESSIONS) sessions.value = sortedSessions.value.slice(0, MAX_SESSIONS)
  currentSessionId.value = s.id
  saveChat()
  if (!silent) {
    sessionDialogVisible.value = false
    ElMessage.success('已新建会话')
  }
}

const switchSession = (id: string) => {
  // 流式输出期间切换会把结果写进错误的会话，直接拦掉
  if (chatting.value) {
    ElMessage.warning('对话进行中，请等待本轮结束再切换')
    return
  }
  sessionDialogVisible.value = false
  if (id === currentSessionId.value) return
  currentSessionId.value = id
  scrollToBottom()
}

const deleteSession = async (id: string) => {
  if (sessions.value.length <= 1) {
    ElMessage.warning('至少保留一段会话')
    return
  }
  try {
    await ElMessageBox.confirm('删除后该段对话不可恢复，确定删除？', '删除确认', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消',
    })
  } catch { return }
  sessions.value = sessions.value.filter(s => s.id !== id)
  if (currentSessionId.value === id) currentSessionId.value = sortedSessions.value[0]?.id ?? ''
  saveChat()
}

const clearAllSessions = async () => {
  try {
    await ElMessageBox.confirm('将删除全部历史会话，此操作不可恢复。', '清空确认', {
      type: 'warning',
      confirmButtonText: '清空',
      cancelButtonText: '取消',
    })
  } catch { return }
  sessions.value = []
  newSession(true)
  ElMessage.success('已清空全部会话')
}

// 消息数量少、变动不频繁，用 deep watch 统一兜住所有写入点（含清空）
watch(chatMessages, saveChat, { deep: true })

const scrollToBottom = () => {
  nextTick(() => {
    if (chatMessagesRef.value) {
      chatMessagesRef.value.scrollTop = chatMessagesRef.value.scrollHeight
    }
  })
}

// 切回对话 Tab 时滚到底部，避免恢复的历史记录停在顶部
watch(activeTab, (val) => {
  if (val === 'chat') scrollToBottom()
})

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

  // 构建传给后端的工具定义（parameters 从字符串解析为对象）
  const toolsForApi = tools.value
    .filter(t => t.name && t.code)
    .map(t => ({
      name: t.name,
      description: t.description,
      parameters: (() => { try { return JSON.parse(t.parameters) } catch { return { type: 'object', properties: {} } } })(),
      code: t.code,
    }))

  // chatMessages 已包含最新 user 消息，直接传给后端（去掉工具消息，只传 user/assistant）
  const messagesForApi = chatMessages.value
    .filter(m => m.role === 'user' || m.role === 'assistant')
    .map(m => ({ role: m.role, content: m.content }))

  chatChunkUnlisten = await listen('ollama-chat-chunk', (event) => {
    const payload = event.payload as any
    streamingContent.value += payload.content
    scrollToBottom()
  })

  chatToolUnlisten = await listen('ollama-tool-call', (event) => {
    const payload = event.payload as any
    chatMessages.value.push({
      role: 'tool',
      content: '',
      toolName: payload.name,
      arguments: payload.arguments,
      result: payload.result,
    })
    scrollToBottom()
  })

  chatDoneUnlisten = await listen('ollama-chat-done', () => {
    if (streamingContent.value) {
      chatMessages.value.push({ role: 'assistant', content: streamingContent.value })
    }
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
      tools: toolsForApi.length > 0 ? toolsForApi : undefined,
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
  if (chatToolUnlisten) { chatToolUnlisten(); chatToolUnlisten = null }
}

const clearChat = () => {
  chatMessages.value = []
  streamingContent.value = ''
}

// ===== 生命周期 =====
onMounted(() => {
  checkService()
  loadTools()
  loadChat()
  scrollToBottom()
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

/* ponytail: 对话 Tab 下让卡片撑满可视高度，避免下方大片留白；其他 Tab 保持原有滚动布局 */
.tool-container.chat-mode {
  display: flex;
  flex-direction: column;
}

.tool-container.chat-mode > .tool-card:not(.chat-card) {
  flex-shrink: 0;
}

.tool-container.chat-mode .chat-card {
  flex: 1 1 auto;
  min-height: 320px;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
}

.tool-container.chat-mode .chat-card .card-header {
  flex-shrink: 0;
}

.tool-container.chat-mode .chat-card .card-body {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.tool-container.chat-mode .chat-container {
  height: auto;
  flex: 1 1 auto;
  min-height: 0;
}

.chat-messages {
  flex: 1;
  min-height: 0;
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
  flex-shrink: 0;
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

/* 工具消息 */
.chat-message.tool .msg-role.tool {
  background: var(--accent-orange);
}

.tool-msg {
  max-width: 85%;
  background: rgba(249, 115, 22, 0.08);
  border: 1px solid rgba(249, 115, 22, 0.3);
}

.tool-name {
  font-weight: 600;
  color: var(--accent-orange);
  margin-bottom: 8px;
  font-size: 14px;
}

.tool-section {
  margin-bottom: 6px;
  font-size: 13px;
}

.tool-label {
  color: var(--text-secondary);
  font-weight: 500;
}

.tool-section code {
  background: var(--bg-input);
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 12px;
  word-break: break-all;
}

.tool-result {
  background: var(--bg-input);
  padding: 8px 10px;
  border-radius: 4px;
  margin-top: 4px;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow: auto;
  font-size: 12px;
  line-height: 1.5;
}

/* 工具管理弹窗 */
.tool-mgr {
  display: flex;
  gap: 16px;
  min-height: 480px;
}

.tool-list {
  width: 200px;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
  padding-right: 12px;
  overflow-y: auto;
}

.tool-item {
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 6px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.tool-item:hover {
  background: var(--bg-input);
}

.tool-item.active {
  background: rgba(0, 212, 255, 0.1);
  border-color: var(--accent-cyan);
}

.tool-item-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.tool-item-desc {
  font-size: 12px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

/* 历史会话弹窗 */
.session-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.session-list {
  max-height: 420px;
  overflow-y: auto;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 6px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.session-item:hover {
  background: var(--bg-input);
}

.session-item.active {
  background: rgba(0, 212, 255, 0.1);
  border-color: var(--accent-cyan);
}

.session-main {
  flex: 1;
  min-width: 0;
}

.session-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-meta {
  font-size: 12px;
  color: var(--text-secondary);
}

.empty-tip {
  text-align: center;
  color: var(--text-secondary);
  padding: 40px 8px;
  font-size: 13px;
}

.tool-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.editor-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.editor-form {
  flex: 1;
  overflow-y: auto;
}
</style>
