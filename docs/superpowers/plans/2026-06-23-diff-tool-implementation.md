# 文本代码对比工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增文本/代码对比工具，支持左右双栏输入、行级/字符级对比、差异高亮展示

**Architecture:** 使用 `diff` npm 包作为底层算法，封装为 `diffUtils.ts` 纯函数，`DiffTool.vue` 页面组件负责 UI 渲染和交互。双栏同步滚动通过监听 scroll 事件实现。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus, `diff` npm 包

---

### Task 1: 安装 diff 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 diff 包**

```powershell
npm.cmd install diff
```

- [ ] **Step 2: 验证安装**

检查 `package.json` 中是否包含 `"diff": "^x.x.x"`

- [ ] **Step 3: Commit**

```powershell
git add package.json package-lock.json
git commit -m "feat: add diff dependency for text comparison tool"
```

---

### Task 2: 创建 diffUtils.ts 工具函数

**Files:**
- Create: `src/utils/diffUtils.ts`

- [ ] **Step 1: 编写 diffUtils.ts**

```typescript
import { diffLines, diffChars, Change } from 'diff'

export interface DiffLine {
  type: 'equal' | 'add' | 'remove' | 'modify'
  leftLine?: string
  rightLine?: string
  leftLineNum?: number
  rightLineNum?: number
  charDiffs?: CharDiff[]
}

export interface CharDiff {
  type: 'equal' | 'add' | 'remove'
  value: string
}

export interface DiffOptions {
  ignoreWhitespace: boolean
  ignoreCase: boolean
}

/**
 * 行级对比
 */
export function computeLineDiff(
  left: string,
  right: string,
  options: DiffOptions = { ignoreWhitespace: false, ignoreCase: false }
): DiffLine[] {
  let leftText = left
  let rightText = right

  if (options.ignoreCase) {
    leftText = leftText.toLowerCase()
    rightText = rightText.toLowerCase()
  }

  const changes = diffLines(leftText, rightText, {
    ignoreWhitespace: options.ignoreWhitespace,
  })

  const result: DiffLine[] = []
  let leftNum = 0
  let rightNum = 0

  for (const change of changes) {
    const lines = change.value.split('\n').filter(l => l.length > 0 || change.value.endsWith('\n'))

    if (change.added) {
      for (const line of lines) {
        rightNum++
        result.push({ type: 'add', rightLine: line, rightLineNum: rightNum })
      }
    } else if (change.removed) {
      for (const line of lines) {
        leftNum++
        result.push({ type: 'remove', leftLine: line, leftLineNum: leftNum })
      }
    } else {
      for (const line of lines) {
        leftNum++
        rightNum++
        result.push({ type: 'equal', leftLine: line, rightLine: line, leftLineNum: leftNum, rightLineNum: rightNum })
      }
    }
  }

  // 标记 modify 行（相邻的 remove + add）
  return markModifications(result)
}

/**
 * 标记修改行（相邻的 remove 和 add 配对为 modify）
 */
function markModifications(lines: DiffLine[]): DiffLine[] {
  const result: DiffLine[] = []
  let i = 0

  while (i < lines.length) {
    if (lines[i].type === 'remove' && i + 1 < lines.length && lines[i + 1].type === 'add') {
      result.push({
        type: 'modify',
        leftLine: lines[i].leftLine,
        rightLine: lines[i + 1].rightLine,
        leftLineNum: lines[i].leftLineNum,
        rightLineNum: lines[i + 1].rightLineNum,
      })
      i += 2
    } else {
      result.push(lines[i])
      i++
    }
  }

  return result
}

/**
 * 字符级对比（用于 modify 行）
 */
export function computeCharDiff(leftLine: string, rightLine: string): CharDiff[] {
  const changes = diffChars(leftLine, rightLine)
  return changes.map(c => ({
    type: c.added ? 'add' : c.removed ? 'remove' : 'equal',
    value: c.value,
  }))
}
```

- [ ] **Step 2: Commit**

```powershell
git add src/utils/diffUtils.ts
git commit -m "feat: add diffUtils with line and char diff functions"
```

---

### Task 3: 创建 DiffTool.vue 页面组件

**Files:**
- Create: `src/views/DiffTool.vue`

- [ ] **Step 1: 编写 DiffTool.vue 模板**

