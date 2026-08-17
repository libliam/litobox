<template>
  <div class="tool-container">
    <!-- 压缩卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩 ZIP</span>
        <div class="card-actions">
          <el-button size="small" @click="addFiles">添加文件</el-button>
          <el-button size="small" @click="addFolders">添加文件夹</el-button>
          <el-button size="small" @click="clearFiles" v-if="selectedFiles.length">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <!-- 待压缩文件列表 -->
        <div v-if="selectedFiles.length" class="zip-file-list">
          <div v-for="(f, i) in selectedFiles" :key="f" class="zip-file-item">
            <el-icon class="zip-file-icon"><FolderOpened v-if="isDir(f)" /><Document v-else /></el-icon>
            <span class="zip-file-name" :title="f">{{ f }}</span>
            <el-button text size="small" class="zip-file-remove" @click="selectedFiles.splice(i, 1)">
              <el-icon><Close /></el-icon>
            </el-button>
          </div>
        </div>
        <div v-else class="zip-empty-tip">
          <p>选择文件或文件夹，将其打包为 ZIP 压缩包</p>
        </div>

        <!-- 压缩选项 -->
        <div class="zip-options">
          <div class="zip-option-row">
            <span class="zip-option-label">输出文件</span>
            <el-input v-model="createOutput" placeholder="选择 ZIP 输出位置" size="small" class="zip-option-input">
              <template #append>
                <el-button @click="chooseCreateOutput">选择</el-button>
              </template>
            </el-input>
          </div>
          <div class="zip-option-row">
            <span class="zip-option-label">压缩级别</span>
            <el-select v-model="createLevel" size="small" class="zip-option-input" style="max-width: 220px">
              <el-option label="存储（不压缩，最快）" :value="0" />
              <el-option label="最快" :value="1" />
              <el-option label="默认（平衡）" :value="6" />
              <el-option label="最大（最慢）" :value="9" />
            </el-select>
          </div>
          <div class="zip-option-row">
            <span class="zip-option-label">加密密码</span>
            <el-input v-model="createPassword" type="password" show-password placeholder="留空则不加密" size="small" class="zip-option-input" />
          </div>
        </div>

        <!-- 进度与结果 -->
        <div v-if="createProgress.show" class="zip-progress">
          <el-progress :percentage="createProgress.percent" :stroke-width="8" />
          <div class="zip-progress-text" v-if="createProgress.percent < 100">
            {{ createProgress.current }} / {{ createProgress.total }} · {{ createProgress.file }}
          </div>
          <div class="zip-result" v-if="createResult">
            <span class="zip-result-ok">✓ 压缩完成</span>
            <span>{{ createResult.file_count }} 个文件 · 原始 {{ formatSize(createResult.total_size) }} → {{ formatSize(createResult.compressed_size) }}</span>
            <span>压缩率 {{ (createResult.ratio * 100).toFixed(1) }}%</span>
            <el-button size="small" type="primary" link @click="openResultPath(createResult.path)">打开位置</el-button>
          </div>
        </div>

        <div class="zip-action-bar">
          <el-button type="primary" size="small" :loading="creating" :disabled="!canCreate" @click="startCreate">
            开始压缩
          </el-button>
        </div>
      </div>
    </div>

    <!-- 解压卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">解压 ZIP</span>
        <div class="card-actions">
          <el-button size="small" @click="chooseZip">选择 ZIP 文件</el-button>
          <el-button size="small" @click="clearZip" v-if="zipPath">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="zipPath" class="zip-file-selected">
          <el-icon class="zip-file-icon"><Box /></el-icon>
          <span class="zip-file-name" :title="zipPath">{{ zipPath }}</span>
        </div>

        <!-- 解压选项 -->
        <div class="zip-options">
          <div class="zip-option-row">
            <span class="zip-option-label">解压到</span>
            <el-input v-model="extractDest" placeholder="选择解压目标目录" size="small" class="zip-option-input">
              <template #append>
                <el-button @click="chooseExtractDest">选择</el-button>
              </template>
            </el-input>
          </div>
          <div class="zip-option-row">
            <span class="zip-option-label">密码</span>
            <el-input v-model="extractPassword" type="password" show-password placeholder="加密 ZIP 请输入密码" size="small" class="zip-option-input" />
          </div>
          <div class="zip-option-row">
            <span class="zip-option-label">已存在文件</span>
            <el-checkbox v-model="extractOverwrite">覆盖</el-checkbox>
          </div>
        </div>

        <!-- 条目列表 -->
        <div v-if="zipEntries.length" class="zip-entry-table">
          <div class="zip-entry-table-header">
            <span>压缩包内容（{{ zipEntries.length }} 项）</span>
            <span class="zip-entry-actions">
              <el-checkbox v-model="selectAllEntries" :disabled="!zipEntries.length">全选</el-checkbox>
              <span class="zip-entry-sum">{{ selectedEntryNames.length }} 项选中</span>
            </span>
          </div>
          <el-scrollbar max-height="320px">
            <div class="zip-entry-row" v-for="e in zipEntries" :key="e.name">
              <el-checkbox
                :model-value="entryChecked(e)"
                @change="(v: boolean | string | number) => toggleEntry(e.name, !!v)"
                :disabled="e.is_dir"
              />
              <el-icon class="zip-entry-icon"><FolderOpened v-if="e.is_dir" /><Document v-else /></el-icon>
              <span class="zip-entry-name" :title="e.name">{{ e.name }}</span>
              <span class="zip-entry-size">{{ e.is_dir ? '目录' : formatSize(e.size) }}</span>
              <span class="zip-entry-compressed">{{ e.is_dir ? '' : formatSize(e.compressed_size) }}</span>
              <span class="zip-entry-modified">{{ e.modified || '' }}</span>
            </div>
          </el-scrollbar>
        </div>
        <div v-else-if="zipPath && !zipLoading && !zipError" class="zip-empty-tip">
          <p>{{ zipLoading ? '解析中...' : '该压缩包为空' }}</p>
        </div>

        <!-- 加载错误（如加密需密码） -->
        <div v-if="zipError" class="zip-error">{{ zipError }}</div>

        <!-- 进度与结果 -->
        <div v-if="extractProgress.show" class="zip-progress">
          <el-progress :percentage="extractProgress.percent" :stroke-width="8" />
          <div class="zip-progress-text" v-if="extractProgress.percent < 100">
            {{ extractProgress.current }} / {{ extractProgress.total }} · {{ extractProgress.file }}
          </div>
          <div class="zip-result" v-if="extractResult">
            <span class="zip-result-ok">✓ 解压完成</span>
            <span>{{ extractResult.file_count }} 个文件 · 共 {{ formatSize(extractResult.extracted_bytes) }}</span>
            <span v-if="extractResult.skipped_existing">跳过已存在 {{ extractResult.skipped_existing }} 项</span>
            <span v-if="extractResult.skipped_unsafe">跳过不安全路径 {{ extractResult.skipped_unsafe }} 项</span>
            <el-button size="small" type="primary" link @click="openResultPath(extractResult.path)">打开位置</el-button>
          </div>
        </div>

        <div class="zip-action-bar">
          <el-button type="primary" size="small" :loading="extracting" :disabled="!canExtract" @click="startExtract">
            {{ selectedEntryNames.length ? `解压选中（${selectedEntryNames.length}）` : '全部解压' }}
          </el-button>
          <el-button size="small" :loading="zipLoading" :disabled="!zipPath" @click="loadZip">重新解析</el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { Box, Close, Document, FolderOpened } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

