//! Hosts 文件管理器 - 解析、序列化、profile、备份
//!
//! 功能：
//! - 解析 hosts 文件为结构化条目（启用/禁用/IP/域名/备注）
//! - 序列化回 hosts 格式
//! - 多环境 profile 管理（JSON 存储）
//! - 自动备份 + 列表恢复
//! - 管理员权限检测

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// debug 模式日志宏（release 模式自动移除）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 常量 ============

/// 系统 hosts 文件路径
const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

/// 自动备份保留份数
const MAX_BACKUPS: usize = 20;

// ============ 数据结构 ============

/// hosts 文件中的一条条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostsEntry {
    pub enabled: bool,
    pub ip: String,
    pub domains: Vec<String>,
    pub comment: String,
}

/// 解析后的完整 hosts 文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsFile {
    pub entries: Vec<HostsEntry>,
    pub raw_lines: Vec<String>,
    pub path: String,
}

/// Profile 元数据（列表展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub entry_count: usize,
    pub updated_at: String,
    pub is_default: bool,
}

/// Profile 完整数据（存储格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub entries: Vec<HostsEntry>,
    pub created_at: String,
    pub updated_at: String,
}

/// 备份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub timestamp: String,
    pub size: u64,
    pub path: String,
}

// ============ 解析与序列化 ============

/// 解析 hosts 文件内容为 HostsFile
///
/// 解析规则：
/// - 空行：跳过（不计入 entries，不计入 raw_lines）
/// - 纯注释行（# 后无 IP）：保留到 raw_lines
/// - 禁用条目（# IP domain...）：解析为 enabled=false
/// - 启用条目（IP domain...）：解析为 enabled=true
/// - 行内注释（IP domain # comment）：comment 字段存储
/// - 无法解析的行：保留到 raw_lines
pub fn parse_hosts(content: &str, path: &str) -> HostsFile {
    let mut entries = Vec::new();
    let mut raw_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // 空行跳过
        if trimmed.is_empty() {
            continue;
        }

        // 尝试解析为条目
        if let Some(entry) = parse_line(trimmed) {
            entries.push(entry);
        } else {
            // 无法解析，保留原样
            raw_lines.push(line.to_string());
        }
    }

    HostsFile {
        entries,
        raw_lines,
        path: path.to_string(),
    }
}

/// 解析单行，返回 None 表示无法解析（纯注释等）
fn parse_line(line: &str) -> Option<HostsEntry> {
    let line = line.trim();

    if line.is_empty() {
        return None;
    }

    let (content, enabled) = if line.starts_with('#') {
        // 去掉 # 前缀，检查剩余是否为有效条目
        let rest = line[1..].trim_start();
        if rest.is_empty() {
            // 纯注释行（# 后无内容）
            return None;
        }
        (rest, false)
    } else {
        (line, true)
    };

    // 提取行内注释
    let (main_part, comment) = if let Some(idx) = content.find('#') {
        let main = content[..idx].trim();
        let cmt = content[idx + 1..].trim();
        (main, cmt.to_string())
    } else {
        (content, String::new())
    };

    if main_part.is_empty() {
        return None;
    }

    let parts: Vec<&str> = main_part.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    // 第一个部分必须是 IP（简单验证：包含 . 或 :）
    let ip = parts[0];
    if !ip.contains('.') && !ip.contains(':') {
        // 不是 IP 格式，视为纯注释
        return None;
    }

    let domains: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    Some(HostsEntry {
        enabled,
        ip: ip.to_string(),
        domains,
        comment,
    })
}

/// 序列化 HostsFile 回 hosts 格式字符串
pub fn serialize_hosts(file: &HostsFile) -> String {
    let mut lines = Vec::new();

    // 先写 raw_lines（注释块等）
    for line in &file.raw_lines {
        lines.push(line.clone());
    }

    // 再写 entries
    for entry in &file.entries {
        let prefix = if entry.enabled { "" } else { "# " };
        let domains_str = entry.domains.join(" ");
        let comment_str = if entry.comment.is_empty() {
            String::new()
        } else {
            format!(" # {}", entry.comment)
        };

        if domains_str.is_empty() {
            lines.push(format!("{}{}{}", prefix, entry.ip, comment_str));
        } else {
            lines.push(format!("{}{} {}{}", prefix, entry.ip, domains_str, comment_str));
        }
    }

    lines.join("\n")
}

