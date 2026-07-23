# 计划任务管理器 V6.2 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增「计划任务」工具页（V6.2），查看 Windows Task Scheduler 列表，支持启用/禁用/立即运行/删除。

**Architecture:** 后端新建独立 Rust 模块 `scheduled_tasks.rs`，用 PowerShell `Get-ScheduledTask` cmdlet 采集 + `Enable/Disable/Start/Unregister-ScheduledTask` 操作，与 V5.6 服务管理（`system_info.rs` 内）走同一条 PowerShell + GBK 解码 + CREATE_NO_WINDOW 路线。前端新建 `ScheduledTasksView.vue`，布局综合 V5.6 `ServiceListView.vue`（admin-banner + el-table）与 V6.1 `NetworkConnections.vue`（stats-row + DataTable + 自动刷新 + CSV 导出 + useConfirmDialog）。

**Tech Stack:** Rust + Tauri 2.0 + PowerShell（GBK 解码）；Vue 3 Composition API + Element Plus + TypeScript。

**Spec:** `docs/superpowers/specs/2026-07-24-scheduled-tasks-design.md`

---

## 文件结构概览

| 文件 | 变更 | 责任 |
|---|---|---|
| `src-tauri/src/scheduled_tasks.rs` | 新建 | Rust 后端模块（结构 + 纯函数 + 5 命令 + 单元测试） |
| `src-tauri/src/main.rs` | 修改 | `mod scheduled_tasks;` + invoke_handler 注册 5 命令 |
| `src/utils/systemInfoClient.ts` | 修改 | TS 接口 + invoke 包装 + formatTriggerBrief 镜像 + console.assert |
| `src/views/ScheduledTasksView.vue` | 新建 | 工具页面 |
| `src/store/index.ts` | 修改 | TOOL_LIST 追加 scheduledTasks 条目 |
| `src/App.vue` | 修改 | import + toolComponentMap 注册 |
| `package.json` | 修改 | 版本 6.1.0 → 6.2.0 |
| `src-tauri/tauri.conf.json` | 修改 | 版本 6.1.0 → 6.2.0 |
| `src-tauri/Cargo.toml` | 修改 | 版本 6.1.0 → 6.2.0 |
| `README.md` | 修改 | 追加 V6.2 功能阶段记录 |
| `docs/superpowers/plans/feature-backlog.md` | 修改 | A3 标 ✅ + 检查清单更新 |

