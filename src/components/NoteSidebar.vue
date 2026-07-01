<template>
  <div class="note-sidebar" @click="closeContextMenu">
    <div class="sidebar-header">
      <span class="sidebar-title">笔记</span>
      <el-dropdown trigger="click" placement="bottom-end" @command="handleCommand">
        <el-button size="small" text>
          <el-icon><Plus /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="create-file">
              <el-icon><DocumentAdd /></el-icon> 新建文件
            </el-dropdown-item>
            <el-dropdown-item command="create-folder">
              <el-icon><FolderAdd /></el-icon> 新建文件夹
            </el-dropdown-item>
            <el-dropdown-item command="open-folder" divided>
              <el-icon><FolderOpened /></el-icon> 打开存储目录
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>

    <!-- 文件树 -->
    <div class="file-tree">
      <NoteTreeItem
        v-for="item in rootItems"
        :key="item.id"
        :item="item"
        :selected-id="selectedId"
        @select="handleSelect"
        @contextmenu="handleContextMenu"
        @rename="handleRename"
        @delete="handleDelete"
        @create-child="handleCreateChild"
      />
    </div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    >
      <template v-if="contextMenu.item?.type === 'folder'">
        <div class="context-menu-item" @click="handleContextAction('create-file')">
          <el-icon><DocumentAdd /></el-icon> 新建文件
        </div>
        <div class="context-menu-item" @click="handleContextAction('create-folder')">
          <el-icon><FolderAdd /></el-icon> 新建文件夹
        </div>
        <div class="context-menu-divider"></div>
        <div class="context-menu-item" @click="handleContextAction('rename')">
          <el-icon><Edit /></el-icon> 重命名
        </div>
        <div class="context-menu-item danger" @click="handleContextAction('delete')">
          <el-icon><Delete /></el-icon> 删除
        </div>
      </template>
      <template v-else>
        <div class="context-menu-item" @click="handleContextAction('rename')">
          <el-icon><Edit /></el-icon> 重命名
        </div>
        <div class="context-menu-item danger" @click="handleContextAction('delete')">
          <el-icon><Delete /></el-icon> 删除
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, FolderAdd, DocumentAdd, FolderOpened, Edit, Delete } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteTreeItem from './NoteTreeItem.vue'

const emit = defineEmits<{
  'select': [item: NoteItem]
}>()

const rootItems = ref<NoteItem[]>([])
const selectedId = ref<number | null>(null)

const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  item: null as NoteItem | null,
})

const loadItems = async (parentId: number | null = null) => {
  try {
    const items = await noteClient.noteList(parentId)
    if (parentId === null) {
      rootItems.value = items
    }
    return items
  } catch (e: any) {
    ElMessage.error(`加载失败: ${e}`)
    return []
  }
}

const handleSelect = (item: NoteItem) => {
  if (item.type === 'file') {
    selectedId.value = item.id
    emit('select', item)
  }
}

const handleCommand = async (command: string) => {
  switch (command) {
    case 'create-file':
      await handleCreateFile()
      break
    case 'create-folder':
      await handleCreateFolder()
      break
    case 'open-folder':
      await handleOpenFolder()
      break
  }
}

const handleCreateFolder = async () => {
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      inputPattern: /^[^\\/:*?"<>|]+$/,
      inputErrorMessage: '名称不能包含 \\ / : * ? " < > |',
    })
    await noteClient.noteCreate(value, 'folder', null)
    await loadItems()
    ElMessage.success('文件夹已创建')
  } catch {
    // 用户取消
  }
}

const handleCreateFile = async () => {
  try {
    const { value } = await ElMessageBox.prompt('文件名称', '新建文件', {
      inputPattern: /^[^\\/:*?"<>|]+$/,
      inputErrorMessage: '名称不能包含 \\ / : * ? " < > |',
    })
    const item = await noteClient.noteCreate(value, 'file', null)
    await loadItems()
    emit('select', item)
    ElMessage.success('文件已创建')
  } catch {
    // 用户取消
  }
}

const handleCreateChild = async (parentId: number, type: 'folder' | 'file') => {
  try {
    const label = type === 'folder' ? '文件夹名称' : '文件名称'
    const pattern = /^[^\\/:*?"<>|]+$/
    const { value } = await ElMessageBox.prompt(label, `新建${type === 'folder' ? '文件夹' : '文件'}`, {
      inputPattern: pattern,
      inputErrorMessage: '名称不能包含 \\ / : * ? " < > |',
    })
    const item = await noteClient.noteCreate(value, type, parentId)
    // 刷新父文件夹的子项列表
    await loadItems(null)
    if (type === 'file') {
      emit('select', item)
    }
    ElMessage.success('已创建')
  } catch {
    // 用户取消
  }
}

const handleRename = async (item: NoteItem) => {
  try {
    const { value } = await ElMessageBox.prompt('新名称', '重命名', {
      inputValue: item.name,
      inputPattern: /^[^\\/:*?"<>|]+$/,
      inputErrorMessage: '名称不能包含 \\ / : * ? " < > |',
    })
    await noteClient.noteRename(item.id, value)
    await loadItems()
    ElMessage.success('已重命名')
  } catch {
    // 用户取消
  }
}

const handleDelete = async (item: NoteItem) => {
  try {
    if (item.type === 'folder') {
      const children = await noteClient.noteList(item.id)
      const count = children.length
      await ElMessageBox.confirm(
        `将删除文件夹及其中的 ${count} 个项目，确定要删除吗？`,
        '删除确认',
        { type: 'warning' }
      )
    } else {
      await ElMessageBox.confirm(`确定要删除 "${item.name}" 吗？`, '删除确认', { type: 'warning' })
    }
    await noteClient.noteDelete(item.id)
    if (selectedId.value === item.id) {
      selectedId.value = null
    }
    await loadItems()
    ElMessage.success('已删除')
  } catch {
    // 用户取消
  }
}

const handleContextMenu = (event: MouseEvent, item: NoteItem | null) => {
  if (!item) return
  event.preventDefault()
  contextMenu.visible = true
  contextMenu.x = event.clientX
  contextMenu.y = event.clientY
  contextMenu.item = item
}

const closeContextMenu = () => {
  contextMenu.visible = false
  contextMenu.item = null
}

const handleContextAction = async (action: string) => {
  const item = contextMenu.item
  closeContextMenu()
  if (!item) return

  switch (action) {
    case 'create-file':
      await handleCreateChild(item.id, 'file')
      break
    case 'create-folder':
      await handleCreateChild(item.id, 'folder')
      break
    case 'rename':
      await handleRename(item)
      break
    case 'delete':
      await handleDelete(item)
      break
  }
}

const handleOpenFolder = async () => {
  try {
    await invoke('open_notes_folder')
  } catch (e: any) {
    ElMessage.error(`打开目录失败: ${e}`)
  }
}

onMounted(() => {
  loadItems()
})
</script>

<style scoped>
.note-sidebar {
  width: 200px;
  min-width: 200px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 140px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.15s;
}

.context-menu-item:hover {
  background: rgba(0, 212, 255, 0.1);
  color: var(--accent-cyan);
}

.context-menu-item.danger {
  color: var(--color-danger);
}

.context-menu-item.danger:hover {
  background: rgba(255, 77, 79, 0.1);
}

.context-menu-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}
</style>
