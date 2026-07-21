# 快捷命令面板实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 LitoBox 新增全局热键 `Ctrl+Alt+P` 呼出的快捷命令面板，模糊搜索并跳转到任意工具。

**Architecture:** 全局热键复用现有 `shortcuts` 表（特殊 id `__palette__`），main.rs 触发时 `show()+set_focus()` 唤起窗口并 emit `command-palette-triggered` 事件；前端 `CommandPalette.vue` 浮层（Teleport to body）搜索 `TOOL_LIST`，选中后复用 `store.openTab` 跳转。搜索逻辑抽成纯函数 `filterTools()` 便于独立测试。

**Tech Stack:** Tauri 2.0 + Vue 3 (Composition API) + TypeScript + Pinia + Element Plus

**Spec:** `docs/superpowers/specs/2026-07-22-command-palette-design.md`

---

## 文件结构

| 文件 | 动作 | 职责 |
|------|------|------|
| `src-tauri/src/db.rs` | 改 | `db_read_shortcuts` default 加 `__palette__` 条目 |
| `src-tauri/src/main.rs` | 改 | 加 `debug_log!` 宏 + `on_shortcut` 回调 `__palette__` 分支 |
| `src/utils/commandPalette.ts` | 新建 | 纯函数 `filterTools()` + `RankedTool` 类型 |
| `src/utils/commandPalette.test.ts` | 新建 | assert 自检脚本（无框架） |
| `src/store/index.ts` | 改 | `isCommandPaletteOpen` 状态 + 方法 + shortcuts 默认值 |
| `src/components/CommandPalette.vue` | 新建 | 浮层 UI + 搜索 + 键盘导航 |
| `src/App.vue` | 改 | 挂载组件 + 监听 `command-palette-triggered` + 全局 `Ctrl+P` toggle |
| `src/components/SidebarNav.vue` | 改 | `SHORTCUT_TOOLS` 加命令面板配置项 |
| `package.json` | 改 | version 5.9.0 → 6.0.0 |
| `src-tauri/tauri.conf.json` | 改 | version 同步（若有） |
| `README.md` | 改 | 功能阶段记录新增 V6.0 |
| `docs/superpowers/plans/feature-backlog.md` | 改 | D3 移到已完成版本表 |

---

## Task 1: 后端 — db.rs 默认值注入 `__palette__`

**Files:**
- Modify: `src-tauri/src/db.rs:1798-1804`

- [ ] **Step 1: 修改 default 列表**

在 `db_read_shortcuts` 的 `default` vec 末尾加 `__palette__` 条目。打开 `src-tauri/src/db.rs`，定位 `let default = vec![` (约 line 1798)，改为：

```rust
let default = vec![
    ("json".to_string(), "CmdOrCtrl+Alt+J".to_string()),
    ("string".to_string(), "CmdOrCtrl+Alt+S".to_string()),
    ("encode".to_string(), "CmdOrCtrl+Alt+E".to_string()),
    ("regex".to_string(), "CmdOrCtrl+Alt+R".to_string()),
    ("http".to_string(), "CmdOrCtrl+Alt+H".to_string()),
    // 命令面板特殊 id（非真实工具），main.rs 触发时走 show+focus+emit 分支
    ("__palette__".to_string(), "CmdOrCtrl+Alt+P".to_string()),
];
```

