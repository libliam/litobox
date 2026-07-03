<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">代码片段管理</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>保存常用代码片段，支持分类、搜索、导入导出</p>
                <p>数据存储在本地浏览器中</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleImport">导入</el-button>
          <el-button size="small" @click="handleExport">导出</el-button>
          <el-button size="small" type="primary" @click="handleNew">新建</el-button>
        </div>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="tool-card">
      <div class="card-body" style="padding: 12px 20px">
        <el-input
          v-model="searchText"
          placeholder="搜索标题、语言或代码内容..."
          clearable
          size="small"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>
    </div>

    <!-- 片段列表 + 编辑区 -->
    <div class="snippet-layout">
      <!-- 左侧列表 -->
      <div class="tool-card snippet-list-card">
        <div class="card-header">
          <span class="card-title">片段列表 ({{ filteredSnippets.length }})</span>
        </div>
        <div class="card-body snippet-list-body">
          <div v-if="filteredSnippets.length === 0" class="empty-list">
            {{ searchText ? '无匹配结果' : '暂无片段，点击"新建"添加' }}
          </div>
          <div
            v-for="snippet in filteredSnippets"
            :key="snippet.id"
            class="snippet-item"
            :class="{ active: selectedId === snippet.id }"
            @click="selectSnippet(snippet)"
          >
            <div class="snippet-item-header">
              <span class="snippet-item-title">{{ snippet.title }}</span>
              <span class="snippet-lang">{{ snippet.lang }}</span>
            </div>
            <div class="snippet-item-preview">{{ snippet.content.slice(0, 80) }}</div>
          </div>
        </div>
      </div>

      <!-- 右侧编辑/查看 -->
      <div class="tool-card snippet-editor-card">
        <div v-if="editingSnippet" class="card-header">
          <div class="header-left">
            <span class="card-title">{{ isEditing ? '编辑片段' : snippetTitle }}</span>
          </div>
          <div class="card-actions">
            <el-button v-if="!isEditing" size="small" @click="startEdit">编辑</el-button>
            <el-button v-if="isEditing" size="small" @click="handleSave">保存</el-button>
            <el-button v-if="isEditing" size="small" @click="cancelEdit">取消</el-button>
            <el-button v-if="!isEditing" size="small" @click="handleCopySnippet">复制</el-button>
            <el-button v-if="!isEditing" size="small" type="danger" @click="handleDelete">删除</el-button>
          </div>
        </div>
        <div v-if="editingSnippet" class="card-body snippet-editor-body">
          <!-- 标题和语言 -->
          <div class="editor-meta">
            <el-input
              v-model="editTitle"
              placeholder="片段标题"
              size="small"
              :disabled="!isEditing"
              style="flex: 1"
            />
            <el-select v-model="editLang" size="small" :disabled="!isEditing" style="width: 140px; margin-left: 8px">
              <el-option label="JavaScript" value="JavaScript" />
              <el-option label="TypeScript" value="TypeScript" />
              <el-option label="Python" value="Python" />
              <el-option label="Java" value="Java" />
              <el-option label="Go" value="Go" />
              <el-option label="Rust" value="Rust" />
              <el-option label="SQL" value="SQL" />
              <el-option label="Shell" value="Shell" />
              <el-option label="HTML" value="HTML" />
              <el-option label="CSS" value="CSS" />
              <el-option label="Vue" value="Vue" />
              <el-option label="JSON" value="JSON" />
              <el-option label="YAML" value="YAML" />
              <el-option label="Markdown" value="Markdown" />
              <el-option label="其他" value="Other" />
            </el-select>
          </div>
          <!-- 代码内容 -->
          <el-input
            v-model="editContent"
            type="textarea"
            :rows="16"
            placeholder="代码内容..."
            resize="vertical"
            :disabled="!isEditing"
            class="code-textarea"
          />
          <!-- 备注 -->
          <el-input
            v-model="editNote"
            type="textarea"
            :rows="2"
            placeholder="备注（可选）"
            :disabled="!isEditing"
            style="margin-top: 8px"
          />
        </div>
        <div v-else class="card-body snippet-empty">
          <p>选择左侧列表中的片段，或点击"新建"创建新片段</p>
        </div>
      </div>
    </div>

    <!-- 隐藏的文件输入用于导入 -->
    <input ref="fileInput" type="file" accept=".json" class="file-input" @change="handleFileImport" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Search } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import * as db from '@/utils/dbClient'
