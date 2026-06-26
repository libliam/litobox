<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="string-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="字符串工具" name="string" />
      <el-tab-pane label="批量处理" name="batch" />
    </el-tabs>

    <!-- 字符串工具 Tab -->
    <div v-if="activeTab === 'string'">
      <div class="tool-card sticky-card">
        <div class="card-header">
          <span class="card-title">操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <div class="group-label">空格处理</div>
              <div class="group-buttons">
                <el-button size="small" @click="applyTransform(stringUtils.trimLeadingTrailing)">首尾去空</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.trimAllSpaces)">全局去空</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.trimSpacesKeepNewlines)">保留换行去空</el-button>
              </div>
            </div>
            <div class="action-group">
              <div class="group-label">大小写转换</div>
              <div class="group-buttons">
                <el-button size="small" @click="applyTransform(stringUtils.toUpperCase)">全大写</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.toLowerCase)">全小写</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.toTitleCase)">首字母大写</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.toCamelCase)">转驼峰</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.toSnakeCase)">转下划线</el-button>
              </div>
            </div>
            <div class="action-group">
              <div class="group-label">文本处理</div>
              <div class="group-buttons">
                <el-button size="small" @click="applyTransform(text => stringUtils.joinLines(text, separator))">拼接</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.removeNewlines)">去除换行</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.removeTabs)">去除制表符</el-button>
                <el-button size="small" @click="applyTransform(stringUtils.removeEmptyLines)">删除空行</el-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">输入</span>
          <div class="card-actions">
            <VariablePicker @select="handleInsertVariable" />
            <el-button size="small" @click="handleClear">清空</el-button>
            <el-button size="small" @click="handlePaste">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="inputValue"
            type="textarea"
            :rows="8"
            placeholder="请输入文本内容..."
            resize="vertical"
          />
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">输出</span>
          <el-button size="small" @click="handleCopy">复制</el-button>
        </div>
        <div class="card-body">
          <el-input
            :model-value="outputValue"
            type="textarea"
            :rows="8"
            readonly
            resize="vertical"
          />
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">分隔符设置</span>
        </div>
        <div class="card-body">
          <el-input
            v-model="separator"
            placeholder="自定义分隔符（用于拼接操作）"
            size="small"
            style="width: 260px"
          />
        </div>
      </div>
    </div>

    <!-- 批量处理 Tab -->
    <div v-if="activeTab === 'batch'">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">批量操作</span>
        </div>
        <div class="card-body">
          <div class="action-grid">
            <div class="action-group">
              <span class="group-label">大小写</span>
              <el-button size="small" @click="applyBatch('toUpperCase')">转大写</el-button>
              <el-button size="small" @click="applyBatch('toLowerCase')">转小写</el-button>
              <el-button size="small" @click="applyBatch('capitalize')">首字母大写</el-button>
            </div>
            <div class="action-group">
              <span class="group-label">空格处理</span>
              <el-button size="small" @click="applyBatch('trimAllSpaces')">去除所有空格</el-button>
              <el-button size="small" @click="applyBatch('normalizeSpaces')">规范化空格</el-button>
            </div>
            <div class="action-group">
              <span class="group-label">文本处理</span>
              <el-button size="small" @click="applyBatch('removeEmptyLines')">删除空行</el-button>
              <el-button size="small" @click="applyBatch('removeDuplicates')">去重</el-button>
              <el-button size="small" @click="applyBatch('reverseLines')">行反转</el-button>
              <el-button size="small" @click="applyBatch('sortLines')">行排序</el-button>
            </div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">输入（每行一个文本片段）</span>
          <div class="card-actions">
            <el-button size="small" @click="handleBatchClear">清空</el-button>
            <el-button size="small" @click="handleBatchPaste">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="batchInputText"
            type="textarea"
            :rows="10"
            placeholder="每行输入一个文本片段..."
            class="tool-textarea"
          />
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">输出结果</span>
          <div class="card-actions">
            <el-button size="small" @click="handleCopyBatchAll">复制全部</el-button>
          </div>
        </div>
        <div class="card-body">
          <div v-if="batchResults.length === 0" class="empty-state">
            暂无结果，请先输入文本并选择操作
          </div>
          <div v-else class="results-list">
            <div v-for="(result, index) in batchResults" :key="index" class="result-item">
              <span class="result-index">{{ index + 1 }}</span>
              <div class="result-content">{{ result }}</div>
              <el-button size="small" @click="copyBatchResult(index)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import * as stringUtils from '@/utils/stringUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const activeTab = ref('string')