- [ ] **Step 2: cargo check 验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat(palette): db_read_shortcuts 默认值注入 __palette__ 热键"
```

---

## Task 2: 后端 — main.rs debug_log 宏 + `__palette__` 分支

**Files:**
- Modify: `src-tauri/src/main.rs` (顶部加宏 + line 246-254 回调改动)

- [ ] **Step 1: 在 main.rs 顶部添加 debug_log 宏**

`main.rs` 顶部 `use` 语句之前（`mod` 声明之后，约 line 22 后）加宏定义。参考 `src-tauri/src/audio_tools.rs:1191-1198` 的标准模式：

```rust
// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}
```

- [ ] **Step 2: 修改 on_shortcut 回调加 __palette__ 分支**

定位 `src-tauri/src/main.rs` 的 `manager.on_shortcut(shortcut, move |_app, _sc, event| { ... })`（约 line 246-254），改为：

```rust
manager.on_shortcut(shortcut, move |_app, _sc, event| {
    if let tauri_plugin_global_shortcut::ShortcutState::Pressed = event.state {
        if let Some(window) = h.get_webview_window("main") {
            if tool == "__palette__" {
                // 命令面板：先唤起窗口到前台（show 幂等，已显示无副作用）
                let _ = window.show();
                let _ = window.set_focus();
                debug_log!("[command_palette] global hotkey triggered, window shown");
                let _ = window.emit("command-palette-triggered", ());
            } else {
                let _ = window.emit("global-shortcut-triggered", &tool);
            }
        }
    }
}).unwrap_or_else(|e| {
    eprintln!("注册快捷键 {} 失败: {}", shortcut_str, e);
});
```

- [ ] **Step 3: cargo check 验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过（`window.show()`/`set_focus()` 是 Tauri 2.0 WebviewWindow 方法，`Emitter` trait 已在 line 24 use）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(palette): main.rs 全局热键 __palette__ 分支 - 唤起窗口并 emit 事件"
```

---

## Task 3: 前端 — filterTools 纯函数 + 测试（TDD）

**Files:**
- Create: `src/utils/commandPalette.ts`
- Test: `src/utils/commandPalette.test.ts`

- [ ] **Step 1: 写测试文件（先于实现，应失败）**

创建 `src/utils/commandPalette.test.ts`，参考 `src/utils/diffUtils.test.ts` 的手写 assert 模式：

```ts
import { filterTools } from './commandPalette'

let passed = 0
let failed = 0

function assert(condition: boolean, message: string) {
  if (condition) {
    passed++
    console.log(`  \u2713 ${message}`)
  } else {
    failed++
    console.error(`  \u2717 ${message}`)
  }
}

// 测试数据（符合 ToolItem 形状，不依赖 @/store 的运行时 import）
const testTools = [
  { id: 'json', name: 'JSON工具', icon: '', iconSvg: '', description: 'JSON格式化压缩校验', keywords: ['json', '格式化', '压缩'] },
  { id: 'hash', name: '哈希计算', icon: '', iconSvg: '', description: 'MD5/SHA 哈希', keywords: ['hash', 'md5', 'sha', '哈希'] },
  { id: 'note', name: '文本编辑器', icon: '', iconSvg: '', description: '草稿本便签', keywords: ['笔记', 'notepad', '草稿'] },
  { id: 'home', name: '首页', icon: '', iconSvg: '', description: '搜索工具最近使用', keywords: ['首页', '搜索', '主页'] },
]

// Test 1: 空查询返回全部
console.log('Test: empty query returns all')
{
  const result = filterTools('', testTools)
  assert(result.length === testTools.length, '空查询返回全部工具')
}

// Test 2: json 命中且 JSON工具排首位
console.log('Test: json query ranks JSON tool first')
{
  const result = filterTools('json', testTools)
  assert(result.length > 0, 'json 有匹配结果')
  assert(result[0].tool.id === 'json', 'JSON工具排首位')
}

// Test 3: 无匹配返回空数组
console.log('Test: no match returns empty')
{
  const result = filterTools('zzz', testTools)
  assert(result.length === 0, 'zzz 无匹配返回空数组')
}

// Test 4: keywords 命中（md5 命中哈希计算）
console.log('Test: keywords match')
{
  const result = filterTools('md5', testTools)
  const hashIdx = result.findIndex(r => r.tool.id === 'hash')
  assert(hashIdx !== -1, 'md5 命中哈希计算工具')
}

// Test 5: name 前缀匹配排序高于 keywords 匹配
console.log('Test: name prefix outranks keywords match')
{
  // '哈希' 命中 hash 工具的 name 前缀（score 50）+ keywords（30）= 80
  const result = filterTools('哈希', testTools)
  assert(result.length > 0, '哈希 有匹配')
  assert(result[0].tool.id === 'hash', '哈希计算排首位')
}

// Test 6: 大小写不敏感
console.log('Test: case insensitive')
{
  const result = filterTools('JSON', testTools)
  assert(result[0].tool.id === 'json', '大写 JSON 也能命中')
}

console.log(`\n${passed} passed, ${failed} failed`)
process.exit(failed > 0 ? 1 : 0)
```

