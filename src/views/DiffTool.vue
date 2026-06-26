<template>
  <div class="tool-container">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持行级和字符级两种对比模式</p>
                <p>• 行级：以行为单位标记新增/删除/修改</p>
                <p>• 字符级：在修改行内高亮具体变化的字符</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-button size="small" @click="handleCompare">开始对比</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">对比模式</span>
            <div class="group-buttons">
              <el-radio-group v-model="diffMode" size="small">
                <el-radio-button label="line">行级</el-radio-button>
                <el-radio-button label="char">字符级</el-radio-button>
              </el-radio-group>
            </div>
          </div>
          <div class="action-group">
            <span class="group-label">选项</span>
            <div class="group-buttons">
              <label class="switch-item"><span>自动执行</span><el-switch v-model="autoCompare" size="small" /></label>
              <label class="switch-item"><span>忽略空白</span><el-switch v-model="ignoreWhitespace" size="small" /></label>
              <label class="switch-item"><span>忽略大小写</span><el-switch v-model="ignoreCase" size="small" /></label>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-section">
      <div class="tool-card input-card">
        <div class="card-header">
          <span class="card-title">原始文本</span>
          <div class="card-actions">
            <VariablePicker @select="handleInsertVariableLeft" />
            <el-button size="small" @click="handleClearLeft">清空</el-button>
            <el-button size="small" @click="handlePasteLeft">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="leftText"
            type="textarea"
            :rows="10"
            placeholder="请输入原始文本..."
            resize="none"
          />
        </div>
      </div>
      <div class="tool-card input-card">
        <div class="card-header">
          <span class="card-title">修改后文本</span>
          <div class="card-actions">
            <VariablePicker @select="handleInsertVariableRight" />
            <el-button size="small" @click="handleClearRight">清空</el-button>
            <el-button size="small" @click="handlePasteRight">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="rightText"
            type="textarea"
            :rows="10"
            placeholder="请输入修改后的文本..."
            resize="none"
          />
        </div>
      </div>
    </div>

    <!-- 对比结果 -->
    <div v-if="diffResult.length > 0" class="tool-card result-card">
      <div class="card-header">
        <span class="card-title">对比结果</span>
        <div class="card-actions">
          <el-tag size="small" type="info">{{ diffResult.length }} 行</el-tag>
        </div>
      </div>
      <div class="card-body result-body">
        <div class="diff-container">
          <div class="diff-left" ref="leftRef" @scroll="handleScroll">
            <div
              v-for="(line, idx) in diffResult"
              :key="'left-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'left')"
            >
              <span class="line-num">{{ line.leftLineNum ?? '' }}</span>
              <span class="line-content">
                <template v-if="line.type === 'modify' && diffMode === 'char' && line.charDiffs">
                  <span
                    v-for="(char, ci) in line.charDiffs.filter(c => c.type !== 'add')"
                    :key="ci"
                    class="char-span"
                    :class="getCharClass(char)"
                  >{{ char.value }}</span>
                </template>
                <template v-else>
                  {{ line.leftLine ?? '' }}
                </template>
              </span>
            </div>
          </div>
          <div class="diff-right" ref="rightRef" @scroll="handleScroll">
            <div
              v-for="(line, idx) in diffResult"
              :key="'right-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'right')"
            >
              <span class="line-num">{{ line.rightLineNum ?? '' }}</span>
              <span class="line-content">
                <template v-if="line.type === 'modify' && diffMode === 'char' && line.charDiffs">
                  <span
                    v-for="(char, ci) in line.charDiffs.filter(c => c.type !== 'remove')"
                    :key="ci"
                    class="char-span"
                    :class="getCharClass(char)"
                  >{{ char.value }}</span>
                </template>
                <template v-else>
                  {{ line.rightLine ?? '' }}
                </template>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { computeLineDiff, computeCharDiff, DiffLine } from '@/utils/diffUtils'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

