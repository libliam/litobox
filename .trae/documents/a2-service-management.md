# A2 Windows 服务管理 — 实现方案

## Context

在 LitoBox 中新增 Windows 服务管理工具，支持查看系统服务列表及启动/停止/重启操作。该功能属于系统工具增强（A2），优先级高，复用现有 `system_info.rs` 的 PowerShell 调用模式和 `ProcessListView.vue` 的前端交互模式。

## 修改文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/system_info.rs` | 修改 | 新增数据结构体 + 4 个命令 + 辅助函数 |
| `src-tauri/src/main.rs` | 修改 | 注册 4 个新命令 |
| `src/utils/systemInfoClient.ts` | 修改 | 新增类型和 invoke 封装 |
| `src/views/ServiceListView.vue` | **新建** | 服务管理页面 |
| `src/App.vue` | 修改 | 导入并注册组件 |
| `src/store/index.ts` | 修改 | TOOL_LIST 新增条目 |

## 后端实现

### 1. 数据结构体（`system_info.rs`）

新增 `ServiceItem` 和 `ServiceResult` 结构体，放在 `KillBatchResult` 之后。

### 2. 命令（`system_info.rs`）

- `get_services` — 使用 `Get-CimInstance Win32_Service | Select-Object Name, DisplayName, State, StartMode, Description | ConvertTo-Json` 获取所有服务，通过 `run_powershell_json` 解析。Win32_Service 在单次 WMI 查询中返回所有字段（包含 Description），无需额外查询。
- `start_service(name)` — `Start-Service -Name 'xxx' -ErrorAction Stop`
- `stop_service(name)` — `Stop-Service -Name 'xxx' -Force -ErrorAction Stop`
- `restart_service(name)` — `Restart-Service -Name 'xxx' -Force -ErrorAction Stop`

**辅助函数** `parse_service_result(output, name, action)` — 解析 PowerShell 输出，返回 `ServiceResult`，包含中英文错误消息友好化处理（权限拒绝、服务不存在、服务被保护等）。

所有命令使用 `debug_log!()` 输出日志，名称中的单引号需转义（`name.replace("'", "''")`）防止注入。

### 3. 注册（`main.rs`）

在 `kill_process_by_name` 之后新增 4 个命令注册。

## 前端实现

### 1. 类型和 invoke 封装（`systemInfoClient.ts`）

新增 `ServiceItem`、`ServiceResult` 接口，以及 `getServices()`、`startService(name)`、`stopService(name)`、`restartService(name)` 函数。

### 2. 页面组件（`ServiceListView.vue`）

**布局**：`tool-container` > `sticky-card`（搜索 + 状态筛选 + 刷新） + `tool-card`（数据表格）

**表格**：名称 | 显示名称 | 状态（el-tag，绿色"运行中"/灰色"已停止"） | 启动类型（中文映射：Auto→自动、Manual→手动、Disabled→禁用） | 描述 | 操作

**操作**：根据状态显示按钮
- 已停止 → "启动"按钮（绿色 link）
- 运行中 → "停止"（红色 link）+ "重启"（橙色 link）

**交互模式**（参照 ProcessListView）：
- 搜索：`el-input` + 300ms 防抖，匹配 name/display_name
- 筛选：`el-select` 状态筛选（全部/运行中/已停止）
- 操作：`ElMessageBox.confirm` 二次确认 → 按钮 loading（Set 管理） → invoke 后端命令 → ElMessage 提示 → 记录历史 → 500ms 后刷新列表
- 挂载时自动 `fetchServices()`

**历史记录**：`tool: 'serviceList'`，`inputFull`/`outputFull` 完整记录

### 3. 注册（`App.vue`）

- 导入 `ServiceListView`
- `toolComponentMap` 新增 `serviceList: ServiceListView`

### 4. 侧边栏（`store/index.ts`）

在 `processList` 条目后新增 `serviceList` 条目，`category: 'system'`。

## 关键设计决策

| 决策 | 原因 |
|------|------|
| 使用 `Get-CimInstance Win32_Service` 而非 `Get-Service` | `Get-Service` 不含 Description 字段，需 O(n) 次额外查询 |
| 不使用 `useBackgroundCollect` 后台采集模式 | 服务管理是交互式页面，每次操作后需刷新，直接调用更简单 |
| 操作后等待 500ms 再刷新 | 服务状态变更不是即时的，给 Windows 服务控制管理器时间 |
| 使用 `ServiceResult` 结构体（非裸字符串） | 与 `KillResult` 模式一致，前端 `success` 字段条件处理 |

## 验证方式

1. `npm run tauri dev` 启动应用
2. 侧边栏"系统工具"分类下应出现"服务管理"菜单项
3. 点击后自动加载服务列表，显示名称/显示名称/状态/启动类型/描述
4. 搜索框输入服务名，应正确过滤
5. 状态筛选下拉框切换，应正确过滤
6. 对已停止的服务点击"启动" → 确认 → 状态变为运行中
7. 对运行中的服务点击"停止" → 确认 → 状态变为已停止
8. 对运行中的服务点击"重启" → 确认 → 状态保持运行中
9. 操作后检查历史记录，应有对应的操作记录
10. 对系统关键服务执行停止操作，应显示友好错误提示