**复用模式参考**：
- 后端模式：[network_connections.rs](file:///d:/work/codes/litobox/src-tauri/src/network_connections.rs)（独立模块 + 私有 PowerShell 封装 + GBK 解码 + CREATE_NO_WINDOW）
- 后端 PowerShell 操作模式：[system_info.rs](file:///d:/work/codes/litobox/src-tauri/src/system_info.rs) 第 992-1113 行（`parse_service_result` + `start_service` 模式）
- 前端页面模板：[NetworkConnections.vue](file:///d:/work/codes/litobox/src/views/NetworkConnections.vue)（stats-row + sticky-card + DataTable + useConfirmDialog + CSV 导出 + 自动刷新）
- 前端 admin-banner 样式：[ServiceListView.vue](file:///d:/work/codes/litobox/src/views/ServiceListView.vue) 第 199-211 行
- Rust 测试惯例：[disk_analyzer.rs](file:///d:/work/codes/litobox/src-tauri/src/disk_analyzer.rs) 第 821 行起 `#[cfg(test)] mod tests`

---

## Task 1: 创建 scheduled_tasks.rs 骨架 + 纯函数 + 单元测试

**Files:**
- Create: `src-tauri/src/scheduled_tasks.rs`

- [ ] **Step 1: 创建模块骨架（结构 + PowerShell 封装 + stub 纯函数 + 单元测试）**

写入 `src-tauri/src/scheduled_tasks.rs`：

```rust
//! 计划任务管理器 — Get-ScheduledTask 采集 + Enable/Disable/Start/Unregister 操作
//!
//! 列出 Windows Task Scheduler 计划任务，关联运行时信息（LastRun/NextRun/Result），
//! 支持 4 种操作（启用/禁用/立即运行/删除）。默认隐藏 \Microsoft\Windows\ 系统子树。

use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize)]
pub struct ScheduledTask {
    pub task_name: String,
    pub task_path: String,
    pub state: String,
    pub description: String,
    pub author: String,
    pub last_run_time: String,
    pub last_task_result: i32,
    pub next_run_time: String,
    pub trigger_brief: String,
    pub action_brief: String,
    pub principal: String,
    pub is_system: bool,
    pub triggers_json: String,
    pub actions_json: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskOpResult {
    pub success: bool,
    pub task_name: String,
    pub action: String,
    pub message: String,
}

// ============ PowerShell 封装（与 network_connections.rs / system_info.rs 模式一致） ============

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("PowerShell 执行失败: {}", e))?;
    if !output.status.success() {
        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
        return Err(format!("PowerShell 错误: {}", stderr));
    }
    // ponytail: 中文 Windows PowerShell 输出为 GBK/CP936 编码
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

// ============ 纯函数（格式化与判定，单元测试覆盖） ============

/// 判断是否为系统任务（路径以 \Microsoft\ 开头）
fn is_system_task(task_path: &str) -> bool {
    task_path.starts_with(r"\Microsoft\")
}

/// 格式化触发器简述：取首个触发器，类型 + 关键时间
/// trigger_type: PowerShell CimClass.CimClassName（如 "MSFT_TaskDailyTrigger"）
/// start_boundary: ISO 8601 时间字符串（如 "2026-07-23T09:00:00"）或空
fn format_trigger_brief(trigger_type: &str, start_boundary: &str) -> String {
    let time = extract_time_from_boundary(start_boundary);
    match trigger_type {
        "MSFT_TaskDailyTrigger" => format!("每日 {}", time),
        "MSFT_TaskWeeklyTrigger" => format!("每周 {}", time),
        "MSFT_TaskLogonTrigger" => "登录时".to_string(),
        "MSFT_TaskBootTrigger" => "启动时".to_string(),
        "MSFT_TaskTimeTrigger" => format!("{} 一次性", time),
        "" => "自定义".to_string(),
        _ => "自定义".to_string(),
    }
}

/// 从 ISO 8601 StartBoundary 提取 HH:mm
/// 输入 "2026-07-23T09:00:00" → "09:00"；空或格式错误 → "—"
fn extract_time_from_boundary(boundary: &str) -> String {
    if boundary.is_empty() {
        return "—".to_string();
    }
    // 格式：2026-07-23T09:00:00 或 2026-07-23T09:00:00+08:00
    if let Some(time_part) = boundary.split('T').nth(1) {
        let clean = time_part.split('+').next().unwrap_or(time_part);
        if clean.len() >= 5 {
            return clean[..5].to_string();
        }
    }
    "—".to_string()
}

/// 格式化动作简述
/// actions_json: 序列化后的 Actions 数组 JSON
/// 每项形如 {"Type":"MSFT_TaskExecAction","Command":"...","Arguments":"..."}
fn format_action_brief(actions_json: &str) -> String {
    if actions_json.trim().is_empty() || actions_json.trim() == "[]" {
        return "—".to_string();
    }

    #[derive(Deserialize)]
    struct ActionItem {
        #[serde(rename = "Type")]
        #[allow(dead_code)]
        action_type: Option<String>,
        #[serde(rename = "Command")]
        command: Option<String>,
        #[serde(rename = "Arguments")]
        arguments: Option<String>,
    }

    let actions: Vec<ActionItem> = match serde_json::from_str(actions_json) {
        Ok(v) => v,
        Err(_) => return "自定义".to_string(),
    };

    if actions.is_empty() {
        return "—".to_string();
    }

    let first = &actions[0];
    let brief = match first.action_type.as_deref() {
        Some("MSFT_TaskExecAction") => {
            let cmd = first.command.clone().unwrap_or_default();
            // ponytail: 路径过长截断，避免表格列爆炸
            let short = if cmd.len() > 50 {
                format!("...{}", &cmd[cmd.len().saturating_sub(47)..])
            } else {
                cmd
            };
            format!("启动程序: {}", short)
        }
        Some("MSFT_TaskEmailAction") => "发送邮件".to_string(),
        Some("MSFT_TaskShowMessageAction") => "显示消息".to_string(),
        _ => "自定义".to_string(),
    };

    if actions.len() > 1 {
        format!("{} + {} 项", brief, actions.len() - 1)
    } else {
        brief
    }
}

/// 解析 PowerShell 操作命令输出，构造友好的 TaskOpResult
/// 复用 V5.6 parse_service_result 模式（SUCCESS/ERROR: 前缀 + 关键字映射）
fn parse_task_op_result(output: &str, task_name: &str, action: &str) -> TaskOpResult {
    let trimmed = output.trim();
    let action_cn = match action {
        "enable" => "启用",
        "disable" => "禁用",
        "run" => "运行",
        "delete" => "删除",
        _ => action,
    };

    if trimmed.starts_with("SUCCESS") {
        return TaskOpResult {
            success: true,
            task_name: task_name.to_string(),
            action: action.to_string(),
            message: format!("已{} 任务 \"{}\"", action_cn, task_name),
        };
    }

    if let Some(err_msg) = trimmed.strip_prefix("ERROR:") {
        let err = err_msg.trim();
        let friendly = if err.contains("denied") || err.contains("拒绝") {
            "拒绝访问，可能需要管理员权限".to_string()
        } else if err.contains("not found") || err.contains("找不到") || err.contains("不存在") {
            "任务不存在（可能已被删除）".to_string()
        } else if err.contains("running") || err.contains("正在运行") {
            "任务正在运行中，无法禁用".to_string()
        } else {
            err.to_string()
        };
        return TaskOpResult {
            success: false,
            task_name: task_name.to_string(),
            action: action.to_string(),
            message: format!("{} 失败: {}", action_cn, friendly),
        };
    }

    TaskOpResult {
        success: false,
        task_name: task_name.to_string(),
        action: action.to_string(),
        message: format!("{} 失败: {}", action_cn, trimmed),
    }
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    // ----- is_system_task -----

    #[test]
    fn is_system_task_microsoft_windows_subtree() {
        assert!(is_system_task(r"\Microsoft\Windows\Update\"));
    }

    #[test]
    fn is_system_task_microsoft_prefix_only() {
        // 仅 \Microsoft\ 前缀（无 Windows 子树）也算系统任务，保守策略
        assert!(is_system_task(r"\Microsoft\"));
    }

    #[test]
    fn is_system_task_third_party() {
        assert!(!is_system_task(r"\Google\Update\"));
    }

    #[test]
    fn is_system_task_root_path() {
        assert!(!is_system_task(r"\"));
    }

    #[test]
    fn is_system_task_user_task() {
        assert!(!is_system_task(r"\MyTask\"));
    }

    // ----- extract_time_from_boundary -----

    #[test]
    fn extract_time_iso_with_timezone() {
        assert_eq!(extract_time_from_boundary("2026-07-23T09:00:00+08:00"), "09:00");
    }

    #[test]
    fn extract_time_iso_without_timezone() {
        assert_eq!(extract_time_from_boundary("2026-07-23T09:00:00"), "09:00");
    }

    #[test]
    fn extract_time_empty() {
        assert_eq!(extract_time_from_boundary(""), "—");
    }

    #[test]
    fn extract_time_malformed() {
        assert_eq!(extract_time_from_boundary("invalid"), "—");
    }

    // ----- format_trigger_brief -----

    #[test]
    fn format_trigger_daily() {
        assert_eq!(format_trigger_brief("MSFT_TaskDailyTrigger", "2026-07-23T09:00:00"), "每日 09:00");
    }

    #[test]
    fn format_trigger_weekly() {
        assert_eq!(format_trigger_brief("MSFT_TaskWeeklyTrigger", "2026-07-23T08:30:00"), "每周 08:30");
    }

    #[test]
    fn format_trigger_logon() {
        assert_eq!(format_trigger_brief("MSFT_TaskLogonTrigger", ""), "登录时");
    }

    #[test]
    fn format_trigger_boot() {
        assert_eq!(format_trigger_brief("MSFT_TaskBootTrigger", ""), "启动时");
    }

    #[test]
    fn format_trigger_time() {
        assert_eq!(format_trigger_brief("MSFT_TaskTimeTrigger", "2026-07-23T15:00:00"), "15:00 一次性");
    }

    #[test]
    fn format_trigger_unknown_type() {
        assert_eq!(format_trigger_brief("MSFT_TaskCalendarTrigger", "2026-07-23T09:00:00"), "自定义");
    }

    #[test]
    fn format_trigger_empty() {
        assert_eq!(format_trigger_brief("", ""), "自定义");
    }

    // ----- format_action_brief -----

    #[test]
    fn format_action_exec() {
        let json = r#"[{"Type":"MSFT_TaskExecAction","Command":"C:\\Program Files\\Google\\Update\\update.exe","Arguments":"/c"}]"#;
        assert_eq!(format_action_brief(json), "启动程序: C:\\Program Files\\Google\\Update\\update.exe");
    }

    #[test]
    fn format_action_exec_long_path_truncated() {
        let long_cmd = "C:\\".to_string() + &"a".repeat(60) + "\\update.exe";
        let json = format!(
            r#"[{{"Type":"MSFT_TaskExecAction","Command":"{}","Arguments":""}}]"#,
            long_cmd
        );
        let result = format_action_brief(&json);
        assert!(result.starts_with("启动程序: ..."), "应截断前缀，实际: {}", result);
        assert!(result.ends_with("update.exe"), "应保留末尾文件名，实际: {}", result);
    }

    #[test]
    fn format_action_multiple() {
        let json = r#"[
            {"Type":"MSFT_TaskExecAction","Command":"C:\\a.exe","Arguments":""},
            {"Type":"MSFT_TaskExecAction","Command":"C:\\b.exe","Arguments":""}
        ]"#;
        assert_eq!(format_action_brief(json), "启动程序: C:\\a.exe + 1 项");
    }

    #[test]
    fn format_action_empty_array() {
        assert_eq!(format_action_brief("[]"), "—");
    }

    #[test]
    fn format_action_empty_string() {
        assert_eq!(format_action_brief(""), "—");
    }

    #[test]
    fn format_action_invalid_json() {
        assert_eq!(format_action_brief("not json"), "自定义");
    }

    // ----- parse_task_op_result -----

    #[test]
    fn parse_op_success_enable() {
        let r = parse_task_op_result("SUCCESS", "MyTask", "enable");
        assert!(r.success);
        assert_eq!(r.action, "enable");
        assert_eq!(r.message, "已启用 任务 \"MyTask\"");
    }

    #[test]
    fn parse_op_success_delete() {
        let r = parse_task_op_result("SUCCESS", "MyTask", "delete");
        assert!(r.success);
        assert_eq!(r.message, "已删除 任务 \"MyTask\"");
    }

    #[test]
    fn parse_op_error_denied() {
        let r = parse_task_op_result("ERROR: Access is denied", "MyTask", "enable");
        assert!(!r.success);
        assert!(r.message.contains("管理员权限"));
    }

    #[test]
    fn parse_op_error_not_found() {
        let r = parse_task_op_result("ERROR: The task was not found", "MyTask", "delete");
        assert!(!r.success);
        assert!(r.message.contains("不存在"));
    }

    #[test]
    fn parse_op_error_running() {
        let r = parse_task_op_result("ERROR: Task is currently running", "MyTask", "disable");
        assert!(!r.success);
        assert!(r.message.contains("运行中"));
    }

    #[test]
    fn parse_op_error_unknown() {
        let r = parse_task_op_result("ERROR: Something weird happened", "MyTask", "run");
        assert!(!r.success);
        assert!(r.message.contains("Something weird happened"));
    }

    #[test]
    fn parse_op_empty_output() {
        let r = parse_task_op_result("", "MyTask", "enable");
        assert!(!r.success);
        assert!(r.message.contains("失败"));
    }
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo check --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml`
Expected: 编译通过，可能有 dead_code 警告（结构体字段未使用），无 error。

- [ ] **Step 3: 运行单元测试**

Run: `cargo test --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml scheduled_tasks`
Expected: 所有 `tests::` 测试通过（约 20 个），输出 `test result: ok. N passed; 0 failed`。

- [ ] **Step 4: 提交**

```bash
git -C d:/work/codes/litobox add src-tauri/src/scheduled_tasks.rs
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): Rust 模块骨架 + 纯函数 + 单元测试"
```

---

## Task 2: 实现 5 个 Tauri 命令

**Files:**
- Modify: `src-tauri/src/scheduled_tasks.rs`（追加在文件末尾，单元测试 `mod tests` 之前）

- [ ] **Step 1: 追加 5 个 Tauri 命令实现**

在 `src-tauri/src/scheduled_tasks.rs` 的 `parse_task_op_result` 函数之后、`#[cfg(test)] mod tests` 之前追加：

```rust
// ============ Tauri 命令 ============

/// PowerShell 采集脚本：Get-ScheduledTask + Get-ScheduledTaskInfo
/// 显式投影 CimClass.CimClassName 为字符串，避免 ConvertTo-Json 丢失元数据
fn build_query_script(include_system: bool) -> String {
    let filter_clause = if include_system {
        "$true".to_string()
    } else {
        "$_.TaskPath -notlike '\\Microsoft\\Windows\\*'"
    };
    format!(
        r#"$tasks = Get-ScheduledTask | Where-Object {{ {filter_clause} }} | ForEach-Object {{
    $info = Get-ScheduledTaskInfo -TaskName $_.TaskName -TaskPath $_.TaskPath
    [PSCustomObject]{{
        TaskName        = $_.TaskName
        TaskPath        = $_.TaskPath
        State           = $_.State.ToString()
        Description     = if ($_.Description) {{ $_.Description }} else {{ '' }}
        Author          = if ($_.Author) {{ $_.Author }} else {{ '' }}
        Principal       = if ($_.Principal -and $_.Principal.UserId) {{ $_.Principal.UserId }} else {{ '' }}
        LastRunTime     = if ($info.LastRunTime) {{ $info.LastRunTime.ToString('yyyy-MM-dd HH:mm:ss') }} else {{ '' }}
        LastTaskResult  = $info.LastTaskResult
        NextRunTime     = if ($info.NextRunTime) {{ $info.NextRunTime.ToString('yyyy-MM-dd HH:mm:ss') }} else {{ '' }}
        Triggers        = @($_.Triggers | ForEach-Object {{
            [PSCustomObject]{{
                Type          = $_.CimClass.CimClassName
                StartBoundary = if ($_.StartBoundary) {{ $_.StartBoundary }} else {{ '' }}
                DaysInterval  = if ($_.DaysInterval) {{ $_.DaysInterval }} else {{ 0 }}
                DaysOfWeek    = if ($_.DaysOfWeek) {{ $_.DaysOfWeek }} else {{ 0 }}
            }}
        }})
        Actions         = @($_.Actions | ForEach-Object {{
            [PSCustomObject]{{
                Type      = $_.CimClass.CimClassName
                Command   = if ($_.Execute) {{ $_.Execute }} else {{ '' }}
                Arguments = if ($_.Arguments) {{ $_.Arguments }} else {{ '' }}
            }}
        }})
    }}
}} | ConvertTo-Json -Depth 4
Write-Output $tasks"#,
        filter_clause = filter_clause
    )
}

#[derive(Deserialize)]
struct PsScheduledTask {
    #[serde(rename = "TaskName")]
    task_name: String,
    #[serde(rename = "TaskPath")]
    task_path: String,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "Principal")]
    principal: Option<String>,
    #[serde(rename = "LastRunTime")]
    last_run_time: Option<String>,
    #[serde(rename = "LastTaskResult")]
    last_task_result: Option<i32>,
    #[serde(rename = "NextRunTime")]
    next_run_time: Option<String>,
    #[serde(rename = "Triggers")]
    triggers: Option<Vec<serde_json::Value>>,
    #[serde(rename = "Actions")]
    actions: Option<Vec<serde_json::Value>>,
}

#[tauri::command]
pub fn get_scheduled_tasks(include_system: bool) -> Result<Vec<ScheduledTask>, String> {
    debug_log!("[scheduled_tasks] 开始采集, include_system={}", include_system);

    let script = build_query_script(include_system);
    let output = run_powershell(&script)?;
    let trimmed = output.trim();

    if trimmed.is_empty() {
        debug_log!("[scheduled_tasks] 无任务");
        return Ok(Vec::new());
    }

    // ponytail: ConvertTo-Json 单元素时输出对象而非数组，需归一化
    let json_val: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("JSON 解析失败: {} - 输入前 200 字: {}", e, &trimmed[..200.min(trimmed.len())]))?;
    let arr: Vec<serde_json::Value> = match json_val {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![serde_json::Value::Object(json_val.as_object().unwrap().clone())],
        _ => Vec::new(),
    };

    let ps_tasks: Vec<PsScheduledTask> = arr
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or(PsScheduledTask {
            task_name: String::new(),
            task_path: String::new(),
            state: String::new(),
            description: None,
            author: None,
            principal: None,
            last_run_time: None,
            last_task_result: None,
            next_run_time: None,
            triggers: None,
            actions: None,
        }))
        .collect();

    let tasks: Vec<ScheduledTask> = ps_tasks
        .into_iter()
        .map(|ps| {
            let triggers_json = serde_json::to_string(&ps.triggers.clone().unwrap_or_default()).unwrap_or_default();
            let actions_json = serde_json::to_string(&ps.actions.clone().unwrap_or_default()).unwrap_or_default();

            // 取首个触发器作为简述依据
            let (trigger_type, trigger_boundary) = ps.triggers
                .as_ref()
                .and_then(|t| t.first())
                .and_then(|t| {
                    let ty = t.get("Type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let bd = t.get("StartBoundary").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    Some((ty, bd))
                })
                .unwrap_or_default();

            ScheduledTask {
                is_system: is_system_task(&ps.task_path),
                trigger_brief: format_trigger_brief(&trigger_type, &trigger_boundary),
                action_brief: format_action_brief(&actions_json),
                task_name: ps.task_name,
                task_path: ps.task_path,
                state: ps.state,
                description: ps.description.unwrap_or_default(),
                author: ps.author.unwrap_or_default(),
                principal: ps.principal.unwrap_or_default(),
                last_run_time: ps.last_run_time.unwrap_or_default(),
                last_task_result: ps.last_task_result.unwrap_or(0),
                next_run_time: ps.next_run_time.unwrap_or_default(),
                triggers_json,
                actions_json,
            }
        })
        .collect();

    debug_log!("[scheduled_tasks] 采集到 {} 个任务", tasks.len());
    Ok(tasks)
}

/// 通用操作执行器：构造 try/catch PowerShell，返回 TaskOpResult
fn execute_task_op(task_name: &str, task_path: &str, action: &str, ps_cmd: &str) -> Result<TaskOpResult, String> {
    debug_log!("[scheduled_tasks] {} task_name={}, task_path={}", action, task_name, task_path);

    // 拒绝操作系统任务（双重保险，前端已禁用）
    if action == "delete" && is_system_task(task_path) {
        return Ok(TaskOpResult {
            success: false,
            task_name: task_name.to_string(),
            action: action.to_string(),
            message: "系统任务不可删除".to_string(),
        });
    }

    // 转义单引号（PowerShell 字符串字面量用单引号包裹，内部单引号需翻倍）
    let escaped_name = task_name.replace('\'', "''");
    let escaped_path = task_path.replace('\'', "''");
    let script = format!(
        r#"try {{ {ps_cmd} -TaskName '{name}' -TaskPath '{path}' -ErrorAction Stop; Write-Output 'SUCCESS' }} catch {{ Write-Output "ERROR:$($_.Exception.Message)" }}"#,
        ps_cmd = ps_cmd,
        name = escaped_name,
        path = escaped_path,
    );

    let output = run_powershell(&script)?;
    let result = parse_task_op_result(&output, task_name, action);
    debug_log!("[scheduled_tasks] {} result: {:?}", action, result);
    Ok(result)
}

#[tauri::command]
pub fn enable_scheduled_task(task_name: String, task_path: String) -> Result<TaskOpResult, String> {
    execute_task_op(&task_name, &task_path, "enable", "Enable-ScheduledTask")
}

#[tauri::command]
pub fn disable_scheduled_task(task_name: String, task_path: String) -> Result<TaskOpResult, String> {
    execute_task_op(&task_name, &task_path, "disable", "Disable-ScheduledTask")
}

#[tauri::command]
pub fn run_scheduled_task(task_name: String, task_path: String) -> Result<TaskOpResult, String> {
    execute_task_op(&task_name, &task_path, "run", "Start-ScheduledTask")
}

#[tauri::command]
pub fn delete_scheduled_task(task_name: String, task_path: String) -> Result<TaskOpResult, String> {
    // Unregister-ScheduledTask 需要 -Confirm:$false 跳过交互确认
    debug_log!("[scheduled_tasks] delete task_name={}, task_path={}", task_name, task_path);

    if is_system_task(&task_path) {
        return Ok(TaskOpResult {
            success: false,
            task_name,
            action: "delete".to_string(),
            message: "系统任务不可删除".to_string(),
        });
    }

    let escaped_name = task_name.replace('\'', "''");
    let escaped_path = task_path.replace('\'', "''");
    let script = format!(
        r#"try {{ Unregister-ScheduledTask -TaskName '{name}' -TaskPath '{path}' -Confirm:$false -ErrorAction Stop; Write-Output 'SUCCESS' }} catch {{ Write-Output "ERROR:$($_.Exception.Message)" }}"#,
        name = escaped_name,
        path = escaped_path,
    );

    let output = run_powershell(&script)?;
    let result = parse_task_op_result(&output, &task_name, "delete");
    debug_log!("[scheduled_tasks] delete result: {:?}", result);
    Ok(result)
}
```

- [ ] **Step 2: 验证编译通过**

Run: `cargo check --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml`
Expected: 编译通过。`dead_code` 警告应消失（结构体字段在命令中使用）。

- [ ] **Step 3: 测试仍通过**

Run: `cargo test --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml scheduled_tasks`
Expected: 所有测试通过。

- [ ] **Step 4: 提交**

```bash
git -C d:/work/codes/litobox add src-tauri/src/scheduled_tasks.rs
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): 实现 5 个 Tauri 命令（采集+启用+禁用+运行+删除）"
```

---

## Task 3: main.rs 注册模块和命令

**Files:**
- Modify: `src-tauri/src/main.rs`（第 23 行附近 + 第 129 行附近）

- [ ] **Step 1: 追加 mod 声明**

编辑 `src-tauri/src/main.rs`，在 `mod network_connections;`（第 23 行）之后追加：

```rust
mod scheduled_tasks;
```

- [ ] **Step 2: 追加 invoke_handler 注册**

在 `.invoke_handler(tauri::generate_handler![` 数组中，`network_connections::get_network_connections,`（第 129 行）之后追加 5 行：

```rust
            // 计划任务管理命令
            scheduled_tasks::get_scheduled_tasks,
            scheduled_tasks::enable_scheduled_task,
            scheduled_tasks::disable_scheduled_task,
            scheduled_tasks::run_scheduled_task,
            scheduled_tasks::delete_scheduled_task,
```

- [ ] **Step 3: 验证编译通过**

Run: `cargo check --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml`
Expected: 编译通过。

- [ ] **Step 4: 提交**

```bash
git -C d:/work/codes/litobox add src-tauri/src/main.rs
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): main.rs 注册模块和 5 个命令"
```

---

## Task 4: 前端 systemInfoClient.ts 扩展

**Files:**
- Modify: `src/utils/systemInfoClient.ts`（在文件末尾 console.assert 之前追加）

- [ ] **Step 1: 追加类型定义和 invoke 包装**

在 `src/utils/systemInfoClient.ts` 文件中，找到 `export function getNetworkConnections()`（约第 315 行）之后、`// ============ 自检 ============` 之前追加：

```ts
// ============ 计划任务管理器 ============

export interface ScheduledTask {
  task_name: string
  task_path: string
  state: string         // "Ready" / "Running" / "Disabled" / "Unknown"
  description: string
  author: string
  last_run_time: string
  last_task_result: number
  next_run_time: string
  trigger_brief: string
  action_brief: string
  principal: string
  is_system: boolean
  triggers_json: string
  actions_json: string
}

export interface TaskOpResult {
  success: boolean
  task_name: string
  action: string
  message: string
}

export function getScheduledTasks(includeSystem: boolean): Promise<ScheduledTask[]> {
  return invoke<ScheduledTask[]>('get_scheduled_tasks', { includeSystem })
}

export function enableScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('enable_scheduled_task', { taskName, taskPath })
}

export function disableScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('disable_scheduled_task', { taskName, taskPath })
}

export function runScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('run_scheduled_task', { taskName, taskPath })
}

export function deleteScheduledTask(taskName: string, taskPath: string): Promise<TaskOpResult> {
  return invoke<TaskOpResult>('delete_scheduled_task', { taskName, taskPath })
}

/**
 * 前端镜像触发器格式化（与 Rust format_trigger_brief 一致），用于详情面板渲染
 */
export function formatTriggerBrief(triggerType: string, startBoundary: string): string {
  const time = extractTimeFromBoundary(startBoundary)
  switch (triggerType) {
    case 'MSFT_TaskDailyTrigger': return `每日 ${time}`
    case 'MSFT_TaskWeeklyTrigger': return `每周 ${time}`
    case 'MSFT_TaskLogonTrigger': return '登录时'
    case 'MSFT_TaskBootTrigger': return '启动时'
    case 'MSFT_TaskTimeTrigger': return `${time} 一次性`
    default: return '自定义'
  }
}

function extractTimeFromBoundary(boundary: string): string {
  if (!boundary) return '—'
  const parts = boundary.split('T')
  if (parts.length < 2) return '—'
  const timePart = parts[1].split('+')[0]
  return timePart.length >= 5 ? timePart.slice(0, 5) : '—'
}
```

- [ ] **Step 2: 在文件末尾的 console.assert 区追加自检**

在 `console.assert(formatUptime(90061) === '1天1小时1分钟', 'formatUptime(90061)')`（约第 325 行）之后追加：

```ts
console.assert(formatTriggerBrief('MSFT_TaskDailyTrigger', '2026-07-23T09:00:00') === '每日 09:00', 'daily trigger')
console.assert(formatTriggerBrief('MSFT_TaskWeeklyTrigger', '2026-07-23T08:30:00') === '每周 08:30', 'weekly trigger')
console.assert(formatTriggerBrief('MSFT_TaskLogonTrigger', '') === '登录时', 'logon trigger')
console.assert(formatTriggerBrief('MSFT_TaskBootTrigger', '') === '启动时', 'boot trigger')
console.assert(formatTriggerBrief('MSFT_TaskTimeTrigger', '2026-07-23T15:00:00') === '15:00 一次性', 'time trigger')
console.assert(formatTriggerBrief('', '') === '自定义', 'empty trigger')
console.assert(formatTriggerBrief('MSFT_TaskUnknown', '2026-07-23T09:00:00') === '自定义', 'unknown trigger')
console.assert(extractTimeFromBoundary('') === '—', 'empty boundary')
console.assert(extractTimeFromBoundary('invalid') === '—', 'malformed boundary')
console.assert(extractTimeFromBoundary('2026-07-23T09:00:00+08:00') === '09:00', 'boundary with tz')
```

- [ ] **Step 3: 验证 TypeScript 类型检查**

Run: `npx vue-tsc --noEmit -p d:\work\codes\litobox\tsconfig.json`
Expected: 无类型错误（如命令报 `vue-tsc not found`，改用 `npm run build` 或在 Vite dev 启动时观察）。

- [ ] **Step 4: 提交**

```bash
git -C d:/work/codes/litobox add src/utils/systemInfoClient.ts
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): systemInfoClient 追加 ScheduledTask 类型 + invoke 包装 + 自检"
```

---

## Task 5: 创建 ScheduledTasksView.vue 页面

**Files:**
- Create: `src/views/ScheduledTasksView.vue`

- [ ] **Step 1: 创建页面文件**

写入 `src/views/ScheduledTasksView.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- 管理员权限提示（复用 V5.6 样式） -->
    <div class="admin-banner">
      <span class="admin-icon">🛡️</span>
      启用/禁用/删除计划任务需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- 统计概览 -->
    <div v-if="!error && tasks.length" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ filteredTasks.length }}</span>
        <span class="stat-label">总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Ready') }}</span>
        <span class="stat-label">就绪</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Running') }}</span>
        <span class="stat-label">运行中</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ stateCount('Disabled') }}</span>
        <span class="stat-label">已禁用</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">计划任务</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索任务名/路径/描述/作者..." style="width: 260px" clearable />
          <el-select v-model="stateFilter" size="small" style="width: 120px">
            <el-option label="全部状态" value="all" />
            <el-option label="就绪" value="Ready" />
            <el-option label="运行中" value="Running" />
            <el-option label="已禁用" value="Disabled" />
            <el-option label="未知" value="Unknown" />
          </el-select>
          <span class="system-toggle">
            <span class="group-label">显示系统任务</span>
            <el-switch v-model="includeSystem" size="small" @change="onSystemToggleChange" />
          </span>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="fetchTasks">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !tasks.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击右上角「刷新」获取计划任务列表" />
      </div>
    </div>

    <!-- 任务表格 -->
    <div v-if="tasks.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">任务列表 ({{ filteredTasks.length }} / {{ tasks.length }})</span>
      </div>
      <div class="card-body">
        <el-table :data="filteredTasks" border size="small" max-height="600" style="width: 100%" v-loading="loading" row-key="rowKey">
          <el-table-column type="expand">
            <template #default="{ row }">
              <div class="expand-detail">
                <div class="detail-row"><span class="detail-label">作者：</span>{{ row.author || '—' }}</div>
                <div class="detail-row"><span class="detail-label">运行账户：</span>{{ row.principal || '—' }}</div>
                <div class="detail-row"><span class="detail-label">完整路径：</span>{{ row.task_path }}{{ row.task_name }}</div>
                <div class="detail-row"><span class="detail-label">描述：</span>{{ row.description || '—' }}</div>
                <div class="detail-row">
                  <span class="detail-label">完整触发器：</span>
                  <pre class="detail-json">{{ formatJson(row.triggers_json) }}</pre>
                </div>
                <div class="detail-row">
                  <span class="detail-label">执行动作：</span>
                  <pre class="detail-json">{{ formatJson(row.actions_json) }}</pre>
                </div>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="名称" min-width="220" sortable :sort-method="sortByName">
            <template #default="{ row }">
              <div class="task-name-cell">
                <span class="task-name">{{ row.task_name }}</span>
                <span class="task-path-hint">{{ row.task_path }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100" sortable :sort-method="sortByState">
            <template #default="{ row }">
              <el-tag :type="stateTagType(row.state)" size="small">{{ stateLabel(row.state) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="触发器" width="130">
            <template #default="{ row }">
              <span>{{ row.trigger_brief }}</span>
            </template>
          </el-table-column>
          <el-table-column label="上次运行" width="180" sortable prop="last_run_time">
            <template #default="{ row }">
              <div class="last-run-cell">
                <span>{{ row.last_run_time || '—' }}</span>
                <span v-if="row.last_run_time" :class="row.last_task_result === 0 ? 'result-ok' : 'result-fail'">
                  {{ row.last_task_result === 0 ? '✓' : `✗ ${row.last_task_result}` }}
                </span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="下次运行" width="170" sortable prop="next_run_time">
            <template #default="{ row }">
              <span>{{ row.next_run_time || '—' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="240" fixed="right">
            <template #default="{ row }">
              <el-button
                v-if="row.state === 'Disabled'"
                type="success" size="small" link
                :loading="operatingTasks.has(row.rowKey)"
                @click="handleAction(row, 'enable')">
                启用
              </el-button>
              <el-button
                v-if="row.state === 'Ready' || row.state === 'Disabled'"
                type="primary" size="small" link
                :loading="operatingTasks.has(row.rowKey)"
                @click="handleAction(row, 'run')">
                立即运行
              </el-button>
              <el-button
                v-if="row.state === 'Ready' || row.state === 'Running'"
                type="warning" size="small" link
                :loading="operatingTasks.has(row.rowKey)"
                @click="handleAction(row, 'disable')">
                禁用
              </el-button>
              <el-tooltip
                :content="row.is_system ? '系统任务不可删除' : ''"
                :disabled="!row.is_system"
                placement="top">
                <span>
                  <el-button
                    type="danger" size="small" link
                    :disabled="row.is_system"
                    :loading="operatingTasks.has(row.rowKey)"
                    @click="handleAction(row, 'delete')">
                    删除
                  </el-button>
                </span>
              </el-tooltip>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 底部栏 -->
    <div v-if="tasks.length" class="tool-card">
      <div class="card-body">
        <div class="bottom-bar">
          <div class="auto-refresh">
            <span class="group-label">自动刷新</span>
            <el-switch v-model="autoRefresh" size="small" @change="toggleAutoRefresh" />
            <el-select v-if="autoRefresh" v-model="refreshInterval" size="small" style="width: 80px" @change="restartAutoRefresh">
              <el-option label="5s" :value="5" />
              <el-option label="30s" :value="30" />
              <el-option label="60s" :value="60" />
            </el-select>
          </div>
          <div class="bottom-actions">
            <el-button size="small" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  getScheduledTasks,
  enableScheduledTask,
  disableScheduledTask,
  runScheduledTask,
  deleteScheduledTask,
  formatTimestamp,
  type ScheduledTask,
  type TaskOpResult,
} from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

const tasks = ref<ScheduledTask[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const stateFilter = ref('all')
const includeSystem = ref(false)
const operatingTasks = ref(new Set<string>())

// 自动刷新（默认关，计划任务变化频率低）
const autoRefresh = ref(false)
const refreshInterval = ref(30)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ============ 计算属性 ============

// 为每行加 rowKey，便于操作按钮 loading 标识
const tasksWithKey = computed(() =>
  tasks.value.map(t => ({ ...t, rowKey: `${t.task_path}|${t.task_name}` }))
)

const filteredTasks = computed(() => {
  let result = tasksWithKey.value
  if (stateFilter.value !== 'all') {
    result = result.filter(t => t.state === stateFilter.value)
  }
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    result = result.filter(t =>
      t.task_name.toLowerCase().includes(q) ||
      t.task_path.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      t.author.toLowerCase().includes(q)
    )
  }
  return result
})

const stateCount = (state: string) => filteredTasks.value.filter(t => t.state === state).length

// ============ 渲染辅助 ============

const stateLabel = (s: string) => ({
  Ready: '就绪', Running: '运行中', Disabled: '已禁用', Unknown: '未知',
}[s] || s)

const stateTagType = (s: string): 'success' | 'primary' | 'info' | 'warning' => ({
  Ready: 'primary', Running: 'success', Disabled: 'info', Unknown: 'warning',
}[s] as any) || 'info'

const sortByName = (a: any, b: any) => a.task_name.localeCompare(b.task_name)
const sortByState = (a: any, b: any) => {
  const order = ['Running', 'Ready', 'Disabled', 'Unknown']
  return order.indexOf(a.state) - order.indexOf(b.state)
}

const formatJson = (jsonStr: string): string => {
  if (!jsonStr || jsonStr === '[]') return '—'
  try {
    return JSON.stringify(JSON.parse(jsonStr), null, 2)
  } catch {
    return jsonStr
  }
}

// ============ 数据采集 ============

const fetchTasks = async () => {
  loading.value = true
  error.value = ''
  try {
    tasks.value = await getScheduledTasks(includeSystem.value)
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'scheduledTasks',
      action: '查看计划任务列表',
      inputPreview: includeSystem.value ? '含系统任务' : '不含系统任务',
      outputPreview: `${tasks.value.length} 个任务`,
      inputFull: JSON.stringify({ includeSystem: includeSystem.value }),
      outputFull: tasks.value.map(t => `${t.task_name} [${t.state}] ${t.task_path}`).join('\n'),
    })
  } catch (e) {
    error.value = '无法获取计划任务列表: ' + String(e)
  } finally {
    loading.value = false
  }
  if (autoRefresh.value && refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(fetchTasks, refreshInterval.value * 1000)
  }
}

const onSystemToggleChange = () => {
  // 显示系统任务开关变更：重新加载（开关开启时才拉取系统任务到内存）
  fetchTasks()
}

// ============ 操作处理 ============

const handleAction = async (task: any, action: 'enable' | 'disable' | 'run' | 'delete') => {
  const actionLabel = { enable: '启用', disable: '禁用', run: '立即运行', delete: '删除' }[action]
  const taskKey = task.rowKey

  // 删除需 danger 二次确认
  if (action === 'delete') {
    const ok = await confirm.ask(
      '删除计划任务',
      `确定删除任务 "${task.task_name}" (路径 ${task.task_path})？\n此操作不可恢复，可能影响相关程序正常运行。`,
      { type: 'danger', confirmText: '删除' }
    )
    if (!ok) return
  }

  operatingTasks.value.add(taskKey)
  try {
    const cmd = {
      enable: enableScheduledTask,
      disable: disableScheduledTask,
      run: runScheduledTask,
      delete: deleteScheduledTask,
    }[action]
    const result: TaskOpResult = await cmd(task.task_name, task.task_path)

    store.addHistory({
      tool: 'scheduledTasks',
      action: `${actionLabel}计划任务`,
      inputPreview: `${task.task_name} (${task.task_path})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ task_name: task.task_name, task_path: task.task_path, action }),
      outputFull: JSON.stringify(result),
    })

    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }

    // 删除成功后从内存移除，其他操作 300ms 后刷新列表
    if (action === 'delete' && result.success) {
      tasks.value = tasks.value.filter(t => `${t.task_path}|${t.task_name}` !== taskKey)
    } else {
      await new Promise(r => setTimeout(r, 300))
      await fetchTasks()
    }
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    operatingTasks.value.delete(taskKey)
  }
}

