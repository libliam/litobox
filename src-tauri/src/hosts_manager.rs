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