- [ ] **Step 2: 跑测试确认失败**

Run: `npx tsx src/utils/commandPalette.test.ts`
Expected: FAIL（`Cannot find module './commandPalette'`）

- [ ] **Step 3: 写 filterTools 实现**

创建 `src/utils/commandPalette.ts`。用 `import type` 仅引入类型（运行时移除，测试无 `@/store` 解析依赖）：

```ts
import type { ToolItem } from '@/store'

export interface RankedTool {
  tool: ToolItem
  score: number
}

/**
 * 模糊搜索工具：按 name/keywords/description/id 子串匹配，加权排序。
 * @param query 搜索词（空则返回全部，score=0）
 * @param tools 工具列表（由调用方传入，保持函数纯净无外部依赖）
 */
export function filterTools(query: string, tools: ToolItem[]): RankedTool[] {
  const q = query.trim().toLowerCase()
  if (!q) {
    return tools.map(tool => ({ tool, score: 0 }))
  }

  const matched: RankedTool[] = []
  for (const tool of tools) {
    const name = tool.name.toLowerCase()
    const id = tool.id.toLowerCase()
    const desc = tool.description.toLowerCase()
    const keywords = (tool.keywords || []).map(k => k.toLowerCase())

    let score = 0
    if (name === q) score += 100
    else if (name.startsWith(q)) score += 50
    else if (name.includes(q)) score += 5

    if (keywords.some(k => k.includes(q))) score += 30
    if (desc.includes(q)) score += 20
    if (id.includes(q)) score += 10

    if (score > 0) matched.push({ tool, score })
  }

  matched.sort((a, b) => b.score - a.score)
  return matched
}
```

- [ ] **Step 4: 跑测试确认通过**

Run: `npx tsx src/utils/commandPalette.test.ts`
Expected: PASS，输出 `6 passed, 0 failed`

- [ ] **Step 5: Commit**

```bash
git add src/utils/commandPalette.ts src/utils/commandPalette.test.ts
git commit -m "feat(palette): filterTools 纯函数 + 自检测试"
```

---

## Task 4: store — isCommandPaletteOpen 状态 + shortcuts 默认值

**Files:**
- Modify: `src/store/index.ts:110-116` (shortcuts 默认) + 状态新增

- [ ] **Step 1: config.shortcuts 默认值加 __palette__**

定位 `src/store/index.ts` 的 `shortcuts:` 配置（约 line 110-115），加入 `__palette__`：

```ts
shortcuts: {
  json: 'CmdOrCtrl+Alt+J',
  string: 'CmdOrCtrl+Alt+S',
  devtools: 'CmdOrCtrl+Alt+D',
  fileprocessing: 'CmdOrCtrl+Alt+F',
  __palette__: 'CmdOrCtrl+Alt+P',
},
```

- [ ] **Step 2: 新增面板状态与方法**

在 `src/store/index.ts` 的 `useToolboxStore` 内（`recentTools` 声明之后，约 line 119 后）加：

```ts
// ============ 命令面板 ============
const isCommandPaletteOpen = ref(false)
const openCommandPalette = () => { isCommandPaletteOpen.value = true }
const closeCommandPalette = () => { isCommandPaletteOpen.value = false }
```

然后在 store 的 return 对象中加入这三个（找到 `return { ... }` 语句，加入）：

```ts
isCommandPaletteOpen,
openCommandPalette,
closeCommandPalette,
```

- [ ] **Step 3: build 验证**

Run: `npm run build`
Expected: vue-tsc 类型检查通过，vite build 成功

- [ ] **Step 4: Commit**

```bash
git add src/store/index.ts
git commit -m "feat(palette): store 新增命令面板显隐状态 + shortcuts 默认热键"
```

---

## Task 5: CommandPalette.vue 浮层组件

**Files:**
- Create: `src/components/CommandPalette.vue`

- [ ] **Step 1: 创建组件**

创建 `src/components/CommandPalette.vue`：

