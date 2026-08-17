<template>
  <div class="tool-container">
    <!-- 顶部操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Excel 处理</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 打开 .xlsx / .xls 文件，多 Sheet 浏览</p>
                <p>• 多个文件按行（纵向）或按列（横向）合并</p>
                <p>• 数据清洗：去空行、去重、修剪空格、空值填充</p>
                <p>• 导出 CSV / JSON / Markdown / SQL，或另存为 Excel</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="handleOpenExcel">打开 Excel</el-button>
          <el-button size="small" @click="handleOpenMergeFiles">添加合并文件</el-button>
          <span v-if="currentFileName" class="file-name-tag">{{ currentFileName }}</span>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" v-if="currentSheets.length > 0">
            <div class="group-label">Sheet</div>
            <el-select v-model="activeSheetIndex" size="small" style="width: 180px">
              <el-option
                v-for="(s, i) in currentSheets"
                :key="s.name + i"
                :label="`${s.name}（${s.rows.length} 行 × ${s.headers.length} 列）`"
                :value="i"
              />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">当前</div>
            <el-tag v-if="activeSheet" size="small" type="info">
              {{ activeSheet.rows.length }} 行 × {{ activeSheet.headers.length }} 列
            </el-tag>
            <el-tag v-else size="small" type="info">未打开文件</el-tag>
          </div>
          <div class="action-group">
            <div class="group-label">数据预览</div>
            <el-switch v-model="previewOnly" size="small" active-text="前100行" inactive-text="全部" />
          </div>
        </div>
      </div>
      <!-- 隐藏文件输入 -->
      <input ref="excelInput" type="file" accept=".xlsx,.xls,.csv,.tsv" style="display: none" @change="handleExcelFile" />
      <input ref="mergeInput" type="file" accept=".xlsx,.xls,.csv,.tsv" multiple style="display: none" @change="handleMergeFiles" />
    </div>

    <el-tabs v-model="activeTab" class="excel-tabs">
      <!-- ============ 表格浏览 ============ -->
      <el-tab-pane label="表格浏览" name="view">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">{{ activeSheet?.name || '预览' }}</span>
            <div class="card-actions">
              <el-button size="small" :disabled="!activeSheet" @click="handleCopySheetTable">复制表格</el-button>
              <el-button size="small" :disabled="!activeSheet" @click="handleSaveAsXlsx">另存为 .xlsx</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-table
              v-if="activeSheet && previewRows.length > 0"
              :data="previewRows"
              border
              size="small"
              max-height="480"
              style="width: 100%"
              :header-cell-style="{ background: 'rgba(0,212,255,0.06)' }"
            >
              <el-table-column v-for="(h, i) in activeSheet.headers" :key="'v' + i" :prop="h" :label="h" min-width="120" show-overflow-tooltip />
            </el-table>
            <div v-else class="empty-tip">打开 Excel 文件后在此预览表格数据</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============ 文件合并 ============ -->
      <el-tab-pane label="文件合并" name="merge">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">合并</span>
            <div class="card-actions">
              <el-button size="small" @click="handleOpenMergeFiles">添加文件</el-button>
              <el-button size="small" type="primary" :disabled="mergeFiles.length < 2" @click="handleMerge">执行合并</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">合并模式</div>
                <el-radio-group v-model="mergeMode" size="small">
                  <el-radio-button value="rows">纵向合并（追加行）</el-radio-button>
                  <el-radio-button value="columns">横向合并（拼接列）</el-radio-button>
                </el-radio-group>
              </div>
            </div>
            <div v-if="mergeFiles.length > 0" class="merge-file-list">
              <div v-for="(f, i) in mergeFiles" :key="f.name + i" class="merge-file-item">
                <span class="merge-file-name">{{ i + 1 }}. {{ f.name }}</span>
                <el-tag size="small" type="info">{{ f.sheets.length }} Sheet</el-tag>
                <el-button size="small" text type="danger" @click="removeMergeFile(i)">移除</el-button>
              </div>
            </div>
            <div v-else class="empty-tip">点击"添加文件"选择多个 Excel/CSV 文件（至少 2 个）</div>

            <div v-if="mergedResult" style="margin-top: 16px">
              <div class="section-title">合并结果 <el-tag size="small" type="success">{{ mergedResult.rows.length }} 行 × {{ mergedResult.headers.length }} 列</el-tag></div>
              <el-table :data="mergedRows" border size="small" max-height="320" style="width: 100%">
                <el-table-column v-for="(h, i) in mergedResult.headers" :key="'m' + i" :prop="h" :label="h" min-width="120" show-overflow-tooltip />
              </el-table>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============ 数据清洗 ============ -->
      <el-tab-pane label="数据清洗" name="clean">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">清洗当前 Sheet</span>
            <div class="card-actions">
              <el-button size="small" :disabled="!activeSheet" type="primary" @click="handleClean">执行清洗</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <label class="switch-item">
                <span>去除整行空白</span>
                <el-switch v-model="cleanOptions.removeEmptyRows" size="small" />
              </label>
              <label class="switch-item">
                <span>按整行去重</span>
                <el-switch v-model="cleanOptions.deduplicate" size="small" />
              </label>
              <label class="switch-item">
                <span>修剪首尾空格</span>
                <el-switch v-model="cleanOptions.trimCells" size="small" />
              </label>
              <label class="switch-item">
                <span>空值填充为 -</span>
                <el-switch v-model="cleanOptions.fillEmpty" size="small" />
              </label>
            </div>
            <div v-if="cleanResult" style="margin-top: 16px">
              <div class="section-title">
                清洗结果
                <el-tag size="small" type="success">{{ cleanResult.rows.length }} 行</el-tag>
                <el-tag v-if="activeSheet" size="small" type="warning">原始 {{ activeSheet.rows.length }} 行</el-tag>
              </div>
              <el-table :data="cleanRows" border size="small" max-height="320" style="width: 100%">
                <el-table-column v-for="(h, i) in cleanResult.headers" :key="'c' + i" :prop="h" :label="h" min-width="120" show-overflow-tooltip />
              </el-table>
            </div>
            <div v-else class="empty-tip">选择清洗选项后点击"执行清洗"，对当前 Sheet 生效</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============ 导出 ============ -->
      <el-tab-pane label="导出" name="export">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">导出</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">数据源</div>
                <el-select v-model="exportSource" size="small" style="width: 160px">
                  <el-option label="当前 Sheet" value="current" :disabled="!activeSheet" />
                  <el-option label="合并结果" value="merged" :disabled="!mergedResult" />
                  <el-option label="清洗结果" value="cleaned" :disabled="!cleanResult" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">格式</div>
                <el-select v-model="exportFormat" size="small" style="width: 140px">
                  <el-option label="CSV" value="csv" />
                  <el-option label="JSON" value="json" />
                  <el-option label="Markdown" value="md" />
                  <el-option label="SQL INSERT" value="sql" />
                </el-select>
              </div>
              <div class="action-group" v-if="exportFormat === 'sql'">
                <div class="group-label">表名</div>
                <el-input v-model="tableName" placeholder="表名" size="small" style="width: 120px" />
              </div>
              <div class="group-buttons">
                <el-button size="small" type="primary" :disabled="!exportSheetData" @click="handleExportCopy">复制结果</el-button>
                <el-button size="small" :disabled="!exportSheetData" @click="handleExportDownload">下载 .xlsx</el-button>
              </div>
            </div>
            <el-input
              v-if="exportResult"
              :model-value="exportResult"
              type="textarea"
              :rows="8"
              readonly
              resize="vertical"
              style="margin-top: 12px; font-family: 'JetBrains Mono', monospace"
            />
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  readExcelFile, parseCsvText, mergeSheets, cleanData, exportSheet,
  sheetToXlsxBlob, toCSV, decodeTextSmart, selfCheck, type SheetData,
} from '@/utils/excelUtils'
import { saveFileWithDialog } from '@/utils/fileSaver'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 浏览状态 ============
const activeTab = ref('view')
const excelInput = ref<HTMLInputElement>()
const mergeInput = ref<HTMLInputElement>()
const currentFileName = ref('')
const currentSheets = ref<SheetData[]>([])
const activeSheetIndex = ref(0)
const previewOnly = ref(true)

