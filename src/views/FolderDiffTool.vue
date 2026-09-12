<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">目录对比</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 递归对比左右两个文件夹的文件差异</p>
                <p>• 忽略规则：逗号或换行分隔，支持 * 与 ? 通配（如 node_modules、*.log、dist/）</p>
                <p>• 默认按文件大小判定修改；勾选内容比对后用 sha256 精确比对</p>
                <p>• 纯只读查看，不会修改任何文件</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="path-row">
          <span class="path-label">左侧</span>
          <el-input v-model="leftPath" size="small" placeholder="选择或输入左侧文件夹" :disabled="scanning" />
          <el-button size="small" :disabled="scanning" @click="selectFolder('left')">选择</el-button>
        </div>
        <div class="path-row">
          <span class="path-label">右侧</span>
          <el-input v-model="rightPath" size="small" placeholder="选择或输入右侧文件夹" :disabled="scanning" />
          <el-button size="small" :disabled="scanning" @click="selectFolder('right')">选择</el-button>
          <el-button size="small" text :disabled="scanning || (!leftPath && !rightPath)" @click="swapPaths">互换</el-button>
        </div>

        <div class="action-grid">
          <div class="action-group grow">
            <span class="group-label">忽略规则</span>
            <el-input
              v-model="ignoreText"
              size="small"
              placeholder="node_modules, .git, dist, *.log"
              :disabled="scanning"
            />
          </div>
          <div class="action-group">
            <el-checkbox v-model="compareContent" size="small" :disabled="scanning">内容比对 (sha256)</el-checkbox>
          </div>
          <div class="action-group">
            <el-button type="primary" size="small" :loading="scanning" @click="startDiff">开始对比</el-button>
            <el-button v-if="scanning" size="small" @click="cancelDiff">取消</el-button>
          </div>
        </div>
        <div v-if="scanError" class="error-tip">{{ scanError }}</div>
      </div>
    </div>

    <!-- 进度卡 -->
    <div v-if="scanning" class="tool-card">
      <div class="card-header">
        <span class="card-title">对比中</span>
        <span class="meta-text">已扫描 {{ progressScanned }} 项</span>
      </div>
      <div class="card-body">
        <el-progress :percentage="100" :indeterminate="true" :show-text="false" :stroke-width="8" />
        <div class="progress-path">{{ progressPath || '正在收集文件…' }}</div>
      </div>
    </div>

    <!-- 汇总卡 -->
    <div v-if="summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">对比结果</span>
        <span class="meta-text">耗时 {{ formatDuration(summary.duration_ms) }} · 共 {{ summary.total }} 项</span>
      </div>
      <div class="card-body">
        <div class="summary-tags">
          <el-tag type="success" size="small">新增 {{ summary.added }}</el-tag>
          <el-tag type="danger" size="small">删除 {{ summary.removed }}</el-tag>
          <el-tag type="warning" size="small">修改 {{ summary.modified }}</el-tag>
          <el-tag type="info" size="small">相同 {{ summary.same }}</el-tag>
          <el-tag v-if="summary.skipped" type="info" size="small" effect="plain">跳过 {{ summary.skipped }}</el-tag>
        </div>
      </div>
    </div>

    <!-- 明细卡 -->
    <div v-if="summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">差异明细</span>
        <span class="meta-text">{{ total }} 条</span>
      </div>
      <div class="card-body">
        <div class="filter-row">
          <el-radio-group v-model="filterStatus" size="small" @change="handleFilterChange">
            <el-radio-button label="">全部</el-radio-button>
            <el-radio-button label="added">新增</el-radio-button>
            <el-radio-button label="removed">删除</el-radio-button>
            <el-radio-button label="modified">修改</el-radio-button>
            <el-radio-button label="same">相同</el-radio-button>
          </el-radio-group>
          <el-input
            v-model="keyword"
            size="small"
            class="filter-input"
            placeholder="按路径筛选"
            clearable
            @input="handleKeywordInput"
          />
        </div>

        <DataTable :data="entries" max-height="520" empty-text="没有匹配的差异">
          <el-table-column prop="path" label="路径" min-width="320" show-overflow-tooltip />
          <el-table-column label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="statusTagType(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="左侧大小" width="120">
            <template #default="{ row }">{{ sizeText(row.left_size) }}</template>
          </el-table-column>
          <el-table-column label="右侧大小" width="120">
            <template #default="{ row }">{{ sizeText(row.right_size) }}</template>
          </el-table-column>
        </DataTable>

        <el-pagination
          v-if="total > pageSize"
          class="pager"
          layout="prev, pager, next, total"
          :current-page="page"
          :page-size="pageSize"
          :total="total"
          @current-change="handlePageChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import DataTable from '@/components/DataTable.vue'
