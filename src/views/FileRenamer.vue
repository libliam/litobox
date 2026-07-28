<template>
  <div class="tool-container">
    <!-- 文件夹选择 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">选择目录</span>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handlePickFolder">
            <el-icon><FolderOpened /></el-icon>
            浏览
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="dir-path">{{ currentDir || '请选择一个文件夹' }}</div>
        <div v-if="files.length > 0" class="file-stats">
          共 {{ files.length }} 个文件，已选 {{ selectedNames.length }} 个
        </div>
      </div>
    </div>

    <!-- 文件列表 -->
    <div v-if="files.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">文件列表</span>
        <div class="card-actions">
          <el-checkbox
            v-model="selectAll"
            :indeterminate="isIndeterminate"
            size="small"
            @change="handleSelectAll"
          >
            全选
          </el-checkbox>
          <el-input
            v-model="filterText"
            size="small"
            placeholder="搜索文件名..."
            clearable
            style="width: 180px"
            :prefix-icon="Search"
          />
        </div>
      </div>
      <div class="card-body rename-card-body">
        <DataTable :data="filteredFiles" max-height="360" @selection-change="handleSelectionChange">
          <el-table-column type="selection" width="40" />
          <el-table-column prop="name" label="文件名" min-width="240" show-overflow-tooltip />
          <el-table-column prop="ext" label="类型" width="80" />
          <el-table-column prop="size" label="大小" width="90" :formatter="formatSize" />
        </DataTable>
      </div>
    </div>

    <!-- 重命名规则 -->
    <div v-if="selectedNames.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">重命名规则</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">模式</div>
            <div class="group-buttons">
              <el-radio-group v-model="rules.mode" size="small">
                <el-radio-button value="replace">替换</el-radio-button>
                <el-radio-button value="regex">正则</el-radio-button>
                <el-radio-button value="prefix_suffix">前后缀</el-radio-button>
                <el-radio-button value="sequence">序号</el-radio-button>
              </el-radio-group>
            </div>
          </div>
        </div>

        <!-- 替换模式 -->
        <div v-if="rules.mode === 'replace'" class="rule-fields">
          <div class="rule-row">
            <label>查找：</label>
            <el-input v-model="replaceFind" size="small" placeholder="要查找的文本" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <label>替换为：</label>
            <el-input v-model="replaceTo" size="small" placeholder="替换后的文本" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <el-checkbox v-model="replaceCaseSensitive" size="small">区分大小写</el-checkbox>
          </div>
        </div>

        <!-- 正则模式 -->
        <div v-if="rules.mode === 'regex'" class="rule-fields">
          <div class="rule-row">
            <label>正则：</label>
            <el-input v-model="regexPattern" size="small" placeholder="正则表达式" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <label>替换为：</label>
            <el-input v-model="regexReplacement" size="small" placeholder="替换文本（支持 $1 捕获组）" clearable style="width: 300px" />
          </div>
        </div>

        <!-- 前后缀模式 -->
        <div v-if="rules.mode === 'prefix_suffix'" class="rule-fields">
          <div class="rule-row">
            <label>前缀：</label>
            <el-input v-model="psPrefix" size="small" placeholder="添加到文件名前" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <label>后缀：</label>
            <el-input v-model="psSuffix" size="small" placeholder="添加到文件名后（保留扩展名）" clearable style="width: 300px" />
          </div>
        </div>

        <!-- 序号模式 -->
        <div v-if="rules.mode === 'sequence'" class="rule-fields">
          <div class="rule-row">
            <label>前缀：</label>
            <el-input v-model="seqPrefix" size="small" placeholder="序号前文字" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <label>后缀：</label>
            <el-input v-model="seqSuffix" size="small" placeholder="序号后文字" clearable style="width: 300px" />
          </div>
          <div class="rule-row">
            <label>起始编号：</label>
            <el-input-number v-model="seqStart" :min="0" :max="99999" size="small" style="width: 140px" />
          </div>
          <div class="rule-row">
            <label>位数补齐：</label>
            <el-input-number v-model="seqPadding" :min="1" :max="10" size="small" style="width: 140px" />
          </div>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div v-if="previews.length > 0" class="tool-card">
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button
              type="primary"
              size="small"
              :loading="isProcessing"
              @click="handleExecute"
            >
              <el-icon><Check /></el-icon>
              执行重命名 ({{ renameCount }})
            </el-button>
            <el-button
              v-if="backups.length > 0"
              size="small"
              :loading="isProcessing"
              @click="handleUndo"
            >
              <el-icon><Refresh /></el-icon>
              撤销 ({{ backups.length }})
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 预览结果 -->
    <div v-if="previews.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">预览</span>
        <div class="card-actions">
          <span class="preview-stats">
            {{ validPreviewCount }} 个文件将被重命名
          </span>
        </div>
      </div>
      <div class="card-body rename-card-body">
        <DataTable :data="previews" max-height="480">
          <el-table-column label="原文件名" min-width="240" show-overflow-tooltip>
            <template #default="{ row }">
              <span :class="{ 'rename-error': row.error && !row.error.includes('未变化') }">
                {{ row.old_name }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="新文件名" min-width="240" show-overflow-tooltip>
            <template #default="{ row }">
              <span v-if="!row.error" class="new-name">{{ row.new_name }}</span>
              <span v-else class="rename-error">{{ row.error }}</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100">
            <template #default="{ row }">
              <el-tag v-if="!row.error && row.old_name !== row.new_name" size="small" type="success">将重命名</el-tag>
              <el-tag v-else-if="row.old_name === row.new_name && !row.error" size="small" type="info">无变化</el-tag>
              <el-tag v-else size="small" type="danger">错误</el-tag>
            </template>
          </el-table-column>
        </DataTable>
      </div>
    </div>

    <!-- 执行结果 -->
    <div v-if="resultMessage" class="tool-card">
      <div class="card-body">
        <el-alert
          :title="resultMessage"
          :type="resultType"
          show-icon
          :closable="true"
          @close="resultMessage = ''"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { FolderOpened, Search, Check, Refresh } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import DataTable from '@/components/DataTable.vue'

interface FileEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  ext: string
}

