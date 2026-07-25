//!
//! 环境变量管理
//!
//! 通过 PowerShell 读写 Windows 注册表中的环境变量
//! （HKCU\Environment 和 HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment），
//! 修改后广播 WM_SETTINGCHANGE 通知系统刷新。

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvVarList {
    pub user: Vec<EnvVar>,
    pub system: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvVarResult {
    pub success: bool,
    pub message: String,
}

// ============ PowerShell 封装 ============

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
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

// ============ 纯函数（单元测试覆盖） ============

/// 转义 PowerShell 字符串中的特殊字符
fn ps_escape(s: &str) -> String {
    s.replace('\'', "''")
}

/// 解析操作结果输出
fn parse_op_result(raw: &str, _action: &str) -> EnvVarResult {
    let trimmed = raw.trim();
    if trimmed.starts_with("SUCCESS:") {
        EnvVarResult {
            success: true,
            message: trimmed[8..].to_string(),
        }
    } else if trimmed.starts_with("ERROR:") {
        EnvVarResult {
            success: false,
            message: trimmed[6..].to_string(),
        }
    } else {
        let truncated: String = if trimmed.len() > 200 {
            trimmed.chars().take(200).collect()
        } else {
            trimmed.to_string()
        };
        EnvVarResult {
            success: false,
            message: format!("未知错误: {}", truncated),
        }
    }
}

// ============ PowerShell 脚本构建 ============

/// 读取注册表环境变量
fn build_query_script() -> String {
    r#"$ErrorActionPreference = 'Stop'

function Get-EnvFromReg($regPath) {
    $vars = @()
    if (Test-Path $regPath) {
        $props = Get-ItemProperty -Path $regPath
        $props.PSObject.Properties | Where-Object { $_.Name -notin @('PSPath','PSParentPath','PSChildName','PSDrive','PSProvider') } | ForEach-Object {
            $vars += [PSCustomObject]@{ Name = $_.Name; Value = $_.Value }
        }
    }
    return $vars
}

$userVars = Get-EnvFromReg 'HKCU:\Environment'
$sysVars = Get-EnvFromReg 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment'

@{
    User   = @($userVars)
    System = @($sysVars)
} | ConvertTo-Json -Depth 3"#.to_string()
}

/// 设置/新增环境变量
fn build_set_script(name: &str, value: &str, scope: &str) -> String {
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $target = if ('{scope}' -eq 'system') {{ [EnvironmentVariableTarget]::Machine }} else {{ [EnvironmentVariableTarget]::User }}
    [Environment]::SetEnvironmentVariable('{name}', '{value}', $target)
    Write-Output 'SUCCESS:已保存'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        name = ps_escape(name),
        value = ps_escape(value),
        scope = scope,
    )
}

/// 删除环境变量
fn build_delete_script(name: &str, scope: &str) -> String {
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $target = if ('{scope}' -eq 'system') {{ [EnvironmentVariableTarget]::Machine }} else {{ [EnvironmentVariableTarget]::User }}
    [Environment]::SetEnvironmentVariable('{name}', $null, $target)
    Write-Output 'SUCCESS:已删除'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        name = ps_escape(name),
        scope = scope,
    )
}

// ============ Tauri 命令 ============

#[derive(Deserialize)]
struct PsEnvRaw {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Value")]
    value: Option<String>,
}

#[derive(Deserialize)]
struct PsEnvListRaw {
    #[serde(rename = "User")]
    user: Option<Vec<PsEnvRaw>>,
    #[serde(rename = "System")]
    system: Option<Vec<PsEnvRaw>>,
}

fn parse_env_list(raw: &[PsEnvRaw]) -> Vec<EnvVar> {
    raw.iter()
        .map(|r| EnvVar {
            name: r.name.clone().unwrap_or_default(),
            value: r.value.clone().unwrap_or_default(),
        })
        .collect()
}

