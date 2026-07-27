<template>
  <div class="tool-container">
    <!-- 统计概览 -->
    <div v-if="!error && connections.length" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ filteredConnections.length }}</span>
        <span class="stat-label">连接总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ tcpCount }}</span>
        <span class="stat-label">TCP</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ udpCount }}</span>
        <span class="stat-label">UDP</span>
      </div>
      <div class="stat-card" v-for="s in stateStats" :key="s.state">
        <span class="stat-number">{{ s.count }}</span>
        <span class="stat-label">{{ stateLabel(s.state) }}</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">网络连接</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索端口/PID/进程名/地址..." style="width: 240px" clearable />
          <el-select v-model="protocolFilter" size="small" style="width: 90px">
            <el-option label="全部" value="all" />
            <el-option label="TCP" value="TCP" />
            <el-option label="UDP" value="UDP" />
          </el-select>
          <el-select v-model="stateFilter" size="small" style="width: 120px">
            <el-option label="全部状态" value="all" />
            <el-option label="LISTENING" value="LISTENING" />
            <el-option label="ESTABLISHED" value="ESTABLISHED" />
            <el-option label="TIME_WAIT" value="TIME_WAIT" />
            <el-option label="CLOSE_WAIT" value="CLOSE_WAIT" />
            <el-option label="SYN_SENT" value="SYN_SENT" />
          </el-select>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="refresh">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !connections.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击「刷新」获取网络连接" />
      </div>
    </div>

    <!-- 连接表格 -->
    <div v-if="connections.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">连接列表 ({{ filteredConnections.length }} / {{ connections.length }})</span>
      </div>
      <div class="card-body">
        <DataTable :data="filteredConnections" max-height="600" style="width: 100%">
          <el-table-column label="协议" width="70">
            <template #default="{ row }">
              <el-tag :type="row.protocol === 'TCP' ? '' : 'success'" size="small">{{ row.protocol }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="本地地址" width="170">
            <template #default="{ row }">
              <span class="mono-text">{{ row.local_addr }}</span>
            </template>
          </el-table-column>
          <el-table-column label="远程地址" width="170">
            <template #default="{ row }">
              <span class="mono-text">{{ row.remote_addr }}</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="120">
            <template #default="{ row }">
              <el-tag v-if="row.state" :type="stateTagType(row.state)" size="small">{{ row.state }}</el-tag>
              <span v-else class="text-secondary">—</span>
            </template>
          </el-table-column>
          <el-table-column prop="pid" label="PID" width="70" sortable />
          <el-table-column prop="process_name" label="进程名" width="140" sortable>
            <template #default="{ row }">
              <span :class="{ 'text-secondary': row.process_name === '(已退出)' }">{{ row.process_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="process_path" label="进程路径" min-width="200" show-overflow-tooltip />
          <el-table-column label="操作" width="180" fixed="right">
            <template #default="{ row }">
              <el-button
                type="danger" size="small" link
                :disabled="row.pid === 0 || row.process_name === '(已退出)'"
                :loading="killingPids.has(row.pid)"
                @click="handleKill(row)">
                结束进程
              </el-button>
              <el-button
                type="warning" size="small" link
                :disabled="row.pid === 0 || row.process_name === '(已退出)'"
                :loading="killingPids.has(row.pid)"
                @click="handleReleasePort(row)">
                释放端口
              </el-button>
            </template>
          </el-table-column>
        </DataTable>
      </div>
    </div>

    <!-- 底部栏 -->
    <div v-if="connections.length" class="tool-card">
      <div class="card-body">
        <div class="bottom-bar">
          <div class="auto-refresh">
            <span class="group-label">自动刷新</span>
            <el-switch v-model="autoRefresh" size="small" @change="toggleAutoRefresh" />
            <el-select v-if="autoRefresh" v-model="refreshInterval" size="small" style="width: 80px" @change="restartAutoRefresh">
              <el-option label="5s" :value="5" />
              <el-option label="10s" :value="10" />
              <el-option label="30s" :value="30" />
            </el-select>
          </div>
          <div class="bottom-actions">
            <el-button size="small" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { getNetworkConnections, killProcess, formatTimestamp, type NetworkConnection } from '@/utils/systemInfoClient'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import DataTable from '@/components/DataTable.vue'

const { confirm } = useConfirmDialog()

const connections = ref<NetworkConnection[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const protocolFilter = ref('all')
const stateFilter = ref('all')
const killingPids = ref(new Set<number>())

// 自动刷新
const autoRefresh = ref(false)
const refreshInterval = ref(5)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ============ 筛选 ============

const filteredConnections = computed(() => {
  let result = connections.value

  // 协议筛选
  if (protocolFilter.value !== 'all') {
    result = result.filter(c => c.protocol === protocolFilter.value)
  }

  // 状态筛选
  if (stateFilter.value !== 'all') {
    result = result.filter(c => c.state === stateFilter.value)
  }

  // 搜索
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    result = result.filter(c =>
      c.local_addr.toLowerCase().includes(q) ||
      c.remote_addr.toLowerCase().includes(q) ||
      c.process_name.toLowerCase().includes(q) ||
      c.pid.toString().includes(q) ||
      c.state.toLowerCase().includes(q)
    )
  }

  return result
})

// ============ 统计 ============

const tcpCount = computed(() => filteredConnections.value.filter(c => c.protocol === 'TCP').length)
const udpCount = computed(() => filteredConnections.value.filter(c => c.protocol === 'UDP').length)

const stateStats = computed(() => {
  const map = new Map<string, number>()
  for (const c of filteredConnections.value) {
    if (c.state) {
      map.set(c.state, (map.get(c.state) || 0) + 1)
    }
  }
  return Array.from(map.entries())
    .map(([state, count]) => ({ state, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 5)
})

const stateLabel = (state: string) => {
  const labels: Record<string, string> = {
    LISTENING: '监听', ESTABLISHED: '已建立', TIME_WAIT: '等待',
    CLOSE_WAIT: '关闭等待', SYN_SENT: '同步发送',
  }
  return labels[state] || state
}

const stateTagType = (state: string): 'success' | 'primary' | 'warning' | 'danger' | 'info' => {
  const map: Record<string, 'success' | 'primary' | 'warning' | 'danger' | 'info'> = {
    LISTENING: 'success', ESTABLISHED: 'primary', TIME_WAIT: 'warning',
    CLOSE_WAIT: 'danger', SYN_SENT: 'info',
  }
  return map[state] || 'info'
}

// ============ 数据采集 ============

const refresh = async () => {
  loading.value = true
  error.value = ''
  try {
    connections.value = await getNetworkConnections()
    lastRefresh.value = formatTimestamp()
  } catch (e) {
    error.value = '无法获取网络连接信息: ' + String(e)
  } finally {
    loading.value = false
  }
  // 重置自动刷新计时器
  if (autoRefresh.value && refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  }
}

// ============ 操作 ============

const handleKill = async (row: NetworkConnection) => {
  const ok = await confirm.ask(
    '结束进程确认',
    `确定结束进程 "${row.process_name}" (PID: ${row.pid})？\n强制结束可能导致未保存的数据丢失。`,
    { type: 'danger', confirmText: '结束' }
  )
  if (!ok) return

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    if (result.success) {
      ElMessage.success(result.message)
    } else {
      ElMessage.error(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    refresh()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}

const handleReleasePort = async (row: NetworkConnection) => {
  const port = row.local_addr.split(':').pop() || ''
  const ok = await confirm.ask(
    '释放端口确认',
    `确定释放端口 ${port}？\n将结束占用进程 "${row.process_name}" (PID: ${row.pid})。`,
    { type: 'warning', confirmText: '释放' }
  )
  if (!ok) return

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    if (result.success) {
      ElMessage.success(`端口 ${port} 已释放：${result.message}`)
    } else {
      ElMessage.error(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    refresh()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}

const exportCsv = async () => {
  const BOM = '\uFEFF'
  const header = '协议,本地地址,远程地址,状态,PID,进程名,进程路径'
  const rows = filteredConnections.value.map(c =>
    `${c.protocol},${c.local_addr},${c.remote_addr},${c.state},${c.pid},"${c.process_name}","${c.process_path}"`
  )
  const csv = BOM + header + '\n' + rows.join('\n')

  const now = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  const filename = `网络连接_${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.csv`

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const savedPath = await invoke<string>('save_text_with_dialog', { content: csv, filename })
    if (savedPath) {
      ElMessage.success(`已导出到: ${savedPath}`)
    }
  } catch (e) {
    ElMessage.error('导出失败: ' + String(e))
  }
}

// ============ 自动刷新 ============

const toggleAutoRefresh = (val: boolean) => {
  if (val) {
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  } else {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }
}

const restartAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  }
}

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

onMounted(() => {
  nextTick(() => {
    try {
      refresh()
    } catch (e) {
      console.error('[networkConnections] init error:', e)
      error.value = '初始化失败: ' + String(e)
    }
  })
})
</script>

<style scoped>
.stats-row { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
.stat-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; padding: 12px 16px; display: flex; flex-direction: column; align-items: center; min-width: 80px; }
.stat-number { font-size: 22px; font-weight: 700; color: var(--accent-cyan); }
.stat-label { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }
.mono-text { font-family: 'Consolas', 'Courier New', monospace; font-size: 12px; }
.text-secondary { color: var(--text-secondary); }
.bottom-bar { display: flex; justify-content: space-between; align-items: center; }
.auto-refresh { display: flex; align-items: center; gap: 8px; }
.bottom-actions { display: flex; gap: 8px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
</style>