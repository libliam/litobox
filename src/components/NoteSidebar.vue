<template>
  <div class="note-sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">笔记</span>
      <el-button size="small" text @click="showNewMenu = true">
        <el-icon><Plus /></el-icon>
      </el-button>
    </div>

    <!-- 新建菜单 -->
    <el-dropdown v-model:visible="showNewMenu" trigger="click" placement="bottom-start">
      <div class="new-menu-trigger" style="display:none"></div>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item @click="handleCreateFolder">
            <el-icon><FolderAdd /></el-icon> 新建文件夹
          </el-dropdown-item>
          <el-dropdown-item @click="handleCreateFile">
            <el-icon><DocumentAdd /></el-icon> 新建文件
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>

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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, FolderAdd, DocumentAdd } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteTreeItem from './NoteTreeItem.vue'

const emit = defineEmits<{
  'select': [item: NoteItem]
}>()

const rootItems = ref<NoteItem[]>([])
const selectedId = ref<number | null>(null)
const showNewMenu = ref(false)

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

const handleCreateFolder = async () => {
  showNewMenu.value = false
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
  showNewMenu.value = false
  try {
    const { value } = await ElMessageBox.prompt('文件名称', '新建文件', {
      inputPattern: /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/,
      inputErrorMessage: '请输入有效的文件名（含扩展名）',
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
    const pattern = type === 'folder'
      ? /^[^\\/:*?"<>|]+$/
      : /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/
    const { value } = await ElMessageBox.prompt(label, `新建${type === 'folder' ? '文件夹' : '文件'}`, {
      inputPattern: pattern,
      inputErrorMessage: type === 'folder' ? '名称不能包含 \\ / : * ? " < > |' : '请输入有效的文件名（含扩展名）',
    })
    await noteClient.noteCreate(value, type, parentId)
    await loadItems()
    ElMessage.success('已创建')
  } catch {
    // 用户取消
  }
}

const handleRename = async (item: NoteItem) => {
  try {
    const { value } = await ElMessageBox.prompt('新名称', '重命名', {
      inputValue: item.name,
      inputPattern: item.type === 'folder' ? /^[^\\/:*?"<>|]+$/ : /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/,
      inputErrorMessage: item.type === 'folder' ? '名称不能包含 \\ / : * ? " < > |' : '请输入有效的文件名（含扩展名）',
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

const handleContextMenu = (_event: MouseEvent, item: NoteItem | null) => {
  if (!item) return
  // 右键菜单：使用 Element Plus 的 ElMessageBox 提供快速操作
  ElMessageBox({
    title: item.name,
    message: `选择操作`,
    showCancelButton: true,
    confirmButtonText: '重命名',
    cancelButtonText: '删除',
    distinguishCancelAndClose: true,
  }).then(() => {
    handleRename(item)
  }).catch((action) => {
    if (action === 'cancel') {
      handleDelete(item)
    }
  })
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
</style>
