<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">进程列表</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索进程名/PID..." style="width: 200px" clearable />
          <el-select v-model="sortBy" size="small" style="width: 120px">
            <el-option label="按 CPU 排序" value="cpu" />
            <el-option label="按内存排序" value="memory" />
          </el-select>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="collecting" @click="collect">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <div v-if="!data || !data.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」采集进程列表" />
      </div>
    </div>

    <div v-if="data && data.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">进程 ({{ filteredData.length }} / {{ data.length }})</span>
      </div>
      <div class="card-body">
        <el-table :data="filteredData" border size="small" max-height="600" style="width: 100%">
          <el-table-column prop="pid" label="PID" width="80" sortable />
          <el-table-column prop="name" label="名称" min-width="160" sortable />
          <el-table-column label="CPU%" width="100" sortable :sort-method="sortByCpu">
            <template #default="{ row }">{{ row.cpu_usage.toFixed(1) }}%</template>
          </el-table-column>
          <el-table-column label="内存" width="120" sortable :sort-method="sortByMemory">
            <template #default="{ row }">{{ formatBytes(row.memory_bytes) }}</template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="80" />
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button type="danger" size="small" link
                :loading="killingPids.has(row.pid)"
                @click="handleKill(row)">
                结束
              </el-button>
              <el-button type="danger" size="small" link
                v-if="getSameNameCount(row.name) > 1"
                :loading="killingNames.has(row.name)"
                @click="handleKillAll(row)">
                全部结束
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { killProcess, killProcessByName, formatBytes, formatTimestamp, type ProcessItem } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useBackgroundCollect } from '@/composables/useBackgroundCollect'

const store = useToolboxStore()
const data = ref<ProcessItem[]>([])
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const sortBy = ref('cpu')

const killingPids = ref(new Set<number>())
const killingNames = ref(new Set<string>())

const getSameNameCount = (name: string) => {
  return data.value.filter(p => p.name === name).length
}

const handleKillAll = async (row: ProcessItem) => {
  const count = getSameNameCount(row.name)
  try {
    await ElMessageBox.confirm(
      `确定结束所有 "${row.name}" 进程？共 ${count} 个进程。\n强制结束可能导致未保存的数据丢失。`,
      '批量结束进程确认',
      { type: 'warning', confirmButtonText: '全部结束', cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  killingNames.value.add(row.name)
  try {
    const result = await killProcessByName(row.name)
    store.addHistory({
      tool: 'processList',
      action: '结束全部同名进程',
      inputPreview: `${row.name} (${count} 个)`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name: row.name, count }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    collect()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingNames.value.delete(row.name)
  }
}

const handleKill = async (row: ProcessItem) => {
  try {
    await ElMessageBox.confirm(
      `确定结束进程 "${row.name}" (PID: ${row.pid})？\n强制结束可能导致未保存的数据丢失。`,
      '结束进程确认',
      { type: 'warning', confirmButtonText: '结束', cancelButtonText: '取消' }
    )
  } catch {
    return  // 用户取消
  }

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    store.addHistory({
      tool: 'processList',
      action: '结束进程',
      inputPreview: `${row.name} (PID: ${row.pid})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ pid: row.pid, name: row.name }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员') || result.message.includes('系统关键') || result.message.includes('无法结束')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    collect()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}

let searchTimer: ReturnType<typeof setTimeout> | null = null
const searchTrigger = ref('')

watch(searchQuery, (val) => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { searchTrigger.value = val }, 300)
})

const filteredData = computed(() => {
  let result = [...data.value]
  const q = searchTrigger.value.toLowerCase().trim()
  if (q) {
    result = result.filter(p =>
      p.name.toLowerCase().includes(q) || p.pid.toString().includes(q)
    )
  }
  if (sortBy.value === 'cpu') {
    result.sort((a, b) => b.cpu_usage - a.cpu_usage)
  } else {
    result.sort((a, b) => b.memory_bytes - a.memory_bytes)
  }
  return result
})

const sortByCpu = (a: ProcessItem, b: ProcessItem) => b.cpu_usage - a.cpu_usage
const sortByMemory = (a: ProcessItem, b: ProcessItem) => b.memory_bytes - a.memory_bytes

const { collect, collecting } = useBackgroundCollect('process')

// 采集完成 → 填充数据 + 记录历史（watch 替代 onMounted，兼容 KeepAlive 缓存）
watch(() => store.collectResults['process'], (val) => {
  if (!val) return
  const list = val as ProcessItem[]
  data.value = list
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'processList',
    action: '查看进程列表',
    inputPreview: '',
    outputPreview: `${list.length} 个进程`,
    inputFull: '',
    outputFull: list.map(p => `${p.name} (PID: ${p.pid})`).join('\n'),
  })
}, { immediate: true })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 8px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
