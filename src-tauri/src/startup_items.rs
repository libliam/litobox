//!
//! 开机启动项管理
//!
//! 从注册表 Run 键和启动文件夹采集 Windows 开机启动项，
//! 支持 4 种操作（启用/禁用/删除/新增）。

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
pub struct StartupItemInfo {
    pub name: String,
    pub command: String,
    pub location: String,       // 注册表路径或启动文件夹路径
    pub source: String,         // "registry" | "startup_folder"
    pub enabled: bool,
    pub is_system: bool,        // HKLM 或公共启动文件夹
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupOpResult {
    pub success: bool,
    pub item_name: String,
    pub action: String,
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

/// 判断位置是否为系统级（HKLM 或公共启动文件夹）
fn is_system_location(location: &str) -> bool {
    let lower = location.to_lowercase();
    lower.contains("hklm") || lower.contains("hkey_local_machine") || lower.contains("\\programdata\\")
}

/// 从注册表值名判断是否已禁用（以 _disabled_ 开头）
fn is_disabled_by_name(name: &str) -> bool {
    name.starts_with("_disabled_")
}

/// 去掉 _disabled_ 前缀，返回原始名称
fn strip_disabled_prefix(name: &str) -> String {
    if is_disabled_by_name(name) {
        name[10..].to_string()
    } else {
        name.to_string()
    }
}

/// 解析操作结果输出
/// PowerShell 操作脚本输出 "SUCCESS:xxx" 或 "ERROR:xxx"
fn parse_op_result(raw: &str, item_name: &str, action: &str) -> StartupOpResult {
    let trimmed = raw.trim();
    if trimmed.starts_with("SUCCESS:") {
        let msg = trimmed[8..].to_string();
        StartupOpResult {
            success: true,
            item_name: item_name.to_string(),
            action: action.to_string(),
            message: msg,
        }
    } else if trimmed.starts_with("ERROR:") {
        let msg = trimmed[6..].to_string();
        StartupOpResult {
            success: false,
            item_name: item_name.to_string(),
            action: action.to_string(),
            message: msg,
        }
    } else {
        let truncated: String = if trimmed.len() > 200 {
            trimmed.chars().take(200).collect()
        } else {
            trimmed.to_string()
        };
        StartupOpResult {
            success: false,
            item_name: item_name.to_string(),
            action: action.to_string(),
            message: format!("未知错误: {}", truncated),
        }
    }
}

// ============ 格式化函数 ============

fn format_source_label(source: &str) -> String {
    match source {
        "registry" => "注册表".to_string(),
        "startup_folder" => "启动文件夹".to_string(),
        _ => source.to_string(),
    }
}

fn format_location_short(location: &str) -> String {
    // 缩短注册表路径显示
    if location.to_lowercase().contains("hklm") {
        location.replace(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\", "HKLM\\...\\")
            .replace("HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\", "HKLM\\...\\")
    } else if location.to_lowercase().contains("hkcu") {
        location.replace(r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\", "HKCU\\...\\")
            .replace("HKEY_CURRENT_USER\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\", "HKCU\\...\\")
    } else {
        location.to_string()
    }
}

// ============ PowerShell 脚本构建 ============

/// 采集脚本：注册表 Run 键 + 启动文件夹两路
fn build_query_script() -> String {
    r#"$ErrorActionPreference = 'Stop'
$items = @()

# 路1: 注册表 Run 键
$regPaths = @(
    @{Path='HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run'; IsSystem=$false},
    @{Path='HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run'; IsSystem=$true}
)
foreach ($rp in $regPaths) {
    if (Test-Path $rp.Path) {
        $props = Get-ItemProperty -Path $rp.Path
        $props.PSObject.Properties | Where-Object { $_.Name -notin @('PSPath','PSParentPath','PSChildName','PSDrive','PSProvider') } | ForEach-Object {
            $enabled = -not $_.Name.StartsWith('_disabled_')
            $items += [PSCustomObject]@{
                Name     = $_.Name
                Command  = $_.Value
                Location = $rp.Path.Replace('Microsoft.PowerShell.Core\Registry::','')
                Source   = 'registry'
                Enabled  = $enabled
                IsSystem = $rp.IsSystem
            }
        }
    }
}

# 路2: 启动文件夹
$folderPaths = @(
    @{Path="$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"; IsSystem=$false},
    @{Path="$env:ProgramData\Microsoft\Windows\Start Menu\Programs\Startup"; IsSystem=$true}
)
foreach ($fp in $folderPaths) {
    if (Test-Path $fp.Path) {
        $shell = New-Object -ComObject WScript.Shell
        Get-ChildItem $fp.Path -Filter '*.lnk' -ErrorAction SilentlyContinue | ForEach-Object {
            $enabled = -not $_.Name.EndsWith('.disabled')
            $cmd = ''
            try { $cmd = $shell.CreateShortcut($_.FullName).TargetPath } catch {}
            $items += [PSCustomObject]@{
                Name     = $_.Name -replace '\.lnk$','' -replace '\.disabled$',''
                Command  = $cmd
                Location = $fp.Path
                Source   = 'startup_folder'
                Enabled  = $enabled
                IsSystem = $fp.IsSystem
            }
        }
        if ($shell) { [System.Runtime.Interopservices.Marshal]::ReleaseComObject($shell) | Out-Null }
    }
}

$items | ConvertTo-Json -Depth 3"#.to_string()
}

/// 注册表启用/禁用：重命名值名
fn build_reg_toggle_script(name: &str, location: &str, enable: bool) -> String {
    let (old_name, new_name) = if enable {
        (format!("_disabled_{}", name), name.to_string())
    } else {
        (name.to_string(), format!("_disabled_{}", name))
    };
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $data = (Get-ItemProperty -Path '{0}' -Name '{1}' -ErrorAction Stop).'{1}'
    New-ItemProperty -Path '{0}' -Name '{2}' -Value $data -PropertyType String -Force | Out-Null
    Remove-ItemProperty -Path '{0}' -Name '{1}' -Force
    Write-Output 'SUCCESS:操作成功'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        location,
        old_name,
        new_name,
    )
}

/// 启动文件夹启用/禁用：重命名 .lnk 文件
fn build_folder_toggle_script(name: &str, location: &str, enable: bool) -> String {
    let (old_suffix, new_suffix) = if enable {
        (".lnk.disabled", ".lnk")
    } else {
        (".lnk", ".lnk.disabled")
    };
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $oldPath = Join-Path '{}' '{}{}'
    $newPath = Join-Path '{}' '{}{}'
    if (-not (Test-Path $oldPath)) {{ throw "文件不存在" }}
    Rename-Item -Path $oldPath -NewName ('{}{}') -ErrorAction Stop
    Write-Output 'SUCCESS:操作成功'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        location.replace("'", "''"),
        name.replace("'", "''"), old_suffix,
        location.replace("'", "''"),
        name.replace("'", "''"), new_suffix,
        name.replace("'", "''"), new_suffix,
    )
}

/// 删除注册表值
fn build_reg_delete_script(name: &str, location: &str) -> String {
    let reg_path = location.replace("HKCU:", "HKCU").replace("HKLM:", "HKLM");
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    reg delete "{}" /v "{}" /f 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {{ throw "删除失败" }}
    Write-Output 'SUCCESS:已删除'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        reg_path.replace("\\", "\\\\"),
        name.replace("\"", "\\\"")
    )
}

/// 删除启动文件夹 .lnk
fn build_folder_delete_script(name: &str, location: &str) -> String {
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $path = Join-Path '{}' '{}.lnk'
    if (-not (Test-Path $path)) {{ $path = Join-Path '{}' '{}.lnk.disabled' }}
    if (-not (Test-Path $path)) {{ throw "文件不存在" }}
    Remove-Item $path -Force -ErrorAction Stop
    Write-Output 'SUCCESS:已删除'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        location.replace("'", "''"),
        name.replace("'", "''"),
        location.replace("'", "''"),
        name.replace("'", "''"),
    )
}

/// 新增启动文件夹 .lnk 快捷方式
fn build_folder_add_script(name: &str, command: &str, location: &str) -> String {
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut('{0}\{1}.lnk')
    $shortcut.TargetPath = '{2}'
    $shortcut.Save()
    [System.Runtime.Interopservices.Marshal]::ReleaseComObject($shell) | Out-Null
    Write-Output 'SUCCESS:已添加'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        location.replace("'", "''"),
        name.replace("'", "''"),
        command.replace("'", "''"),
    )
}

/// 新增注册表启动项
fn build_add_script(name: &str, command: &str, location: &str) -> String {
    let reg_path = location.replace("HKCU:", "HKCU").replace("HKLM:", "HKLM");
    format!(
        r#"$ErrorActionPreference = 'Stop'
try {{
    reg add "{}" /v "{}" /d "{}" /f 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {{ throw "添加失败" }}
    Write-Output 'SUCCESS:已添加'
}} catch {{
    Write-Output "ERROR:$($_.Exception.Message)"
}}"#,
        reg_path.replace("\\", "\\\\"),
        name.replace("\"", "\\\""),
        command.replace("\"", "\\\"")
    )
}

// ============ Tauri 命令 ============

#[derive(Deserialize)]
struct PsStartupRaw {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Command")]
    command: Option<String>,
    #[serde(rename = "Location")]
    location: Option<String>,
    #[serde(rename = "Source")]
    source: Option<String>,
    #[serde(rename = "Enabled")]
    enabled: Option<bool>,
    #[serde(rename = "IsSystem")]
    is_system: Option<bool>,
}

#[tauri::command]
pub fn get_startup_items() -> Result<Vec<StartupItemInfo>, String> {
    debug_log!("[startup_items] 开始采集");

    let script = build_query_script();
    let output = run_powershell(&script)?;
    let trimmed = output.trim();

    if trimmed.is_empty() {
        debug_log!("[startup_items] 无启动项");
        return Ok(Vec::new());
    }

    let raw_items: Vec<PsStartupRaw> = serde_json::from_str(trimmed)
        .map_err(|e| format!("JSON 解析失败: {} - 输入前 200 字: {}", e, &trimmed[..200.min(trimmed.len())]))?;

    let items: Vec<StartupItemInfo> = raw_items
        .into_iter()
        .map(|r| StartupItemInfo {
            name: r.name.unwrap_or_default(),
            command: r.command.unwrap_or_default(),
            location: r.location.unwrap_or_default(),
            source: r.source.unwrap_or_default(),
            enabled: r.enabled.unwrap_or(true),
            is_system: r.is_system.unwrap_or(false),
        })
        .collect();

    debug_log!("[startup_items] 采集到 {} 个启动项", items.len());
    Ok(items)
}

#[tauri::command]
pub fn enable_startup_item(name: String, location: String, source: String) -> Result<StartupOpResult, String> {
    debug_log!("[startup_items] enable: name={}, source={}", name, source);

    let script = if source == "startup_folder" {
        build_folder_toggle_script(&name, &location, true)
    } else {
        build_reg_toggle_script(&name, &location, true)
    };
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, &name, "enable"))
}