interface ZipEntry {
  name: string
  is_dir: boolean
  size: number
  compressed_size: number
  modified: string | null
  crc32: number
}

interface ZipCreateResult {
  path: string
  file_count: number
  dir_count: number
  total_size: number
  compressed_size: number
  ratio: number
}

interface ZipExtractResult {
  path: string
  file_count: number
  extracted_bytes: number
  skipped_existing: number
  skipped_unsafe: number
}

interface ZipProgressPayload {
  stage: 'create' | 'extract' | 'done'
  percent: number
  current: number
  total: number
  file: string
}

// ---------- 压缩 ----------
const selectedFiles = ref<string[]>([])
const createOutput = ref('')
const createLevel = ref(6)
const createPassword = ref('')
const creating = ref(false)
const createProgress = ref({ show: false, percent: 0, current: 0, total: 0, file: '' })
const createResult = ref<ZipCreateResult | null>(null)

const canCreate = computed(
  () => selectedFiles.value.length > 0 && !!createOutput.value && !creating.value,
)

function isDir(path: string): boolean {
  // 启发式：无常见扩展名结尾的路径视为文件夹（仅影响图标展示）
  return !/\.[^/\\]+$/.test(path)
}

async function addFiles() {
  const selected = await open({ multiple: true, directory: false })
  if (!selected) return
  const list = Array.isArray(selected) ? selected : [selected]
  const existing = new Set(selectedFiles.value)
  for (const p of list) if (!existing.has(p)) selectedFiles.value.push(p)
}

