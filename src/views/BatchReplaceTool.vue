<template>
  <div class="tool-container">
    <!-- 1. 扫描配置卡片（sticky） -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">批量内容替换</span>
        <div class="card-actions">
          <el-button size="small" @click="resetAll">重置</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group action-group-flex-2">
            <div class="group-label">搜索路径</div>
            <el-input v-model="searchPath" placeholder="选择或输入要搜索的目录路径" size="small" clearable>
              <template #append>
                <el-button size="small" @click="selectFolder">浏览</el-button>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">扩展名</div>
            <el-input v-model="extFilterText" placeholder="ts,js 或 !exe,dll" size="small" style="width: 160px" />
          </div>
        </div>
        <div class="action-grid action-grid-mt-sm">
          <div class="action-group action-group-flex-2">
            <div class="group-label">查找内容</div>
            <el-input
              v-model="searchText"
              placeholder="要查找的文本或正则表达式"
              size="small"
              clearable
              @keyup.enter="startScan"
            />
          </div>
          <div class="action-group action-group-flex-2">
            <div class="group-label">替换为</div>
            <el-input
              v-model="replacementText"
              placeholder="替换后的内容（正则模式支持 $1 引用分组）"
              size="small"
              clearable
              @keyup.enter="startScan"
            />
          </div>
        </div>
        <div class="action-grid action-grid-mt-sm">
          <el-checkbox v-model="useRegex">正则模式</el-checkbox>
          <el-checkbox v-model="caseSensitive">区分大小写</el-checkbox>
          <el-checkbox v-model="includeHidden">包含隐藏</el-checkbox>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!searchPath || !searchText"
                :loading="scanning"
                @click="startScan"
              >
                扫描
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 2. 错误卡片 -->
    <div v-if="scanError" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ scanError }}</div>
      </div>
    </div>

    <!-- 3. 结果卡片 -->
    <div v-if="scanned" class="tool-card">
      <div class="card-header">
        <span class="card-title">命中文件</span>
        <div class="card-actions">
          <span class="summary-text">
            命中 {{ totalMatches }} 处 / {{ resultItems.length }} 文件
            <span v-if="truncated" class="warn-text">(已达上限 1000，结果截断)</span>
          </span>
          <el-button
            type="danger"
            size="small"
            :disabled="!selectedPaths.length"
            :loading="replacing"
            @click="confirmReplace"
          >
            替换选中 ({{ selectedPaths.length }})
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table
          ref="tableRef"
          :data="resultItems"
          stripe
          size="small"
          max-height="520"
          @selection-change="onSelectionChange"
        >
          <el-table-column type="selection" width="40" />
          <el-table-column label="文件" min-width="280">
            <template #default="{ row }">
              <div class="file-name"><strong>{{ row.name }}</strong></div>
              <div class="file-path">{{ row.path }}</div>
            </template>
          </el-table-column>
          <el-table-column label="命中数" width="80" align="center">
            <template #default="{ row }">{{ row.match_count }}</template>
          </el-table-column>
          <el-table-column label="替换结果" min-width="220">
            <template #default="{ row }">
              <template v-if="row.replaceResult">
                <span v-if="row.replaceResult.success" class="result-ok">
                  替换 {{ row.replaceResult.match_count }} 处
                </span>
                <span v-else class="result-fail" :title="row.replaceResult.error || ''">
                  失败：{{ (row.replaceResult.error || '').slice(0, 40) }}
                </span>
              </template>
              <span v-else class="result-pending">—</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="90">
            <template #default="{ row }">
              <el-button size="small" link @click="locateFile(row)">定位</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 4. 备份说明卡片 -->
    <div v-if="lastBackupDir" class="tool-card">
      <div class="card-header">
        <span class="card-title">备份</span>
      </div>
      <div class="card-body">
        <div class="backup-info">
          本次替换前原文件已自动备份到：
          <el-link type="primary" @click="locateBackup">{{ lastBackupDir }}</el-link>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useToolboxStore } from '@/store'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import {
  fileSearchStart,
  fileSearchGetResults,
  fileSearchCancel,
  fileSearchClear,
} from '@/utils/fileSearcherClient'
import type { SearchOptions, SearchResultItem } from '@/utils/fileSearcherTypes'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

