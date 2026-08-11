<template>
  <div class="tool-container">
    <!-- 服务控制 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">本地静态服务器</span>
          <el-tag :type="running ? 'success' : 'info'" size="small" effect="dark" class="status-tag">
            {{ running ? `运行中 · 端口 ${info.port}` : '已停止' }}
          </el-tag>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 选择目录启动 HTTP 服务，浏览器 / 局域网设备可访问</p>
                <p>• 目录页面支持文件下载、网页上传、ZIP 打包</p>
                <p>• 端口被占用时自动 +1 避让</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group dir-group">
            <div class="group-label">服务目录</div>
            <el-input v-model="rootDir" size="small" placeholder="选择要共享的目录" readonly class="dir-input">
              <template #append>
                <el-button :icon="FolderOpened" :disabled="running" @click="pickDir" />
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">端口</div>
            <el-input-number
              v-model="port"
              :min="1"
              :max="65535"
              size="small"
              :disabled="running"
              :controls="false"
              class="port-input"
            />
            <span class="port-hint">被占用自动 +1</span>
          </div>
          <div class="action-group">
            <div class="group-label">操作</div>
            <el-button v-if="!running" type="primary" size="small" :loading="starting" @click="startServer">
              启动服务
            </el-button>
            <el-button v-else type="danger" size="small" @click="stopServer">停止服务</el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 访问地址 -->
    <div v-if="running" class="tool-card">
      <div class="card-header">
        <span class="card-title">访问地址</span>
      </div>
      <div class="card-body">
        <div class="url-list">
          <div v-for="url in info.urls" :key="url" class="url-row">
            <code class="url-code">{{ url }}</code>
            <div class="url-actions">
              <el-button size="small" type="primary" plain @click="openBrowser(url)">打开浏览器</el-button>
              <el-button size="small" @click="copyUrl(url)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 访问日志 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">访问日志</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!logs.length" @click="clearLogs">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <DataTable v-if="logs.length" :data="logs" :max-height="360">
          <el-table-column prop="time" label="时间" width="165" />
          <el-table-column prop="ip" label="IP" width="130" />
          <el-table-column prop="method" label="方法" width="70" />
          <el-table-column prop="path" label="路径" min-width="200" show-overflow-tooltip />
          <el-table-column label="状态" width="80" align="center">
            <template #default="{ row }">
              <el-tag :type="row.status < 400 ? 'success' : 'danger'" size="small">{{ row.status }}</el-tag>
            </template>
          </el-table-column>
        </DataTable>
        <div v-else class="empty-tip">启动服务后，浏览器访问产生的请求会实时显示在这里</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onActivated, onDeactivated } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, FolderOpened } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import DataTable from '@/components/DataTable.vue'
import { useToolboxStore } from '@/store'

// ============ 类型（与 Rust snake_case 字段一致） ============
interface ServerInfo {
  running: boolean
  root_dir: string
  port: number
  urls: string[]
}
interface LogEntry {
  time: string
  ip: string
  method: string
  path: string
  status: number
}

const store = useToolboxStore()

// ============ 状态 ============
const rootDir = ref('')
const port = ref(8000)
const running = ref(false)
const starting = ref(false)
const info = ref<ServerInfo>({ running: false, root_dir: '', port: 0, urls: [] })
const logs = ref<LogEntry[]>([])
let pollTimer: ReturnType<typeof setInterval> | null = null

// ============ 选择目录 ============
const pickDir = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({ directory: true, multiple: false })
  if (typeof selected === 'string' && selected) {
    rootDir.value = selected
  }
}

// ============ 历史记录 ============
const addHistory = (action: string, output: string) => {
  store.addHistory({
    tool: 'staticServer',
    action,
    inputPreview: rootDir.value.slice(0, 50),
    outputPreview: output.slice(0, 50),
    inputFull: rootDir.value,
    outputFull: output,
  })
}

// ============ 启动 / 停止 ============
const startServer = async () => {
  if (!rootDir.value) {
    ElMessage.warning('请先选择服务目录')
    return
  }
  starting.value = true
  try {
    const res = await invoke<ServerInfo>('http_server_start', { rootDir: rootDir.value, port: port.value })
    info.value = res
    running.value = true
    addHistory('启动服务', res.urls[0] || '')
    ElMessage.success(`服务已启动，端口 ${res.port}`)
    await refreshLogs()
  } catch (e: any) {
    ElMessage.error('启动失败: ' + (e.message || e))
  } finally {
    starting.value = false
  }
}

const stopServer = async () => {
  try {
    await invoke('http_server_stop')
    running.value = false
    logs.value = []
    addHistory('停止服务', '')
    ElMessage.success('服务已停止')
  } catch (e: any) {
    ElMessage.error('停止失败: ' + (e.message || e))
  }
}

// ============ 状态与日志 ============
const refreshStatus = async () => {
  try {
    const s = await invoke<ServerInfo>('http_server_status')
    info.value = s
    running.value = s.running
    if (!s.running) {
      rootDir.value = s.root_dir || rootDir.value
    }
  } catch {
    /* 忽略，下次轮询重试 */
  }
}

const refreshLogs = async () => {
  if (!running.value) return
  try {
    logs.value = await invoke<LogEntry[]>('http_server_logs')
  } catch {
    /* 忽略 */
  }
}

const openBrowser = async (url: string) => {
  await invoke('http_open_url', { url })
}

const copyUrl = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url)
    ElMessage.success('已复制: ' + url)
  } catch {
    ElMessage.error('复制失败')
  }
}

const clearLogs = async () => {
  await invoke('http_server_clear_logs')
  logs.value = []
}

// KeepAlive 缓存：每次激活恢复状态并轮询日志，离开页面停止轮询
onActivated(() => {
  refreshStatus()
  if (pollTimer) clearInterval(pollTimer)
  pollTimer = setInterval(() => {
    refreshStatus()
    refreshLogs()
  }, 1200)
})

onDeactivated(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
})
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.status-tag {
  margin-left: 4px;
}

/* 控制区 */
.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
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
}
.dir-group {
  flex: 2 1 320px;
}
.dir-input {
  width: 100%;
}
.port-input {
  width: 110px;
}
.port-hint {
  font-size: 11px;
  color: var(--text-muted);
}

/* 访问地址 */
.url-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.url-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding: 10px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}
.url-code {
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 13px;
  color: var(--accent-cyan);
  word-break: break-all;
}
.url-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

/* 日志空态 */
.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 20px 0;
  font-size: 12px;
}

/* 提示 */
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
.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}
.tooltip-content p {
  margin: 2px 0;
}
</style>
