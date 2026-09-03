<template>
  <div class="changelog-container">
    <div class="tool-card changelog-card">
      <div class="card-header">
        <span class="card-title">📋 更新日志</span>
        <span class="version-badge">v{{ appVersion }}</span>
      </div>

      <div class="card-body">
        <!-- 每个版本一个 section，从新到旧 -->
        <div v-for="entry in parsedEntries" :key="entry.version" class="version-section">
          <!-- 版本小标题 -->
          <div class="version-head">
            <span class="version-tag">v{{ entry.version }}</span>
            <span v-if="entry.date" class="version-date">{{ entry.date }}</span>
          </div>

          <!-- 功能内容 -->
          <div class="feature-block">
            <p v-if="entry.title" class="feature-title">{{ entry.title }}</p>
            <div v-if="entry.lines.length" class="feature-lines" :class="{ collapsed: isCollapsed(entry.version) }">
              <p v-for="(line, i) in visibleLines(entry)" :key="i" class="feature-line">{{ line }}</p>
            </div>
            <el-button v-if="entry.foldable" class="fold-btn" size="small" text @click="toggle(entry.version)">
              {{ isCollapsed(entry.version) ? `展开全部 ${entry.lines.length} 条` : '收起' }}
            </el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { changelogData, type ChangelogEntry } from '@/utils/changelogData'

const appVersion = __APP_VERSION__

interface ParsedEntry extends ChangelogEntry {
  title: string
  /** 详情按 ；/。 拆成可换行的条目 */
  lines: string[]
  /** 内容过长需要折叠 */
  foldable: boolean
}

const MAX_LINE_CHARS = 120
const COLLAPSE_LINES = 4
const COLLAPSE_TOTAL = 220

/**
 * 拆分内容：
 * 1. 取首个「功能名」标题：内容开头到首个 （/：【 前的片段，长度 2~32 视为标题。
 *    无冒号的修复/优化条目（如「修复 X（原因）」）也能正确取到短标题，避免整段加粗。
 * 2. 剩余正文按 ；/。 拆成独立条目，实现长文换行
 */
function parseContent(content: string): { title: string; rest: string } {
  const headMatch = content.match(/^([^（【：:]{2,32})(?=[（【：:])/)
  if (headMatch) return { title: headMatch[1].trim(), rest: content.slice(headMatch[1].length) }
  return { title: '', rest: content }
}

const parsedEntries = computed<ParsedEntry[]>(() =>
  changelogData.entries.map((e) => {
    const { title, rest } = parseContent(e.content)
    const lines = rest
      .split(/[；;。]+/)
      .map((t) => t.replace(/^[（(：:\s]+/, '').trim())
      .filter((t) => t.length > 0)
    const total = title.length + rest.length
    const foldable = lines.length > COLLAPSE_LINES || total > COLLAPSE_TOTAL
    return { ...e, title, lines, foldable }
  }),
)

// ============ 折叠状态 ============
const collapsedSet = ref(new Set(parsedEntries.value.filter((e) => e.foldable).map((e) => e.version)))

const isCollapsed = (version: string) => collapsedSet.value.has(version)

const toggle = (version: string) => {
  const set = new Set(collapsedSet.value)
  if (set.has(version)) set.delete(version)
  else set.add(version)
  collapsedSet.value = set
}

/** 折叠时：只显示前几条且单条超长截断；展开时全部原样 */
const visibleLines = (entry: ParsedEntry): string[] => {
  if (!isCollapsed(entry.version)) return entry.lines
  return entry.lines.slice(0, COLLAPSE_LINES).map((l) =>
    l.length > MAX_LINE_CHARS ? `${l.slice(0, MAX_LINE_CHARS)}…` : l,
  )
}
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
  padding: 16px 4px;
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

/* ============ 功能内容 ============ */
.feature-block {
  padding-left: 4px;
}

.feature-title {
  margin: 0 0 6px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.6;
}

.feature-lines {
  position: relative;
}

.feature-line {
  position: relative;
  margin: 4px 0;
  padding-left: 14px;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.8;
  word-break: break-word;
}

/* 行首小圆点 */
.feature-line::before {
  content: '';
  position: absolute;
  left: 2px;
  top: 10px;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent-cyan);
  opacity: 0.6;
}

.fold-btn {
  margin-top: 6px;
  color: var(--accent-cyan);
  font-size: 12px;
}
</style>
