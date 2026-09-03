<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 词组级转换（OpenCC）：头发 → 頭髮、发生 → 發生 不会误转</p>
                <p>• 自动检测：按文本中繁/简字数占比自动选择方向</p>
                <p>• 支持整段混合文本，粘贴后自动转换</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">转换方向</span>
            <el-radio-group v-model="direction" size="small">
              <el-radio-button label="auto">自动检测</el-radio-button>
              <el-radio-button label="s2t">简体 → 繁体</el-radio-button>
              <el-radio-button label="t2s">繁体 → 简体</el-radio-button>
            </el-radio-group>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (简体/繁体均可)</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="10" placeholder="例如:&#10;云龙区的档案里有一个文件（简体）&#10;我的頭髮發生了問題（繁体）" resize="vertical" />
      </div>
    </div>

    <!-- 输出卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-tag v-if="statLabel" size="small" type="success">{{ statLabel }}</el-tag>
          <el-button v-if="output" size="small" @click="handleCopyOutput">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input :model-value="output" type="textarea" :rows="12" readonly resize="vertical" placeholder="转换结果将显示在此处" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import { convertTc, type TcDirection } from '@/utils/tcConvertUtils'

const store = useToolboxStore()

const input = ref('')
const output = ref('')
const direction = ref<TcDirection>('auto')
// auto 模式下实际生效的方向（转换后回填，用于提示）
const effectiveDir = ref<'s2t' | 't2s'>('s2t')
const replaced = ref(0)

const statLabel = computed(() => {
  if (!output.value) return ''
  const dirText = direction.value === 'auto'
    ? `自动识别：${effectiveDir.value === 's2t' ? '简体→繁体' : '繁体→简体'}`
    : (direction.value === 's2t' ? '简体→繁体' : '繁体→简体')
  return `${dirText} · 替换 ${replaced.value} 字`
})

// ============ 转换（输入 500ms 防抖；方向切换立即执行） ============
let execTimer: ReturnType<typeof setTimeout> | null = null
let runSeq = 0
let lastSavedKey = ''

const runConvert = async (immediate = false) => {
  if (!immediate && execTimer) clearTimeout(execTimer)
  const doRun = async () => {
    const token = ++runSeq
    const text = input.value
    if (!text.trim()) {
      output.value = ''
      effectiveDir.value = 's2t'
      replaced.value = 0
      lastSavedKey = ''
      return
    }
    // 首次转换会懒加载 opencc 词典（约 1MB），期间忽略过期结果
    const result = await convertTc(text, direction.value)
    if (token !== runSeq) return
    output.value = result.text
    effectiveDir.value = result.direction
    replaced.value = result.replaced
    recordHistory(text, result.text)
  }
  if (immediate) {
    void doRun()
  } else {
    execTimer = setTimeout(doRun, 500)
  }
}

const recordHistory = (inputFull: string, outputFull: string) => {
  const key = `${inputFull}|${direction.value}`
  if (key === lastSavedKey) return
  lastSavedKey = key
  store.addHistory({
    tool: 'tcConvert',
    action: '中文繁简转换',
    inputPreview: inputFull.slice(0, 50),
    outputPreview: outputFull.slice(0, 50),
    inputFull,
    outputFull,
    options: { direction: direction.value },
  })
}

watch(input, () => runConvert())
watch(direction, () => runConvert(true))

onBeforeUnmount(() => {
  if (execTimer) clearTimeout(execTimer)
})

// ============ 历史双击还原（KeepAlive 缓存后靠 watch 触发） ============
watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (!data || data.tool !== 'tcConvert') return
    if (data.input) {
      input.value = data.input
      if (data.options?.direction === 's2t' || data.options?.direction === 't2s' || data.options?.direction === 'auto') {
        direction.value = data.options.direction
      }
      store.clearHistoryRestore()
      runConvert(true)
    }
  },
)

// ============ 输入区操作 ============
const handleClear = () => {
  input.value = ''
  output.value = ''
  effectiveDir.value = 's2t'
  replaced.value = 0
  lastSavedKey = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      ElMessage.warning('剪贴板为空')
      return
    }
    input.value = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}

const handleCopyOutput = async () => {
  try {
    await navigator.clipboard.writeText(output.value)
    ElMessage.success('已复制转换结果')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>
