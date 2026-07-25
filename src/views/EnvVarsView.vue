<template>
  <div class="tool-container">
    <!-- 管理员权限提示 -->
    <div v-if="activeTab === 'system'" class="admin-banner">
      <span class="admin-icon">🛡️</span>
      系统级环境变量的修改需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- 统计概览 -->
    <div v-if="!error && (data.user.length || data.system.length)" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ data.user.length }}</span>
        <span class="stat-label">用户变量</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ data.system.length }}</span>
        <span class="stat-label">系统变量</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">环境变量</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索变量名..." style="width: 220px" clearable />
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="fetchVars">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !data.user.length && !data.system.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」获取环境变量列表" />
      </div>
    </div>

    <!-- Tab: 用户变量 / 系统变量 -->
    <div v-if="data.user.length || data.system.length" class="tool-card">
      <div class="card-header">
        <div class="env-tabs">
          <span
            :class="['tab-item', { active: activeTab === 'user' }]"
            @click="activeTab = 'user'"
          >用户变量 ({{ data.user.length }})</span>
          <span
            :class="['tab-item', { active: activeTab === 'system' }]"
            @click="activeTab = 'system'"
          >系统变量 ({{ data.system.length }})</span>
        </div>
        <el-button type="success" size="small" @click="showAddDialog = true">新增变量</el-button>
      </div>
      <div class="card-body">
        <el-table
          :data="filteredVars" border size="small" max-height="600"
          style="width: 100%" v-loading="loading" row-key="rowKey"
          :row-class-name="tableRowClassName">
          <el-table-column label="变量名" width="180" sortable :sort-method="(a:any,b:any) => a.name.localeCompare(b.name)">
            <template #default="{ row }">
              <template v-if="editingRow === row.rowKey && editingField === 'name'">
                <el-input
                  v-model="editName" size="small"
                  @blur="saveEdit(row)"
                  @keyup.enter="saveEdit(row)"
                  @keyup.escape="cancelEdit"
                  ref="editInputRef"
                />
              </template>
              <template v-else>
                <span
                  class="var-name"
                  :class="{ 'is-path': row.name === 'Path' || row.name === 'PATH' }"
                  title="双击编辑变量名"
                  @dblclick="startEdit(row, 'name')"
                >{{ row.name }}</span>
              </template>
            </template>
          </el-table-column>
          <el-table-column label="值" min-width="300">
            <template #default="{ row }">
              <!-- PATH 变量：展开逐行编辑器 -->
              <template v-if="row.name === 'Path' || row.name === 'PATH'">
                <div v-if="pathExpanded === row.rowKey" class="path-editor">
                  <div v-for="(_p, pi) in pathEntries" :key="pi" class="path-row">
                    <span class="path-index">{{ pi + 1 }}</span>
                    <el-input v-model="pathEntries[pi]" size="small" class="path-input" />
                    <el-button size="small" :icon="Top" circle @click="movePathEntry(pi, -1)" :disabled="pi === 0" />
                    <el-button size="small" :icon="Bottom" circle @click="movePathEntry(pi, 1)" :disabled="pi === pathEntries.length - 1" />
                    <el-button size="small" :icon="Delete" circle type="danger" @click="removePathEntry(pi)" :disabled="pathEntries.length <= 1" />
                  </div>
                  <div class="path-actions">
                    <el-button size="small" @click="addPathEntry">+ 添加目录</el-button>
                    <el-button size="small" type="primary" :loading="pathSaving" @click="savePath(row)">保存 PATH</el-button>
                    <el-button size="small" @click="pathExpanded = null">取消</el-button>
                  </div>
                </div>
                <span v-else class="path-preview" title="双击展开逐行编辑 PATH" @dblclick="expandPath(row)">
                  {{ row.value.split(';').join(' ; ') }}
                </span>
              </template>
              <!-- 普通变量：行内编辑 -->
              <template v-else>
                <template v-if="editingRow === row.rowKey && editingField === 'value'">
                  <el-input
                    v-model="editValue" size="small"
                    @blur="saveEdit(row)"
                    @keyup.enter="saveEdit(row)"
                    @keyup.escape="cancelEdit"
                    ref="editInputRef"
                  />
                </template>
                <template v-else>
                  <span class="var-value" title="双击编辑变量值" @dblclick="startEdit(row, 'value')">{{ row.value }}</span>
                </template>
              </template>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80" fixed="right" align="center">
            <template #default="{ row }">
              <el-button
                v-if="row.name !== 'Path' && row.name !== 'PATH'"
                type="danger" size="small" plain
                :loading="deletingRow === row.rowKey"
                @click="handleDelete(row)"
              >删除</el-button>
              <span v-else class="path-protected" title="PATH 为系统关键变量，不可删除">受保护</span>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 底部栏 -->
    <div v-if="data.user.length || data.system.length" class="tool-card">
      <div class="card-body">
        <div class="bottom-bar">
          <span class="group-label">💡 鼠标悬停查看提示，双击变量名/值编辑，PATH 变量双击展开逐行编辑</span>
          <el-button size="small" @click="exportCsv">导出 CSV</el-button>
        </div>
      </div>
    </div>

    <!-- 新增变量弹窗 -->
    <el-dialog v-model="showAddDialog" title="新增环境变量" width="480px" :close-on-click-modal="false" @closed="addForm = { name: '', value: '', scope: 'user' }">
      <el-form :model="addForm" label-width="80px" label-position="left">
        <el-form-item label="变量名">
          <el-input v-model="addForm.name" placeholder="如 MY_VAR" />
        </el-form-item>
        <el-form-item label="变量值">
          <el-input v-model="addForm.value" placeholder="变量值" />
        </el-form-item>
        <el-form-item label="作用域">
          <el-radio-group v-model="addForm.scope">
            <el-radio value="user">用户变量</el-radio>
            <el-radio value="system">系统变量</el-radio>
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
import { ref, computed, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { Top, Bottom, Delete } from '@element-plus/icons-vue'
import {
  getEnvVars, setEnvVar, deleteEnvVar, formatTimestamp,
  type EnvVarInfo,
} from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

const data = ref<{ user: EnvVarInfo[]; system: EnvVarInfo[] }>({ user: [], system: [] })
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const activeTab = ref('user')

// 行内编辑状态
const editingRow = ref<string | null>(null)
const editingField = ref<'name' | 'value' | null>(null)
const editName = ref('')
const editValue = ref('')
const editInputRef = ref()

// PATH 逐行编辑器状态
const pathExpanded = ref<string | null>(null)
const pathEntries = ref<string[]>([])
const pathSaving = ref(false)

// 删除状态
const deletingRow = ref<string | null>(null)

// 新增弹窗
const showAddDialog = ref(false)
const adding = ref(false)
const addForm = ref({ name: '', value: '', scope: 'user' })

// ============ 计算属性 ============

const currentVars = computed(() => activeTab.value === 'system' ? data.value.system : data.value.user)

const varsWithKey = computed(() =>
  currentVars.value.map((v, i) => ({ ...v, rowKey: `${activeTab.value}|${v.name}|${i}` }))
)

const filteredVars = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return varsWithKey.value
  return varsWithKey.value.filter(v => v.name.toLowerCase().includes(q))
})

