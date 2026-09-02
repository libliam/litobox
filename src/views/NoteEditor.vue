<template>
  <div class="note-editor-container">
    <NoteSidebar @select="handleSelectFile" />

    <div class="editor-main">
      <NoteToolbar
        v-if="activeDoc"
        :language="activeDoc.language"
        :is-modified="isActiveDocModified"
        :file-name="activeDoc.name"
        :word-wrap="activeDoc.wrap"
        :is-file="activeDoc.kind === 'file'"
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
        @toggle-wrap="handleToggleWrap"
        @goto-line="showGotoLineDlg = true"
        @open-local="openLocalFile"
      />

      <!-- 多文件 Tab 栏 -->
      <div v-if="docs.length > 0" class="editor-tabs" @wheel.prevent="handleTabWheel">
        <div
          v-for="doc in docs"
          :key="doc.key"
          class="editor-tab"
          :class="{ active: doc.key === activeKey }"
          :title="doc.path"
          @click="switchDoc(doc)"
          @mousedown.middle.prevent="closeDoc(doc)"
        >
          <el-icon v-if="doc.kind === 'note'" class="tab-icon"><Document /></el-icon>
          <el-icon v-else class="tab-icon file-icon"><FolderOpened /></el-icon>
          <span class="tab-name">{{ doc.name }}</span>
          <span v-if="doc.content !== doc.original" class="tab-dot">●</span>
          <span class="tab-close" @click.stop="closeDoc(doc)">
            <el-icon :size="12"><Close /></el-icon>
          </span>
        </div>
      </div>

      <div v-if="!activeDoc" class="editor-empty">
        <el-icon :size="48" color="var(--text-muted)"><Document /></el-icon>
        <p>选择或创建一个笔记开始编辑</p>
        <el-button type="primary" plain size="small" @click="openLocalFile">
          <el-icon><FolderOpened /></el-icon>&nbsp;打开本地文件
        </el-button>
      </div>

      <template v-else>
        <div class="editor-wrapper">
          <CodeMirrorEditor
            :key="activeDoc.key"
            ref="editorRef"
            v-model="activeContent"
            :language="activeDoc.language"
            :initial-wrap="activeDoc.wrap"
            @change="handleContentChange"
            @status="handleStatus"
            @goto-line="showGotoLineDlg = true"
            @save="handleSave"
          />
        </div>
        <div class="editor-statusbar">
          <div class="status-left">
            <span class="status-item" :title="saveStatusTitle">
              <el-icon :size="12"><component :is="isActiveDocModified ? Warning : CircleCheck" /></el-icon>
              {{ isActiveDocModified ? (activeDoc.kind === 'file' ? '未保存' : '未保存(自动保存中)') : '已保存' }}
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
import { ref, reactive, computed, onMounted, onUnmounted, onDeactivated } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Document, Warning, CircleCheck, FolderOpened, Close } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteSidebar from '@/components/NoteSidebar.vue'
import NoteToolbar from '@/components/NoteToolbar.vue'
import CodeMirrorEditor, { type EditorStatus } from '@/components/CodeMirrorEditor.vue'

// ===================== 多 Tab 文档模型 =====================
interface EditorDoc {
  key: string              // note:{id} 或 file:{path}
  kind: 'note' | 'file'    // note=笔记库(自动保存) / file=外部本地文件(手动保存)
  noteId?: number          // kind=note 时的笔记 id
  name: string
  path: string
  language: string         // 当前高亮语言（工具栏可改）
  wrap: boolean            // 自动换行状态（切 tab 保留）
  encoding: string         // 读取时检测的编码（file 类保存时参考）
  content: string
  original: string         // 已保存到磁盘的内容（用于 dirty 判断）
}

const docs = ref<EditorDoc[]>([])
const activeKey = ref<string | null>(null)
const editorRef = ref()
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

const activeDoc = computed<EditorDoc | null>(
  () => docs.value.find(d => d.key === activeKey.value) ?? null
)

