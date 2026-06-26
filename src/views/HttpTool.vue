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
        <div class="action-bar">
          <div class="action-bar-left">
            <el-select v-model="currentEnvId" size="small" style="width: 160px" placeholder="环境变量" clearable @change="onEnvChange">
              <el-option
                v-for="env in environments"
                :key="env.id"
                :label="env.name"
                :value="env.id"
              >
                <span>{{ env.name }}</span>
                <span class="env-var-count">{{ Object.keys(JSON.parse(env.variables_json || '{}')).length }}个变量</span>
              </el-option>
            </el-select>
            <el-button size="small" @click="showEnvDialog = true">管理</el-button>
            <el-button size="small" @click="handleSaveBookmark" :disabled="!url.trim()">收藏</el-button>
          </div>
          <el-button size="small" type="primary" @click="handleSend" :loading="loading">发送</el-button>
        </div>
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
          >
            <template v-if="currentEnv?.base_url" #prepend>
              <span class="url-base-prefix">{{ currentEnv.base_url }}</span>
            </template>
          </el-input>
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
          <VariablePicker @select="handleInsertVariable" />
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

    <!-- 历史/收藏/环境管理 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">历史与收藏</span>
        <el-button size="small" @click="refreshHistory" :disabled="loading">刷新</el-button>
      </div>
      <div class="card-body">
        <el-tabs v-model="historyTab" size="small">
          <el-tab-pane label="请求历史" name="history">
            <div v-if="httpHistory.length === 0" class="empty-hint">暂无请求历史</div>
            <div v-else class="history-list">
              <div v-for="(item, idx) in httpHistory" :key="idx" class="history-item" @click="restoreFromHistory(item)">
                <div class="history-method" :class="'method-' + item.method.toLowerCase()">{{ item.method }}</div>
                <div class="history-info">
                  <div class="history-url">{{ item.url }}</div>
                  <div class="history-meta">
                    <span>{{ item.created_at }}</span>
                    <span v-if="item.status">· {{ item.status }}</span>
                  </div>
                </div>
              </div>
            </div>
          </el-tab-pane>
          <el-tab-pane label="收藏" name="bookmarks">
            <div v-if="bookmarks.length === 0" class="empty-hint">暂无收藏，点击「收藏」按钮保存当前请求</div>
            <div v-else class="bookmark-list">
              <div v-for="(bm, idx) in bookmarks" :key="idx" class="history-item" @click="restoreFromBookmark(bm)">
                <div class="bookmark-left">
                  <div class="history-method" :class="'method-' + bm.method.toLowerCase()">{{ bm.method }}</div>
                  <div class="history-info">
                    <div class="history-url">{{ bm.name }}</div>
                    <div class="history-meta">{{ bm.url }}</div>
                  </div>
                </div>
                <el-button size="small" type="danger" :icon="Delete" circle @click.stop="deleteBookmark(bm.id)" />
              </div>
            </div>
          </el-tab-pane>
          <el-tab-pane label="环境变量" name="envs">
            <div class="env-vars-section">
              <div class="env-vars-header">
                <span v-if="currentEnv">当前环境：{{ currentEnv.name }}</span>
                <span v-else class="text-muted">未选择环境</span>
                <el-button size="small" @click="showEnvDialog = true">管理环境</el-button>
              </div>
              <div v-if="currentEnv" class="env-vars-list">
                <div v-for="(val, key) in parsedEnvVars" :key="key" class="env-var-row">
                  <code class="env-var-key">\x7b\x7b{{ key }}\x7d\x7d</code>
                  <span class="env-var-val">{{ val }}</span>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>

    <!-- 环境管理对话框 -->
    <el-dialog v-model="showEnvDialog" title="管理环境变量" width="600px" destroy-on-close>
      <div class="env-dialog-body">
        <div class="env-list-panel">
          <div class="env-list-header">
            <span>环境列表</span>
            <el-button size="small" @click="addEnvironment">+ 新建</el-button>
          </div>
          <div v-for="env in environments" :key="env.id"
            :class="['env-list-item', { active: editingEnvId === env.id }]"
            @click="selectEnvForEdit(env.id)">
            <span>{{ env.name }}</span>
            <el-button size="small" type="danger" :icon="Delete" circle @click.stop="deleteEnvironment(env.id)" />
          </div>
        </div>
        <div v-if="editingEnv" class="env-edit-panel">
          <el-form label-width="80px" size="small">
            <el-form-item label="环境名称">
              <el-input v-model="editingEnv.name" placeholder="如: dev / staging / prod" />
            </el-form-item>
            <el-form-item label="接口地址">
              <el-input v-model="editingEnv.base_url" placeholder="如: https://api.dev.example.com" />
              <div class="form-hint">选中此环境后，URL 输入框会显示此前缀，只需输入路径部分</div>
            </el-form-item>
            <el-form-item label="变量列表">
              <div class="env-edit-vars">
                <div v-for="(item, idx) in editingEnvVars" :key="idx" class="env-var-edit-row">
                  <el-input v-model="item.key" placeholder="变量名" style="width: 140px" />
                  <el-input v-model="item.value" placeholder="值" style="flex: 1; margin: 0 6px" />
                  <el-button size="small" type="danger" :icon="Delete" circle @click="editingEnvVars.splice(idx, 1)" />
                </div>
                <el-button size="small" @click="editingEnvVars.push({ key: '', value: '' })">+ 添加变量</el-button>
              </div>
            </el-form-item>
          </el-form>
        </div>
      </div>
      <template #footer>
        <el-button @click="showEnvDialog = false">取消</el-button>
        <el-button type="primary" @click="saveEnvironment">保存</el-button>
      </template>
    </el-dialog>

    <!-- 收藏命名对话框 -->
    <el-dialog v-model="showBookmarkDialog" title="收藏请求" width="400px" destroy-on-close>
      <el-form label-width="70px" size="small">
        <el-form-item label="名称">
          <el-input v-model="bookmarkName" placeholder="为此请求起个名字" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showBookmarkDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmSaveBookmark">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Delete, Loading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import * as db from '@/utils/dbClient'

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
const historyTab = ref('history')
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
  try {
    return JSON.stringify(JSON.parse(responseBody.value), null, 2)
  } catch {
    return responseBody.value
  }
})

