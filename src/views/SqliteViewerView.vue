<template>
  <div class="tool-container sqlite-viewer">
    <!-- 文件选择栏 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">数据库文件</span>
        <div class="card-actions">
          <el-button size="small" @click="handleSelectFile">选择 .db 文件</el-button>
          <el-button v-if="dbPath" size="small" @click="handleRefresh">刷新</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="dbPath" class="file-path">{{ dbPath }}</div>
        <div v-else class="empty-hint">请选择一个 SQLite 数据库文件</div>
      </div>
    </div>

    <div v-if="dbPath" class="viewer-body">
      <!-- 左侧：表列表 -->
      <div class="tool-card table-list-card">
        <div class="card-header">
          <span class="card-title">表 ({{ tables.length }})</span>
        </div>
        <div class="card-body table-list-body">
          <div
            v-for="table in tables"
            :key="table.name"
            class="table-item"
            :class="{ active: selectedTable === table.name }"
            @click="handleSelectTable(table.name)"
          >
            <span class="table-name">{{ table.name }}</span>
            <span class="table-rows">{{ table.row_count }}</span>
          </div>
          <div v-if="tables.length === 0" class="empty-hint">无表</div>
        </div>
      </div>

      <!-- 右侧：SQL 编辑器 + 结果 -->
      <div class="viewer-main">
        <!-- SQL 编辑器 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">SQL 查询</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button type="primary" size="small" @click="handleExecuteQuery">执行</el-button>
              <el-button size="small" @click="handleClearSql">清空</el-button>
              <el-button size="small" @click="handleExportCsv" :disabled="!lastResult">导出CSV</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="sqlText"
              type="textarea"
              :rows="5"
              placeholder="SELECT * FROM table_name LIMIT 100"
              resize="vertical"
              @keydown.ctrl.enter="handleExecuteQuery"
            />
          </div>
        </div>

        <!-- 结果表格 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">查询结果</span>
            <div class="result-meta" v-if="lastResult">
              耗时: {{ lastResult.execution_ms }}ms | 返回: {{ lastResult.rows.length }}行
              <span v-if="lastResult.rows.length >= 1000" class="truncated-hint">（已截断）</span>
            </div>
          </div>
          <div class="card-body result-body">
            <div v-if="queryError" class="error-message">{{ queryError }}</div>
            <el-table
              v-else-if="lastResult && lastResult.rows.length > 0"
              :data="tableData"
              border
              stripe
              size="small"
              height="100%"
            >
              <el-table-column
                v-for="(col, idx) in lastResult.columns"
                :key="idx"
                :prop="String(idx)"
                :label="col"
                min-width="120"
                show-overflow-tooltip
              />
            </el-table>
            <div v-else-if="lastResult" class="empty-hint">查询结果为空</div>
            <div v-else class="empty-hint">执行查询后在此显示结果</div>
          </div>
        </div>

        <!-- 表结构面板 -->
        <div v-if="schema.length > 0" class="tool-card">
          <div class="card-header">
            <span class="card-title">表结构: {{ selectedTable }}</span>
          </div>
          <div class="card-body">
            <el-table :data="schema" border stripe size="small">
              <el-table-column prop="name" label="字段名" min-width="120" />
              <el-table-column prop="data_type" label="类型" width="120" />
              <el-table-column label="主键" width="60" align="center">
                <template #default="{ row }">
                  <span v-if="row.is_primary_key">是</span>
                </template>
              </el-table-column>
              <el-table-column label="非空" width="60" align="center">
                <template #default="{ row }">
                  <span v-if="row.not_null">是</span>
                </template>
              </el-table-column>
              <el-table-column prop="default_value" label="默认值" min-width="100" />
            </el-table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElLoading } from 'element-plus'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  sqliteListTables,
  sqliteGetSchema,
  sqliteQuery,
  sqliteTablePreview,
  sqliteExportCsv,
  type TableInfo,
  type ColumnInfo,
  type QueryResult,
} from '@/utils/sqliteClient'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const dbPath = ref('')
const tables = ref<TableInfo[]>([])
const selectedTable = ref('')
const sqlText = ref('')
const lastResult = ref<QueryResult | null>(null)
const queryError = ref('')
const schema = ref<ColumnInfo[]>([])

// 将结果行转为 el-table 可用的对象数组
const tableData = computed(() => {
  if (!lastResult.value) return []
  return lastResult.value.rows.map((row) => {
    const obj: Record<string, unknown> = {}
    row.forEach((val, idx) => {
      obj[String(idx)] = val
    })
    return obj
  })
})

