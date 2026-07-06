# 多 Tab 导航模式 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把当前单一 `activeTool` 视图切换改造成浏览器风格的多 Tab 模式，允许同时打开多个工具并通过顶部 Tab 栏切换。

**Architecture:** Store 中 `activeTool: string` → `tabs: Tab[]` + `activeTabId: string`，新增 `openTab/closeTab/switchTab/closeOthers` 方法。App.vue 的 `v-if` 链改为动态 `<component :is>` + `<KeepAlive :max="8">`，配合变化的 `:key` 实现关闭即清理状态。新增 `TabBar.vue` 组件渲染顶部 Tab 栏。SidebarNav/HomeView 的点击行为统一走 `store.openTab`。

**Tech Stack:** Vue 3 Composition API + Pinia + Element Plus + TypeScript

**约束：**
- 同工具不允许多实例（点击已打开工具 → 激活对应 Tab）
- Tab 上限 8 个，超出 LRU 关闭最早非 home Tab
- Tab 列表不持久化（重启只恢复 `lastTool` 单工具）
- home Tab 固定首位，不可关闭

---

## File Structure

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/store/index.ts` | 修改 | Tab 状态管理：tabs 数组、activeTabId、openTab/closeTab/switchTab/closeOthers/closeAllTabs |
| `src/components/TabBar.vue` | 新增 | 顶部 Tab 栏 UI：标题、关闭按钮、右键菜单（关闭其他/全部关闭） |
| `src/App.vue` | 修改 | v-if 链 → 动态 `<component :is>` + KeepAlive；集成 TabBar；快捷键监听改 openTab |
| `src/components/SidebarNav.vue` | 修改 | 点击行为 emit `openTool`；active 判断改为 `tabs.find(t => t.toolId === id)?.id === activeTabId` |
| `src/views/HomeView.vue` | 修改 | `onSelectTool` 改走 `store.openTab` |
| `src/style/theme.css` | 修改 | 新增 tab 相关 CSS 变量（复用现有变量为主） |
| `package.json` | 修改 | 版本号 4.4.0 → 4.5.0 |
| `README.md` | 修改 | 版本路线表追加 V4.5 |

---

## Task 1: Store 改造 - Tab 状态与操作方法

**Files:**
- Modify: `src/store/index.ts:114` （替换 `activeTool` 定义）
- Modify: `src/store/index.ts:251-264` （return 块补充新方法）
- Test: `src/store/tabs.test.ts` （新增，验证 LRU 与关闭逻辑）

### Step 1: 写失败测试

- [ ] **创建测试文件 `src/store/tabs.test.ts`**

```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useToolboxStore } from './index'