const isActiveDocModified = computed(() => {
  const d = activeDoc.value
  return !!d && d.content !== d.original
})

const saveStatusTitle = computed(() => {
  if (!activeDoc.value) return ''
  if (activeDoc.value.kind === 'file') {
    return isActiveDocModified.value ? '有未保存的修改，按 Ctrl+S 保存到原文件' : '已保存'
  }
  return isActiveDocModified.value ? '有未保存的修改（自动保存中）' : '已自动保存'
})

// v-model：绑定到当前激活文档
const activeContent = computed({
  get: () => activeDoc.value?.content ?? '',
  set: (v: string) => {
    if (activeDoc.value) activeDoc.value.content = v
  },
})

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
  return map[activeDoc.value?.language ?? 'plaintext'] || activeDoc.value?.language || '纯文本'
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

const makeDocKey = (kind: 'note' | 'file', idOrPath: number | string): string =>
  kind === 'note' ? `note:${idOrPath}` : `file:${idOrPath}`

// ===================== 保存 =====================

// 写回磁盘，成功后同步 original。note 类自动保存、file 类手动保存都走这里。
const saveDoc = async (doc: EditorDoc): Promise<boolean> => {
  if (doc.content === doc.original) return true
  try {
    await noteClient.noteWrite(doc.path, doc.content)
    doc.original = doc.content
    return true
  } catch (e: any) {
    ElMessage.error(`保存 "${doc.name}" 失败: ${e}`)
    return false
  }
}

const saveAllModifiedDocs = async () => {
  for (const doc of docs.value) {
    if (doc.kind === 'note' && doc.content !== doc.original) {
      await saveDoc(doc)  // 笔记自动保存，静默
    }
  }
}

// 手动保存（Ctrl+S / 工具栏保存按钮）：file 类保存到原文件，note 类立即落盘
const handleSave = async () => {
  const doc = activeDoc.value
  if (!doc) return
  const ok = await saveDoc(doc)
  if (ok) {
    if (doc.kind === 'file') ElMessage.success(`已保存: ${doc.name}`)
    if (autoSaveTimer) { clearTimeout(autoSaveTimer); autoSaveTimer = null }
  }
}

// 笔记库文件变更：1s 防抖自动保存
const handleContentChange = () => {
  const doc = activeDoc.value
  if (!doc) return
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    autoSaveTimer = null
    if (doc.kind === 'note' && doc.content !== doc.original) saveDoc(doc)
  }, 1000)
}

// 另存为
const handleSaveAs = async () => {
  const doc = activeDoc.value
  if (!doc) return

  try {
    const result = await noteClient.saveTextWithDialog(doc.content, doc.name)
    if (result === 'cancelled') return

    const fileName = result.split(/[\\/]/).pop() || doc.name
    doc.path = result
    doc.name = fileName
    doc.original = doc.content

    // 笔记库文件另存为新名字时，同步重命名笔记条目
    if (doc.kind === 'note' && doc.noteId) {
      await noteClient.noteRename(doc.noteId, fileName)
    }

    ElMessage.success(`已另存为: ${fileName}`)
  } catch (e: any) {
    ElMessage.error(`另存为失败: ${e}`)
  }
}

// ===================== 打开 / 切换 / 关闭 =====================

// 打开笔记库笔记（不存在则新建 tab）
const openNote = async (item: NoteItem) => {
  const key = makeDocKey('note', item.id)
  let doc = docs.value.find(d => d.key === key)

  if (!doc) {
    if (!item.file_path) {
      ElMessage.warning('该笔记没有文件内容')
      return
    }
    try {
      const result = await noteClient.noteRead(item.file_path)
      doc = {
        key,
        kind: 'note',
        noteId: item.id,
        name: item.name,
        path: item.file_path,
        language: item.language !== 'plaintext' ? item.language : detectLanguage(item.name),
        wrap: false,
        encoding: result.encoding,
        content: result.content,
        original: result.content,
      }
      docs.value.push(doc)
      if (result.size > 1024 * 1024) ElMessage.warning('文件较大，加载可能较慢')
    } catch (e: any) {
      ElMessage.error(`读取文件失败: ${e}`)
      return
    }
  } else {
    // 已在 tab 中打开：同步侧边栏可能发生的重命名/移动
    doc.name = item.name
    if (item.file_path) doc.path = item.file_path
  }

  await switchDoc(doc)
  if (item.type === 'file') await noteClient.noteSetLastOpened(item.id)
}

