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
                <p>• 输入任意命名风格/短语，自动拆词并转出全部风格</p>
                <p>• 支持缩写词识别：HTTPServer → HTTP + Server</p>
                <p>• 每行一个标识符，多行批量转换；粘贴后自动转换</p>
                <p>• 点击单元格复制单个值，点击列头图标复制整列</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">风格</span>
            <el-checkbox
              :model-value="isAllSelected"
              :indeterminate="isIndeterminate"
              size="small"
              @change="handleToggleAll"
            >全选</el-checkbox>
            <el-checkbox-group v-model="enabledStyles" size="small" class="style-checks">
              <el-checkbox v-for="s in NAME_STYLE_META" :key="s.key" :label="s.key" size="small">
                {{ s.label }}
              </el-checkbox>
            </el-checkbox-group>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (每行一个，支持任意命名风格)</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="10" placeholder="例如:&#10;userID&#10;HTTP_Server_URL&#10;xml http request" resize="vertical" />
      </div>
    </div>

    <!-- 输出卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-tag v-if="rows.length" size="small" type="success">{{ rows.length }} 行 × {{ visibleStyles.length }} 风格</el-tag>
        </div>
      </div>
      <div class="card-body">
        <el-table
          v-if="rows.length"
          :data="rows"
          size="small"
          border
          stripe
          :max-height="520"
          class="name-case-table"
          row-key="source"
        >
          <el-table-column type="index" label="#" width="44" align="center" fixed="left" />
          <el-table-column label="原始" min-width="150" fixed="left" show-overflow-tooltip>
            <template #default="{ row }">
              <span class="cell-value source-cell" title="点击复制原始输入" @click="copyCell(row.source, '原始')">{{ row.source }}</span>
            </template>
          </el-table-column>
          <el-table-column
            v-for="s in visibleStyles"
            :key="s.key"
            :label="s.label"
            min-width="150"
            show-overflow-tooltip
          >
            <template #header>
              <span class="col-header">
                <el-tooltip content="复制整列" placement="top" effect="dark">
                  <el-icon class="col-copy" @click="copyColumn(s)"><CopyDocument /></el-icon>
                </el-tooltip>
                <span class="col-label">{{ s.label }}</span>
              </span>
            </template>
            <template #default="{ row }">
              <span class="cell-value" :title="`点击复制 ${s.label}`" @click="copyCell(row.results[s.key], s.label)">{{ row.results[s.key] }}</span>
            </template>
          </el-table-column>
        </el-table>
        <div v-else class="empty-tip">输入内容后自动转换，结果将展示在此处</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, CopyDocument } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import {
  NAME_STYLE_META, convertNameText,
  type NameStyle, type NameCaseRow,
} from '@/utils/nameCaseUtils'

const store = useToolboxStore()

const input = ref('')
const rows = ref<NameCaseRow[]>([])
const enabledStyles = ref<NameStyle[]>(NAME_STYLE_META.map(s => s.key))

const ALL_STYLE_KEYS = NAME_STYLE_META.map(s => s.key) as NameStyle[]
const visibleStyles = computed(() => NAME_STYLE_META.filter(s => enabledStyles.value.includes(s.key)))

const isAllSelected = computed(() => enabledStyles.value.length === NAME_STYLE_META.length)
const isIndeterminate = computed(() => enabledStyles.value.length > 0 && !isAllSelected.value)

const handleToggleAll = (checked: boolean | string | number) => {
  enabledStyles.value = checked ? [...ALL_STYLE_KEYS] : []
}

// 至少保留一种风格，避免空列/空输出
watch(enabledStyles, (val) => {
  if (val.length === 0) enabledStyles.value = [ALL_STYLE_KEYS[0]]
})

// ============ 转换（粘贴/输入 500ms 防抖自动执行） ============
let execTimer: ReturnType<typeof setTimeout> | null = null
let lastSavedKey = ''

const runConvert = () => {
  const text = input.value
  rows.value = text.trim() ? convertNameText(text) : []
  const enabled = enabledStyles.value
  if (!rows.value.length || !enabled.length) return
  // 去重：同一输入+同风格集合只记录一次
  const key = `${text}|${enabled.join(',')}`
  if (key === lastSavedKey) return
  lastSavedKey = key
  store.addHistory({
    tool: 'nameCase',
    action: '变量命名转换',
    inputPreview: text.slice(0, 50),
    outputPreview: rows.value.length ? `${rows.value[0].source} => ${enabled.map(s => rows.value[0].results[s]).join(' | ')}` : '',
    inputFull: text,
    outputFull: rows.value.map(r => `${r.source} => ${enabled.map(s => r.results[s]).join(' | ')}`).join('\n'),
    options: { styles: [...enabled] },
  })
}

watch(input, () => {
  if (execTimer) clearTimeout(execTimer)
  execTimer = setTimeout(runConvert, 500)
})

onBeforeUnmount(() => {
  if (execTimer) clearTimeout(execTimer)
})

// ============ 历史双击还原（KeepAlive 缓存后靠 watch 触发） ============
watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (!data || data.tool !== 'nameCase') return
    if (data.input) {
      input.value = data.input
      if (data.options?.styles?.length) {
        enabledStyles.value = [...data.options.styles]
      }
      store.clearHistoryRestore()
      runConvert()
    }
  },
)

// ============ 复制 ============
const copyText = async (text: string, label: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(`已复制: ${label}`)
  } catch {
    ElMessage.error('复制失败')
  }
}

const copyCell = (value: string, label: string) => {
  if (value) copyText(value, label)
}

const copyColumn = (s: { key: NameStyle; label: string }) => {
  const text = rows.value.map(r => r.results[s.key]).join('\n')
  if (text) copyText(text, `${s.label} 列`)
}

// ============ 输入区操作 ============
const handleClear = () => {
  input.value = ''
  rows.value = []
  lastSavedKey = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      ElMessage.warning('剪贴板为空')
      return
    }
    input.value = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}
</script>

<style scoped>
/* 页面特有样式；布局类（.tool-card/.card-header 等）使用 theme.css 全局定义 */
.style-checks {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 14px;
}

.empty-tip {
  padding: 24px 0;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.name-case-table :deep(.el-table__header th) {
  background: var(--bg-input);
}

.col-header {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
}

.col-copy {
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 13px;
  transition: color 0.2s;
}

.col-copy:hover {
  color: var(--accent-cyan);
}

.cell-value {
  display: block;
  width: 100%;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  cursor: pointer;
  transition: color 0.2s;
}

.cell-value:hover {
  color: var(--accent-cyan);
}
</style>
