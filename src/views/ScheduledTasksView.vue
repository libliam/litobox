<template>
  <div class="tool-container">
    <!-- 管理员权限提示 -->
    <div class="admin-banner">
      <span class="admin-icon">🛡️</span>
      启用/禁用/删除计划任务需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- 统计概览 -->
    <div v-if="!error && tasks.length" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ filteredTasks.length }}</span>
        <span class="stat-label">总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Ready') }}</span>
        <span class="stat-label">就绪</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Running') }}</span>
        <span class="stat-label">运行中</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Disabled') }}</span>
        <span class="stat-label">已禁用</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">计划任务</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索任务名/路径/描述/作者..." style="width: 260px" clearable />
          <el-select v-model="stateFilter" size="small" style="width: 120px">
            <el-option label="全部状态" value="all" />
            <el-option label="就绪" value="Ready" />
            <el-option label="运行中" value="Running" />
            <el-option label="已禁用" value="Disabled" />
            <el-option label="未知" value="Unknown" />
          </el-select>
          <span class="system-toggle">
            <span class="group-label">显示系统任务</span>
            <el-switch v-model="includeSystem" size="small" @change="onSystemToggleChange" />
          </span>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="fetchTasks">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !tasks.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」获取计划任务列表" />
      </div>
    </div>

    <!-- 任务表格 -->
    <div v-if="tasks.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">任务列表 ({{ filteredTasks.length }} / {{ tasks.length }})</span>
      </div>
      <div class="card-body">
        <el-table
          :data="filteredTasks" border size="small" max-height="600"
          style="width: 100%" v-loading="loading" row-key="rowKey">
          <el-table-column type="expand">
            <template #default="{ row }">
              <div class="expand-detail">
                <div class="detail-row"><span class="detail-label">作者：</span>{{ row.author || '—' }}</div>
                <div class="detail-row"><span class="detail-label">运行账户：</span>{{ row.principal || '—' }}</div>
                <div class="detail-row"><span class="detail-label">完整路径：</span>{{ row.task_path }}{{ row.task_name }}</div>
                <div class="detail-row"><span class="detail-label">描述：</span>{{ row.description || '—' }}</div>
                <div class="detail-row">
                  <span class="detail-label">完整触发器：</span>
                  <pre class="detail-json">{{ formatJson(row.triggers_json) }}</pre>
                </div>
                <div class="detail-row">
                  <span class="detail-label">执行动作：</span>
                  <pre class="detail-json">{{ formatJson(row.actions_json) }}</pre>
                </div>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="名称" min-width="220" sortable :sort-method="sortByName">
            <template #default="{ row }">
              <div class="task-name-cell">
                <span class="task-name">{{ row.task_name }}</span>
                <span class="task-path-hint">{{ row.task_path }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100" sortable :sort-method="sortByState">
            <template #default="{ row }">
              <el-tag :type="stateTagType(row.state)" size="small">{{ stateLabel(row.state) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="触发器" width="130">
            <template #default="{ row }">
              <span>{{ row.trigger_brief }}</span>
            </template>
          </el-table-column>
          <el-table-column label="上次运行" width="180" sortable prop="last_run_time">
            <template #default="{ row }">
              <div class="last-run-cell">
                <span>{{ row.last_run_time || '—' }}</span>
                <span v-if="row.last_run_time" :class="row.last_task_result === 0 ? 'result-ok' : 'result-fail'">
                  {{ row.last_task_result === 0 ? '✓' : `✗ ${row.last_task_result}` }}
                </span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="下次运行" width="170" sortable prop="next_run_time">
            <template #default="{ row }">
              <span>{{ row.next_run_time || '—' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="100" fixed="right" align="center">
            <template #default="{ row }">
              <el-dropdown
                trigger="click"
                @command="(cmd: string) => handleAction(row, cmd as 'enable' | 'disable' | 'run' | 'delete')"
                :disabled="operatingTasks.has(row.rowKey)">
                <el-button type="primary" size="small" plain :loading="operatingTasks.has(row.rowKey)">
                  操作<el-icon class="el-icon--right"><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="run" v-if="row.state === 'Ready' || row.state === 'Disabled'">
                      <el-icon style="margin-right: 6px"><VideoPlay /></el-icon>立即运行
                    </el-dropdown-item>
                    <el-dropdown-item command="enable" v-if="row.state === 'Disabled'">
                      <el-icon style="margin-right: 6px; color: var(--el-color-success)"><CircleCheck /></el-icon>启用
                    </el-dropdown-item>
                    <el-dropdown-item command="disable" v-if="row.state === 'Ready' || row.state === 'Running'">
                      <el-icon style="margin-right: 6px; color: var(--el-color-warning)"><VideoPause /></el-icon>禁用
                    </el-dropdown-item>
                    <el-dropdown-item divided command="delete" :disabled="row.is_system">
                      <el-icon style="margin-right: 6px; color: var(--el-color-danger)"><Delete /></el-icon>
                      <span :style="{ color: row.is_system ? 'var(--text-muted)' : '' }">删除{{ row.is_system ? '（系统任务）' : '' }}</span>
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
    <div v-if="tasks.length" class="tool-card">
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  getScheduledTasks,
  enableScheduledTask,
  disableScheduledTask,
  runScheduledTask,
  deleteScheduledTask,
  formatTimestamp,
  type ScheduledTask,
  type TaskOpResult,
} from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import { ArrowDown, VideoPlay, VideoPause, Delete, CircleCheck } from '@element-plus/icons-vue'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

const tasks = ref<ScheduledTask[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const stateFilter = ref('all')
const includeSystem = ref(false)
const operatingTasks = ref(new Set<string>())

// 自动刷新（默认关，计划任务变化频率低）
const autoRefresh = ref(false)
const refreshInterval = ref(30)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ============ 计算属性 ============

// 为每行加 rowKey，便于操作按钮 loading 标识
const tasksWithKey = computed(() =>
  tasks.value.map(t => ({ ...t, rowKey: `${t.task_path}|${t.task_name}` }))
)

const filteredTasks = computed(() => {
  let result = tasksWithKey.value
  if (stateFilter.value !== 'all') {
    result = result.filter(t => t.state === stateFilter.value)
  }
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    result = result.filter(t =>
      t.task_name.toLowerCase().includes(q) ||
      t.task_path.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      t.author.toLowerCase().includes(q)
    )
  }
  return result
})

const stateCount = (state: string) => filteredTasks.value.filter(t => t.state === state).length

// ============ 渲染辅助 ============

const stateLabel = (s: string) => {
  const map: Record<string, string> = { Ready: '就绪', Running: '运行中', Disabled: '已禁用', Unknown: '未知' }
  return map[s] || s
}

const stateTagType = (s: string): 'success' | 'primary' | 'info' | 'warning' => {
  const map: Record<string, string> = { Ready: 'primary', Running: 'success', Disabled: 'info', Unknown: 'warning' }
  return (map[s] || 'info') as 'success' | 'primary' | 'info' | 'warning'
}

const sortByName = (a: any, b: any) => a.task_name.localeCompare(b.task_name)
const sortByState = (a: any, b: any) => {
  const order = ['Running', 'Ready', 'Disabled', 'Unknown']
  return order.indexOf(a.state) - order.indexOf(b.state)
}

const formatJson = (jsonStr: string): string => {
  if (!jsonStr || jsonStr === '[]') return '—'
  try {
    return JSON.stringify(JSON.parse(jsonStr), null, 2)
  } catch {
    return jsonStr
  }
}

// ============ 数据采集 ============

const fetchTasks = async () => {
  loading.value = true
  error.value = ''
  try {
    tasks.value = await getScheduledTasks(includeSystem.value)
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'scheduledTasks',
      action: '查看计划任务列表',
      inputPreview: includeSystem.value ? '含系统任务' : '不含系统任务',
      outputPreview: `${tasks.value.length} 个任务`,
      inputFull: JSON.stringify({ includeSystem: includeSystem.value }),
      outputFull: tasks.value.map(t => `${t.task_name} [${t.state}] ${t.task_path}`).join('\n'),
    })
  } catch (e) {
    error.value = '无法获取计划任务列表: ' + String(e)
  } finally {
    loading.value = false
  }
}

const onSystemToggleChange = () => {
  fetchTasks()
}

// ============ 操作处理 ============

const handleAction = async (task: any, action: 'enable' | 'disable' | 'run' | 'delete') => {
  const actionLabel = { enable: '启用', disable: '禁用', run: '立即运行', delete: '删除' }[action]
  const taskKey = task.rowKey

  // 删除需 danger 二次确认
  if (action === 'delete') {
    const ok = await confirm.ask(
      '删除计划任务',
      `确定删除任务 "${task.task_name}" (路径 ${task.task_path})？\n此操作不可恢复，可能影响相关程序正常运行。`,
      { type: 'danger', confirmText: '删除' }
    )
    if (!ok) return
  }

  operatingTasks.value.add(taskKey)
  try {
    const cmd = {
      enable: enableScheduledTask,
      disable: disableScheduledTask,
      run: runScheduledTask,
      delete: deleteScheduledTask,
    }[action]
    const result: TaskOpResult = await cmd(task.task_name, task.task_path)

    store.addHistory({
      tool: 'scheduledTasks',
      action: `${actionLabel}计划任务`,
      inputPreview: `${task.task_name} (${task.task_path})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ task_name: task.task_name, task_path: task.task_path, action }),
      outputFull: JSON.stringify(result),
    })

    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }

    // 删除成功后从内存移除，其他操作 300ms 后刷新列表
    if (action === 'delete' && result.success) {
      tasks.value = tasks.value.filter(t => `${t.task_path}|${t.task_name}` !== taskKey)
    } else {
      await new Promise(r => setTimeout(r, 300))
      await fetchTasks()
    }
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    operatingTasks.value.delete(taskKey)
  }
}

// ============ CSV 导出 ============

const exportCsv = async () => {
  const BOM = '\uFEFF'
  const header = '任务名,路径,状态,触发器,上次运行,上次结果,下次运行,作者,运行账户,描述'
  const rows = filteredTasks.value.map(t =>
    `"${t.task_name}","${t.task_path}","${t.state}","${t.trigger_brief}","${t.last_run_time}","${t.last_task_result}","${t.next_run_time}","${t.author}","${t.principal}","${t.description.replace(/"/g, '""')}"`
  )
  const csv = BOM + header + '\n' + rows.join('\n')

  const now = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  const filename = `计划任务_${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.csv`

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
    refreshTimer = setInterval(fetchTasks, refreshInterval.value * 1000)
  } else if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

const restartAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(fetchTasks, refreshInterval.value * 1000)
  }
}

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

onMounted(() => {
  fetchTasks()
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

.task-name-cell { display: flex; flex-direction: column; }
.task-name { font-weight: 500; }
.task-path-hint { font-size: 11px; color: var(--text-secondary); margin-top: 2px; }

.last-run-cell { display: flex; align-items: center; gap: 6px; }
.result-ok { color: var(--accent-green); font-weight: 600; }
.result-fail { color: var(--accent-red); font-weight: 600; font-size: 11px; }

.expand-detail { padding: 8px 16px; background: var(--bg-input); border-radius: 4px; }
.detail-row { margin-bottom: 8px; font-size: 13px; line-height: 1.6; }
.detail-label { color: var(--accent-cyan); font-weight: 500; }
.detail-json {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 8px;
  margin-top: 4px;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  max-height: 200px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
}

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
</style>