interface RenameRules {
  mode: string
  find_text: string | null
  replace_text: string | null
  case_sensitive: boolean | null
  pattern: string | null
  replacement: string | null
  prefix: string | null
  suffix: string | null
  seq_prefix: string | null
  seq_suffix: string | null
  start_number: number | null
  padding: number | null
}

interface RenamePreview {
  old_name: string
  new_name: string
  error: string | null
}

interface RenamePair {
  old_path: string
  new_path: string
}

interface RenameExecuteResult {
  success_count: number
  failed_count: number
  failures: [string, string][]
  backups: RenamePair[]
}

const store = useToolboxStore()

// ============ 状态 ============
const currentDir = ref('')
const files = ref<FileEntry[]>([])
const selectedFiles = ref<FileEntry[]>([])
const filterText = ref('')
const selectAll = ref(false)
const isIndeterminate = ref(false)

const rules = reactive<RenameRules>({
  mode: 'replace',
  find_text: null,
  replace_text: null,
  case_sensitive: true,
  pattern: null,
  replacement: null,
  prefix: null,
  suffix: null,
  seq_prefix: null,
  seq_suffix: null,
  start_number: 1,
  padding: 2,
})

const previews = ref<RenamePreview[]>([])
const backups = ref<RenamePair[]>([])
const isProcessing = ref(false)
const resultMessage = ref('')
const resultType = ref<'success' | 'warning' | 'error'>('success')

// ============ 计算属性 ============
const filteredFiles = computed(() => {
  if (!filterText.value) return files.value
  const q = filterText.value.toLowerCase()
  return files.value.filter(f => !f.is_dir && f.name.toLowerCase().includes(q))
})

const selectedNames = computed(() => selectedFiles.value.map(f => f.name))

const renameCount = computed(() =>
  previews.value.filter(p => !p.error && p.old_name !== p.new_name).length
)

const validPreviewCount = computed(() => renameCount.value)

// ============ 选中文件与刷新后文件列表对齐 ============
const reconcileSelection = () => {
  const newSelected: FileEntry[] = []
  for (const p of previews.value) {
    // 重命名成功的用新名，失败的用原名
    const targetName = p.error && p.old_name !== p.new_name ? p.old_name : p.new_name
    const entry = files.value.find(f => f.name === targetName)
    if (entry) newSelected.push(entry)
  }
  if (newSelected.length > 0) {
    selectedFiles.value = newSelected
    selectAll.value = newSelected.length === filteredFiles.value.length
    isIndeterminate.value = newSelected.length > 0 && newSelected.length < filteredFiles.value.length
  }
}

