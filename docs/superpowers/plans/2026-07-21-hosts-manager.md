# Hosts 文件管理器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 Hosts 文件管理器，提供表格化编辑、多环境 profile 切换、自动备份恢复功能。

**Architecture:** 后端新增 `hosts_manager.rs` 模块（hosts 解析/序列化 + profile CRUD + 备份管理 + 管理员检测），前端新增 `HostsView.vue`（3 Tab：编辑/Profile/备份）。Profile 和备份以 JSON/原文件存储在 `%APPDATA%\com.dev.toolbox\` 下，与 SQLite 分离。

**Tech Stack:** Rust（windows-sys 管理员检测 + std::fs 文件操作）、Vue 3 + Element Plus（表格 + Tab）、Tauri 2.0 命令。

---

## 文件结构

| 文件 | 职责 | 操作 |
|------|------|------|
| `src-tauri/src/hosts_manager.rs` | hosts 解析/序列化、profile CRUD、备份管理、管理员检测、13 个 Tauri 命令 | 新建 |
| `src-tauri/src/main.rs` | 注册 `mod hosts_manager` + 13 个命令 | 修改 |
| `src-tauri/Cargo.toml` | 添加 `Win32_Security` feature | 修改 |
| `src/views/HostsView.vue` | 3 Tab 页面（编辑/Profile/备份） | 新建 |
| `src/store/index.ts` | TOOL_LIST 添加 hostsManager 条目 | 修改 |
| `src/App.vue` | import + toolComponentMap 注册 | 修改 |
| `package.json` | 版本 5.8.0 → 5.9.0 | 修改 |
| `src-tauri/tauri.conf.json` | 版本 5.8.0 → 5.9.0 | 修改 |
| `src-tauri/Cargo.toml` | 版本 5.8.0 → 5.9.0 | 修改 |
| `README.md` | V5.9 功能记录 | 修改 |

---

## Task 1: 添加 Win32_Security feature 到 Cargo.toml

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 windows-sys features 中添加 Win32_Security**

打开 `d:\work\codes\litobox\src-tauri\Cargo.toml`，找到 `[target.'cfg(windows)'.dependencies]` 块（约第 44 行），在 features 数组中添加 `"Win32_Security",`。

修改后的完整块：

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Graphics_Gdi",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_Security",
] }
```

- [ ] **Step 2: 验证依赖能正常拉取**

Run: `cd src-tauri; cargo check`
Expected: 编译通过（无错误，可能有 unused warning）。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(deps): 添加 Win32_Security feature 用于管理员权限检测"
```

---

## Task 2: 实现 hosts 解析与序列化核心 + 单元测试

**Files:**
- Create: `src-tauri/src/hosts_manager.rs`
- Modify: `src-tauri/src/main.rs`（添加 `mod hosts_manager;`）
- Test: `src-tauri/src/hosts_manager.rs` 内 `#[cfg(test)] mod tests`

- [ ] **Step 1: 创建 hosts_manager.rs 并实现数据结构 + 解析 + 序列化**

创建 `d:\work\codes\litobox\src-tauri\src\hosts_manager.rs`：

```rust
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
use std::time::SystemTime;

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
}
```

- [ ] **Step 2: 在 main.rs 中注册模块**

打开 `d:\work\codes\litobox\src-tauri\src\main.rs`，在第 21 行 `mod hotkey_data;` 之后添加：

```rust
mod hosts_manager;
```

- [ ] **Step 3: 运行测试验证**

