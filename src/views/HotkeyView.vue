<template>
  <div class="tool-container">
    <!-- 统计卡片 -->
    <div class="tool-card">
      <div class="card-header"><span class="card-title">快捷键占用概览</span></div>
      <div class="card-body">
        <div class="stats-grid">
          <div class="stat-item">总探测数 <strong>{{ stats?.total ?? 0 }}</strong></div>
          <div class="stat-item">被占用 <strong class="stat-danger">{{ stats?.occupied ?? 0 }}</strong></div>
          <div class="stat-item">可注册 <strong class="stat-success">{{ stats?.available ?? 0 }}</strong></div>
          <div class="stat-item">系统保留 <strong class="stat-warning">{{ stats?.reserved ?? 0 }}</strong></div>
        </div>
      </div>
    </div>

    <!-- 搜索栏 + 操作区 -->
    <div class="tool-card sticky-card">
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 2">
            <el-input v-model="searchKeyword" placeholder="搜索热键或进程名" clearable size="small" />
          </div>
          <div class="action-group">
            <el-radio-group v-model="filterStatus" size="small">
              <el-radio-button label="">全部</el-radio-button>
              <el-radio-button label="Occupied">被占用</el-radio-button>
              <el-radio-button label="Available">可注册</el-radio-button>
              <el-radio-button label="SystemReserved">系统保留</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <el-input v-model="extraKeysInput" placeholder="自定义热键" style="width: 200px" size="small" />
            <el-tooltip content="输入要探测的自定义热键，逗号分隔，如: Ctrl+Shift+S, Alt+F7" placement="top">
              <el-icon size="16"><HelpFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="action-group">
            <el-button type="primary" size="small" :loading="isProbing" @click="startProbe">开始探测</el-button>
            <el-button v-if="isProbing" size="small" @click="cancelProbe">取消</el-button>
            <el-button size="small" :disabled="!results.length" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
        <el-progress v-if="isProbing" :percentage="progressPercent" :format="formatProgress" :stroke-width="6" style="margin-top: 8px" />
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 结果表格 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">探测结果</span>
        <div class="card-actions">
          <span class="group-label">{{ filteredResults.length }} / {{ results.length }} 条</span>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="pagedResults" border size="small" max-height="600" style="width: 100%">
          <el-table-column prop="label" label="热键组合" width="140" sortable />
          <el-table-column label="状态" width="120" sortable :sort-method="sortByStatus">
            <template #default="{ row }">
              <el-tag :type="statusTagType(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="占用进程" width="180" sortable :sort-method="sortByProcess">
            <template #default="{ row }">
              {{ row.process_display || (row.process_name || '-') }}
            </template>
          </el-table-column>
          <el-table-column prop="process_pid" label="PID" width="90" sortable />
          <el-table-column label="进程路径" show-overflow-tooltip>
            <template #default="{ row }">
              <span :class="{ 'path-muted': !row.process_path }">{{ row.process_path || '—' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="来源" width="110">
            <template #default="{ row }">{{ sourceLabel(row.source) }}</template>
          </el-table-column>
        </el-table>
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredResults.length"
          layout="prev, pager, next, total"
          class="pagination-right"
          style="margin-top: 12px"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onDeactivated, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { HelpFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

// Rust struct 字段为 snake_case，前端 interface 必须一致（AGENTS 经验 16）
// 但 enum 序列化为 PascalCase（serde rename_all = "PascalCase"）
interface HotkeyResult {
  label: string
  mod_flags: number
  vk: number
  status: 'Available' | 'Occupied' | 'SystemReserved'
  process_name: string | null
  process_display: string | null
  process_pid: number | null
  process_path: string | null
  source: 'MapTable' | 'ProcessScan' | 'SelfRegistered' | 'None'
}

interface ProbeStats {
  total: number
  available: number
  occupied: number
  reserved: number
}

interface ProbeProgress {
  probe_id: string
  done: number
  total: number
  last_key: string
  is_finished: boolean
  error: string | null
}

interface ProbeCompletePayload {
  probe_id: string
  results: HotkeyResult[]
  stats: ProbeStats
  cancelled: boolean
}

const store = useToolboxStore()

const results = ref<HotkeyResult[]>([])
const stats = ref<ProbeStats | null>(null)
const isProbing = ref(false)
const error = ref('')
const progress = ref<ProbeProgress | null>(null)
const searchKeyword = ref('')
const filterStatus = ref('')
const extraKeysInput = ref('')
const currentPage = ref(1)
const pageSize = ref(50)

let unlistenProgress: UnlistenFn | null = null
let unlistenComplete: UnlistenFn | null = null
let pollTimer: number | null = null
let done = false
// ponytail: completed 守卫 handleComplete 入口，防止 event + 轮询竞态导致重复处理（AGENTS 经验 10）
let completed = false

const progressPercent = computed(() => {
  if (!progress.value || progress.value.total === 0) return 0
  return Math.round((progress.value.done / progress.value.total) * 100)
})

const filteredResults = computed(() => {
  return results.value.filter(r => {
    if (filterStatus.value && r.status !== filterStatus.value) return false
    if (searchKeyword.value) {
      const kw = searchKeyword.value.toLowerCase()
      return r.label.toLowerCase().includes(kw)
        || (r.process_display?.toLowerCase().includes(kw) ?? false)
        || (r.process_name?.toLowerCase().includes(kw) ?? false)
    }
    return true
  })
})

const pagedResults = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredResults.value.slice(start, start + pageSize.value)
})

function statusLabel(s: HotkeyResult['status']): string {
  return { Available: '可注册', Occupied: '被占用', SystemReserved: '系统保留' }[s]
}

function statusTagType(s: HotkeyResult['status']): 'success' | 'danger' | 'warning' | 'info' {
  return ({ Available: 'success', Occupied: 'danger', SystemReserved: 'warning' } as const)[s]
}

function sourceLabel(s: HotkeyResult['source']): string {
  return { MapTable: '映射表', ProcessScan: '进程扫描', SelfRegistered: '自身注册', None: '-' }[s]
}

function sortByStatus(a: HotkeyResult, b: HotkeyResult): number {
  return a.status.localeCompare(b.status)
}

function sortByProcess(a: HotkeyResult, b: HotkeyResult): number {
  return (a.process_display || '').localeCompare(b.process_display || '')
}

function formatProgress(percentage: number): string {
  if (!progress.value) return `${percentage}%`
  return `${percentage}% (${progress.value.done}/${progress.value.total})`
}

async function startProbe() {
  if (isProbing.value) return

  // 解析自定义补充热键
  const extraKeys = extraKeysInput.value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)

  isProbing.value = true
  error.value = ''
  done = false
  completed = false
  currentPage.value = 1

  try {
    await invoke<string>('hotkey_probe_start', { extraKeys })
    startPolling()
  } catch (e) {
    isProbing.value = false
    error.value = String(e)
    ElMessage.error(`探测启动失败: ${e}`)
  }
}