```vue
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
```

> 注意：`type="text"` 写在 `class` 行之后另起一行仅为编辑器对齐，实际可写在一行。`v-html` 渲染 `iconSvg`（项目内联 SVG，可信来源）。

- [ ] **Step 2: build 验证**

Run: `npm run build`
Expected: vue-tsc 类型检查通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src/components/CommandPalette.vue
git commit -m "feat(palette): CommandPalette 浮层组件 - 搜索/键盘导航/分组显示"
```

---

## Task 6: App.vue — 挂载组件 + 监听事件 + 全局 Ctrl+P

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 模板挂载组件**

在 `src/App.vue` 的 `<template>` 内 `<div class="app-layout">` 闭合标签前加 `<CommandPalette />`：

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
    <CommandPalette />
  </div>
</template>
```

- [ ] **Step 2: script 加 import**

在 `src/App.vue` `<script setup>` 的 import 区（现有 `import HostsView from '@/views/HostsView.vue'` 之后）加：

```ts
import CommandPalette from '@/components/CommandPalette.vue'
```

- [ ] **Step 3: 新增 command-palette-triggered 事件监听**

定位 `src/App.vue` 的 `let unlistenShortcut` 声明（约 line 154），新增一个 unlisten 变量：

```ts
let unlistenShortcut: (() => void) | null = null
let unlistenPalette: (() => void) | null = null
let globalKeydownHandler: ((e: KeyboardEvent) => void) | null = null
```

在 `onMounted` 内现有 `unlistenShortcut = await listen(...)` 之后加：

```ts
unlistenPalette = await listen('command-palette-triggered', () => {
  store.openCommandPalette()
})

// 应用内 Ctrl+P toggle 命令面板（仅应用激活时生效）
globalKeydownHandler = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {
    e.preventDefault()
    if (store.isCommandPaletteOpen) {
      store.closeCommandPalette()
    } else {
      store.openCommandPalette()
    }
  }
}
window.addEventListener('keydown', globalKeydownHandler)
```

- [ ] **Step 4: onUnmounted 清理**

定位 `src/App.vue` 的 `onUnmounted`（约 line 202-206），扩展清理：

```ts
onUnmounted(() => {
  if (unlistenShortcut) {
    unlistenShortcut()
  }
  if (unlistenPalette) {
    unlistenPalette()
  }
  if (globalKeydownHandler) {
    window.removeEventListener('keydown', globalKeydownHandler)
  }
})
```

- [ ] **Step 5: build 验证**

Run: `npm run build`
Expected: vue-tsc 通过，vite build 成功

- [ ] **Step 6: Commit**

```bash
git add src/App.vue
git commit -m "feat(palette): App.vue 挂载面板 + 监听 command-palette-triggered + Ctrl+P toggle"
```

---

## Task 7: SidebarNav — 快捷键设置加命令面板配置项

**Files:**
- Modify: `src/components/SidebarNav.vue:201-215` (`SHORTCUT_TOOLS`)

- [ ] **Step 1: SHORTCUT_TOOLS 加命令面板项**

定位 `src/components/SidebarNav.vue` 的 `const SHORTCUT_TOOLS = [`（约 line 201-215），在末尾 `snippet` 之后加一项：

```ts
const SHORTCUT_TOOLS = [
  { id: 'json', label: 'JSON工具' },
  { id: 'string', label: '字符串工具' },
  { id: 'encode', label: '编码工具' },
  { id: 'regex', label: '正则测试' },
  { id: 'http', label: 'HTTP 请求' },
  { id: 'time', label: '时间工具' },
  { id: 'uuid', label: 'UUID生成' },
  { id: 'ocr', label: 'OCR识别' },
  { id: 'clipboard', label: '剪贴板' },
  { id: 'diff', label: '文本对比' },
  { id: 'color', label: '颜色工具' },
  { id: 'password', label: '密码工具' },
  { id: 'snippet', label: '代码片段' },
  { id: '__palette__', label: '命令面板' },
]
```

> 现有 `initShortcutList`（line 217-224）从 `store.config.shortcuts` 读默认值，`saveShortcuts`（line 265-280）保存到 shortcuts 表 + 提示重启 —— 命令面板热键自动复用，无需改动这两处。

