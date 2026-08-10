<template>
  <div class="changelog-container">
    <div class="tool-card changelog-card">
      <div class="card-header">
        <span class="card-title">📋 更新日志</span>
        <span class="version-badge">v{{ appVersion }}</span>
      </div>

      <div class="card-body">
        <!-- 每个版本一个 section，从新到旧 -->
        <div
          v-for="entry in parsedEntries"
          :key="entry.version"
          class="version-section"
        >
          <!-- 版本小标题 -->
          <div class="version-head">
            <span class="version-tag">v{{ entry.version }}</span>
            <span v-if="entry.date" class="version-date">{{ entry.date }}</span>
          </div>

          <!-- 功能要点 -->
          <div class="feature-block">
            <p class="feature-title">{{ entry.title }}</p>
            <p v-if="entry.detail" class="feature-detail">{{ entry.detail }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { changelogData, type ChangelogEntry } from '@/utils/changelogData'

const appVersion = __APP_VERSION__

interface ParsedEntry extends ChangelogEntry {
  title: string
  detail: string
}

/** 把 "功能名：详细描述" 拆成 title + detail
 *  只认前 30 字符内出现的首个冒号，避免误拆括号内的冒号
 */
function parseContent(content: string): { title: string; detail: string } {
  const idx = content.search(/[：:]/)
  if (idx > 0 && idx <= 30) {
    return {
      title: content.substring(0, idx).trim(),
      detail: content.substring(idx + 1).trim()
    }
  }
  return { title: content.trim(), detail: '' }
}

const parsedEntries = computed<ParsedEntry[]>(() =>
  changelogData.entries.map(e => ({ ...e, ...parseContent(e.content) }))
)
</script>

<style scoped>
.changelog-container {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

.changelog-card {
  max-width: 900px;
  margin: 0 auto;
}

.version-badge {
  padding: 2px 10px;
  background: var(--accent-cyan);
  color: #000;
  font-weight: 700;
  font-size: 12px;
  border-radius: 12px;
  font-family: 'JetBrains Mono', monospace;
  box-shadow: 0 0 12px rgba(0, 212, 255, 0.4);
}

/* ============ 版本 section ============ */
.version-section {
  padding: 18px 0;
  border-bottom: 1px solid var(--border-color);
}

.version-section:last-child {
  border-bottom: none;
}

.version-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.version-tag {
  padding: 3px 12px;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.15), rgba(0, 212, 255, 0.05));
  color: var(--accent-cyan);
  font-weight: 700;
  font-size: 14px;
  font-family: 'JetBrains Mono', monospace;
  border-radius: 6px;
  border: 1px solid rgba(0, 212, 255, 0.3);
}

.version-date {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}

/* ============ 功能要点 ============ */
.feature-block {
  padding-left: 4px;
}

.feature-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.6;
}

.feature-detail {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.7;
}
</style>