// ============ 对话框命令 ============
const handlePickFolder = async () => {
  try {
    const dir = await invoke<string | null>('rename_pick_folder')
    if (dir) {
      currentDir.value = dir
      await loadFiles(dir)
    }
  } catch (e) {
    ElMessage.error('选择文件夹失败: ' + String(e))
  }
}

const loadFiles = async (dir: string, keepState = false) => {
  try {
    const entries = await invoke<FileEntry[]>('rename_list_files', { path: dir })
    files.value = entries.filter(f => !f.is_dir)
    if (!keepState) {
      selectedFiles.value = []
      selectAll.value = false
      isIndeterminate.value = false
      previews.value = []
      backups.value = []
    }
  } catch (e) {
    ElMessage.error('加载文件列表失败: ' + String(e))
  }
}

// ============ 选择逻辑 ============
const handleSelectionChange = (selection: FileEntry[]) => {
  selectedFiles.value = selection
  const totalFiltered = filteredFiles.value.length
  const selectedCount = selection.length
  selectAll.value = totalFiltered > 0 && selectedCount === totalFiltered
  isIndeterminate.value = selectedCount > 0 && selectedCount < totalFiltered
  // 选中变化时自动重新预览
  debouncedPreview()
}

const handleSelectAll = (val: boolean) => {
  if (val) {
    selectedFiles.value = [...filteredFiles.value]
  } else {
    selectedFiles.value = []
  }
}

// ============ 规则绑定（双向绑定到 reactive rules） ============
const replaceFind = computed({
  get: () => rules.find_text ?? '',
  set: (v) => { rules.find_text = v || null; debouncedPreview() }
})
const replaceTo = computed({
  get: () => rules.replace_text ?? '',
  set: (v) => { rules.replace_text = v || null; debouncedPreview() }
})
const replaceCaseSensitive = computed({
  get: () => rules.case_sensitive ?? true,
  set: (v) => { rules.case_sensitive = v; debouncedPreview() }
})
const regexPattern = computed({
  get: () => rules.pattern ?? '',
  set: (v) => { rules.pattern = v || null; debouncedPreview() }
})
const regexReplacement = computed({
  get: () => rules.replacement ?? '',
  set: (v) => { rules.replacement = v || null; debouncedPreview() }
})
const psPrefix = computed({
  get: () => rules.prefix ?? '',
  set: (v) => { rules.prefix = v || null; debouncedPreview() }
})
const psSuffix = computed({
  get: () => rules.suffix ?? '',
  set: (v) => { rules.suffix = v || null; debouncedPreview() }
})
const seqPrefix = computed({
  get: () => rules.seq_prefix ?? '',
  set: (v) => { rules.seq_prefix = v || null; debouncedPreview() }
})
const seqSuffix = computed({
  get: () => rules.seq_suffix ?? '',
  set: (v) => { rules.seq_suffix = v || null; debouncedPreview() }
})
const seqStart = computed({
  get: () => rules.start_number ?? 1,
  set: (v) => { rules.start_number = v; debouncedPreview() }
})
const seqPadding = computed({
  get: () => rules.padding ?? 2,
  set: (v) => { rules.padding = v; debouncedPreview() }
})

// ============ 模式切换时自动预览 ============
watch(() => rules.mode, () => {
  debouncedPreview()
})

// ============ 防抖预览 ============
let previewTimer: ReturnType<typeof setTimeout> | null = null
const debouncedPreview = () => {
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(() => {
    generatePreview()
  }, 300)
}

const generatePreview = async () => {
  if (selectedNames.value.length === 0) {
    previews.value = []
    resultMessage.value = ''
    return
  }

  try {
    previews.value = await invoke<RenamePreview[]>('rename_preview', {
      files: selectedNames.value,
      rules: { ...rules },
    })
  } catch (e) {
    // 静默处理
  }
}

