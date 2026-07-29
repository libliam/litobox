<template>
  <Teleport to="body">
    <div v-if="visible" class="ql-overlay" @click.self="close">
      <div class="ql-container" @click.stop>
        <input
          ref="inputRef"
          v-model="query"
          class="ql-input"
          type="text"
          placeholder="搜索文件名…"
          autocomplete="off"
          spellcheck="false"
          @input="onQueryInput"
          @keydown="handleKeydown"
        />
        <div v-if="results.length > 0" class="ql-results">
          <div
            v-for="(item, idx) in results"
            :key="item.id"
            class="ql-result-item"
            :class="{ active: selectedIndex === idx }"
            @click="handleOpen(item.path)"
            @mouseenter="selectedIndex = idx"
          >
            <span class="ql-result-icon">{{ getFileIcon(item.extension) }}</span>
            <div class="ql-result-info">
              <div class="ql-result-name">{{ item.name }}</div>
              <div class="ql-result-path">{{ item.path }}</div>
            </div>
          </div>
        </div>
        <div v-else-if="searched" class="ql-empty">未找到匹配的文件</div>
        <div class="ql-hint">
          <span>↑↓ 选择</span>
          <span>Enter 打开</span>
          <span>Esc 关闭</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { qlSearch, qlOpenFile, type QuickLaunchResult } from '@/utils/quickLaunchClient'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean] }>()

const query = ref('')
const results = ref<QuickLaunchResult[]>([])
const selectedIndex = ref(0)
const searched = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.visible, (v) => {
  if (v) {
    query.value = ''
    results.value = []
    selectedIndex.value = 0
    searched.value = false
    nextTick(() => inputRef.value?.focus())
  }
})

function onQueryInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
}

function close() {
  emit('update:visible', false)
}

async function doSearch() {
  if (!query.value.trim()) {
    results.value = []
    searched.value = false
    return
  }
  try {
    results.value = await qlSearch(query.value.trim())
    searched.value = true
    selectedIndex.value = 0
  } catch {
    results.value = []
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = results.value[selectedIndex.value]
    if (item) handleOpen(item.path)
  } else if (e.key === 'Escape') {
    e.preventDefault()
    close()
  }
}

function handleOpen(path: string) {
  qlOpenFile(path).catch(() => {})
  close()
}

function getFileIcon(ext: string): string {
  const icons: Record<string, string> = {
    txt: '📄', pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️',
    mp3: '🎵', mp4: '🎬', zip: '📦', exe: '⚙️',
  }
  return icons[ext.toLowerCase()] || '📄'
}
</script>

<style scoped>
.ql-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.ql-container {
  width: 600px;
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

.ql-input {
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

.ql-input:focus {
  border-bottom-color: var(--accent-cyan);
}

.ql-results {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.ql-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 18px;
  cursor: pointer;
  transition: background 0.12s;
}

.ql-result-item.active,
.ql-result-item:hover {
  background: var(--bg-secondary);
}

.ql-result-icon {
  flex-shrink: 0;
  font-size: 18px;
  width: 24px;
  text-align: center;
}

.ql-result-info {
  flex: 1;
  min-width: 0;
}

.ql-result-name {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.3;
}

.ql-result-path {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.ql-empty {
  padding: 32px 18px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.ql-hint {
  display: flex;
  gap: 18px;
  padding: 8px 18px;
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
}
</style>
