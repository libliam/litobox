<template>
  <div class="tab-bar" @wheel.prevent="handleWheel">
    <div class="tab-list" ref="tabListRef">
      <div
        v-for="tab in tabs"
        :key="tab.toolId"
        class="tab-item"
        :class="{
          active: tab.toolId === activeTabId,
          'closable': tab.toolId !== 'home'
        }"
        @click="handleClick(tab.toolId)"
        @contextmenu.prevent="handleContextMenu($event, tab.toolId)"
        @mousedown.middle.prevent="handleMiddleClick(tab.toolId)"
        :title="getToolName(tab.toolId)"
      >
        <span class="tab-icon" v-html="getToolIcon(tab.toolId)"></span>
        <span class="tab-label">{{ getToolName(tab.toolId) }}</span>
        <span
          v-if="tab.toolId !== 'home'"
          class="tab-close"
          @click.stop="handleClose(tab.toolId)"
        >
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </span>
      </div>
    </div>

    <!-- 右键菜单 -->
    <ul v-if="ctxMenu.visible" class="tab-ctx-menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }">
      <li v-if="ctxMenu.toolId !== 'home'" @click="handleCtxClose">关闭</li>
      <li v-if="ctxMenu.toolId !== 'home'" @click="handleCtxCloseOthers">关闭其他</li>
      <li @click="handleCtxCloseAll">关闭全部</li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useToolboxStore, TOOL_LIST } from '@/store'
import { storeToRefs } from 'pinia'

defineOptions({ name: 'TabBar' })

const store = useToolboxStore()
const { tabs, activeTabId } = storeToRefs(store)

const ctxMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  toolId: '',
})

const tabListRef = ref<HTMLElement | null>(null)

const getToolName = (toolId: string) => {
  return TOOL_LIST.find(t => t.id === toolId)?.name || toolId
}

const getToolIcon = (toolId: string) => {
  return TOOL_LIST.find(t => t.id === toolId)?.iconSvg || ''
}

const handleClick = (toolId: string) => {
  store.switchTab(toolId)
}

const handleClose = (toolId: string) => {
  store.closeTab(toolId)
}

const handleMiddleClick = (toolId: string) => {
  // 中键关闭（home 除外）
  if (toolId !== 'home') store.closeTab(toolId)
}

const handleContextMenu = (e: MouseEvent, toolId: string) => {
  ctxMenu.visible = true
  ctxMenu.x = e.clientX
  ctxMenu.y = e.clientY
  ctxMenu.toolId = toolId
}

const closeCtxMenu = () => {
  ctxMenu.visible = false
}

const handleCtxClose = () => {
  store.closeTab(ctxMenu.toolId)
  closeCtxMenu()
}

const handleCtxCloseOthers = () => {
  store.closeOthers(ctxMenu.toolId)
  closeCtxMenu()
}

const handleCtxCloseAll = () => {
  store.closeAllTabs()
  closeCtxMenu()
}

const handleWheel = (e: WheelEvent) => {
  // 横向滚轮支持（鼠标滚轮在 tab 栏上滚动时横向滚动）
  if (tabListRef.value) {
    tabListRef.value.scrollLeft += e.deltaY
  }
}

const onDocClick = () => {
  if (ctxMenu.visible) closeCtxMenu()
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
})
</script>

<style scoped>
.tab-bar {
  height: 36px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-shrink: 0;
  position: relative;
}

.tab-list {
  display: flex;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: thin;
}

.tab-list::-webkit-scrollbar {
  height: 2px;
}

.tab-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  height: 36px;
  min-width: 80px;
  max-width: 180px;
  cursor: pointer;
  border-right: 1px solid var(--border-color);
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s;
  position: relative;
  user-select: none;
}

.tab-item:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.tab-item.active {
  background: var(--bg-primary);
  color: var(--accent-cyan);
  border-bottom: 2px solid var(--accent-cyan);
}

.tab-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  color: var(--text-muted);
}

.tab-item.active .tab-icon {
  color: var(--accent-cyan);
}

.tab-icon :deep(svg) {
  width: 14px;
  height: 14px;
}

.tab-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  color: var(--text-muted);
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.tab-item:hover .tab-close {
  opacity: 0.6;
}

.tab-close:hover {
  background: rgba(239, 68, 68, 0.2);
  color: var(--accent-red);
  opacity: 1 !important;
}

.tab-ctx-menu {
  position: fixed;
  z-index: 9999;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 120px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  list-style: none;
  margin: 0;
}

.tab-ctx-menu li {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary);
  transition: background 0.15s;
}

.tab-ctx-menu li:hover {
  background: var(--hover-bg);
  color: var(--accent-cyan);
}
</style>