const activeSheet = computed<SheetData | null>(() => {
  const s = currentSheets.value[activeSheetIndex.value]
  return s || null
})

const previewRows = computed(() => {
  if (!activeSheet.value) return []
  const rows = previewOnly.value ? activeSheet.value.rows.slice(0, 100) : activeSheet.value.rows
  return rows.map((row, idx) => {
    const obj: Record<string, any> = { __row: idx + 1 }
    activeSheet.value!.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
})

// ============ 合并状态 ============
const mergeMode = ref<'rows' | 'columns'>('rows')
const mergeFiles = ref<Array<{ name: string; sheets: SheetData[] }>>([])
const mergedResult = ref<SheetData | null>(null)

const mergedRows = computed(() => {
  if (!mergedResult.value) return []
  return mergedResult.value.rows.slice(0, 200).map((row, idx) => {
    const obj: Record<string, any> = { __row: idx + 1 }
    mergedResult.value!.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
})

// ============ 清洗状态 ============
const cleanOptions = ref({ removeEmptyRows: true, deduplicate: false, trimCells: true, fillEmpty: false })
const cleanResult = ref<SheetData | null>(null)

const cleanRows = computed(() => {
  if (!cleanResult.value) return []
  return cleanResult.value.rows.slice(0, 200).map((row, idx) => {
    const obj: Record<string, any> = { __row: idx + 1 }
    cleanResult.value!.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
})

// ============ 导出状态 ============
const exportSource = ref<'current' | 'merged' | 'cleaned'>('current')
const exportFormat = ref<'csv' | 'json' | 'md' | 'sql'>('csv')
const tableName = ref('sheet_data')
const exportResult = ref('')

const exportSheetData = computed<SheetData | null>(() => {
  if (exportSource.value === 'merged') return mergedResult.value
  if (exportSource.value === 'cleaned') return cleanResult.value
  return activeSheet.value
})

// ============ 文件读取 ============
async function loadFile(file: File, target: 'browse' | 'merge') {
  try {
    let sheets: SheetData[]
    if (file.name.toLowerCase().endsWith('.csv') || file.name.toLowerCase().endsWith('.tsv')) {
      const bytes = new Uint8Array(await file.arrayBuffer())
      const text = decodeTextSmart(bytes)
      const delim = file.name.toLowerCase().endsWith('.tsv') ? '\t' : ','
      sheets = [parseCsvText(text, delim)]
    } else {
      sheets = await readExcelFile(file)
    }
    if (sheets.length === 0) {
      ElMessage.warning(`文件 ${file.name} 无有效数据`)
      return
    }
    if (target === 'browse') {
      currentSheets.value = sheets
      currentFileName.value = file.name
      activeSheetIndex.value = 0
      mergedResult.value = null
      cleanResult.value = null
      exportResult.value = ''
      activeTab.value = 'view'
      ElMessage.success(`已打开 ${file.name}（${sheets.length} 个 Sheet）`)
    } else {
      mergeFiles.value.push({ name: file.name, sheets })
      ElMessage.success(`已添加 ${file.name} 到合并列表`)
    }
  } catch (e: any) {
    ElMessage.error(`读取 ${file.name} 失败: ${e.message}`)
  }
}

const handleOpenExcel = () => excelInput.value?.click()

const handleExcelFile = (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) loadFile(file, 'browse')
  target.value = ''
}

const handleOpenMergeFiles = () => mergeInput.value?.click()

const handleMergeFiles = (e: Event) => {
  const target = e.target as HTMLInputElement
  const files = Array.from(target.files || [])
  for (const f of files) loadFile(f, 'merge')
  target.value = ''
}

const removeMergeFile = (idx: number) => {
  mergeFiles.value.splice(idx, 1)
  if (mergeFiles.value.length < 2) mergedResult.value = null
}

// ============ 合并 ============
const handleMerge = () => {
  if (mergeFiles.value.length < 2) {
    ElMessage.warning('请至少添加 2 个文件')
    return
  }
  const sources: SheetData[] = []
  for (const f of mergeFiles.value) {
    for (const s of f.sheets) sources.push(s)
  }
  mergedResult.value = mergeSheets(sources, { mode: mergeMode.value })
  exportResult.value = ''
  ElMessage.success(`合并完成: ${mergedResult.value.rows.length} 行 × ${mergedResult.value.headers.length} 列`)
  store.addHistory({
    tool: 'excelTool',
    action: mergeMode.value === 'rows' ? 'Excel 纵向合并' : 'Excel 横向合并',
    inputPreview: mergeFiles.value.map(f => f.name).join('、').slice(0, 50),
    outputPreview: `${mergedResult.value.rows.length} 行 × ${mergedResult.value.headers.length} 列`,
  })
}

// ============ 清洗 ============
const handleClean = () => {
  if (!activeSheet.value) {
    ElMessage.warning('请先打开 Excel 文件')
    return
  }
  cleanResult.value = cleanData(activeSheet.value, { ...cleanOptions.value })
  exportResult.value = ''
  ElMessage.success(`清洗完成: ${activeSheet.value.rows.length} 行 → ${cleanResult.value.rows.length} 行`)
  store.addHistory({
    tool: 'excelTool',
    action: 'Excel 数据清洗',
    inputPreview: `${activeSheet.value.name}（${activeSheet.value.rows.length} 行）`,
    outputPreview: `${cleanResult.value.rows.length} 行`,
  })
}

// ============ 导出 ============
const handleExportCopy = async () => {
  const sheet = exportSheetData.value
  if (!sheet) {
    ElMessage.warning('数据源为空')
    return
  }
  const text = exportSheet(sheet, exportFormat.value, tableName.value)
  if (text.startsWith('错误') || text.includes('-- 表名不能为空')) {
    ElMessage.warning('请输入有效的表名')
    return
  }
  exportResult.value = text
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.success('导出完成')
  }
}

const handleExportDownload = async () => {
  const sheet = exportSheetData.value
  if (!sheet) {
    ElMessage.warning('数据源为空')
    return
  }
  const ext = exportFormat.value === 'md' ? 'md' : exportFormat.value === 'json' ? 'json' : exportFormat.value === 'sql' ? 'sql' : 'csv'
  const text = exportSheet(sheet, exportFormat.value, tableName.value)
  const blob = new Blob(['\ufeff' + text], { type: 'text/plain;charset=utf-8' })
  await saveFileWithDialog(blob, `export.${ext}`, ext)
}

// ============ 其他操作 ============
const handleCopySheetTable = async () => {
  if (!activeSheet.value) return
  const text = toCSV(activeSheet.value, '\t')
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制为 TSV（可直接粘贴到 Excel）')
  } catch {
    ElMessage.warning('复制失败')
  }
}

const handleSaveAsXlsx = async () => {
  if (!activeSheet.value) return
  const blob = sheetToXlsxBlob(activeSheet.value)
  await saveFileWithDialog(blob, 'export.xlsx', 'xlsx')
}

// ============ 自检 ============
const errors = selfCheck()
if (errors.length > 0) {
  console.warn('excelUtils 自检失败:', errors)
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px 28px;
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
.card-actions { display: flex; align-items: center; gap: 8px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.file-name-tag {
  font-size: 12px;
  color: var(--text-secondary);
  background: rgba(0, 212, 255, 0.08);
  border: 1px solid rgba(0, 212, 255, 0.25);
  border-radius: 4px;
  padding: 2px 8px;
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

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
.group-buttons { display: flex; gap: 8px; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.excel-tabs :deep(.el-tabs__header) {
  padding-left: 20px;
}

.excel-tabs :deep(.el-tabs__nav-wrap::after) {
  height: 1px;
  background: var(--border-color);
}

.merge-file-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.merge-file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
}

.merge-file-name {
  flex: 1;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.section-title {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
