<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 按行去重，支持保留首次或末次出现</p>
                <p>• 可选忽略大小写和首尾空格</p>
                <p>• 显示重复行及其出现次数</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">模式</div>
            <el-radio-group v-model="mode" size="small">
              <el-radio-button label="first">保留首次</el-radio-button>
              <el-radio-button label="last">保留末次</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <label class="switch-item">
              <span>忽略大小写</span>
              <el-switch v-model="ignoreCase" size="small" />
            </label>
            <label class="switch-item">
              <span>忽略首尾空格</span>
              <el-switch v-model="ignoreWhitespace" size="small" />
            </label>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleDedup">去重</el-button>
              <el-button size="small" @click="handleCopy">复制结果</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (每行一个)</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="10" placeholder="请输入文本，每行一个条目..." resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-tag v-if="stats" size="small" type="success">{{ stats.uniqueLines }} / {{ stats.originalLines }} 行</el-tag>
        </div>
      </div>
      <div class="card-body">
        <el-input :model-value="output" type="textarea" :rows="8" readonly resize="vertical" />
        <div v-if="stats && stats.duplicateLines > 0" class="stats-info">
          <span>去除了 <strong>{{ stats.duplicateLines }}</strong> 行重复</span>
        </div>
        <div v-if="duplicateList.length > 0" class="duplicate-list">
          <div class="duplicate-title">重复项 (最多显示20条):</div>
          <div v-for="(item, idx) in duplicateList.slice(0, 20)" :key="idx" class="duplicate-item">
            <span class="duplicate-text">{{ item.text }}</span>
            <el-tag size="small" type="warning">{{ item.count }} 次</el-tag>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { dedupLines, type DedupResult } from '@/utils/dedupUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const output = ref('')
const mode = ref<'first' | 'last'>('first')
const ignoreCase = ref(false)
const ignoreWhitespace = ref(false)
const stats = ref<DedupResult | null>(null)

const duplicateList = computed(() => {
  if (!stats.value) return []
  return Array.from(stats.value.duplicates.entries()).map(([text, count]) => ({ text, count }))
})

const handleDedup = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  try {
    const result = dedupLines(input.value, {
      mode: mode.value,
      ignoreCase: ignoreCase.value,
      ignoreWhitespace: ignoreWhitespace.value,
    })
    output.value = result.output
    stats.value = result
    ElMessage.success(`去重完成: ${result.originalLines} → ${result.uniqueLines} 行`)
    store.addHistory({
      tool: 'dedup',
      action: '文本去重',
      inputPreview: input.value.slice(0, 50),
      outputPreview: `${result.uniqueLines} 行`,
    })
  } catch (e: any) {
    ElMessage.error('去重失败: ' + e.message)
  }
}

const handleClear = () => {
  input.value = ''
  output.value = ''
  stats.value = null
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleCopy = () => {
  if (!output.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  navigator.clipboard.writeText(output.value)
  ElMessage.success('已复制')
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
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.stats-info {
  margin-top: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.stats-info strong { color: var(--accent-cyan); }

.duplicate-list { margin-top: 12px; }
.duplicate-title { font-size: 13px; color: var(--text-secondary); margin-bottom: 6px; }
.duplicate-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 13px;
  border-radius: 4px;
  background: var(--bg-input);
  margin-bottom: 4px;
}
.duplicate-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 80%;
}
</style>
