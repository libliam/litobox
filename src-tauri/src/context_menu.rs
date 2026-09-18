//! 右键菜单管理器 — 操作 Windows 注册表的 shell 项
//!
//! 功能：
//! - 列出文件/文件夹/桌面右键的 shell 菜单项
//! - 添加自定义右键菜单项（名称、命令、图标）
//! - 删除菜单项（自动备份到 .reg 文件）
//! - 备份/恢复注册表项
//!
//! 实现：通过 PowerShell 调用 reg 命令操作注册表，不引入新依赖。
//! 注册表路径（每个作用域读取多个来源）：
//! - 静态菜单项(shell)：HKCR\*\shell、HKCR\Directory\shell、HKCR\Directory\Background\shell
//! - COM 处理器(shellex)：各级 ContextMenuHandlers
//! - 通用位置：HKCR\AllFilesystemObjects、HKCR\Folder

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式日志
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 常量 ============

/// 右键菜单注册表来源 (scope, 短路径, 类型)
/// 类型: "shell" = 静态菜单项（默认值=显示名，command 子键=命令）
///       "shellex" = COM 处理器（默认值=CLSID，需解析出名称与 DLL）
const REG_SOURCES: &[(&str, &str, &str)] = &[
    ("file", r"HKCR\*\shell", "shell"),
    ("file", r"HKCR\*\shellex\ContextMenuHandlers", "shellex"),
    ("file", r"HKCR\AllFilesystemObjects\shell", "shell"),
    (
        "file",
        r"HKCR\AllFilesystemObjects\shellex\ContextMenuHandlers",
        "shellex",
    ),
    ("folder", r"HKCR\Directory\shell", "shell"),
    (
        "folder",
        r"HKCR\Directory\shellex\ContextMenuHandlers",
        "shellex",
    ),
    ("folder", r"HKCR\Folder\shellex\ContextMenuHandlers", "shellex"),
    ("folder", r"HKCR\AllFilesystemObjects\shell", "shell"),
    (
        "folder",
        r"HKCR\AllFilesystemObjects\shellex\ContextMenuHandlers",
        "shellex",
    ),
    ("desktop", r"HKCR\Directory\Background\shell", "shell"),
    (
        "desktop",
        r"HKCR\Directory\Background\shellex\ContextMenuHandlers",
        "shellex",
    ),
];

/// 各作用域的静态菜单项基础路径（用于「添加项」）
fn shell_path_for_scope(scope: &str) -> Option<&'static str> {
    REG_SOURCES
        .iter()
        .find(|(s, _, kind)| *s == scope && *kind == "shell")
        .map(|(_, p, _)| *p)
}

/// 来源的友好名称，用于列表展示
fn source_label(path: &str) -> &'static str {
    match path {
        r"HKCR\*\shell" => "文件菜单项",
        r"HKCR\*\shellex\ContextMenuHandlers" => "文件 COM 处理器",
        r"HKCR\AllFilesystemObjects\shell" => "所有文件系统对象",
        r"HKCR\AllFilesystemObjects\shellex\ContextMenuHandlers" => "所有文件系统对象 COM",
        r"HKCR\Directory\shell" => "文件夹菜单项",
        r"HKCR\Directory\shellex\ContextMenuHandlers" => "文件夹 COM 处理器",
        r"HKCR\Folder\shellex\ContextMenuHandlers" => "文件夹(Folder) COM 处理器",
        r"HKCR\Directory\Background\shell" => "桌面菜单项",
        r"HKCR\Directory\Background\shellex\ContextMenuHandlers" => "桌面 COM 处理器",
        _ => "其他",
    }
}

/// reg query 输出时会把短根键展开为长根键，匹配输出需要用长路径
fn short_to_long_path(short: &str) -> String {
    short
        .replace("HKCR\\", "HKEY_CLASSES_ROOT\\")
        .replace("HKCU\\", "HKEY_CURRENT_USER\\")
        .replace("HKLM\\", "HKEY_LOCAL_MACHINE\\")
        .replace("HKU\\", "HKEY_USERS\\")
        .replace("HKCC\\", "HKEY_CURRENT_CONFIG\\")
}