// ============ 状态 ============
const searchPath = ref('')
const searchText = ref('')
const replacementText = ref('')
const extFilterText = ref('')
const useRegex = ref(true)
const caseSensitive = ref(false)
const includeHidden = ref(false)

const scanning = ref(false)
const scanned = ref(false)
const scanError = ref('')
const searchId = ref('')
const resultItems = ref<Array<SearchResultItem & { replaceResult?: ReplaceResult | null }>>([])
const totalMatches = ref(0)
const truncated = ref(false)
const selectedPaths = ref<string[]>([])
const replacing = ref(false)
const lastBackupDir = ref('')
const tableRef = ref()

interface ReplaceResult {
  path: string
  success: boolean
  match_count: number
  backup_path: string | null
  error: string | null
}

// ============ 工具函数 ============
function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// ============ 目录选择 ============
async function selectFolder() {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    searchPath.value = selected as string
  }
}

// ============ 扫描 ============
async function startScan() {
  if (!searchPath.value || !searchText.value) return
  scanError.value = ''
  scanned.value = false
  resultItems.value = []
  totalMatches.value = 0
  truncated.value = false
  selectedPaths.value = []
  lastBackupDir.value = ''

  // 扩展名过滤解析（与 FileSearcher 一致）
  const tokens = extFilterText.value
    .split(',')
    .map((s) => s.trim().replace(/^\.+/, '').toLowerCase())
    .filter((s) => s.length > 0)
  const opts: SearchOptions = {
    mode: 'content',
    query: useRegex.value ? searchText.value : escapeRegex(searchText.value),
    caseSensitive: caseSensitive.value,
    extensions: [],
    excludeExtensions: [],
    includeHidden: includeHidden.value,
    maxContentFileBytes: 10 * 1024 * 1024,
  }
  if (tokens.length > 0) {
    if (tokens.some((t) => t.startsWith('!'))) {
      opts.excludeExtensions = tokens.map((t) => t.replace(/^!/, ''))
    } else {
      opts.extensions = tokens
    }
  }

  try {
    scanning.value = true
    const id = await fileSearchStart(searchPath.value, opts)
    searchId.value = id
    // 等待完成后一次性拉取全部结果（后端最多 1000 条）
    await pollSearchDone(id)
    const page = await fileSearchGetResults(id, 10000, 0)
    resultItems.value = page.items.map((it) => ({ ...it, replaceResult: null }))
    totalMatches.value = resultItems.value.reduce((sum, it) => sum + it.matchCount, 0)
    // 截断标记：直接调用后端 summary 判断
    const status = await invoke<{ status: string; truncated?: boolean }>('file_search_status', {
      searchId: id,
    })
    if (status.status === 'failed') {
      scanError.value = '扫描失败'
      scanned.value = false
    } else {
      scanned.value = true
      const summary = await invoke<any>('file_search_get_summary', { searchId: id })
      truncated.value = !!summary?.truncated
      if (resultItems.value.length === 0) {
        ElMessage.info('未找到匹配的文件')
      }
    }
  } catch (e: any) {
    scanError.value = String(e)
  } finally {
    scanning.value = false
  }
}

function pollSearchDone(id: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setInterval(async () => {
      try {
        const status = await invoke<{ status: string; error?: string }>('file_search_status', {
          searchId: id,
        })
        if (status.status === 'running') return
        clearInterval(timer)
        if (status.status === 'failed') {
          reject(new Error(status.error || '扫描失败'))
        } else {
          resolve()
        }
      } catch (e: any) {
        clearInterval(timer)
        reject(e)
      }
    }, 300)
  })
}

// ============ 替换 ============
function onSelectionChange(rows: SearchResultItem[]) {
  selectedPaths.value = rows.map((r) => r.path)
}

