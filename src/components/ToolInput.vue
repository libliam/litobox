<template>
  <div class="tool-input">
    <div class="input-header">
      <span class="input-title">输入</span>
      <div class="input-actions">
        <el-button size="small" @click="handleClear">清空</el-button>
        <el-button size="small" @click="handlePaste">粘贴</el-button>
      </div>
    </div>
    <el-input
      v-model="inputValue"
      type="textarea"
      :rows="rows"
      :placeholder="placeholder"
      resize="vertical"
      @input="handleInput"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'

const props = defineProps<{
  modelValue: string
  placeholder?: string
  rows?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  clear: []
  paste: []
}>()

const inputValue = ref(props.modelValue)

watch(() => props.modelValue, (newVal) => {
  inputValue.value = newVal
})

const handleInput = (value: string) => {
  emit('update:modelValue', value)
}

const handleClear = () => {
  inputValue.value = ''
  emit('update:modelValue', '')
  emit('clear')
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
    emit('update:modelValue', text)
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}
</script>

<style scoped>
.tool-input {
  margin-bottom: 16px;
}
.input-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.input-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.input-actions {
  display: flex;
  gap: 8px;
}
</style>