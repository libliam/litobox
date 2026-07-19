# Ponytail, lazy senior dev mode

You are a lazy senior developer. Lazy means efficient, not careless. The best code is the code never written.

Before writing any code, stop at the first rung that holds:

1. Does this need to be built at all? (YAGNI)
2. Does it already exist in this codebase? Reuse the helper, util, or pattern that's already here, don't re-write it.
3. Does the standard library already do this? Use it.
4. Does a native platform feature cover it? Use it.
5. Does an already-installed dependency solve it? Use it.
6. Can this be one line? Make it one line.
7. Only then: write the minimum code that works.

The ladder runs after you understand the problem, not instead of it: read the task and the code it touches, trace the real flow end to end, then climb.

Bug fix = root cause, not symptom: a report names a symptom. Grep every caller of the function you touch and fix the shared function once — one guard there is a smaller diff than one per caller, and patching only the path the ticket names leaves a sibling caller still broken.

Rules:

- No abstractions that weren't explicitly requested.
- No new dependency if it can be avoided.
- No boilerplate nobody asked for.
- Deletion over addition. Boring over clever. Fewest files possible.
- Shortest working diff wins, but only once you understand the problem. The smallest change in the wrong place isn't lazy, it's a second bug.
- Question complex requests: "Do you actually need X, or does Y cover it?"
- Pick the edge-case-correct option when two stdlib approaches are the same size, lazy means less code, not the flimsier algorithm.
- Mark intentional simplifications with a `ponytail:` comment. If the shortcut has a known ceiling (global lock, O(n²) scan, naive heuristic), the comment names the ceiling and the upgrade path.
- **版本号更新必须同步更新 README**：每次更新版本号意味着新增了功能，必须在 README.md 的功能阶段记录中添加新功能条目。
- **经验总结注重通用性**：当需要总结经验、提炼解决方法时，要考虑通用性和可复用性，避免只针对特定场景。
- **开发新功能时主动沟通确认**：在实现过程中遇到模糊需求、多种可行方案、或需要做取舍决策时，主动向用户提问确认，而不是自行猜测。宁可多问一句，也不要做出不符合用户预期的实现。沟通内容包括但不限于：功能边界确认、UI交互细节、参数默认值、异常场景处理策略。目标是确保最终交付的功能真正满足用户需求，减少返工。