// 历史记录还原：从历史记录页双击跳转时恢复 SQL
onMounted(() => {
  const restore = store.pendingHistoryRestore
  if (restore && restore.tool === 'sqliteViewer') {
    sqlText.value = restore.input
    store.clearHistoryRestore()
  }
})

const handleSelectFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'SQLite 数据库', extensions: ['db', 'sqlite', 'sqlite3'] }],
  })
  if (typeof selected !== 'string') return

  dbPath.value = selected
  await loadTables()
}

const loadTables = async () => {
  const loading = ElLoading.service({ text: '加载表列表...' })
  try {
    tables.value = await sqliteListTables(dbPath.value)
    selectedTable.value = ''
    schema.value = []
    lastResult.value = null
    queryError.value = ''
    ElMessage.success(`已加载 ${tables.value.length} 个表`)
  } catch (e) {
    ElMessage.error(String(e))
    tables.value = []
  } finally {
    loading.close()
  }
}

const handleSelectTable = async (tableName: string) => {
  selectedTable.value = tableName
  sqlText.value = `SELECT * FROM "${tableName.replace(/"/g, '""')}" LIMIT 100`
  queryError.value = ''

  const loading = ElLoading.service({ text: '加载数据...' })
  try {
    // 并行加载预览数据和表结构
    const [preview, schemaResult] = await Promise.all([
      sqliteTablePreview(dbPath.value, tableName),
      sqliteGetSchema(dbPath.value, tableName),
    ])
    lastResult.value = preview
    schema.value = schemaResult
  } catch (e) {
    lastResult.value = null
    queryError.value = String(e)
    schema.value = []
  } finally {
    loading.close()
  }
}

const handleExecuteQuery = async () => {
  if (!sqlText.value.trim()) {
    ElMessage.warning('请输入 SQL 语句')
    return
  }
  const loading = ElLoading.service({ text: '执行查询...' })
  try {
    lastResult.value = await sqliteQuery(dbPath.value, sqlText.value)
    queryError.value = ''
    store.addHistory({
      tool: 'sqliteViewer',
      action: '执行查询',
      inputPreview: sqlText.value.slice(0, 50),
      outputPreview: `${lastResult.value.rows.length}行结果`,
      inputFull: sqlText.value,
      outputFull: JSON.stringify(lastResult.value.rows.slice(0, 50)),
    })
    ElMessage.success(`查询完成，返回 ${lastResult.value.rows.length} 行`)
  } catch (e) {
    lastResult.value = null
    queryError.value = String(e)
    ElMessage.error('查询失败')
  } finally {
    loading.close()
  }
}

const handleInsertVariable = (value: string) => {
  sqlText.value = value
}

const handleClearSql = () => {
  sqlText.value = ''
  lastResult.value = null
  queryError.value = ''
}

const handleExportCsv = async () => {
  if (!lastResult.value || !sqlText.value.trim()) return
  const savePath = await save({
    filters: [{ name: 'CSV 文件', extensions: ['csv'] }],
    defaultPath: 'query_result.csv',
  })
  if (!savePath) return

  const loading = ElLoading.service({ text: '导出中...' })
  try {
    const count = await sqliteExportCsv(dbPath.value, sqlText.value, savePath)
    ElMessage.success(`已导出 ${count} 行到 ${savePath}`)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    loading.close()
  }
}

const handleRefresh = async () => {
  await loadTables()
}
</script>

<style scoped>
.sqlite-viewer {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  padding: 16px 20px;
}

.file-path {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

.empty-hint {
  color: var(--text-secondary);
  text-align: center;
  padding: 24px;
  font-size: 13px;
}

.viewer-body {
  display: flex;
  gap: 16px;
  min-height: 0;
}

.table-list-card {
  width: 220px;
  flex-shrink: 0;
}

.table-list-body {
  padding: 8px;
  max-height: calc(100vh - 200px);
  overflow-y: auto;
}

.table-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.table-item:hover {
  background: rgba(0, 212, 255, 0.08);
}

.table-item.active {
  background: rgba(0, 212, 255, 0.15);
  color: var(--accent-cyan);
}

.table-name {
  font-weight: 500;
}

.table-rows {
  color: var(--text-secondary);
  font-size: 11px;
}

.viewer-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.result-meta {
  font-size: 12px;
  color: var(--text-secondary);
}

.truncated-hint {
  color: var(--accent-orange, #eab308);
}

.result-body {
  height: 320px;
  overflow: hidden;
}

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}
</style>