Run: `cd src-tauri; cargo test hosts_manager::tests -- --nocapture`
Expected: 8 个测试全部通过。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/hosts_manager.rs src-tauri/src/main.rs
git commit -m "feat(hosts): 新增 hosts 解析与序列化核心模块 + 8 个单元测试"
```

---

## Task 3: 实现管理员检测、文件读写、原子写入、自动备份

**Files:**
- Modify: `src-tauri/src/hosts_manager.rs`（追加函数）
- Test: `src-tauri/src/hosts_manager.rs` 内 tests 模块追加测试

- [ ] **Step 1: 在 hosts_manager.rs 末尾（tests 模块之前）追加以下函数**

```rust
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
    // 1. 自动备份
    auto_backup()?;

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
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("同步文件失败: {}", e))?;
    drop(file);

    fs::rename(&tmp_path, HOSTS_PATH)
        .map_err(|e| format!("替换 hosts 文件失败: {}", e))?;

    Ok(())
}

// ============ 备份管理 ============

/// 生成时间戳字符串：YYYYMMDD_HHMMSS
fn timestamp_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // 简单的时间转换（ponytail: 不引入 chrono，手动计算够用）
    let days = secs / 86400;
    let remainder = secs % 86400;
    let hour = remainder / 3600;
    let min = (remainder % 3600) / 60;
    let sec = remainder % 60;

    // 从 1970-01-01 计算年月日
    let (year, month, day) = days_to_ymd(days as i64);

    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", year, month, day, hour, min, sec)
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

/// 预览备份内容
pub fn preview_backup(filename: &str) -> Result<String, String> {
    let path = backups_dir().join(filename);
    fs::read_to_string(&path)
        .map_err(|e| format!("读取备份失败: {}", e))
}

/// 恢复备份（恢复前再备份一次当前 hosts）
pub fn restore_backup(filename: &str) -> Result<(), String> {
    // 恢复前备份当前
    auto_backup()?;

    let backup_path = backups_dir().join(filename);
    let content = fs::read(&backup_path)
        .map_err(|e| format!("读取备份失败: {}", e))?;

    // 原子写入
    let tmp_path = format!("{}.tmp", HOSTS_PATH);
    fs::write(&tmp_path, &content)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    fs::rename(&tmp_path, HOSTS_PATH)
        .map_err(|e| format!("替换 hosts 文件失败: {}", e))?;

    Ok(())
}

/// 删除指定备份
pub fn delete_backup(filename: &str) -> Result<(), String> {
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

/// 格式化 SystemTime 为可读字符串
fn format_systemtime(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64;
    let remainder = secs % 86400;
    let hour = remainder / 3600;
    let min = (remainder % 3600) / 60;
    let sec = remainder % 60;
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, min, sec)
}
```

- [ ] **Step 2: 追加测试到 tests 模块**

在 `#[cfg(test)] mod tests {` 内追加：

```rust
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
```

- [ ] **Step 3: 运行测试**

Run: `cd src-tauri; cargo test hosts_manager::tests -- --nocapture`
Expected: 11 个测试全部通过。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/hosts_manager.rs
git commit -m "feat(hosts): 实现管理员检测、文件读写、原子写入、自动备份"
```

---

## Task 4: 实现 profile 管理 + Tauri 命令注册

**Files:**
- Modify: `src-tauri/src/hosts_manager.rs`（追加 profile 函数 + Tauri 命令）
- Modify: `src-tauri/src/main.rs`（注册 13 个命令）

- [ ] **Step 1: 在 hosts_manager.rs 追加 profile 管理函数**

在 `format_systemtime` 函数之后、`#[cfg(test)]` 之前追加：

```rust
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

/// 加载指定 profile 的条目
pub fn profile_load(name: &str) -> Result<Vec<HostsEntry>, String> {
    if name == "默认" {
        return read_hosts().map(|f| f.entries);
    }

    let path = profiles_dir().join(format!("{}.json", name));
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 profile 失败: {}", e))?;
    let profile: Profile = serde_json::from_str(&content)
        .map_err(|e| format!("解析 profile 失败: {}", e))?;
    Ok(profile.entries)
}

/// 保存 profile（已存在则覆盖，不存在则创建）
pub fn profile_save(name: &str, entries: &[HostsEntry]) -> Result<(), String> {
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
        .map_err(|e| format!("写入 profile 失败: {}", e))?;

    Ok(())
}

/// 删除 profile（默认不可删）
pub fn profile_delete(name: &str) -> Result<(), String> {
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
    let entries = profile_load(name)?;
    save_hosts(&entries)
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn hosts_read() -> Result<HostsFile, String> {
    read_hosts()
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
```

