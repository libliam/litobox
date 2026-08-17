<template>
  <div class="tool-container">
    <!-- 操作配置卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">格式化配置</span>
          <el-tooltip placement="top">
            <template #content>
              <div class="tooltip-content">
                <p>基于 Prettier 的多语言代码格式化，支持 JS/TS/JSON/CSS/HTML/Vue/Markdown/YAML</p>
                <p>首次格式化时按需加载对应语言引擎，仅一次</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">语言</div>
            <el-select v-model="language" size="small" style="width: 220px">
              <el-option
                v-for="lang in languageList"
                :key="lang.id"
                :label="lang.label"
                :value="lang.id"
              />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">缩进</div>
            <el-radio-group v-model="tabWidth" size="small">
              <el-radio-button :value="2">2 空格</el-radio-button>
              <el-radio-button :value="4">4 空格</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">分号</div>
            <el-switch v-model="semi" size="small" active-text="保留" inactive-text="省略" />
          </div>
          <div class="action-group">
            <div class="group-label">引号</div>
            <el-radio-group v-model="quoteStyle" size="small">
              <el-radio-button :value="true">单引号</el-radio-button>
              <el-radio-button :value="false">双引号</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :loading="formatting"
                :disabled="!input.trim()"
                @click="handleFormat"
              >
                格式化
              </el-button>
              <el-button size="small" @click="loadSample">示例</el-button>
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
          <el-button size="small" @click="input = ''">清空</el-button>
          <el-button size="small" @click="pasteInput">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="input"
          type="textarea"
          :rows="12"
          placeholder="粘贴需要格式化的代码..."
          resize="vertical"
          class="code-input"
        />
      </div>
    </div>

    <!-- 输出卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!output" @click="copyOutput">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          :model-value="output"
          type="textarea"
          :rows="12"
          readonly
          resize="vertical"
          class="code-input"
          :class="{ 'error': isError }"
        />
        <div v-if="error" class="error-message">{{ error }}</div>
        <div v-if="input && !output && !error" class="hint-message">
          点击「格式化」生成结果
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore, type HistoryRestoreState } from '@/store'
import { formatCode, LANGUAGE_LIST, getLanguageConfig } from '@/utils/codeFormatterUtils'

const store = useToolboxStore()

// ============ 状态 ============
const language = ref('javascript')
const tabWidth = ref<2 | 4>(2)
const semi = ref(true)
const quoteStyle = ref(true)
const input = ref('')
const output = ref('')
const error = ref('')
const isError = ref(false)
const formatting = ref(false)

const languageList = LANGUAGE_LIST

// ============ 操作 ============
const handleFormat = async () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入代码')
    return
  }
  formatting.value = true
  error.value = ''
  isError.value = false
  try {
    const result = await formatCode(input.value, language.value, {
      tabWidth: tabWidth.value,
      semi: semi.value,
      singleQuote: quoteStyle.value,
    })
    if (result.success) {
      output.value = result.data || ''
      recordHistory()
    } else {
      output.value = ''
      error.value = result.error || '格式化失败'
      isError.value = true
    }
  } finally {
    formatting.value = false
  }
}

const loadSample = () => {
  const conf = getLanguageConfig(language.value)
  input.value = conf?.sample || ''
}

const pasteInput = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const copyOutput = async () => {
  try {
    await navigator.clipboard.writeText(output.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const recordHistory = () => {
  store.addHistory({
    tool: 'codeFormatter',
    action: `格式化（${getLanguageConfig(language.value)?.label || language.value}）`,
    inputPreview: input.value.slice(0, 50),
    outputPreview: output.value.slice(0, 50),
    inputFull: input.value,
    outputFull: output.value,
    options: {
      language: language.value,
      tabWidth: tabWidth.value,
      semi: semi.value,
      singleQuote: quoteStyle.value,
    },
  })
}

// ============ 历史还原（KeepAlive 用 watch 兜底） ============
const restoreFromHistory = (data: HistoryRestoreState) => {
  input.value = data.input || ''
  output.value = data.output || ''
  error.value = ''
  isError.value = false
  if (data.options?.language) language.value = data.options.language
  if (data.options?.tabWidth) tabWidth.value = data.options.tabWidth
  if (typeof data.options?.semi === 'boolean') semi.value = data.options.semi
  if (typeof data.options?.singleQuote === 'boolean') quoteStyle.value = data.options.singleQuote
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}

watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (data?.tool === 'codeFormatter') {
      restoreFromHistory(data)
      store.clearHistoryRestore()
    }
  }
)
</script>

<style scoped>
/* ===== 页面特有样式 ===== */
.code-input :deep(.el-textarea__inner) {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow: auto;
}

.hint-message {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
