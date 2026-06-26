<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="logo-area">
        <span class="logo-icon">⚡</span>
        <div class="logo-text">
          <h1 class="app-title">栗的百宝箱</h1>
          <span class="app-version">v2.12</span>
        </div>
      </div>
    </div>

    <nav class="sidebar-nav">
      <!-- 首页和历史 -->
      <div class="nav-section">
        <div
          v-for="tool in fixedTools"
          :key="tool.id"
          class="nav-item"
          :class="{ active: modelValue === tool.id }"
          @click="handleSelect(tool.id)"
        >
          <span class="nav-icon" v-html="tool.iconSvg"></span>
          <span class="nav-label">{{ tool.name }}</span>
          <span
            v-if="tool.id !== 'home'"
            class="fav-btn"
            :class="{ active: isFavorite(tool.id) }"
            @click.stop="handleToggleFavorite(tool.id)"
            title="收藏/取消收藏"
          >★</span>
        </div>
      </div>

      <!-- 收藏工具 -->
      <div v-if="favoritedTools.length > 0" class="nav-section">
        <div class="nav-section-title">收藏</div>
        <div
          v-for="tool in favoritedTools"
          :key="tool.id"
          class="nav-item"
          :class="{ active: modelValue === tool.id }"
          @click="handleSelect(tool.id)"
        >
          <span class="nav-icon" v-html="tool.iconSvg"></span>
          <span class="nav-label">{{ tool.name }}</span>
          <span
            class="fav-btn active"
            @click.stop="handleToggleFavorite(tool.id)"
            title="取消收藏"
          >★</span>
        </div>
      </div>

      <!-- 分类工具 -->
      <div v-for="category in categorizedTools" :key="category.name" class="nav-section">
        <div class="nav-section-title">{{ category.name }}</div>
        <div
          v-for="tool in category.tools"
          :key="tool.id"
          class="nav-item"
          :class="{ active: modelValue === tool.id }"
          @click="handleSelect(tool.id)"
        >
          <span class="nav-icon" v-html="tool.iconSvg"></span>
          <span class="nav-label">{{ tool.name }}</span>
          <span
            class="fav-btn"
            :class="{ active: isFavorite(tool.id) }"
            @click.stop="handleToggleFavorite(tool.id)"
            title="收藏/取消收藏"
          >★</span>
        </div>
      </div>

      <!-- 未分类工具 -->
      <div v-if="uncategorizedTools.length > 0" class="nav-section">
        <div class="nav-section-title">工具</div>
        <div
          v-for="tool in uncategorizedTools"
          :key="tool.id"
          class="nav-item"
          :class="{ active: modelValue === tool.id }"
          @click="handleSelect(tool.id)"
        >
          <span class="nav-icon" v-html="tool.iconSvg"></span>
          <span class="nav-label">{{ tool.name }}</span>
          <span
            class="fav-btn"
            :class="{ active: isFavorite(tool.id) }"
            @click.stop="handleToggleFavorite(tool.id)"
            title="收藏/取消收藏"
          >★</span>
        </div>
      </div>
    </nav>

    <div class="sidebar-footer">
      <el-tooltip :content="isPinned ? '取消置顶' : '窗口置顶'" placement="top">
        <button 
          class="pin-btn" 
          :class="{ active: isPinned }"
          @click="togglePin"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 17V12"/>
            <path d="M19 12V7a5 5 0 00-10 0v5"/>
            <path d="M8 12l4 5 4-5"/>
          </svg>
        </button>
      </el-tooltip>
      
      <el-select v-model="currentTheme" size="small" class="theme-select">
        <el-option label="跟随系统" value="auto" />
        <el-option label="深色模式" value="dark" />
        <el-option label="浅色模式" value="light" />
      </el-select>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useToolboxStore, TOOL_LIST } from '@/store'

defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const store = useToolboxStore()
const currentTheme = ref(store.config.theme)
const isPinned = ref(false)

const togglePin = async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const currentWindow = getCurrentWindow()
    isPinned.value = !isPinned.value
    await currentWindow.setAlwaysOnTop(isPinned.value)
  } catch (error) {
    console.error('设置窗口置顶失败:', error)
  }
}

onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const currentWindow = getCurrentWindow()
    isPinned.value = await currentWindow.isAlwaysOnTop()
  } catch (error) {
    console.error('获取窗口置顶状态失败:', error)
  }
})

// 固定工具（首页、剪贴板、历史、工作流）
const fixedTools = computed(() => {
  return TOOL_LIST.filter(t => t.id === 'home' || t.id === 'clipboard' || t.id === 'history' || t.id === 'workflow')
})