Not lazy about: understanding the problem (read it fully and trace the real flow before picking a rung, a small diff you don't understand is just laziness dressed up as efficiency), input validation at trust boundaries, error handling that prevents data loss, security, accessibility, the calibration real hardware needs (the platform is never the spec ideal, a clock drifts, a sensor reads off), anything explicitly requested. Lazy code without its check is unfinished: non-trivial logic leaves ONE runnable check behind, the smallest thing that fails if the logic breaks (an assert-based demo/self-check or one small test file; no frameworks, no fixtures). Trivial one-liners need no test.

---

# AGENTS.md - LitoBox开发指南

## 项目概述

**项目名称**: 栗的百宝箱 (LitoBox)

**产品定位**: 轻量化、无网络、无广告、常驻本地的Windows桌面集成工具箱，统一收纳高频开发小工具。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 (Composition API) + TypeScript + Vite |
| UI | Element Plus |
| 桌面 | Tauri 2.0 (Rust) |
| 存储 | SQLite (rusqlite) + localStorage |
| 工具库 | json5, prettier, lodash, js-base64 |

## 项目结构

```
litobox/
├── src/                    # 前端Vue源码
│   ├── components/         # 公共组件
│   ├── views/              # 功能页面
│   ├── utils/              # 核心工具方法
│   ├── store/              # Pinia状态管理
│   ├── style/              # 样式文件
│   ├── App.vue
│   └── main.ts
├── src-tauri/              # Tauri Rust底层
│   ├── src/
│   │   ├── main.rs
│   │   ├── db.rs           # SQLite数据库操作
│   │   ├── tray.rs
│   │   ├── hotkey.rs
│   │   └── window.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
└── vite.config.ts
```

## 开发规范

### 代码风格
- TypeScript严格模式
- Vue 3 Composition API + `<script setup>`
- 组件命名 PascalCase，工具函数 camelCase，常量 UPPER_SNAKE_CASE

### 组件开发规范
- 单一职责，Props类型明确定义，使用Emits声明事件
- 避免直接操作DOM

### UI设计规范（科技风）
- 卡片式布局（`.tool-card`），包含标题栏和内容区
- 卡片标题青色、大写、字母间距1px
- 操作按钮按功能分组（`action-grid` + `action-group`）
- 输入/输出区域包含标题栏和操作按钮（清空/粘贴/复制）
- 错误提示红色边框+发光效果
- **禁止硬编码颜色**，所有颜色使用 `theme.css` 中定义的CSS变量
- 新增页面基于 `src/views/_ToolTemplate.vue` 模板创建

### 多Tab页面规范
- 每个Tab独立维护 `input/output/error`，切换不互相覆盖
- 粘贴后自动执行（300ms防抖）
- 使用 `watch` 监听当前Tab的input，避免 `watchEffect` 和 `deep watch`
- Tab栏使用 `position: sticky` 置顶

### 工具函数规范
- 纯函数设计，无副作用
- 明确输入输出类型
- 错误处理使用try-catch
- 大文本处理注意性能

### 状态管理规范
- Pinia状态管理
- 配置数据持久化到localStorage
- 操作历史最多保留10条

### 通用注意事项
- **禁止添加网络请求相关代码**
- **禁止引入广告/推荐内容**
- 大文本处理使用虚拟滚动
- 页面销毁时清理监听器和缓存
- 打包时不生成安装包 — `bundle.targets` 保持为空数组
- 耗时操作必须显示加载提示 — 使用 `ElLoading.service()` + `finally` 确保关闭
- **历史记录必须传 `inputFull` / `outputFull`** — `store.addHistory()` 调用时必须同时传入完整输入输出（`inputFull: 完整输入, outputFull: 完整输出`），否则操作历史页面的双击跳转功能无法还原数据。`inputPreview`/`outputPreview` 仅用于列表展示（截断50字符），`inputFull`/`outputFull` 用于详情还原

## 后端开发指南

### SQLite 数据库
- 数据库路径：`%APPDATA%\com.dev.toolbox\litobox.db`
- 存储内容：工作流、变量池、操作历史、代码片段、配置
- 所有数据库操作通过 `src-tauri/src/db.rs` 暴露，使用 `with_conn()` 确保线程安全

### Tauri 命令规范
- 命令参数使用 camelCase，前端传参必须匹配
- Rust 函数参数为 snake_case（如 `note_type`）时，必须在 `#[tauri::command]` 上添加 `rename_all = "snake_case"`
- boa_engine API 使用注意 `unsafe` 块和引用链式调用
- 多线程日志使用 `Arc<Mutex<Vec<LogEntry>>>` 安全共享
- **后端新增功能代码必须添加 debug 日志**：在关键逻辑分支、错误处理路径和性能敏感点添加 `debug_log!()` 日志，用于后续调试和问题定位，release 模式下自动移除。

### 子进程沙箱避坑（PowerShell / reg）
Tauri 2.x 子进程（`Command::new("powershell")` / `Command::new("reg")`）受沙箱限制：

**核心原则**：
- 子进程中不要依赖 `$_`、脚本块 `{ }` 等 PowerShell 高级特性
- 过滤逻辑优先在服务端/数据源侧完成（如 WMI 的 `-Filter`），不要在 PowerShell 管道里过滤
- 所有子进程输出用 `encoding_rs::GBK.decode()` 解码，中文 Windows 默认是 GBK 编码
- 所有控制台子进程加 `CREATE_NO_WINDOW` 标志，避免弹黑框/蓝框

### 避坑指南
1. **SQLite NULL 比较必须用 IS 而非 =**：`WHERE parent_id = ?` 在参数为 NULL 时永远返回空，必须用 `WHERE parent_id IS ?` 或拆分为 `IS NULL` / `= ?` 两种情况
2. **with_conn 内禁止嵌套调用其他 with_conn 函数**：会导致死锁（应用卡死），应在当前连接上直接执行 SQL
3. **do_note_create 必须实际写盘**：仅生成路径并存入数据库不够，必须调用 `std::fs::File::create(&path)` 创建空文件
4. **Rust 后端修改后必须重启 Tauri 开发服务器**：`cargo check` 通过不代表热更新生效，必须 `Ctrl+C` 停止后重新 `npm run tauri dev`
5. **控制台子进程必须加 `CREATE_NO_WINDOW`**：Tauri 是 GUI 应用，调用 `powershell` / `reg` / `ipconfig` 等控制台程序时会弹出终端窗口，加 `cmd.creation_flags(CREATE_NO_WINDOW)`（值 `0x08000000`）隐藏
6. **Windows 子进程输出编码是 GBK 不是 UTF-8**：中文 Windows 控制台输出为 GBK/CP936 编码，`String::from_utf8()` 遇到中文会报错。统一用 `encoding_rs::GBK.decode()` 解码所有子进程输出
7. **外部系统查询优先用原生过滤参数**：WMI 用 `-Filter`（WQL）、数据库用 `WHERE`，不要在管道里用 `Where-Object` 等后过滤，既慢又可能受环境限制
8. **同一问题超过 2 次未解决，立即加日志定位**：不要靠猜调 bug。`unwrap_or_default()` 会吞掉错误，改用 `match` 分支输出错误日志。debug 模式用 `debug_log!()`，release 模式自动移除
9. **依赖外部系统的数据查询必须有降级/回退**：WMI、注册表、网络接口等在不同环境下表现可能不同，主要路径失败时应有备选方案，确保至少展示部分数据而不是全空
10. **前端监听后端事件必须配合轮询兜底**：不能只依赖事件，因为：(a) 事件可能在监听器注册前就发出（快速完成的任务）；(b) 事件可能被 ID 过滤掉（陈旧事件防护）；(c) 事件可能丢失。模式：`listen(event)` + 定时 `invoke('status')` 兜底查询，两者都到达时用 `done` flag 去重。典型场景：长时间运行的后台任务（搜索、扫描、批处理）
11. **用户交互响应必须用时间驱动而非计数驱动**：取消检查、进度上报这类需要及时响应的逻辑，不要用"每 N 个文件检查一次"，因为大目录下单次遍历可能阻塞数秒。改用 `Instant::now()` 记录上次检查时间，超过固定间隔（如 200ms）就检查一次，保证响应延迟有上限
12. **KeepAlive 缓存的组件用 watch 替代 onMounted 处理跨页跳转**：Vue 的 KeepAlive 缓存组件再次激活时 `onMounted` 不会触发，导致从其他页面跳转回来时初始化逻辑（如读取 store 中的待还原数据）被跳过。模式：用 `watch(() => store.someState, (val) => { if (val) handle(val) })` 替代 `onMounted` 里的一次性检查，或用 `onActivated`
13. **Windows 路径展示前必须去掉 `\\?\` 前缀**：Rust 的 `Path::canonicalize()` 在 Windows 上会自动加上 `\\?\` 长路径前缀（支持 32767 字符路径），展示给用户时不友好。所有 canonicalize 后的路径在存入状态/返回前端前，统一用 `strip_prefix(r"\\?\")` 去掉
14. **二进制文件检测优先用扩展名而非内容嗅探**：仅靠 BOM 和 `\0` 字节检测会误判 PDF（开头是 `%PDF-1.x` 纯文本）、ZIP（开头是 `PK`）等格式。模式：扩展名匹配常见二进制格式（pdf/zip/jpg/mp3/avi 等）优先返回 true → BOM 检测 → `\0` 字节检测，三级判断
15. **Rust 图片处理命令必须用 async + spawn_blocking**：`image` crate 的解码/合成操作会阻塞主线程导致 UI 卡死（未响应）。所有图片处理命令必须用 `async fn` + `tauri::async_runtime::spawn_blocking(move || { do_xxx(...) })` 在后台线程执行。禁止同步执行大图操作。
16. **Rust 结构体字段名不会自动转 camelCase**：Tauri 不会将 Rust 的 snake_case 字段名自动转为 JS 的 camelCase。前端接口定义和模板必须使用与 Rust 结构体完全一致的 snake_case 命名（如 `original_size` 而非 `originalSize`）。**但命令函数参数相反**：Rust 侧用 snake_case（如 `time_point`），前端 invoke 时必须传 camelCase（如 `timePoint`）。两者规则不同，注意区分。
17. **获取文件信息用专用轻量命令**：不要调用处理命令（如 `image_compress`）来获取文件大小。应新增专用的 `get_file_info` 命令，只用 `metadata().len()` 获取大小，避免解码图片导致卡顿。
18. **耗时操作不要用 ElLoading 全屏锁**：`ElLoading.service({ lock: true })` 会锁住整个 UI 导致无法切换工具。改用按钮自身的 `loading` 属性（`:loading="isProcessing"`），不影响其他功能切换。
19. **文件预览用 read_file_base64 后端命令**：Tauri asset scope 限制可能导致 `URL.createObjectURL()` 无法展示本地文件。应通过后端命令读取文件内容并返回 base64 字符串，前端用 `data:image/xxx;base64,...` 展示。
20. **多 Tab 页面布局规范**：Tab 栏放在独立的 `.tool-card.sticky-card` 中，使用自定义 class（如 `pdf-tabs`/`image-tabs`），在 scoped 样式中定义完整 Tab 样式。各 Tab 内容用 `v-if="activeTab === 'xxx'"` 的 `.tool-card` 独立渲染，不要放在 `el-tab-pane` 内。参考 `_ToolTemplate.vue`、PdfTool.vue。
21. **scoped 样式中禁止重复定义全局类名**：在 `<style scoped>` 中重复定义 `.tool-card`/`.card-header`/`.card-body` 等全局类名会导致样式冲突（padding 被覆盖等）。只定义页面特有样式，全局样式由 `theme.css` 提供。如需强制覆盖，用非 scoped `<style>` 块 + `!important`。
22. **侧边栏菜单顺序由 `TOOL_LIST` 数组顺序决定**：`SidebarNav.vue` 按 `category` 分组，同组内按 `TOOL_LIST` 中的先后顺序排列。调整菜单顺序 = 在 `src/store/index.ts` 的 `TOOL_LIST` 中移动对应条目的位置（同 category 内调整），不是改 `SidebarNav.vue` 的渲染逻辑。
23. **ffmpeg 实时进度必须用 `-progress pipe:1`**：ffmpeg 默认把进度输出到 stderr，且用 `\r`（回车）刷新同一行。Rust 的 `BufReader::lines()` 按 `\n` 分割，会导致所有进度更新堆积成一行直到进程退出，进度条从 0% 直接跳到 100%。正确做法：加 `-progress pipe:1 -nostats` 参数，让 ffmpeg 把结构化进度输出到 stdout（每行 `\n` 分隔），解析 `out_time_us=` 字段计算百分比。同时注意：同时捕获 stdout 和 stderr 时，必须用独立线程读取其中一个流，避免管道缓冲区满导致 ffmpeg 阻塞死锁。

## 工作流与变量池集成

### 新增工具页面必须考虑
1. **工作流集成** — 工具功能必须能被工作流调用，在 `WorkflowView.vue` 的 `executeStep()` 中添加对应分支
2. **变量池集成** — 输入区必须添加 `VariablePicker` 组件
3. **历史记录** — 操作必须记录到 SQLite

### 工作流执行规范
- **禁止重复实现** — 必须复用现有 `utils/` 函数，不得重新编写处理逻辑
- 输入源支持：执行输入、上一步输出、手动输入、变量池
- 执行完成后可选择将结果保存到变量池

## 安全要求
- 纯本地离线运行，无网络请求
- 仅保留必要权限：剪贴板、窗口控制、全局热键、本地存储
- 所有数据本地存储，不上传