async function confirmReplace() {
  if (!selectedPaths.value.length) return
  const ok = await confirm.ask(
    '替换确认',
    `确定替换选中的 ${selectedPaths.value.length} 个文件中的「${searchText.value}」吗？\n替换前会自动备份原文件，可在执行后查看备份路径。`,
    { type: 'danger', confirmText: '执行替换' }
  )
  if (!ok) return

  // 生成备份目录：搜索根目录下 .litobox_backup/{时间戳}
  const ts = new Date()
  const tsStr = [
    ts.getFullYear(),
    String(ts.getMonth() + 1).padStart(2, '0'),
    String(ts.getDate()).padStart(2, '0'),
    '_',
    String(ts.getHours()).padStart(2, '0'),
    String(ts.getMinutes()).padStart(2, '0'),
    String(ts.getSeconds()).padStart(2, '0'),
  ].join('')
  const backupDir = `${searchPath.value.replace(/[\\/]+$/, '')}/.litobox_backup/${tsStr}`

  try {
    replacing.value = true
    const results = await invoke<ReplaceResult[]>('batch_replace_execute', {
      paths: selectedPaths.value,
      root_dir: searchPath.value,
      search: searchText.value,
      replacement: replacementText.value,
      use_regex: useRegex.value,
      case_sensitive: caseSensitive.value,
      backup_dir: backupDir,
    })
    // 关联结果到表格行
    const resultMap = new Map(results.map((r) => [r.path, r]))
    for (const item of resultItems.value) {
      item.replaceResult = resultMap.get(item.path) ?? null
    }
    const successCount = results.filter((r) => r.success && r.match_count > 0).length
    const failCount = results.filter((r) => !r.success).length
    lastBackupDir.value = results.find((r) => r.backup_path)?.backup_path?.replace(/[\\/][^\\/]+$/, '') || ''

    if (failCount > 0) {
      ElMessage.warning(`替换完成：成功 ${successCount} 个文件，失败 ${failCount} 个`)
    } else {
      ElMessage.success(`替换完成：共替换 ${successCount} 个文件`)
    }
    recordHistory(successCount, failCount)
  } catch (e: any) {
    ElMessage.error('替换失败: ' + String(e))
  } finally {
    replacing.value = false
  }
}

function recordHistory(successCount: number, failCount: number) {
  store.addHistory({
    tool: 'batchReplace',
    action: `批量替换（${useRegex.value ? '正则' : '文本'}）`,
    inputPreview: `${searchPath.value} | ${searchText.value}`.slice(0, 50),
    outputPreview: `成功 ${successCount} 文件 / 失败 ${failCount}`.slice(0, 50),
    inputFull: `路径: ${searchPath.value}\n查找: ${searchText.value}\n替换: ${replacementText.value}\n正则: ${useRegex.value ? '是' : '否'}`,
    outputFull: `成功替换 ${successCount} 个文件，失败 ${failCount} 个\n备份目录: ${lastBackupDir.value || '无'}`,
    options: {
      searchPath: searchPath.value,
      searchText: searchText.value,
      replacementText: replacementText.value,
      useRegex: useRegex.value,
      caseSensitive: caseSensitive.value,
    },
  })
}

// ============ 定位 ============
async function locateFile(row: SearchResultItem) {
  try {
    await invoke('disk_locate_in_explorer', { path: row.path })
  } catch (e: any) {
    ElMessage.error('定位失败: ' + String(e))
  }
}

async function locateBackup() {
  if (!lastBackupDir.value) return
  try {
    await invoke('disk_locate_in_explorer', { path: lastBackupDir.value })
  } catch (e: any) {
    ElMessage.error('定位失败: ' + String(e))
  }
}

// ============ 重置 ============
async function resetAll() {
  if (searchId.value) {
    fileSearchCancel(searchId.value).catch(() => {})
    fileSearchClear(searchId.value).catch(() => {})
  }
  searchPath.value = ''
  searchText.value = ''
  replacementText.value = ''
  extFilterText.value = ''
  useRegex.value = true
  caseSensitive.value = false
  includeHidden.value = false
  scanning.value = false
  scanned.value = false
  scanError.value = ''
  searchId.value = ''
  resultItems.value = []
  selectedPaths.value = []
  lastBackupDir.value = ''
  ElMessage.success('已重置')
}
</script>

<style scoped>
.action-group-flex-2 {
  flex: 2;
}
.action-grid-mt-sm {
  margin-top: 8px;
}
.summary-text {
  font-size: 12px;
  color: var(--text-secondary);
  margin-right: 8px;
}
.warn-text {
  color: var(--accent-orange);
  margin-left: 6px;
}
.file-name {
  font-size: 13px;
}
.file-path {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
  word-break: break-all;
}
.result-ok {
  color: var(--accent-green, #10b981);
  font-size: 12px;
}
.result-fail {
  color: var(--accent-red);
  font-size: 12px;
}
.result-pending {
  color: var(--text-secondary);
  font-size: 12px;
}
.error-message {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}
.backup-info {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.6;
}
</style>
