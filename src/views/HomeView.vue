<template>
  <div class="home-container">
    <!-- 搜索区域 -->
    <div class="tool-card search-card">
      <div class="card-header">
        <span class="card-title">搜索工具</span>
      </div>
      <div class="card-body">
        <div class="search-wrapper">
          <div class="search-input-box">
            <span class="search-icon-svg">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65"/>
              </svg>
            </span>
            <input
              v-model="searchQuery"
              type="text"
              class="search-input"
              placeholder="输入关键词搜索工具..."
              @input="handleSearch"
            />
            <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </div>
        <!-- 搜索结果 -->
        <div v-if="searchResults.length > 0" class="search-results">
          <div
            v-for="tool in searchResults"
            :key="tool.id"
            class="search-result-item"
            @click="handleSelectTool(tool.id)"
          >
            <span class="result-icon" v-html="tool.iconSvg"></span>
            <div class="result-info">
              <span class="result-name">{{ tool.name }}</span>
              <span class="result-desc">{{ tool.description }}</span>
            </div>
          </div>
        </div>
        <div v-else-if="searchQuery && searchQuery.length > 0" class="search-empty">
          <span>未找到匹配的工具</span>
        </div>
      </div>
    </div>

    <!-- 最近使用 -->
    <div v-if="recentToolItems.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">最近使用</span>
      </div>
      <div class="card-body">
        <div class="tool-grid">
          <div
            v-for="tool in recentToolItems"
            :key="tool.id"
            class="tool-card-item"
            @click="handleSelectTool(tool.id)"
          >
            <span class="tool-icon" v-html="tool.iconSvg"></span>
            <span class="tool-name">{{ tool.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 常用工具推荐 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">常用工具</span>
      </div>
      <div class="card-body">
        <div class="tool-grid">
          <div
            v-for="tool in favoriteToolItems"
            :key="tool.id"
            class="tool-card-item"
            @click="handleSelectTool(tool.id)"
          >
            <span class="tool-icon" v-html="tool.iconSvg"></span>
            <span class="tool-name">{{ tool.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 全部工具 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">全部工具</span>
      </div>
      <div class="card-body">
        <div class="tool-grid">
          <div
            v-for="tool in allToolItems"
            :key="tool.id"
            class="tool-card-item"
            @click="handleSelectTool(tool.id)"
          >
            <span class="tool-icon" v-html="tool.iconSvg"></span>
            <span class="tool-name">{{ tool.name }}</span>
            <span class="tool-desc">{{ tool.description }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useToolboxStore, TOOL_LIST } from '@/store'

const props = defineProps<{
  onSelectTool: (toolId: string) => void
}>()

const store = useToolboxStore()
const searchQuery = ref('')

const searchResults = computed(() => {
  if (!searchQuery.value.trim()) return []
  const query = searchQuery.value.toLowerCase().trim()
  return TOOL_LIST.filter(tool => {
    if (tool.id === 'home') return false
    return (
      tool.name.toLowerCase().includes(query) ||
      tool.description.toLowerCase().includes(query) ||
      tool.keywords.some(k => k.toLowerCase().includes(query))
    )
  })
})

const recentToolItems = computed(() => {
  return store.recentTools
    .map(id => TOOL_LIST.find(t => t.id === id))
    .filter(Boolean) as typeof TOOL_LIST
})

const favoriteToolItems = computed(() => {
  return store.config.favorites
    .map(id => TOOL_LIST.find(t => t.id === id))
    .filter(Boolean) as typeof TOOL_LIST
})

const allToolItems = computed(() => {
  return TOOL_LIST.filter(t => t.id !== 'home')
})

const handleSearch = () => {
  // 搜索逻辑由 computed 处理
}

const handleSelectTool = (toolId: string) => {
  props.onSelectTool(toolId)
}
</script>

<style scoped>
.home-container {
  padding: 16px;
  overflow-y: auto;
  height: 100%;
}

/* 卡片间距 */
.home-container .tool-card {
  margin-bottom: 14px;
}

.search-card {
  margin-bottom: 14px;
}

/* 卡片标题间距 */
.card-header {
  padding: 10px 14px;
}

.card-body {
  padding: 12px 14px;
}

.search-wrapper {
  max-width: 480px;
}

.search-input-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0 12px;
  height: 38px;
  transition: all 0.2s;
}

.search-input-box:focus-within {
  border-color: var(--accent-cyan);
  box-shadow: var(--glow-cyan);
}

.search-icon-svg {
  color: var(--text-muted);
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  -webkit-appearance: none;
}

.search-input::placeholder {
  color: var(--text-muted);
}

/* 强制深色主题下的输入框样式 */
html.dark .search-input {
  background: transparent;
  color: var(--text-primary);
}

html.light .search-input {
  background: transparent;
  color: var(--text-primary);
}

.search-clear {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 3px;
  display: flex;
  align-items: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.search-clear:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.search-results {
  margin-top: 12px;
  max-width: 480px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.search-result-item:hover {
  background: rgba(0, 212, 255, 0.06);
  border-color: rgba(0, 212, 255, 0.2);
}

.result-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.result-icon :deep(svg) {
  width: 18px;
  height: 18px;
}

.result-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.result-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.result-desc {
  font-size: 11px;
  color: var(--text-secondary);
}

.search-empty {
  margin-top: 12px;
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.tool-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 10px;
}

.tool-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 12px;
  border-radius: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s;
}

.tool-card-item:hover {
  border-color: rgba(0, 212, 255, 0.3);
  background: rgba(0, 212, 255, 0.04);
  transform: translateY(-2px);
}

.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.tool-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.tool-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: center;
}

.tool-desc {
  font-size: 10px;
  color: var(--text-muted);
  text-align: center;
  line-height: 1.3;
}
</style>