const handleSelectFile = (item: NoteItem) => openNote(item)

// 打开本地文件（外部文件系统，非笔记库）
const openLocalFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: false,
      filters: [
        { name: '文本文件', extensions: ['txt', 'log', 'md', 'json', 'yml', 'yaml', 'xml', 'html', 'css', 'js', 'ts', 'py', 'rs', 'sql', 'sh', 'bat', 'ps1', 'ini', 'conf', 'properties', 'env', 'csv'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    })
    if (!selected || Array.isArray(selected)) return

    const path = selected as string
    const key = makeDocKey('file', path)
    let doc = docs.value.find(d => d.key === key)
    if (doc) {
      await switchDoc(doc)
      return
    }

    const result = await noteClient.noteRead(path)
    const name = path.split(/[\\/]/).pop() || path
    // note_write 固定写 UTF-8；非 UTF-8 文件先告知（避免用户误以为保留原编码）
    const enc = (result.encoding || '').toUpperCase()
    if (enc && enc !== 'UTF-8' && enc !== 'ASCII' && !enc.includes('UTF-16')) {
      ElMessage.warning(`文件编码为 ${result.encoding}，保存时将转为 UTF-8`)
    }
    doc = {
      key,
      kind: 'file',
      name,
      path,
      language: detectLanguage(name),
      wrap: false,
      encoding: result.encoding,
      content: result.content,
      original: result.content,
    }
    docs.value.push(doc)
    if (result.size > 1024 * 1024) ElMessage.warning('文件较大，加载可能较慢')
    await switchDoc(doc)
  } catch (e: any) {
    ElMessage.error(`打开文件失败: ${e}`)
  }
}

// 切换激活 tab
const switchDoc = async (doc: EditorDoc): Promise<boolean> => {
  if (activeKey.value === doc.key) return true

  const prev = activeDoc.value
  if (prev && prev.key !== doc.key && prev.content !== prev.original) {
    // note 自动保存静默落盘；file 询问是否保存
    if (prev.kind === 'note') {
      await saveDoc(prev)
    } else {
      const action = await ElMessageBox.confirm(
        `"${prev.name}" 有未保存的修改，是否保存？`,
        '保存确认',
        {
          confirmButtonText: '保存',
          cancelButtonText: '不保存',
          distinguishCancelAndClose: true,
          type: 'warning',
        }
      ).catch(() => 'close')
      if (action === 'confirm') {
        const ok = await saveDoc(prev)
        if (!ok) return false
      } else if (action === 'close') {
        return false  // 用户取消切换
      }
    }
  }

  if (autoSaveTimer) { clearTimeout(autoSaveTimer); autoSaveTimer = null }
  activeKey.value = doc.key
  // 重置状态栏（编辑器重建后会重新上报）
  Object.assign(status, { line: 1, column: 1, selectedCount: 0, selectedLines: 0 })
  return true
}

