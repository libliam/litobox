# 计划任务管理器设计文档

> **版本**: V6.2
> **日期**: 2026-07-24
> **Backlog**: A3 计划任务管理

## 概述

新增独立工具页「计划任务」，查看 Windows Task Scheduler 计划任务列表，支持启用/禁用/立即运行/删除操作。默认隐藏 `\Microsoft\Windows\` 系统子树（开关控制展开），主表精简展示 + 展开行详情。与 V5.6 服务管理互补（服务管理看 Windows Services，本工具看 Task Scheduler 计划任务），覆盖 Windows 系统调度的另一面。

## 技术架构

```
┌─ Frontend (Vue 3) ──────────────────────────────────────────┐
│  ScheduledTasksView.vue                                     │
│  ├─ admin-banner (管理员权限提示, 复用 V5.6 样式)             │
│  ├─ stats-row (总数 / Ready / Running / Disabled 4 卡片)     │
│  ├─ sticky-card「计划任务」                                  │
│  │   └─ 搜索 + 状态筛选 + 显示系统任务开关 + 刷新             │
│  ├─ tool-card「任务列表」                                    │
│  │   └─ el-table type="expand"                              │
│  │       ├─ [展开] 详情面板 (作者/账户/触发器/动作/描述)     │
│  │       └─ 操作列: 启用/立即运行/禁用/删除                  │
│  └─ 底部栏: 自动刷新开关 (默认关) + 导出CSV                  │
├─────────────────────────────────────────────────────────────┤
│  systemInfoClient.ts (扩展)                                  │
│  invoke('get_scheduled_tasks', { includeSystem })           │
│  invoke('enable_scheduled_task', { taskName, taskPath })    │
│  invoke('disable_scheduled_task', ...)                      │
│  invoke('run_scheduled_task', ...)                           │
│  invoke('delete_scheduled_task', ...)                       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─ Backend (Rust) ───────────────────────────────────────────┐
│  scheduled_tasks.rs (新建, ~350 行)                         │
│  ├─ run_powershell / run_powershell_json (模块私有)          │
│  ├─ get_scheduled_tasks(include_system)                     │
│  │   ├─ Get-ScheduledTask + Get-ScheduledTaskInfo            │
│  │   ├─ 触发器/动作格式化 (format_trigger_brief)             │
│  │   └─ 过滤 \Microsoft\Windows\ (开关控制)                 │
│  ├─ enable/disable/run/delete_scheduled_task                │
│  │   └─ Enable/Disable/Start/Unregister-ScheduledTask        │
│  └─ 友好化错误 (复用 V5.6 parse_service_result 模式)         │
└─────────────────────────────────────────────────────────────┘
```

## 决策记录

| # | 决策 | 选项 | 选定 | 理由 |
|---|---|---|---|---|
| 1 | 后端技术方案 | A. Get-ScheduledTask / B. schtasks.exe / C. WMI Win32_ScheduledJob | A | 字段结构化、状态枚举明确、操作幂等；与 V5.6 服务管理 Get-CimInstance 模式一致；WMI Win32_ScheduledJob 只看 AT 任务（Win7 起废弃），不可用 |
| 2 | 显示范围 | A. 默认隐藏系统任务 / B. 全量 / C. 路径前缀下拉 | A | \Microsoft\Windows\ 子树 100+ 项用户基本不动；提供「显示系统任务」开关需要时展开；默认 30-80 项契合高频需求 |
| 3 | 操作范围 | A. 启用/禁用/立即运行/删除 / B. 不含删除 / C. 5 种含停止实例 | A | 与 backlog 描述一致；删除需 danger 二次确认 + 系统任务禁删；「停止运行中实例」使用频率低，YAGNI |
| 4 | 详情展示 | A. 主表精简 + 展开行 / B. 不展开 / C. 主表全字段 | A | 主表 7 列阅读舒适；展开行承载完整触发器/动作/作者/账户详情，避免横向滚动 |

## 数据结构

### Rust 后端

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ScheduledTask {
    pub task_name: String,           // "GoogleUpdateTaskMachineCore"
    pub task_path: String,           // "\Google\Update\"（不含任务名）
    pub state: String,               // "Ready"/"Running"/"Disabled"/"Unknown"
    pub description: String,         // 任务描述（可空）
    pub author: String,              // 作者（可空）
    pub last_run_time: String,       // "2026-07-23 09:00:12"（空则 "—")
    pub last_task_result: i32,       // 0=成功，其他=错误码（HRESULT）
    pub next_run_time: String,       // 下次运行时间（空则 "—")
    pub trigger_brief: String,       // "每日 09:00"/"登录时"/"启动时"
    pub action_brief: String,        // "启动程序: C:\...\update.exe"
    pub principal: String,           // "SYSTEM"/"Users"/"管理员"
    pub is_system: bool,              // task_path 以 \Microsoft\ 开头
    pub triggers_json: String,       // 完整 Triggers 数组 JSON（详情区用）
    pub actions_json: String,        // 完整 Actions 数组 JSON（详情区用）
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskOpResult {
    pub success: bool,
    pub task_name: String,
    pub action: String,              // "enable"/"disable"/"run"/"delete"
    pub message: String,             // 友好消息或错误描述
}
```

