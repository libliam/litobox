<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="xml-yaml-tabs">
      <!-- XML Tab -->
      <el-tab-pane label="XML 工具" name="xml">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 格式化：美化 XML 缩进</p>
                    <p>• 压缩：移除多余空白</p>
                    <p>• 校验：检查 XML 语法</p>
                    <p>• XML↔JSON 互转</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">缩进</div>
                <el-radio-group v-model="xmlIndent" size="small">
                  <el-radio-button :label="2">2空格</el-radio-button>
                  <el-radio-button :label="4">4空格</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleXmlFormat">格式化</el-button>
                  <el-button size="small" @click="handleXmlCompress">压缩</el-button>
                  <el-button type="warning" size="small" @click="handleXmlValidate">校验</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleXmlToJson">XML→JSON</el-button>
                  <el-button size="small" @click="handleJsonToXml">JSON→XML</el-button>
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
            <el-input v-model="xmlInput" type="textarea" :rows="8" placeholder="请输入 XML 或 JSON 内容..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="xmlOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': xmlIsError }" />
            <div v-if="xmlError" class="error-message">{{ xmlError }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- YAML Tab -->
      <el-tab-pane label="YAML 工具" name="yaml">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 格式化：标准化 YAML 格式</p>
                    <p>• 校验：检查 YAML 语法</p>
                    <p>• YAML↔JSON 互转</p>
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
                  <el-button type="primary" size="small" @click="handleYamlFormat">格式化</el-button>
                  <el-button type="warning" size="small" @click="handleYamlValidate">校验</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleYamlToJson">YAML→JSON</el-button>
                  <el-button size="small" @click="handleJsonToYaml">JSON→YAML</el-button>
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
              <el-button size="small" @click="handleYamlClear">清空</el-button>
              <el-button size="small" @click="handleYamlPaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="yamlInput" type="textarea" :rows="8" placeholder="请输入 YAML 或 JSON 内容..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleYamlCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="yamlOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': yamlIsError }" />
            <div v-if="yamlError" class="error-message">{{ yamlError }}</div>
          </div>
        </div>
      </el-tab-pane>
      <!-- 配置格式互转 Tab -->
      <el-tab-pane label="配置互转" name="convert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">格式</span>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>支持 JSON / YAML / TOML / INI / Properties 5 种格式环形互转</p>
                    <p>统一以 JS 对象为中间层：源格式 → 对象 → 目标格式</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleConvert">转换</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">源格式</div>
                <el-select v-model="convSrc" size="small" style="width: 110px">
                  <el-option v-for="f in CONFIG_FORMATS" :key="f.value" :label="f.label" :value="f.value" />
                </el-select>
              </div>
              <el-icon class="arrow-icon"><Right /></el-icon>
              <div class="action-group">
                <div class="group-label">目标格式</div>
                <el-select v-model="convDst" size="small" style="width: 110px">
                  <el-option v-for="f in CONFIG_FORMATS" :key="f.value" :label="f.label" :value="f.value" />
                </el-select>
              </div>
              <!-- 交换与下拉框同一行（功能不同，不与转换按钮并排） -->
              <el-button size="small" @click="swapFormat">交换</el-button>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 ({{ convSrcLabel }})</span>
            <div class="card-actions">
              <VariablePicker @select="convInput += `{{${$event}}}`" />
              <el-button size="small" @click="convInput = ''; convOutput = ''; convError = ''">清空</el-button>
              <el-button size="small" @click="handleConvPaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="convInput"
              type="textarea"
              :rows="8"
              :placeholder="convPlaceholder"
              resize="vertical"
            />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出 ({{ convDstLabel }})</span>
            <el-button size="small" @click="handleConvCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="convOutput"
              type="textarea"
              :rows="8"
              readonly
              resize="vertical"
              :class="{ 'error': convIsError }"
            />
            <div v-if="convError" class="error-message">{{ convError }}</div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Right } from '@element-plus/icons-vue'
import {
  formatXml, validateXml, xmlToJson, jsonToXml, parseYaml, jsonToYaml,
  parseConfig, stringifyConfig, type ConfigFormat,
} from '@/utils/xmlYamlUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const activeTab = ref('xml')

// XML 状态
const xmlInput = ref('')
const xmlOutput = ref('')
const xmlError = ref('')
const xmlIsError = ref(false)
const xmlIndent = ref(2)

// YAML 状态
const yamlInput = ref('')
const yamlOutput = ref('')
const yamlError = ref('')
const yamlIsError = ref(false)

// ============ 配置互转 Tab ============
const CONFIG_FORMATS: { value: ConfigFormat; label: string; example: string }[] = [
  { value: 'json', label: 'JSON', example: '{"name": "test", "port": 8080, "server": {"host": "localhost"}}' },
  { value: 'yaml', label: 'YAML', example: 'name: test\nport: 8080\nserver:\n  host: localhost' },
  { value: 'toml', label: 'TOML', example: 'name = "test"\nport = 8080\n\n[server]\nhost = "localhost"' },
  { value: 'ini', label: 'INI', example: 'name=test\nport=8080\n\n[server]\nhost=localhost' },
  { value: 'properties', label: 'Properties', example: 'name=test\nport=8080\nserver.host=localhost' },
]

const convSrc = ref<ConfigFormat>('json')
const convDst = ref<ConfigFormat>('toml')
const convInput = ref('')
const convOutput = ref('')
const convError = ref('')
const convIsError = ref(false)