describe('Tab 状态管理', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('初始状态只有 home tab，activeTabId 为 home', () => {
    const store = useToolboxStore()
    expect(store.tabs).toHaveLength(1)
    expect(store.tabs[0].toolId).toBe('home')
    expect(store.activeTabId).toBe('home')
  })

  it('openTab 新工具时追加到 tabs 末尾并激活', () => {
    const store = useToolboxStore()
    store.openTab('json')
    expect(store.tabs).toHaveLength(2)
    expect(store.tabs[1].toolId).toBe('json')
    expect(store.activeTabId).toBe('json')
  })

  it('openTab 已打开工具时仅激活，不重复添加', () => {
    const store = useToolboxStore()
    store.openTab('json')
    store.openTab('time')
    store.openTab('json')  // 再次打开 json
    expect(store.tabs).toHaveLength(3)  // home + json + time
    expect(store.activeTabId).toBe('json')
  })

  it('openTab 超过 8 个时 LRU 关闭最早非 home tab', () => {
    const store = useToolboxStore()
    // 打开 8 个工具（加上 home 共 9 个，会触发 LRU）
    store.openTab('t1' as any)
    store.openTab('t2' as any)
    store.openTab('t3' as any)
    store.openTab('t4' as any)
    store.openTab('t5' as any)
    store.openTab('t6' as any)
    store.openTab('t7' as any)
    store.openTab('t8' as any)
    // home + 7 个工具 = 8 个，t1 被淘汰
    expect(store.tabs).toHaveLength(8)
    expect(store.tabs.find(t => t.toolId === 't1')).toBeUndefined()
    expect(store.tabs[0].toolId).toBe('home')
  })

  it('closeTab 关闭非 active tab 不影响 activeTabId', () => {
    const store = useToolboxStore()
    store.openTab('json')
    store.openTab('time')
    store.closeTab('json')  // 关闭非当前
    expect(store.activeTabId).toBe('time')
    expect(store.tabs).toHaveLength(2)  // home + time
  })

  it('closeTab 关闭 active tab 时激活相邻 tab', () => {
    const store = useToolboxStore()
    store.openTab('json')
    store.openTab('time')
    store.closeTab('time')  // 关闭当前，应激活 json（前一个）
    expect(store.activeTabId).toBe('json')
  })

  it('closeTab 不允许关闭 home', () => {
    const store = useToolboxStore()
    store.closeTab('home')
    expect(store.tabs.find(t => t.toolId === 'home')).toBeDefined()
  })

  it('closeOthers 保留 active 和 home，关闭其他', () => {
    const store = useToolboxStore()
    store.openTab('json')
    store.openTab('time')
    store.openTab('sql')
    store.switchTab('json')
    store.closeOthers('json')
    expect(store.tabs.map(t => t.toolId)).toEqual(['home', 'json'])
  })

  it('closeTab 后 closedCount 递增（用于 KeepAlive key 变化）', () => {
    const store = useToolboxStore()
    store.openTab('json')
    store.closeTab('json')
    expect(store.closedCount['json']).toBe(1)
    store.openTab('json')
    store.closeTab('json')
    expect(store.closedCount['json']).toBe(2)
  })
})
```

### Step 2: 运行测试验证失败

- [ ] **运行测试**

Run: `npx vitest run src/store/tabs.test.ts`
Expected: FAIL，提示 `store.tabs is undefined` 或类似错误（因为 store 还没改造）

注：项目当前未安装 vitest。若不想引入新依赖，可改用以下 node 脚本手动验证：

```bash
# 替代方案：node 脚本验证
node -e "console.log('manual check placeholder')"
```

实际本项目无单测框架（只有 diffUtils.test.ts 用 node --test）。采用 **assert 内联自检** 替代：

- [ ] **改用 assert 自检文件 `src/store/tabs.selfcheck.ts`**

```typescript
// ponytail: 简单 assert 自检，不引入测试框架。开发时运行 npx tsx src/store/tabs.selfcheck.ts 验证
import { setActivePinia, createPinia } from 'pinia'
import { useToolboxStore } from './index'

function assert(cond: boolean, msg: string) {
  if (!cond) { console.error('❌ FAIL:', msg); process.exit(1) }
  else console.log('✅ PASS:', msg)
}

setActivePinia(createPinia())
const store = useToolboxStore()

assert(store.tabs.length === 1, '初始只有 home tab')
assert(store.tabs[0].toolId === 'home', '初始 tab 是 home')
assert(store.activeTabId === 'home', '初始 activeTabId 是 home')

store.openTab('json')
assert(store.tabs.length === 2, 'openTab 新工具追加')
assert(store.activeTabId === 'json', 'openTab 激活新 tab')

store.openTab('time')
store.openTab('json')
assert(store.tabs.length === 3, 'openTab 已存在工具不重复添加')
assert(store.activeTabId === 'json', 'openTab 已存在工具仅激活')

store.closeTab('time')
assert(store.activeTabId === 'json', 'closeTab 非当前不影响 active')
assert(store.tabs.length === 2, 'closeTab 减少数量')

store.closeTab('json')
assert(store.activeTabId === 'home', 'closeTab 当前 tab 激活相邻')

