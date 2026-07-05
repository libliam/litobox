<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">生成选项</span>
        <el-button size="small" type="primary" @click="handleGenerate">生成 UUID</el-button>
      </div>
      <div class="card-body">
        <div class="options-row">
          <div class="option-group">
            <span class="option-label">数量</span>
            <el-input-number v-model="count" :min="1" :max="100" size="small" style="width: 120px" />
          </div>
          <div class="option-group">
            <el-checkbox v-model="uppercase" size="small">大写</el-checkbox>
            <el-checkbox v-model="removeDashes" size="small">无横线</el-checkbox>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">生成结果</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCopyAll">复制全部</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="uuids.length === 0" class="empty-state">
          点击"生成 UUID"开始生成
        </div>
        <div v-else class="uuid-list">
          <div
            v-for="(uuid, index) in uuids"
            :key="index"
            class="uuid-item"
          >
            <span class="uuid-index">#{{ index + 1 }}</span>
            <code class="uuid-text">{{ uuid }}</code>
            <el-button size="small" text @click="handleCopy(uuid)">复制</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { generateUUIDs } from '@/utils/uuidUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const count = ref(1)
const uppercase = ref(false)
const removeDashes = ref(false)
const uuids = ref<string[]>([])

const handleGenerate = () => {
  uuids.value = generateUUIDs({
    count: count.value,
    uppercase: uppercase.value,
    removeDashes: removeDashes.value
  })

  store.addHistory({
    tool: 'uuid',
    action: 'generate',
    inputPreview: `count=${count.value}`,
    outputPreview: uuids.value[0],
    inputFull: `count=${count.value}, uppercase=${uppercase.value}, removeDashes=${removeDashes.value}`,
    outputFull: uuids.value.join('\n'),
  })

  ElMessage.success(`已生成 ${count.value} 个 UUID`)
}

const handleCopy = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyAll = async () => {
  const allUuids = uuids.value.join('\n')
  try {
    await navigator.clipboard.writeText(allUuids)
    ElMessage.success('已复制全部')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions {
  display: flex;
  gap: 8px;
}
.card-body {
  padding: 20px;
}

.options-row {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-wrap: wrap;
}
.option-group {
  display: flex;
  align-items: center;
  gap: 8px;
}
.option-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.empty-state {
  color: var(--text-muted);
  text-align: center;
  padding: 30px;
}

.uuid-list {
  max-height: 400px;
  overflow-y: auto;
}
.uuid-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 8px;
  transition: border-color 0.3s;
}
.uuid-item:hover {
  border-color: var(--accent-cyan);
}
.uuid-index {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  min-width: 30px;
  text-align: center;
}
.uuid-text {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}
</style>