// 收藏工具（排除固定工具）
const favoritedTools = computed(() => {
  const favorites = store.config.favorites
  return TOOL_LIST.filter(t =>
    t.id !== 'home' && t.id !== 'clipboard' && t.id !== 'history' && t.id !== 'workflow' && favorites.includes(t.id)
  )
})

// 分类工具（有 category 字段的，排除固定和收藏）
const categorizedTools = computed(() => {
  const favorites = store.config.favorites
  const categorized = TOOL_LIST.filter(t =>
    t.category && t.id !== 'home' && t.id !== 'clipboard' && t.id !== 'history' && t.id !== 'workflow' && !favorites.includes(t.id)
  )

  const categoryMap = new Map<string, typeof TOOL_LIST>()
  const categoryNames: Record<string, string> = {
    text: '文本工具',
    dev: '开发工具',
    security: '安全工具',
    utility: '实用工具'
  }

  for (const tool of categorized) {
    const cat = tool.category!
    if (!categoryMap.has(cat)) {
      categoryMap.set(cat, [])
    }
    categoryMap.get(cat)!.push(tool)
  }

  return Array.from(categoryMap.entries()).map(([key, tools]) => ({
    name: categoryNames[key] || key,
    tools
  }))
})

// 未分类工具（没有 category 字段，排除固定和收藏）
const uncategorizedTools = computed(() => {
  const favorites = store.config.favorites
  return TOOL_LIST.filter(t =>
    !t.category && t.id !== 'home' && t.id !== 'clipboard' && t.id !== 'history' && !favorites.includes(t.id)
  )
})

const isFavorite = (toolId: string) => {
  return store.config.favorites.includes(toolId)
}

const handleSelect = (toolId: string) => {
  emit('update:modelValue', toolId)
  store.addRecentTool(toolId)
}

const handleToggleFavorite = (toolId: string) => {
  store.toggleFavorite(toolId)
}

watch(currentTheme, (newTheme) => {
  store.saveConfig({ theme: newTheme as 'auto' | 'dark' | 'light' })
  applyTheme(newTheme)
})

const applyTheme = (theme: string) => {
  const html = document.documentElement
  html.classList.remove('dark', 'light')

  if (theme === 'dark') {
    html.classList.add('dark')
  } else if (theme === 'light') {
    html.classList.add('light')
  } else {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) {
      html.classList.add('dark')
    } else {
      html.classList.add('light')
    }
  }
}
</script>

<style scoped>
.sidebar {
  width: 220px;
  min-width: 220px;
  height: 100vh;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.logo-area {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo-icon {
  font-size: 22px;
  filter: drop-shadow(0 0 6px rgba(0, 212, 255, 0.5));
}

.logo-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.app-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 1px;
  margin: 0;
  line-height: 1.2;
}

.app-version {
  font-size: 10px;
  color: var(--accent-cyan);
  background: rgba(8, 145, 178, 0.08);
  padding: 1px 6px;
  border-radius: 8px;
  border: 1px solid rgba(8, 145, 178, 0.2);
  width: fit-content;
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.nav-section {
  padding: 0 8px;
}

.nav-section-title {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 8px 12px 6px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 2px;
  position: relative;
}

.nav-item:hover {
  background: rgba(0, 212, 255, 0.06);
}

.nav-item.active {
  background: rgba(0, 212, 255, 0.1);
  color: var(--accent-cyan);
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.nav-icon :deep(svg) {
  width: 18px;
  height: 18px;
}

.nav-item.active .nav-icon {
  color: var(--accent-cyan);
}

.nav-label {
  font-size: 13px;
  color: var(--text-secondary);
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-item.active .nav-label {
  color: var(--accent-cyan);
  font-weight: 600;
}

.fav-btn {
  font-size: 12px;
  color: var(--text-muted);
  opacity: 0;
  transition: all 0.2s;
  cursor: pointer;
  padding: 2px 4px;
}

.nav-item:hover .fav-btn {
  opacity: 0.5;
}

.fav-btn:hover {
  opacity: 1 !important;
}

.fav-btn.active {
  opacity: 1;
  color: var(--accent-cyan);
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 8px;
}

.pin-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.pin-btn:hover {
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.pin-btn.active {
  background: rgba(0, 212, 255, 0.15);
  border-color: var(--accent-cyan);
  color: var(--accent-cyan);
}

.theme-select {
  flex: 1;
}

.sidebar-nav::-webkit-scrollbar {
  width: 4px;
}

.sidebar-nav::-webkit-scrollbar-track {
  background: transparent;
}

.sidebar-nav::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
</style>