// ============ CSV 导出 ============

const exportCsv = async () => {
  const BOM = '\uFEFF'
  const header = '任务名,路径,状态,触发器,上次运行,上次结果,下次运行,作者,运行账户,描述'
  const rows = filteredTasks.value.map(t =>
    `"${t.task_name}","${t.task_path}","${t.state}","${t.trigger_brief}","${t.last_run_time}","${t.last_task_result}","${t.next_run_time}","${t.author}","${t.principal}","${t.description.replace(/"/g, '""')}"`
  )
  const csv = BOM + header + '\n' + rows.join('\n')

  const now = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  const filename = `计划任务_${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.csv`

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const savedPath = await invoke<string>('save_text_with_dialog', { content: csv, filename })
    if (savedPath) {
      ElMessage.success('CSV 已导出')
    }
  } catch (e) {
    ElMessage.error('导出失败: ' + String(e))
  }
}

// ============ 自动刷新 ============

const toggleAutoRefresh = (val: boolean) => {
  if (val) {
    refreshTimer = setInterval(fetchTasks, refreshInterval.value * 1000)
  } else if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

const restartAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(fetchTasks, refreshInterval.value * 1000)
  }
}

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

onMounted(() => {
  fetchTasks()
})
</script>

<style scoped>
.admin-banner {
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #f59e0b;
  margin-bottom: 16px;
}
.admin-icon { font-size: 16px; }