- [ ] **Step 2: build 验证**

Run: `npm run build`
Expected: 通过

- [ ] **Step 3: Commit**

```bash
git add src/components/SidebarNav.vue
git commit -m "feat(palette): SidebarNav 快捷键设置加命令面板配置项"
```

---

## Task 8: 手动验收 + 版本号 + README + backlog

**Files:**
- Modify: `package.json` (version)
- Modify: `src-tauri/tauri.conf.json` (version，若有)
- Modify: `README.md`
- Modify: `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 启动 dev 环境手动验收**

Run: `npm run tauri dev`（首次需重启让 main.rs 热键注册生效）

按 spec 手动验收清单逐项验证：

1. 全局热键 `Ctrl+Alt+P` 呼出面板（窗口前台/最小化/后台三态）
2. 面板输入框自动聚焦
3. 输入 "json" → 实时过滤，JSON工具排首位
4. `↑` `↓` 跨分组移动选中项
5. `Enter` 跳转工具 + 面板关闭
6. `Esc` 关闭面板
7. 鼠标点击结果项跳转
8. 点击遮罩关闭
9. 应用内 `Ctrl+P` toggle 显隐
10. 侧边栏「快捷键设置」弹窗可见「命令面板」项，录制新热键，保存提示重启
11. 重启后新热键生效
12. 深色/浅色主题下面板样式正常

任一项失败：回到对应 Task 修复，不在本步 commit。

- [ ] **Step 2: 更新版本号**

`package.json` 的 `"version": "5.9.0"` 改为 `"6.0.0"`。

检查 `src-tauri/tauri.conf.json` 是否有 `version` 字段，若有同步改为 `6.0.0`。

- [ ] **Step 3: README 功能阶段记录**

在 `README.md` 的功能阶段记录表（参考 spec 已完成版本表格式）加一行：

```markdown
| V6.0 | ✅  | 快捷命令面板（全局热键 Ctrl+Alt+P 呼出，模糊搜索工具一键跳转） | 2026-07-22 |
```

- [ ] **Step 4: feature-backlog.md 更新**

`docs/superpowers/plans/feature-backlog.md`：
- 在「已完成版本」表加一行 V6.0
- D3 行标记为 `✅ 已完成 V6.0`（参考现有 A2/A8/A9 的标记格式）

- [ ] **Step 5: 最终 build 验证**

Run: `npm run build`
Expected: 通过

- [ ] **Step 6: Commit**

```bash
git add package.json src-tauri/tauri.conf.json README.md docs/superpowers/plans/feature-backlog.md
git commit -m "feat(palette): V6.0 快捷命令面板完成 - 版本号/README/backlog 同步"
```

---

## Self-Review

**Spec 覆盖检查：**
- ✅ 全局热键 Ctrl+Alt+P → Task 1 (db default) + Task 2 (main.rs 分支)
- ✅ 浮层 UI + 搜索 + 键盘导航 → Task 5
- ✅ filterTools 纯函数 → Task 3
- ✅ store 状态 → Task 4
- ✅ App.vue 监听 → Task 6
- ✅ SidebarNav 配置项 → Task 7
- ✅ 边界处理（无结果空状态/已开幂等/Esc/选中已有 Tab）→ Task 5 内置
- ✅ 测试策略 → Task 3（filterTools 自检）+ Task 8（手动验收）
- ✅ 版本号 + README + backlog → Task 8

**Placeholder 扫描：** 无 TBD/TODO，每步含完整代码或精确命令。

**类型一致性：** `RankedTool`、`FlatItem`、`filterTools(query, tools)`、`isCommandPaletteOpen`/`openCommandPalette`/`closeCommandPalette`、`__palette__` id、`command-palette-triggered` 事件名在各 Task 中一致。

**实现顺序合理性：** 后端 (1-2) → 纯函数+测试 (3) → store (4) → 组件 (5) → 接线 (6-7) → 验收+收尾 (8)。每步可独立 commit，无前向依赖阻塞（Task 6 依赖 4+5，Task 7 独立）。
