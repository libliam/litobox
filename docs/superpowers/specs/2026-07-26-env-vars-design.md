# 环境变量编辑器 — 设计文档

**日期**: 2026-07-26
**版本**: V6.4.0
**状态**: 设计中

---

## 1. 功能概述

新增独立的环境变量管理工具，支持查看、新增、修改、删除 Windows 用户级和系统级环境变量。PATH 变量提供逐行列表编辑器，方便管理目录路径。

## 2. 技术方案

### 2.1 后端（Rust）

新增 `src-tauri/src/env_vars.rs`，参照 `startup_items.rs` 模式：

- `run_powershell()` 封装 PowerShell 执行
- `debug_log!()` 宏用于调试日志
- 读写注册表路径 `HKCU\Environment` 和 `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`
- 修改后广播 `WM_SETTINGCHANGE` 通知系统刷新

#### 命令列表

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_env_vars` | 无 | `{ user: EnvVar[], system: EnvVar[] }` | 读取两处注册表 |
| `set_env_var` | `name, value, scope` | `{ success, message }` | 写入/更新变量 |
| `delete_env_var` | `name, scope` | `{ success, message }` | 删除变量 |

#### 数据结构

```rust
struct EnvVar {
    name: String,
    value: String,
}

struct EnvVarResult {
    success: bool,
    message: String,
}
```

### 2.2 前端（Vue）

新增 `src/views/EnvVarsView.vue`，参照 `StartupItemsView.vue` 的 UI 模式。

#### 页面结构

```
┌─ 管理员权限提示（仅系统变量 Tab）──────────┐
├─ 统计概览（用户变量 N 个 / 系统变量 M 个）──┤
├─ 筛选栏（搜索名称 + 刷新按钮）──────────────┤
├─ Tab: 用户变量 | 系统变量 ──────────────────┤
│  ├─ PATH 变量 → 特殊行，展开逐行列表编辑器  │
│  └─ 普通变量 → 表格行内编辑（双击编辑）     │
├─ 底部栏（导出 CSV）─────────────────────────┤
└─ 新增变量弹窗 ──────────────────────────────┘
```

#### PATH 逐行编辑器

PATH 行展开子面板，每行一个目录路径：
- 上移/下移调整顺序
- 添加新行 / 删除行
- 编辑路径文本
- 保存时自动拼接为分号分隔字符串写回注册表

#### 表格行内编辑

非 PATH 变量在表格中双击即可编辑变量名和值：
- 双击进入编辑模式，失焦或回车保存
- 保存失败时恢复原值并提示错误
- 编辑时行高亮，显示操作按钮

## 3. 路由与菜单

- 工具 ID: `envVars`
- 名称: "环境变量"
- 图标: 📝
- 分类: `system`（系统工具）
- 位置: 放在"开机启动项"之后

## 4. 版本号

- 6.3.0 → 6.4.0（新增工具菜单项，minor 版本升级）

## 5. 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/env_vars.rs` | 新增 | 后端命令实现 |
| `src-tauri/src/main.rs` | 修改 | 注册模块和命令 |
| `src/views/EnvVarsView.vue` | 新增 | 前端页面 |
| `src/utils/systemInfoClient.ts` | 修改 | 添加类型和 invoke 封装 |
| `src/store/index.ts` | 修改 | 添加 TOOL_LIST 条目 |
| `src/App.vue` | 修改 | 注册路由 |
| `package.json` | 修改 | 版本号 |
| `README.md` | 修改 | 版本路线 + 功能阶段 |
| `docs/superpowers/plans/feature-backlog.md` | 修改 | 标记 A6 完成 |

## 6. 注意事项

- 系统级变量修改需管理员权限，失败时提示用户以管理员身份运行
- PATH 拼接时保留原有分隔符，不引入多余分号
- 修改后广播 `WM_SETTINGCHANGE` 确保新启动的进程能看到变更
- 所有注册表操作通过 PowerShell 执行，遵循 Tauri 沙箱限制