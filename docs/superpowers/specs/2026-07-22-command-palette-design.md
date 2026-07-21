# 快捷命令面板设计文档

## 目标

在 LitoBox 中新增快捷命令面板，通过全局热键 `Ctrl+Alt+P` 快速呼出浮动搜索浮层，模糊搜索并跳转到任意工具，替代在侧边栏多层菜单中查找工具的低效操作。对标 VS Code `Cmd+Shift+P` / Raycast 的快速切换体验。

## 功能范围

### 包含
- 全局热键 `Ctrl+Alt+P` 呼出命令面板（窗口在后台时自动唤起到前台）
- 浮动浮层 UI：输入框 + 实时搜索结果列表（按 category 分组）
- 模糊搜索工具：匹配 `name` / `keywords` / `description` / `id`，带排序权重
- 键盘导航（`↑` `↓` `Enter` `Esc`）+ 鼠标点击
- 选中工具 → 切换/打开 Tab（复用 `store.openTab`）+ 记录最近使用
- 应用内 `Ctrl+P` toggle 显隐（应用激活时可用，纯前端 keydown 监听，不重复注册全局热键；与全局 `Ctrl+Alt+P` 互补——前台用 `Ctrl+P` 更顺手）
- 命令面板热键可在现有「快捷键设置」弹窗中自定义

### 不包含
- 动作命令（切换主题、清空历史等）—— YAGNI，纯工具搜索已覆盖核心价值
- 历史/收藏整合到面板
- Fuzzy 匹配算法（子串 `includes` 足够 50 条数据）
- 面板内最近使用排序
- 热键动态重注册（改键需重启，与现有 4 个工具热键同样的局限）
- 虚拟滚动（50 条数据不需要）

## 整体架构

### 触发链路（全局热键，窗口可能在后台）

```
用户按 Ctrl+Alt+P
  → main.rs on_shortcut 回调命中 tool_id="__palette__"
  → window.show() + window.set_focus()      // 新增：唤起到前台（幂等）
  → emit("command-palette-triggered")         // 新增独立事件
  → App.vue 监听 → store.openCommandPalette()
  → CommandPalette.vue 显示，输入框自动聚焦
```

### 搜索 + 选中链路（纯前端）

```
用户输入 "json"
  → CommandPalette 从 TOOL_LIST 过滤（name/keywords/description/id 子串匹配）
  → 实时显示匹配结果（category 分组 + icon + description）
  → ↑↓ 选择 + Enter / 鼠标点击
  → store.openTab(toolId) + store.addRecentTool(toolId)
  → closeCommandPalette()，清空输入
```

### 组件清单

| 文件 | 动作 | 职责 |
|------|------|------|
| `src/components/CommandPalette.vue` | **新增** | 浮层 UI + 搜索过滤 + 键盘导航 |
| `src/utils/commandPalette.ts` | **新增** | 纯函数 `filterTools()` 搜索排序逻辑（可独立测试） |
| `src/App.vue` | 改动 | 挂载面板 + 监听 `command-palette-triggered` 事件 |
| `src/store/index.ts` | 改动 | 加 `isCommandPaletteOpen` 状态 + `openCommandPalette`/`closeCommandPalette` |
| `src-tauri/src/main.rs` | 改动 | 触发回调加 `__palette__` 分支：show+focus+emit 特殊事件 |
| `src-tauri/src/db.rs` | 改动 | `db_read_shortcuts` 默认值加 `__palette__` → `CmdOrCtrl+Alt+P` |
| `src/components/SidebarNav.vue` | 改动 | `SHORTCUT_TOOLS` 加 `{ id: '__palette__', label: '命令面板' }` |

## 前端组件设计

### CommandPalette.vue

通过 `<Teleport to="body">` 挂载，脱离 App.vue 层级。

**结构：**

```
.palette-overlay          // 半透明遮罩，点击关闭
  .palette-container      // 顶部居中浮层（Spotlight 风格）
    .palette-input        // 自动聚焦输入框，placeholder "搜索工具…"
    .palette-results      // 结果列表，按 category 分组
      .palette-group      // 分组标题 + 项
        .palette-item     // icon + name + description
    .palette-empty        // 无匹配时空状态
    .palette-hint         // 底部快捷键提示（↑↓ 选择 · Enter 跳转 · Esc 关闭）
```