### PowerShell 采集脚本

实际执行采用一次性脚本：合并 `Get-ScheduledTask` + `Get-ScheduledTaskInfo` 为单个 PowerShell 脚本，避免多次 spawn。脚本输出 JSON 数组，每项含 task 基础字段 + info 运行时字段。

**关键约束**：PowerShell `ConvertTo-Json` 默认不序列化 CimClass 元数据（触发器类型如 `MSFT_TaskDailyTrigger` 会丢失），必须在脚本端显式投影为普通字符串字段，再交给 Rust 解析。

```powershell
$tasks = Get-ScheduledTask | Where-Object {
    $_.TaskPath -notlike '\Microsoft\Windows\*' -or $includeSystem
} | ForEach-Object {
    $info = Get-ScheduledTaskInfo -TaskName $_.TaskName -TaskPath $_.TaskPath
    # 显式展平嵌套对象，避免 ConvertTo-Json 丢失 CimClass / 嵌套对象
    [PSCustomObject]@{
        TaskName        = $_.TaskName
        TaskPath        = $_.TaskPath
        State           = $_.State.ToString()
        Description     = $_.Description
        Author          = $_.Author
        Principal       = if ($_.Principal) { $_.Principal.UserId } else { '' }
        LastRunTime     = $info.LastRunTime.ToString('yyyy-MM-dd HH:mm:ss')
        LastTaskResult  = $info.LastTaskResult
        NextRunTime     = if ($info.NextRunTime) { $info.NextRunTime.ToString('yyyy-MM-dd HH:mm:ss') } else { '' }
        # 触发器/动作用 CimClass 名称 + 关键字段构造数组，Rust 端再做格式化
        Triggers        = @($_.Triggers | ForEach-Object {
            [PSCustomObject]@{
                Type        = $_.CimClass.CimClassName
                StartBoundary = $_.StartBoundary
                DaysInterval = $_.DaysInterval
                DaysOfWeek   = $_.DaysOfWeek
            }
        })
        Actions         = @($_.Actions | ForEach-Object {
            [PSCustomObject]@{
                Type      = $_.CimClass.CimClassName
                Command   = $_.Execute
                Arguments = $_.Arguments
            }
        })
    }
} | ConvertTo-Json -Depth 4
```

Rust 端解析后填充 `ScheduledTask` 结构，包括 `trigger_brief` / `action_brief` / `principal` / `is_system` 的预处理。

**注意**：`Where-Object` 在此处使用是安全的，因为 Get-ScheduledTask 返回的是托管对象集合，不涉及 Tauri 子进程 PowerShell 沙箱对 `$_` 脚本块的限制（该限制主要影响纯命令行 schtasks）。如运行时遇到脚本块解析失败，降级方案为在 Rust 端过滤 `task_path` 而非 PowerShell 端过滤。

### Tauri 命令

| 命令 | 签名 | PowerShell 调用 |
|---|---|---|
| `get_scheduled_tasks` | `(include_system: bool) -> Result<Vec<ScheduledTask>, String>` | `Get-ScheduledTask` + `Get-ScheduledTaskInfo` |
| `enable_scheduled_task` | `(task_name: String, task_path: String) -> Result<TaskOpResult, String>` | `Enable-ScheduledTask -TaskName '...' -TaskPath '...'` |
| `disable_scheduled_task` | 同上 | `Disable-ScheduledTask` |
| `run_scheduled_task` | 同上 | `Start-ScheduledTask`（立即触发一次） |
| `delete_scheduled_task` | 同上 | `Unregister-ScheduledTask -Confirm:$false` |

**Tauri 命令参数命名规则**：函数参数 snake_case（`task_name`、`task_path`、`include_system`），前端 invoke 时传 camelCase（`taskName`、`taskPath`、`includeSystem`），Tauri 2.x 自动转换。结构体字段 snake_case，前端 TS 接口同步 snake_case。