store.closeTab('home')
assert(store.tabs.find(t => t.toolId === 'home') !== undefined, 'home 不可关闭')

// LRU
const s2 = useToolboxStore()
;(s2 as any).tabs = [{ toolId: 'home' }]
;(s2 as any).activeTabId = 'home'
for (let i = 1; i <= 8; i++) s2.openTab(`t${i}` as any)
assert(s2.tabs.length === 8, 'LRU 限制 8 个')
assert(s2.tabs.find((t: any) => t.toolId === 't1') === undefined, 'LRU 淘汰最早非 home')
assert(s2.tabs[0].toolId === 'home', 'LRU 保留 home')

console.log('\n全部通过')
```

### Step 3: 改造 store

- [ ] **修改 `src/store/index.ts` 第 114 行附近，替换 `activeTool` 定义**

原代码（第 114 行）：
```typescript
  const activeTool = ref('home')
```

替换为：
```typescript
  // ============ 多 Tab 状态 ============
  interface Tab {
    toolId: string  // 同工具不允许多实例，toolId 即 tab 唯一标识
  }

  const MAX_TABS = 8
  const tabs = ref<Tab[]>([{ toolId: 'home' }])
  const activeTabId = ref('home')
  // 记录每个 toolId 被关闭的次数，作为 KeepAlive :key 的一部分，
  // 关闭后重新打开时 key 变化 → 强制创建新实例（不复用旧缓存状态）
  const closedCount = ref<Record<string, number>>({})

  /** 打开工具：已存在则激活，否则新建 tab（超出上限 LRU 关闭最早非 home tab） */
  const openTab = (toolId: string) => {
    const existing = tabs.value.find(t => t.toolId === toolId)
    if (existing) {
      activeTabId.value = toolId
      return
    }
    // LRU：超出上限时关闭最早的非 home tab
    if (tabs.value.length >= MAX_TABS) {
      const idx = tabs.value.findIndex(t => t.toolId !== 'home')
      if (idx !== -1) {
        const removed = tabs.value.splice(idx, 1)[0]
        closedCount.value[removed.toolId] = (closedCount.value[removed.toolId] || 0) + 1
      }
    }
    tabs.value.push({ toolId })
    activeTabId.value = toolId
  }

  /** 切换 tab */
  const switchTab = (toolId: string) => {
    if (tabs.value.find(t => t.toolId === toolId)) {
      activeTabId.value = toolId
    }
  }

  /** 关闭 tab：home 不可关闭；关闭当前 tab 时激活相邻 tab */
  const closeTab = (toolId: string) => {
    if (toolId === 'home') return
    const idx = tabs.value.findIndex(t => t.toolId === toolId)
    if (idx === -1) return
    tabs.value.splice(idx, 1)
    closedCount.value[toolId] = (closedCount.value[toolId] || 0) + 1
    // 调整 activeTabId
    if (activeTabId.value === toolId) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)]
      activeTabId.value = next ? next.toolId : 'home'
    }
  }

  /** 关闭其他：保留 home 和指定 tab */
  const closeOthers = (keepToolId: string) => {
    const removed = tabs.value.filter(t => t.toolId !== 'home' && t.toolId !== keepToolId)
    for (const t of removed) {
      closedCount.value[t.toolId] = (closedCount.value[t.toolId] || 0) + 1
    }
    tabs.value = tabs.value.filter(t => t.toolId === 'home' || t.toolId === keepToolId)
    activeTabId.value = keepToolId
  }

  /** 关闭全部：仅保留 home */
  const closeAllTabs = () => {
    const removed = tabs.value.filter(t => t.toolId !== 'home')
    for (const t of removed) {
      closedCount.value[t.toolId] = (closedCount.value[t.toolId] || 0) + 1
    }
    tabs.value = [{ toolId: 'home' }]
    activeTabId.value = 'home'
  }

  /** 获取 KeepAlive 的 :key（toolId + 关闭计数，保证关闭后重开是新实例） */
  const getTabKey = (toolId: string) => `${toolId}-${closedCount.value[toolId] || 0}`

  /** 兼容旧代码：activeTool 作为计算属性指向 activeTabId */
  const activeTool = computed(() => activeTabId.value)