**搜索算法**（抽成纯函数 `filterTools(query, toolList): RankedTool[]`，放 `src/utils/commandPalette.ts`）：

- 数据源：`TOOL_LIST`（约 50 条，纯前端遍历，无防抖 —— 数据量小不需要）
- 匹配字段：`name` + `keywords[]` + `description` + `id`，全部转小写后 `includes()` 子串匹配
- 排序权重（高 → 低）：
  1. name 精确匹配（全等）
  2. name 前缀匹配
  3. keywords 命中
  4. description 命中
  5. id 命中
  6. 其他子串命中
- 空查询：返回全部工具，按 `category` 分组展示

**键盘交互：**

- `↑` / `↓`：跨分组连续移动 `selectedIndex`
- `Enter`：选中并跳转
- `Esc`：关闭面板
- `Ctrl+P`（应用内）：toggle 显隐（开 → 关，关 → 开；纯前端 keydown 监听，仅应用激活时生效，不重复注册全局热键）

**鼠标交互：**

- 点击结果项：跳转
- 点击遮罩：关闭
- hover：高亮并同步 `selectedIndex`

**选中后行为：**

```
store.openTab(toolId)
store.addRecentTool(toolId)
closeCommandPalette()   // 同时清空输入框
```

### 样式规范

全部使用 [theme.css](file:///d:/work/codes/litobox/src/style/theme.css) CSS 变量，禁止硬编码颜色（遵循避坑 #21）：

| 元素 | 样式 |
|------|------|
| `.palette-overlay` | `rgba(0,0,0,0.45)` 蒙层，`z-index` 高于 App 内容 |
| `.palette-container` | `var(--bg-card)` 背景、`var(--border-color)` 边框、圆角 8px、阴影、顶部居中 |
| `.palette-input` | `var(--bg-input)` 背景、`var(--text-primary)` 文字、聚焦边框 `var(--accent-cyan)` |
| `.palette-item` 选中/hover | `var(--bg-secondary)` 背景 |
| `.palette-group` 标题 | 青色 `var(--accent-cyan)`、小写字母间距 |
| `.palette-hint` | `var(--text-secondary)`、小字号 |

scoped 样式只定义页面特有类名（`.palette-*`），不复用全局类名（遵循避坑 #21）。深色/浅色双主题由 CSS 变量自动适配。

### store 改动（src/store/index.ts）

新增状态与方法：

```ts
const isCommandPaletteOpen = ref(false)
const openCommandPalette = () => { isCommandPaletteOpen.value = true }
const closeCommandPalette = () => { isCommandPaletteOpen.value = false }
```

`config.shortcuts` 初始化时若 localStorage 无 `__palette__`，填默认 `CmdOrCtrl+Alt+P`。

### App.vue 改动

- 在模板 `<div class="app-layout">` 内挂载 `<CommandPalette />`
- `onMounted` 新增监听 `command-palette-triggered`：

```ts
unlistenPalette = await listen('command-palette-triggered', () => {
  store.openCommandPalette()
})
```

- `onUnmounted` 清理 unlistenPalette（与现有 unlistenShortcut 同模式）
- 现有 `global-shortcut-triggered` 监听保持不变（仍走 `openTab`，不会误触面板）

## 后端改动

### main.rs（src-tauri/src/main.rs:246-254）

当前回调统一 emit `global-shortcut-triggered`。改动：加 `__palette__` 分支，仅对命令面板热键做 `show + set_focus + emit` 特殊事件。

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
})
```

- `show()` + `set_focus()` **仅加在 `__palette__` 分支**，不改动现有 4 个工具热键行为（最小 diff，避免意外影响）
- 失败用 `let _ =` 忽略，与现有 emit 风格一致
- 加 `debug_log!` 标记命中分支（遵循后端 debug 日志规范）

### db.rs（src-tauri/src/db.rs:1798-1804）

`db_read_shortcuts` 的 `default` 列表加一项，确保开箱即用：

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

## 配置与持久化

### 存储

复用 `config.shortcuts` 表，`__palette__` 作为 key。默认值由 `db_read_shortcuts` 注入（后端），前端 store 初始化时若 localStorage 缺失也补默认。

### 配置 UI

`SidebarNav.vue` 的 `SHORTCUT_TOOLS` 数组加一项：

```ts
{ id: '__palette__', label: '命令面板' },
```

复用现有热键录制（[SidebarNav.vue:226-255](file:///d:/work/codes/litobox/src/components/SidebarNav.vue#L226-L255)）+ `saveShortcuts()` 保存流程（写 shortcuts 表 + 提示"重启应用后生效"）。

> ponytail: 热键改后需重启才生效（main.rs 启动时一次性注册，不动态重注册）。这是现有 4 个工具热键同样的局限，本次不解决。升级路径：未来用 `manager.unregister` + `register` 实现动态切换，改动集中在 main.rs 一个注册函数内。

## 边界与错误处理

| 场景 | 处理 |
|------|------|
| 搜索无结果 | 显示空状态「未找到匹配工具」 |
| 面板已开再按热键 | `show()+set_focus()` 幂等，面板保持打开、窗口置顶（不做 toggle 关闭，简单） |
| `Esc` 关闭 | 关闭面板，焦点回主内容区 |
| 热键注册冲突 | 现有 `unwrap_or_else` 打 stderr，用户在配置页改键 + HotkeyView 排查（不主动检测） |
| 选中工具已在其他 Tab 打开 | `store.openTab` 内部激活已有 Tab，不重复创建 |
| 大列表性能 | 50 条数据，纯前端遍历，无需虚拟滚动 |
| 窗口最小化时按热键 | `show()` 恢复窗口，`set_focus()` 抢焦点 |
| listen 注册失败 | App.vue 现有 unlisten 模式，`await listen` 失败不崩溃（与现有 unlistenShortcut 同模式） |

## 测试策略

遵循 AGENTS.md「非平凡逻辑留一个可运行检查，no frameworks, no fixtures」。

### 前端搜索逻辑（重点）

`filterTools()` 是唯一非平凡逻辑，抽成纯函数后写独立 assert 自检脚本：

- 文件：`src/utils/commandPalette.test.ts`（或 `.test.js`，`node`/`tsx` 可跑）
- 断言：
  1. 空查询返回全部工具（数量 = TOOL_LIST.length）
  2. 查询 `"json"` 命中「JSON工具」，且 name 精确匹配排在最前
  3. 查询 `"zzz"` 无匹配返回空数组
  4. keywords 命中（如查 `"md5"` 命中「哈希计算」）排序正确

### 后端数据校验

`db.rs` 现有 `tests` 模块加一个断言：`db_read_shortcuts` 在 config 为空时返回的默认值含 `__palette__` → `CmdOrCtrl+Alt+P` 条目。

### 不测试的部分

- main.rs 的 `__palette__` 分支：配置驱动，难单测，靠手动验证
- UI 交互：靠手动验证（热键呼出、搜索、键盘导航、选中跳转）

## 手动验收清单

1. 全局热键 `Ctrl+Alt+P` 呼出面板（窗口在前台/最小化/后台三种状态）
2. 面板输入框自动聚焦
3. 输入 "json" → 实时过滤，JSON工具排首位
4. `↑` `↓` 跨分组移动选中项
5. `Enter` 跳转工具 + 面板关闭
6. `Esc` 关闭面板
7. 鼠标点击结果项跳转
8. 点击遮罩关闭
9. 应用内 `Ctrl+P` toggle 关闭
10. 侧边栏「快捷键设置」弹窗可见「命令面板」项，可录制新热键，保存提示重启
11. 重启后新热键生效
12. 深色/浅色主题下面板样式均正常

## 版本号

命令面板不属于"新增侧边栏工具菜单项"（不进 `TOOL_LIST`），而是平台级全局效率功能，量级与影响力不亚于一个新工具。建议版本号从 V5.9 → **V6.0**（minor bump，体现效率工具新阶段）。按 AGENTS.md 规范同步更新 README 功能阶段记录。

> 版本号最终以用户在实现收尾阶段确认为准（如倾向保守可议 V5.10）。

## 实现顺序建议

1. 后端：`db.rs` 默认值 + `main.rs` `__palette__` 分支（cargo check）
2. store：`isCommandPaletteOpen` 状态 + 方法 + shortcuts 默认值
3. 纯函数：`src/utils/commandPalette.ts` `filterTools()` + 测试脚本
4. 组件：`CommandPalette.vue`（UI + 搜索 + 键盘导航）
5. App.vue：挂载组件 + 监听事件
6. SidebarNav：`SHORTCUT_TOOLS` 加配置项
7. `npm run build` 验证 + 手动验收清单
8. 更新 README + feature-backlog.md（移到已完成版本表）