// 字符串工具状态
const inputValue = ref('')
const outputValue = ref('')
const separator = ref(',')

// 批量处理状态
const batchInputText = ref('')
const batchResults = ref<string[]>([])

const operationMap: Record<string, (text: string) => string> = {
  toUpperCase: stringUtils.toUpperCase,
  toLowerCase: stringUtils.toLowerCase,
  capitalize: stringUtils.capitalize,
  trimAllSpaces: stringUtils.removeAllSpaces,
  normalizeSpaces: stringUtils.normalizeSpaces,
  removeEmptyLines: stringUtils.removeEmptyLines,
  removeDuplicates: stringUtils.removeDuplicates,
  reverseLines: stringUtils.reverseLines,
  sortLines: stringUtils.sortLines
}

const handleTabClick = () => {
  // ponytail: tab切换时保持各子状态，后续可扩展tab缓存
}

// 字符串工具方法
const applyTransform = (transform: (text: string) => string) => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入文本内容')
    return
  }

  outputValue.value = transform(inputValue.value)
  store.addHistory({
    tool: 'string',
    action: 'transform',
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50)
  })
  ElMessage.success('处理完成')
}

const handleClear = () => {
  inputValue.value = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  inputValue.value = value
}

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(outputValue.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 批量处理方法
const applyBatch = (operation: string) => {
  if (!batchInputText.value.trim()) {
    ElMessage.warning('请先输入文本')
    return
  }

  const fn = operationMap[operation]
  if (!fn) {
    ElMessage.error('未知操作')
    return
  }

  const lines = batchInputText.value.split('\n')
  batchResults.value = lines.map(line => fn(line))

  store.addHistory({
    tool: 'string',
    action: `batch-${operation}`,
    inputPreview: batchInputText.value.slice(0, 50),
    outputPreview: batchResults.value.join('\n').slice(0, 50)
  })

  ElMessage.success(`已处理 ${batchResults.value.length} 行文本`)
}

const handleBatchClear = () => {
  batchInputText.value = ''
  batchResults.value = []
}

const handleBatchPaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    batchInputText.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败')
  }
}

const copyBatchResult = async (index: number) => {
  try {
    await navigator.clipboard.writeText(batchResults.value[index])
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyBatchAll = async () => {
  try {
    await navigator.clipboard.writeText(batchResults.value.join('\n'))
    ElMessage.success('已复制全部')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
}

.string-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .string-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.string-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.string-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.string-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.string-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.string-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions {
  display: flex;
  gap: 8px;
}
.card-body {
  padding: 20px;
}

.action-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.action-group {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}
.group-label {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 80px;
  padding-top: 4px;
  font-weight: 500;
}
.group-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex: 1;
}

/* 批量处理样式 */
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  font-size: 13px;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.result-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 13px;
}

.result-index {
  color: var(--accent-cyan);
  font-weight: 600;
  min-width: 24px;
  flex-shrink: 0;
}

.result-content {
  flex: 1;
  word-break: break-all;
  white-space: pre-wrap;
  color: var(--text-primary);
}

.results-list::-webkit-scrollbar {
  width: 4px;
}

.results-list::-webkit-scrollbar-track {
  background: transparent;
}

.results-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
</style>
