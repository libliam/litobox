<template>
  <el-popover
    ref="popoverRef"
    placement="bottom-end"
    :width="280"
    trigger="click"
    popper-class="variable-picker-popover"
  >
    <template #reference>
      <el-button size="small" :icon="Collection" @mousedown="captureActiveElement">变量</el-button>
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
const popoverRef = ref()
const savedActiveElement = ref<Element | null>(null)

function captureActiveElement() {
  // ponytail: mousedown 在失焦前触发，此时 document.activeElement 还是 textarea
  savedActiveElement.value = document.activeElement
}

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

function insertAtCursor(nativeInput: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const start = nativeInput.selectionStart ?? 0
  const end = nativeInput.selectionEnd ?? 0
  const text = nativeInput.value
  nativeInput.value = text.slice(0, start) + value + text.slice(end)
  // ponytail: 直接设 selectionStart/End 比 setSelectionRange 兼容性更好
  nativeInput.selectionStart = nativeInput.selectionEnd = start + value.length
  nativeInput.focus()
  // ponytail: dispatch input 事件触发 v-model 更新
  nativeInput.dispatchEvent(new Event('input', { bubbles: true }))
}

function handleSelect(v: db.PoolVariable) {
  const active = savedActiveElement.value || document.activeElement
  // ponytail: 如果焦点在 textarea/input 上则插入到光标处，否则 fallback 到 emit
  if (active && (active.tagName === 'TEXTAREA' || active.tagName === 'INPUT')) {
    insertAtCursor(active as HTMLInputElement | HTMLTextAreaElement, v.value)
  } else {
    emit('select', v.value)
  }
  popoverRef.value?.hide()
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