async function addFolders() {
  const selected = await open({ directory: true })
  if (!selected) return
  const existing = new Set(selectedFiles.value)
  if (!existing.has(selected)) selectedFiles.value.push(selected)
}

function clearFiles() {
  selectedFiles.value = []
  createResult.value = null
}

async function chooseCreateOutput() {
  // 默认定位到第一个待压缩文件所在目录，文件名 archive.zip
  let defaultPath = 'archive.zip'
  if (selectedFiles.value.length) {
    const first = selectedFiles.value[0]
    const dir = first.replace(/[/\\][^/\\]*$/, '')
    defaultPath = dir ? `${dir}/archive.zip` : 'archive.zip'
  }
  const result = await save({
    defaultPath,
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (result) createOutput.value = result
}

async function startCreate() {
  creating.value = true
  createResult.value = null
  createProgress.value = { show: true, percent: 0, current: 0, total: 0, file: '' }
  const unlisten = await listen<ZipProgressPayload>('zip-progress', (e) => {
    if (e.payload.stage !== 'create') return
    createProgress.value.percent = Math.round(e.payload.percent)
    createProgress.value.current = e.payload.current
    createProgress.value.total = e.payload.total
    createProgress.value.file = e.payload.file
  })
  try {
    const res = await invoke<ZipCreateResult>('zip_create', {
      files: selectedFiles.value,
      dest: createOutput.value,
      level: createLevel.value,
      password: createPassword.value || null,
    })
    createResult.value = res
    createProgress.value.percent = 100
    store.addHistory({
      tool: 'zipTool',
      action: '压缩',
      inputPreview: `${selectedFiles.value.length} 项 → ${createOutput.value.split(/[/\\]/).pop()}`,
      outputPreview: createOutput.value.split(/[/\\]/).pop() || '',
      inputFull: selectedFiles.value.join('\n'),
      outputFull: createOutput.value,
      options: { level: createLevel.value, encrypted: !!createPassword.value },
    })
  } catch (e: any) {
    createProgress.value.show = false
    ElMessage.error(typeof e === 'string' ? e : e.message || '压缩失败')
  } finally {
    unlisten()
    creating.value = false
  }
}

// ---------- 解压 ----------
const zipPath = ref('')
const zipLoading = ref(false)
const zipError = ref('')
const zipEntries = ref<ZipEntry[]>([])
const checkedEntries = ref<Set<string>>(new Set())
const extractDest = ref('')
const extractPassword = ref('')
const extractOverwrite = ref(false)
const extracting = ref(false)
const extractProgress = ref({ show: false, percent: 0, current: 0, total: 0, file: '' })
const extractResult = ref<ZipExtractResult | null>(null)

const selectAllEntries = computed({
  get: () => zipEntries.value.length > 0 && zipEntries.value.every((e) => e.is_dir || checkedEntries.value.has(e.name)),
  set: (v: boolean) => {
    if (v) {
      for (const e of zipEntries.value) if (!e.is_dir) checkedEntries.value.add(e.name)
    } else {
      checkedEntries.value.clear()
    }
  },
})

const selectedEntryNames = computed(() => zipEntries.value.filter((e) => !e.is_dir && checkedEntries.value.has(e.name)).map((e) => e.name))

const canExtract = computed(() => !!zipPath.value && !!extractDest.value && !extracting.value)

function entryChecked(e: ZipEntry): boolean {
  return e.is_dir || checkedEntries.value.has(e.name)
}

function toggleEntry(name: string, v: boolean) {
  if (v) checkedEntries.value.add(name)
  else checkedEntries.value.delete(name)
}

async function chooseZip() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (!selected) return
  zipPath.value = selected
  zipEntries.value = []
  checkedEntries.value = new Set()
  zipError.value = ''
  extractResult.value = null
  extractDest.value = ''
  await loadZip()
}

async function loadZip() {
  if (!zipPath.value) return
  zipLoading.value = true
  zipError.value = ''
  zipEntries.value = []
  checkedEntries.value = new Set()
  try {
    zipEntries.value = await invoke<ZipEntry[]>('zip_list', {
      path: zipPath.value,
      password: extractPassword.value || null,
    })
    // 默认全选（目录除外）
    for (const e of zipEntries.value) if (!e.is_dir) checkedEntries.value.add(e.name)
  } catch (e: any) {
    zipError.value = typeof e === 'string' ? e : e.message || '解析 ZIP 失败'
    if (!extractDest.value) {
      // 默认解压目录 = ZIP 同名文件夹
      const base = zipPath.value.replace(/\.zip$/i, '')
      extractDest.value = base
    }
  } finally {
    zipLoading.value = false
  }
}

async function chooseExtractDest() {
  const selected = await open({ directory: true })
  if (selected) extractDest.value = selected
}

async function startExtract() {
  if (!zipPath.value) return
  const wants = selectedEntryNames.value
  if (!wants.length && zipEntries.value.some((e) => !e.is_dir)) {
    ElMessage.warning('请至少勾选一个要解压的文件')
    return
  }
  extracting.value = true
  extractResult.value = null
  extractProgress.value = { show: true, percent: 0, current: 0, total: 0, file: '' }
  const unlisten = await listen<ZipProgressPayload>('zip-progress', (e) => {
    if (e.payload.stage !== 'extract') return
    extractProgress.value.percent = Math.round(e.payload.percent)
    extractProgress.value.current = e.payload.current
    extractProgress.value.total = e.payload.total
    extractProgress.value.file = e.payload.file
  })
  try {
    const res = await invoke<ZipExtractResult>('zip_extract', {
      path: zipPath.value,
      dest: extractDest.value,
      entries: wants.length ? wants : null,
      password: extractPassword.value || null,
      overwrite: extractOverwrite.value,
    })
    extractResult.value = res
    extractProgress.value.percent = 100
    store.addHistory({
      tool: 'zipTool',
      action: '解压',
      inputPreview: `${zipPath.value.split(/[/\\]/).pop()} → ${extractDest.value}`,
      outputPreview: extractDest.value,
      inputFull: zipPath.value,
      outputFull: extractDest.value,
      options: { entries: wants.length, overwrite: extractOverwrite.value },
    })
  } catch (e: any) {
    extractProgress.value.show = false
    ElMessage.error(typeof e === 'string' ? e : e.message || '解压失败')
  } finally {
    unlisten()
    extracting.value = false
  }
}

function clearZip() {
  zipPath.value = ''
  zipEntries.value = []
  checkedEntries.value = new Set()
  zipError.value = ''
  extractResult.value = null
}

// 密码变化后，若之前因加密解析失败则自动重新解析
watch(extractPassword, () => {
  if (zipPath.value && zipError.value && !zipLoading.value) {
    loadZip()
  }
})

// 监听历史恢复（不自动执行）
// 清理进度监听
onUnmounted(() => {
  // listen 的 unlisten 在每次操作后已调用
})

// ---------- 工具函数 ----------
function formatSize(bytes: number): string {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const val = bytes / Math.pow(1024, i)
  return `${val.toFixed(val >= 100 ? 0 : 1)} ${units[i]}`
}

async function openResultPath(path: string) {
  try {
    await invoke('http_open_url', { url: path })
  } catch {
    ElMessage.error('打开失败')
  }
}
</script>

<style scoped>
.zip-file-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  max-height: 180px;
  overflow-y: auto;
  border: 1px solid var(--border-color, #e4e7ed);
  border-radius: 6px;
  padding: 6px;
}
.zip-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 3px 6px;
  border-radius: 4px;
}
.zip-file-item:hover {
  background: var(--hover-bg, rgba(0, 0, 0, 0.04));
}
.zip-file-icon {
  flex-shrink: 0;
  color: var(--color-secondary, #909399);
}
.zip-file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}
.zip-file-remove {
  flex-shrink: 0;
}
.zip-empty-tip {
  color: var(--color-secondary, #909399);
  font-size: 13px;
  padding: 8px 0;
  text-align: center;
}
.zip-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 8px 0 12px;
}
.zip-option-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.zip-option-label {
  width: 78px;
  flex-shrink: 0;
  font-size: 13px;
  color: var(--text-secondary, #606266);
}
.zip-option-input {
  flex: 1;
}
.zip-progress {
  margin: 4px 0 12px;
}
.zip-progress-text {
  font-size: 12px;
  color: var(--color-secondary, #909399);
  margin-top: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.zip-result {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 13px;
  color: var(--text-primary, #303133);
  background: var(--success-bg, rgba(16, 185, 129, 0.08));
  border: 1px solid var(--success-border, rgba(16, 185, 129, 0.3));
  border-radius: 6px;
  padding: 8px 12px;
  margin-top: 8px;
}
.zip-result-ok {
  color: #10b981;
  font-weight: 600;
}
.zip-action-bar {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}
.zip-file-selected {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}
.zip-entry-table {
  border: 1px solid var(--border-color, #e4e7ed);
  border-radius: 6px;
  overflow: hidden;
  margin: 8px 0 12px;
}
.zip-entry-table-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-secondary, #f5f7fa);
  font-size: 13px;
  color: var(--text-secondary, #606266);
  font-weight: 600;
}
.zip-entry-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 400;
}
.zip-entry-sum {
  font-size: 12px;
  color: var(--color-secondary, #909399);
}
.zip-entry-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  font-size: 13px;
  border-bottom: 1px solid var(--border-light, #f0f0f0);
}
.zip-entry-row:last-child {
  border-bottom: none;
}
.zip-entry-row:hover {
  background: var(--hover-bg, rgba(0, 0, 0, 0.03));
}
.zip-entry-icon {
  color: var(--color-secondary, #909399);
  flex-shrink: 0;
}
.zip-entry-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.zip-entry-size {
  width: 80px;
  text-align: right;
  color: var(--text-secondary, #606266);
  font-size: 12px;
  flex-shrink: 0;
}
.zip-entry-compressed {
  width: 80px;
  text-align: right;
  color: var(--color-secondary, #909399);
  font-size: 12px;
  flex-shrink: 0;
}
.zip-entry-modified {
  width: 130px;
  text-align: right;
  color: var(--color-secondary, #909399);
  font-size: 12px;
  flex-shrink: 0;
}
.zip-error {
  color: #f56c6c;
  font-size: 13px;
  margin: 8px 0;
  background: rgba(245, 108, 108, 0.08);
  border: 1px solid rgba(245, 108, 108, 0.25);
  border-radius: 6px;
  padding: 8px 12px;
}
</style>