- [ ] **Step 2: 在 main.rs 注册命令**

打开 `d:\work\codes\litobox\src-tauri\src\main.rs`，在 `hotkey_probe::hotkey_probe_export_csv,` 之后（约第 193 行）追加：

```rust
            // Hosts 文件管理器命令
            hosts_manager::hosts_read,
            hosts_manager::hosts_save,
            hosts_manager::hosts_check_admin,
            hosts_manager::hosts_list_backups,
            hosts_manager::hosts_preview_backup,
            hosts_manager::hosts_restore_backup,
            hosts_manager::hosts_delete_backup,
            hosts_manager::hosts_create_backup,
            hosts_manager::hosts_profile_list,
            hosts_manager::hosts_profile_load,
            hosts_manager::hosts_profile_save,
            hosts_manager::hosts_profile_delete,
            hosts_manager::hosts_profile_apply,
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri; cargo check`
Expected: 编译通过，0 错误。

- [ ] **Step 4: 运行所有测试**

Run: `cd src-tauri; cargo test hosts_manager::tests -- --nocapture`
Expected: 11 个测试全部通过。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/hosts_manager.rs src-tauri/src/main.rs
git commit -m "feat(hosts): 实现 profile 管理 + 13 个 Tauri 命令注册"
```

---

## Task 5: 实现前端 HostsView.vue - Tab 1 编辑

**Files:**
- Create: `src/views/HostsView.vue`

- [ ] **Step 1: 创建 HostsView.vue，实现 Tab 1（编辑）**

创建 `d:\work\codes\litobox\src\views\HostsView.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- admin-banner -->
    <div v-if="!isAdmin" class="admin-banner">
      <span class="admin-icon">🛡️</span>
      编辑 hosts 文件需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- Tab 栏（sticky） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="hosts-tabs">
        <el-tab-pane label="Hosts 编辑" name="editor" />
        <el-tab-pane label="Profile 管理" name="profiles" />
        <el-tab-pane label="备份恢复" name="backups" />
      </el-tabs>
    </div>

    <!-- Tab 1: 编辑 -->
    <div v-if="activeTab === 'editor'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Hosts 条目 ({{ entries.length }} 条，启用 {{ enabledCount }} 条)</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索 IP/域名/备注..." style="width: 200px" clearable />
          <el-button size="small" @click="loadHosts" :loading="loading">刷新</el-button>
          <el-button type="primary" size="small" @click="addEntry">添加条目</el-button>
          <el-button type="success" size="small" :disabled="!isAdmin" :loading="saving" @click="saveHosts">保存</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="filteredEntries" border size="small" max-height="600" style="width: 100%" v-loading="loading">
          <el-table-column label="启用" width="70">
            <template #default="{ row }">
              <el-checkbox v-model="row.enabled" />
            </template>
          </el-table-column>
          <el-table-column label="IP 地址" width="180">
            <template #default="{ row }">
              <el-input v-model="row.ip" size="small" placeholder="127.0.0.1" />
            </template>
          </el-table-column>
          <el-table-column label="域名" min-width="280">
            <template #default="{ row }">
              <el-input
                v-model="row.domainsText"
                size="small"
                type="textarea"
                :autosize="{ minRows: 1, maxRows: 3 }"
                placeholder="example.com api.example.com"
              />
            </template>
          </el-table-column>
          <el-table-column label="备注" width="200">
            <template #default="{ row }">
              <el-input v-model="row.comment" size="small" placeholder="备注（可选）" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80" fixed="right">
            <template #default="{ $index }">
              <el-button type="danger" size="small" link @click="removeEntry($index)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Tab 2: Profile 管理（后续 Task 6 实现） -->
    <div v-if="activeTab === 'profiles'" class="tool-card">
      <div class="card-body"><el-empty description="Profile 管理功能开发中..." /></div>
    </div>

    <!-- Tab 3: 备份恢复（后续 Task 6 实现） -->
    <div v-if="activeTab === 'backups'" class="tool-card">
      <div class="card-body"><el-empty description="备份恢复功能开发中..." /></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('editor')