```

- [ ] **在文件顶部 import 中添加 `computed`**

原代码（第 2 行）：
```typescript
import { ref } from 'vue'
```

替换为：
```typescript
import { ref, computed } from 'vue'
```

- [ ] **修改 return 块（第 251-264 行附近），导出新成员**

原代码：
```typescript
  return {
    config,
    history,
    recentTools,
    saveConfig,
    addHistory,
    clearHistory,
    addRecentTool,
    toggleFavorite,
    pendingHistoryRestore,
    triggerHistoryRestore,
    clearHistoryRestore,
    activeTool,
  }
```

替换为：
```typescript
  return {
    config,
    history,
    recentTools,
    saveConfig,
    addHistory,
    clearHistory,
    addRecentTool,
    toggleFavorite,
    pendingHistoryRestore,
    triggerHistoryRestore,
    clearHistoryRestore,
    activeTool,
    // 多 Tab
    tabs,
    activeTabId,
    closedCount,
    openTab,
    switchTab,
    closeTab,
    closeOthers,
    closeAllTabs,
    getTabKey,
  }
```

### Step 4: 运行自检验证通过

- [ ] **运行自检脚本**

Run: `npx tsx src/store/tabs.selfcheck.ts`
Expected: 输出 `全部通过`，所有 ✅ PASS

注：若未安装 tsx，可临时用 `npx ts-node src/store/tabs.selfcheck.ts` 或在 dev server 控制台手动调用。

### Step 5: 提交

- [ ] **git 提交**

```bash
git add src/store/index.ts src/store/tabs.selfcheck.ts
git commit -m "feat: store 新增多 Tab 状态管理（tabs/activeTabId/openTab/closeTab/LRU）"
```

---

## Task 2: 创建 TabBar 组件

**Files:**
- Create: `src/components/TabBar.vue`

### Step 1: 创建组件

- [ ] **创建 `src/components/TabBar.vue`**

```vue
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
```

注：使用 `defineOptions({ name: 'TabBar' })` 设置组件名（Vue 3.3+ 原生支持），便于 KeepAlive include 与 devtools 识别。

### Step 2: 样式

- [ ] **追加 `<style scoped>` 到 `src/components/TabBar.vue`**

```vue
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
```

### Step 3: 提交

- [ ] **git 提交**

```bash
git add src/components/TabBar.vue
git commit -m "feat: 新增 TabBar 组件（顶部 Tab 栏、关闭按钮、右键菜单）"
```

---

## Task 3: App.vue 改造 - 动态组件 + TabBar 集成

**Files:**
- Modify: `src/App.vue:1-60` （template）
- Modify: `src/App.vue:62-170` （script）

### Step 1: 改造 template

- [ ] **替换 `src/App.vue` 的 `<template>` 部分（第 1-60 行）**

原代码：
```vue
<template>
  <div class="app-layout">
    <SidebarNav v-model="activeTool" />
    
    <div class="app-content">
      <main class="app-main">
        <KeepAlive :max="4">
          <HomeView v-if="activeTool === 'home'" :key="'home'" :on-select-tool="handleSelectTool" />
          <JsonTool v-else-if="activeTool === 'json'" :key="'json'" />
          ...（48 个 v-else-if）
        </KeepAlive>
      </main>
      
      <div class="app-footer">
        <span>© 2026 栗的百宝箱 · Made by liam</span>
      </div>
    </div>
  </div>