```vue
<template>
  <div class="tool-container">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持行级和字符级两种对比模式</p>
                <p>• 行级：以行为单位标记新增/删除/修改</p>
                <p>• 字符级：在修改行内高亮具体变化的字符</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-button size="small" @click="handleCompare">开始对比</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">对比模式</span>
            <div class="group-buttons">
              <el-radio-group v-model="diffMode" size="small">
                <el-radio-button label="line">行级</el-radio-button>
                <el-radio-button label="char">字符级</el-radio-button>
              </el-radio-group>
            </div>
          </div>
          <div class="action-group">
            <span class="group-label">选项</span>
            <div class="group-buttons">
              <el-switch v-model="autoCompare" active-text="自动执行" size="small" />
              <el-switch v-model="ignoreWhitespace" active-text="忽略空白" size="small" />
              <el-switch v-model="ignoreCase" active-text="忽略大小写" size="small" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-section">
      <div class="tool-card input-card">
        <div class="card-header">
          <span class="card-title">原始文本</span>
          <div class="card-actions">
            <el-button size="small" @click="handleClearLeft">清空</el-button>
            <el-button size="small" @click="handlePasteLeft">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="leftText"
            type="textarea"
            :rows="10"
            placeholder="请输入原始文本..."
            resize="none"
          />
        </div>
      </div>
      <div class="tool-card input-card">
        <div class="card-header">
          <span class="card-title">修改后文本</span>
          <div class="card-actions">
            <el-button size="small" @click="handleClearRight">清空</el-button>
            <el-button size="small" @click="handlePasteRight">粘贴</el-button>
          </div>
        </div>
        <div class="card-body">
          <el-input
            v-model="rightText"
            type="textarea"
            :rows="10"
            placeholder="请输入修改后的文本..."
            resize="none"
          />
        </div>
      </div>
    </div>

    <!-- 对比结果 -->
    <div v-if="diffResult.length > 0" class="tool-card result-card">
      <div class="card-header">
        <span class="card-title">对比结果</span>
        <div class="card-actions">
          <el-tag size="small" type="info">{{ diffResult.length }} 行</el-tag>
        </div>
      </div>
      <div class="card-body result-body">
        <div class="diff-container" @scroll="handleScroll">
          <div class="diff-left" ref="leftRef">
            <div
              v-for="(line, idx) in diffResult"
              :key="'left-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'left')"
            >
              <span class="line-num">{{ line.leftLineNum || '' }}</span>
              <span class="line-content">
                <template v-if="line.type === 'modify' && diffMode === 'char' && line.charDiffs">
                  <span
                    v-for="(char, ci) in line.charDiffs.filter(c => c.type !== 'add')"
                    :key="ci"
                    class="char-span"
                    :class="getCharClass(char)"
                  >{{ char.value }}</span>
                </template>
                <template v-else>
                  {{ line.leftLine ?? '' }}
                </template>
              </span>
            </div>
          </div>
          <div class="diff-right" ref="rightRef">
            <div
              v-for="(line, idx) in diffResult"
              :key="'right-' + idx"
              class="diff-line"
              :class="getLineClass(line, 'right')"
            >
              <span class="line-num">{{ line.rightLineNum || '' }}</span>
              <span class="line-content">
                <template v-if="line.type === 'modify' && diffMode === 'char' && line.charDiffs">
                  <span
                    v-for="(char, ci) in line.charDiffs.filter(c => c.type !== 'remove')"
                    :key="ci"
                    class="char-span"
                    :class="getCharClass(char)"
                  >{{ char.value }}</span>
                </template>
                <template v-else>
                  {{ line.rightLine ?? '' }}
                </template>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 2: 编写 DiffTool.vue script**

```vue
<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { computeLineDiff, computeCharDiff, DiffLine } from '@/utils/diffUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const leftText = ref('')
const rightText = ref('')
const diffMode = ref<'line' | 'char'>('line')
const autoCompare = ref(true)
const ignoreWhitespace = ref(false)
const ignoreCase = ref(false)
const diffResult = ref<DiffLine[]>([])

const leftRef = ref<HTMLElement>()
const rightRef = ref<HTMLElement>()

let autoCompareTimer: ReturnType<typeof setTimeout> | null = null

const runDiff = () => {
  if (!leftText.value && !rightText.value) {
    diffResult.value = []
    return
  }

  const lines = computeLineDiff(leftText.value, rightText.value, {
    ignoreWhitespace: ignoreWhitespace.value,
    ignoreCase: ignoreCase.value,
  })

  // 字符级模式：为 modify 行计算字符差异
  if (diffMode.value === 'char') {
    for (const line of lines) {
      if (line.type === 'modify' && line.leftLine && line.rightLine) {
        line.charDiffs = computeCharDiff(line.leftLine, line.rightLine)
      }
    }
  }

  diffResult.value = lines

  store.addHistory({
    tool: 'diff',
    action: diffMode.value === 'line' ? '行级对比' : '字符级对比',
    inputPreview: leftText.value.slice(0, 50),
    outputPreview: `${lines.length} 行差异`,
  })
}