// ============ 数据 ============
interface HostsEntry {
  enabled: boolean
  ip: string
  domains: string[]
  comment: string
  // 前端编辑用：域名文本（空格分隔）
  domainsText: string
}

const entries = ref<HostsEntry[]>([])
const isAdmin = ref(false)
const loading = ref(false)
const saving = ref(false)
const searchQuery = ref('')

// ============ 计算属性 ============
const enabledCount = computed(() => entries.value.filter(e => e.enabled).length)

const filteredEntries = computed(() => {
  if (!searchQuery.value) return entries.value
  const kw = searchQuery.value.toLowerCase()
  return entries.value.filter(e =>
    e.ip.toLowerCase().includes(kw) ||
    e.domainsText.toLowerCase().includes(kw) ||
    e.comment.toLowerCase().includes(kw)
  )
})

// ============ 方法 ============
async function checkAdmin() {
  try {
    isAdmin.value = await invoke<boolean>('hosts_check_admin')
  } catch (e) {
    console.error('检测管理员权限失败:', e)
    isAdmin.value = false
  }
}

async function loadHosts() {
  loading.value = true
  try {
    const file = await invoke<{ entries: HostsEntry[], raw_lines: string[], path: string }>('hosts_read')
    // 转换：domains 数组 → domainsText 字符串
    entries.value = file.entries.map(e => ({
      enabled: e.enabled,
      ip: e.ip,
      domains: e.domains || [],
      comment: e.comment || '',
      domainsText: (e.domains || []).join(' ')
    }))
  } catch (e) {
    ElMessage.error(`读取 hosts 失败: ${e}`)
  } finally {
    loading.value = false
  }
}

function addEntry() {
  entries.value.push({
    enabled: true,
    ip: '',
    domains: [],
    comment: '',
    domainsText: ''
  })
}

function removeEntry(index: number) {
  entries.value.splice(index, 1)
}

