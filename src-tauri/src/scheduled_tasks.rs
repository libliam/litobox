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
        // 用 serde_json::json! 宏构造，自动转义反斜杠（手工 format! 会丢失转义）
        let long_cmd = format!("C:\\{}\\update.exe", "a".repeat(60));
        let json = serde_json::json!([{
            "Type": "MSFT_TaskExecAction",
            "Command": long_cmd,
            "Arguments": ""
        }]).to_string();
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
