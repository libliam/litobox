<template>
  <div class="tool-container file-container">
    <el-tabs v-model="activeTab" class="file-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="批量文本处理" name="batchText" />
      <el-tab-pane label="文件编码" name="fileEncoding" />
      <el-tab-pane label="十六进制查看" name="hexViewer" />
    </el-tabs>

    <BatchTextTool v-show="activeTab === 'batchText'" class="file-subtool" />
    <FileEncodingTool v-show="activeTab === 'fileEncoding'" class="file-subtool" />
    <HexViewerTool v-show="activeTab === 'hexViewer'" class="file-subtool" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BatchTextTool from './BatchTextTool.vue'
import FileEncodingTool from './FileEncodingTool.vue'
import HexViewerTool from './HexViewerTool.vue'

const activeTab = ref('batchText')

const handleTabClick = () => {
  // ponytail: 预留扩展，后续添加更多文件处理子功能
}
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  padding: 16px 16px 16px 8px;
}

/* 让当前子工具撑满 Tab 栏之外的剩余空间，配合子组件内部的填充布局 */
.file-container {
  display: flex;
  flex-direction: column;
}
.file-subtool {
  flex: 1;
  min-height: 0;
}

/* 一级 Tab（开发工具/文件处理） */
.file-tabs {
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  padding-left: 8px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
}

html.light .file-tabs {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.file-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
}

.file-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 12px;
}

.file-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.file-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.file-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.file-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}
</style>