/// 备份目录名
const BACKUP_DIR_NAME: &str = "context_menu_backups";

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenuItem {
    pub scope: String,        // "file" | "folder" | "desktop"
    pub key_name: String,     // 注册表键名（保留原始空格）
    pub display_name: String, // 显示名称
    pub command: String,      // 执行命令（静态项）
    pub icon: String,         // 图标路径
    pub kind: String,         // "shell" | "shellex"
    pub source_path: String,  // 来源基础路径（短路径）
    pub source_label: String, // 来源友好名称
    pub item_path: String,    // 该项完整注册表路径（用于删除/唯一标识）
    pub dll_path: String,     // COM 处理器 DLL（shellex 项）
    pub is_custom: bool,      // 是否为第三方/用户自定义项
    pub is_enabled: bool,     // 是否启用（未被 Blocked 列表屏蔽）
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextMenuResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub timestamp_ms: u128,
    pub scope: String,
    pub size: u64,
    pub path: String,
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
    let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
    let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
    if !output.status.success() {
        return Err(format!("PowerShell 错误: {}", stderr.trim()));
    }
    Ok(stdout.into_owned())
}

fn run_reg(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("reg");
    cmd.args(args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("reg 执行失败: {}", e))?;
    let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
    let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
    if !output.status.success() {
        return Err(format!("reg 错误: {}", stderr.trim()));
    }
    Ok(stdout.into_owned())
}

// ============ 工具函数 ============