.stats-row { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }
.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 80px;
}
.stat-number { font-size: 22px; font-weight: 700; color: var(--accent-cyan); }
.stat-label { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }

.system-toggle { display: flex; align-items: center; gap: 6px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }

.task-name-cell { display: flex; flex-direction: column; }
.task-name { font-weight: 500; }
.task-path-hint { font-size: 11px; color: var(--text-secondary); margin-top: 2px; }

.last-run-cell { display: flex; align-items: center; gap: 6px; }
.result-ok { color: var(--accent-green); font-weight: 600; }
.result-fail { color: var(--accent-red); font-weight: 600; font-size: 11px; }

.expand-detail { padding: 8px 16px; background: var(--bg-input); border-radius: 4px; }
.detail-row { margin-bottom: 8px; font-size: 13px; line-height: 1.6; }
.detail-label { color: var(--accent-cyan); font-weight: 500; }
.detail-json {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 8px;
  margin-top: 4px;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  max-height: 200px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
}

.bottom-bar { display: flex; justify-content: space-between; align-items: center; }
.auto-refresh { display: flex; align-items: center; gap: 8px; }
.bottom-actions { display: flex; gap: 8px; }

.error-message {
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
}

:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 验证 Vue 类型检查**

Run: `npx vue-tsc --noEmit -p d:\work\codes\litobox\tsconfig.json`
Expected: 无类型错误。如 `vue-tsc` 不可用，跳过此步，在 Task 6 完成后用 `npm run build` 验证。