### PowerShell 子进程规范

- `Command::new("powershell")` + `["-NoProfile", "-NonInteractive", "-Command", script]`
- `creation_flags(CREATE_NO_WINDOW)` 隐藏控制台窗口
- `encoding_rs::GBK.decode()` 解码输出（中文 Windows 默认 GBK 编码）
- 操作命令用 `try { ...; Write-Output 'SUCCESS' } catch { Write-Output "ERROR:$($_.Exception.Message)" }` 模式

### 触发器格式化规则

`format_trigger_brief(trigger_type, start_time) -> String`，纯函数：

| 触发器类型 | 输出 |
|---|---|
| `MSFT_TaskDailyTrigger` (CimClass 名 `MSFT_TaskDailyTrigger`) | "每日 HH:mm" |
| `MSFT_TaskWeeklyTrigger` | "每周X HH:mm"（X = 周一/二/...） |
| `MSFT_TaskLogonTrigger` | "登录时" |
| `MSFT_TaskBootTrigger` | "启动时" |
| `MSFT_TaskTimeTrigger` | "HH:mm 一次性" |
| 其他/空 | "自定义" |

### 动作格式化规则

`format_action_brief(actions_json) -> String`：
- 启动程序（`ExecAction`）→ "启动程序: <Command> <Arguments>"（截断超长路径）
- 发邮件（`EmailAction`，已废弃但兼容）→ "发送邮件: <Subject>"
- 显示消息（`ShowMessageAction`，已废弃但兼容）→ "显示消息: <Title>"
- 多个动作 → "启动程序 + 1 项"

## 前端实现

### 文件清单

| 文件 | 变更 | 说明 |
|---|---|---|
| `src-tauri/src/scheduled_tasks.rs` | 新建 | 后端模块，~350 行 |
| `src-tauri/src/main.rs` | 修改 | `mod scheduled_tasks;` + invoke_handler 注册 5 命令 |
| `src/utils/systemInfoClient.ts` | 修改 | 追加 ScheduledTask / TaskOpResult 接口 + 5 个 invoke 包装 |
| `src/views/ScheduledTasksView.vue` | 新建 | 工具页面 |
| `src/store/index.ts` | 修改 | TOOL_LIST 追加 scheduledTasks 条目 |
| `src/App.vue` | 修改 | import + toolComponentMap 注册 |
| `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml` | 修改 | 版本号 6.1.0 → 6.2.0 |
| `README.md` | 修改 | 追加 V6.2 功能阶段记录 |
| `docs/superpowers/plans/feature-backlog.md` | 修改 | A3 标 ✅，从检查清单移除 |

### 页面布局

```
┌─ admin-banner（黄底，复用 V5.6 样式）─────────────────────┐
│ 🛡️ 启用/禁用/删除计划任务需要管理员权限。请以管理员身份... │
└────────────────────────────────────────────────────────────┘

┌─ stats-row ──────────────────────────────────────────────┐
│ [总数 30] [就绪 25] [运行中 1] [禁用 4]                   │
└──────────────────────────────────────────────────────────┘

┌─ sticky-card「计划任务」 ────────────────────────────────┐
│ [搜索框    ] [全部状态▼] [显示系统任务 ◯] 09:15:23 [刷新] │
└──────────────────────────────────────────────────────────┘

┌─ tool-card「任务列表 (30 / 30)」 ────────────────────────┐
│ ▶ 名称              状态  触发器    上次运行  下次运行  操作│
│ ▼ GoogleUpdate...   就绪  每日09:00  07-23 ✓   07-24 ✓  [启用][立即运行][禁用][删除]│
│   作者: Google LLC                                          │
│   运行账户: SYSTEM                                          │
│   完整触发器: [ { "Type": "Daily", "StartBoundary": ... } ]│
│   执行动作: [ { "Type": "Exec", "Path": "...update.exe" } ]│
│   描述: Google Update 核心任务                              │
└──────────────────────────────────────────────────────────┘

┌─ tool-card 底部栏 ────────────────────────────────────────┐
│ 自动刷新 ◯ [5s▼]                          [导出 CSV]      │
└──────────────────────────────────────────────────────────┘
```

### 关键交互

