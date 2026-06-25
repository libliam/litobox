<template>
  <div class="tool-actions">
    <el-button
      v-for="action in actions"
      :key="action.label"
      :type="action.type || 'default'"
      size="small"
      @click="handleAction(action)"
    >
      {{ action.label }}
    </el-button>
  </div>
</template>

<script setup lang="ts">
export interface ToolAction {
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default'
  handler: () => void
}

defineProps<{
  actions: ToolAction[]
}>()

const handleAction = (action: ToolAction) => {
  action.handler()
}
</script>

<style scoped>
.tool-actions {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
</style>