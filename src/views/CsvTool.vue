<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持自定义分隔符（逗号/分号/Tab/竖线）</p>
                <p>• 可识别引号转义和分隔符内换行</p>
                <p>• 导出为 JSON 或 SQL INSERT 语句</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleFileImport">导入文件</el-button>
          <input ref="fileInput" type="file" accept=".csv,.tsv,.txt" style="display: none" @change="handleFileChange" />
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">分隔符</div>
            <el-select v-model="delimiter" size="small" style="width: 100px">
              <el-option label="逗号" value="," />
              <el-option label="分号" value=";" />
              <el-option label="Tab" value="\t" />
              <el-option label="竖线" value="|" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <label class="switch-item">
              <span>首行为表头</span>
              <el-switch v-model="hasHeader" size="small" />
            </label>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <el-button type="primary" size="small" @click="handleParse">解析</el-button>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (CSV文本)</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="8" placeholder="name,age,city&#10;张三,25,北京&#10;李四,30,上海" resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">表格预览</span>
        <div class="card-actions">
          <el-tag v-if="rowCount > 0" size="small" type="info">{{ rowCount }} 行</el-tag>
        </div>
      </div>
      <div class="card-body">
        <el-table v-if="csvData.rows.length > 0" :data="tableData" border size="small" max-height="400" style="width: 100%">
          <el-table-column v-for="header in csvData.headers" :key="header" :prop="header" :label="header" />
        </el-table>
        <div v-else class="empty-tip">解析后将在此显示表格</div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">导出</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">表名</div>
            <el-input v-model="tableName" placeholder="请输入表名" size="small" style="width: 140px" />
          </div>
          <div class="action-group">
            <div class="group-label">导出</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleExportJson">导出 JSON</el-button>
              <el-button size="small" @click="handleExportSql">导出 SQL</el-button>
            </div>
          </div>
        </div>
        <el-input v-if="exportResult" :model-value="exportResult" type="textarea" :rows="6" readonly resize="vertical" style="margin-top: 12px" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { parseCsv, csvToJson, csvToSql, type CsvData } from '@/utils/csvUtils'
import { decodeTextSmart } from '@/utils/textEncoding'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const input = ref('')
const delimiter = ref<',' | ';' | '\t' | '|'>(',')
const hasHeader = ref(true)
const tableName = ref('')
const exportResult = ref('')
const fileInput = ref<HTMLInputElement>()

const csvData = ref<CsvData>({ headers: [], rows: [] })
const rowCount = computed(() => csvData.value.rows.length)

const tableData = computed(() => {
  return csvData.value.rows.map(row => {
    const obj: Record<string, string> = {}
    csvData.value.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
})

const handleParse = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入 CSV 内容')
    return
  }
  try {
    csvData.value = parseCsv(input.value, {
      delimiter: delimiter.value,
      hasHeader: hasHeader.value,
    })
    exportResult.value = ''
    ElMessage.success(`解析完成: ${csvData.value.rows.length} 行, ${csvData.value.headers.length} 列`)
    store.addHistory({
      tool: 'csv',
      action: 'CSV解析',
      inputPreview: input.value.slice(0, 50),
      outputPreview: `${csvData.value.rows.length} 行`,
      inputFull: input.value,
      outputFull: csvData.value.rows.map(r => r.join(',')).join('\n'),
    })
  } catch (e: any) {
    ElMessage.error('解析失败: ' + e.message)
  }
}

const handleExportJson = () => {
  if (csvData.value.rows.length === 0) {
    ElMessage.warning('请先解析 CSV 数据')
    return
  }
  exportResult.value = csvToJson(csvData.value)
  navigator.clipboard.writeText(exportResult.value)
  ElMessage.success('已导出为 JSON 并复制到剪贴板')
}

const handleExportSql = () => {
  if (csvData.value.rows.length === 0) {
    ElMessage.warning('请先解析 CSV 数据')
    return
  }
  exportResult.value = csvToSql(csvData.value, tableName.value)
  if (!exportResult.value.startsWith('错误')) {
    navigator.clipboard.writeText(exportResult.value)
    ElMessage.success('已导出为 SQL 并复制到剪贴板')
  }
}

const handleClear = () => {
  input.value = ''
  csvData.value = { headers: [], rows: [] }
  exportResult.value = ''
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}

const handleFileImport = () => {
  fileInput.value?.click()
}

const handleFileChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  if (file.name.endsWith('.tsv')) {
    delimiter.value = '\t'
  }

  file.arrayBuffer().then((buf) => {
    input.value = decodeTextSmart(new Uint8Array(buf))
    ElMessage.success(`已导入文件: ${file.name}`)
  }).catch(() => {
    ElMessage.error(`读取 ${file.name} 失败`)
  })
  target.value = ''
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

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
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}
</style>