1. **进入页面**：`onMounted` 调用 `fetchTasks()` 自动加载一次
2. **「显示系统任务」开关**：变更时立即触发 `fetchTasks(includeSystem)`，记录用户偏好（不持久化，每次进入页面默认关）
3. **筛选逻辑**：`filteredTasks` computed 联动 搜索 + 状态 + 系统任务开关（开关变更走 `fetchTasks` 而非纯前端过滤，因为系统任务不加载到内存）
4. **操作按钮条件渲染**：
   - `state === 'Disabled'` → 「启用」success link
   - `state === 'Ready' || 'Disabled'` → 「立即运行」primary link
   - `state === 'Ready' || 'Running'` → 「禁用」warning link
   - `!is_system` → 「删除」danger link（系统任务删除按钮 `:disabled` + tooltip "系统任务不可删除"）
5. **操作按钮 loading**：`operatingTasks = ref(new Set<string>())`，键为 `${task_path}|${task_name}`
6. **删除二次确认**（`useConfirmDialog`）：
   ```ts
   const ok = await confirm.ask(
     '删除计划任务',
     `确定删除任务 "${task.task_name}" (路径 ${task.task_path})？\n此操作不可恢复，可能影响相关程序正常运行。`,
     { type: 'danger', confirmText: '删除' }
   )
   ```
7. **操作后刷新**：`await new Promise(r => setTimeout(r, 300))` + `fetchTasks()`，与 V5.6 一致
8. **自动刷新**：默认关（计划任务变化频率低），开关开启后 `setInterval`，间隔可选 5s/30s/60s，`onUnmounted` 清理 timer
9. **CSV 导出**：复用 V6.1 模式，通过 `save_text_with_dialog` 后端命令保存到用户选择路径

### 字段映射

| 表格列 | 字段 | 宽度 | 排序 | 渲染 |
|---|---|---|---|---|
| 名称 | `task_name` + `task_path` 灰色小字 | min 200 | 是 | 主文本 + 灰色小字路径 |
| 状态 | `state` | 90 | 自定义排序 | el-tag: Ready=primary / Running=success / Disabled=info / Unknown=warning |
| 触发器 | `trigger_brief` | 120 | 否 | 普通文本 |
| 上次运行 | `last_run_time` + `last_task_result` | 160 | 是 | 时间 + ✓/✗ 图标（0=绿✓，其他=红✗ + 错误码） |
| 下次运行 | `next_run_time` | 160 | 是 | 普通文本 |
| 操作 | - | 240 fixed | 否 | 条件渲染按钮组 |

### 详情展开面板

`el-table` `type="expand"`，展开行内显示：
- 作者
- 运行账户（principal）
- 完整触发器列表（解析 `triggers_json`，每个触发器显示类型 + 起始时间 + 边界）
- 完整执行动作（解析 `actions_json`，每个动作显示类型 + 命令 + 参数）
- 任务描述
- 完整任务路径

### 状态筛选下拉

| label | value |
|---|---|
| 全部状态 | all |
| 就绪 | Ready |
| 运行中 | Running |
| 已禁用 | Disabled |
| 未知 | Unknown |

### 操作历史

遵循 AGENTS.md「inputFull/outputFull 必须」规则：

```ts
store.addHistory({
  tool: 'scheduledTasks',
  action: '启用计划任务',  // 或 禁用/立即运行/删除/查看列表
  inputPreview: `${task.task_name} (${task.task_path})`,
  outputPreview: result.message,
  inputFull: JSON.stringify({ task_name: task.task_name, task_path: task.task_path, action }),
  outputFull: JSON.stringify(result),
})
```

「查看列表」操作记录：`inputPreview: ''`，`outputPreview: '${n} 个任务'`，`inputFull: ''`，`outputFull: tasks.map(t => '${t.task_name} [${t.state}]').join('\n')`。

## 错误处理与降级

遵循 project_memory.md「多层降级」原则：

| 失败场景 | 处理 |
|---|---|
| PowerShell 不可用 | 返回 `Err("PowerShell 执行失败: ...")`，前端 error-message 红色提示 |
| 任务列表为空 | el-empty「暂无数据，点击右上角「刷新」获取计划任务列表」 |
| 操作返回 ERROR: | 解析错误信息，友好化映射：denied/拒绝 → "拒绝访问，可能需要管理员权限"；not found/找不到 → "任务不存在（可能已被删除）"；正在运行/running → "任务正在运行中，无法禁用"；其他 → 原始错误 |
| 删除系统任务 | 前端 `:disabled` + tooltip；后端拒绝，返回 "系统任务不可删除" |
| 删除二次确认取消 | 不执行，无提示 |
| 操作非管理员 | 顶部 admin-banner 静态提示 + 操作失败时 TaskOpResult.message 友好提示 |

## 测试与自检

### Rust 端单元测试

`scheduled_tasks.rs` 末尾追加 `#[cfg(test)] mod tests`，覆盖纯函数：