// 关闭 tab
const closeDoc = async (doc: EditorDoc) => {
  if (activeKey.value !== doc.key) {
    docs.value = docs.value.filter(d => d !== doc)
    return
  }

  if (doc.content !== doc.original) {
    let shouldClose = true
    if (doc.kind === 'note') {
      await saveDoc(doc)  // 笔记自动保存，静默
    } else {
      const action = await ElMessageBox.confirm(
        `"${doc.name}" 有未保存的修改，是否保存？`,
        '保存确认',
        {
          confirmButtonText: '保存',
          cancelButtonText: '不保存',
          distinguishCancelAndClose: true,
          type: 'warning',
        }
      ).catch(() => 'close')
      if (action === 'confirm') {
        shouldClose = await saveDoc(doc)
      } else if (action === 'close') {
        return  // 取消关闭
      }
    }
    if (!shouldClose) return
  }

  const idx = docs.value.indexOf(doc)
  docs.value = docs.value.filter(d => d !== doc)

  // 激活相邻 tab
  if (activeKey.value === doc.key) {
    const next = docs.value[Math.min(idx, docs.value.length - 1)]
    activeKey.value = next ? next.key : null
    Object.assign(status, { line: 1, column: 1, selectedCount: 0, selectedLines: 0 })
  }
}

const handleTabWheel = (e: WheelEvent) => {
  const el = e.currentTarget as HTMLElement
  el.scrollLeft += e.deltaY
}

// ===================== 编辑器操作 =====================

const handleLanguageChange = (lang: string) => {
  const doc = activeDoc.value
  if (!doc) return
  doc.language = lang === 'auto' ? detectLanguage(doc.name) : lang
  editorRef.value?.updateLanguage(doc.language)
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
  if (activeDoc.value) activeDoc.value.wrap = editorRef.value?.isWordWrap() ?? false
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

// 初始化：进入文本编辑器时自动加载上次打开的文件或草稿
onMounted(async () => {
  try {
    const lastOpenedId = await noteClient.noteGetLastOpened()
    if (lastOpenedId) {
      const items = await noteClient.noteList(null)
      const lastItem = items.find(item => item.id === lastOpenedId)
      if (lastItem && lastItem.type === 'file' && lastItem.file_path) {
        await openNote(lastItem)
        return
      }
    }

    const draft = await noteClient.noteEnsureDraft()
    await openNote(draft)
  } catch (e: any) {
    ElMessage.error(`加载笔记失败: ${e}`)
  }
})

// KeepAlive 缓存：切走(deactivated)与卸载(unmounted)时都要落盘未保存的笔记
// (AGENTS 经验 12: onMounted 只在首次挂载触发，离开页面的清理用 onDeactivated)
onDeactivated(() => {
  if (autoSaveTimer) { clearTimeout(autoSaveTimer); autoSaveTimer = null }
  saveAllModifiedDocs()
})

onUnmounted(() => {
  if (autoSaveTimer) { clearTimeout(autoSaveTimer); autoSaveTimer = null }
  saveAllModifiedDocs()
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

/* 多文件 Tab 栏 */
.editor-tabs {
  display: flex;
  overflow-x: auto;
  overflow-y: hidden;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  scrollbar-width: thin;
  max-height: 34px;
}

.editor-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  height: 33px;
  min-width: 0;
  max-width: 220px;
  cursor: pointer;
  border-right: 1px solid var(--border-color);
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
  user-select: none;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
  border-top: 2px solid transparent;
}

.editor-tab:hover {
  background: var(--hover-bg);
}

.editor-tab.active {
  background: var(--bg-card);
  color: var(--accent-cyan);
  border-top: 2px solid var(--accent-cyan);
  border-bottom: 2px solid var(--bg-card);
}

.editor-tab .tab-icon {
  color: var(--text-muted);
  flex-shrink: 0;
  display: flex;
}

.editor-tab .file-icon {
  color: var(--accent-yellow, #f0b90b);
}

.editor-tab .tab-name {
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.editor-tab .tab-dot {
  color: var(--accent-cyan);
  font-size: 8px;
  flex-shrink: 0;
}

.editor-tab .tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  color: var(--text-muted);
  opacity: 0;
  flex-shrink: 0;
}

.editor-tab:hover .tab-close {
  opacity: 0.7;
}

.editor-tab.active .tab-close {
  opacity: 0.5;
}

.editor-tab .tab-close:hover {
  background: rgba(239, 68, 68, 0.2);
  color: var(--accent-red);
  opacity: 1;
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
