<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 结构化对比两个 JSON：忽略对象键顺序，数组按索引逐项比较</p>
                <p>• 完全相同的子树不产生差异，仅列出 新增/删除/修改 节点</p>
                <p>• 点击差异路径或值可复制；左右任一 JSON 不合法会标红提示</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">操作</span>
            <el-button size="small" @click="handleSwap">交换左右</el-button>
            <el-button size="small" @click="loadSample">加载示例</el-button>
            <el-button size="small" @click="handleClear">清空两侧</el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入卡：左右并排 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (两侧 JSON 合法后自动对比)</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
        </div>
      </div>
      <div class="card-body">
        <div class="diff-inputs">
          <div class="diff-input-col" :class="{ 'err-col': leftErr }">
            <div class="diff-col-head">
              <span class="col-title">左值 · 旧</span>
              <div class="col-actions">
                <el-button size="small" @click="handleSidePaste('left')">粘贴</el-button>
                <el-button size="small" @click="handleSideClear('left')">清空</el-button>
              </div>
            </div>
            <el-input v-model="left" type="textarea" :rows="10" resize="vertical" @focus="lastFocused = 'left'" />
            <div v-if="leftErr" class="diff-err">{{ leftErr }}</div>
          </div>
          <div class="diff-input-col" :class="{ 'err-col': rightErr }">
            <div class="diff-col-head">
              <span class="col-title">右值 · 新</span>
              <div class="col-actions">
                <el-button size="small" @click="handleSidePaste('right')">粘贴</el-button>
                <el-button size="small" @click="handleSideClear('right')">清空</el-button>
              </div>
            </div>
            <el-input v-model="right" type="textarea" :rows="10" resize="vertical" @focus="lastFocused = 'right'" />
            <div v-if="rightErr" class="diff-err">{{ rightErr }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 输出卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <template v-if="compared">
            <el-tag v-if="identical" size="small" type="success">完全一致</el-tag>
            <template v-else>
              <span class="counts">
                <span class="c-add">+{{ counts.added }}</span>
                <span class="c-del">-{{ counts.removed }}</span>
                <span class="c-chg">~{{ counts.changed }}</span>
              </span>
              <el-tag size="small" type="warning">差异 {{ rows.length }} 处</el-tag>
              <el-button size="small" @click="copyDiffText">复制差异文本</el-button>
            </template>
          </template>
        </div>
      </div>
      <div class="card-body">
        <div v-if="compared && identical" class="empty-tip success-tip">
          <el-icon class="check-icon"><CircleCheck /></el-icon>
          两个 JSON 结构完全一致（键顺序、数组索引均一致）
        </div>
        <DataTable v-else-if="rows.length" :data="displayRows" max-height="520">
          <el-table-column label="类型" width="86" align="center">
            <template #default="{ row }">
              <el-tag :type="kindTag(row.kind)" size="small" effect="dark">{{ kindText(row.kind) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="差异路径" min-width="230" show-overflow-tooltip>
            <template #default="{ row }">
              <span class="cell-value path-cell" title="点击复制路径" @click="copyCell(row.path, '路径')">{{ row.path }}</span>
            </template>
          </el-table-column>
          <el-table-column label="左值" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <span
                v-if="row.left !== undefined"
                class="cell-value"
                :title="`点击复制完整值`"
                @click="copyFull(row.left, '左值')"
              >{{ cellDisplay(row.left) }}</span>
              <span v-else class="void-cell">∅</span>
            </template>
          </el-table-column>
          <el-table-column label="右值" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <span
                v-if="row.right !== undefined"
                class="cell-value"
                title="点击复制完整值"
                @click="copyFull(row.right, '右值')"
              >{{ cellDisplay(row.right) }}</span>
              <span v-else class="void-cell">∅</span>
            </template>
          </el-table-column>
        </DataTable>
        <div v-else class="empty-tip">在左右两侧输入合法的 JSON 后自动进行结构对比</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, CircleCheck } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import DataTable from '@/components/DataTable.vue'
import {
  diffJson, formatValue,
  type JsonDiffKind, type JsonDiffRow,
} from '@/utils/jsonDiffUtils'

const store = useToolboxStore()

const left = ref('')
const right = ref('')
const leftErr = ref('')
const rightErr = ref('')
const compared = ref(false)
const identical = ref(false)
const rows = ref<JsonDiffRow[]>([])
const counts = ref({ added: 0, removed: 0, changed: 0 })
const lastFocused = ref<'left' | 'right'>('left')

const KIND_TEXT: Record<JsonDiffKind, string> = { added: '+ 新增', removed: '- 删除', changed: '~ 修改' }
const KIND_TAG: Record<JsonDiffKind, 'success' | 'danger' | 'warning'> = { added: 'success', removed: 'danger', changed: 'warning' }
const kindText = (k: JsonDiffKind) => KIND_TEXT[k]
const kindTag = (k: JsonDiffKind) => KIND_TAG[k]

const displayRows = computed(() => (rows.value.length > 1000 ? rows.value.slice(0, 1000) : rows.value))

// ============ 对比（500ms 防抖自动执行） ============
let execTimer: ReturnType<typeof setTimeout> | null = null
let lastSavedKey = ''

const tryParse = (text: string): { value?: unknown; error?: string } => {
  if (!text.trim()) return {}
  try {
    return { value: JSON.parse(text) }
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    return { error: `JSON 解析失败: ${msg.slice(0, 140)}` }
  }
}

const runCompare = () => {
  const lText = left.value
  const rText = right.value
  leftErr.value = ''
  rightErr.value = ''
  if (!lText.trim() || !rText.trim()) {
    compared.value = false
    return
  }
  const l = tryParse(lText)
  const r = tryParse(rText)
  leftErr.value = l.error ?? ''
  rightErr.value = r.error ?? ''
  if (l.error || r.error) return
  const result = diffJson(l.value, r.value)
  compared.value = true
  identical.value = result.identical
  rows.value = result.rows
  counts.value = result.counts
  recordHistory(lText, rText, result)
}

const recordHistory = (lText: string, rText: string, result: { rows: JsonDiffRow[]; identical: boolean }) => {
  const key = `${lText}|${rText}`
  if (key === lastSavedKey) return
  lastSavedKey = key
  const lines = diffToLines(result.rows)
  store.addHistory({
    tool: 'jsonDiff',
    action: 'JSON 结构化对比',
    inputPreview: `${lText.slice(0, 40)} ⇄ ${rText.slice(0, 40)}`,
    outputPreview: result.identical ? '完全一致' : lines[0]?.slice(0, 50) ?? '',
    inputFull: lText,
    outputFull: result.identical ? '两个 JSON 完全一致' : lines.join('\n'),
    options: { right: rText },
  })
}

watch([left, right], () => {
  if (execTimer) clearTimeout(execTimer)
  execTimer = setTimeout(runCompare, 500)
})

onBeforeUnmount(() => {
  if (execTimer) clearTimeout(execTimer)
})

// ============ 差异文本导出（复制/历史共用） ============
const valToText = (v: unknown): string => (v === undefined ? '(空)' : formatValue(v))
const diffToLines = (rows: JsonDiffRow[]): string[] =>
  rows.map((r) => {
    const l = valToText(r.left)
    const rt = valToText(r.right)
    return `${KIND_TEXT[r.kind]} ${r.path}  ${l} → ${rt}`
  })

const copyDiffText = async () => {
  try {
    await navigator.clipboard.writeText(diffToLines(rows.value).join('\n'))
    ElMessage.success('已复制差异文本')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 单元格复制 ============
const trunc = (s: string, max: number) => (s.length > max ? `${s.slice(0, max)}…` : s)
const cellDisplay = (v: unknown) => trunc(formatValue(v), 300)
const copyText = async (text: string, label: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(`已复制: ${label}`)
  } catch {
    ElMessage.error('复制失败')
  }
}
const copyCell = (text: string, label: string) => copyText(text, label)
const copyFull = (v: unknown, label: string) => copyText(formatValue(v), label)

// ============ 输入区操作 ============
const handleSwap = () => {
  const t = left.value
  left.value = right.value
  right.value = t
}

const handleClear = () => {
  left.value = ''
  right.value = ''
  leftErr.value = ''
  rightErr.value = ''
  compared.value = false
  rows.value = []
  lastSavedKey = ''
}

const handleSideClear = (side: 'left' | 'right') => {
  if (side === 'left') left.value = ''
  else right.value = ''
}

const handleSidePaste = async (side: 'left' | 'right') => {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      ElMessage.warning('剪贴板为空')
      return
    }
    if (side === 'left') left.value = text
    else right.value = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  if (lastFocused.value === 'left') left.value = value
  else right.value = value
}

const loadSample = () => {
  left.value = JSON.stringify(
    {
      name: 'HTTP 服务配置(旧)',
      version: 1,
      server: { port: 8080, host: '127.0.0.1', timeout: 30 },
      headers: ['Content-Type', 'Accept', 'Authorization'],
      features: { cache: true, retry: { times: 3 } },
      deprecatedField: '待删除',
    },
    null,
    2,
  )
  right.value = JSON.stringify(
    {
      name: 'HTTP 服务配置(新)',
      version: 2,
      server: { port: 9090, host: '0.0.0.0', timeout: 30 },
      headers: ['Content-Type', 'Accept'],
      features: { cache: true, retry: { times: 5, backoff: 200 } },
      logging: { level: 'info' },
    },
    null,
    2,
  )
}

// ============ 历史双击还原（KeepAlive 缓存后靠 watch 触发） ============
watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (!data || data.tool !== 'jsonDiff') return
    if (data.input) {
      left.value = data.input
      right.value = typeof data.options?.right === 'string' ? data.options.right : ''
      store.clearHistoryRestore()
      runCompare()
    }
  },
)
</script>

<style scoped>
/* 页面特有样式；布局类使用 theme.css 全局定义 */
.diff-inputs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.diff-col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.col-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
  letter-spacing: 1px;
}

.col-actions {
  display: flex;
  gap: 6px;
}

.diff-input-col.err-col :deep(.el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 8px color-mix(in srgb, var(--accent-red) 35%, transparent);
}

.diff-err {
  margin-top: 4px;
  font-size: 12px;
  color: var(--accent-red);
  word-break: break-all;
}

.empty-tip {
  padding: 24px 0;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.success-tip {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: var(--accent-green);
}

.check-icon {
  font-size: 16px;
}

.counts {
  display: inline-flex;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
}

.c-add { color: var(--accent-green); }
.c-del { color: var(--accent-red); }
.c-chg { color: var(--accent-orange); }

.cell-value {
  display: block;
  width: 100%;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  cursor: pointer;
  transition: color 0.2s;
  word-break: break-all;
}

.path-cell {
  color: var(--accent-cyan);
}

.cell-value:hover {
  color: var(--accent-cyan);
}

.void-cell {
  color: var(--text-muted);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}
</style>