// ============ 路径辅助 ============

/// Profile 存储目录：%APPDATA%\com.dev.toolbox\hosts_profiles\
fn profiles_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    PathBuf::from(appdata).join("com.dev.toolbox").join("hosts_profiles")
}

/// 备份存储目录：%APPDATA%\com.dev.toolbox\hosts_backups\
fn backups_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    PathBuf::from(appdata).join("com.dev.toolbox").join("hosts_backups")
}

/// 确保目录存在
fn ensure_dir(dir: &PathBuf) -> Result<(), String> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(())
}

// ============ 管理员检测 ============

/// 检测当前进程是否以管理员权限运行
#[cfg(windows)]
pub fn is_admin() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut ret_len = 0u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );

        CloseHandle(token);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_admin() -> bool {
    false
}

// ============ 文件读写 ============

/// 读取系统 hosts 文件
pub fn read_hosts() -> Result<HostsFile, String> {
    let content = fs::read_to_string(HOSTS_PATH)
        .map_err(|e| format!("读取 hosts 文件失败: {}", e))?;
    Ok(parse_hosts(&content, HOSTS_PATH))
}

/// 原子写入 hosts 文件
/// 1. 自动备份当前 hosts
/// 2. 写入同目录临时文件
/// 3. rename 替换（同分区保证原子性）
pub fn save_hosts(entries: &[HostsEntry]) -> Result<(), String> {
    debug_log!("[hosts] save_hosts: entries={}", entries.len());

    // 1. 自动备份
    auto_backup()?;
    debug_log!("[hosts] save_hosts: auto_backup done");

    // 2. 读取当前 hosts（保留 raw_lines）
    let current = read_hosts().unwrap_or(HostsFile {
        entries: vec![],
        raw_lines: vec![],
        path: HOSTS_PATH.to_string(),
    });

    // 3. 构建新内容
    let new_file = HostsFile {
        entries: entries.to_vec(),
        raw_lines: current.raw_lines,
        path: HOSTS_PATH.to_string(),
    };
    let content = serialize_hosts(&new_file);

    // 4. 原子写入：写入临时文件 → rename
    let tmp_path = format!("{}.tmp", HOSTS_PATH);
    let mut file = fs::File::create(&tmp_path)
        .map_err(|e| {
            debug_log!("[hosts] save_hosts: create tmp failed: {}", e);
            format!("创建临时文件失败: {}", e)
        })?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("同步文件失败: {}", e))?;
    drop(file);

    fs::rename(&tmp_path, HOSTS_PATH)
        .map_err(|e| {
            debug_log!("[hosts] save_hosts: rename failed: {}", e);
            format!("替换 hosts 文件失败: {}", e)
        })?;

    debug_log!("[hosts] save_hosts: done");
    Ok(())
}

// ============ 备份管理 ============

/// 生成时间戳字符串：YYYYMMDD_HHMMSS（本地时间）
fn timestamp_str() -> String {
    let st = local_systemtime();
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond)
}

/// 获取当前本地时间（Windows）或 UTC（其他平台，仅用于编译兼容）
#[cfg(windows)]
fn local_systemtime() -> windows_sys::Win32::Foundation::SYSTEMTIME {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st); }
    st
}

#[cfg(not(windows))]
fn local_systemtime() -> LocalSt {
    // 非 Windows 平台 fallback，仅保证编译
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let remainder = secs % 86400;
    let (year, month, day) = days_to_ymd(days as i64);
    LocalSt {
        wYear: year as u16,
        wMonth: month as u16,
        wDay: day as u16,
        wHour: (remainder / 3600) as u16,
        wMinute: ((remainder % 3600) / 60) as u16,
        wSecond: (remainder % 60) as u16,
    }
}

#[cfg(not(windows))]
struct LocalSt {
    wYear: u16,
    wMonth: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
}