// ============ 请求头操作 ============
const addHeader = () => {
  headers.value.push({ key: '', value: '' })
}

const removeHeader = (idx: number) => {
  headers.value.splice(idx, 1)
}

const handleClearBody = () => { body.value = '' }

const handlePasteBody = async () => {
  try {
    body.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  body.value = value
}

// ============ 环境变量 ============
const environments = ref<db.HttpEnvironment[]>([])
const currentEnvId = ref('')
const showEnvDialog = ref(false)
const editingEnvId = ref('')
const editingEnv = computed(() => environments.value.find(e => e.id === editingEnvId.value))
const editingEnvVars = ref<Array<{ key: string; value: string }>>([])
const currentEnv = computed(() => environments.value.find(e => e.id === currentEnvId.value))
const parsedEnvVars = computed(() => {
  if (!currentEnv.value) return {}
  try {
    return JSON.parse(currentEnv.value.variables_json || '{}') as Record<string, string>
  } catch { return {} }
})

const loadEnvironments = async () => {
  try {
    environments.value = await db.listHttpEnvironments()
  } catch { environments.value = [] }
}

const addEnvironment = () => {
  const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 5)
  editingEnvId.value = id
  environments.value.unshift({
    id, name: '', base_url: '', variables_json: '{}',
    created_at: new Date().toISOString(), updated_at: new Date().toISOString()
  })
  editingEnvVars.value = []
}

const selectEnvForEdit = (id: string) => {
  editingEnvId.value = id
  if (editingEnv.value) {
    try {
      const vars = JSON.parse(editingEnv.value.variables_json || '{}')
      editingEnvVars.value = Object.entries(vars).map(([key, value]) => ({ key, value: value as string }))
    } catch {
      editingEnvVars.value = []
    }
  }
}

const saveEnvironment = async () => {
  const env = editingEnv.value
  if (!env || !env.name.trim()) {
    ElMessage.warning('环境名称不能为空')
    return
  }
  const varsObj: Record<string, string> = {}
  for (const item of editingEnvVars.value) {
    if (item.key.trim()) varsObj[item.key.trim()] = item.value
  }
  env.variables_json = JSON.stringify(varsObj)
  env.updated_at = new Date().toISOString()
  try {
    await db.saveHttpEnvironment({ ...env })
    ElMessage.success('环境已保存')
    showEnvDialog.value = false
  } catch {
    ElMessage.error('保存失败')
  }
}