// ============ 执行重命名 ============
const handleExecute = async () => {
  const toRename = previews.value.filter(
    p => !p.error && p.old_name !== p.new_name
  )
  if (toRename.length === 0) {
    ElMessage.warning('没有需要重命名的文件')
    return
  }

  isProcessing.value = true
  resultMessage.value = ''

  try {
    // 构建重命名对
    const renames: RenamePair[] = toRename.map(p => {
      const entry = selectedFiles.value.find(f => f.name === p.old_name)
      const newPath = currentDir.value + '\\' + p.new_name
      return {
        old_path: entry?.path || currentDir.value + '\\' + p.old_name,
        new_path: newPath,
      }
    })

    const result = await invoke<RenameExecuteResult>('rename_execute', { renames })
    backups.value = result.backups

    // 先显示操作结果反馈
    if (result.success_count > 0) {
      ElMessage.success(`重命名成功 ${result.success_count} 个文件`)
    }
    if (result.failed_count > 0) {
      const failMsg = result.failures.map(([name]) => name).join('、')
      ElMessage.error(`${result.failed_count} 个文件重命名失败：${failMsg}`)
    }

    // 重新加载文件列表（保留预览和撤销记录）
    await loadFiles(currentDir.value, true)
    // 将选中映射到刷新后的文件条目，后续切换规则模式时预览基于新文件名
    reconcileSelection()

    if (result.success_count > 0 || result.failed_count > 0) {
      let msg = ''
      if (result.success_count > 0) msg += `成功重命名 ${result.success_count} 个文件`
      if (result.failed_count > 0) {
        msg += msg ? '；' : ''
        msg += `${result.failed_count} 个失败`
      }
      resultType.value = result.failed_count > 0 ? 'warning' : 'success'
      if (result.failures.length > 0) {
        const detail = result.failures.map(([name, err]) => `${name}: ${err}`).join('；')
        msg += '。' + detail
      }
      resultMessage.value = msg
    }

    store.addHistory({
      tool: 'fileRenamer',
      action: `批量重命名（${rules.mode}模式）`,
      inputPreview: `${toRename.length} 个文件`,
      outputPreview: `成功 ${result.success_count} 个`,
      inputFull: JSON.stringify(toRename.map(p => p.old_name)),
      outputFull: resultMessage.value,
    })
  } catch (e) {
    ElMessage.error('执行重命名失败: ' + String(e))
    resultType.value = 'error'
    resultMessage.value = '执行重命名失败: ' + String(e)
  } finally {
    isProcessing.value = false
  }
}

// ============ 撤销 ============
const handleUndo = async () => {
  if (backups.value.length === 0) return

  isProcessing.value = true
  resultMessage.value = ''

  try {
    const result = await invoke<RenameExecuteResult>('rename_undo', {
      backups: [...backups.value],
    })

    if (result.success_count > 0) {
      ElMessage.success(`撤销成功 ${result.success_count} 个文件`)
      resultType.value = 'success'
      resultMessage.value = `撤销成功 ${result.success_count} 个文件`
      backups.value = []
      previews.value = []
      selectedFiles.value = []
      selectAll.value = false
      isIndeterminate.value = false
      rules.mode = 'replace'
      rules.find_text = null
      rules.replace_text = null
      await loadFiles(currentDir.value, true)
    } else {
      ElMessage.warning('撤销失败：没有文件被恢复')
      resultType.value = 'warning'
      resultMessage.value = '撤销失败'
    }
  } catch (e) {
    ElMessage.error('撤销失败: ' + String(e))
    resultType.value = 'error'
    resultMessage.value = '撤销失败: ' + String(e)
  } finally {
    isProcessing.value = false
  }
}

// ============ 格式化 ============
const formatSize = (_row: any, _col: any, val: number) => {
  if (val < 1024) return val + ' B'
  if (val < 1024 * 1024) return (val / 1024).toFixed(1) + ' KB'
  return (val / (1024 * 1024)).toFixed(1) + ' MB'
}
</script>

<style scoped>
.dir-path {
  color: var(--text-secondary);
  font-size: 13px;
  word-break: break-all;
  font-family: monospace;
}

.file-stats {
  color: var(--text-muted);
  font-size: 12px;
  margin-top: 6px;
}

.rename-card-body {
  padding: 0 !important;
}

.rule-fields {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.rule-row label {
  color: var(--text-secondary);
  font-size: 13px;
  min-width: 60px;
  text-align: right;
  flex-shrink: 0;
}

.preview-stats {
  color: var(--text-muted);
  font-size: 12px;
}

.new-name {
  color: var(--accent-green);
  font-weight: 500;
}

.rename-error {
  color: var(--accent-red);
}

:deep(.el-table) {
  --el-table-border-color: var(--border-color);
  --el-table-header-bg-color: var(--bg-card);
  --el-table-tr-bg-color: var(--bg-card);
  --el-table-row-hover-bg-color: rgba(0, 212, 255, 0.05);
}

html.light :deep(.el-table) {
  --el-table-header-bg-color: var(--bg-card);
  --el-table-tr-bg-color: var(--bg-card);
}
</style>
