<template>
  <div class="note-toolbar">
    <div class="toolbar-group">
      <el-button size="small" @click="$emit('find')">
        <el-icon><Search /></el-icon> 查找
      </el-button>
      <el-button size="small" @click="$emit('replace')">
        <el-icon><Search /></el-icon> 替换
      </el-button>
    </div>

    <div class="toolbar-group">
      <el-select v-model="localLanguage" size="small" style="width: 120px" @change="$emit('language-change', $event)">
        <el-option label="自动检测" value="auto" />
        <el-option label="纯文本" value="plaintext" />
        <el-option label="JavaScript" value="javascript" />
        <el-option label="TypeScript" value="typescript" />
        <el-option label="JSON" value="json" />
        <el-option label="HTML" value="html" />
        <el-option label="CSS" value="css" />
        <el-option label="Markdown" value="markdown" />
        <el-option label="XML" value="xml" />
        <el-option label="SQL" value="sql" />
        <el-option label="Python" value="python" />
        <el-option label="Rust" value="rust" />
      </el-select>
    </div>

    <div class="toolbar-group">
      <el-button size="small" @click="$emit('sort-lines')">排序行</el-button>
      <el-button size="small" @click="$emit('dedup-lines')">去重行</el-button>
      <el-button size="small" @click="$emit('reverse-lines')">反转行</el-button>
      <el-button size="small" @click="$emit('to-upper')">转大写</el-button>
      <el-button size="small" @click="$emit('to-lower')">转小写</el-button>
    </div>

    <div class="toolbar-group">
      <el-button type="primary" size="small" @click="$emit('format')">格式化</el-button>
    </div>

    <div class="toolbar-status" :class="{ modified: isModified }">
      {{ isModified ? '未保存 ●' : '已保存' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Search } from '@element-plus/icons-vue'

defineProps<{
  language?: string
  isModified?: boolean
}>()

defineEmits<{
  'find': []
  'replace': []
  'language-change': [lang: string]
  'sort-lines': []
  'dedup-lines': []
  'reverse-lines': []
  'to-upper': []
  'to-lower': []
  'format': []
}>()

const localLanguage = ref('auto')
</script>

<style scoped>
.note-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border-color);
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-status {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
}

.toolbar-status.modified {
  color: var(--accent-cyan);
}
</style>