- [ ] **Step 3: 提交**

```bash
git -C d:/work/codes/litobox add src/views/ScheduledTasksView.vue
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): ScheduledTasksView.vue 工具页面"
```

---

## Task 6: 注册到 TOOL_LIST 和 App.vue

**Files:**
- Modify: `src/store/index.ts`（约第 98 行后追加）
- Modify: `src/App.vue`（约第 86 行 import + 约第 145 行 toolComponentMap）

- [ ] **Step 1: store/index.ts 追加 TOOL_LIST 条目**

在 `src/store/index.ts` 找到 `networkConnections` 条目（约第 98 行）：

```ts
  { id: 'networkConnections', name: '网络连接', ... },
```

在该行之后追加（注意：行尾的 `,` 保留，新条目位于 `]` 之前）：

```ts
  { id: 'scheduledTasks', name: '计划任务', icon: '🗓', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/><circle cx="8" cy="15" r="1.5"/><circle cx="12" cy="15" r="1.5"/><circle cx="16" cy="15" r="1.5"/></svg>`, description: '查看 Windows 计划任务列表，支持启用/禁用/立即运行/删除', keywords: ['计划任务', 'scheduled', 'task', 'schtasks', '定时', 'task scheduler'], category: 'system' },
```

- [ ] **Step 2: App.vue 追加 import**

在 `src/App.vue` 找到 `import NetworkConnections from '@/views/NetworkConnections.vue'`（约第 86 行），在其后追加：

```ts
import ScheduledTasksView from '@/views/ScheduledTasksView.vue'
```

- [ ] **Step 3: App.vue 追加 toolComponentMap 注册**

在 `src/App.vue` 找到 `networkConnections: NetworkConnections,`（约第 145 行），在其后追加：

```ts
  scheduledTasks: ScheduledTasksView,