#[tauri::command]
pub fn get_env_vars() -> Result<EnvVarList, String> {
    debug_log!("[env_vars] 开始采集环境变量");

    let script = build_query_script();
    let output = run_powershell(&script)?;
    let trimmed = output.trim();

    if trimmed.is_empty() {
        debug_log!("[env_vars] 无环境变量");
        return Ok(EnvVarList {
            user: Vec::new(),
            system: Vec::new(),
        });
    }

    let raw: PsEnvListRaw = serde_json::from_str(trimmed)
        .map_err(|e| format!("JSON 解析失败: {} - 输入前 200 字: {}", e, &trimmed[..200.min(trimmed.len())]))?;

    let user = parse_env_list(&raw.user.unwrap_or_default());
    let system = parse_env_list(&raw.system.unwrap_or_default());

    debug_log!("[env_vars] 采集到 user={}, system={}", user.len(), system.len());
    Ok(EnvVarList { user, system })
}

#[tauri::command]
pub fn set_env_var(name: String, value: String, scope: String) -> Result<EnvVarResult, String> {
    debug_log!("[env_vars] set: name={}, scope={}", name, scope);

    if name.trim().is_empty() {
        return Ok(EnvVarResult {
            success: false,
            message: "变量名不能为空".to_string(),
        });
    }

    let script = build_set_script(&name, &value, &scope);
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, "set"))
}

#[tauri::command]
pub fn delete_env_var(name: String, scope: String) -> Result<EnvVarResult, String> {
    debug_log!("[env_vars] delete: name={}, scope={}", name, scope);

    let script = build_delete_script(&name, &scope);
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, "delete"))
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_escape_single_quote() {
        assert_eq!(ps_escape("it's"), "it''s");
        assert_eq!(ps_escape("hello"), "hello");
        assert_eq!(ps_escape(""), "");
    }

    #[test]
    fn test_parse_op_result_success() {
        let r = parse_op_result("SUCCESS:已保存", "set");
        assert!(r.success);
        assert_eq!(r.message, "已保存");
    }

    #[test]
    fn test_parse_op_result_error() {
        let r = parse_op_result("ERROR:拒绝访问", "set");
        assert!(!r.success);
        assert_eq!(r.message, "拒绝访问");
    }

    #[test]
    fn test_parse_op_result_empty() {
        let r = parse_op_result("", "delete");
        assert!(!r.success);
        assert!(r.message.contains("未知错误"));
    }

    #[test]
    fn test_parse_op_result_unknown() {
        let r = parse_op_result("unexpected output", "set");
        assert!(!r.success);
        assert!(r.message.contains("未知错误"));
    }

    #[test]
    fn test_build_set_script_contains_name() {
        let script = build_set_script("MY_VAR", r"C:\test", "user");
        assert!(script.contains("MY_VAR"));
        assert!(script.contains("SetEnvironmentVariable"));
    }

    #[test]
    fn test_build_set_script_system_scope() {
        let script = build_set_script("MY_VAR", r"C:\test", "system");
        assert!(script.contains("Machine"));
    }

    #[test]
    fn test_build_delete_script_contains_name() {
        let script = build_delete_script("MY_VAR", "user");
        assert!(script.contains("MY_VAR"));
        assert!(script.contains("SetEnvironmentVariable"));
        assert!(script.contains("$null"));
    }

    #[test]
    fn test_parse_env_list_empty() {
        let result = parse_env_list(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_env_list_with_items() {
        let raw = vec![
            PsEnvRaw { name: Some("PATH".into()), value: Some("C:\\bin".into()) },
            PsEnvRaw { name: Some("JAVA_HOME".into()), value: Some("C:\\java".into()) },
        ];
        let result = parse_env_list(&raw);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "PATH");
        assert_eq!(result[1].value, "C:\\java");
    }

    #[test]
    fn test_parse_env_list_missing_fields() {
        let raw = vec![PsEnvRaw { name: None, value: None }];
        let result = parse_env_list(&raw);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "");
        assert_eq!(result[0].value, "");
    }
}