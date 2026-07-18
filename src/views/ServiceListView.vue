<template>
  <div class="tool-container">
    <div class="admin-banner">
      <span class="admin-icon">🛡️</span>
      启动/停止/重启服务需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">服务列表</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索服务名/显示名称..." style="width: 220px" clearable />
          <el-select v-model="statusFilter" size="small" style="width: 120px">
            <el-option label="全部状态" value="all" />
            <el-option label="运行中" value="Running" />
            <el-option label="已停止" value="Stopped" />
          </el-select>
          <el-button type="primary" size="small" :loading="loading" @click="fetchServices">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <div v-if="!loading && !error && !services.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」获取服务列表" />
      </div>
    </div>

    <div v-if="services.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">服务 ({{ filteredServices.length }} / {{ services.length }})</span>
      </div>
      <div class="card-body">
        <el-table :data="filteredServices" border size="small" max-height="600" style="width: 100%" v-loading="loading">
          <el-table-column prop="name" label="名称" min-width="160" sortable />
          <el-table-column prop="display_name" label="显示名称" min-width="200" sortable />
          <el-table-column label="状态" width="90" sortable :sort-method="sortByStatus">
            <template #default="{ row }">
              <el-tag :type="row.status === 'Running' ? 'success' : 'info'" size="small">
                {{ row.status === 'Running' ? '运行中' : '已停止' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="启动类型" width="80">
            <template #default="{ row }">
              {{ startTypeLabel(row.start_type) }}
            </template>
          </el-table-column>
          <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button
                v-if="row.status === 'Stopped'"
                type="success" size="small" link
                :loading="operatingServices.has(row.name)"
                @click="handleServiceAction(row, 'start')">
                启动
              </el-button>
              <el-button
                v-if="row.status === 'Running'"
                type="danger" size="small" link
                :loading="operatingServices.has(row.name)"
                @click="handleServiceAction(row, 'stop')">
                停止
              </el-button>
              <el-button
                v-if="row.status === 'Running'"
                type="warning" size="small" link
                :loading="operatingServices.has(row.name)"
                @click="handleServiceAction(row, 'restart')">
                重启
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { ElMessageBox, ElMessage } from 'element-plus'
import { getServices, startService, stopService, restartService, type ServiceItem, type ServiceResult } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const services = ref<ServiceItem[]>([])
const error = ref('')
const loading = ref(false)
const searchQuery = ref('')
const statusFilter = ref('all')
const operatingServices = ref(new Set<string>())

function startTypeLabel(type: string): string {
  const map: Record<string, string> = {
    'Auto': '自动',
    'Manual': '手动',
    'Disabled': '禁用',
  }
  return map[type] || type
}

const sortByStatus = (a: ServiceItem, b: ServiceItem) => {
  if (a.status === b.status) return 0
  return a.status === 'Running' ? -1 : 1
}

let searchTimer: ReturnType<typeof setTimeout> | null = null
const searchTrigger = ref('')

watch(searchQuery, (val) => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { searchTrigger.value = val }, 300)
})

const filteredServices = computed(() => {
  let result = [...services.value]
  const q = searchTrigger.value.toLowerCase().trim()
  if (q) {
    result = result.filter(s =>
      s.name.toLowerCase().includes(q) ||
      s.display_name.toLowerCase().includes(q)
    )
  }
  if (statusFilter.value !== 'all') {
    result = result.filter(s => s.status === statusFilter.value)
  }
  return result
})

const handleServiceAction = async (service: ServiceItem, action: 'start' | 'stop' | 'restart') => {
  const actionLabel = { start: '启动', stop: '停止', restart: '重启' }[action]
  try {
    await ElMessageBox.confirm(
      `确定${actionLabel}服务 "${service.display_name}" (${service.name})？`,
      `${actionLabel}服务确认`,
      { type: 'warning', confirmButtonText: actionLabel, cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  operatingServices.value.add(service.name)
  try {
    const cmd = { start: startService, stop: stopService, restart: restartService }[action]
    const result: ServiceResult = await cmd(service.name)
    store.addHistory({
      tool: 'serviceList',
      action: `${actionLabel}服务`,
      inputPreview: `${service.name} (${service.display_name})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name: service.name, displayName: service.display_name, action }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }
    await new Promise(r => setTimeout(r, 500))
    await fetchServices()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    operatingServices.value.delete(service.name)
  }
}

const fetchServices = async () => {
  loading.value = true
  error.value = ''
  try {
    services.value = await getServices()
    store.addHistory({
      tool: 'serviceList',
      action: '查看服务列表',
      inputPreview: '',
      outputPreview: `${services.value.length} 个服务`,
      inputFull: '',
      outputFull: services.value.map(s => `${s.name} (${s.display_name}) [${s.status}]`).join('\n'),
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => fetchServices())
</script>

<style scoped>
.admin-banner {
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #f59e0b;
  margin-bottom: 16px;
}
.admin-icon { font-size: 16px; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>