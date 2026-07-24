<template>
  <div class="tool-container">
    <!-- 管理员权限提示 -->
    <div class="admin-banner">
      <span class="admin-icon">🛡️</span>
      系统级启动项（HKLM / 公共启动文件夹）的修改需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- 统计概览 -->
    <div v-if="!error && items.length" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ filteredItems.length }}</span>
        <span class="stat-label">总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ enabledCount }}</span>
        <span class="stat-label">已启用</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ disabledCount }}</span>
        <span class="stat-label">已禁用</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">开机启动项</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索名称/命令..." style="width: 220px" clearable />
          <el-select v-model="sourceFilter" size="small" style="width: 120px">
            <el-option label="全部来源" value="all" />
            <el-option label="注册表" value="registry" />
            <el-option label="启动文件夹" value="startup_folder" />
          </el-select>
          <el-select v-model="statusFilter" size="small" style="width: 100px">
            <el-option label="全部状态" value="all" />
            <el-option label="已启用" value="enabled" />
            <el-option label="已禁用" value="disabled" />
          </el-select>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="fetchItems">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !items.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」获取开机启动项列表" />
      </div>
    </div>

    <!-- 表格 -->
    <div v-if="items.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">启动项列表 ({{ filteredItems.length }} / {{ items.length }})</span>
        <el-button type="success" size="small" @click="showAddDialog = true">新增启动项</el-button>
      </div>
      <div class="card-body">
        <el-table
          :data="filteredItems" border size="small" max-height="600"
          style="width: 100%" v-loading="loading" row-key="rowKey">
          <el-table-column label="名称" min-width="180" sortable :sort-method="(a:any,b:any) => a.name.localeCompare(b.name)">
            <template #default="{ row }">
              <span class="item-name">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column label="命令" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <span class="item-command">{{ row.command || '—' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="来源" width="110" align="center">
            <template #default="{ row }">
              <el-tag size="small" :type="row.source === 'registry' ? 'primary' : 'success'">
                {{ row.source === 'registry' ? '注册表' : '启动文件夹' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="位置" width="120" show-overflow-tooltip>
            <template #default="{ row }">
              <span class="location-text">{{ locationShort(row.location) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="90" align="center">
            <template #default="{ row }">
              <el-tag size="small" :type="row.enabled ? 'success' : 'info'">
                {{ row.enabled ? '已启用' : '已禁用' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="100" fixed="right" align="center">
            <template #default="{ row }">
              <el-dropdown
                trigger="click"
                @command="(cmd: string) => handleAction(row, cmd as 'enable' | 'disable' | 'delete')"
                :disabled="operatingItems.has(row.rowKey)">
                <el-button type="primary" size="small" plain :loading="operatingItems.has(row.rowKey)">
                  操作<el-icon class="el-icon--right"><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="enable" v-if="!row.enabled">
                      <el-icon style="margin-right: 6px; color: var(--el-color-success)"><CircleCheck /></el-icon>启用
                    </el-dropdown-item>
                    <el-dropdown-item command="disable" v-if="row.enabled">
                      <el-icon style="margin-right: 6px; color: var(--el-color-warning)"><VideoPause /></el-icon>禁用
                    </el-dropdown-item>
                    <el-dropdown-item divided command="delete">
                      <el-icon style="margin-right: 6px; color: var(--el-color-danger)"><Delete /></el-icon>删除
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 底部栏 -->
    <div v-if="items.length" class="tool-card">
      <div class="card-body">
        <div class="bottom-bar">
          <div class="auto-refresh">
            <span class="group-label">自动刷新</span>
            <el-switch v-model="autoRefresh" size="small" @change="toggleAutoRefresh" />
            <el-select v-if="autoRefresh" v-model="refreshInterval" size="small" style="width: 80px" @change="restartAutoRefresh">
              <el-option label="5s" :value="5" />
              <el-option label="30s" :value="30" />
              <el-option label="60s" :value="60" />
            </el-select>
          </div>
          <div class="bottom-actions">
            <el-button size="small" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增启动项弹窗 -->
    <el-dialog v-model="showAddDialog" title="新增启动项" width="480px" :close-on-click-modal="false" @closed="addForm = { name: '', command: '', source: 'startup_folder' }">
      <el-form :model="addForm" label-width="80px" label-position="left">
        <el-form-item label="名称">
          <el-input v-model="addForm.name" placeholder="启动项显示名称" />
        </el-form-item>
        <el-form-item label="命令">
          <div style="display: flex; gap: 6px; width: 100%;">
            <el-input v-model="addForm.command" placeholder="程序路径，如 C:\app.exe" style="flex: 1" />
            <el-button @click="pickFile" :icon="FolderOpened">选择</el-button>
          </div>
        </el-form-item>
        <el-form-item label="位置">
          <el-radio-group v-model="addForm.source">
            <el-radio value="registry">注册表 (HKCU\Run)</el-radio>
            <el-radio value="startup_folder">启动文件夹</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" :loading="adding" @click="handleAdd">确认添加</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  getStartupItems,
  enableStartupItem,
  disableStartupItem,
  deleteStartupItem,
  addStartupItem,
  formatTimestamp,
  type StartupItemInfo,
  type StartupOpResult,
} from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import { ArrowDown, VideoPause, Delete, CircleCheck, FolderOpened } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { homeDir } from '@tauri-apps/api/path'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

const items = ref<StartupItemInfo[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const sourceFilter = ref('all')
const statusFilter = ref('all')
const operatingItems = ref(new Set<string>())
const showAddDialog = ref(false)
const adding = ref(false)
const addForm = ref({ name: '', command: '', source: 'startup_folder' })

const autoRefresh = ref(false)
const refreshInterval = ref(30)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ============ 计算属性 ============

const itemsWithKey = computed(() =>
  items.value.map((item, i) => ({ ...item, rowKey: `${item.location}|${item.name}|${i}` }))
)

const filteredItems = computed(() => {
  let result = itemsWithKey.value
  if (sourceFilter.value !== 'all') {
    result = result.filter(t => t.source === sourceFilter.value)
  }
  if (statusFilter.value === 'enabled') {
    result = result.filter(t => t.enabled)
  } else if (statusFilter.value === 'disabled') {
    result = result.filter(t => !t.enabled)
  }
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    result = result.filter(t =>
      t.name.toLowerCase().includes(q) ||
      t.command.toLowerCase().includes(q)
    )
  }
  return result
})

const enabledCount = computed(() => items.value.filter(t => t.enabled).length)
const disabledCount = computed(() => items.value.filter(t => !t.enabled).length)

const locationShort = (loc: string) => {
  if (loc.toLowerCase().includes('hklm')) return 'HKLM\\...\\Run'
  if (loc.toLowerCase().includes('hkcu')) return 'HKCU\\...\\Run'
  if (loc.toLowerCase().includes('startup')) {
    return loc.includes('ProgramData') ? '公共启动文件夹' : '用户启动文件夹'
  }
  return loc
}

// ============ 数据采集 ============

const fetchItems = async () => {
  loading.value = true
  error.value = ''
  try {
    items.value = await getStartupItems()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'startupItems',
      action: '查看开机启动项',
      inputPreview: '',
      outputPreview: `${items.value.length} 个启动项`,
      inputFull: '',
      outputFull: items.value.map(t => `${t.name} [${t.enabled ? '启用' : '禁用'}] ${t.command}`).join('\n'),
    })
  } catch (e) {
    error.value = '无法获取启动项列表: ' + String(e)
  } finally {
    loading.value = false
  }
}

// ============ 操作处理 ============

const handleAction = async (item: any, action: 'enable' | 'disable' | 'delete') => {
  const actionLabel = { enable: '启用', disable: '禁用', delete: '删除' }[action]
  const itemKey = item.rowKey

  if (action === 'delete') {
    const ok = await confirm.ask(
      '删除启动项',
      `确定删除启动项 "${item.name}"？\n此操作不可恢复。`,
      { type: 'danger', confirmText: '删除' }
    )
    if (!ok) return
  }

  operatingItems.value.add(itemKey)
  try {
    const cmd = {
      enable: enableStartupItem,
      disable: disableStartupItem,
      delete: deleteStartupItem,
    }[action]
    const result: StartupOpResult = await cmd(item.name, item.location, item.source)

    store.addHistory({
      tool: 'startupItems',
      action: `${actionLabel}启动项`,
      inputPreview: `${item.name} (${item.source})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name: item.name, location: item.location, source: item.source, action }),
      outputFull: JSON.stringify(result),
    })

    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员') || result.message.includes('拒绝')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }

    if (action === 'delete' && result.success) {
      items.value = items.value.filter((_, i) => itemsWithKey.value[i].rowKey !== itemKey)
    } else {
      await new Promise(r => setTimeout(r, 300))
      await fetchItems()
    }
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    operatingItems.value.delete(itemKey)
  }
}

const pickFile = async () => {
  const home = await homeDir()
  const defaultPath = `${home.replace(/\\$/, '')}\\AppData\\Local\\Programs`
  const selected = await open({
    multiple: false,
    defaultPath,
    filters: [{ name: '可执行文件', extensions: ['exe', 'bat', 'cmd', 'com'] }],
  })
  if (selected) {
    addForm.value.command = selected as string
    // 名称未填时自动回显文件名（不含扩展名）
    if (!addForm.value.name.trim()) {
      const fileName = (selected as string).split('\\').pop() || ''
      const dotIndex = fileName.lastIndexOf('.')
      addForm.value.name = dotIndex > 0 ? fileName.substring(0, dotIndex) : fileName
    }
  }
}

const handleAdd = async () => {
  const name = addForm.value.name.trim()
  const command = addForm.value.command.trim()
  if (!name || !command) {
    ElMessage.warning('名称和命令不能为空')
    return
  }
  adding.value = true
  try {
    const result = await addStartupItem(name, command, addForm.value.source)
    if (result.success) {
      ElMessage.success('已添加')
      showAddDialog.value = false
      addForm.value = { name: '', command: '', source: 'startup_folder' }
      await new Promise(r => setTimeout(r, 300))
      await fetchItems()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    adding.value = false
  }
}

// ============ CSV 导出 ============

const exportCsv = async () => {
  const BOM = '\uFEFF'
  const header = '名称,命令,来源,位置,状态'
  const rows = filteredItems.value.map(t =>
    `${t.name},${t.command},${t.source === 'registry' ? '注册表' : '启动文件夹'},${t.location},${t.enabled ? '已启用' : '已禁用'}`
  )
  const csv = BOM + header + '\n' + rows.join('\n')
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `开机启动项_${new Date().toISOString().slice(0, 10)}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

// ============ 自动刷新 ============

const toggleAutoRefresh = (val: boolean) => {
  if (val) {
    refreshTimer = setInterval(fetchItems, refreshInterval.value * 1000)
  } else if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

const restartAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(fetchItems, refreshInterval.value * 1000)
  }
}

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

onMounted(() => {
  fetchItems()
})
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
  color: var(--text-primary);
  margin-bottom: 16px;
}
.admin-icon { font-size: 16px; }

.stats-row { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 80px;
}
.stat-number { font-size: 22px; font-weight: 700; color: var(--accent-cyan); }
.stat-label { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }

.system-toggle { display: flex; align-items: center; gap: 6px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }

.item-name { font-weight: 500; }
.item-command { font-size: 12px; color: var(--text-secondary); font-family: 'Consolas', 'Courier New', monospace; }
.location-text { font-size: 12px; color: var(--text-muted); }

.bottom-bar { display: flex; justify-content: space-between; align-items: center; }
.auto-refresh { display: flex; align-items: center; gap: 8px; }
.bottom-actions { display: flex; gap: 8px; }

.error-message {
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
}

:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
:deep(.el-table__fixed-right) { background: var(--bg-card) !important; }
:deep(.el-table__fixed-right::before) {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 1px;
  height: 100%;
  background: var(--border-color);
}
:deep(.el-table__fixed-right-patch) { background: var(--bg-input) !important; }

:deep(.el-dialog) { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; }
:deep(.el-dialog__title) { color: var(--accent-cyan); font-weight: 600; }
:deep(.el-dialog__body) { padding: 20px 24px; }
</style>