fn get_app_data_dir() -> PathBuf {
    if let Some(local) = dirs::data_local_dir() {
        let dir = local.join("com.dev.toolbox").join(BACKUP_DIR_NAME);
        let _ = fs::create_dir_all(&dir);
        dir
    } else {
        PathBuf::from(".")
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// 白名单校验：只允许删除已注册来源下的直接子键
fn is_allowed_item_path(item_path: &str) -> bool {
    REG_SOURCES.iter().any(|(_, base, _)| {
        match item_path.strip_prefix(&format!("{}\\", base)) {
            Some(rest) => !rest.is_empty() && !rest.contains('\\'),
            None => false,
        }
    })
}

/// 生成安全的文件名片段
fn sanitize_filename(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    if out.trim_matches('_').is_empty() {
        out = "item".to_string();
    }
    if out.chars().count() > 30 {
        out = out.chars().take(30).collect();
    }
    out
}

/// 判断是否为系统内置项（常见系统项名称）
fn is_system_item(key_name: &str) -> bool {
    let system_items = [
        "open",
        "Open",
        "openas",
        "OpenWith",
        "print",
        "Print",
        "edit",
        "Edit",
        "explore",
        "Explore",
        "find",
        "Find",
        "runas",
        "RunAs",
        "shellnew",
        "ShellNew",
        "pintohome",
        "PinToHome",
        "pintostartscreen",
        "PinToStartScreen",
        "windows.storage",
        "Windows.Storage",
    ];
    let lower = key_name.to_lowercase();
    system_items.iter().any(|s| s.to_lowercase() == lower)
}

/// 查询某个 scope 下的所有菜单项（静态菜单项 + COM 处理器）
fn list_scope_items(scope: &str) -> Vec<ContextMenuItem> {
    let blocked = blocked_clsids();
    let mut items = Vec::new();

    for (s, reg_path, kind) in REG_SOURCES.iter() {
        if *s != scope {
            continue;
        }
        items.extend(list_source_items(scope, reg_path, kind, &blocked));
    }

    items
}

/// 读取单个来源基础路径下的所有直接子键
fn list_source_items(
    scope: &str,
    reg_path: &str,
    kind: &str,
    blocked: &[String],
) -> Vec<ContextMenuItem> {
    let mut items = Vec::new();

    let output = match run_reg(&["query", reg_path]) {
        Ok(o) => o,
        Err(e) => {
            debug_log!("[context_menu] 读取 {} 失败: {}", reg_path, e);
            return items;
        }
    };

    // reg 输出会把 HKCR 展开为 HKEY_CLASSES_ROOT，用长路径匹配
    let long_path = short_to_long_path(reg_path);

    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with(&long_path) || line.len() <= long_path.len() {
            continue;
        }
        // 子键名可能带前导空格（如 " FileSyncEx"），需保留原样用于删除
        let key_name = line[long_path.len()..].trim_start_matches('\\').to_string();
        if key_name.trim().is_empty() || key_name.contains('\\') {
            continue;
        }

        let item_path = format!("{}\\{}", reg_path, key_name);
        // 默认值：静态项为显示名，COM 处理器为 CLSID
        let default_value = query_value(reg_path, &key_name, "").unwrap_or_default();
        let mut command = String::new();
        let mut icon = String::new();
        let mut dll_path = String::new();

        let (display_name, is_custom) = if kind == "shellex" {
            let clsid = default_value.trim().to_string();
            let (name, dll) = if clsid.starts_with('{') {
                resolve_clsid(&clsid)
            } else {
                (String::new(), String::new())
            };
            dll_path = dll;

            let display = if !name.trim().is_empty() {
                name.trim().to_string()
            } else if !clsid.is_empty() {
                clsid
            } else {
                key_name.trim().to_string()
            };
            let is_custom = match dll_is_system(&dll_path) {
                Some(sys) => !sys,
                None => !is_system_item(key_name.trim()),
            };
            (display, is_custom)
        } else {
            let display = if !default_value.trim().is_empty() {
                default_value.trim().to_string()
            } else {
                let muiverb = query_value(reg_path, &key_name, "MUIVerb").unwrap_or_default();
                if muiverb.trim().is_empty() {
                    key_name.trim().to_string()
                } else {
                    muiverb.trim().to_string()
                }
            };
            command = query_value(&item_path, "command", "").unwrap_or_default();
            icon = query_value(reg_path, &key_name, "Icon").unwrap_or_default();
            (display, !is_system_item(key_name.trim()))
        };

        // 被 Windows Blocked 列表屏蔽的 COM 处理器标记为未启用
        let is_enabled = if kind == "shellex" {
            let clsid = default_value.trim();
            !clsid.starts_with('{') || !blocked.iter().any(|b| b.eq_ignore_ascii_case(clsid))
        } else {
            true
        };

        items.push(ContextMenuItem {
            scope: scope.to_string(),
            key_name,
            display_name,
            command,
            icon,
            kind: kind.to_string(),
            source_path: reg_path.to_string(),
            source_label: source_label(reg_path).to_string(),
            item_path,
            dll_path,
            is_custom,
            is_enabled,
        });
    }

    items
}

/// 解析 CLSID → (显示名称, DLL 路径)
fn resolve_clsid(clsid: &str) -> (String, String) {
    let name = query_value(r"HKCR\CLSID", clsid, "").unwrap_or_default();
    let dll = query_value(&format!(r"HKCR\CLSID\{}", clsid), "InprocServer32", "")
        .unwrap_or_default();
    (name, dll)
}

/// 根据 DLL 路径判断是否系统组件：Some(true)=系统, Some(false)=第三方, None=未知
fn dll_is_system(dll: &str) -> Option<bool> {
    let d = dll.trim().to_lowercase();
    if d.is_empty() {
        return None;
    }
    if d.contains("%systemroot%") || d.contains("%windir%") {
        return Some(true);
    }
    if d.contains(r"\windows\system32") || d.contains(r"\windows\syswow64") {
        return Some(true);
    }
    Some(false)
}

/// 读取被 Windows 屏蔽（Blocked）的 shell 扩展 CLSID 列表
fn blocked_clsids() -> Vec<String> {
    let mut result = Vec::new();
    let paths = [
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Blocked",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Blocked",
    ];
    for path in paths {
        let output = match run_reg(&["query", path]) {
            Ok(o) => o,
            Err(e) => {
                debug_log!("[context_menu] 读取 Blocked 列表 {} 失败: {}", path, e);
                continue;
            }
        };
        let long_path = short_to_long_path(path);
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(&long_path) {
                continue;
            }
            // 值行：CLSID  REG_SZ  名称
            if let Some(ws) = line.find(char::is_whitespace) {
                let name = line[..ws].trim();
                if name.starts_with('{') {
                    result.push(name.to_string());
                }
            }
        }
    }
    result
}

