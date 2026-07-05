<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="batch-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="批量读取" name="read" />
      <el-tab-pane label="批量替换" name="replace" />
    </el-tabs>

    <!-- 批量读取 Tab -->
    <div v-if="activeTab === 'read'" class="tool-card">
      <div class="card-header">
        <span class="card-title">选择文件</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-buttons">
              <el-button size="small" @click="selectFiles">选择多个文件</el-button>
              <el-button size="small" @click="selectFolder">选择整个文件夹</el-button>
            </div>
          </div>
        </div>
        <div v-if="readFiles.length > 0" class="file-list">
          <div v-for="(file, index) in readFiles" :key="index" class="file-item">
            <span class="file-name">{{ file }}</span>
            <el-button size="small" @click="removeReadFile(index)">×</el-button>
          </div>
        </div>
        <div v-else class="empty-hint">请选择文件或文件夹</div>
      </div>
    </div>

    <div v-if="activeTab === 'read'" class="tool-card">
      <div class="card-header">
        <span class="card-title">编码设置</span>
      </div>
      <div class="card-body">
        <el-select v-model="readEncoding" style="width: 150px">
          <el-option label="UTF-8" value="UTF-8" />
          <el-option label="GBK" value="GBK" />
          <el-option label="GB2312" value="GB2312" />
        </el-select>
      </div>
    </div>

    <div v-if="activeTab === 'read'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleBatchRead" :disabled="readFiles.length === 0">批量读取</el-button>
      </div>
      <div class="card-body">
        <div v-if="readLoading" class="loading-state">读取中...</div>
        <div v-else>
          <div v-for="(result, index) in readResults" :key="index" class="read-result">
            <div class="result-header">
              <span class="result-file">{{ result.path }}</span>
              <span :class="['result-status', result.success ? 'success' : 'error']">
                {{ result.success ? '成功' : '失败' }}
              </span>
              <el-button v-if="result.success" size="small" @click="handleCopy(result.content || '')">复制</el-button>
            </div>
            <el-input
              v-if="result.success && result.content"
              :model-value="result.content"
              readonly
              type="textarea"
              :rows="4"
              resize="vertical"
              class="result-content"
            />
            <div v-if="!result.success && result.error" class="error-message">{{ result.error }}</div>
          </div>
          <div v-if="readResults.length === 0 && !readLoading" class="empty-hint">点击"批量读取"开始</div>
        </div>
      </div>
    </div>

    <!-- 批量替换 Tab -->
    <div v-if="activeTab === 'replace'" class="tool-card">
      <div class="card-header">
        <span class="card-title">选择文件</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-buttons">
              <el-button size="small" @click="selectReplaceFiles">选择多个文件</el-button>
              <el-button size="small" @click="selectReplaceFolder">选择整个文件夹</el-button>
            </div>
          </div>
        </div>
        <div v-if="replaceFiles.length > 0" class="file-list">
          <div v-for="(file, index) in replaceFiles" :key="index" class="file-item">
            <span class="file-name">{{ file }}</span>
            <el-button size="small" @click="removeReplaceFile(index)">×</el-button>
          </div>
        </div>
        <div v-else class="empty-hint">请选择文件或文件夹</div>
      </div>
    </div>

    <div v-if="activeTab === 'replace'" class="tool-card">
      <div class="card-header">
        <span class="card-title">替换设置</span>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>查找：</label>
          <el-input v-model="searchText" placeholder="要查找的文本" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>替换为：</label>
          <el-input v-model="replaceText" placeholder="替换后的文本" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>编码：</label>
          <el-select v-model="replaceEncoding" style="width: 120px">
            <el-option label="UTF-8" value="UTF-8" />
            <el-option label="GBK" value="GBK" />
          </el-select>
          <label style="margin-left: 16px">输出：</label>
          <el-radio-group v-model="replaceOutputMode" size="small">
            <el-radio-button label="overwrite">覆盖原文件</el-radio-button>
            <el-radio-button label="newdir">输出到新目录</el-radio-button>
          </el-radio-group>
        </div>
        <div v-if="replaceOutputMode === 'newdir'" class="input-row" style="margin-top: 12px">
          <label>输出目录：</label>
          <el-input v-model="replaceOutputDir" placeholder="选择输出目录" readonly>
            <template #append>
              <el-button @click="selectOutputDir">选择</el-button>
            </template>
          </el-input>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'replace'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleBatchReplace" :disabled="replaceFiles.length === 0 || !searchText">批量替换</el-button>
      </div>
      <div class="card-body">
        <div v-if="replaceLoading" class="loading-state">替换中...</div>
        <div v-else>
          <div v-for="(result, index) in replaceResults" :key="index" class="read-result">
            <div class="result-header">
              <span class="result-file">{{ result.path }}</span>
              <span :class="['result-status', result.success ? 'success' : 'error']">
                {{ result.success ? '成功' : '失败' }}
              </span>
            </div>
            <div v-if="!result.success && result.error" class="error-message">{{ result.error }}</div>
          </div>
          <div v-if="replaceResults.length === 0 && !replaceLoading" class="empty-hint">点击"批量替换"开始</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const activeTab = ref('read')

