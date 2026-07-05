<!--
  LitoBox 工具页面模板
  使用说明：
  1. 复制此文件为新页面（如 NewTool.vue）
  2. 替换所有 "NewTool" 相关命名
  3. 根据需求增删 Tab 面板
  4. 样式部分已包含完整规范，无需修改
-->
<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="new-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 示例功能 -->
      <el-tab-pane label="功能一" name="feature1">
        <!-- 操作卡片（sticky 置顶） -->
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>功能说明提示文字</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">选项</div>
                <el-select v-model="feature1Option" size="small" style="width: 140px">
                  <el-option label="选项A" value="a" />
                  <el-option label="选项B" value="b" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleFeature1">执行</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('feature1')">清空</el-button>
              <el-button size="small" @click="handlePaste('feature1')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.feature1.input"
              type="textarea"
              :rows="8"
              placeholder="请输入内容..."
              resize="vertical"
            />
          </div>
        </div>

        <!-- 输出卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('feature1')">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="tabState.feature1.output"
              type="textarea"
              :rows="8"
              readonly
              resize="vertical"
              :class="{ 'error': tabState.feature1.isError }"
            />
            <div v-if="tabState.feature1.error" class="error-message">
              {{ tabState.feature1.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 另一个功能 -->
      <el-tab-pane label="功能二" name="feature2">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleFeature2">执行</el-button>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('feature2')">清空</el-button>
              <el-button size="small" @click="handlePaste('feature2')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.feature2.input"
              type="textarea"
              :rows="8"
              placeholder="请输入内容..."
              resize="vertical"
            />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('feature2')">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="tabState.feature2.output"
              type="textarea"
              :rows="8"
              readonly
              resize="vertical"
              :class="{ 'error': tabState.feature2.isError }"
            />
            <div v-if="tabState.feature2.error" class="error-message">
              {{ tabState.feature2.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('feature1')

// 每个 Tab 独立输入/输出状态
const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  feature1: { input: '', output: '', error: '', isError: false },
  feature2: { input: '', output: '', error: '', isError: false }
})

// 当前 Tab 的状态引用（方便在单个 Tab 内使用）
const currentInput = computed({
  get: () => tabState[activeTab.value].input,
  set: (val) => { tabState[activeTab.value].input = val }
})
const currentOutput = computed({
  get: () => tabState[activeTab.value].output,
  set: (val) => { tabState[activeTab.value].output = val }
})
const currentError = computed({
  get: () => tabState[activeTab.value].error,
  set: (val) => { tabState[activeTab.value].error = val }
})
const currentIsError = computed({
  get: () => tabState[activeTab.value].isError,
  set: (val) => { tabState[activeTab.value].isError = val }
})

// ============ 功能配置 ============
const feature1Option = ref('a')

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
}

const handlePaste = async (tab: string) => {
  try {
    const text = await navigator.clipboard.readText()
    tabState[tab].input = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (tab: string) => {
  const text = tabState[tab].output || tabState[tab].input
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

const addHistory = (action: string) => {
  store.addHistory({
    tool: 'newTool',
    action,
    inputPreview: currentInput.value.slice(0, 50),
    outputPreview: currentOutput.value.slice(0, 50),
    inputFull: currentInput.value,
    outputFull: currentOutput.value,
  })
}

// ============ 自动执行（粘贴/输入后 300ms 触发） ============
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(() => tabState[activeTab.value].input, (val) => {
  if (!val.trim()) return
  if (autoExecTimer) clearTimeout(autoExecTimer)
  autoExecTimer = setTimeout(() => {
    autoExecute()
  }, 300)
})

const autoExecute = () => {
  const tab = activeTab.value
  switch (tab) {
    case 'feature1':
      handleFeature1()
      break
    case 'feature2':
      handleFeature2()
      break
  }
}

// ============ 功能实现 ============
const handleFeature1 = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  // TODO: 实现功能逻辑
  currentOutput.value = currentInput.value
  currentError.value = ''
  currentIsError.value = false
  addHistory('功能一')
  ElMessage.success('执行成功')
}

const handleFeature2 = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  // TODO: 实现功能逻辑
  currentOutput.value = currentInput.value
  currentError.value = ''
  currentIsError.value = false
  addHistory('功能二')
  ElMessage.success('执行成功')
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.new-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .new-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.new-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.new-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.new-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.new-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.new-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 工具卡片 ===== */
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

/* Sticky 卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
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
  align-items: center;
  gap: 6px;
}

/* 操作按钮 */
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

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}

/* 错误提示 */
.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}
</style>
