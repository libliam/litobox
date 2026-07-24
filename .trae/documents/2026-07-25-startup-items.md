# A4 开机启动项管理 — V6.3 实现计划

## Context

用户选择开发 A4 开机启动项管理功能。启动项数据采集已在 `system_info.rs` 中通过 `Win32_StartupCommand` 实现，但前端 `SoftwareEnvView.vue` 未展示，且无操作能力。本次需要创建独立的管理页面，支持查看/启用/禁用/删除/新增启动项。

## 数据源

- 注册表：`HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` 和 `HKLM\...\Run`
- 启动文件夹：`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup` 和 `%ProgramData%\...`

## 操作方式

- **启用/禁用**：重命名前缀法（注册表值名加/去 `_disabled_` 前缀，.lnk 文件加/去 `.disabled` 后缀）
- **删除**：`reg delete` 或 `Remove-Item`
- **新增**：仅允许 HKCU 注册表和用户启动文件夹

## 关键文件

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/startup_items.rs` | **新建** | 后端模块：数据结构、PowerShell 脚本、Tauri 命令、单元测试 |
| `src-tauri/src/main.rs` | 修改 | `mod startup_items;` + 注册 5 个命令 |
| `src/views/StartupItemsView.vue` | **新建** | 前端页面：筛选栏、表格、下拉菜单操作列、新增弹窗 |
| `src/utils/systemInfoClient.ts` | 修改 | 类型定义 + invoke 封装函数 |
| `src/store/index.ts` | 修改 | TOOL_LIST 添加条目 |
| `src/App.vue` | 修改 | 导入组件 + toolComponentMap 注册 |

## 实现步骤

### Task 1: 后端 Rust 模块
- 数据结构：`StartupItemInfo`（name, command, location, source, enabled, is_system）, `StartupOpResult`
- PowerShell 脚本构建：采集（注册表 + 启动文件夹两路）、启用/禁用（重命名前缀）、删除（reg delete / Remove-Item）、新增（reg add / CreateShortcut）
- Tauri 命令：`get_startup_items`, `enable_startup_item`, `disable_startup_item`, `delete_startup_item`, `add_startup_item`
- 辅助函数：`run_powershell()`, `is_system_location()`, `parse_startup_op_result()`
- 单元测试：各种格式化函数、操作结果解析

### Task 2: 前端 TypeScript 接口
- `systemInfoClient.ts` 添加 `StartupItemInfo`、`StartupOpResult` 接口
- 添加 5 个 invoke 封装函数

### Task 3: 前端页面
- 布局：管理员权限提示横幅 → 统计概览 → 筛选栏 → 表格 → 底部栏
- 表格列：名称 | 命令 | 来源 | 位置 | 状态 | 操作（el-dropdown）
- 新增弹窗：名称输入 + 命令输入 + 位置选择
- 删除二次确认（useConfirmDialog）
- 自动刷新 + CSV 导出

### Task 4: 注册与集成
- `main.rs` 注册模块和命令
- `store/index.ts` TOOL_LIST 添加
- `App.vue` 注册组件

### Task 5: 版本号与文档
- 版本号 6.2.0 → 6.3.0（package.json / tauri.conf.json / Cargo.toml）
- README.md 添加 V6.3 功能记录
- feature-backlog.md 标记 A4 完成

## 验证

1. `cargo test startup_items -- --nocapture` 单元测试全部通过
2. `cargo check` 编译通过
3. `npx vue-tsc --noEmit` 类型检查通过
4. 进入页面可看到启动项列表，操作按钮可用