#[tauri::command]
pub fn disable_startup_item(name: String, location: String, source: String) -> Result<StartupOpResult, String> {
    debug_log!("[startup_items] disable: name={}, source={}", name, source);

    let script = if source == "startup_folder" {
        build_folder_toggle_script(&name, &location, false)
    } else {
        build_reg_toggle_script(&name, &location, false)
    };
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, &name, "disable"))
}

#[tauri::command]
pub fn delete_startup_item(name: String, location: String, source: String) -> Result<StartupOpResult, String> {
    debug_log!("[startup_items] delete: name={}, source={}", name, source);

    let script = if source == "startup_folder" {
        build_folder_delete_script(&name, &location)
    } else {
        build_reg_delete_script(&name, &location)
    };
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, &name, "delete"))
}

#[tauri::command]
pub fn add_startup_item(name: String, command: String, source: String) -> Result<StartupOpResult, String> {
    debug_log!("[startup_items] add: name={}, source={}", name, source);

    if name.trim().is_empty() || command.trim().is_empty() {
        return Ok(StartupOpResult {
            success: false,
            item_name: name,
            action: "add".to_string(),
            message: "名称和命令不能为空".to_string(),
        });
    }

    let location = if source == "startup_folder" {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        format!(r"{}\Microsoft\Windows\Start Menu\Programs\Startup", appdata)
    } else {
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run".to_string()
    };

    let script = if source == "startup_folder" {
        build_folder_add_script(&name, &command, &location)
    } else {
        build_add_script(&name, &command, &location)
    };
    let output = run_powershell(&script)?;
    Ok(parse_op_result(&output, &name, "add"))
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_location_hklm() {
        assert!(is_system_location(r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"));
        assert!(is_system_location(r"HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"));
        assert!(!is_system_location(r"HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"));
    }

    #[test]
    fn test_is_system_location_programdata() {
        assert!(is_system_location(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup"));
        assert!(!is_system_location(r"C:\Users\test\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup"));
    }

    #[test]
    fn test_is_disabled_by_name() {
        assert!(is_disabled_by_name("_disabled_MyApp"));
        assert!(!is_disabled_by_name("MyApp"));
        assert!(!is_disabled_by_name("disabled_MyApp"));
    }

    #[test]
    fn test_strip_disabled_prefix() {
        assert_eq!(strip_disabled_prefix("_disabled_MyApp"), "MyApp");
        assert_eq!(strip_disabled_prefix("MyApp"), "MyApp");
        assert_eq!(strip_disabled_prefix(""), "");
    }

    #[test]
    fn test_parse_op_result_success() {
        let r = parse_op_result("SUCCESS:操作成功", "Test", "enable");
        assert!(r.success);
        assert_eq!(r.item_name, "Test");
        assert_eq!(r.action, "enable");
        assert_eq!(r.message, "操作成功");
    }

    #[test]
    fn test_parse_op_result_error() {
        let r = parse_op_result("ERROR:拒绝访问", "Test", "delete");
        assert!(!r.success);
        assert_eq!(r.message, "拒绝访问");
    }

    #[test]
    fn test_parse_op_result_empty() {
        let r = parse_op_result("", "Test", "delete");
        assert!(!r.success);
        assert!(r.message.contains("未知错误"));
    }

    #[test]
    fn test_parse_op_result_unknown() {
        let r = parse_op_result("something unexpected", "Test", "enable");
        assert!(!r.success);
        assert!(r.message.contains("未知错误"));
    }

    #[test]
    fn test_format_source_label() {
        assert_eq!(format_source_label("registry"), "注册表");
        assert_eq!(format_source_label("startup_folder"), "启动文件夹");
        assert_eq!(format_source_label("other"), "other");
    }

    #[test]
    fn test_format_location_short() {
        let hklm = format_location_short(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run");
        assert!(hklm.contains("HKLM\\...\\"));
        let hkcu = format_location_short(r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run");
        assert!(hkcu.contains("HKCU\\...\\"));
    }

    #[test]
    fn test_build_add_script_contains_name() {
        let script = build_add_script("MyApp", r"C:\app.exe", r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run");
        assert!(script.contains("MyApp"));
        assert!(script.contains("app.exe"));
    }

    #[test]
    fn test_build_reg_delete_script_contains_name() {
        let script = build_reg_delete_script("MyApp", r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run");
        assert!(script.contains("MyApp"));
        assert!(script.contains("delete"));
    }
}