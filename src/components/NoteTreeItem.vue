<template>
  <div class="tree-item">
    <div
      class="tree-item-content"
      :class="{ active: item.id === selectedId, 'is-folder': item.type === 'folder' }"
      @click="handleClick"
      @contextmenu.prevent="$emit('contextmenu', $event, item)"
    >
      <el-icon v-if="item.type === 'folder'" class="tree-icon" @click.stop="toggleExpand">
        <Folder v-if="!expanded" />
        <FolderOpened v-else />
      </el-icon>
      <el-icon v-else class="tree-icon">
        <Document />
      </el-icon>
      <span class="tree-label" :title="item.name">{{ item.name }}</span>
      <span v-if="item.type === 'file' && isModified" class="unsaved-dot">●</span>
    </div>

    <!-- 子项 -->
    <div v-if="item.type === 'folder' && expanded" class="tree-children">
      <NoteTreeItem
        v-for="child in children"
        :key="child.id"
        :item="child"
        :selected-id="selectedId"
        @select="$emit('select', $event)"
        @contextmenu="handleChildContextmenu"
        @rename="$emit('rename', $event)"
        @delete="$emit('delete', $event)"
        @create-child="handleChildCreate"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Folder, FolderOpened, Document } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'

const props = defineProps<{
  item: NoteItem
  selectedId: number | null
  isModified?: boolean
}>()

const emit = defineEmits<{
  'select': [item: NoteItem]
  'contextmenu': [event: MouseEvent, item: NoteItem]
  'rename': [item: NoteItem]
  'delete': [item: NoteItem]
  'create-child': [parentId: number, type: 'folder' | 'file']
}>()

const expanded = ref(false)
const children = ref<NoteItem[]>([])

const toggleExpand = async () => {
  expanded.value = !expanded.value
  if (expanded.value && children.value.length === 0) {
    children.value = await noteClient.noteList(props.item.id)
  }
}

const handleClick = () => {
  if (props.item.type === 'folder') {
    toggleExpand()
  } else {
    emit('select', props.item)
  }
}

const handleChildContextmenu = (event: MouseEvent, childItem: NoteItem) => {
  emit('contextmenu', event, childItem)
}

const handleChildCreate = (parentId: number, type: 'folder' | 'file') => {
  emit('create-child', parentId, type)
}

onMounted(() => {
  // 默认不展开
})
</script>

<style scoped>
.tree-item-content {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  cursor: pointer;
  border-radius: 4px;
  margin: 1px 8px;
  transition: background 0.2s;
}

.tree-item-content:hover {
  background: rgba(0, 212, 255, 0.06);
}

.tree-item-content.active {
  background: rgba(0, 212, 255, 0.1);
}

.tree-icon {
  font-size: 16px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.tree-item-content.active .tree-icon {
  color: var(--accent-cyan);
}

.tree-label {
  flex: 1;
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-item-content.active .tree-label {
  color: var(--accent-cyan);
}

.unsaved-dot {
  color: var(--accent-cyan);
  font-size: 10px;
}

.tree-children {
  padding-left: 12px;
}
</style>
