<template>
  <Teleport to="body">
    <div v-if="isCommandPaletteOpen" class="palette-overlay" @click="closePalette">
      <div class="palette-container" @click.stop>
        <input
          ref="inputRef"
          v-model="query"
          class="palette-input"
          type="text"
          placeholder="搜索工具…"
          autocomplete="off"
          spellcheck="false"
          @keydown="handleKeydown"
        />
        <div v-if="flatList.length" class="palette-results">
          <div v-for="group in grouped" :key="group.category" class="palette-group">
            <div class="palette-group-title">{{ categoryLabel(group.category) }}</div>
            <div
              v-for="item in group.items"
              :key="item.tool.id"
              class="palette-item"
              :class="{ active: item.flatIndex === selectedIndex }"
              @click="openTool(item.tool.id)"
              @mouseenter="selectedIndex = item.flatIndex"
            >
              <span class="palette-item-icon" v-html="item.tool.iconSvg || item.tool.icon"></span>
              <div class="palette-item-text">
                <div class="palette-item-name">{{ item.tool.name }}</div>
                <div class="palette-item-desc">{{ item.tool.description }}</div>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="palette-empty">未找到匹配工具</div>
        <div class="palette-hint">
          <span>↑↓ 选择</span>
          <span>Enter 跳转</span>
          <span>Esc 关闭</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { storeToRefs } from 'pinia'
import { useToolboxStore, TOOL_LIST } from '@/store'
import { filterTools, type RankedTool } from '@/utils/commandPalette'

interface FlatItem extends RankedTool {
  flatIndex: number
}

const store = useToolboxStore()
const { isCommandPaletteOpen } = storeToRefs(store)

const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const flatList = computed<FlatItem[]>(() => {
  const results = filterTools(query.value, TOOL_LIST)
  return results.map((r, i) => ({ ...r, flatIndex: i }))
})

const grouped = computed(() => {
  const map = new Map<string, FlatItem[]>()
  for (const item of flatList.value) {
    const cat = item.tool.category || '其他'
    if (!map.has(cat)) map.set(cat, [])
    map.get(cat)!.push(item)
  }
  return Array.from(map.entries()).map(([category, items]) => ({ category, items }))
})

const CATEGORY_LABELS: Record<string, string> = {
  text: '文本工具',
  dev: '开发工具',
  security: '安全工具',
  utility: '实用工具',
  system: '系统工具',
  其他: '其他',
}

const categoryLabel = (cat: string) => CATEGORY_LABELS[cat] || cat

const openTool = (toolId: string) => {
  store.openTab(toolId)
  store.addRecentTool(toolId)
  closePalette()
}

const closePalette = () => {
  store.closeCommandPalette()
  query.value = ''
  selectedIndex.value = 0
}

const moveSelection = (delta: number) => {
  if (flatList.value.length === 0) return
  const len = flatList.value.length
  selectedIndex.value = (selectedIndex.value + delta + len) % len
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    moveSelection(1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    moveSelection(-1)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = flatList.value[selectedIndex.value]
    if (item) openTool(item.tool.id)
  } else if (e.key === 'Escape') {
    e.preventDefault()
    closePalette()
  }
}

// 面板打开时聚焦输入框、重置状态；关闭时清空
watch(isCommandPaletteOpen, (open) => {
  if (open) {
    query.value = ''
    selectedIndex.value = 0
    nextTick(() => {
      inputRef.value?.focus()
    })
  }
})

// 输入变化时重置选中项到首位
watch(query, () => {
  selectedIndex.value = 0
})
</script>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.palette-container {
  width: 560px;
  max-width: 90vw;
  max-height: 70vh;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.palette-input {
  width: 100%;
  padding: 14px 18px;
  background: var(--bg-input);
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 15px;
  outline: none;
  box-sizing: border-box;
}

.palette-input:focus {
  border-bottom-color: var(--accent-cyan);
}

.palette-results {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.palette-group-title {
  padding: 8px 18px 4px;
  font-size: 11px;
  color: var(--accent-cyan);
  letter-spacing: 1px;
  text-transform: uppercase;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 18px;
  cursor: pointer;
  transition: background 0.12s;
}

.palette-item.active,
.palette-item:hover {
  background: var(--bg-secondary);
}

.palette-item-icon {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}

.palette-item-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.palette-item-text {
  flex: 1;
  min-width: 0;
}

.palette-item-name {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.3;
}

.palette-item-desc {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.palette-empty {
  padding: 32px 18px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.palette-hint {
  display: flex;
  gap: 18px;
  padding: 8px 18px;
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
}
</style>