```

- [ ] **Step 4: 验证前端构建**

Run: `npm --prefix d:/work/codes/litobox run build`
Expected: 构建成功，无类型错误，无 import 报错。

- [ ] **Step 5: 提交**

```bash
git -C d:/work/codes/litobox add src/store/index.ts src/App.vue
git -C d:/work/codes/litobox commit -m "feat(scheduled-tasks): 注册到 TOOL_LIST 和 App.vue 组件映射"
```

---

## Task 7: 版本号同步 + README + backlog 收尾

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 版本号 6.1.0 → 6.2.0（3 处）**

1. 编辑 `d:\work\codes\litobox\package.json`，第 3 行：`"version": "6.1.0"` → `"version": "6.2.0"`
2. 编辑 `d:\work\codes\litobox\src-tauri\tauri.conf.json`，第 3 行：`"version": "6.1.0"` → `"version": "6.2.0"`
3. 编辑 `d:\work\codes\litobox\src-tauri\Cargo.toml`，第 3 行：`version = "6.1.0"` → `version = "6.2.0"`

- [ ] **Step 2: README.md 追加 V6.2 功能阶段记录**

在 `d:\work\codes\litobox\README.md` 找到 V6.1 行（约第 326 行）：

```
| V6.1 | ✅ | 网络连接查看器（TCP/UDP 全量连接，关联进程，筛选/自动刷新/结束进程/释放端口/导出CSV） | 2026-07-23 |
```

在其后追加一行：

```
| V6.2 | ✅ | 计划任务管理器：查看 Windows 计划任务列表（默认隐藏 \Microsoft\Windows\ 系统任务，开关控制展开），状态/搜索筛选，启用/禁用/立即运行/删除（系统任务禁删、danger 二次确认），触发器简述 + 展开行详情（完整触发器/动作/作者/运行账户），CSV 导出，操作历史记录 | 2026-07-24 |
```

- [ ] **Step 3: feature-backlog.md 更新**

在 `d:\work\codes\litobox\docs\superpowers\plans\feature-backlog.md`：

1. 「已完成版本」表（约第 40 行 V6.1 行）后追加：

```
| V6.2 | ✅  | 计划任务管理器（Get-ScheduledTask 采集，启用/禁用/立即运行/删除，默认隐藏系统任务，触发器简述+展开行详情，CSV 导出）                                 | 2026-07-24 |
```

2. 候选池 A3 行（约第 57 行）更新为：

```
| A3  | ✅ **计划任务管理**   | 查看 Windows 计划任务列表，支持启用/禁用/删除 — 已完成 V6.2                                                                                                               | — 已完成 V6.2 — | 2026-07-06 brainstorming |
```

3. 「下次 brainstorming 检查清单」中（约第 197 行）：

   原：`- [ ] A3 计划任务管理 或 A4 开机启动项管理`
   改为：`- [x] A3 计划任务管理 — 已完成 V6.2`

- [ ] **Step 4: 验证 cargo check 仍通过**

Run: `cargo check --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml`
Expected: 编译通过（版本号变更不影响编译，但兜底验证）。

- [ ] **Step 5: 提交**

```bash
git -C d:/work/codes/litobox add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml README.md docs/superpowers/plans/feature-backlog.md
git -C d:/work/codes/litobox commit -m "chore(release): 发布 v6.2.0 - 新增 计划任务管理器"
```

---

## Task 8: 启动 dev 服务器进行手动验证

**Files:** 无文件变更

- [ ] **Step 1: 启动 Tauri dev 服务器**

Run（前台或后台）：`npm --prefix d:/work/codes/litobox run tauri dev`
Expected: 应用启动，无编译错误，主窗口显示。

注意：Rust 后端修改后必须重启 dev 服务器（Ctrl+C 停止后重新 `npm run tauri dev`），cargo check 通过不代表热更新生效（避坑指南 #4）。

- [ ] **Step 2: 验证功能清单**

进入侧边栏「系统工具」分类，点击「计划任务」菜单项，逐项验证：

- [ ] 页面加载后自动拉取任务列表（默认不含 \Microsoft\Windows\）
- [ ] admin-banner 黄色横幅显示
- [ ] stats-row 4 个卡片数字正确（总数/就绪/运行中/已禁用）
- [ ] 搜索框输入任务名 → 列表实时筛选
- [ ] 状态筛选下拉切换 → 列表筛选
- [ ] 「显示系统任务」开关打开 → 重新拉取，列表项明显增多
- [ ] 点击行展开 ▶ → 显示作者/运行账户/完整路径/描述/触发器 JSON/动作 JSON
- [ ] 状态为 Disabled 的任务 → 显示「启用」+「立即运行」按钮
- [ ] 状态为 Ready 的任务 → 显示「立即运行」+「禁用」按钮
- [ ] 状态为 Running 的任务 → 显示「禁用」按钮
- [ ] 系统任务（路径以 \Microsoft\ 开头）→ 「删除」按钮置灰，hover 显示 tooltip
- [ ] 点击「删除」非系统任务 → 弹出 danger 二次确认弹窗，取消后无操作
- [ ] 确认删除 → 操作执行，列表中该任务消失
- [ ] 点击「启用」/「禁用」/「立即运行」→ ElMessage 提示成功/失败
- [ ] 操作历史页面 → 能看到「查看计划任务列表」/「启用计划任务」等记录，双击可还原
- [ ] 底部「导出 CSV」→ 弹出保存对话框，CSV 文件内容正确（含 BOM 头，中文不乱码）
- [ ] 底部「自动刷新」开关打开 → 每 30s 自动刷新一次
- [ ] 切换到其他工具页再切回 → 自动刷新 timer 已清理，重新加载

- [ ] **Step 3: 验证错误场景**

- [ ] 以非管理员身份运行 → 启用/禁用/删除返回 "拒绝访问，可能需要管理员权限" 友好提示
- [ ] 删除一个系统任务（强制通过开发者工具移除 disabled）→ 后端返回 "系统任务不可删除"（双重保险）

- [ ] **Step 4: 验证单元测试**

Run: `cargo test --manifest-path d:\work\codes\litobox\src-tauri\Cargo.toml scheduled_tasks`
Expected: 所有测试通过，输出 `test result: ok. N passed; 0 failed`。

- [ ] **Step 5: 提交验证记录（可选）**

如发现 bug 修复后，按 `fix(scheduled-tasks): ...` 风格提交修复。验证全部通过后无需提交。

---

## 自审检查

**1. Spec 覆盖率**：
- ✅ Rust 后端独立模块（Task 1-2）
- ✅ 5 个 Tauri 命令签名与 spec 一致（Task 2）
- ✅ PowerShell 采集脚本 + CimClass 投影（Task 2 build_query_script）
- ✅ 触发器/动作格式化规则（Task 1 纯函数 + 单元测试）
- ✅ 错误处理与降级（Task 1 parse_task_op_result + Task 5 错误显示）
- ✅ 前端页面布局 admin-banner/stats-row/sticky-card/tool-card/底部栏（Task 5）
- ✅ 字段映射 7 列 + expand 详情（Task 5）
- ✅ 操作按钮条件渲染（Task 5）
- ✅ 删除 danger 二次确认 + 系统任务禁删（Task 5）
- ✅ 操作历史 inputFull/outputFull（Task 5）
- ✅ CSV 导出（Task 5）
- ✅ 自动刷新默认关（Task 5）
- ✅ main.rs 注册（Task 3）
- ✅ systemInfoClient.ts 类型 + invoke 包装 + 自检（Task 4）
- ✅ TOOL_LIST 注册（Task 6）
- ✅ App.vue 组件注册（Task 6）
- ✅ 版本号 6.1.0 → 6.2.0（Task 7）
- ✅ README 追加 V6.2（Task 7）
- ✅ backlog A3 标 ✅ + 检查清单更新（Task 7）
- ✅ Rust 单元测试（Task 1）
- ✅ 前端 console.assert 自检（Task 4）
- ✅ 手动验证清单（Task 8）

**2. Placeholder 扫描**：无 TBD / TODO / 模糊要求。所有代码块完整可执行。

**3. 类型一致性**：
- `ScheduledTask` 字段名 Rust ↔ TS 一致（snake_case）
- `TaskOpResult` 字段名 Rust ↔ TS 一致
- invoke 命令名 Rust ↔ TS 一致：`get_scheduled_tasks` / `enable_scheduled_task` / `disable_scheduled_task` / `run_scheduled_task` / `delete_scheduled_task`
- 前端 invoke 参数 camelCase（`includeSystem` / `taskName` / `taskPath`）与 Rust 函数参数 snake_case（`include_system` / `task_name` / `task_path`）一致，Tauri 2.x 自动转换
- `format_trigger_brief` Rust 端与 TS `formatTriggerBrief` 行为镜像，单元测试覆盖相同用例
