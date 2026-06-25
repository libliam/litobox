<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="js-tabs">
      <!-- Tab 1: JS 沙箱运行 -->
      <el-tab-pane label="JS 沙箱运行" name="sandbox">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="executeJs">执行代码</el-button>
                  <el-button size="small" @click="loadExample">加载示例</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">输入参数 (JSON)</span>
              <el-tooltip content="输入的 JSON 对象会作为全局变量 input 注入到 JS 执行环境中。例如输入 {&quot;name&quot;: &quot;world&quot;}，代码中可通过 input.name 访问。" placement="top">
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="clearSandboxInput">清空</el-button>
              <el-button size="small" @click="pasteToSandboxInput">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="sandboxState.input"
              type="textarea"
              :rows="4"
              placeholder='输入参数，JSON 格式，例如：{"name": "world"}'
              resize="vertical"
            />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">JS 代码</span>
              <el-tooltip placement="top">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 代码在沙箱中执行，不支持 <code>return</code> 语句（顶层执行）</p>
                    <p>• 支持 <code>console.log()</code>、<code>console.warn()</code>、<code>console.error()</code></p>
                    <p>• 最后一条表达式的值将作为返回结果</p>
                    <p>• 超时时间 5 秒，超时将终止执行</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="clearSandboxCode">清空</el-button>
              <el-button size="small" @click="pasteToSandboxCode">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="sandboxState.code"
              type="textarea"
              :rows="8"
              placeholder="输入 JavaScript 代码..."
              resize="vertical"
            />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">执行日志</span>
            <el-button size="small" @click="copySandboxOutput">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="sandboxState.output"
              type="textarea"
              :rows="6"
              readonly
              resize="vertical"
              :class="{ 'error': sandboxState.isError }"
            />
            <div v-if="sandboxState.error" class="error-message">
              {{ sandboxState.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: JS 格式化 -->
      <el-tab-pane label="JS 格式化" name="format">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleFormat">格式化</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="clearFormatInput">清空</el-button>
              <el-button size="small" @click="pasteToFormatInput">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="formatState.input"
              type="textarea"
              :rows="10"
              placeholder="请输入压缩的 JavaScript 代码..."
              resize="vertical"
            />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="copyFormatOutput">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="formatState.output"
              type="textarea"
              :rows="10"
              readonly
              resize="vertical"
              :class="{ 'error': formatState.isError }"
            />
            <div v-if="formatState.error" class="error-message">
              {{ formatState.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: JS 压缩混淆 -->
      <el-tab-pane label="JS 压缩混淆" name="compress">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleCompress">压缩</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="clearCompressInput">清空</el-button>
              <el-button size="small" @click="pasteToCompressInput">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="compressState.input"
              type="textarea"
              :rows="10"
              placeholder="请输入 JavaScript 代码..."
              resize="vertical"
            />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="copyCompressOutput">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="compressState.output"
              type="textarea"
              :rows="6"
              readonly
              resize="vertical"
              :class="{ 'error': compressState.isError }"
            />
            <div v-if="compressState.error" class="error-message">
              {{ compressState.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: JSON 提取代码 -->
      <el-tab-pane label="JSON 提取代码" name="extract">
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">提取路径</div>
                <el-input
                  v-model="extractState.path"
                  placeholder="例如：user.address.city"
                  size="small"
                  style="width: 200px"
                  clearable
                />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleGenerateExtract">生成提取代码</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">JSON 数据</span>
            <div class="card-actions">
              <el-button size="small" @click="clearExtractInput">清空</el-button>
              <el-button size="small" @click="pasteToExtractInput">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="extractState.jsonInput"
              type="textarea"
              :rows="8"
              placeholder="请输入 JSON 数据..."
              resize="vertical"
            />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">生成的代码</span>
            <el-button size="small" @click="copyExtractOutput">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="extractState.output"
              type="textarea"
              :rows="8"
              readonly
              resize="vertical"
              :class="{ 'error': extractState.isError }"
            />
            <div v-if="extractState.error" class="error-message">
              {{ extractState.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { formatJsCode, compressJsCode, generateJsonExtractCode } from '@/utils/jsUtils'

const activeTab = ref('sandbox')

// Tab 1: 沙箱运行状态
const sandboxState = reactive({
  input: '',
  code: '',
  output: '',
  error: '',
  isError: false
})

// Tab 2: 格式化状态
const formatState = reactive({
  input: '',
  output: '',
  error: '',
  isError: false
})

// Tab 3: 压缩状态
const compressState = reactive({
  input: '',
  output: '',
  error: '',
  isError: false
})

// Tab 4: 提取代码生成状态
const extractState = reactive({
  jsonInput: '',
  path: '',
  output: '',
  error: '',
  isError: false
})

// ============ 沙箱运行相关 ============
const executeJs = async () => {
  if (!sandboxState.code.trim()) {
    ElMessage.warning('请输入 JavaScript 代码')
    return
  }

  let params = {}
  if (sandboxState.input.trim()) {
    try {
      params = JSON.parse(sandboxState.input)
    } catch {
      sandboxState.error = '输入参数格式错误，请输入有效的 JSON'
      sandboxState.isError = true
      return
    }
  }

  try {
    sandboxState.output = ''
    sandboxState.error = ''
    sandboxState.isError = false

    const result = await invoke<any>('execute_js', {
      code: sandboxState.code,
      input: JSON.stringify(params),
      timeoutMs: 5000
    })

    const logs = result.logs || []
    const logStr = logs.map((l: any) => `[${l.level}] ${l.message}`).join('\n')
    
    if (result.success) {
      sandboxState.output = logStr ? `${logStr}\n\n返回结果:\n${result.result}` : (result.result || '执行成功，无返回值')
    } else {
      sandboxState.error = result.error || '执行失败'
      sandboxState.isError = true
      if (logStr) {
        sandboxState.output = logStr
      }
    }
  } catch (err: any) {
    sandboxState.error = `执行错误: ${err}`
    sandboxState.isError = true
  }
}

const loadExample = () => {
  sandboxState.code = `// 示例代码
const greeting = \`Hello, \${input.name || 'World'}!\`;
console.log(greeting);
greeting;`
  sandboxState.input = '{"name": "LitoBox"}'
}

const clearSandboxInput = () => { sandboxState.input = '' }
const clearSandboxCode = () => { sandboxState.code = '' }
const pasteToSandboxInput = async () => {
  try {
    const text = await navigator.clipboard.readText()
    sandboxState.input = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
const pasteToSandboxCode = async () => {
  try {
    const text = await navigator.clipboard.readText()
    sandboxState.code = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
const copySandboxOutput = async () => {
  const text = sandboxState.output || sandboxState.error
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

// ============ 格式化相关 ============
const handleFormat = () => {
  if (!formatState.input.trim()) {
    ElMessage.warning('请输入 JavaScript 代码')
    return
  }

  const result = formatJsCode(formatState.input)
  if (result.success) {
    formatState.output = result.data || ''
    formatState.error = ''
    formatState.isError = false
  } else {
    formatState.output = ''
    formatState.error = `错误: ${result.error}`
    formatState.isError = true
  }
}

const clearFormatInput = () => { formatState.input = '' }
const pasteToFormatInput = async () => {
  try {
    const text = await navigator.clipboard.readText()
    formatState.input = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
const copyFormatOutput = async () => {
  const text = formatState.output
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

// ============ 压缩相关 ============
const handleCompress = () => {
  if (!compressState.input.trim()) {
    ElMessage.warning('请输入 JavaScript 代码')
    return
  }

  const result = compressJsCode(compressState.input)
  if (result.success) {
    compressState.output = result.data || ''
    compressState.error = ''
    compressState.isError = false
  } else {
    compressState.output = ''
    compressState.error = `错误: ${result.error}`
    compressState.isError = true
  }
}

const clearCompressInput = () => { compressState.input = '' }
const pasteToCompressInput = async () => {
  try {
    const text = await navigator.clipboard.readText()
    compressState.input = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
const copyCompressOutput = async () => {
  const text = compressState.output
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

// ============ JSON 提取代码生成相关 ============
const handleGenerateExtract = () => {
  if (!extractState.jsonInput.trim()) {
    ElMessage.warning('请输入 JSON 数据')
    return
  }
  if (!extractState.path.trim()) {
    ElMessage.warning('请输入提取路径')
    return
  }

  const result = generateJsonExtractCode(extractState.jsonInput, extractState.path)
  if (result.success) {
    extractState.output = result.data || ''
    extractState.error = ''
    extractState.isError = false
  } else {
    extractState.output = ''
    extractState.error = `错误: ${result.error}`
    extractState.isError = true
  }
}

const clearExtractInput = () => {
  extractState.jsonInput = ''
  extractState.path = ''
}
const pasteToExtractInput = async () => {
  try {
    const text = await navigator.clipboard.readText()
    extractState.jsonInput = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
const copyExtractOutput = async () => {
  const text = extractState.output
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
.js-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .js-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.js-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.js-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

.js-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
}

.js-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.js-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

/* 工具卡片 */
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

/* 标题栏布局：标题在左，操作按钮在右 */
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

/* 操作区域布局 */
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