const leftText = ref('')
const rightText = ref('')
const diffMode = ref<'line' | 'char'>('line')
const autoCompare = ref(true)
const ignoreWhitespace = ref(false)
const ignoreCase = ref(false)
const diffResult = ref<DiffLine[]>([])

const leftRef = ref<HTMLElement>()
const rightRef = ref<HTMLElement>()

let autoCompareTimer: ReturnType<typeof setTimeout> | null = null

const runDiff = () => {
  if (!leftText.value && !rightText.value) {
    diffResult.value = []
    return
  }

  const lines = computeLineDiff(leftText.value, rightText.value, {
    ignoreWhitespace: ignoreWhitespace.value,
    ignoreCase: ignoreCase.value,
  })

  // 字符级模式：为 modify 行计算字符差异
  if (diffMode.value === 'char') {
    for (const line of lines) {
      if (line.type === 'modify' && line.leftLine && line.rightLine) {
        line.charDiffs = computeCharDiff(line.leftLine, line.rightLine)
      }
    }
  }

  diffResult.value = lines

  store.addHistory({
    tool: 'diff',
    action: diffMode.value === 'line' ? '行级对比' : '字符级对比',
    inputPreview: leftText.value.slice(0, 50),
    outputPreview: `${lines.length} 行差异`,
  })
}

const handleCompare = () => {
  runDiff()
  ElMessage.success('对比完成')
}

const handleClearLeft = () => { leftText.value = '' }
const handleClearRight = () => { rightText.value = '' }

const handlePasteLeft = async () => {
  try {
    leftText.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariableLeft = (value: string) => {
  leftText.value = value
}

const handlePasteRight = async () => {
  try {
    rightText.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariableRight = (value: string) => {
  rightText.value = value
}

const getLineClass = (line: DiffLine, side: 'left' | 'right') => {
  if (line.type === 'equal') return ''
  if (line.type === 'add' && side === 'right') return 'line-add'
  if (line.type === 'remove' && side === 'left') return 'line-remove'
  if (line.type === 'modify') return 'line-modify'
  return 'line-empty'
}

const getCharClass = (char: { type: string }) => {
  if (char.type === 'add') return 'char-add'
  if (char.type === 'remove') return 'char-remove'
  return ''
}

const handleScroll = (e: Event) => {
  const target = e.target as HTMLElement
  if (leftRef.value && rightRef.value) {
    if (target === leftRef.value) {
      rightRef.value.scrollTop = target.scrollTop
    } else {
      leftRef.value.scrollTop = target.scrollTop
    }
  }
}

// 自动对比
watch([leftText, rightText, diffMode, ignoreWhitespace, ignoreCase], () => {
  if (autoCompare.value) {
    if (autoCompareTimer) clearTimeout(autoCompareTimer)
    autoCompareTimer = setTimeout(runDiff, 300)
  }
})
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
.group-label { color: var(--accent-cyan); font-size: 13px; font-weight: 600; white-space: nowrap; }
.group-buttons { display: flex; gap: 16px; align-items: center; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.input-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.input-card { margin-bottom: 0; }

.result-body { padding: 0; }
.diff-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  max-height: 60vh;
  overflow: auto;
}
.diff-left, .diff-right {
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.diff-line {
  display: flex;
  padding: 2px 8px;
  min-height: 22px;
  white-space: pre;
}
.line-num {
  width: 40px;
  text-align: right;
  padding-right: 12px;
  color: var(--text-muted);
  user-select: none;
  flex-shrink: 0;
}
.line-content { flex: 1; }

.line-add { background: rgba(34, 197, 94, 0.15); }
.line-remove { background: rgba(239, 68, 68, 0.15); }
.line-modify { background: rgba(234, 179, 8, 0.1); }
.line-empty { opacity: 0.3; }

.char-add { background: rgba(34, 197, 94, 0.3); border-radius: 2px; }
.char-remove { background: rgba(239, 68, 68, 0.3); border-radius: 2px; text-decoration: line-through; }
</style>
