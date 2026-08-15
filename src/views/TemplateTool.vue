<template>
  <div class="tool-container">
    <!-- 顶部操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">模板渲染</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 左侧输入 Handlebars 模板，右侧输入 JSON 数据</p>
                <p>• 实时预览渲染结果，支持 &#123;&#123;变量&#125;&#125;、#each、#if 等标签</p>
                <p>• 点击上方片段按钮可快捷插入模板代码</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="loadExample">示例</el-button>
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" type="primary" @click="handleRender(true)">渲染</el-button>
          <div class="realtime-toggle">
            <span class="toggle-label">实时</span>
            <el-switch v-model="realtime" size="small" />
          </div>
        </div>
      </div>
      <div class="card-body">
        <div class="snippet-bar">
          <span class="snippet-label">快捷插入</span>
          <el-button
            v-for="s in SNIPPETS"
            :key="s.name"
            size="small"
            class="snippet-btn"
            @click="insertSnippet(s.insert)"
          >{{ s.name }}</el-button>
        </div>
      </div>
    </div>

    <!-- 模板 + 数据 双栏 -->
    <div class="split-grid">
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">模板（Handlebars）</span>
        </div>
        <div class="card-body">
          <textarea
            ref="templateTextarea"
            v-model="templateText"
            class="code-textarea"
            rows="16"
            spellcheck="false"
            placeholder="输入 Handlebars 模板，如：Hello &#123;&#123;name&#125;&#125;"
          />
        </div>
      </div>
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">数据（JSON）</span>
        </div>
        <div class="card-body">
          <textarea
            v-model="dataText"
            class="code-textarea"
            rows="16"
            spellcheck="false"
            placeholder='{"name": "张三"}'
          />
        </div>
      </div>
    </div>

    <!-- 预览输出 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">预览</span>
        <div class="card-actions" v-if="renderOk">
          <el-radio-group v-model="previewMode" size="small">
            <el-radio-button value="render">渲染预览</el-radio-button>
            <el-radio-button value="source">HTML 源码</el-radio-button>
          </el-radio-group>
          <el-button size="small" @click="handleCopyOutput">复制结果</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="result.templateError" class="error-box">
          <div class="error-title">模板错误</div>
          <div class="error-msg">{{ result.templateError }}</div>
        </div>
        <div v-else-if="result.dataError" class="error-box">
          <div class="error-title">数据错误</div>
          <div class="error-msg">{{ result.dataError }}</div>
        </div>
        <div v-else-if="renderOk" class="preview-wrap">
          <div v-if="previewMode === 'render'" class="render-box" v-html="result.output" />
          <pre v-else class="source-box">{{ result.output }}</pre>
        </div>
        <div v-else class="empty-tip">输入模板与数据后，渲染结果将显示在此处</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  renderTemplate, EXAMPLE_TEMPLATE, EXAMPLE_DATA, SNIPPETS, selfCheck,
  type TemplateRenderResult,
} from '@/utils/templateUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ 状态 ============
const templateText = ref('')
const dataText = ref('')
const realtime = ref(true)
const previewMode = ref<'render' | 'source'>('render')
const templateTextarea = ref<HTMLTextAreaElement>()
const result = ref<TemplateRenderResult>({ ok: false })
let debounceTimer: ReturnType<typeof setTimeout> | null = null

const renderOk = computed(() => result.value.ok && !!result.value.output)

// ============ 渲染 ============
// fromButton: 仅手动点击"渲染"按钮时记录历史，实时模式不记录（避免刷爆历史）
const handleRender = (fromButton = false) => {
  const r = renderTemplate(templateText.value, dataText.value)
  result.value = r
  if (fromButton && r.ok && r.output) {
    store.addHistory({
      tool: 'templateTool',
      action: '模板渲染',
      inputPreview: templateText.value.slice(0, 60),
      outputPreview: r.output.slice(0, 60),
      inputFull: templateText.value,
      outputFull: r.output,
      options: { dataText: dataText.value },
    })
  }
}

// 实时预览（防抖 300ms）
watch([templateText, dataText], () => {
  if (!realtime.value) return
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    handleRender()
  }, 300)
})

// 历史记录还原：恢复模板 + 数据，不自动记录新历史
watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== 'templateTool') return
  templateText.value = restore.input
  dataText.value = restore.options?.dataText ?? ''
  result.value = renderTemplate(templateText.value, dataText.value)
  previewMode.value = 'render'
  store.clearHistoryRestore()
})

// ============ 快捷插入 ============
const insertSnippet = (snippet: string) => {
  const ta = templateTextarea.value
  if (!ta) {
    templateText.value += snippet
    return
  }
  const start = ta.selectionStart ?? templateText.value.length
  const end = ta.selectionEnd ?? templateText.value.length
  templateText.value =
    templateText.value.slice(0, start) + snippet + templateText.value.slice(end)
  nextTick(() => {
    ta.focus()
    const pos = start + snippet.length
    ta.setSelectionRange(pos, pos)
  })
}

// ============ 示例 / 清空 ============
const loadExample = () => {
  templateText.value = EXAMPLE_TEMPLATE
  dataText.value = EXAMPLE_DATA
  previewMode.value = 'render'
  handleRender()
  ElMessage.success('已载入示例')
}

const handleClear = () => {
  templateText.value = ''
  dataText.value = ''
  result.value = { ok: false }
  ElMessage.success('已清空')
}

// ============ 复制 ============
const handleCopyOutput = async () => {
  if (!renderOk.value) return
  try {
    await navigator.clipboard.writeText(result.value.output!)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.warning('复制失败')
  }
}

// ============ 初始化 / 自检 ============
const errors = selfCheck()
if (errors.length > 0) {
  console.warn('templateUtils 自检失败:', errors)
}
</script>

<style scoped>
.tool-container {
  height: 100%;
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

.realtime-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 4px;
}
.toggle-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.snippet-bar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.snippet-label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}
.snippet-btn {
  font-family: 'JetBrains Mono', Consolas, monospace;
}

.split-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.code-textarea {
  width: 100%;
  min-height: 320px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-input);
  color: var(--text-primary);
  font-family: 'JetBrains Mono', Consolas, 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  transition: border-color 0.2s;
}
.code-textarea:focus {
  border-color: rgba(0, 212, 255, 0.5);
}

.preview-wrap {
  min-height: 120px;
}
.render-box {
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow-x: auto;
  color: var(--text-primary);
}
.render-box h1 { font-size: 20px; margin: 0 0 8px; }
.render-box p { margin: 4px 0; }
.render-box ul { margin: 4px 0; padding-left: 24px; }
.render-box li { margin: 2px 0; }
.source-box {
  margin: 0;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow-x: auto;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', Consolas, 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.error-box {
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.25);
  border-radius: 6px;
}
.error-title {
  color: var(--danger-color, #ef4444);
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 6px;
}
.error-msg {
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
}

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

@media (max-width: 1100px) {
  .split-grid {
    grid-template-columns: 1fr;
  }
}
</style>