async function cancelProbe() {
  try {
    await invoke('hotkey_probe_cancel')
    ElMessage.info('已请求取消探测')
  } catch (e) {
    ElMessage.error(`取消失败: ${e}`)
  }
}

function startPolling() {
  stopPolling()
  pollTimer = window.setInterval(async () => {
    if (done) { stopPolling(); return }
    try {
      const status = await invoke<ProbeProgress>('hotkey_probe_status')
      progress.value = status
      if (status.is_finished) {
        done = true
        stopPolling()
        // 兜底：万一 complete 事件丢失，主动拉取结果
        if (status.error) {
          error.value = status.error
          isProbing.value = false
        } else {
          try {
            const finalResults = await invoke<HotkeyResult[]>('hotkey_probe_get_results')
            handleComplete(finalResults)
          } catch (e) {
            isProbing.value = false
            error.value = `拉取结果失败: ${e}`
          }
        }
      }
    } catch (e) {
      debug_log(`轮询失败: ${e}`)
    }
  }, 2000)
}

function stopPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

function handleComplete(payload: ProbeCompletePayload | HotkeyResult[]) {
  if (completed) return
  completed = true
  // 兼容两种调用：事件 payload 是 ProbeCompletePayload，兜底拉取是 HotkeyResult[]
  if (Array.isArray(payload)) {
    results.value = payload
    stats.value = computeStats(payload)
  } else {
    results.value = payload.results
    stats.value = payload.stats
  }
  isProbing.value = false
  done = true
  stopPolling()

  // 缓存到 store（仅内存，不持久化）
  store.hotkeyLastResult = results.value
  store.hotkeyLastStats = stats.value

  // 记录历史（AGENTS 规范：必须传 inputFull/outputFull）
  const inputFull = `候选集: ${results.value.length} 个热键 + 自定义: ${extraKeysInput.value || '无'}`
  const outputFull = buildOutputFull(results.value, stats.value!)
  store.addHistory({
    tool: 'hotkeyViewer',
    action: '探测全局快捷键占用',
    inputPreview: inputFull.slice(0, 50),
    outputPreview: `占用:${stats.value!.occupied} 可注册:${stats.value!.available} 系统保留:${stats.value!.reserved}`,
    inputFull,
    outputFull,
  })

  ElMessage.success(`探测完成: 共 ${stats.value!.total} 个，被占用 ${stats.value!.occupied} 个`)
}