// 批量读取
const readFiles = ref<string[]>([])
const readEncoding = ref('UTF-8')
const readResults = ref<{ path: string; success: boolean; content: string | null; error: string | null }[]>([])
const readLoading = ref(false)

// 批量替换
const replaceFiles = ref<string[]>([])
const searchText = ref('')
const replaceText = ref('')
const replaceEncoding = ref('UTF-8')
const replaceOutputMode = ref<'overwrite' | 'newdir'>('newdir')
const replaceOutputDir = ref('')
const replaceResults = ref<{ path: string; success: boolean; error: string | null }[]>([])
const replaceLoading = ref(false)

const handleTabClick = () => {}

const selectFiles = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: true
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      readFiles.value = [...readFiles.value, ...paths as string[]]
    }
  } catch (error) {
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

const selectFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false
    })
    if (selected) {
      // ponytail: 简化版，需要用户手动添加文件，后续可扩展自动扫描文件夹
      ElMessage.info('已选择文件夹，请手动添加其中的 txt 文件')
    }
  } catch (error) {
    ElMessage.error(`选择文件夹失败: ${error}`)
  }
}

const removeReadFile = (index: number) => {
  readFiles.value.splice(index, 1)
}

const handleBatchRead = async () => {
  if (readFiles.value.length === 0) {
    ElMessage.warning('请选择文件')
    return
  }
  readLoading.value = true
  readResults.value = []
  try {
    readResults.value = await invoke<any>('batch_read_txt_files', {
      paths: readFiles.value,
      encoding: readEncoding.value
    })
    const successCount = readResults.value.filter(r => r.success).length
    ElMessage.success(`读取完成：${successCount}/${readFiles.value.length} 成功`)
  } catch (error) {
    ElMessage.error(`批量读取失败: ${error}`)
  } finally {
    readLoading.value = false
  }
}

const selectReplaceFiles = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: true
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      replaceFiles.value = [...replaceFiles.value, ...paths as string[]]
    }
  } catch (error) {
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

const selectReplaceFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false
    })
    if (selected) {
      ElMessage.info('已选择文件夹，请手动添加其中的 txt 文件')
    }
  } catch (error) {
    ElMessage.error(`选择文件夹失败: ${error}`)
  }
}

const removeReplaceFile = (index: number) => {
  replaceFiles.value.splice(index, 1)
}

const selectOutputDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false
    })
    if (selected) {
      replaceOutputDir.value = selected as string
    }
  } catch (error) {
    ElMessage.error(`选择目录失败: ${error}`)
  }
}

const handleBatchReplace = async () => {
  if (replaceFiles.value.length === 0) {
    ElMessage.warning('请选择文件')
    return
  }
  if (!searchText.value) {
    ElMessage.warning('请输入查找文本')
    return
  }
  if (replaceOutputMode.value === 'newdir' && !replaceOutputDir.value) {
    ElMessage.warning('请选择输出目录')
    return
  }

  replaceLoading.value = true
  replaceResults.value = []
  try {
    const outputDir = replaceOutputMode.value === 'newdir' ? replaceOutputDir.value : null
    replaceResults.value = await invoke<any>('batch_replace_in_files', {
      paths: replaceFiles.value,
      search: searchText.value,
      replacement: replaceText.value,
      encoding: replaceEncoding.value,
      outputDir: outputDir
    })
    const successCount = replaceResults.value.filter(r => r.success).length
    ElMessage.success(`替换完成：${successCount}/${replaceFiles.value.length} 成功`)
    store.addHistory({
      tool: 'batchText',
      action: 'replace',
      inputPreview: `${replaceFiles.value.length} 个文件, 查找: ${searchText.value.slice(0, 30)}`,
      outputPreview: `${successCount}/${replaceFiles.value.length} 成功`,
      inputFull: `${replaceFiles.value.length} 个文件, 查找: ${searchText.value}, 替换: ${replaceText.value}`,
      outputFull: `${successCount}/${replaceFiles.value.length} 成功`,
    })
  } catch (error) {
    ElMessage.error(`批量替换失败: ${error}`)
  } finally {
    replaceLoading.value = false
  }
}

const handleCopy = async (text: string) => {
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 0;
}

/* 二级 Tab（子功能切换） */
.batch-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.batch-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.batch-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.batch-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.batch-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 400;
  padding: 0 12px;
  height: 28px;
  line-height: 28px;
  border-radius: 4px;
  margin-right: 4px;
  transition: all 0.2s;
}

.batch-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.batch-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.batch-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .batch-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .batch-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .batch-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(8, 145, 178, 0.1);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
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

.card-body {
  padding: 16px 20px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}

.file-list {
  margin-top: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 4px;
}

.file-name {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  flex: 1;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.input-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  min-width: 60px;
}

.read-result {
  margin-bottom: 16px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
}

.read-result:last-child {
  margin-bottom: 0;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-bottom: 1px solid var(--border-color);
}

.result-file {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  flex: 1;
}

.result-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.result-status.success {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.result-status.error {
  color: var(--accent-red);
  background: rgba(239, 68, 68, 0.1);
}

.result-content {
  padding: 8px;
}

.loading-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
}

.empty-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}
</style>