/// 查询注册表值
fn query_value(base_path: &str, key: &str, value_name: &str) -> Result<String, String> {
    let full_path = format!("{}\\{}", base_path, key);
    let args = if value_name.is_empty() {
        vec!["query", full_path.as_str(), "/ve"]
    } else {
        vec!["query", full_path.as_str(), "/v", value_name]
    };

    let output = run_reg(&args)?;

    // reg 输出路径行用长路径，匹配时也要转长路径
    let long_full_path = short_to_long_path(&full_path);

    // 解析输出：格式为 "    名称    类型    数据"
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(&long_full_path) {
            continue;
        }
        // 跳过标题行（中文/英文）
        if (line.contains("名称") && line.contains("类型") && line.contains("数据"))
            || (line.contains("Name") && line.contains("Type") && line.contains("Data"))
        {
            continue;
        }
        // 手动按空白分段：名称 [空白+] 类型 [空白+] 数据（数据可含空格）
        if let Some(pos1) = line.find(char::is_whitespace) {
            let rest = line[pos1..].trim_start();
            if let Some(pos2) = rest.find(char::is_whitespace) {
                let data = rest[pos2..].trim_start().to_string();
                return Ok(data);
            }
        }
    }

    Ok(String::new())
}

/// 导出单个注册表路径到 .reg 文件
fn export_to_file(reg_path: &str, scope: &str, label: &str) -> Result<BackupInfo, String> {
    let backup_dir = get_app_data_dir();
    let timestamp = now_ms();
    let filename = format!("{}_{}_{}.reg", scope, sanitize_filename(label), timestamp);
    let backup_path = backup_dir.join(&filename);

    run_reg(&["export", reg_path, backup_path.to_str().unwrap(), "/y"])?;

    let size = fs::metadata(&backup_path).map(|m| m.len()).unwrap_or(0);
    Ok(BackupInfo {
        filename,
        timestamp_ms: timestamp,
        scope: scope.to_string(),
        size,
        path: backup_path.to_string_lossy().to_string(),
    })
}

/// 备份单个菜单项（删除前精确备份该项）
fn backup_item(scope: &str, item_path: &str, label: &str) -> Result<BackupInfo, String> {
    export_to_file(item_path, scope, label)
}

/// 备份某个 scope 的全部来源，返回各自生成的备份文件
fn backup_scope(scope: &str) -> Result<Vec<BackupInfo>, String> {
    let sources: Vec<&str> = REG_SOURCES
        .iter()
        .filter(|(s, _, _)| *s == scope)
        .map(|(_, p, _)| *p)
        .collect();
    if sources.is_empty() {
        return Err("无效的 scope".to_string());
    }

    let mut backups = Vec::new();
    for path in sources {
        match export_to_file(path, scope, path) {
            Ok(b) => backups.push(b),
            Err(e) => debug_log!("[context_menu] 导出 {} 失败: {}", path, e),
        }
    }
    Ok(backups)
}

// ============ Tauri 命令 ============