const handleCompare = () => {
  runDiff()
  ElMessage.success('对比完成')
}

const handleClearLeft = () => { leftText.value = '' }
const handleClearRight = () => { rightText.value = '' }

const handlePasteLeft = async () => {
  try {
    leftText.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handlePasteRight = async () => {
  try {
    rightText.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const getLineClass = (line: DiffLine, side: 'left' | 'right') => {
  if (line.type === 'equal') return ''
  if (line.type === 'add' && side === 'right') return 'line-add'
  if (line.type === 'remove' && side === 'left') return 'line-remove'
  if (line.type === 'modify') return 'line-modify'
  return 'line-empty'
}

const getCharClass = (char: { type: string }) => {
  if (char.type === 'add') return 'char-add'
  if (char.type === 'remove') return 'char-remove'
  return ''
}

const handleScroll = (e: Event) => {
  const target = e.target as HTMLElement
  if (leftRef.value && rightRef.value) {
    if (target === leftRef.value) {
      rightRef.value.scrollTop = target.scrollTop
    } else {
      leftRef.value.scrollTop = target.scrollTop
    }
  }
}

// 自动对比
watch([leftText, rightText, diffMode, ignoreWhitespace, ignoreCase], () => {
  if (autoCompare.value) {
    if (autoCompareTimer) clearTimeout(autoCompareTimer)
    autoCompareTimer = setTimeout(runDiff, 300)
  }
})
</script>
```

- [ ] **Step 3: 编写 DiffTool.vue 样式**

```vue
<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.input-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.input-card { margin-bottom: 0; }

.result-body { padding: 0; }
.diff-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  max-height: 60vh;
  overflow: auto;
}
.diff-left, .diff-right {
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.diff-line {
  display: flex;
  padding: 2px 8px;
  min-height: 22px;
  white-space: pre;
}
.line-num {
  width: 40px;
  text-align: right;
  padding-right: 12px;
  color: var(--text-muted);
  user-select: none;
  flex-shrink: 0;
}
.line-content { flex: 1; }

.line-add { background: rgba(34, 197, 94, 0.15); }
.line-remove { background: rgba(239, 68, 68, 0.15); }
.line-modify { background: rgba(234, 179, 8, 0.1); }
.line-empty { opacity: 0.3; }

.char-add { background: rgba(34, 197, 94, 0.3); border-radius: 2px; }
.char-remove { background: rgba(239, 68, 68, 0.3); border-radius: 2px; text-decoration: line-through; }
</style>
```

- [ ] **Step 4: Commit**

```powershell
git add src/views/DiffTool.vue
git commit -m "feat: add DiffTool page component with dual-pane diff view"
```

---

### Task 4: 注册路由和侧边栏入口

**Files:**
- Modify: `src/App.vue`
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 store/index.ts 添加工具定义**

在 `TOOL_LIST` 数组末尾添加：

```typescript
{ id: 'diff', name: '文本对比', icon: '≠', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h16M4 12h16M4 18h16"/><path d="M9 3v18M15 3v18"/></svg>`, description: '文本/代码对比，支持行级和字符级差异高亮', keywords: ['对比', 'diff', '差异', '代码对比'] },
```

- [ ] **Step 2: 在 App.vue 注册组件和路由**

在 `<script setup>` 中添加 import：

```typescript
import DiffTool from '@/views/DiffTool.vue'
```

在 `<main>` 中添加路由：

```vue
<DiffTool v-else-if="activeTool === 'diff'" />
```

- [ ] **Step 3: Commit**

```powershell
git add src/App.vue src/store/index.ts
git commit -m "feat: register DiffTool in app routes and sidebar"
```

---

### Task 5: 测试验证

**Files:**
- 手动测试

- [ ] **Step 1: 启动开发服务器**

```powershell
npm.cmd run dev
```

- [ ] **Step 2: 测试场景**

1. 打开「文本对比」工具，确认侧边栏入口正常
2. 在左栏输入 `hello\nworld`，右栏输入 `hello\nworld\nfoo`，确认自动对比显示新增行
3. 切换到字符级模式，修改右栏为 `hello\nword`，确认字符级高亮（d 被删除）
4. 测试「忽略空白」开关：左栏 `a b`，右栏 `a  b`，开启后应显示相同
5. 测试「忽略大小写」开关：左栏 `Hello`，右栏 `hello`，开启后应显示相同
6. 测试双栏同步滚动
7. 测试清空和粘贴功能

- [ ] **Step 3: Commit（如有修改）**

```powershell
git add .
git commit -m "fix: address issues found during diff tool testing"
```