</template>
```

替换为：
```vue
<template>
  <div class="app-layout">
    <SidebarNav v-model="activeTool" />
    
    <div class="app-content">
      <TabBar />
      <main class="app-main">
        <KeepAlive :max="8">
          <component
            :is="toolComponentMap[activeTabId]"
            :key="store.getTabKey(activeTabId)"
            v-bind="activeTabId === 'home' ? { onSelectTool: handleSelectTool } : {}"
          />
        </KeepAlive>
      </main>
      
      <div class="app-footer">
        <span>© 2026 栗的百宝箱 · Made by liam</span>
      </div>
    </div>
  </div>
</template>
```

### Step 2: 改造 script

- [ ] **替换 `src/App.vue` 的 `<script setup>` 部分（第 62-170 行）**

原代码（关键部分）：
```typescript
import { watch, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { storeToRefs } from 'pinia'
import SidebarNav from '@/components/SidebarNav.vue'
import HomeView from '@/views/HomeView.vue'
import JsonTool from '@/views/JsonTool.vue'
...（48 个 import）

const store = useToolboxStore()
const { activeTool } = storeToRefs(store)

activeTool.value = store.config.lastTool

let unlistenShortcut: (() => void) | null = null

const handleSelectTool = (toolId: string) => {
  activeTool.value = toolId
}

watch(activeTool, (newTool: string) => {
  store.saveConfig({ lastTool: newTool })
})
```

替换为：
```typescript
import { watch, onMounted, onUnmounted, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { storeToRefs } from 'pinia'
import SidebarNav from '@/components/SidebarNav.vue'
import TabBar from '@/components/TabBar.vue'
import HomeView from '@/views/HomeView.vue'
import JsonTool from '@/views/JsonTool.vue'
import StringTool from '@/views/StringTool.vue'
import EncodeTool from '@/views/EncodeTool.vue'
import TimeTool from '@/views/TimeTool.vue'
import URLTool from '@/views/URLTool.vue'
import RegexTool from '@/views/RegexTool.vue'
import BaseConverter from '@/views/BaseConverter.vue'
import UUIDTool from '@/views/UUIDTool.vue'
import DevTools from '@/views/DevTools.vue'
import FileProcessing from '@/views/FileProcessing.vue'
import SqlTool from '@/views/SqlTool.vue'
import JSTool from '@/views/JSTool.vue'
import MockDataTool from '@/views/MockDataTool.vue'
import OcrTool from '@/views/OcrTool.vue'
import DiffTool from '@/views/DiffTool.vue'
import ClipboardTool from '@/views/ClipboardTool.vue'
import ImageTool from '@/views/ImageTool.vue'
import CsvTool from '@/views/CsvTool.vue'
import PdfTool from '@/views/PdfTool.vue'
import HashTool from '@/views/HashTool.vue'
import XmlYamlTool from '@/views/XmlYamlTool.vue'
import DedupTool from '@/views/DedupTool.vue'
import CssTool from '@/views/CssTool.vue'
import JwtTool from '@/views/JwtTool.vue'
import WordCountTool from '@/views/WordCountTool.vue'
import CronTool from '@/views/CronTool.vue'
import MarkdownTool from '@/views/MarkdownTool.vue'
import ColorTool from '@/views/ColorTool.vue'
import PasswordTool from '@/views/PasswordTool.vue'
import QrTool from '@/views/QrTool.vue'
import SnippetTool from '@/views/SnippetTool.vue'
import HttpTool from '@/views/HttpTool.vue'
import HistoryView from '@/views/HistoryView.vue'
import WorkflowView from '@/views/WorkflowView.vue'
import NoteEditor from '@/views/NoteEditor.vue'
import CalculatorTool from '@/views/CalculatorTool.vue'
import SystemInfoView from '@/views/SystemInfoView.vue'
import NetworkInfoView from '@/views/NetworkInfoView.vue'
import ProcessListView from '@/views/ProcessListView.vue'
import HardwareInfoView from '@/views/HardwareInfoView.vue'
import SoftwareEnvView from '@/views/SoftwareEnvView.vue'
import SqliteViewerView from '@/views/SqliteViewerView.vue'
import DiskSpaceAnalyzer from '@/views/DiskSpaceAnalyzer.vue'

// toolId → 组件 映射表（替代 v-if 链）
const toolComponentMap: Record<string, any> = {
  home: HomeView,
  json: JsonTool,
  string: StringTool,
  encode: EncodeTool,
  time: TimeTool,
  url: URLTool,
  regex: RegexTool,
  baseConverter: BaseConverter,
  uuid: UUIDTool,
  devtools: DevTools,
  fileprocessing: FileProcessing,
  sql: SqlTool,
  js: JSTool,
  mockData: MockDataTool,
  ocr: OcrTool,
  diff: DiffTool,
  clipboard: ClipboardTool,
  image: ImageTool,
  csv: CsvTool,
  pdf: PdfTool,
  hash: HashTool,
  xmlYaml: XmlYamlTool,
  dedup: DedupTool,
  css: CssTool,
  jwt: JwtTool,
  wordCount: WordCountTool,
  cron: CronTool,
  markdown: MarkdownTool,
  color: ColorTool,
  password: PasswordTool,
  qr: QrTool,
  snippet: SnippetTool,
  http: HttpTool,
  history: HistoryView,
  workflow: WorkflowView,
  note: NoteEditor,
  calculator: CalculatorTool,
  systemInfo: SystemInfoView,
  networkInfo: NetworkInfoView,
  processList: ProcessListView,
  hardwareInfo: HardwareInfoView,
  softwareEnv: SoftwareEnvView,
  sqliteViewer: SqliteViewerView,
  diskAnalyzer: DiskSpaceAnalyzer,
}

const store = useToolboxStore()
const { activeTabId } = storeToRefs(store)

// 兼容 SidebarNav 的 v-model="activeTool"：activeTool 仍是 computed 指向 activeTabId
const activeTool = computed({
  get: () => store.activeTabId,
  set: (val: string) => store.openTab(val),  // SidebarNav 设置时走 openTab
})

// 初始化：恢复上次使用的工具（单 tab，不持久化 tab 列表）
store.openTab(store.config.lastTool || 'home')

let unlistenShortcut: (() => void) | null = null

const handleSelectTool = (toolId: string) => {
  store.openTab(toolId)
  store.addRecentTool(toolId)
}

// lastTool 跟随 activeTabId 变化
watch(activeTabId, (newTool: string) => {
  store.saveConfig({ lastTool: newTool })
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

onMounted(async () => {
  applyTheme(store.config.theme)
  
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (store.config.theme === 'auto') {
      applyTheme('auto')
    }
  })
  
  unlistenShortcut = await listen('global-shortcut-triggered', (event) => {
    const toolId = event.payload as string
    if (toolId) {
      store.openTab(toolId)
      store.addRecentTool(toolId)
    }
  })
})

onUnmounted(() => {
  if (unlistenShortcut) {
    unlistenShortcut()
  }
})
```

### Step 3: 调整 CSS（app-main 高度计算）

- [ ] **修改 `src/App.vue` 的 `<style scoped>` 中 `.app-main`，确保 TabBar 占位后剩余空间给 main**

原代码：
```css
.app-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100vh;
}

.app-main {
  flex: 1;
  overflow: hidden;
}
```

保持不变（TabBar 是 `.app-content` 的直接子元素，flex 布局会自动分配空间，`flex-shrink: 0` 已在 TabBar 中设置）。

### Step 4: 启动 dev 验证

- [ ] **启动开发服务器**

Run: `npm run tauri dev`
Expected: 应用启动，顶部出现 Tab 栏，默认显示 home Tab。点击侧边栏工具会新增 Tab 并切换。切换 Tab 不丢失工具内部状态。关闭 Tab 后重新打开是新实例。

### Step 5: 提交

- [ ] **git 提交**

```bash
git add src/App.vue
git commit -m "feat: App.vue 改造为动态组件 + TabBar 集成，KeepAlive 支持多 Tab 缓存"
```

---

## Task 4: SidebarNav 改造 - 点击行为与 active 判断

**Files:**
- Modify: `src/components/SidebarNav.vue:374-377` （handleSelect）
- Modify: `src/components/SidebarNav.vue` template 中的 active 判断

### Step 1: 改造点击行为

- [ ] **修改 `src/components/SidebarNav.vue` 第 374-377 行的 `handleSelect`**

原代码：
```typescript
const handleSelect = (toolId: string) => {
  emit('update:modelValue', toolId)
  store.addRecentTool(toolId)
}
```

替换为：
```typescript
const handleSelect = (toolId: string) => {
  // 多 Tab 模式：点击工具直接走 openTab（通过 v-model 触发 App.vue 的 setter）
  emit('update:modelValue', toolId)
  store.addRecentTool(toolId)
}
```

注：emit 逻辑不变，因为 App.vue 中 `activeTool` 的 setter 已改为 `store.openTab(val)`。`addRecentTool` 仍在侧边栏调用（保持原行为）。

### Step 2: 改造 active 判断

- [ ] **修改 `src/components/SidebarNav.vue` template 中所有 `:class="{ active: modelValue === tool.id }"`**

原代码（多处，如第 20、44、75、108 行）：
```vue
:class="{ active: modelValue === tool.id }"
```

替换为：
```vue
:class="{ active: modelValue === tool.id }"
```

注：**保持不变**。因为 `modelValue`（即 App.vue 的 `activeTool` computed get）现在指向 `store.activeTabId`，判断逻辑依然正确——只有当前激活的 tab 对应的工具会高亮。已打开但未激活的工具在侧边栏不高亮（这是预期行为，侧边栏表示「当前焦点」）。

### Step 3: 验证

- [ ] **手动验证**

1. 点击侧边栏「JSON工具」→ 新增 JSON Tab 并激活
2. 再点击「时间工具」→ 新增 Time Tab 并激活
3. 切回 JSON Tab → 侧边栏「JSON工具」高亮，「时间工具」不高亮
4. 再次点击「JSON工具」→ 不新增，仅激活（已打开）

### Step 4: 提交

- [ ] **git 提交**

```bash
git add src/components/SidebarNav.vue
git commit -m "feat: SidebarNav 点击行为对接多 Tab（通过 v-model setter 走 openTab）"
```

---

## Task 5: HomeView 改造 - openTab 调用

**Files:**
- Modify: `src/views/HomeView.vue:160-162` （handleSelectTool）

### Step 1: 改造 handleSelectTool

- [ ] **修改 `src/views/HomeView.vue` 第 160-162 行**

原代码：
```typescript
const handleSelectTool = (toolId: string) => {
  props.onSelectTool(toolId)
}
```

替换为：
```typescript
const handleSelectTool = (toolId: string) => {
  props.onSelectTool(toolId)
  // 多 Tab 模式下，addRecentTool 由 App.vue 的 handleSelectTool 统一调用
}
```

注：**实际无需修改 HomeView**。`props.onSelectTool` 在 App.vue 中已绑定到 `handleSelectTool`，后者调用 `store.openTab` + `store.addRecentTool`。HomeView 只是透传 toolId，逻辑保持不变。

- [ ] **跳过此任务，直接进入下一任务**

---

## Task 6: theme.css 增加 tab 样式变量（可选）

**Files:**
- Modify: `src/style/theme.css`

### Step 1: 评估是否需要新增变量

- [ ] **检查 TabBar.vue 中使用的 CSS 变量是否已存在**

TabBar.vue 使用的变量：`--bg-secondary`、`--border-color`、`--hover-bg`、`--bg-primary`、`--accent-cyan`、`--text-secondary`、`--text-primary`、`--text-muted`、`--bg-card`、`--accent-red`。

全部已存在于 theme.css 第 1-20 行（:root）和第 88-107 行（html.dark）。

- [ ] **跳过此任务，无需修改 theme.css**

---

## Task 7: 版本号 + README 更新

**Files:**
- Modify: `package.json:3`
- Modify: `src/components/SidebarNav.vue:8` （侧边栏显示的版本号）
- Modify: `README.md:304` （版本路线表追加 V4.5）

### Step 1: 更新 package.json 版本号

- [ ] **修改 `package.json` 第 3 行**

原代码：
```json
  "version": "4.4.0",
```

替换为：
```json
  "version": "4.5.0",
```

### Step 2: 更新侧边栏版本号显示

- [ ] **修改 `src/components/SidebarNav.vue` 第 8 行**

原代码：
```vue
          <span class="app-version">v4.3</span>
```

替换为：
```vue
          <span class="app-version">v4.5</span>
```

### Step 3: 更新 README 版本路线

- [ ] **修改 `README.md` 第 304 行后追加新行**

原代码（第 304 行）：
```
| V4.4 | ✅ | 进程 kill 与端口释放（ProcessListView 加结束按钮、NetworkInfoView 监听端口加释放按钮，共享 kill_process 后端命令） |
```

在其下方追加：
```
| V4.5 | ✅ | 多 Tab 导航模式（顶部 Tab 栏、同时打开多个工具、KeepAlive 状态保留、右键菜单关闭其他/全部、LRU 上限 8 个） |
```

### Step 4: 提交

- [ ] **git 提交**

```bash
git add package.json src/components/SidebarNav.vue README.md
git commit -m "chore: 版本号 4.4.0 → 4.5.0，README 追加 V4.5 多 Tab 导航功能"
```

---

## 验收清单

完成所有任务后，手动验证以下场景：

- [ ] 应用启动默认显示 home Tab，侧边栏「首页」高亮
- [ ] 点击侧边栏工具 → 新增 Tab 并激活
- [ ] 点击已打开工具的侧边栏项 → 不新增，仅激活对应 Tab
- [ ] 切换 Tab → 工具内部状态保留（输入框内容、滚动位置等）
- [ ] 点击 Tab 关闭按钮 → 关闭该 Tab，激活相邻 Tab
- [ ] 中键点击 Tab → 关闭该 Tab
- [ ] 右键 Tab → 显示菜单（关闭/关闭其他/关闭全部）
- [ ] home Tab 不可关闭（无关闭按钮，右键无「关闭」选项）
- [ ] 打开 8 个工具后再开新工具 → LRU 关闭最早非 home Tab
- [ ] 关闭 Tab 后重新打开同工具 → 是新实例（输入框为空）
- [ ] 全局快捷键触发 → 打开/激活对应工具 Tab
- [ ] HomeView 点击工具卡片 → 走 openTab
- [ ] 历史记录双击跳转 → 在当前 Tab 还原（不新开 Tab）
- [ ] 重启应用 → 只恢复 lastTool 单个 Tab（不持久化 tab 列表）
- [ ] 深色/浅色主题切换 → Tab 栏样式正确

---

## 风险点与回滚

**风险1：KeepAlive 缓存 key 失效**
- 现象：关闭 Tab 后重开，看到旧状态
- 检查：`store.getTabKey(toolId)` 是否在 closeTab 后递增 closedCount
- 回滚：把 `:key="store.getTabKey(activeTabId)"` 改回 `:key="activeTabId"`

**风险2：部分工具组件多实例冲突**
- 现象：NoteEditor/WorkflowView 等有数据库连接的工具在多 Tab 下报错
- 检查：开发者控制台是否有 SQL 锁定/连接泄漏错误
- 缓解：因不允许多实例（同工具只能开一个 Tab），实际同时挂载的是不同工具，冲突概率低

**风险3：Tab 栏横向溢出**
- 现象：8 个 Tab 时横向滚动不流畅
- 检查：`.tab-list` 的 `overflow-x: auto` 是否生效
- 缓解：已设置 `min-width: 80px; max-width: 180px` 限制单 Tab 宽度