/// 将天数（从 1970-01-01）转为 (year, month, day)
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    // ponytail: 简单的算法，够用，不处理闰秒等边界
    let mut y = 1970i64;
    let mut d = days;

    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let yd = if leap { 366 } else { 365 };
        if d < yd {
            break;
        }
        d -= yd;
        y += 1;
    }

    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0;
    while d >= mdays[m] {
        d -= mdays[m];
        m += 1;
    }

    (y, (m + 1) as u32, (d + 1) as u32)
}

/// 自动备份当前 hosts 文件
pub fn auto_backup() -> Result<(), String> {
    ensure_dir(&backups_dir())?;

    // 读取当前 hosts
    if let Ok(content) = fs::read(HOSTS_PATH) {
        let filename = format!("hosts_{}", timestamp_str());
        let path = backups_dir().join(&filename);
        fs::write(&path, &content)
            .map_err(|e| format!("备份失败: {}", e))?;

        // 清理超过 MAX_BACKUPS 的旧备份
        cleanup_old_backups()?;
    }

    Ok(())
}

/// 清理旧备份，保留最近 MAX_BACKUPS 份
fn cleanup_old_backups() -> Result<(), String> {
    let dir = backups_dir();
    let mut backups: Vec<(PathBuf, SystemTime)> = fs::read_dir(&dir)
        .map_err(|e| format!("读取备份目录失败: {}", e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((path, mtime))
        })
        .collect();

    if backups.len() <= MAX_BACKUPS {
        return Ok(());
    }

    // 按 mtime 降序排序（最新在前）
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // 删除多余的
    for (path, _) in backups.iter().skip(MAX_BACKUPS) {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

/// 列出所有备份
pub fn list_backups() -> Result<Vec<BackupInfo>, String> {
    ensure_dir(&backups_dir())?;

    let mut backups: Vec<BackupInfo> = fs::read_dir(&backups_dir())
        .map_err(|e| format!("读取备份目录失败: {}", e))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let metadata = e.metadata().ok()?;
            let filename = path.file_name()?.to_string_lossy().to_string();
            let size = metadata.len();
            let mtime = metadata.modified().ok()?;
            let timestamp = format_systemtime(mtime);
            Some(BackupInfo {
                filename,
                timestamp,
                size,
                path: path.to_string_lossy().to_string(),
            })
        })
        .collect();

    // 按时间降序（最新在前）
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(backups)
}

/// 校验备份文件名（防路径遍历，与 timestamp_str() 生成的格式匹配）
fn validate_backup_filename(filename: &str) -> Result<(), String> {
    // 期望格式：hosts_YYYYMMDD_HHMMSS
    let rest = match filename.strip_prefix("hosts_") {
        Some(r) => r,
        None => return Err("非法备份文件名".to_string()),
    };
    // 剩余部分应为 8 位日期 + '_' + 6 位时间
    let bytes = rest.as_bytes();
    if bytes.len() != 15
        || !bytes[0..8].iter().all(|b| b.is_ascii_digit())
        || bytes[8] != b'_'
        || !bytes[9..15].iter().all(|b| b.is_ascii_digit())
    {
        return Err("非法备份文件名".to_string());
    }
    Ok(())
}

/// 预览备份内容
pub fn preview_backup(filename: &str) -> Result<String, String> {
    validate_backup_filename(filename)?;
    let path = backups_dir().join(filename);
    fs::read_to_string(&path)
        .map_err(|e| format!("读取备份失败: {}", e))
}

/// 恢复备份（恢复前再备份一次当前 hosts）
pub fn restore_backup(filename: &str) -> Result<(), String> {
    debug_log!("[hosts] restore_backup: filename={}", filename);
    validate_backup_filename(filename)?;
    // 恢复前备份当前
    auto_backup()?;

    let backup_path = backups_dir().join(filename);
    let content = fs::read(&backup_path)
        .map_err(|e| {
            debug_log!("[hosts] restore_backup: read backup failed: {}", e);
            format!("读取备份失败: {}", e)
        })?;

    // 原子写入
    let tmp_path = format!("{}.tmp", HOSTS_PATH);
    fs::write(&tmp_path, &content)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    fs::rename(&tmp_path, HOSTS_PATH)
        .map_err(|e| {
            debug_log!("[hosts] restore_backup: rename failed: {}", e);
            format!("替换 hosts 文件失败: {}", e)
        })?;

    debug_log!("[hosts] restore_backup: done");
    Ok(())
}