import { saveFileWithDialog } from '@/utils/fileSaver'

const store = useToolboxStore()

// ============ 数据结构 ============
interface Snippet {
  id: string
  title: string
  lang: string
  content: string
  note: string
  createdAt: number
  updatedAt: number
}

const snippets = ref<Snippet[]>([])
const selectedId = ref<string | null>(null)
const searchText = ref('')
const isEditing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

// 编辑状态
const editTitle = ref('')
const editLang = ref('JavaScript')
const editContent = ref('')
const editNote = ref('')

const editingSnippet = computed(() => snippets.value.find(s => s.id === selectedId.value))
const snippetTitle = computed(() => editingSnippet.value?.title || '')

// 搜索过滤
const filteredSnippets = computed(() => {
  if (!searchText.value) return snippets.value
  const q = searchText.value.toLowerCase()
  return snippets.value.filter(s =>
    s.title.toLowerCase().includes(q) ||
    s.lang.toLowerCase().includes(q) ||
    s.content.toLowerCase().includes(q) ||
    s.note.toLowerCase().includes(q)
  )
})

// ============ 持久化 ============
const loadSnippets = async () => {
  try {
    const rows = await db.listSnippets()
    snippets.value = rows.map(r => ({
      id: r.id,
      title: r.title,
      lang: r.lang,
      content: r.content,
      note: r.note,
      createdAt: new Date(r.created_at).getTime(),
      updatedAt: new Date(r.updated_at).getTime(),
    }))
  } catch {
    snippets.value = []
  }
}

onMounted(() => {
  loadSnippets()
})

// ============ 操作 ============
const selectSnippet = (snippet: Snippet) => {
  selectedId.value = snippet.id
  isEditing.value = false
  editTitle.value = snippet.title
  editLang.value = snippet.lang
  editContent.value = snippet.content
  editNote.value = snippet.note
}

const handleNew = async () => {
  const now = Date.now()
  const id = now.toString(36) + Math.random().toString(36).slice(2, 7)
  const newSnippet: Snippet = {
    id,
    title: '未命名片段',
    lang: 'JavaScript',
    content: '',
    note: '',
    createdAt: now,
    updatedAt: now
  }
  try {
    await db.saveSnippet({
      id, title: newSnippet.title, lang: newSnippet.lang,
      content: newSnippet.content, note: newSnippet.note,
      created_at: new Date(now).toISOString(),
      updated_at: new Date(now).toISOString(),
    })
    snippets.value.unshift(newSnippet)
    selectSnippet(newSnippet)
    isEditing.value = true
    ElMessage.success('已创建新片段')
  } catch {
    ElMessage.error('创建失败')
  }
}

const startEdit = () => {
  isEditing.value = true
}

const cancelEdit = () => {
  if (editingSnippet.value) {
    editTitle.value = editingSnippet.value.title
    editLang.value = editingSnippet.value.lang
    editContent.value = editingSnippet.value.content
    editNote.value = editingSnippet.value.note
  }
  isEditing.value = false
}

const handleSave = async () => {
  if (!editTitle.value.trim()) {
    ElMessage.warning('标题不能为空')
    return
  }
  if (!editContent.value.trim()) {
    ElMessage.warning('代码内容不能为空')
    return
  }
  const snippet = snippets.value.find(s => s.id === selectedId.value)
  if (snippet) {
    snippet.title = editTitle.value.trim()
    snippet.lang = editLang.value
    snippet.content = editContent.value
    snippet.note = editNote.value
    snippet.updatedAt = Date.now()
    try {
      await db.saveSnippet({
        id: snippet.id, title: snippet.title, lang: snippet.lang,
        content: snippet.content, note: snippet.note,
        created_at: new Date(snippet.createdAt).toISOString(),
        updated_at: new Date(snippet.updatedAt).toISOString(),
      })
      isEditing.value = false
      ElMessage.success('保存成功')
      store.addHistory({ tool: 'snippet', action: 'save', inputPreview: snippet.title, outputPreview: snippet.lang, inputFull: snippet.title, outputFull: snippet.content })
    } catch {
      ElMessage.error('保存失败')
    }
  }
}

