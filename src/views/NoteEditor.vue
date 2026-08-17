<template>
  <div class="note-editor-container">
    <NoteSidebar @select="handleSelectFile" />

    <div class="editor-main">
      <NoteToolbar
        v-if="currentFile"
        :language="currentLanguage"
        :is-modified="isModified"
        :file-name="currentFile.name"
        @find="handleFind"
        @replace="handleReplace"
        @language-change="handleLanguageChange"
        @sort-lines="handleSortLines"
        @dedup-lines="handleDedupLines"
        @reverse-lines="handleReverseLines"
        @to-upper="handleToUpper"
        @to-lower="handleToLower"
        @format="handleFormat"
        @save="handleSave"
        @save-as="handleSaveAs"
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
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
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

// 加载文件内容
const loadFile = async (item: NoteItem) => {
  currentFile.value = item
  currentLanguage.value = item.language !== 'plaintext' ? item.language : detectLanguage(item.name)

  try {
    if (item.file_path) {
      const result = await noteClient.noteRead(item.file_path)
      editorContent.value = result.content
      originalContent.value = result.content
      isModified.value = false

      if (result.size > 1024 * 1024) {
        ElMessage.warning('文件较大，加载可能较慢')
      }
    }
  } catch (e: any) {
    ElMessage.error(`读取文件失败: ${e}`)
  }
}

// 切换到指定文件（带保存确认）
const switchToFile = async (item: NoteItem) => {
  if (currentFile.value && isModified.value) {
    const action = await ElMessageBox.confirm(
      `"${currentFile.value.name}" 有未保存的修改，是否保存？`,
      '保存确认',
      {
        confirmButtonText: '保存',
        cancelButtonText: '不保存',
        distinguishCancelAndClose: true,
        type: 'warning',
      }
    ).catch(() => 'close')

    if (action === 'confirm') {
      await saveCurrentFile()
    }
  }

  await loadFile(item)
  await noteClient.noteSetLastOpened(item.id)
}

const handleSelectFile = async (item: NoteItem) => {
  await switchToFile(item)
}

const handleContentChange = () => {
  isModified.value = editorContent.value !== originalContent.value

  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    saveCurrentFile()
  }, 1000)
}

// 直接保存到当前文件路径
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

// 手动保存：弹出对话框选择保存位置
const handleSave = async () => {
  if (!currentFile.value) return

  try {
    const result = await noteClient.saveTextWithDialog(editorContent.value, currentFile.value.name)
    if (result === 'cancelled') return

    // 更新当前文件的路径和名称
    const fileName = result.split(/[\\/]/).pop() || currentFile.value.name
    currentFile.value.file_path = result
    currentFile.value.name = fileName
    originalContent.value = editorContent.value
    isModified.value = false

    // 更新数据库记录
    await noteClient.noteRename(currentFile.value.id, fileName)

    ElMessage.success(`已保存到: ${fileName}`)
  } catch (e: any) {
    ElMessage.error(`保存失败: ${e}`)
  }
}

// 另存为
const handleSaveAs = async () => {
  if (!currentFile.value) return

  try {
    const result = await noteClient.saveTextWithDialog(editorContent.value, currentFile.value.name)
    if (result === 'cancelled') return

    const fileName = result.split(/[\\/]/).pop() || currentFile.value.name
    currentFile.value.file_path = result
    currentFile.value.name = fileName
    originalContent.value = editorContent.value
    isModified.value = false

    await noteClient.noteRename(currentFile.value.id, fileName)

    ElMessage.success(`已另存为: ${fileName}`)
  } catch (e: any) {
    ElMessage.error(`另存为失败: ${e}`)
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

// 初始化：进入文本编辑器时自动加载草稿或上次打开的文件
onMounted(async () => {
  try {
    const lastOpenedId = await noteClient.noteGetLastOpened()
    if (lastOpenedId) {
      const items = await noteClient.noteList(null)
      const lastItem = items.find(item => item.id === lastOpenedId)
      if (lastItem && lastItem.type === 'file' && lastItem.file_path) {
        await loadFile(lastItem)
        return
      }
    }

    const draft = await noteClient.noteEnsureDraft()
    await loadFile(draft)
    await noteClient.noteSetLastOpened(draft.id)
  } catch (e: any) {
    ElMessage.error(`加载笔记失败: ${e}`)
  }
})

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
  height: 100%; /* 与全局 tool-container 一致，100vh 会溢出 main 底部被裁剪 */
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
  padding: 8px 8px 32px;
}
</style>
