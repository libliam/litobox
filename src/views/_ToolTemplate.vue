<!--
  LitoBox 工具页面模板
  使用说明：
  1. 复制此文件为新页面（如 NewTool.vue）
  2. 替换所有 "NewTool" / "newTool" / "new-tool" 相关命名
  3. 根据需求增删 Tab 面板
  4. 样式部分已包含完整规范，无需修改

  ⚠️ 布局规范（重要）：
  - Tab 栏放在独立的 `.tool-card.sticky-card` 中，使用 `class="xxx-tabs"`（自定义类名）
  - 各 Tab 内容用 `v-if="activeTab === 'xxx'"` 的 `.tool-card` 独立渲染，不要放在 el-tab-pane 内
  - 参考：PdfTool.vue、ImageToolEnhanced.vue
-->
<template>
  <div class="tool-container">
    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="new-tool-tabs">
        <el-tab-pane label="功能一" name="feature1" />
        <el-tab-pane label="功能二" name="feature2" />
      </el-tabs>
    </div>

    <!-- Tab 1: 功能一 -->
    <div v-if="activeTab === 'feature1'" class="tool-card">
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

    <!-- Tab 2: 功能二 -->
    <div v-if="activeTab === 'feature2'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <el-button type="primary" size="small" @click="handleFeature2">执行</el-button>
      </div>
    </div>

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
/* ===== Tab 样式（必须自定义类名，参考 PdfTool/ImageToolEnhanced） ===== */
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

/* ===== 页面特有样式 ===== */
:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}
</style>
