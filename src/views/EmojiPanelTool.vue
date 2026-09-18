<template>
  <div class="tool-container fill-height">
    <!-- 搜索 + 最近使用 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">表情符号面板</span>
      </div>
      <div class="card-body">
        <div class="search-row">
          <el-input
            v-model="searchText"
            placeholder="搜索表情符号..."
            size="small"
            clearable
            class="search-input"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        <div v-if="recentEmojis.length && !searchText" class="recent-row">
          <span class="recent-label">最近使用</span>
          <div class="emoji-grid recent-grid">
            <span
              v-for="emoji in recentEmojis"
              :key="emoji"
              class="emoji-item"
              :data-name="getEmojiName(emoji)"
              @click="copyEmoji(emoji)"
            >{{ emoji }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 分类标签 -->
    <div class="tool-card category-card">
      <div class="category-tabs">
        <div
          v-for="cat in categories"
          :key="cat.id"
          class="category-tab"
          :class="{ active: activeCategory === cat.id }"
          @click="activeCategory = cat.id"
        >
          <span class="cat-icon">{{ cat.icon }}</span>
          <span class="cat-name">{{ cat.name }}</span>
        </div>
      </div>
    </div>

    <!-- Emoji 网格 -->
    <div class="tool-card emoji-list-card">
      <div class="card-body">
        <div v-if="filteredEmojis.length === 0" class="empty-tip">
          未找到匹配的表情符号
        </div>
        <div v-else class="emoji-grid">
          <span
            v-for="emoji in filteredEmojis"
            :key="emoji"
            class="emoji-item"
            :data-name="getEmojiName(emoji)"
            @click="copyEmoji(emoji)"
          >{{ emoji }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { EMOJI_CATEGORIES, ALL_EMOJIS } from '@/utils/emojiData'
import { getEmojiName } from '@/utils/emojiNames'

const categories = EMOJI_CATEGORIES
const activeCategory = ref(categories[0].id)
const searchText = ref('')
const recentEmojis = ref<string[]>([])

// 搜索结果：跨所有分类，匹配 emoji 字符或名称
const filteredEmojis = computed(() => {
  if (searchText.value.trim()) {
    const kw = searchText.value.trim().toLowerCase()
    return ALL_EMOJIS.filter(e =>
      e.includes(kw) || getEmojiName(e).toLowerCase().includes(kw)
    )
  }
  const cat = categories.find(c => c.id === activeCategory.value)
  return cat ? cat.emojis : []
})

const copyEmoji = async (emoji: string) => {
  try {
    await navigator.clipboard.writeText(emoji)
    // 加入最近使用
    recentEmojis.value = [emoji, ...recentEmojis.value.filter(e => e !== emoji)].slice(0, 24)
    ElMessage.success(`已复制 ${emoji}`)
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-container {
  padding: 20px;
  overflow-y: auto;
}

.search-row {
  display: flex;
  align-items: center;
}

.search-input {
  max-width: 360px;
}

.recent-row {
  margin-top: 16px;
}

.recent-label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  font-weight: 500;
}

.category-card {
  position: sticky;
  top: 0;
  z-index: 5;
}

.category-tabs {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  padding: 8px;
}

.category-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}

.category-tab:hover {
  background: var(--bg-input);
}

.category-tab.active {
  background: rgba(0, 212, 255, 0.15);
}

.cat-icon {
  font-size: 20px;
}

.cat-name {
  font-size: 11px;
  color: var(--text-secondary);
}

.category-tab.active .cat-name {
  color: var(--accent-cyan);
  font-weight: 600;
}

.emoji-list-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.emoji-list-card .card-body {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 32px;
}

.emoji-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.emoji-item {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  height: 44px;
  padding: 0 8px;
  font-size: 22px;
  line-height: 1;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s, transform 0.1s;
  user-select: none;
  white-space: nowrap;
}

.emoji-item:hover {
  background: var(--bg-input);
  transform: scale(1.08);
  z-index: 10;
}

.emoji-item:active {
  transform: scale(0.95);
}

/* CSS hover 名称提示 — 零组件开销 */
.emoji-item:hover::after {
  content: attr(data-name);
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 8px;
  background: #303133;
  color: #fff;
  font-size: 12px;
  line-height: 1.4;
  border-radius: 4px;
  white-space: nowrap;
  pointer-events: none;
  z-index: 100;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

/* 边缘 emoji 的提示不溢出 */
.emoji-item:first-child:hover::after {
  left: 0;
  transform: none;
}
.emoji-item:last-child:hover::after {
  left: auto;
  right: 0;
  transform: none;
}

.recent-grid {
  gap: 4px;
}

.recent-grid .emoji-item {
  min-width: 40px;
  height: 40px;
  font-size: 20px;
}

.empty-tip {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