import { formatBytes } from '@/utils/systemInfoClient'
import {
  dirDiffStart, dirDiffCancel, dirDiffStatus, dirDiffGetSummary, dirDiffGetEntries, dirDiffClear,
  type DiffEntry, type DiffSummary,
} from '@/utils/dirDiffClient'

const store = useToolboxStore()

const leftPath = ref(localStorage.getItem('folderDiff.lastLeft') || '')
const rightPath = ref(localStorage.getItem('folderDiff.lastRight') || '')
const ignoreText = ref(localStorage.getItem('folderDiff.ignore') || '')
const compareContent = ref(localStorage.getItem('folderDiff.compareContent') === '1')

const scanning = ref(false)
const scanError = ref('')
const scanId = ref('')
const progressScanned = ref(0)
const progressPath = ref('')

const summary = ref<DiffSummary | null>(null)
const entries = ref<DiffEntry[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = 200
const filterStatus = ref('')
const keyword = ref('')

let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null
let pollTimer: ReturnType<typeof setInterval> | null = null
let keywordTimer: ReturnType<typeof setTimeout> | null = null
let finalized = false

const STATUS_LABEL: Record<string, string> = { added: '新增', removed: '删除', modified: '修改', same: '相同' }
const STATUS_TAG: Record<string, 'success' | 'danger' | 'warning' | 'info'> = {
  added: 'success', removed: 'danger', modified: 'warning', same: 'info',
}

const statusLabel = (s: string) => STATUS_LABEL[s] ?? s
const statusTagType = (s: string) => STATUS_TAG[s] ?? 'info'
const sizeText = (n: number | null) => (n === null || n === undefined ? '-' : formatBytes(n))

const formatDuration = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  return `${Math.floor(s / 60)}m ${s % 60}s`
}

const parseIgnore = () =>
  ignoreText.value.split(/[\s,，]+/).map((s) => s.trim()).filter(Boolean)

const selectFolder = async (side: 'left' | 'right') => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === 'string') {
      if (side === 'left') leftPath.value = selected
      else rightPath.value = selected
    }
  } catch (e) {
    ElMessage.error(`选择文件夹失败: ${e}`)
  }
}

const swapPaths = () => {
  const t = leftPath.value
  leftPath.value = rightPath.value
  rightPath.value = t
}

const stopPolling = () => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

const startDiff = async () => {
  if (!leftPath.value || !rightPath.value) {
    ElMessage.warning('请先选择左右两个文件夹')
    return
  }
  if (scanId.value) {
    try {
      await dirDiffClear(scanId.value)
    } catch {
      /* 旧 scan_id 已过期，忽略 */
    }
  }

  scanning.value = true
  scanError.value = ''
  summary.value = null
  entries.value = []
  total.value = 0
  page.value = 1
  progressScanned.value = 0
  progressPath.value = ''
  finalized = false

  localStorage.setItem('folderDiff.lastLeft', leftPath.value)
  localStorage.setItem('folderDiff.lastRight', rightPath.value)
  localStorage.setItem('folderDiff.ignore', ignoreText.value)
  localStorage.setItem('folderDiff.compareContent', compareContent.value ? '1' : '0')

  try {
    scanId.value = await dirDiffStart(leftPath.value, rightPath.value, parseIgnore(), compareContent.value)
  } catch (e) {
    scanning.value = false
    scanError.value = `启动对比失败: ${e}`
    return
  }

  // ponytail: 取消/失败路径后端不发 complete 事件，用轮询兜底（AGENTS 规则10）
  pollTimer = setInterval(async () => {
    if (!scanId.value) return
    try {
      const state = await dirDiffStatus(scanId.value)
      if (state.status === 'completed') {
        await finalizeWithSummary()
      } else if (state.status === 'cancelled') {
        finish()
        ElMessage.info('已取消对比')
      } else if (state.status === 'failed') {
        finish()
        scanError.value = `对比失败: ${state.error}`
      }
    } catch {
      /* 忽略瞬时查询失败 */
    }
  }, 800)
}

