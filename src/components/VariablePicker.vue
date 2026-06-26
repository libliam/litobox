<template>
  <el-popover
    placement="bottom-end"
    :width="280"
    trigger="click"
    popper-class="variable-picker-popover"
  >
    <template #reference>
      <el-button size="small" :icon="Collection">变量</el-button>
    </template>
    <div class="variable-picker">
      <div v-if="loading" class="loading-hint">加载中...</div>
      <div v-else-if="variables.length === 0" class="empty-hint">
        变量池为空，请先到工作流页面添加变量
      </div>
      <div v-else class="variable-list">
        <div
          v-for="v in variables"
          :key="v.id"
          class="variable-item"
          @click="handleSelect(v)"
        >
          <span class="var-name">{{ v.name }}</span>
          <span class="var-value">{{ truncate(v.value, 30) }}</span>
        </div>
      </div>
    </div>
  </el-popover>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Collection } from '@element-plus/icons-vue'
import * as db from '@/utils/dbClient'

const emit = defineEmits<{
  select: [value: string]
}>()

const variables = ref<db.PoolVariable[]>([])
const loading = ref(false)

function truncate(str: string, len: number): string {
  return str.length > len ? str.slice(0, len) + '...' : str
}

async function loadVariables() {
  loading.value = true
  try {
    variables.value = await db.listVariables()
  } catch {
    // 静默失败
  } finally {
    loading.value = false
  }
}

function handleSelect(v: db.PoolVariable) {
  emit('select', v.value)
}

onMounted(loadVariables)
</script>

<style scoped>
.variable-picker {
  max-height: 300px;
  overflow-y: auto;
}
.loading-hint,
.empty-hint {
  padding: 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}
.variable-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.variable-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}
.variable-item:hover {
  background: rgba(0, 212, 255, 0.1);
}
.var-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-cyan);
}
.var-value {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: monospace;
}
</style>
