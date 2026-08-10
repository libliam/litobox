<template>
  <div class="tool-container">
    <!-- Schema 输入卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">JSON Schema（draft-07）</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 校验 JSON 数据是否符合 Schema</p>
                <p>• 依据 Schema 一键生成 Mock 测试数据</p>
                <p>• 导出为 TypeScript 接口定义</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="loadExample">示例 Schema</el-button>
          <el-button size="small" @click="handleClearSchema">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="schemaText"
          type="textarea"
          :rows="8"
          placeholder='{"type": "object", "properties": {...}}'
          resize="vertical"
          class="code-input"
        />
        <div v-if="schemaError" class="schema-error">{{ schemaError }}</div>
      </div>
    </div>

    <el-tabs v-model="activeTab" class="schema-tabs">
      <!-- ============ 校验 ============ -->
      <el-tab-pane label="数据校验" name="validate">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">JSON 数据</span>
            <div class="card-actions">
              <el-button size="small" @click="loadExampleData">示例数据</el-button>
              <el-button size="small" type="primary" @click="handleValidate">校验</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="dataText"
              type="textarea"
              :rows="8"
              placeholder="输入待校验的 JSON 数据..."
              resize="vertical"
              class="code-input"
            />
            <div v-if="validateResult" class="validate-result" :class="validateResult.ok ? 'ok' : 'fail'">
              <div class="validate-summary">
                <span class="dot" />
                <span v-if="validateResult.ok">校验通过：数据符合 Schema</span>
                <span v-else-if="validateResult.schemaError">{{ validateResult.schemaError }}</span>
                <span v-else>校验失败：发现 {{ validateResult.issues.length }} 个错误</span>
              </div>
              <div v-if="validateResult.issues.length" class="issue-list">
                <div v-for="(issue, i) in validateResult.issues" :key="i" class="issue-item">
                  <span class="issue-path">{{ issue.path }}</span>
                  <span class="issue-keyword">{{ issue.keyword }}</span>
                  <span class="issue-msg">{{ issue.message }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- ============ Mock 生成 ============ -->
      <el-tab-pane label="Mock 生成" name="mock">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">Mock 数据</span>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleGenerateMock">生成 Mock</el-button>
              <el-button size="small" @click="handleCopyMock">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <label class="switch-item">
                <span>包含可选字段</span>
                <el-switch v-model="mockOptions.alwaysFakeOptionals" size="small" />
              </label>
              <label class="switch-item">
                <span>优先使用默认值</span>
                <el-switch v-model="mockOptions.useDefaultValue" size="small" />
              </label>
            </div>
            <el-input
              :model-value="mockResult"
              type="textarea"
              :rows="10"
              readonly
              resize="vertical"
              placeholder="点击生成 Mock 按钮，生成符合 Schema 的测试数据"
              class="code-input"
            />
          </div>
        </div>
      </el-tab-pane>

      <!-- ============ TS 导出 ============ -->
      <el-tab-pane label="TS 接口" name="ts">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">TypeScript 接口</span>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleExportTs">导出</el-button>
              <el-button size="small" @click="handleCopyTs">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">类型名</div>
                <el-input v-model="tsTypeName" placeholder="User" size="small" style="width: 160px" />
              </div>
            </div>
            <el-input
              :model-value="tsResult"
              type="textarea"
              :rows="10"
              readonly
              resize="vertical"
              placeholder="点击导出按钮，将 Schema 转换为 TypeScript 接口"
            />
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  validateJsonData, generateMock, schemaToTs,
  EXAMPLE_SCHEMA, EXAMPLE_DATA, selfCheck,
  type ValidationResult, type MockOptions,
} from '@/utils/schemaUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 状态 ============
const activeTab = ref('validate')
const schemaText = ref('')
const dataText = ref('')
const validateResult = ref<ValidationResult | null>(null)
const mockOptions = ref<MockOptions>({ alwaysFakeOptionals: true, useDefaultValue: true })
const mockResult = ref('')
const tsTypeName = ref('User')
const tsResult = ref('')
const schemaError = ref('')

// ============ 示例 ============
const loadExample = () => {
  schemaText.value = EXAMPLE_SCHEMA
  schemaError.value = ''
  ElMessage.success('已载入示例 Schema')
}

const loadExampleData = () => {
  dataText.value = EXAMPLE_DATA
  ElMessage.success('已载入示例数据')
}

