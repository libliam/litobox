# JS 工具箱设计文档

## 概述

在 LitoBox 中新增 **JS 工具箱** 模块，提供 JavaScript 代码的沙箱执行、格式化、压缩和 JSON 代码生成能力。所有功能纯本地离线运行，无网络请求。

## 架构

### 整体结构

```
JS 工具箱 (js)
├── Tab 1: 沙箱运行器    → 前端页面 + Rust 后端 (boa_engine)
├── Tab 2: 代码格式化    → 纯前端
├── Tab 3: 代码压缩      → 纯前端
└── Tab 4: JSON→代码    → 纯前端
```

### 沙箱执行架构

```
Vue 前端 (JSTool.vue)
  │
  │  Tauri invoke('execute_js', { code, input, timeout })
  ▼
Rust 后端 (src/js_executor.rs)
  │  boa_engine 执行 JS
  │  捕获 console.log/warn/error
  │  超时终止 (5s)
  ▼
返回 { result, logs[], error, line? }
```

## 组件设计

### 1. 前端页面 — `src/views/JSTool.vue`

遵循现有 SQL 工具的多 Tab 模式，每个 Tab 独立 `tabState`。

#### Tab 1: 沙箱运行器

**布局：**
- **操作栏**（sticky）：执行按钮、清空按钮、超时设置（默认 5s）、入参 JSON 输入框
- **代码编辑器**：多行 textarea，等宽字体，支持粘贴自动执行
- **输出结果**：展示 `return` 值（JSON 序列化），带复制按钮
- **日志面板**：展示 `console.log/warn/error` 输出，带颜色标签区分级别

**交互：**
- 粘贴/输入代码后 300ms 防抖自动执行
- 入参 JSON 作为全局变量 `input` 注入沙箱
- 执行超时显示错误提示
- 语法错误/运行时错误高亮显示，标注行号

#### Tab 2: 代码格式化

**配置项：**
- 缩进：2 空格 / 4 空格
- 分号：保留 / 移除
- 引号：单引号 / 双引号
- 换行宽度：80 / 100 / 120

**实现：** 纯前端字符串处理，基于 token 化 + 规则重写（类似 sqlFormatter 模式）

#### Tab 3: 代码压缩

**配置项：**
- 变量名混淆：开启 / 关闭
- 保留注释：开启 / 关闭
- ES 版本：ES5 / ES6

**实现：** 纯前端，去除空白/换行/注释，可选变量名缩短

#### Tab 4: JSON→代码

**三种模式：**
1. **解构代码** — 根据 JSON 生成 `const { a, b } = data` 解构语句
2. **TS 类型声明** — 根据 JSON 生成 `interface` 类型定义
3. **默认值模板** — 根据 JSON 生成带默认值的对象模板

**实现：** 纯前端，递归遍历 JSON 结构生成代码

### 2. Rust 后端 — `src-tauri/src/js_executor.rs`

**依赖：** `boa_engine` crate（纯 Rust JS 引擎）

**Tauri 命令：**
```rust
#[tauri::command]
fn execute_js(code: String, input: String, timeout_ms: u64) -> ExecuteResult
```

**功能：**
- 创建 boa 引擎实例
- 注入 `input` 全局变量（解析 JSON 字符串）
- 注入 `console.log/warn/error` 方法（捕获到日志数组）
- 执行代码，捕获 `return` 值
- 超时控制：使用 `std::sync::mpsc` 通道 + `thread::spawn`，超时则终止
- 返回结构化结果：`{ result, logs: [{level, message}], error, line }`

### 3. 工具函数 — `src/utils/jsUtils.ts`

纯前端工具函数：
- `formatJs(code, options)` — JS 代码格式化
- `compressJs(code, options)` — JS 代码压缩
- `jsonToDestruct(json)` — JSON 生成解构代码
- `jsonToInterface(json)` — JSON 生成 TS 类型
- `jsonToDefaultTemplate(json)` — JSON 生成默认值模板

### 4. Store 集成 — `src/store/index.ts`

- `TOOL_LIST` 新增 JS 工具箱条目
- `shortcuts` 新增 `js: 'CmdOrCtrl+Alt+J'`（需调整现有冲突）

### 5. 路由集成 — `src/App.vue`

- 导入 `JSTool` 组件
- 添加 `v-else-if="activeTool === 'js'"` 分支

## 数据流

### 沙箱执行流程

```
用户输入代码 + 入参 JSON
  → 前端校验 JSON 格式
  → Tauri invoke('execute_js', { code, input: jsonStr, timeout_ms: 5000 })
  → Rust boa_engine 执行
  → 返回结果
  → 前端展示：输出结果 + 日志面板 + 错误信息
  → 记录历史
```

### 纯前端功能流程

```
用户输入代码
  → 300ms 防抖
  → 调用对应工具函数
  → 展示结果
  → 记录历史
```

## 错误处理

| 错误类型 | 处理方式 |
|---------|---------|
| JSON 入参格式错误 | 前端校验，红色边框提示 |
| JS 语法错误 | Rust 捕获，返回错误信息和行号，前端高亮 |
| JS 运行时错误 | Rust 捕获，返回错误信息和行号 |
| 执行超时 | Rust 终止线程，返回超时错误 |
| 死循环检测 | 超时机制自动终止 |

## 安全

- boa_engine 沙箱隔离，无法访问 Tauri API、浏览器 DOM、文件系统
- 仅暴露 `input` 和 `console` 全局变量
- 纯本地执行，无网络请求
- 执行超时 5 秒强制终止

## 文件清单

| 文件 | 说明 |
|------|------|
| `src/views/JSTool.vue` | JS 工具箱主页面 |
| `src/utils/jsUtils.ts` | 纯前端工具函数 |
| `src-tauri/src/js_executor.rs` | Rust JS 执行引擎 |
| `src-tauri/Cargo.toml` | 新增 boa_engine 依赖 |
| `src-tauri/src/main.rs` | 注册 Tauri 命令 |
| `src/store/index.ts` | 新增工具注册 |
| `src/App.vue` | 路由集成 |
