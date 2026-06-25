<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 实时统计字符数、单词数、行数</p>
                <p>• 区分中英文字数统计</p>
                <p>• 估算阅读时间</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">阅读速度</div>
            <el-select v-model="readSpeed" size="small" style="width: 120px">
              <el-option label="中文 (300字/分)" :value="300" />
              <el-option label="英文 (200词/分)" :value="200" />
              <el-option label="快速 (500字/分)" :value="500" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleCount">统计</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="8" placeholder="请输入文本..." resize="vertical" @input="handleAutoCount" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">统计结果</span>
      </div>
      <div class="card-body">
        <div v-if="stats" class="stats-grid">
          <div class="stat-item">
            <div class="stat-value">{{ stats.charCount }}</div>
            <div class="stat-label">字符数（含空格）</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.charCountNoSpace }}</div>
            <div class="stat-label">字符数（不含空格）</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.chineseCount }}</div>
            <div class="stat-label">中文字数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.englishWords }}</div>
            <div class="stat-label">英文单词数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.lineCount }}</div>
            <div class="stat-label">行数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.paragraphCount }}</div>
            <div class="stat-label">段落数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.readTime }}</div>
            <div class="stat-label">阅读时间</div>
          </div>
        </div>
        <div v-else class="empty-tip">输入文本后将在此显示统计结果</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const readSpeed = ref(300)

interface Stats {
  charCount: number
  charCountNoSpace: number
  chineseCount: number
  englishWords: number
  lineCount: number
  paragraphCount: number
  readTime: string
}

const stats = ref<Stats | null>(null)

const countText = (text: string): Stats => {
  const charCount = text.length
  const charCountNoSpace = text.replace(/\s/g, '').length
  const chineseCount = (text.match(/[\u4e00-\u9fa5]/g) || []).length
  const englishWords = text.trim() ? (text.match(/[a-zA-Z]+/g) || []).length : 0
  const lineCount = text ? text.split('\n').length : 0
  const paragraphCount = text ? text.split(/\n\s*\n/).filter(p => p.trim()).length : 0

  const totalWords = chineseCount + englishWords
  const minutes = Math.ceil(totalWords / readSpeed.value)
  const readTime = minutes < 1 ? '< 1分钟' : `${minutes}分钟`

  return { charCount, charCountNoSpace, chineseCount, englishWords, lineCount, paragraphCount, readTime }
}

const handleCount = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入文本')
    return
  }
  stats.value = countText(input.value)
  store.addHistory({
    tool: 'wordCount',
    action: '字数统计',
    inputPreview: input.value.slice(0, 50),
    outputPreview: `字符数: ${stats.value.charCount}`
  })
  ElMessage.success('统计完成')
}

const handleAutoCount = () => {
  if (input.value.trim()) {
    stats.value = countText(input.value)
  } else {
    stats.value = null
  }
}

const handleClear = () => {
  input.value = ''
  stats.value = null
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
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

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.stat-item {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
  transition: border-color 0.2s;
}
.stat-item:hover { border-color: rgba(0, 212, 255, 0.3); }
.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent-cyan);
  line-height: 1.2;
}
.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
