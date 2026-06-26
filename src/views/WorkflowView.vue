<template>
  <div class="tool-container">
    <!-- 工作流列表 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">工作流</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>编排多个处理步骤，一键执行连续转换</p>
                <p>• 支持字符串处理、JSON格式化、编码转换等</p>
                <p>• 上一步输出自动作为下一步输入</p>
                <p>• 可保存常用流程，随时调用</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button type="primary" size="small" @click="handleNewWorkflow">新建工作流</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="workflows.length === 0" class="empty-state">
          暂无工作流，点击上方"新建工作流"创建
        </div>
        <div v-else class="workflow-list">
          <div
            v-for="wf in workflows"
            :key="wf.id"
            class="workflow-item"
            :class="{ active: selectedWorkflow?.id === wf.id }"
          >
            <div class="workflow-item-header">
              <span class="workflow-item-name">{{ wf.name }}</span>
              <div class="workflow-item-actions">
                <el-button size="small" type="primary" @click.stop="handleRunWorkflow(wf)">执行</el-button>
                <el-button size="small" @click.stop="handleEditWorkflow(wf)">编辑</el-button>
                <el-button size="small" type="danger" @click.stop="handleDeleteWorkflow(wf)">删除</el-button>
              </div>
            </div>
            <div class="workflow-item-desc">{{ wf.description || '暂无描述' }}</div>
            <div class="workflow-item-steps">
              <span class="step-badge" v-for="(step, i) in parseSteps(wf.steps_json)" :key="i">
                {{ step.tool }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 工作流编辑区 -->
    <div v-if="editingWorkflow" class="tool-card">
      <div class="card-header">
        <span class="card-title">编辑工作流</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCancelEdit">取消</el-button>
          <el-button type="primary" size="small" @click="handleSaveWorkflow">保存</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="edit-form">
          <div class="form-row">
            <label>名称</label>
            <el-input v-model="editingWorkflow.name" placeholder="工作流名称" size="small" />
          </div>
          <div class="form-row">
            <label>描述</label>
            <el-input v-model="editingWorkflow.description" placeholder="可选描述" size="small" />
          </div>
        </div>

        <div class="steps-editor">
          <div class="steps-header">
            <span class="card-title" style="font-size: 13px;">步骤编排</span>
            <el-button type="primary" size="small" @click="handleAddStep">添加步骤</el-button>
          </div>

          <div
            v-for="(step, index) in editingSteps"
            :key="index"
            class="step-item"
          >
            <div class="step-number">{{ index + 1 }}</div>
            <div class="step-content">
              <div class="step-row">
                <el-select v-model="step.tool" placeholder="选择工具" size="small" style="width: 140px">
                  <el-option label="字符串处理" value="string" />
                  <el-option label="JSON格式化" value="json" />
                  <el-option label="编码转换" value="encode" />
                  <el-option label="正则替换" value="regex" />
                  <el-option label="SQL转换" value="sql" />
                  <el-option label="Base64编解码" value="base64" />
                </el-select>
                <el-select v-model="step.action" placeholder="选择操作" size="small" style="width: 140px">
                  <el-option
                    v-for="action in getActionsForTool(step.tool)"
                    :key="action"
                    :label="action"
                    :value="action"
                  />
                </el-select>
                <el-select v-model="step.input" placeholder="输入来源" size="small" style="width: 120px">
                  <el-option v-if="index === 0" label="执行输入" value="exec_input" />
                  <el-option v-if="index > 0" label="上一步输出" value="prev_output" />
                  <el-option label="手动输入" value="manual" />
                  <el-option label="变量池" value="variable" />
                </el-select>
                <el-input
                  v-if="step.input === 'manual'"
                  v-model="step.manualInput"
                  placeholder="输入内容"
                  size="small"
                  class="step-manual-input"
                />
                <el-select
                  v-else-if="step.input === 'variable'"
                  v-model="step.variableName"
                  placeholder="选择变量"
                  size="small"
                  class="step-var-input"
                >
                  <el-option
                    v-for="v in variables"
                    :key="v.id"
                    :label="v.name"
                    :value="v.name"
                  />
                </el-select>
              </div>
            </div>
            <el-button
              size="small"
              type="danger"
              :icon="null"
              @click="handleRemoveStep(index)"
              style="margin-left: 8px; flex-shrink: 0;"
            >
              ×
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 执行输入 -->
    <div v-if="executingWorkflow" class="tool-card">
      <div class="card-header">
        <span class="card-title">执行输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClearExecInput">清空</el-button>
          <el-button size="small" @click="handlePasteExecInput">粘贴</el-button>
          <el-button type="primary" size="small" @click="handleExecute">执行</el-button>
          <el-button size="small" @click="handleCloseExec">关闭</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="execInput"
          type="textarea"
          :rows="6"
          placeholder="输入初始数据..."
          resize="vertical"
        />
      </div>
    </div>

    <!-- 执行输出 -->
    <div v-if="executingWorkflow && (execLoading || execOutput || execError)" class="tool-card">
      <div class="card-header">
        <span class="card-title">执行结果</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCopyExecOutput">复制</el-button>
          <el-button size="small" @click="handleCloseExec">关闭</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="execLoading" class="exec-progress">
          <el-progress :percentage="execProgress" :stroke-width="8" />
          <div class="exec-step-text">正在执行第 {{ execCurrentStep + 1 }}/{{ executingWorkflow.steps.length }} 步: {{ executingWorkflow.steps[execCurrentStep]?.tool }}</div>
        </div>
        <el-input
          v-else
          :model-value="execOutput"
          type="textarea"
          :rows="8"
          readonly
          placeholder="执行结果将显示在这里..."
        />
        <div v-if="execError" class="error-message">{{ execError }}</div>
      </div>
    </div>

    <!-- 变量池面板 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">变量池</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>跨工具共享的临时变量存储</p>
                <p>• 手动命名并保存变量值</p>
                <p>• 工作流步骤可引用变量池中的数据</p>
                <p>• 在步骤编排中选择"变量池"作为输入来源</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button type="primary" size="small" @click="handleAddVariable">添加变量</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="variables.length === 0" class="empty-state">
          暂无变量
        </div>
        <div v-else class="variable-list">
          <div v-for="v in variables" :key="v.id" class="variable-item">
            <div class="variable-info">
              <span class="variable-name">{{ v.name }}</span>
              <span class="variable-value">{{ truncate(v.value, 50) }}</span>
              <span class="variable-source" :class="v.source">{{ v.source === 'auto' ? '自动缓存' : '手动' }}</span>
            </div>
            <div class="variable-actions">
              <el-button size="small" @click="handleCopyVariable(v.value)">复制</el-button>
              <el-button size="small" type="danger" @click="handleDeleteVariable(v.name)">删除</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加变量对话框 -->
    <el-dialog v-model="showAddVariable" title="添加变量" width="400px">
      <div class="form-row">
        <label>变量名</label>
        <el-input v-model="newVariable.name" placeholder="变量名称" size="small" />
      </div>
      <div class="form-row">
        <label>变量值</label>
        <el-input v-model="newVariable.value" type="textarea" :rows="4" placeholder="变量值" size="small" />
      </div>
      <template #footer>
        <el-button size="small" @click="showAddVariable = false">取消</el-button>
        <el-button type="primary" size="small" @click="handleConfirmAddVariable">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import * as db from '@/utils/dbClient'
// 复用已有工具函数
import {
  toUpperCase, toLowerCase, toCamelCase, toSnakeCase,
  removeEmptyLines, removeDuplicates, reverseLines, sortLines,
  trimLeadingTrailing,
} from '@/utils/stringUtils'
import { formatJson, compressJson, validateJson } from '@/utils/jsonUtils'
import { urlEncode, urlDecode, base64Encode, base64Decode } from '@/utils/encodeUtils'
import { convertToSqlIn } from '@/utils/sqlUtils'

// 工作流列表
const workflows = ref<db.Workflow[]>([])
const selectedWorkflow = ref<db.Workflow | null>(null)
const editingWorkflow = ref<db.Workflow | null>(null)
const editingSteps = ref<WorkflowStep[]>([])

// 执行状态
const executingWorkflow = ref<WorkflowWithSteps | null>(null)
const execInput = ref('')
const execOutput = ref('')
const execError = ref('')
const execLoading = ref(false)
const execProgress = ref(0)
const execCurrentStep = ref(0)

// 变量池
const variables = ref<db.PoolVariable[]>([])
const showAddVariable = ref(false)
const newVariable = ref({ name: '', value: '' })

// 类型定义
interface WorkflowStep {
  tool: string
  action: string
  input: string
  manualInput: string
  variableName: string
}

interface WorkflowWithSteps extends db.Workflow {
  steps: WorkflowStep[]
}

// 工具操作映射
const TOOL_ACTIONS: Record<string, string[]> = {
  string: [
    '转大写', '转小写', '去除首尾空格', '去除空行', '行号排序', '去重', '反转',
    '转驼峰命名', '转下划线命名', 'Base64编码', 'Base64解码',
  ],
  json: ['格式化', '压缩', '校验'],
  encode: ['Base64编码', 'Base64解码', 'URL编码', 'URL解码'],
  regex: ['正则匹配', '正则替换'],
  sql: ['转SQL IN', '转SQL VALUES'],
  base64: ['编码', '解码'],
}

function getActionsForTool(tool: string): string[] {
  return TOOL_ACTIONS[tool] || []
}

function parseSteps(stepsJson: string): WorkflowStep[] {
  try {
    return JSON.parse(stepsJson)
  } catch {
    return []
  }
}

function truncate(str: string, len: number): string {
  return str.length > len ? str.slice(0, len) + '...' : str
}

// 加载工作流列表
async function loadWorkflows() {
  try {
    workflows.value = await db.listWorkflows()
  } catch (e: any) {
    ElMessage.error('加载工作流失败: ' + (e.message || e))
  }
}

// 加载变量池
async function loadVariables() {
  try {
    variables.value = await db.listVariables()
  } catch (e: any) {
    ElMessage.error('加载变量池失败: ' + (e.message || e))
  }
}

// 新建工作流
function handleNewWorkflow() {
  const now = new Date().toISOString()
  editingWorkflow.value = {
    id: 'wf_' + Date.now(),
    name: '新工作流',
    description: '',
    steps_json: '[]',
    created_at: now,
    updated_at: now,
  }
  editingSteps.value = []
  selectedWorkflow.value = null
}

// 选择工作流（仅高亮，不进入编辑）
// 编辑工作流
function handleEditWorkflow(wf: db.Workflow) {
  selectedWorkflow.value = wf
  editingWorkflow.value = { ...wf }
  editingSteps.value = parseSteps(wf.steps_json)
}

// 取消编辑
function handleCancelEdit() {
  editingWorkflow.value = null
  editingSteps.value = []
}

// 添加步骤
function handleAddStep() {
  const isFirst = editingSteps.value.length === 0
  editingSteps.value.push({
    tool: '',
    action: '',
    input: isFirst ? 'exec_input' : 'prev_output',
    manualInput: '',
    variableName: '',
  })
}

// 删除步骤
function handleRemoveStep(index: number) {
  editingSteps.value.splice(index, 1)
}

// 保存工作流
async function handleSaveWorkflow() {
  if (!editingWorkflow.value) return
  if (editingSteps.value.length === 0) {
    ElMessage.warning('请至少添加一个步骤')
    return
  }

  const wf = {
    ...editingWorkflow.value,
    steps_json: JSON.stringify(editingSteps.value),
    updated_at: new Date().toISOString(),
  }

  try {
    await db.saveWorkflow(wf)
    ElMessage.success('保存成功')
    editingWorkflow.value = null
    editingSteps.value = []
    await loadWorkflows()
  } catch (e: any) {
    ElMessage.error('保存失败: ' + (e.message || e))
  }
}

// 删除工作流
async function handleDeleteWorkflow(wf: db.Workflow) {
  try {
    await ElMessageBox.confirm(`确定删除工作流 "${wf.name}"？`, '确认删除', { type: 'warning' })
    await db.deleteWorkflow(wf.id)
    ElMessage.success('已删除')
    if (editingWorkflow.value?.id === wf.id) {
      editingWorkflow.value = null
      editingSteps.value = []
    }
    await loadWorkflows()
  } catch (e: any) {
    if (e !== 'cancel') {
      ElMessage.error('删除失败: ' + (e.message || e))
    }
  }
}

// 执行工作流
function handleRunWorkflow(wf: db.Workflow) {
  selectedWorkflow.value = wf
  editingWorkflow.value = null
  editingSteps.value = []
  executingWorkflow.value = {
    ...wf,
    steps: parseSteps(wf.steps_json),
  }
  execInput.value = ''
  execOutput.value = ''
  execError.value = ''
  execLoading.value = false
  execProgress.value = 0
  execCurrentStep.value = 0
}

// 清空执行输入
function handleClearExecInput() {
  execInput.value = ''
}

// 关闭执行面板
function handleCloseExec() {
  executingWorkflow.value = null
  execInput.value = ''
  execOutput.value = ''
  execError.value = ''
  execLoading.value = false
  execProgress.value = 0
  execCurrentStep.value = 0
}

// 粘贴执行输入
async function handlePasteExecInput() {
  try {
    execInput.value = await navigator.clipboard.readText()
  } catch (e: any) {
    ElMessage.error('粘贴失败: ' + (e.message || e))
  }
}

// 复制执行输出
async function handleCopyExecOutput() {
  try {
    await navigator.clipboard.writeText(execOutput.value)
    ElMessage.success('已复制到剪贴板')
  } catch (e: any) {
    ElMessage.error('复制失败: ' + (e.message || e))
  }
}

// 执行工作流
async function handleExecute() {
  if (!executingWorkflow.value) return
  if (!execInput.value.trim()) {
    ElMessage.warning('请输入初始数据')
    return
  }

  execLoading.value = true
  execProgress.value = 0
  execCurrentStep.value = 0
  execOutput.value = ''
  execError.value = ''

  const steps = executingWorkflow.value.steps
  let currentInput = execInput.value

  try {
    for (let i = 0; i < steps.length; i++) {
      execCurrentStep.value = i
      execProgress.value = Math.round(((i + 1) / steps.length) * 100)

      const step = steps[i]
      
      // 获取输入
      let input = currentInput
      if (step.input === 'variable') {
        try {
          input = await db.getVariable(step.variableName)
        } catch {
          input = ''
        }
      } else if (step.input === 'manual') {
        input = step.manualInput || currentInput
      } else if (step.input === 'exec_input') {
        // 第一步使用执行输入框的内容
        input = execInput.value || currentInput
      } else if (step.input === 'prev_output') {
        // 后续步骤使用上一步输出
        input = currentInput
      }

      // 执行操作
      const result = await executeStep(step.tool, step.action, input)
      currentInput = result
    }

    execOutput.value = currentInput
    ElMessage.success('执行完成')
  } catch (e: any) {
    execError.value = e.message || '执行失败'
    ElMessage.error('执行失败: ' + execError.value)
  } finally {
    execLoading.value = false
  }
}

// 执行单个步骤
async function executeStep(tool: string, action: string, input: string): Promise<string> {
  switch (tool) {
    case 'string':
      return executeStringAction(action, input)
    case 'json':
      return executeJsonAction(action, input)
    case 'encode':
      return executeEncodeAction(action, input)
    case 'base64':
      return executeBase64Action(action, input)
    case 'regex':
      return executeRegexAction(action, input)
    case 'sql':
      return executeSqlAction(action, input)
    default:
      return input
  }
}

// 字符串处理 — 复用 stringUtils
function executeStringAction(action: string, input: string): string {
  switch (action) {
    case '转大写': return toUpperCase(input)
    case '转小写': return toLowerCase(input)
    case '去除首尾空格': return trimLeadingTrailing(input)
    case '去除空行': return removeEmptyLines(input)
    case '行号排序': return sortLines(input)
    case '去重': return removeDuplicates(input)
    case '反转': return reverseLines(input)
    case '转驼峰命名': return toCamelCase(input)
    case '转下划线命名': return toSnakeCase(input)
    case 'Base64编码': return base64Encode(input)
    case 'Base64解码': return base64Decode(input)
    default: return input
  }
}

// JSON处理 — 复用 jsonUtils
function executeJsonAction(action: string, input: string): string {
  switch (action) {
    case '格式化': {
      const r = formatJson(input)
      return r.success ? r.data! : r.error!
    }
    case '压缩': {
      const r = compressJson(input)
      return r.success ? r.data! : r.error!
    }
    case '校验': {
      const r = validateJson(input)
      return r.success ? 'JSON 格式正确' : r.error!
    }
    default: return input
  }
}

// 编码转换 — 复用 encodeUtils
function executeEncodeAction(action: string, input: string): string {
  switch (action) {
    case 'Base64编码': return base64Encode(input)
    case 'Base64解码': return base64Decode(input)
    case 'URL编码': return urlEncode(input)
    case 'URL解码': return urlDecode(input)
    default: return input
  }
}

// Base64编解码 — 复用 encodeUtils
function executeBase64Action(action: string, input: string): string {
  switch (action) {
    case '编码': return base64Encode(input)
    case '解码': return base64Decode(input)
    default: return input
  }
}

// 正则处理 — 复用 regexUtils
function executeRegexAction(_action: string, input: string): string {
  // 工作流中暂不支持正则（需要额外配置 pattern/flags）
  return input
}

// SQL处理 — 复用 sqlUtils
function executeSqlAction(action: string, input: string): string {
  switch (action) {
    case '转SQL IN': return convertToSqlIn(input, 'single')
    case '转SQL VALUES': {
      const items = input.split('\n').map(s => s.trim()).filter(Boolean)
      return items.map(s => `('${s}')`).join(',\n')
    }
    default: return input
  }
}

// 添加变量
function handleAddVariable() {
  newVariable.value = { name: '', value: '' }
  showAddVariable.value = true
}

// 确认添加变量
async function handleConfirmAddVariable() {
  if (!newVariable.value.name.trim()) {
    ElMessage.warning('请输入变量名')
    return
  }
  try {
    await db.setVariable(newVariable.value.name, newVariable.value.value, 'manual')
    ElMessage.success('变量已保存')
    showAddVariable.value = false
    await loadVariables()
  } catch (e: any) {
    ElMessage.error('保存失败: ' + (e.message || e))
  }
}

// 删除变量
async function handleDeleteVariable(name: string) {
  try {
    await db.deleteVariable(name)
    ElMessage.success('已删除')
    await loadVariables()
  } catch (e: any) {
    ElMessage.error('删除失败: ' + (e.message || e))
  }
}

// 复制变量值
async function handleCopyVariable(value: string) {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success('已复制到剪贴板')
  } catch (e: any) {
    ElMessage.error('复制失败: ' + (e.message || e))
  }
}