const deleteEnvironment = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定删除此环境？', '确认', { type: 'warning' })
    await db.deleteHttpEnvironment(id)
    environments.value = environments.value.filter(e => e.id !== id)
    if (currentEnvId.value === id) currentEnvId.value = ''
    ElMessage.success('已删除')
  } catch { /* 取消 */ }
}

const resolveEnvVars = (text: string): string => {
  const vars = parsedEnvVars.value
  return text.replace(/\{\{(.+?)\}\}/g, (_, key) => vars[key.trim()] || `{{${key}}}`)
}

const onEnvChange = () => {
  ElMessage.info(`已切换至：${currentEnv.value?.name || '无环境'}`)
}

// ============ 请求历史 ============
const httpHistory = ref<db.HttpHistoryItem[]>([])

const loadHistory = async () => {
  try {
    httpHistory.value = await db.listHttpHistory(50)
  } catch { httpHistory.value = [] }
}

const refreshHistory = () => {
  if (!loading.value) loadHistory()
}

const restoreFromHistory = (item: db.HttpHistoryItem) => {
  method.value = item.method
  url.value = item.url
  bodyType.value = item.body_type
  body.value = item.body || ''
  try {
    const hdrs = JSON.parse(item.headers_json || '{}')
    headers.value = Object.entries(hdrs).map(([key, value]) => ({ key, value: value as string }))
  } catch {
    headers.value = []
  }
  ElMessage.success('已恢复历史请求')
}

// ============ 收藏 ============
const bookmarks = ref<db.HttpBookmark[]>([])
const showBookmarkDialog = ref(false)
const bookmarkName = ref('')

const loadBookmarks = async () => {
  try {
    bookmarks.value = await db.listHttpBookmarks()
  } catch { bookmarks.value = [] }
}

const handleSaveBookmark = () => {
  if (!url.value.trim()) {
    ElMessage.warning('请输入 URL')
    return
  }
  bookmarkName.value = `${method.value} ${url.value.slice(0, 40)}`
  showBookmarkDialog.value = true
}

const confirmSaveBookmark = async () => {
  if (!bookmarkName.value.trim()) {
    ElMessage.warning('请输入收藏名称')
    return
  }
  const headerMap: Record<string, string> = {}
  for (const h of headers.value) {
    if (h.key.trim()) headerMap[h.key.trim()] = h.value
  }
  const now = new Date().toISOString()
  const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 5)
  try {
    await db.saveHttpBookmark({
      id, name: bookmarkName.value.trim(), method: method.value, url: url.value,
      headers_json: JSON.stringify(headerMap), body: body.value || null, body_type: bodyType.value,
      created_at: now, updated_at: now,
    })
    showBookmarkDialog.value = false
    loadBookmarks()
    ElMessage.success('已收藏')
  } catch { ElMessage.error('收藏失败') }
}

const restoreFromBookmark = (bm: db.HttpBookmark) => {
  method.value = bm.method
  url.value = bm.url
  bodyType.value = bm.body_type
  body.value = bm.body || ''
  try {
    const hdrs = JSON.parse(bm.headers_json || '{}')
    headers.value = Object.entries(hdrs).map(([key, value]) => ({ key, value: value as string }))
  } catch {
    headers.value = []
  }
  ElMessage.success(`已加载：${bm.name}`)
}

const deleteBookmark = async (id: string) => {
  try {
    await db.deleteHttpBookmark(id)
    bookmarks.value = bookmarks.value.filter(b => b.id !== id)
    ElMessage.success('已删除')
  } catch { ElMessage.error('删除失败') }
}