async function saveHosts() {
  if (!isAdmin.value) {
    ElMessage.warning('需要管理员权限才能保存')
    return
  }

  // 转换：domainsText → domains 数组
  const payload = entries.value.map(e => ({
    enabled: e.enabled,
    ip: e.ip,
    domains: e.domainsText.split(/\s+/).filter(s => s.length > 0),
    comment: e.comment
  }))

  saving.value = true
  try {
    await invoke('hosts_save', { entries: payload })
    ElMessage.success('保存成功')
    // 保存后重新加载（确保 raw_lines 一致）
    await loadHosts()
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

// ============ 生命周期 ============
onMounted(async () => {
  await checkAdmin()
  await loadHosts()
})

onActivated(async () => {
  await checkAdmin()
  if (entries.value.length === 0) {
    await loadHosts()
  }
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
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 在 store/index.ts 添加 TOOL_LIST 条目**

打开 `d:\work\codes\litobox\src\store\index.ts`，在 `hotkeyViewer` 条目之后（约第 96 行）添加：

```typescript
  { id: 'hostsManager', name: 'Hosts管理', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: '编辑 hosts 文件，多环境 profile 切换，自动备份恢复', keywords: ['hosts', '域名', 'dns', '解析', 'profile'], category: 'system' },
```

- [ ] **Step 3: 在 App.vue 注册组件**

打开 `d:\work\codes\litobox\src\App.vue`：

在 `import HotkeyView from '@/views/HotkeyView.vue'` 之后添加：

```typescript
import HostsView from '@/views/HostsView.vue'
```

在 `toolComponentMap` 中 `hotkeyViewer: HotkeyView,` 之后添加：

```typescript
  hostsManager: HostsView,
```

- [ ] **Step 4: 前端编译验证**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc 0 errors, vite build 成功。

- [ ] **Step 5: 提交**

```bash
git add src/views/HostsView.vue src/store/index.ts src/App.vue
git commit -m "feat(hosts): 实现 HostsView 前端页面 Tab1 编辑功能"
```

---

## Task 6: 实现前端 Tab 2（Profile 管理）和 Tab 3（备份恢复）

**Files:**
- Modify: `src/views/HostsView.vue`

- [ ] **Step 1: 替换 Tab 2 和 Tab 3 的占位内容**

打开 `d:\work\codes\litobox\src\views\HostsView.vue`，将 Tab 2 占位替换为：

```vue
    <!-- Tab 2: Profile 管理 -->
    <div v-if="activeTab === 'profiles'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Profile 列表 ({{ profiles.length }})</span>
        <div class="card-actions">
          <el-button size="small" @click="loadProfiles" :loading="profilesLoading">刷新</el-button>
          <el-button size="small" @click="showNewProfileDialog">新建 Profile</el-button>
          <el-button type="primary" size="small" @click="saveCurrentAsProfile">从当前 Hosts 保存为 Profile</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="profiles" border size="small" style="width: 100%" v-loading="profilesLoading">
          <el-table-column prop="name" label="名称" width="160" />
          <el-table-column prop="entry_count" label="条目数" width="100" />
          <el-table-column prop="updated_at" label="更新时间" min-width="180" />
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button type="primary" size="small" link @click="applyProfile(row.name)">切换</el-button>
              <el-button v-if="!row.is_default" type="danger" size="small" link @click="deleteProfile(row.name)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Tab 3: 备份恢复 -->
    <div v-if="activeTab === 'backups'" class="tool-card">
      <div class="card-header">
        <span class="card-title">备份列表 ({{ backups.length }})</span>
        <div class="card-actions">
          <el-button size="small" @click="loadBackups" :loading="backupsLoading">刷新</el-button>
          <el-button type="primary" size="small" @click="createBackupNow">立即备份</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="backups" border size="small" style="width: 100%" v-loading="backupsLoading">
          <el-table-column prop="timestamp" label="时间" min-width="180" />
          <el-table-column label="大小" width="100">
            <template #default="{ row }">{{ formatSize(row.size) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button size="small" link @click="previewBackup(row.filename)">预览</el-button>
              <el-button type="warning" size="small" link @click="restoreBackup(row.filename)">恢复</el-button>
              <el-button type="danger" size="small" link @click="deleteBackup(row.filename)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
```

- [ ] **Step 2: 在 script setup 中追加 Profile 和备份的状态与方法**

在 `const searchQuery = ref('')` 之后追加：

```typescript
// ============ Profile 数据 ============
interface ProfileMeta {
  name: string
  entry_count: number
  updated_at: string
  is_default: boolean
}

interface BackupInfo {
  filename: string
  timestamp: string
  size: number
  path: string
}

const profiles = ref<ProfileMeta[]>([])
const profilesLoading = ref(false)
const backups = ref<BackupInfo[]>([])
const backupsLoading = ref(false)

// ============ Profile 方法 ============
async function loadProfiles() {
  profilesLoading.value = true
  try {
    profiles.value = await invoke<ProfileMeta[]>('hosts_profile_list')
  } catch (e) {
    ElMessage.error(`加载 profile 列表失败: ${e}`)
  } finally {
    profilesLoading.value = false
  }
}

async function applyProfile(name: string) {
  try {
    await ElMessageBox.confirm(
      `切换到 profile "${name}"？当前 hosts 将被覆盖（会自动备份）。`,
      '确认切换',
      { type: 'warning' }
    )
    await invoke('hosts_profile_apply', { name })
    ElMessage.success(`已切换到 profile: ${name}`)
    await loadHosts()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(`切换失败: ${e}`)
  }
}

async function deleteProfile(name: string) {
  try {
    await ElMessageBox.confirm(`删除 profile "${name}"？此操作不可恢复。`, '确认删除', { type: 'warning' })
    await invoke('hosts_profile_delete', { name })
    ElMessage.success('已删除')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(`删除失败: ${e}`)
  }
}

async function showNewProfileDialog() {
  try {
    const { value } = await ElMessageBox.prompt('输入 profile 名称', '新建 Profile', {
      inputPattern: /^[^<>:"/\\|?*]+$/,
      inputErrorMessage: '名称包含非法字符'
    })
    await invoke('hosts_profile_save', { name: value, entries: [] })
    ElMessage.success('已创建空 profile')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`创建失败: ${e}`)
  }
}

async function saveCurrentAsProfile() {
  try {
    const { value } = await ElMessageBox.prompt('输入 profile 名称（已存在则覆盖）', '从当前 Hosts 保存', {
      inputPattern: /^[^<>:"/\\|?*]+$/,
      inputErrorMessage: '名称包含非法字符'
    })
    const payload = entries.value.map(e => ({
      enabled: e.enabled,
      ip: e.ip,
      domains: e.domainsText.split(/\s+/).filter(s => s.length > 0),
      comment: e.comment
    }))
    await invoke('hosts_profile_save', { name: value, entries: payload })
    ElMessage.success('已保存为 profile')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`保存失败: ${e}`)
  }
}

// ============ 备份方法 ============
async function loadBackups() {
  backupsLoading.value = true
  try {
    backups.value = await invoke<BackupInfo[]>('hosts_list_backups')
  } catch (e) {
    ElMessage.error(`加载备份列表失败: ${e}`)
  } finally {
    backupsLoading.value = false
  }
}

async function createBackupNow() {
  try {
    await invoke('hosts_create_backup')
    ElMessage.success('已创建备份')
    await loadBackups()
  } catch (e) {
    ElMessage.error(`备份失败: ${e}`)
  }
}

async function previewBackup(filename: string) {
  try {
    const content = await invoke<string>('hosts_preview_backup', { filename })
    await ElMessageBox.alert(content, `预览: ${filename}`, {
      customClass: 'hosts-preview-dialog',
      confirmButtonText: '关闭'
    })
  } catch (e) {
    ElMessage.error(`预览失败: ${e}`)
  }
}

async function restoreBackup(filename: string) {
  try {
    await ElMessageBox.confirm(
      `恢复备份 "${filename}"？当前 hosts 将被覆盖（会自动备份当前）。`,
      '确认恢复',
      { type: 'warning' }
    )
    await invoke('hosts_restore_backup', { filename })
    ElMessage.success('已恢复')
    await loadHosts()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(`恢复失败: ${e}`)
  }
}

async function deleteBackup(filename: string) {
  try {
    await ElMessageBox.confirm(`删除备份 "${filename}"？此操作不可恢复。`, '确认删除', { type: 'warning' })
    await invoke('hosts_delete_backup', { filename })
    ElMessage.success('已删除')
    await loadBackups()
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(`删除失败: ${e}`)
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
```

- [ ] **Step 3: 在 onActivated 中补充加载逻辑**

将 `onActivated` 修改为：

```typescript
onActivated(async () => {
  await checkAdmin()
  if (entries.value.length === 0) {
    await loadHosts()
  }
  if (activeTab.value === 'profiles' && profiles.value.length === 0) {
    await loadProfiles()
  }
  if (activeTab.value === 'backups' && backups.value.length === 0) {
    await loadBackups()
  }
})
```

- [ ] **Step 4: 添加 Tab 切换时的加载 watch**

在 `onActivated` 之后添加：

```typescript
import { watch } from 'vue'

watch(activeTab, async (newTab) => {
  if (newTab === 'profiles' && profiles.value.length === 0) {
    await loadProfiles()
  } else if (newTab === 'backups' && backups.value.length === 0) {
    await loadBackups()
  }
})
```

注意：将 `import { ref, computed, onMounted, onActivated } from 'vue'` 改为 `import { ref, computed, onMounted, onActivated, watch } from 'vue'`。

- [ ] **Step 5: 前端编译验证**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc 0 errors, vite build 成功。

- [ ] **Step 6: 提交**

```bash
git add src/views/HostsView.vue
git commit -m "feat(hosts): 实现 Profile 管理和备份恢复 Tab"
```

---

## Task 7: 版本号更新 + README + backlog 更新

**Files:**
- Modify: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `README.md`, `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 更新版本号 5.8.0 → 5.9.0**

三个文件同步更新：
- `package.json`：`"version": "5.8.0"` → `"version": "5.9.0"`
- `src-tauri/tauri.conf.json`：`"version": "5.8.0"` → `"version": "5.9.0"`
- `src-tauri/Cargo.toml`：`version = "5.8.0"` → `version = "5.9.0"`

- [ ] **Step 2: 更新 README.md**

在 README 的功能阶段记录中追加：

```markdown
- **V5.9** Hosts 文件管理器（表格编辑/Profile切换/自动备份恢复）
```

- [ ] **Step 3: 更新 feature-backlog.md**

在"已完成版本"表追加一行：

```markdown
| V5.9 | ✅  | Hosts 文件管理器（表格编辑/Profile切换/自动备份恢复）                                | 2026-07-21 |
```

在 A9 行标记已完成：

```markdown
| A9  | ✅ **Hosts 文件管理器**  | 编辑 hosts 文件，语法高亮、启用/禁用条目（注释切换）、多环境 profile（dev/test 一键切换）、备份/恢复。需管理员权限写入 — 已完成 V5.9 | — 已完成 V5.9 — | 2026-07-08 brainstorming |
```

- [ ] **Step 4: 提交**

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml README.md docs/superpowers/plans/feature-backlog.md
git commit -m "chore(release): 发布 v5.9.0 - 新增 Hosts 文件管理器"
```

---

## Task 8: 整体验证

**Files:** 无（仅验证）

- [ ] **Step 1: Rust 测试**

Run: `cd src-tauri; cargo test -- --nocapture`
Expected: 所有测试通过（包含 hosts_manager 的 11 个测试）。

- [ ] **Step 2: 前端构建**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc 0 errors, vite build 成功。

- [ ] **Step 3: 手动验证清单**

运行 `npm run tauri dev`，逐项验证：

1. [ ] 侧边栏「系统工具」分类下出现「Hosts管理」菜单项
2. [ ] 点击进入后显示 hosts 条目表格
3. [ ] 非管理员运行时显示橙色 banner，保存按钮置灰
4. [ ] 管理员运行时可编辑、保存
5. [ ] 复选框启用/禁用切换正常
6. [ ] 添加条目：表格末尾新增空行
7. [ ] 删除条目：行被移除
8. [ ] 保存后重新加载，数据一致
9. [ ] 多域名：一行输入多个域名（空格分隔），保存后重新加载正确解析
10. [ ] 搜索框：实时过滤
11. [ ] Profile Tab：列表显示「默认」+ 用户创建的 profile
12. [ ] 新建 Profile：弹窗输入名称，创建成功
13. [ ] 从当前 Hosts 保存为 Profile：弹窗输入名称，保存成功
14. [ ] 切换 Profile：确认后 hosts 内容更新
15. [ ] 删除 Profile：确认后删除（默认不可删）
16. [ ] 备份 Tab：列表显示备份（时间 + 大小）
17. [ ] 立即备份：创建成功，列表更新
18. [ ] 预览备份：弹窗显示内容
19. [ ] 恢复备份：确认后 hosts 内容恢复
20. [ ] 删除备份：确认后删除
21. [ ] KeepAlive：切换到其他工具再回来，保留数据

- [ ] **Step 4: 如有问题，回到对应 Task 修复**