/// 删除指定备份
pub fn delete_backup(filename: &str) -> Result<(), String> {
    validate_backup_filename(filename)?;
    let path = backups_dir().join(filename);
    fs::remove_file(&path)
        .map_err(|e| format!("删除备份失败: {}", e))
}

/// 立即创建备份
pub fn create_backup() -> Result<BackupInfo, String> {
    ensure_dir(&backups_dir())?;

    let content = fs::read(HOSTS_PATH)
        .map_err(|e| format!("读取 hosts 失败: {}", e))?;
    let filename = format!("hosts_{}", timestamp_str());
    let path = backups_dir().join(&filename);
    fs::write(&path, &content)
        .map_err(|e| format!("备份失败: {}", e))?;

    let metadata = fs::metadata(&path)
        .map_err(|e| format!("读取备份信息失败: {}", e))?;

    Ok(BackupInfo {
        filename,
        timestamp: format_systemtime(metadata.modified().map_err(|e| e.to_string())?),
        size: metadata.len(),
        path: path.to_string_lossy().to_string(),
    })
}

/// 格式化 SystemTime 为本地时间可读字符串：YYYY-MM-DD HH:MM:SS
#[cfg(windows)]
fn format_systemtime(t: SystemTime) -> String {
    use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows_sys::Win32::Storage::FileSystem::FileTimeToLocalFileTime;
    use windows_sys::Win32::System::Time::FileTimeToSystemTime;

    let dur = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    // UNIX epoch(1970) 与 FILETIME epoch(1601) 偏移：11644473600 秒 = 116444736000000000 (100ns 单位)
    let intervals = dur.as_nanos() as u64 / 100 + 116_444_736_000_000_000;
    let utc_ft = FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    };
    let mut local_ft = utc_ft;
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        FileTimeToLocalFileTime(&utc_ft, &mut local_ft);
        FileTimeToSystemTime(&local_ft, &mut st);
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond)
}

#[cfg(not(windows))]
fn format_systemtime(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = (secs / 86400) as i64;
    let remainder = secs % 86400;
    let hour = remainder / 3600;
    let min = (remainder % 3600) / 60;
    let sec = remainder % 60;
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, min, sec)
}

// ============ Profile 管理 ============

/// 列出所有 profile（包含虚拟的"默认"项）
pub fn profile_list() -> Result<Vec<ProfileMeta>, String> {
    ensure_dir(&profiles_dir())?;

    let mut profiles: Vec<ProfileMeta> = vec![
        ProfileMeta {
            name: "默认".to_string(),
            entry_count: read_hosts().map(|f| f.entries.len()).unwrap_or(0),
            updated_at: String::new(),
            is_default: true,
        },
    ];

    // 读取 profiles_dir 下的所有 .json 文件
    if let Ok(entries) = fs::read_dir(profiles_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        profiles.push(ProfileMeta {
                            name: profile.name,
                            entry_count: profile.entries.len(),
                            updated_at: profile.updated_at,
                            is_default: false,
                        });
                    }
                }
            }
        }
    }

    Ok(profiles)
}

/// 校验 profile 名称（防止路径遍历，trust boundary input validation）
fn validate_profile_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("profile 名称不能为空".to_string());
    }
    if name == "默认" {
        return Ok(());
    }
    // is_alphanumeric() 已包含 CJK 等所有 Unicode 字母
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("profile 名称只能包含字母、数字、下划线、连字符和中文字符".to_string());
    }
    Ok(())
}

/// 加载指定 profile 的条目
pub fn profile_load(name: &str) -> Result<Vec<HostsEntry>, String> {
    debug_log!("[hosts] profile_load: name={}", name);
    validate_profile_name(name)?;
    if name == "默认" {
        return read_hosts().map(|f| f.entries);
    }

    let path = profiles_dir().join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .map_err(|e| {
            debug_log!("[hosts] profile_load: read failed: {}", e);
            format!("读取 profile 失败: {}", e)
        })?;
    let profile: Profile = serde_json::from_str(&content)
        .map_err(|e| format!("解析 profile 失败: {}", e))?;
    debug_log!("[hosts] profile_load: entries={}", profile.entries.len());
    Ok(profile.entries)
}