const convSrcLabel = computed(() => CONFIG_FORMATS.find(f => f.value === convSrc.value)?.label || convSrc.value)
const convDstLabel = computed(() => CONFIG_FORMATS.find(f => f.value === convDst.value)?.label || convDst.value)
const convPlaceholder = computed(() => {
  const f = CONFIG_FORMATS.find(x => x.value === convSrc.value)
  return `示例（${convSrcLabel.value}）：\n${f?.example || ''}`
})

const swapFormat = () => {
  const s = convSrc.value; convSrc.value = convDst.value; convDst.value = s
  // 把已有输出反过来作为输入
  if (convOutput.value || convInput.value) {
    const tmp = convInput.value
    convInput.value = convOutput.value
    convOutput.value = tmp
    convError.value = ''
  }
}

const handleConvert = () => {
  if (!convInput.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  try {
    const obj = parseConfig(convInput.value, convSrc.value)
    convOutput.value = stringifyConfig(obj, convDst.value)
    convError.value = ''
    convIsError.value = false
    ElMessage.success(`${convSrcLabel.value} → ${convDstLabel.value} 转换完成`)
    store.addHistory({
      tool: 'xmlYaml',
      action: `${convSrcLabel.value}→${convDstLabel.value}`,
      inputPreview: convInput.value.slice(0, 50),
      outputPreview: convOutput.value.slice(0, 50),
      inputFull: `${convSrc.value}→${convDst.value}\n${convInput.value}`,
      outputFull: convOutput.value,
    })
  } catch (e: any) {
    convOutput.value = ''
    convError.value = '转换失败: ' + (e.message || '未知错误')
    convIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleConvPaste = async () => { try { convInput.value = await navigator.clipboard.readText() } catch { ElMessage.warning('无法读取剪贴板') } }
const handleConvCopy = () => { navigator.clipboard.writeText(convOutput.value || convError.value); ElMessage.success('已复制') }

// XML 操作
const handleXmlFormat = () => {
  try {
    const result = formatXml(xmlInput.value, xmlIndent.value)
    xmlOutput.value = result
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('格式化完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('格式化失败')
  }
}

const handleXmlCompress = () => {
  try {
    xmlOutput.value = xmlInput.value.replace(/\s+/g, ' ').trim()
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('压缩完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
  }
}

const handleXmlValidate = () => {
  const result = validateXml(xmlInput.value)
  if (result.valid) {
    xmlOutput.value = '✓ XML 格式正确'
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('XML 校验通过')
  } else {
    xmlOutput.value = ''
    xmlError.value = '✗ ' + result.error
    xmlIsError.value = true
    ElMessage.error('XML 校验失败')
  }
}

const handleXmlToJson = () => {
  try {
    xmlOutput.value = xmlToJson(xmlInput.value)
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('XML→JSON 转换完成')
    store.addHistory({ tool: 'xmlYaml', action: 'XML→JSON', inputPreview: xmlInput.value.slice(0, 50), outputPreview: xmlOutput.value.slice(0, 50), inputFull: xmlInput.value, outputFull: xmlOutput.value })
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleJsonToXml = () => {
  try {
    xmlOutput.value = jsonToXml(xmlInput.value)
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('JSON→XML 转换完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleClear = () => { xmlInput.value = ''; xmlOutput.value = ''; xmlError.value = '' }
const handlePaste = async () => { try { xmlInput.value = await navigator.clipboard.readText() } catch { ElMessage.warning('无法读取剪贴板') } }
const handleInsertVariable = (value: string) => { xmlInput.value = value }
const handleCopy = () => { navigator.clipboard.writeText(xmlOutput.value || xmlError.value); ElMessage.success('已复制') }

// YAML 操作
const handleYamlFormat = () => {
  try {
    const parsed = parseYaml(yamlInput.value)
    yamlOutput.value = jsonToYaml(JSON.stringify(parsed))
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('格式化完成')
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('格式化失败')
  }
}

const handleYamlValidate = () => {
  try {
    parseYaml(yamlInput.value)
    yamlOutput.value = '✓ YAML 格式正确'
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('YAML 校验通过')
  } catch (e: any) {
    yamlOutput.value = ''
    yamlError.value = '✗ ' + e.message
    yamlIsError.value = true
    ElMessage.error('YAML 校验失败')
  }
}

const handleYamlToJson = () => {
  try {
    const parsed = parseYaml(yamlInput.value)
    yamlOutput.value = JSON.stringify(parsed, null, 2)
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('YAML→JSON 转换完成')
    store.addHistory({ tool: 'xmlYaml', action: 'YAML→JSON', inputPreview: yamlInput.value.slice(0, 50), outputPreview: yamlOutput.value.slice(0, 50), inputFull: yamlInput.value, outputFull: yamlOutput.value })
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleJsonToYaml = () => {
  try {
    yamlOutput.value = jsonToYaml(yamlInput.value)
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('JSON→YAML 转换完成')
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleYamlClear = () => { yamlInput.value = ''; yamlOutput.value = ''; yamlError.value = '' }
const handleYamlPaste = async () => { try { yamlInput.value = await navigator.clipboard.readText() } catch { ElMessage.warning('无法读取剪贴板') } }
const handleYamlCopy = () => { navigator.clipboard.writeText(yamlOutput.value || yamlError.value); ElMessage.success('已复制') }
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

/* Tab 样式 - 滚动置顶 */
.xml-yaml-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.xml-yaml-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
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

.error-message { margin-top: 8px; padding: 8px 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; line-height: 1.5; }
:deep(.el-textarea.error .el-textarea__inner) { border-color: var(--accent-red); box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1); }

/* ===== 配置互转 Tab ===== */
.arrow-icon {
  font-size: 18px;
  color: var(--accent-cyan);
  margin: 0 4px;
}
</style>
