<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="encoding-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="单文件转换" name="single" />
      <el-tab-pane label="批量转换" name="batch" />
    </el-tabs>

    <!-- 单文件转换 -->
    <div v-if="activeTab === 'single'">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">文件操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button size="small" @click="selectFile">选择文件</el-button>
              <el-button size="small" @click="detectEncoding" :disabled="!filePath">检测编码</el-button>
              <el-button size="small" type="primary" @click="convertFile" :disabled="!filePath || !targetEncoding">转换并保存</el-button>
            </div>
          </div>
          
          <div v-if="filePath" class="file-info">
            <span class="file-path">{{ filePath }}</span>
            <span v-if="detectedEncoding" class="encoding-badge">
              检测编码: {{ detectedEncoding }}
            </span>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">编码设置</span>
        </div>
        <div class="card-body">
          <div class="encoding-selectors">
            <div class="selector-group">
              <label>源编码:</label>
              <el-select v-model="sourceEncoding" placeholder="自动检测">
                <el-option label="自动检测" value="auto" />
                <el-option label="UTF-8" value="UTF-8" />
                <el-option label="GBK" value="GBK" />
                <el-option label="GB2312" value="GB2312" />
                <el-option label="ISO-8859-1" value="ISO-8859-1" />
              </el-select>
            </div>
            <div class="selector-group">
              <label>目标编码:</label>
              <el-select v-model="targetEncoding" placeholder="选择目标编码">
                <el-option label="UTF-8" value="UTF-8" />
                <el-option label="GBK" value="GBK" />
                <el-option label="GB2312" value="GB2312" />
                <el-option label="ISO-8859-1" value="ISO-8859-1" />
              </el-select>
            </div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">内容预览</span>
          <div class="card-actions">
            <el-button size="small" @click="loadPreview" :disabled="!filePath">刷新预览</el-button>
          </div>
        </div>
        <div class="card-body">
          <div v-if="loading" class="loading-state">加载中...</div>
          <el-input
            v-else
            v-model="previewContent"
            type="textarea"
            :rows="15"
            placeholder="选择文件后预览内容..."
            class="tool-textarea"
            readonly
          />
        </div>
      </div>
    </div>

    <!-- 批量转换 -->
    <div v-if="activeTab === 'batch'">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">选择文件</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <el-button size="small" @click="selectBatchFiles">选择多个文件</el-button>
              <el-button size="small" @click="clearBatchFiles" :disabled="batchFiles.length === 0">清空列表</el-button>
            </div>
          </div>
          <div v-if="batchFiles.length > 0" class="file-list">
            <div v-for="(file, index) in batchFiles" :key="index" class="file-item">
              <span class="file-name">{{ file }}</span>
              <el-button size="small" @click="removeBatchFile(index)">×</el-button>
            </div>
          </div>
          <div v-else class="empty-hint">请选择需要转换的文件</div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">编码设置</span>
        </div>
        <div class="card-body">
          <div class="encoding-selectors">
            <div class="selector-group">
              <label>源编码:</label>
              <el-select v-model="batchFromEncoding" style="width: 120px">
                <el-option label="UTF-8" value="UTF-8" />
                <el-option label="GBK" value="GBK" />
                <el-option label="GB2312" value="GB2312" />
                <el-option label="ISO-8859-1" value="ISO-8859-1" />
              </el-select>
            </div>
            <div class="selector-group">
              <label>目标编码:</label>
              <el-select v-model="batchToEncoding" style="width: 120px">
                <el-option label="UTF-8" value="UTF-8" />
                <el-option label="GBK" value="GBK" />
                <el-option label="GB2312" value="GB2312" />
                <el-option label="ISO-8859-1" value="ISO-8859-1" />
              </el-select>
            </div>
          </div>
          <div class="output-dir-row">
            <label>输出目录:</label>
            <el-input v-model="batchOutputDir" placeholder="选择输出目录" readonly style="flex: 1">
              <template #append>
                <el-button @click="selectOutputDir">选择</el-button>
              </template>
            </el-input>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">操作</span>
          <el-button size="small" type="primary" @click="handleBatchConvert" :disabled="batchFiles.length === 0 || !batchOutputDir">批量转换</el-button>
        </div>
        <div class="card-body">
          <div v-if="batchLoading" class="loading-state">转换中...</div>
          <div v-else>
            <div v-for="(result, index) in batchResults" :key="index" class="batch-result-item">
              <span class="result-file">{{ result.path }}</span>
              <span class="result-arrow">→</span>
              <span class="result-output">{{ result.output_path }}</span>
              <span :class="['result-status', result.success ? 'success' : 'error']">
                {{ result.success ? '成功' : '失败' }}
              </span>
              <span v-if="!result.success && result.error" class="result-error">{{ result.error }}</span>
            </div>
            <div v-if="batchResults.length === 0 && !batchLoading" class="empty-hint">点击"批量转换"开始</div>
          </div>
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
const activeTab = ref('single')

// 单文件
const filePath = ref('')
const sourceEncoding = ref('auto')
const targetEncoding = ref('')
const detectedEncoding = ref('')
const previewContent = ref('')
const loading = ref(false)

// 批量
const batchFiles = ref<string[]>([])
const batchFromEncoding = ref('UTF-8')
const batchToEncoding = ref('GBK')
const batchOutputDir = ref('')
const batchResults = ref<{ path: string; output_path: string; success: boolean; error: string | null }[]>([])
const batchLoading = ref(false)