function computeStats(list: HotkeyResult[]): ProbeStats {
  let available = 0, occupied = 0, reserved = 0
  for (const r of list) {
    if (r.status === 'Available') available++
    else if (r.status === 'Occupied') occupied++
    else if (r.status === 'SystemReserved') reserved++
  }
  return { total: list.length, available, occupied, reserved }
}

function buildOutputFull(list: HotkeyResult[], s: ProbeStats): string {
  const header = `占用: ${s.occupied} | 可注册: ${s.available} | 系统保留: ${s.reserved}\n详细列表:`
  const lines = list.map(r => {
    const proc = r.process_display || r.process_name || '-'
    return `${r.label} - ${statusLabel(r.status)} - ${proc}`
  })
  return `${header}\n${lines.join('\n')}`
}

async function exportCsv() {
  if (!results.value.length) return
  try {
    const path = await invoke<string>('hotkey_probe_export_csv', { results: results.value })
    ElMessage.success(`已导出到: ${path}`)
  } catch (e) {
    ElMessage.error(`导出失败: ${e}`)
  }
}

function debug_log(msg: string) {
  // ponytail: 简易 console 输出，仅 dev 模式
  if (import.meta.env.DEV) console.log(`[HotkeyView] ${msg}`)
}

let retryCount = 0
const MAX_RETRIES = 10

async function initAndProbe() {
  try {
    unlistenProgress = await listen<ProbeProgress>('hotkey-probe-progress', (e) => {
      progress.value = e.payload
    })
    unlistenComplete = await listen<ProbeCompletePayload>('hotkey-probe-complete', (e) => {
      handleComplete(e.payload)
    })
    startProbe()
  } catch (e) {
    retryCount++
    if (retryCount >= MAX_RETRIES) {
      debug_log(`初始化失败超过 ${MAX_RETRIES} 次，停止重试`)
      error.value = '初始化失败，API 未就绪'
      return
    }
    debug_log(`初始化失败，第 ${retryCount} 次重试: ${e}`)
    setTimeout(initAndProbe, 500)
  }
}

onMounted(async () => {
  if (store.hotkeyLastResult?.length) {
    results.value = store.hotkeyLastResult
    stats.value = store.hotkeyLastStats
  }
  setTimeout(initAndProbe, 100)
})

onActivated(() => {
  // AGENTS 经验 12: KeepAlive 缓存组件 onMounted 不会再次触发
  // 但 onActivated 会，用于切换回来时自动重新探测
  if (!isProbing.value && !done) {
    startProbe()
  }
})

onDeactivated(() => {
  stopPolling()
})

onUnmounted(() => {
  stopPolling()
  unlistenProgress?.()
  unlistenComplete?.()
})
</script>

<style scoped>
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.stat-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.stat-item strong {
  font-size: 22px;
  color: var(--text-primary);
  font-weight: 600;
}
.stat-danger { color: var(--accent-red) !important; }
.stat-success { color: var(--accent-green) !important; }
.stat-warning { color: var(--accent-orange) !important; }
.path-muted { color: var(--text-secondary); font-style: italic; }
.pagination-right {
  display: flex;
  justify-content: flex-end;
}
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