- `format_trigger_brief`：6 种触发器类型 + 空 trigger + 未知类型
- `format_action_brief`：ExecAction / EmailAction / ShowMessageAction / 多动作 / 空 actions
- `parse_task_op_result`：SUCCESS 前缀 / ERROR: 前缀 / denied 关键字 / not found 关键字 / running 关键字 / 未知错误
- `is_system_task`：`\Microsoft\Windows\...` true / `\Microsoft\` 仅前缀 true / `\Google\Update\` false / 根路径 false

不引入测试框架以外依赖，使用 `cargo test`。

### 前端自检

`systemInfoClient.ts` 末尾追加 `console.assert` 自检（与既有 `formatBytes` / `formatUptime` 自检模式一致）：

```ts
// 触发器简述格式化（前端镜像版本）
console.assert(formatTriggerBrief('MSFT_TaskDailyTrigger', '2026-07-23T09:00:00') === '每日 09:00', 'daily trigger')
console.assert(formatTriggerBrief('MSFT_TaskLogonTrigger', '') === '登录时', 'logon trigger')
console.assert(formatTriggerBrief('', '') === '自定义', 'unknown trigger')
```

## 收尾（版本号与 README 同步）

### 版本号 6.1.0 → 6.2.0

按 project_memory.md 规则：新增侧边栏菜单项为 minor bump。

| 文件 | 变更 |
|---|---|
| `package.json` | `"version": "6.1.0"` → `"6.2.0"` |
| `src-tauri/tauri.conf.json` | `"version": "6.1.0"` → `"6.2.0"` |
| `src-tauri/Cargo.toml` | `version = "6.1.0"` → `"6.2.0"` |

### TOOL_LIST 注册

在 `src/store/index.ts` 的 TOOL_LIST 数组中，`networkConnections` 条目（line 98）之后追加：

```ts
{ id: 'scheduledTasks', name: '计划任务', icon: '🗓', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/><circle cx="8" cy="15" r="1.5"/><circle cx="12" cy="15" r="1.5"/><circle cx="16" cy="15" r="1.5"/></svg>`, description: '查看 Windows 计划任务列表，支持启用/禁用/立即运行/删除', keywords: ['计划任务', 'scheduled', 'task', 'schtasks', '定时', 'task scheduler'], category: 'system' },
```

### main.rs 注册

```rust
mod scheduled_tasks;  // 在 mod network_connections; 之后

// invoke_handler 中，network_connections::get_network_connections 之后追加：
scheduled_tasks::get_scheduled_tasks,
scheduled_tasks::enable_scheduled_task,
scheduled_tasks::disable_scheduled_task,
scheduled_tasks::run_scheduled_task,
scheduled_tasks::delete_scheduled_task,
```

### App.vue 注册

```ts
import ScheduledTasksView from '@/views/ScheduledTasksView.vue'

const toolComponentMap = {
  // ...
  networkConnections: NetworkConnections,
  scheduledTasks: ScheduledTasksView,
}
```

### README.md 追加

在 V6.1 行之后追加：

```
| V6.2 | ✅ | 计划任务管理器：查看 Windows 计划任务列表（默认隐藏 \Microsoft\Windows\ 系统任务，开关控制展开），状态/搜索筛选，启用/禁用/立即运行/删除（系统任务禁删、danger 二次确认），触发器简述 + 展开行详情（完整触发器/动作/作者/运行账户），CSV 导出，操作历史记录 | 2026-07-24 |
```

### feature-backlog.md 更新

- A3 行更新为 `✅ **计划任务管理** — 已完成 V6.2`
- 「下次 brainstorming 检查清单」中 `A3 计划任务管理 或 A4 开机启动项管理` 项改为 `A3 计划任务管理 — 已完成 V6.2`
- 「已完成版本」表追加 V6.2 行

## 不做（YAGNI）

明确排除以下功能，避免范围蔓延：

- **新建计划任务**：UI 复杂（触发器配置/动作配置/账户选择多层表单），低频需求，YAGNI
- **修改计划任务属性**：与新建同样复杂，YAGNI
- **导入/导出 XML**：高级用户场景，可后续通过 Get-ScheduledTask | Export-ScheduledTask 命令自行操作
- **触发器可视化编辑**：与 V1.x cron 工具不同场景，本工具只读查看 + 启停运行删除
- **远程计算机任务查看**：需要 RPC 权限和网络访问，违反「纯本地离线」定位
- **任务运行日志查看**：Windows 事件查看器已覆盖，YAGNI
- **通知/告警集成**：违反「无网络请求」定位
