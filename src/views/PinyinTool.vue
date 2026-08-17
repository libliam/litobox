<template>
  <div class="tool-container">
    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="pinyin-tabs">
        <el-tab-pane label="拼音转换" name="single" />
        <el-tab-pane label="批量处理" name="batch" />
      </el-tabs>
    </div>

    <!-- Tab 1: 单文本拼音转换 -->
    <div v-if="activeTab === 'single'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">输出模式</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>选择拼音输出格式，输入文本后实时转换</p>
                <p>非中文字符（字母/数字/符号）原样保留</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="input += `{{${$event}}}`" />
          <el-button size="small" @click="input = ''; output = ''">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">格式</div>
            <el-select v-model="mode" size="small" style="width: 180px">
              <el-option v-for="m in MODES" :key="m.value" :label="m.label" :value="m.value" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">分隔符</div>
            <el-select v-model="separator" size="small" style="width: 100px">
              <el-option label="空格" value=" " />
              <el-option label="无分隔" value="" />
              <el-option label="下划线" value="_" />
              <el-option label="短横线" value="-" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">多音字</div>
            <el-switch v-model="heteronym" size="small" />
            <span v-if="heteronym" class="hint-text">显示所有候选</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'single'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <span class="stat-text">{{ input.length }} 字符</span>
      </div>
      <div class="card-body">
        <el-input
          v-model="input"
          type="textarea"
          :rows="5"
          placeholder="输入中文文本，如：你好世界"
          resize="vertical"
        />
      </div>
    </div>

    <div v-if="activeTab === 'single'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCopy(output)" :disabled="!output">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          :model-value="output"
          type="textarea"
          :rows="5"
          readonly
          resize="vertical"
          placeholder="转换结果将在此显示..."
        />
        <!-- 多音字候选展示 -->
        <div v-if="heteronymList.length" class="heteronym-block">
          <div class="heteronym-title">多音字候选（共 {{ heteronymList.length }} 字）</div>
          <div v-for="(item, i) in heteronymList" :key="i" class="heteronym-item">
            <span class="heteronym-char">{{ item.char }}</span>
            <span class="heteronym-pinyins">{{ item.pinyins.join(' / ') }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 2: 批量处理 -->
    <div v-if="activeTab === 'batch'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">批量转换</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>每行一条文本，批量转换为拼音</p>
                <p>输出保持逐行对应</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="batchInput = ''; batchOutput = ''">清空</el-button>
          <el-button size="small" @click="handlePasteBatch">粘贴</el-button>
          <el-button size="small" type="primary" @click="handleBatchConvert">转换</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">格式</div>
            <el-select v-model="mode" size="small" style="width: 180px">
              <el-option v-for="m in MODES" :key="m.value" :label="m.label" :value="m.value" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">分隔符</div>
            <el-select v-model="separator" size="small" style="width: 100px">
              <el-option label="空格" value=" " />
              <el-option label="无分隔" value="" />
              <el-option label="下划线" value="_" />
              <el-option label="短横线" value="-" />
            </el-select>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'batch'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入（每行一条）</span>
        <span class="stat-text">{{ batchLineCount }} 条</span>
      </div>
      <div class="card-body">
        <el-input
          v-model="batchInput"
          type="textarea"
          :rows="8"
          placeholder="每行一条中文文本，例如：&#10;张三&#10;李四&#10;王五"
          resize="vertical"
        />
      </div>
    </div>

    <div v-if="activeTab === 'batch'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCopy(batchOutput)" :disabled="!batchOutput">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          :model-value="batchOutput"
          type="textarea"
          :rows="8"
          readonly
          resize="vertical"
          placeholder="批量转换结果..."
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { pinyin } from 'pinyin-pro'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

// ============ 输出模式定义 ============
const MODES = [
  { value: 'tone', label: '全拼带声调', desc: 'nǐ hǎo shì jiè' },
  { value: 'none', label: '全拼无声调', desc: 'ni hao shi jie' },
  { value: 'initial', label: '首字母大写', desc: 'N H S J' },
  { value: 'camel', label: '驼峰式', desc: 'NiHaoShiJie' },
  { value: 'lower', label: '全小写无间隔', desc: 'nihaoshijie' },
] as const

// ============ Tab 状态 ============
const activeTab = ref('single')

// 还原期间禁止写历史
let isRestoring = false
let restoreTimer: ReturnType<typeof setTimeout> | null = null
const blockHistory = () => {
  isRestoring = true
  if (restoreTimer) clearTimeout(restoreTimer)
  restoreTimer = setTimeout(() => { isRestoring = false }, 500)
}

// ============ 单文本 ============
const input = ref('')
const output = ref('')
const mode = ref<typeof MODES[number]['value']>('tone')
const separator = ref(' ')
const heteronym = ref(false)
const heteronymList = ref<{ char: string; pinyins: string[] }[]>([])

/** 核心：根据当前模式转换拼音 */
const convert = (text: string, modeVal: typeof mode.value, sep: string): string => {
  if (modeVal === 'initial') {
    // 首字母：取每个拼音首字母大写
    const arr = pinyin(text, { toneType: 'none', type: 'array' }) as unknown as string[]
    return arr.map(p => p.charAt(0).toUpperCase()).join(sep)
  }
  if (modeVal === 'camel') {
    // 驼峰：每个拼音首字母大写，无分隔
    const arr = pinyin(text, { toneType: 'none', type: 'array' }) as unknown as string[]
    return arr.map(p => p.charAt(0).toUpperCase() + p.slice(1)).join('')
  }
  if (modeVal === 'lower') {
    // 全小写无间隔
    return pinyin(text, { toneType: 'none', type: 'string' }).replace(/\s+/g, '')
  }
  // tone / none：标准输出，用分隔符连接
  const result = pinyin(text, { toneType: modeVal === 'tone' ? 'symbol' : 'none', type: 'string' })
  if (sep === ' ') return result
  return result.replace(/\s+/g, sep)
}

/** 提取多音字候选 */
const extractHeteronyms = (text: string) => {
  if (!heteronym.value) {
    heteronymList.value = []
    return
  }
  const list: { char: string; pinyins: string[] }[] = []
  for (const char of text) {
    // 只处理中文字符
    if (!/[\u4e00-\u9fa5]/.test(char)) continue
    // multiple 仅对单字生效，返回所有读音
    const all = pinyin(char, { toneType: 'none', type: 'array', multiple: true }) as unknown as string[]
    if (all.length > 1) {
      list.push({ char, pinyins: [...new Set(all)] })
    }
  }
  heteronymList.value = list
}

const runConvert = () => {
  if (!input.value.trim()) {
    output.value = ''
    heteronymList.value = []
    return
  }
  try {
    output.value = convert(input.value, mode.value, separator.value)
    extractHeteronyms(input.value)
    if (!isRestoring && output.value) {
      store.addHistory({
        tool: 'pinyin',
        action: '拼音转换',
        inputPreview: `${mode.value} | ${input.value.slice(0, 40)}`,
        outputPreview: output.value.slice(0, 50),
        inputFull: `${mode.value}\n${separator.value}\n${input.value}`,
        outputFull: output.value,
      })
    }
  } catch (e: any) {
    output.value = ''
    ElMessage.error('转换失败：' + (e.message || '未知错误'))
  }
}

watch([input, mode, separator, heteronym], () => runConvert())

// ============ 批量处理 ============
const batchInput = ref('')
const batchOutput = ref('')

const batchLineCount = computed(() =>
  batchInput.value.split('\n').filter(l => l.trim().length > 0).length
)

const handleBatchConvert = () => {
  if (!batchInput.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  try {
    const lines = batchInput.value.split('\n')
    batchOutput.value = lines
      .map(line => line.trim() ? convert(line, mode.value, separator.value) : '')
      .join('\n')
    if (!isRestoring) {
      store.addHistory({
        tool: 'pinyin',
        action: '批量拼音转换',
        inputPreview: `${mode.value} | ${batchLineCount.value} 条`,
        outputPreview: `${batchLineCount.value} 条已转换`,
        inputFull: `${mode.value}\n${separator.value}\n${batchInput.value}`,
        outputFull: batchOutput.value,
      })
    }
    ElMessage.success(`已转换 ${batchLineCount.value} 条`)
  } catch (e: any) {
    ElMessage.error('转换失败：' + (e.message || '未知错误'))
  }
}

// ============ 历史还原 ============
watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== 'pinyin') return
  blockHistory()
  const action = restore.action
  // inputFull 格式: "mode\nseparator\n内容"
  const parts = (restore.input || '').split('\n')
  if (parts.length >= 3) {
    mode.value = parts[0] as typeof mode.value
    separator.value = parts[1]
    const content = parts.slice(2).join('\n')
    if (action === '批量拼音转换') {
      activeTab.value = 'batch'
      batchInput.value = content
      handleBatchConvert()
    } else {
      activeTab.value = 'single'
      input.value = content
    }
  }
  store.clearHistoryRestore()
})

// ============ 通用方法 ============
const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    input.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handlePasteBatch = async () => {
  try {
    const text = await navigator.clipboard.readText()
    batchInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.pinyin-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
html.light .pinyin-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.pinyin-tabs :deep(.el-tabs__nav-wrap) { padding-left: 4px; }
.pinyin-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}
.pinyin-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.pinyin-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.pinyin-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }

/* ===== 通用 ===== */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.header-left { display: flex; align-items: center; gap: 8px; }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }
.stat-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-right: 8px;
}
.hint-text {
  font-size: 12px;
  color: var(--text-muted);
}

/* ===== 多音字候选 ===== */
.heteronym-block {
  margin-top: 16px;
  padding: 12px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-left: 3px solid var(--accent-cyan);
  border-radius: 6px;
}
.heteronym-title {
  font-size: 12px;
  color: var(--accent-cyan);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 10px;
}
.heteronym-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 0;
  border-bottom: 1px dashed var(--border-color);
}
.heteronym-item:last-child { border-bottom: none; }
.heteronym-char {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  min-width: 24px;
  text-align: center;
}
.heteronym-pinyins {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
