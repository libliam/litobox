<template>
  <div class="note-editor-container">
    <NoteSidebar @select="handleSelectFile" />

    <div class="editor-main">
      <NoteToolbar
        v-if="currentFile"
        :language="currentLanguage"
        :is-modified="isModified"
        :file-name="currentFile.name"
        :word-wrap="wordWrap"
        @find="handleFind"
        @replace="handleReplace"
        @language-change="handleLanguageChange"
        @sort-lines="handleSortLines"
        @dedup-lines="handleDedupLines"
        @reverse-lines="handleReverseLines"
        @to-upper="handleToUpper"
        @to-lower="handleToLower"
        @format="handleFormat"
        @save-as="handleSaveAs"
        @toggle-wrap="handleToggleWrap"
        @goto-line="showGotoLineDlg = true"
      />

      <div v-if="!currentFile" class="editor-empty">
        <el-icon :size="48" color="var(--text-muted)"><Document /></el-icon>
        <p>选择或创建一个笔记开始编辑</p>
      </div>

      <template v-else>
        <div class="editor-wrapper">
          <CodeMirrorEditor
            ref="editorRef"
            v-model="editorContent"
            :language="currentLanguage"
            @change="handleContentChange"
            @status="handleStatus"
          />
        </div>
        <div class="editor-statusbar">
          <div class="status-left">
            <span class="status-item" :title="isModified ? '有未保存的修改（自动保存中）' : '已自动保存'">
              <el-icon :size="12"><component :is="isModified ? Warning : CircleCheck" /></el-icon>
              {{ isModified ? '未保存' : '已保存' }}
            </span>
            <span class="status-sep">|</span>
            <span class="status-item">{{ langLabel }}</span>
            <span class="status-sep">|</span>
            <span class="status-item">{{ status.lineEnding }}</span>
          </div>
          <div class="status-right">
            <span v-if="status.selectedCount > 0" class="status-item">{{ status.selectedCount }} 字符 · {{ status.selectedLines }} 行</span>
            <span v-if="status.selectedCount > 0" class="status-sep">|</span>
            <span class="status-item">行 {{ status.line }} · 列 {{ status.column }}</span>
            <span class="status-sep">|</span>
            <span class="status-item">{{ status.totalLines }} 行 · {{ formatSize(status.totalChars) }}</span>
          </div>
        </div>
      </template>
    </div>

    <el-dialog
      v-model="showGotoLineDlg"
      title="跳转到行"
      width="340px"
      :close-on-click-modal="false"
      append-to-body
      @open="gotoLineInput = String(editorRef.value?.getCurrentLine() ?? 1)"
    >
      <div class="goto-line-body">
        <span>输入行号 (1 - {{ editorRef?.getLineCount() ?? 0 }})</span>
        <el-input
          v-model="gotoLineInput"
          type="number"
          placeholder="例如: 28"
          @keyup.enter="doGotoLine"
          ref="gotoLineInputRef"
        />
      </div>
      <template #footer>
        <el-button @click="showGotoLineDlg = false">取消</el-button>
        <el-button type="primary" @click="doGotoLine">跳转</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Document, Warning, CircleCheck } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteSidebar from '@/components/NoteSidebar.vue'
import NoteToolbar from '@/components/NoteToolbar.vue'
import CodeMirrorEditor, { type EditorStatus } from '@/components/CodeMirrorEditor.vue'

const currentFile = ref<NoteItem | null>(null)
const editorContent = ref('')
const originalContent = ref('')
const isModified = ref(false)
const currentLanguage = ref('plaintext')
const editorRef = ref()
const wordWrap = ref(false)
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

// 编辑器状态（行号、选中文本等）
const status = reactive<EditorStatus>({
  line: 1,
  column: 1,
  selectedCount: 0,
  selectedLines: 0,
  totalLines: 1,
  totalChars: 0,
  lineEnding: 'CRLF',
})

// 语言代码 → 展示名
const langLabel = computed(() => {
  const map: Record<string, string> = {
    plaintext: '纯文本', javascript: 'JavaScript', typescript: 'TypeScript',
    json: 'JSON', html: 'HTML', css: 'CSS', markdown: 'Markdown',
    xml: 'XML', sql: 'SQL', python: 'Python', rust: 'Rust',
    yaml: 'YAML', shell: 'Shell', bash: 'Bash',
  }
  return map[currentLanguage.value] || currentLanguage.value
})

// 字节数 → 友好大小
const formatSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

const handleStatus = (info: EditorStatus) => {
  Object.assign(status, info)
}

// 根据文件扩展名推断语言
const detectLanguage = (filename: string): string => {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const langMap: Record<string, string> = {
    js: 'javascript', mjs: 'javascript', cjs: 'javascript',
    ts: 'typescript', mts: 'typescript', cts: 'typescript',
    json: 'json', map: 'json',
    html: 'html', htm: 'html', xml: 'xml',
    css: 'css', scss: 'css', less: 'css',
    md: 'markdown', markdown: 'markdown',
    sql: 'sql',
    py: 'python', pyw: 'python',
    rs: 'rust',
    yaml: 'yaml', yml: 'yaml',
    sh: 'shell', bash: 'bash', zsh: 'shell', fish: 'shell',
    ps1: 'shell', cmd: 'shell', bat: 'shell',
    conf: 'shell', env: 'shell',
    txt: 'plaintext', log: 'plaintext', properties: 'plaintext',
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
const handleToggleWrap = () => {
  editorRef.value?.toggleWordWrap()
  wordWrap.value = editorRef.value?.isWordWrap() ?? false
}

// 跳转到行
const showGotoLineDlg = ref(false)
const gotoLineInput = ref('')
const gotoLineInputRef = ref()
const doGotoLine = () => {
  const n = parseInt(gotoLineInput.value, 10)
  if (!n || n < 1) {
    ElMessage.warning('请输入有效的行号')
    return
  }
  const total = editorRef.value?.getLineCount() ?? 0
  if (n > total) {
    ElMessage.warning(`文件共 ${total} 行`)
    return
  }
  const actual = editorRef.value?.gotoLine(n) ?? n
  showGotoLineDlg.value = false
  if (actual !== n) ElMessage.info(`已跳转到第 ${actual} 行`)
}

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
  height: 100%;
  overflow: hidden;
}

.editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
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
  min-height: 0;
}

/* 底部状态栏 — VS Code 风格 */
.editor-statusbar {
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-item {
  font-size: 12px;
  color: var(--text-secondary);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.status-item .el-icon {
  flex-shrink: 0;
}

.status-sep {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.5;
}

.goto-line-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
