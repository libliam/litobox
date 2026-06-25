<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="dev-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="编码工具" name="encode" />
      <el-tab-pane label="加解密" name="crypto" />
      <el-tab-pane label="时间工具" name="time" />
      <el-tab-pane label="URL工具" name="url" />
      <el-tab-pane label="正则工具" name="regex" />
      <el-tab-pane label="进制转换" name="base" />
      <el-tab-pane label="UUID生成" name="uuid" />
    </el-tabs>

    <EncodeTool v-if="activeTab === 'encode'" />
    <CryptoTool v-if="activeTab === 'crypto'" />
    <TimeTool v-if="activeTab === 'time'" />
    <URLTool v-if="activeTab === 'url'" />
    <RegexTool v-if="activeTab === 'regex'" />
    <BaseConverter v-if="activeTab === 'base'" />
    <UUIDTool v-if="activeTab === 'uuid'" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import EncodeTool from './EncodeTool.vue'
import CryptoTool from './CryptoTool.vue'
import TimeTool from './TimeTool.vue'
import URLTool from './URLTool.vue'
import RegexTool from './RegexTool.vue'
import BaseConverter from './BaseConverter.vue'
import UUIDTool from './UUIDTool.vue'

const activeTab = ref('encode')

const handleTabClick = () => {
  // ponytail: tab切换时保持各子组件状态，后续可扩展tab缓存
}

onMounted(() => {
  const saved = localStorage.getItem('devtools_active_tab')
  if (saved && ['encode', 'crypto', 'time', 'url', 'regex', 'base', 'uuid'].includes(saved)) {
    activeTab.value = saved
  }
})

import { watch } from 'vue'
watch(activeTab, (val) => {
  localStorage.setItem('devtools_active_tab', val)
})
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 16px 16px 16px 8px;
}

/* 一级 Tab（开发工具/文件处理） */
.dev-tabs {
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  padding-left: 8px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .dev-tabs {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.dev-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
}

.dev-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 12px;
}

.dev-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.dev-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.dev-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.dev-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}
</style>