#[tauri::command(rename_all = "snake_case")]
pub fn cm_list_items(scope: String) -> Result<Vec<ContextMenuItem>, String> {
    debug_log!("[context_menu] 列出 {} 右键菜单项", scope);
    Ok(list_scope_items(&scope))
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_add_item(
    scope: String,
    key_name: String,
    display_name: String,
    command: String,
    icon: String,
) -> Result<ContextMenuResult, String> {
    debug_log!("[context_menu] 添加项: {} -> {}", scope, key_name);

    let reg_path = shell_path_for_scope(&scope).ok_or("无效的 scope")?;

    if key_name.trim().is_empty() {
        return Err("键名不能为空".to_string());
    }
    if display_name.trim().is_empty() {
        return Err("显示名称不能为空".to_string());
    }
    if command.trim().is_empty() {
        return Err("命令不能为空".to_string());
    }

    let item_path = format!("{}\\{}", reg_path, key_name.trim());

    // 创建项并设置默认值（显示名称）
    run_reg(&[
        "add",
        item_path.as_str(),
        "/ve",
        "/d",
        display_name.trim(),
        "/f",
    ])?;

    // 设置图标（可选）
    if !icon.trim().is_empty() {
        let _ = run_reg(&[
            "add",
            item_path.as_str(),
            "/v",
            "Icon",
            "/d",
            icon.trim(),
            "/f",
        ]);
    }

    // 创建 command 子键并设置命令
    let command_path = format!("{}\\command", item_path);
    run_reg(&[
        "add",
        command_path.as_str(),
        "/ve",
        "/d",
        command.trim(),
        "/f",
    ])?;

    Ok(ContextMenuResult {
        success: true,
        message: "添加成功".to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_delete_item(
    scope: String,
    item_path: String,
    display_name: String,
) -> Result<ContextMenuResult, String> {
    debug_log!("[context_menu] 删除项: {}", item_path);

    // 白名单校验，防止误删任意注册表项
    if !is_allowed_item_path(&item_path) {
        return Err(format!("不允许的注册表路径: {}", item_path));
    }

    // 删除前精确备份该项
    let backup = backup_item(&scope, &item_path, &display_name)?;
    debug_log!("[context_menu] 已备份: {}", backup.path);

    run_reg(&["delete", item_path.as_str(), "/f"])?;

    Ok(ContextMenuResult {
        success: true,
        message: format!("删除成功，已备份为 {}", backup.filename),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_backup(scope: String) -> Result<Vec<BackupInfo>, String> {
    debug_log!("[context_menu] 备份: {}", scope);
    backup_scope(&scope)
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_restore(backup_path: String) -> Result<ContextMenuResult, String> {
    debug_log!("[context_menu] 恢复: {}", backup_path);

    run_reg(&["import", backup_path.as_str()])?;

    Ok(ContextMenuResult {
        success: true,
        message: "恢复成功".to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_list_backups() -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_app_data_dir();
    let mut backups = Vec::new();

    if let Ok(entries) = fs::read_dir(&backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "reg").unwrap_or(false) {
                let filename = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                let metadata = entry.metadata().ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        t.duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis())
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);

                // 从文件名解析 scope
                let scope = filename
                    .split('_')
                    .next()
                    .unwrap_or("unknown")
                    .to_string();

                backups.push(BackupInfo {
                    filename,
                    timestamp_ms: modified,
                    scope,
                    size,
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    backups.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    Ok(backups)
}

#[tauri::command(rename_all = "snake_case")]
pub fn cm_delete_backup(filename: String) -> Result<ContextMenuResult, String> {
    let backup_dir = get_app_data_dir();
    let path = backup_dir.join(&filename);
    fs::remove_file(&path).map_err(|e| format!("删除备份失败: {}", e))?;
    Ok(ContextMenuResult {
        success: true,
        message: "备份已删除".to_string(),
    })
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_system_item() {
        assert!(is_system_item("open"));
        assert!(is_system_item("Open"));
        assert!(is_system_item("print"));
        assert!(!is_system_item("MyCustomItem"));
        assert!(!is_system_item("打开我的工具"));
    }

    #[test]
    fn test_shell_path_for_scope() {
        assert_eq!(shell_path_for_scope("file"), Some(r"HKCR\*\shell"));
        assert_eq!(shell_path_for_scope("folder"), Some(r"HKCR\Directory\shell"));
        assert_eq!(
            shell_path_for_scope("desktop"),
            Some(r"HKCR\Directory\Background\shell")
        );
        assert_eq!(shell_path_for_scope("invalid"), None);
    }

    #[test]
    fn test_is_allowed_item_path() {
        assert!(is_allowed_item_path(r"HKCR\*\shell\MyTool"));
        assert!(is_allowed_item_path(r"HKCR\*\shellex\ContextMenuHandlers\7-Zip"));
        assert!(is_allowed_item_path(
            r"HKCR\Directory\Background\shellex\ContextMenuHandlers\ACE"
        ));
        // 不允许嵌套子键
        assert!(!is_allowed_item_path(r"HKCR\*\shell\MyTool\command"));
        // 不允许来源之外的路径
        assert!(!is_allowed_item_path(r"HKCR\CLSID\{0000-0000}"));
        // 不允许空子键
        assert!(!is_allowed_item_path(r"HKCR\*\shell\"));
    }

    #[test]
    fn test_dll_is_system() {
        assert_eq!(dll_is_system(r"%SystemRoot%\system32\shell32.dll"), Some(true));
        assert_eq!(dll_is_system(r"C:\Windows\System32\ntshrui.dll"), Some(true));
        assert_eq!(
            dll_is_system(r"C:\Program Files\7-Zip\7-zip.dll"),
            Some(false)
        );
        assert_eq!(dll_is_system(""), None);
    }
}