/// 保存 profile（已存在则覆盖，不存在则创建）
pub fn profile_save(name: &str, entries: &[HostsEntry]) -> Result<(), String> {
    debug_log!("[hosts] profile_save: name={}, entries={}", name, entries.len());
    validate_profile_name(name)?;
    if name == "默认" {
        return Err("默认 profile 不可保存".to_string());
    }

    ensure_dir(&profiles_dir())?;

    let now = format_systemtime(SystemTime::now());

    // 检查是否已存在（决定 created_at）
    let path = profiles_dir().join(format!("{}.json", name));
    let created_at = if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str::<Profile>(&content)
                .map(|p| p.created_at)
                .unwrap_or(now.clone())
        } else {
            now.clone()
        }
    } else {
        now.clone()
    };

    let profile = Profile {
        name: name.to_string(),
        entries: entries.to_vec(),
        created_at,
        updated_at: now,
    };

    let json = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("序列化 profile 失败: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| {
            debug_log!("[hosts] profile_save: write failed: {}", e);
            format!("写入 profile 失败: {}", e)
        })?;

    debug_log!("[hosts] profile_save: done");
    Ok(())
}

/// 删除 profile（默认不可删）
pub fn profile_delete(name: &str) -> Result<(), String> {
    validate_profile_name(name)?;
    if name == "默认" {
        return Err("默认 profile 不可删除".to_string());
    }

    let path = profiles_dir().join(format!("{}.json", name));
    if !path.exists() {
        return Err(format!("profile {} 不存在", name));
    }

    fs::remove_file(&path)
        .map_err(|e| format!("删除 profile 失败: {}", e))
}