const handleTabClick = () => {}

// 单文件方法
const selectFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Text Files', extensions: ['txt', 'csv', 'log', 'md', 'json', 'xml', 'html'] }]
    })
    
    if (selected) {
      filePath.value = selected as string
      await loadPreview()
    }
  } catch (error) {
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

const loadPreview = async () => {
  if (!filePath.value) return
  
  loading.value = true
  try {
    const enc = sourceEncoding.value === 'auto' ? 'UTF-8' : sourceEncoding.value
    previewContent.value = await invoke<string>('read_file_with_encoding', {
      path: filePath.value,
      encoding: enc
    })
  } catch (error) {
    ElMessage.error(`读取文件失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const detectEncoding = async () => {
  if (!filePath.value) return
  
  try {
    detectedEncoding.value = await invoke<string>('detect_file_encoding', {
      path: filePath.value
    })
    ElMessage.success(`检测到编码: ${detectedEncoding.value}`)
  } catch (error) {
    ElMessage.error(`检测编码失败: ${error}`)
  }
}

const convertFile = async () => {
  if (!filePath.value || !targetEncoding.value) {
    ElMessage.warning('请选择文件和目标编码')
    return
  }
  
  const fromEnc = sourceEncoding.value === 'auto' 
    ? (detectedEncoding.value || 'UTF-8') 
    : sourceEncoding.value
  
  const outputPath = filePath.value.replace(/(\.[^.]+)$/, `_converted$1`)
  
  try {
    const result = await invoke<string>('convert_file_encoding', {
      path: filePath.value,
      fromEncoding: fromEnc,
      toEncoding: targetEncoding.value,
      outputPath: outputPath
    })
    
    ElMessage.success(`转换成功: ${result}`)
    
    store.addHistory({
      tool: 'fileEncoding',
      action: `${fromEnc} -> ${targetEncoding.value}`,
      inputPreview: filePath.value,
      outputPreview: result,
      inputFull: filePath.value,
      outputFull: result,
    })
  } catch (error) {
    ElMessage.error(`转换失败: ${error}`)
  }
}

// 批量方法
const selectBatchFiles = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Text Files', extensions: ['txt', 'csv', 'log', 'md', 'json', 'xml', 'html'] }]
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      batchFiles.value = [...batchFiles.value, ...paths as string[]]
    }
  } catch (error) {
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

const clearBatchFiles = () => {
  batchFiles.value = []
}

const removeBatchFile = (index: number) => {
  batchFiles.value.splice(index, 1)
}

const selectOutputDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false
    })
    if (selected) {
      batchOutputDir.value = selected as string
    }
  } catch (error) {
    ElMessage.error(`选择目录失败: ${error}`)
  }
}

const handleBatchConvert = async () => {
  if (batchFiles.value.length === 0) {
    ElMessage.warning('请选择文件')
    return
  }
  if (!batchOutputDir.value) {
    ElMessage.warning('请选择输出目录')
    return
  }
  
  batchLoading.value = true
  batchResults.value = []
  try {
    batchResults.value = await invoke<any>('batch_convert_encoding', {
      paths: batchFiles.value,
      fromEncoding: batchFromEncoding.value,
      toEncoding: batchToEncoding.value,
      outputDir: batchOutputDir.value
    })
    const successCount = batchResults.value.filter(r => r.success).length
    ElMessage.success(`批量转换完成：${successCount}/${batchFiles.value.length} 成功`)
    store.addHistory({
      tool: 'fileEncoding',
      action: `batch ${batchFromEncoding.value} -> ${batchToEncoding.value}`,
      inputPreview: `${batchFiles.value.length} 个文件`,
      outputPreview: `${successCount}/${batchFiles.value.length} 成功`,
      inputFull: batchFiles.value.join('\n'),
      outputFull: batchResults.value.map(r => `${r.path}: ${r.success ? 'OK' : r.error}`).join('\n'),
    })
  } catch (error) {
    ElMessage.error(`批量转换失败: ${error}`)
  } finally {
    batchLoading.value = false
  }
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}

/* 二级 Tab（子功能切换） */
.encoding-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.encoding-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.encoding-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.encoding-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.encoding-tabs :deep(.el-tabs__item) {
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

.encoding-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.encoding-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.encoding-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .encoding-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .encoding-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .encoding-tabs :deep(.el-tabs__item.is-active) {
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

.card-actions {
  display: flex;
  gap: 8px;
}

.action-grid {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.file-info {
  margin-top: 12px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.file-path {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  flex: 1;
}

.encoding-badge {
  font-size: 12px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 8px;
  border-radius: 12px;
  border: 1px solid rgba(0, 212, 255, 0.3);
  white-space: nowrap;
}

.encoding-selectors {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
  align-items: center;
}

.selector-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.selector-group label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.output-dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.output-dir-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.tool-textarea {
  width: 100%;
}

.loading-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
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

.empty-hint {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.batch-result-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 6px;
  font-size: 13px;
}

.result-file {
  color: var(--text-primary);
  word-break: break-all;
  flex: 1;
}

.result-arrow {
  color: var(--text-secondary);
}

.result-output {
  color: var(--accent-cyan);
  word-break: break-all;
  flex: 1;
}

.result-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
  white-space: nowrap;
}

.result-status.success {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.result-status.error {
  color: var(--accent-red);
  background: rgba(239, 68, 68, 0.1);
}

.result-error {
  color: var(--accent-red);
  font-size: 12px;
}
</style>
