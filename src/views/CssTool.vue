<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="css-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 颜色转换 -->
      <el-tab-pane label="颜色转换" name="color">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 支持 Hex、RGB、HSL 格式</p>
                    <p>• 自动识别输入格式并转换</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleColorConvert">转换</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('color')">清空</el-button>
              <el-button size="small" @click="handlePaste('color')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="tabState.color.input" type="textarea" :rows="4" placeholder="输入颜色值，如 #ff0000、rgb(255, 0, 0)、hsl(0, 100%, 50%)" resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('color')">复制</el-button>
          </div>
          <div class="card-body">
            <div v-if="colorResult" class="color-results">
              <div class="color-preview" :style="{ backgroundColor: colorResult.hex }"></div>
              <div class="color-item"><span class="color-label">HEX:</span><code>{{ colorResult.hex }}</code></div>
              <div class="color-item"><span class="color-label">RGB:</span><code>{{ colorResult.rgb }}</code></div>
              <div class="color-item"><span class="color-label">HSL:</span><code>{{ colorResult.hsl }}</code></div>
            </div>
            <div v-if="tabState.color.error" class="error-message">{{ tabState.color.error }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 单位换算 -->
      <el-tab-pane label="单位换算" name="unit">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">数值</div>
                <el-input-number v-model="unitValue" :precision="4" size="small" style="width: 120px" />
              </div>
              <div class="action-group">
                <div class="group-label">从</div>
                <el-select v-model="unitFrom" size="small" style="width: 80px">
                  <el-option label="px" value="px" />
                  <el-option label="rem" value="rem" />
                  <el-option label="em" value="em" />
                  <el-option label="vw" value="vw" />
                  <el-option label="vh" value="vh" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">到</div>
                <el-select v-model="unitTo" size="small" style="width: 80px">
                  <el-option label="px" value="px" />
                  <el-option label="rem" value="rem" />
                  <el-option label="em" value="em" />
                  <el-option label="vw" value="vw" />
                  <el-option label="vh" value="vh" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">基准字号</div>
                <el-input-number v-model="baseFontSize" :min="12" :max="32" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleUnitConvert">转换</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('unit')">复制</el-button>
          </div>
          <div class="card-body">
            <div v-if="unitResult !== null" class="unit-result">
              <span class="unit-value">{{ unitValue }}{{ unitFrom }}</span>
              <span class="unit-arrow">=</span>
              <span class="unit-value">{{ unitResult }}{{ unitTo }}</span>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: CSS压缩 -->
      <el-tab-pane label="CSS压缩" name="compress">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleCssCompress">压缩</el-button>
                  <el-button size="small" @click="handleCssFormat">格式化</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('compress')">清空</el-button>
              <el-button size="small" @click="handlePaste('compress')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="tabState.compress.input" type="textarea" :rows="8" placeholder="请输入 CSS 代码..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('compress')">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="tabState.compress.output" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': tabState.compress.isError }" />
            <div v-if="tabState.compress.error" class="error-message">{{ tabState.compress.error }}</div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import { parseColor, convertUnit, compressCss, formatCss } from '@/utils/cssUtils'

const store = useToolboxStore()

const activeTab = ref('color')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  color: { input: '', output: '', error: '', isError: false },
  unit: { input: '', output: '', error: '', isError: false },
  compress: { input: '', output: '', error: '', isError: false }
})

const colorResult = ref<{ hex: string; rgb: string; hsl: string } | null>(null)

const unitValue = ref(16)
const unitFrom = ref('px')
const unitTo = ref('rem')
const baseFontSize = ref(16)
const unitResult = ref<number | null>(null)

const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'color') colorResult.value = null
  if (tab === 'unit') unitResult.value = null
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
  const text = tab === 'color'
    ? (colorResult.value ? `HEX: ${colorResult.value.hex}\nRGB: ${colorResult.value.rgb}\nHSL: ${colorResult.value.hsl}` : '')
    : tab === 'unit'
    ? (unitResult.value !== null ? `${unitValue.value}${unitFrom.value} = ${unitResult.value}${unitTo.value}` : '')
    : tabState[tab].output

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

const handleColorConvert = () => {
  const input = tabState.color.input.trim()
  if (!input) {
    ElMessage.warning('请输入颜色值')
    return
  }

  const result = parseColor(input)
  if (result) {
    colorResult.value = result
    tabState.color.error = ''
    tabState.color.isError = false
    store.addHistory({ tool: 'css', action: '颜色转换', inputPreview: input, outputPreview: result.hex, inputFull: input, outputFull: JSON.stringify(result, null, 2) })
    ElMessage.success('转换成功')
  } else {
    colorResult.value = null
    tabState.color.error = '无法识别的颜色格式'
    tabState.color.isError = true
    ElMessage.error('无法识别的颜色格式')
  }
}

const handleUnitConvert = () => {
  const result = convertUnit(unitValue.value, unitFrom.value, unitTo.value, baseFontSize.value)
  if (result !== null) {
    unitResult.value = parseFloat(result.toFixed(4))
    ElMessage.success('转换成功')
  } else {
    unitResult.value = null
    ElMessage.error('不支持的单位')
  }
}

const handleCssCompress = () => {
  const input = tabState.compress.input.trim()
  if (!input) {
    ElMessage.warning('请输入 CSS 代码')
    return
  }
  tabState.compress.output = compressCss(input)
  tabState.compress.error = ''
  tabState.compress.isError = false
  store.addHistory({ tool: 'css', action: 'CSS压缩', inputPreview: input.slice(0, 50), outputPreview: tabState.compress.output.slice(0, 50), inputFull: input, outputFull: tabState.compress.output })
  ElMessage.success('压缩完成')
}

const handleCssFormat = () => {
  const input = tabState.compress.input.trim()
  if (!input) {
    ElMessage.warning('请输入 CSS 代码')
    return
  }
  tabState.compress.output = formatCss(input)
  tabState.compress.error = ''
  tabState.compress.isError = false
  store.addHistory({ tool: 'css', action: 'CSS格式化', inputPreview: input.slice(0, 50), outputPreview: tabState.compress.output.slice(0, 50), inputFull: input, outputFull: tabState.compress.output })
  ElMessage.success('格式化完成')
}

// 参数变化时清空旧结果，避免误导
watch([unitValue, unitFrom, unitTo, baseFontSize], () => {
  unitResult.value = null
})
</script>

<style scoped>
.css-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .css-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.css-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.css-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.css-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.css-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.css-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

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
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

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

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

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

.color-results { display: flex; flex-direction: column; gap: 12px; }
.color-preview { height: 60px; border-radius: 8px; border: 1px solid var(--border-color); }
.color-item { display: flex; align-items: center; gap: 8px; }
.color-label { font-weight: 600; color: var(--accent-cyan); min-width: 50px; }
.color-item code {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-input);
  padding: 4px 8px;
  border-radius: 4px;
}

.unit-result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 20px 0;
  font-size: 24px;
}
.unit-value { font-weight: 600; color: var(--text-primary); }
.unit-arrow { color: var(--accent-cyan); }
</style>