// ============ 发送请求 ============
const handleSend = async () => {
  if (!url.value.trim()) {
    ElMessage.warning('请输入请求 URL')
    return
  }

  // 替换环境变量
  let finalUrl = url.value.trim()
  if (currentEnv.value?.base_url && !finalUrl.startsWith('http://') && !finalUrl.startsWith('https://')) {
    finalUrl = currentEnv.value.base_url.replace(/\/+$/, '') + '/' + finalUrl.replace(/^\/+/, '')
  }
  const resolvedUrl = resolveEnvVars(finalUrl)
  const resolvedHeaders: Record<string, string> = {}
  for (const h of headers.value) {
    if (h.key.trim()) resolvedHeaders[h.key.trim()] = resolveEnvVars(h.value)
  }
  const resolvedBody = body.value ? resolveEnvVars(body.value) : null

  loading.value = true
  errorMsg.value = ''
  responseStatus.value = 0

  try {
    const result = await invoke('send_http_request', {
      request: {
        method: method.value,
        url: resolvedUrl,
        headers: resolvedHeaders,
        body: resolvedBody,
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

    // 保存历史到 SQLite
    const headerMap: Record<string, string> = {}
    for (const h of headers.value) {
      if (h.key.trim()) headerMap[h.key.trim()] = h.value
    }
    const now = new Date().toISOString()
    try {
      await db.addHttpHistory({
        method: method.value,
        url: url.value.trim(),
        headers_json: JSON.stringify(headerMap),
        body: body.value || null,
        body_type: bodyType.value,
        env_name: currentEnv.value?.name || null,
        status: result.status,
        created_at: now,
      })
      // 刷新历史列表
      loadHistory()
    } catch { /* 历史保存失败不影响主流程 */ }
  } catch (e: any) {
    errorMsg.value = e.message || '请求失败'
    ElMessage.error('请求失败')
  } finally {
    loading.value = false
  }
}

// ============ 工具方法 ============
const handleCopyResponse = async () => {
  if (!responseBody.value) { ElMessage.warning('没有可复制的内容'); return }
  try {
    await navigator.clipboard.writeText(responseBody.value)
    ElMessage.success('已复制到剪贴板')
  } catch { ElMessage.error('复制失败') }
}

const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(() => {
  loadEnvironments()
  loadHistory()
  loadBookmarks()
})
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

/* ===== 动作栏 ===== */
.action-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex: 1;
  margin-left: 12px;
}

.action-bar-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.env-var-count {
  font-size: 11px;
  color: var(--text-muted);
  margin-left: 6px;
}

/* ===== 历史/收藏列表 ===== */
.history-list, .bookmark-list {
  max-height: 280px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.15s;
  gap: 8px;
}

.history-item:hover {
  background: var(--bg-input);
}

.bookmark-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.history-method {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  flex-shrink: 0;
  min-width: 44px;
  text-align: center;
}

.method-get { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
.method-post { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.method-put { background: rgba(245, 158, 11, 0.15); color: #f59e0b; }
.method-delete { background: rgba(239, 68, 68, 0.15); color: #ef4444; }
.method-patch { background: rgba(168, 85, 247, 0.15); color: #a855f7; }
.method-head { background: rgba(99, 102, 241, 0.15); color: #6366f1; }
.method-options { background: rgba(236, 72, 153, 0.15); color: #ec4899; }

.history-info {
  flex: 1;
  min-width: 0;
}

.history-url {
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-meta {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* ===== 环境变量区域 ===== */
.env-vars-section {
  font-size: 13px;
}

.env-vars-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.env-vars-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.env-var-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-input);
  padding: 4px 10px;
  border-radius: 4px;
}

.env-var-key {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  color: var(--accent-cyan);
}

.env-var-val {
  color: var(--text-secondary);
  font-size: 12px;
}

.text-muted {
  color: var(--text-muted);
}

/* ===== 环境管理对话框 ===== */
.env-dialog-body {
  display: flex;
  gap: 16px;
  min-height: 300px;
}

.env-list-panel {
  width: 180px;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
  padding-right: 12px;
}

.env-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

.env-list-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  margin-bottom: 4px;
}

.env-list-item:hover {
  background: var(--bg-input);
}

.env-list-item.active {
  background: rgba(0, 212, 255, 0.1);
  color: var(--accent-cyan);
}

.env-edit-panel {
  flex: 1;
  min-width: 0;
}

.env-edit-vars {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.env-var-edit-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.url-base-prefix {
  font-size: 12px;
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  opacity: 0.8;
}

.form-hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
  line-height: 1.4;
}
</style>