const tableRowClassName = ({ row }: { row: any }) => {
  if (row.name === 'Path' || row.name === 'PATH') return 'path-row-highlight'
  return ''
}

// ============ 数据采集 ============

const fetchVars = async () => {
  loading.value = true
  error.value = ''
  try {
    data.value = await getEnvVars()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'envVars',
      action: '查看环境变量',
      inputPreview: '',
      outputPreview: `用户 ${data.value.user.length} 个, 系统 ${data.value.system.length} 个`,
      inputFull: '',
      outputFull: JSON.stringify(data.value),
    })
  } catch (e) {
    error.value = '无法获取环境变量: ' + String(e)
  } finally {
    loading.value = false
  }
}

// ============ 行内编辑 ============

const startEdit = (row: any, field: 'name' | 'value') => {
  editingRow.value = row.rowKey
  editingField.value = field
  editName.value = row.name
  editValue.value = row.value
  nextTick(() => {
    editInputRef.value?.focus?.()
  })
}

const cancelEdit = () => {
  editingRow.value = null
  editingField.value = null
}

const saveEdit = async (row: any) => {
  if (!editingRow.value) return

  const newName = editName.value.trim()
  const newValue = editValue.value
  const scope = activeTab.value

  // 如果名称变了，需要先删除旧变量再创建新变量
  if (editingField.value === 'name' && newName !== row.name) {
    if (currentVars.value.some(v => v.name === newName && v.name !== row.name)) {
      ElMessage.warning('变量名已存在')
      editingRow.value = null
      editingField.value = null
      return
    }
    const delResult = await deleteEnvVar(row.name, scope)
    if (!delResult.success) {
      ElMessage.error(delResult.message)
      editingRow.value = null
      editingField.value = null
      return
    }
    const result = await setEnvVar(newName, newValue, scope)
    editingRow.value = null
    editingField.value = null
    if (result.success) {
      ElMessage.success('已保存')
      store.addHistory({
        tool: 'envVars',
        action: '重命名变量',
        inputPreview: `${row.name} → ${newName} (${scope})`,
        outputPreview: result.message,
        inputFull: JSON.stringify({ oldName: row.name, newName, value: newValue, scope }),
        outputFull: JSON.stringify(result),
      })
      await fetchVars()
    } else {
      ElMessage.error(result.message)
    }
    return
  }

  // 值变了，直接更新
  const result = await setEnvVar(row.name, newValue, scope)
  editingRow.value = null
  editingField.value = null
  if (result.success) {
    ElMessage.success('已保存')
    store.addHistory({
      tool: 'envVars',
      action: '修改变量',
      inputPreview: `${row.name} (${scope})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name: row.name, value: newValue, scope }),
      outputFull: JSON.stringify(result),
    })
    await fetchVars()
  } else {
    ElMessage.error(result.message)
  }
}

// ============ PATH 逐行编辑 ============

const expandPath = (row: any) => {
  pathExpanded.value = row.rowKey
  pathEntries.value = row.value.split(';').filter((p: string) => p.trim())
}

const addPathEntry = () => {
  pathEntries.value.push('')
}

const removePathEntry = (index: number) => {
  pathEntries.value.splice(index, 1)
}

const movePathEntry = (index: number, direction: number) => {
  const newIndex = index + direction
  if (newIndex < 0 || newIndex >= pathEntries.value.length) return
  const temp = pathEntries.value[index]
  pathEntries.value[index] = pathEntries.value[newIndex]
  pathEntries.value[newIndex] = temp
}

const savePath = async (row: any) => {
  const newValue = pathEntries.value.join(';')
  const scope = activeTab.value
  pathSaving.value = true
  try {
    const result = await setEnvVar(row.name, newValue, scope)
    if (result.success) {
      ElMessage.success('PATH 已保存')
      pathExpanded.value = null
      store.addHistory({
        tool: 'envVars',
        action: '编辑 PATH',
        inputPreview: `${row.name} (${scope})`,
        outputPreview: result.message,
        inputFull: JSON.stringify({ name: row.name, value: newValue, scope }),
        outputFull: JSON.stringify(result),
      })
      await fetchVars()
    } else {
      ElMessage.error(result.message)
    }
  } finally {
    pathSaving.value = false
  }
}

// ============ 删除 ============

const handleDelete = async (row: any) => {
  const ok = await confirm.ask(
    '删除环境变量',
    `确定删除${activeTab.value === 'system' ? '系统' : '用户'}变量 "${row.name}"？\n此操作不可恢复。`,
    { type: 'danger', confirmText: '删除' }
  )
  if (!ok) return

  deletingRow.value = row.rowKey
  try {
    const result = await deleteEnvVar(row.name, activeTab.value)
    store.addHistory({
      tool: 'envVars',
      action: '删除变量',
      inputPreview: `${row.name} (${activeTab.value})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name: row.name, scope: activeTab.value }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success('已删除')
      await fetchVars()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    deletingRow.value = null
  }
}

// ============ 新增 ============

const handleAdd = async () => {
  const name = addForm.value.name.trim()
  const value = addForm.value.value
  if (!name) {
    ElMessage.warning('变量名不能为空')
    return
  }
  const targetList = addForm.value.scope === 'system' ? data.value.system : data.value.user
  if (targetList.some(v => v.name === name)) {
    ElMessage.warning('变量名已存在')
    return
  }
  adding.value = true
  try {
    const result = await setEnvVar(name, value, addForm.value.scope)
    store.addHistory({
      tool: 'envVars',
      action: '新增变量',
      inputPreview: `${name} (${addForm.value.scope})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ name, value, scope: addForm.value.scope }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success('已添加')
      showAddDialog.value = false
      addForm.value = { name: '', value: '', scope: 'user' }
      await fetchVars()
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
  const header = '变量名,值,作用域'
  const scope = activeTab.value === 'system' ? '系统' : '用户'
  const rows = filteredVars.value.map(v => `${v.name},"${v.value.replace(/"/g, '""')}",${scope}`)
  const csv = BOM + header + '\n' + rows.join('\n')
  const blob = new Blob([csv], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `环境变量_${activeTab.value}_${new Date().toISOString().slice(0, 10)}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

onMounted(() => {
  fetchVars()
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

.refresh-time { font-size: 12px; color: var(--text-muted); }

.env-tabs { display: flex; gap: 0; }
.tab-item {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}
.tab-item:hover { color: var(--text-primary); }
.tab-item.active { color: var(--accent-cyan); border-bottom-color: var(--accent-cyan); }

.var-name {
  font-weight: 500;
  cursor: pointer;
  position: relative;
}
.var-name:hover {
  color: var(--accent-cyan);
}
.var-name:hover::after {
  content: ' ✎';
  font-size: 11px;
  color: var(--text-muted);
}
.var-name.is-path { color: var(--accent-cyan); }
.var-value {
  cursor: pointer;
  position: relative;
}
.var-value:hover {
  color: var(--accent-cyan);
}
.var-value:hover::after {
  content: ' ✎';
  font-size: 11px;
  color: var(--text-muted);
}

.path-protected {
  font-size: 12px;
  color: var(--text-muted);
}

.path-preview {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'Consolas', 'Courier New', monospace;
  cursor: pointer;
  word-break: break-all;
  position: relative;
}
.path-preview:hover {
  color: var(--accent-cyan);
}
.path-preview:hover::after {
  content: ' ✎ 展开编辑';
  font-size: 11px;
  color: var(--text-muted);
}

.path-editor {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 12px;
}
.path-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}
.path-index {
  width: 28px;
  text-align: right;
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'Consolas', 'Courier New', monospace;
  flex-shrink: 0;
}
.path-input { flex: 1; }
.path-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.bottom-bar { display: flex; justify-content: space-between; align-items: center; }

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

:deep(.path-row-highlight) { background: rgba(0, 212, 255, 0.05) !important; }

:deep(.el-dialog) { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; }
:deep(.el-dialog__title) { color: var(--accent-cyan); font-weight: 600; }
:deep(.el-dialog__body) { padding: 20px 24px; }
</style>