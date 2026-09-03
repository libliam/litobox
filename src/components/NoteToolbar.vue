<template>
  <div class="note-toolbar">
    <div class="toolbar-file-name" :title="fileName">
      <el-icon><Document /></el-icon>
      <span>{{ fileName }}</span>
      <span v-if="isModified" class="modified-dot">●</span>
    </div>

    <el-tooltip content="打开本地文件" placement="bottom">
      <el-button
        size="small"
        class="toolbar-action"
        @click="$emit('open-local')"
      >
        <el-icon><FolderOpened /></el-icon>
      </el-button>
    </el-tooltip>

    <el-tooltip :content="isFile ? '保存到原文件 (Ctrl+S)' : '保存 (Ctrl+S)'" placement="bottom">
      <el-button
        v-if="isFile"
        size="small"
        type="primary"
        class="toolbar-action"
        @click="$emit('save')"
      >
        <el-icon><Finished /></el-icon>
      </el-button>
    </el-tooltip>

    <el-tooltip :content="toolbarExpanded ? '收起工具栏' : '展开工具栏'" placement="bottom">
      <el-button
        size="small"
        :class="['toolbar-toggle', { active: toolbarExpanded }]"
        @click="toolbarExpanded = !toolbarExpanded"
      >
        <el-icon><Fold v-if="toolbarExpanded" /><Expand v-else /></el-icon>
      </el-button>
    </el-tooltip>

    <el-tooltip :content="wordWrap ? '关闭自动换行' : '开启自动换行'" placement="bottom">
      <el-button
        size="small"
        :class="['wrap-toggle', { active: wordWrap }]"
        @click="$emit('toggle-wrap')"
      >
        <el-icon><Connection v-if="wordWrap" /><Open v-else /></el-icon>
      </el-button>
    </el-tooltip>

    <template v-if="toolbarExpanded">
      <div class="toolbar-group">
        <el-button size="small" @click="$emit('find')">
          <el-icon><Search /></el-icon> 查找
        </el-button>
        <el-button size="small" @click="$emit('replace')">
          <el-icon><Search /></el-icon> 替换
        </el-button>
        <el-button size="small" @click="$emit('goto-line')">
          <el-icon><Aim /></el-icon> 跳转行
        </el-button>
      </div>

      <div class="toolbar-group">
        <el-select v-model="localLanguage" size="small" style="width: 120px" @change="$emit('language-change', $event)">
          <el-option label="自动检测" value="auto" />
          <el-option label="纯文本" value="plaintext" />
          <el-option label="Shell" value="shell" />
          <el-option label="Bash" value="bash" />
          <el-option label="YAML" value="yaml" />
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
        <el-select v-model="editorTheme" size="small" style="width: 140px" @change="onThemeChange">
          <el-option-group label="深色主题">
            <el-option label="One Dark" value="oneDark" />
            <el-option label="Dracula（推荐）" value="dracula" />
            <el-option label="Monokai" value="monokai" />
            <el-option label="VS Code Dark+" value="vscodeDark" />
            <el-option label="GitHub Dark" value="githubDark" />
            <el-option label="Sublime" value="sublime" />
          </el-option-group>
          <el-option-group label="浅色主题">
            <el-option label="VS Code Light+" value="vscodeLight" />
            <el-option label="GitHub Light" value="githubLight" />
            <el-option label="无（白底）" value="none" />
          </el-option-group>
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
        <el-button size="small" @click="$emit('format')">格式化</el-button>
        <el-button size="small" @click="$emit('save-as')">
          <el-icon><Download /></el-icon> 另存为
        </el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Search, Document, Download, Fold, Expand, Connection, Open, Aim, FolderOpened, Finished } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const props = defineProps<{
  language?: string
  isModified?: boolean
  fileName?: string
  wordWrap?: boolean
  isFile?: boolean
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
  'save': []
  'save-as': []
  'toggle-wrap': []
  'goto-line': []
  'open-local': []
}>()

const toolbarExpanded = ref(false)
const localLanguage = ref('auto')
const editorTheme = ref(store.config.editorTheme || 'oneDark')

watch(() => props.language, (val) => {
  if (val) localLanguage.value = val
}, { immediate: true })

// 监听 store 变化（跨页面切换回来时保持同步）
watch(() => store.config.editorTheme, (val) => {
  if (val) editorTheme.value = val
}, { immediate: true })

const onThemeChange = async (name: string) => {
  await store.saveConfig({ editorTheme: name })
}
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

.toolbar-file-name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 0;
  max-width: 260px;
  flex-shrink: 1;
}

.toolbar-file-name span:not(.modified-dot) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modified-dot {
  color: var(--accent-cyan);
  font-size: 10px;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-toggle,
.wrap-toggle {
  padding: 4px 8px;
}
.toolbar-toggle.active,
.wrap-toggle.active {
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
  color: #fff;
}
</style>
