<template>
  <div class="note-editor-container">
    <NoteSidebar @select="handleSelectFile" />

    <div class="editor-main">
      <NoteToolbar
        v-if="currentFile"
        :language="currentLanguage"
        :is-modified="isModified"
        @find="handleFind"
        @replace="handleReplace"
        @language-change="handleLanguageChange"
        @sort-lines="handleSortLines"
        @dedup-lines="handleDedupLines"
        @reverse-lines="handleReverseLines"
        @to-upper="handleToUpper"
        @to-lower="handleToLower"
        @format="handleFormat"
      />

      <div v-if="!currentFile" class="editor-empty">
        <el-icon :size="48" color="var(--text-muted)"><Document /></el-icon>
        <p>选择或创建一个笔记开始编辑</p>
      </div>

      <div v-else class="editor-wrapper">
        <CodeMirrorEditor
          ref="editorRef"
          v-model="editorContent"
          :language="currentLanguage"
          @change="handleContentChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Document } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteSidebar from '@/components/NoteSidebar.vue'
import NoteToolbar from '@/components/NoteToolbar.vue'
import CodeMirrorEditor from '@/components/CodeMirrorEditor.vue'

const currentFile = ref<NoteItem | null>(null)
const editorContent = ref('')
const originalContent = ref('')
const isModified = ref(false)
const currentLanguage = ref('plaintext')
const editorRef = ref()
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

// 根据文件扩展名推断语言
const detectLanguage = (filename: string): string => {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const langMap: Record<string, string> = {
    js: 'javascript', ts: 'typescript', json: 'json',
    html: 'html', htm: 'html', css: 'css', md: 'markdown',
    xml: 'xml', sql: 'sql', py: 'python', rs: 'rust',
    txt: 'plaintext', log: 'plaintext',
  }
  return langMap[ext] || 'plaintext'
}

const handleSelectFile = async (item: NoteItem) => {
  // 保存当前文件
  if (currentFile.value && isModified.value) {
    await saveCurrentFile()
  }

  currentFile.value = item
  currentLanguage.value = item.language !== 'plaintext' ? item.language : detectLanguage(item.name)

  try {
    if (item.file_path) {
      const result = await noteClient.noteRead(item.file_path)
      editorContent.value = result.content
      originalContent.value = result.content

      if (result.size > 1024 * 1024) {
        ElMessage.warning('文件较大，加载可能较慢')
      }
    }
  } catch (e: any) {
    ElMessage.error(`读取文件失败: ${e}`)
  }
}

const handleContentChange = () => {
  isModified.value = editorContent.value !== originalContent.value

  // 自动保存：停止输入 1 秒后保存
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    saveCurrentFile()
  }, 1000)
}

const saveCurrentFile = async () => {
  if (!currentFile.value || !currentFile.value.file_path || !isModified.value) return

  try {
    await noteClient.noteWrite(currentFile.value.file_path, editorContent.value)
    originalContent.value = editorContent.value
    isModified.value = false
  } catch (e: any) {
    ElMessage.error(`保存失败: ${e}`)
  }
}

const handleLanguageChange = (lang: string) => {
  currentLanguage.value = lang === 'auto' ? detectLanguage(currentFile.value?.name || '') : lang
  editorRef.value?.updateLanguage(currentLanguage.value)
}

const handleFind = () => editorRef.value?.openFind()
const handleReplace = () => editorRef.value?.openReplace()
const handleSortLines = () => editorRef.value?.sortLines()
const handleDedupLines = () => editorRef.value?.dedupLines()
const handleReverseLines = () => editorRef.value?.reverseLines()
const handleToUpper = () => editorRef.value?.toUpperCase()
const handleToLower = () => editorRef.value?.toLowerCase()
const handleFormat = () => editorRef.value?.formatCode()

// 关闭时保存
onUnmounted(() => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  if (currentFile.value && isModified.value) {
    saveCurrentFile()
  }
})
</script>

<style scoped>
.note-editor-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.editor-empty p {
  margin-top: 16px;
  font-size: 14px;
}

.editor-wrapper {
  flex: 1;
  overflow: hidden;
  padding: 8px;
}
</style>