// ============ 校验 ============
const handleValidate = () => {
  if (!schemaText.value.trim()) {
    ElMessage.warning('请先输入 JSON Schema')
    return
  }
  if (!dataText.value.trim()) {
    ElMessage.warning('请先输入待校验的 JSON 数据')
    return
  }
  validateResult.value = validateJsonData(schemaText.value, dataText.value)
  const r = validateResult.value
  if (r.ok) {
    ElMessage.success('校验通过')
    store.addHistory({
      tool: 'schemaTool',
      action: 'JSON Schema 校验',
      inputPreview: schemaText.value.slice(0, 60),
      outputPreview: '校验通过',
    })
  } else if (r.schemaError) {
    schemaError.value = r.schemaError
  } else {
    ElMessage.warning(`校验失败：${r.issues.length} 个错误`)
    store.addHistory({
      tool: 'schemaTool',
      action: 'JSON Schema 校验',
      inputPreview: schemaText.value.slice(0, 60),
      outputPreview: `失败 ${r.issues.length} 个错误`,
    })
  }
}

// ============ Mock ============
const handleGenerateMock = () => {
  if (!schemaText.value.trim()) {
    ElMessage.warning('请先输入 JSON Schema')
    return
  }
  try {
    mockResult.value = generateMock(schemaText.value, { ...mockOptions.value })
    ElMessage.success('Mock 数据已生成')
    store.addHistory({
      tool: 'schemaTool',
      action: 'Schema Mock 生成',
      inputPreview: schemaText.value.slice(0, 60),
      outputPreview: mockResult.value.slice(0, 60),
    })
  } catch (e: any) {
    ElMessage.error('Mock 生成失败: ' + e.message)
  }
}

const handleCopyMock = async () => {
  if (!mockResult.value) {
    ElMessage.warning('请先生成 Mock 数据')
    return
  }
  try {
    await navigator.clipboard.writeText(mockResult.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.warning('复制失败')
  }
}

// ============ TS 导出 ============
const handleExportTs = () => {
  if (!schemaText.value.trim()) {
    ElMessage.warning('请先输入 JSON Schema')
    return
  }
  try {
    tsResult.value = schemaToTs(schemaText.value, tsTypeName.value)
    ElMessage.success('TS 接口已生成')
    store.addHistory({
      tool: 'schemaTool',
      action: 'Schema → TS 接口',
      inputPreview: schemaText.value.slice(0, 60),
      outputPreview: tsResult.value.slice(0, 60),
    })
  } catch (e: any) {
    ElMessage.error('TS 导出失败: ' + e.message)
  }
}

const handleCopyTs = async () => {
  if (!tsResult.value) {
    ElMessage.warning('请先导出 TS 接口')
    return
  }
  try {
    await navigator.clipboard.writeText(tsResult.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.warning('复制失败')
  }
}

// ============ 清空 ============
const handleClearSchema = () => {
  schemaText.value = ''
  schemaError.value = ''
  validateResult.value = null
  mockResult.value = ''
  tsResult.value = ''
  ElMessage.success('已清空')
}

// Schema 变化时清空旧的校验结果
watch(schemaText, () => {
  validateResult.value = null
  schemaError.value = ''
})

// ============ 自检 ============
const errors = selfCheck()
if (errors.length > 0) {
  console.warn('schemaUtils 自检失败:', errors)
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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
.card-actions { display: flex; align-items: center; gap: 8px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.code-input :deep(.el-textarea__inner) {
  font-family: 'JetBrains Mono', Consolas, 'Courier New', monospace;
  font-size: 13px;
  background: var(--bg-input);
  color: var(--text-primary);
}

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.schema-error {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: var(--danger-color, #ef4444);
  font-size: 13px;
  word-break: break-all;
}

.schema-tabs :deep(.el-tabs__header) {
  padding-left: 20px;
}
.schema-tabs :deep(.el-tabs__nav-wrap::after) {
  height: 1px;
  background: var(--border-color);
}

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.validate-result {
  margin-top: 12px;
  border-radius: 6px;
  padding: 12px 16px;
  font-size: 13px;
}
.validate-result.ok {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
}
.validate-result.fail {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.25);
}
.validate-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}
.validate-result.ok .validate-summary { color: var(--success-color, #10b981); }
.validate-result.fail .validate-summary { color: var(--danger-color, #ef4444); }
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.validate-result.ok .dot { background: #10b981; }
.validate-result.fail .dot { background: #ef4444; }

.issue-list {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 200px;
  overflow-y: auto;
}
.issue-item {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 4px 8px;
  background: var(--bg-input);
  border-radius: 4px;
}
.issue-path {
  font-family: 'JetBrains Mono', Consolas, monospace;
  color: var(--accent-cyan);
  white-space: nowrap;
}
.issue-keyword {
  font-size: 11px;
  padding: 0 6px;
  border-radius: 3px;
  background: rgba(139, 92, 246, 0.2);
  color: #8b5cf6;
  white-space: nowrap;
}
.issue-msg { color: var(--text-secondary); }
</style>
