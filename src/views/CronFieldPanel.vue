<!-- src/views/CronFieldPanel.vue -->
<template>
  <div class="cron-field-panel">
    <div class="panel-header">
      <div class="field-label">{{ label }}</div>
      <el-input
        v-model="manualInput"
        size="small"
        placeholder="手动输入"
        @blur="handleManualInput"
        style="width: 100px;"
      />
    </div>
    
    <div class="quick-actions">
      <el-button size="small" @click="selectAll">全选</el-button>
      <el-button size="small" @click="clearAll">清空</el-button>
      <el-button size="small" @click="toggleAll">反选</el-button>
    </div>
    
    <div class="grid-container" :class="`field-${field}`">
      <div 
        v-for="value in range" 
        :key="value"
        class="grid-item"
        :class="{ selected: values.has(value) }"
        @mousedown="startDrag(value)"
        @mouseenter="handleDragEnter(value)"
      >
        {{ value }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'

interface Props {
  field: 'second' | 'minute' | 'hour' | 'day' | 'month' | 'weekday'
  label: string
  values: Set<number>
  min: number
  max: number
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update-values': [field: 'second' | 'minute' | 'hour' | 'day' | 'month' | 'weekday', values: Set<number>]
}>()

// 手动输入框
const manualInput = ref('')

// 拖拽状态
const isDragging = ref(false)
const dragStartValue = ref<number | null>(null)

// 计算范围
const range = computed(() => {
  return Array.from({ length: props.max - props.min + 1 }, (_, i) => i + props.min)
})

// 切换值
const toggleValue = (value: number) => {
  const newValues = new Set(props.values)
  if (newValues.has(value)) {
    newValues.delete(value)
  } else {
    newValues.add(value)
  }
  emit('update-values', props.field, newValues)
}

// 全选
const selectAll = () => {
  const newValues = new Set<number>()
  for (let i = props.min; i <= props.max; i++) {
    newValues.add(i)
  }
  emit('update-values', props.field, newValues)
}

// 清空
const clearAll = () => {
  emit('update-values', props.field, new Set<number>())
}

// 反选
const toggleAll = () => {
  const newValues = new Set<number>()
  for (let i = props.min; i <= props.max; i++) {
    if (!props.values.has(i)) {
      newValues.add(i)
    }
  }
  emit('update-values', props.field, newValues)
}

// 拖拽开始
const startDrag = (value: number) => {
  isDragging.value = true
  dragStartValue.value = value
  toggleValue(value)
}

// 拖拽进入
const handleDragEnter = (value: number) => {
  if (!isDragging.value || dragStartValue.value === null) return
  
  // 计算从起始值到当前值的范围
  const start = Math.min(dragStartValue.value, value)
  const end = Math.max(dragStartValue.value, value)
  
  // 获取当前选中状态
  const newValues = new Set(props.values)
  
  // 根据起始点的状态决定操作
  if (props.values.has(dragStartValue.value)) {
    // 如果起始点是选中状态，则添加中间的值
    for (let i = start; i <= end; i++) {
      newValues.add(i)
    }
  } else {
    // 如果起始点是未选中状态，则移除中间的值
    for (let i = start; i <= end; i++) {
      newValues.delete(i)
    }
  }
  
  emit('update-values', props.field, newValues)
}

// 处理鼠标松开（结束拖拽）
const handleMouseUp = () => {
  isDragging.value = false
  dragStartValue.value = null
}

// 监听鼠标松开事件
watch(isDragging, (newValue) => {
  if (newValue) {
    window.addEventListener('mouseup', handleMouseUp)
  } else {
    window.removeEventListener('mouseup', handleMouseUp)
  }
})

// 处理手动输入
const handleManualInput = () => {
  if (!manualInput.value.trim()) return
  
  try {
    // 解析手动输入的值
    const newValues = new Set<number>()
    const segments = manualInput.value.split(',')
    
    for (const segment of segments) {
      if (segment === '*') {
        // 全选
        for (let i = props.min; i <= props.max; i++) {
          newValues.add(i)
        }
      } else if (segment.includes('-')) {
        // 范围
        const [startStr, endStr] = segment.split('-')
        const start = parseInt(startStr)
        const end = parseInt(endStr)
        
        if (!isNaN(start) && !isNaN(end) && start >= props.min && end <= props.max && start <= end) {
          for (let i = start; i <= end; i++) {
            newValues.add(i)
          }
        }
      } else {
        // 单个数字
        const num = parseInt(segment)
        if (!isNaN(num) && num >= props.min && num <= props.max) {
          newValues.add(num)
        }
      }
    }
    
    emit('update-values', props.field, newValues)
  } catch (e) {
    console.error('Invalid input:', e)
  }
}

// 监听props.values的变化，更新手动输入框
watch(() => props.values, (newValues) => {
  if (newValues.size === 0) {
    manualInput.value = ''
  } else if (newValues.size === props.max - props.min + 1) {
    // 全选
    manualInput.value = '*'
  } else {
    // 显示选中的值（简单实现，实际可能需要更复杂的逻辑来生成最优表达式）
    const sortedValues = Array.from(newValues).sort((a, b) => a - b)
    manualInput.value = sortedValues.join(',')
  }
}, { immediate: true })

// 组件卸载时清理事件监听
onUnmounted(() => {
  window.removeEventListener('mouseup', handleMouseUp)
})
</script>

<style scoped>
.cron-field-panel {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.field-label {
  font-weight: 600;
  color: var(--accent-cyan);
  font-size: 14px;
}

.quick-actions {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(10, 1fr);
  gap: 4px;
}

/* 根据字段类型调整列数 */
.cron-field-panel .grid-container.field-second,
.cron-field-panel .grid-container.field-minute {
  grid-template-columns: repeat(12, 1fr);
}

.cron-field-panel .grid-container.field-hour {
  grid-template-columns: repeat(12, 1fr);
}

.cron-field-panel .grid-container.field-day {
  grid-template-columns: repeat(10, 1fr);
}

.cron-field-panel .grid-container.field-month {
  grid-template-columns: repeat(6, 1fr);
}

.cron-field-panel .grid-container.field-weekday {
  grid-template-columns: repeat(7, 1fr);
}

.grid-item {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  user-select: none;
  font-size: 12px;
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.grid-item:hover {
  border-color: var(--accent-cyan);
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.1);
}

.grid-item.selected {
  background: var(--accent-cyan);
  color: var(--bg-primary);
  border-color: var(--accent-cyan);
  box-shadow: 0 0 8px rgba(0, 212, 255, 0.5);
}
</style>