const handleDelete = async () => {
  try {
    await ElMessageBox.confirm('确定要删除这个片段吗？', '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning'
    })
    const id = selectedId.value
    if (!id) return
    await db.deleteSnippet(id)
    snippets.value = snippets.value.filter(s => s.id !== id)
    selectedId.value = null
    ElMessage.success('已删除')
  } catch {
    // 用户取消
  }
}

const handleCopySnippet = async () => {
  if (!editingSnippet.value) return
  try {
    await navigator.clipboard.writeText(editingSnippet.value.content)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 导入导出 ============
const handleExport = async () => {
  if (snippets.value.length === 0) {
    ElMessage.warning('没有可导出的片段')
    return
  }
  const data = JSON.stringify(snippets.value, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  await saveFileWithDialog(blob, `litobox-snippets-${new Date().toISOString().slice(0, 10)}.json`, 'json')
}

const handleImport = () => {
  fileInput.value?.click()
}

const handleFileImport = async (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (ev) => {
    try {
      const imported = JSON.parse(ev.target?.result as string) as Snippet[]
      if (!Array.isArray(imported)) {
        ElMessage.error('无效的导入文件格式')
        return
      }
      let added = 0
      for (const s of imported) {
        if (s.title && s.content) {
          const exists = snippets.value.some(existing => existing.id === s.id)
          if (!exists) {
            const now = Date.now()
            const id = s.id || now.toString(36) + Math.random().toString(36).slice(2, 7)
            const snippet: Snippet = {
              id, title: s.title, lang: s.lang || 'Other',
              content: s.content, note: s.note || '',
              createdAt: s.createdAt || now, updatedAt: s.updatedAt || now
            }
            try {
              await db.saveSnippet({
                id: snippet.id, title: snippet.title, lang: snippet.lang,
                content: snippet.content, note: snippet.note,
                created_at: new Date(snippet.createdAt).toISOString(),
                updated_at: new Date(snippet.updatedAt).toISOString(),
              })
              snippets.value.unshift(snippet)
              added++
            } catch { /* 跳过冲突 */ }
          }
        }
      }
      ElMessage.success(`成功导入 ${added} 个片段`)
      store.addHistory({ tool: 'snippet', action: 'import', inputPreview: `${added} snippets`, outputPreview: 'imported', inputFull: file.name, outputFull: `${added} snippets imported` })
    } catch {
      ElMessage.error('导入失败，文件格式错误')
    }
  }
  reader.readAsText(file)
  if (fileInput.value) fileInput.value.value = ''
}
</script>

<style scoped>
/* ===== 一级卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

/* ===== 布局 ===== */
.snippet-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 16px;
  align-items: start;
}

.snippet-layout .tool-card {
  margin-bottom: 0;
}

/* ===== 片段列表 ===== */
.snippet-list-body {
  padding: 8px 12px;
  max-height: 600px;
  overflow-y: auto;
}

.empty-list {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  font-size: 13px;
}

.snippet-item {
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
  margin-bottom: 4px;
  border: 1px solid transparent;
}

.snippet-item:hover {
  background: rgba(0, 212, 255, 0.05);
}

.snippet-item.active {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.3);
}

.snippet-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.snippet-item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 8px;
}

.snippet-lang {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
  white-space: nowrap;
}

.snippet-item-preview {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ===== 编辑器 ===== */
.snippet-editor-body {
  padding: 16px 20px;
}

.editor-meta {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.code-textarea {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
}

.code-textarea :deep(.el-textarea__inner) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  background: var(--bg-input);
  line-height: 1.6;
}

.snippet-empty {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-muted);
}

.snippet-empty p {
  margin: 0;
}

/* ===== 文件输入 ===== */
.file-input {
  display: none;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .snippet-layout {
    grid-template-columns: 1fr;
  }

  .snippet-list-body {
    max-height: 200px;
  }
}
</style>