/// 将 profile 写入系统 hosts（自动备份当前）
pub fn profile_apply(name: &str) -> Result<(), String> {
    debug_log!("[hosts] profile_apply: name={}", name);
    let entries = profile_load(name)?;
    save_hosts(&entries)?;
    debug_log!("[hosts] profile_apply: done");
    Ok(())
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn hosts_read() -> Result<HostsFile, String> {
    let f = read_hosts()?;
    debug_log!("[hosts] hosts_read: entries={}", f.entries.len());
    Ok(f)
}

#[tauri::command]
pub async fn hosts_save(entries: Vec<HostsEntry>) -> Result<(), String> {
    save_hosts(&entries)
}

#[tauri::command]
pub async fn hosts_check_admin() -> Result<bool, String> {
    Ok(is_admin())
}

#[tauri::command]
pub async fn hosts_list_backups() -> Result<Vec<BackupInfo>, String> {
    list_backups()
}

#[tauri::command]
pub async fn hosts_preview_backup(filename: String) -> Result<String, String> {
    preview_backup(&filename)
}

#[tauri::command]
pub async fn hosts_restore_backup(filename: String) -> Result<(), String> {
    restore_backup(&filename)
}

#[tauri::command]
pub async fn hosts_delete_backup(filename: String) -> Result<(), String> {
    delete_backup(&filename)
}

#[tauri::command]
pub async fn hosts_create_backup() -> Result<BackupInfo, String> {
    create_backup()
}

#[tauri::command]
pub async fn hosts_profile_list() -> Result<Vec<ProfileMeta>, String> {
    profile_list()
}

#[tauri::command]
pub async fn hosts_profile_load(name: String) -> Result<Vec<HostsEntry>, String> {
    profile_load(&name)
}

#[tauri::command]
pub async fn hosts_profile_save(name: String, entries: Vec<HostsEntry>) -> Result<(), String> {
    profile_save(&name, &entries)
}

#[tauri::command]
pub async fn hosts_profile_delete(name: String) -> Result<(), String> {
    profile_delete(&name)
}

#[tauri::command]
pub async fn hosts_profile_apply(name: String) -> Result<(), String> {
    profile_apply(&name)
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_entry() {
        let content = "127.0.0.1 localhost";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].enabled, true);
        assert_eq!(file.entries[0].ip, "127.0.0.1");
        assert_eq!(file.entries[0].domains, vec!["localhost"]);
        assert_eq!(file.entries[0].comment, "");
    }

    #[test]
    fn test_parse_disabled_entry() {
        let content = "# 192.168.1.1 myserver.local";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].enabled, false);
        assert_eq!(file.entries[0].ip, "192.168.1.1");
        assert_eq!(file.entries[0].domains, vec!["myserver.local"]);
    }

    #[test]
    fn test_parse_inline_comment() {
        let content = "127.0.0.1 localhost # 本地回环";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].comment, "本地回环");
    }

    #[test]
    fn test_parse_multiple_domains() {
        let content = "192.168.1.1 api.test.com web.test.com";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].domains, vec!["api.test.com", "web.test.com"]);
    }

    #[test]
    fn test_parse_pure_comment_preserved() {
        let content = "# This is a comment\n127.0.0.1 localhost";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.raw_lines.len(), 1);
        assert_eq!(file.raw_lines[0], "# This is a comment");
    }

    #[test]
    fn test_parse_empty_lines_skipped() {
        let content = "\n\n127.0.0.1 localhost\n\n";
        let file = parse_hosts(content, "");
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.raw_lines.len(), 0);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let content = "# Comment line\n127.0.0.1 localhost # local\n# 192.168.1.1 disabled.com\n192.168.1.1 a.com b.com";
        let file = parse_hosts(content, "");
        let serialized = serialize_hosts(&file);
        let reparsed = parse_hosts(&serialized, "");
        assert_eq!(file.entries, reparsed.entries);
    }

    #[test]
    fn test_serialize_disabled_entry() {
        let entry = HostsEntry {
            enabled: false,
            ip: "192.168.1.1".to_string(),
            domains: vec!["test.com".to_string()],
            comment: "测试".to_string(),
        };
        let file = HostsFile {
            entries: vec![entry],
            raw_lines: vec![],
            path: "".to_string(),
        };
        let serialized = serialize_hosts(&file);
        assert!(serialized.starts_with("# 192.168.1.1 test.com # 测试"));
    }

    #[test]
    fn test_timestamp_str_format() {
        let ts = timestamp_str();
        assert_eq!(ts.len(), 15);
        assert!(ts.contains('_'));
        // 格式 YYYYMMDD_HHMMSS
        assert_eq!(ts.split('_').count(), 2);
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        // 1970-01-01 = day 0
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_2024() {
        // 2024-01-01 ≈ day 19723
        let (y, _m, _d) = days_to_ymd(19723);
        assert_eq!(y, 2024);
    }

    #[test]
    fn test_validate_profile_name() {
        // 合法
        assert!(validate_profile_name("默认").is_ok());
        assert!(validate_profile_name("dev").is_ok());
        assert!(validate_profile_name("test-env").is_ok());
        assert!(validate_profile_name("test_env").is_ok());
        assert!(validate_profile_name("测试环境").is_ok());
        assert!(validate_profile_name("env123").is_ok());
        // 非法
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("../etc").is_err());
        assert!(validate_profile_name("a/b").is_err());
        assert!(validate_profile_name("a:b").is_err());
        assert!(validate_profile_name("a\\b").is_err());
        assert!(validate_profile_name("a*b").is_err());
        assert!(validate_profile_name("a b").is_err());
    }

    #[test]
    fn test_validate_backup_filename() {
        // 合法
        assert!(validate_backup_filename("hosts_20260721_143025").is_ok());
        assert!(validate_backup_filename("hosts_00000000_000000").is_ok());
        // 非法
        assert!(validate_backup_filename("").is_err());
        assert!(validate_backup_filename("hosts_20260721").is_err());       // 太短
        assert!(validate_backup_filename("hosts_20260721_14302").is_err()); // 少一位
        assert!(validate_backup_filename("hosts_2026AB21_143025").is_err()); // 非数字
        assert!(validate_backup_filename("hosts_20260721-143025").is_err()); // 分隔符错
        assert!(validate_backup_filename("../../../etc/passwd").is_err());
        assert!(validate_backup_filename("hosts_20260721_143025.txt").is_err()); // 多后缀
    }
}