onMounted(() => {
  loadWorkflows()
  loadVariables()
})
</script>

<style scoped>
/* 工具卡片 */
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

/* 标题栏 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
  gap: 12px;
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
  white-space: nowrap;
}
.card-body { padding: 16px 20px; }

/* 操作按钮 */
.card-actions { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

/* 提示图标 */
.hint-icon { font-size: 15px; color: var(--text-secondary); cursor: pointer; transition: color 0.2s; flex-shrink: 0; }
.hint-icon:hover { color: var(--accent-cyan); }
.header-left { display: flex; align-items: center; gap: 8px; }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }
.tooltip-content code { background: rgba(0, 212, 255, 0.1); padding: 1px 4px; border-radius: 3px; font-size: 12px; }

/* 工作流列表 */
.workflow-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.workflow-item {
  padding: 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}
.workflow-item:hover {
  border-color: rgba(0, 212, 255, 0.3);
}
.workflow-item.active {
  border-color: var(--accent-cyan);
  box-shadow: 0 0 0 1px var(--accent-cyan);
}
.workflow-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.workflow-item-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}
.workflow-item-actions {
  display: flex;
  gap: 8px;
}
.workflow-item-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 10px;
}
.workflow-item-steps {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.step-badge {
  font-size: 12px;
  padding: 2px 8px;
  background: rgba(0, 212, 255, 0.1);
  color: var(--accent-cyan);
  border-radius: 4px;
}

/* 编辑表单 */
.edit-form {
  margin-bottom: 16px;
}
.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}
.form-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  min-width: 60px;
  text-align: right;
}
.form-row .el-input {
  flex: 1;
}

/* 步骤编辑器 */
.steps-editor {
  margin-top: 16px;
}
.steps-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.step-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  margin-bottom: 12px;
}
.step-number {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
}
.step-content {
  flex: 1;
  min-width: 0;
}
.step-row {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.step-manual-input {
  flex: 1;
  min-width: 200px;
}
.step-var-input {
  width: 140px;
}

/* 执行进度 */
.exec-progress {
  margin-bottom: 12px;
}
.exec-step-text {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 8px;
}

/* 变量池 */
.variable-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.variable-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}
.variable-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}
.variable-name {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  flex-shrink: 0;
}
.variable-value {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.variable-source {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  flex-shrink: 0;
}
.variable-source.auto {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
}
.variable-source.manual {
  background: rgba(0, 212, 255, 0.1);
  color: var(--accent-cyan);
}
.variable-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
  margin-left: 12px;
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 24px;
  color: var(--text-secondary);
  font-size: 13px;
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

/* 对话框表单 */
:deep(.el-dialog__body) {
  padding: 20px;
}
:deep(.el-dialog__footer) {
  padding: 12px 20px;
}
</style>