const finish = () => {
  scanning.value = false
  stopPolling()
}

const finalizeWithSummary = async () => {
  if (finalized) return
  finalized = true
  finish()
  try {
    const s = await dirDiffGetSummary(scanId.value)
    summary.value = s
    await loadEntries()
    store.addHistory({
      tool: 'folderDiff',
      action: '目录对比',
      inputPreview: `${leftPath.value} ↔ ${rightPath.value}`.slice(0, 50),
      outputPreview: `新增${s.added} 删除${s.removed} 修改${s.modified} 相同${s.same}`,
      inputFull: `${leftPath.value}\n${rightPath.value}`,
      outputFull: JSON.stringify(s),
      options: { ignore: ignoreText.value, compareContent: compareContent.value },
    })
  } catch (e) {
    scanError.value = `读取结果失败: ${e}`
  }
}

const loadEntries = async () => {
  if (!scanId.value) return
  try {
    const res = await dirDiffGetEntries(
      scanId.value,
      filterStatus.value || undefined,
      keyword.value.trim() || undefined,
      pageSize,
      (page.value - 1) * pageSize,
    )
    entries.value = res.items
    total.value = res.total
  } catch (e) {
    ElMessage.error(`读取明细失败: ${e}`)
  }
}

const handleFilterChange = () => {
  page.value = 1
  loadEntries()
}

const handlePageChange = (p: number) => {
  page.value = p
  loadEntries()
}

const handleKeywordInput = () => {
  if (keywordTimer) clearTimeout(keywordTimer)
  keywordTimer = setTimeout(() => {
    page.value = 1
    loadEntries()
  }, 300)
}

const handleProgress = (event: { payload: { scan_id: string; scanned: number; current_path: string } }) => {
  if (event.payload.scan_id !== scanId.value) return
  progressScanned.value = event.payload.scanned
  progressPath.value = event.payload.current_path
}

const handleComplete = (event: { payload: { scan_id: string; summary: DiffSummary } }) => {
  if (event.payload.scan_id !== scanId.value) return
  finalizeWithSummary()
}

const cancelDiff = async () => {
  if (!scanId.value) return
  try {
    await dirDiffCancel(scanId.value)
    ElMessage.info('已请求取消…')
  } catch (e) {
    ElMessage.error(`取消失败: ${e}`)
  }
}

onMounted(async () => {
  unlistenProgress = await listen('dir-diff-progress', handleProgress as any)
  unlistenComplete = await listen('dir-diff-complete', handleComplete as any)
})

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress()
  if (unlistenComplete) unlistenComplete()
  stopPolling()
  if (keywordTimer) clearTimeout(keywordTimer)
  if (scanId.value) dirDiffClear(scanId.value).catch(() => {})
})

watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (!data || data.tool !== 'folderDiff') return
    const lines = (data.input || '').split('\n')
    if (lines[0]) leftPath.value = lines[0]
    if (lines[1]) rightPath.value = lines[1]
    if (data.options?.ignore !== undefined) ignoreText.value = data.options.ignore
    if (data.options?.compareContent !== undefined) compareContent.value = data.options.compareContent
    store.clearHistoryRestore()
  },
)
</script>

<style scoped>
.path-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.path-label {
  width: 40px;
  flex-shrink: 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.action-grid {
  margin-top: 4px;
}

.action-group.grow {
  flex: 1;
  min-width: 240px;
}

.action-group.grow :deep(.el-input) {
  width: 100%;
}

.error-tip {
  margin-top: 8px;
  color: var(--color-danger, #f56c6c);
  font-size: 13px;
}

.meta-text {
  color: var(--text-secondary);
  font-size: 12px;
}

.progress-path {
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.filter-input {
  width: 240px;
}

.pager {
  margin-top: 12px;
  justify-content: flex-end;
}
</style>
