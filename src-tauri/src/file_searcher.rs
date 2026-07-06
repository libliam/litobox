use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
// 沿用 disk_analyzer 的宏定义
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Deserialize)]
pub struct SearchOptions {
    pub mode: String,                   // "filename" | "content"
    pub query: String,                  // 正则表达式
    pub caseSensitive: bool,
    pub extensions: Vec<String>,        // 包含列表，如 ["ts","js"]；空=不限
    pub excludeExtensions: Vec<String>, // 排除列表，如 ["exe","dll"]
    pub includeHidden: bool,
    pub maxContentFileBytes: u64,       // 内容模式：超过此大小跳过内容只匹配文件名
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedLine {
    pub lineNumber: u32,
    pub lineText: String,                   // 截断 500 字符
    pub matchRanges: Vec<(u32, u32)>,       // 字符偏移，单行最多 5 个
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultItem {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub sizeBytes: u64,
    pub modifiedMs: i64,
    pub matchCount: u32,                    // 文件总命中数（文件名模式=1）
    pub matchedLines: Vec<MatchedLine>,     // 内容模式最多 3 行预览；文件名模式为空
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum SearchStatus {
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchProgress {
    pub searchId: String,
    pub filesScanned: u64,
    pub bytesScanned: u64,
    pub matchesFound: u32,
    pub currentPath: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchSummary {
    pub totalFiles: u64,
    pub totalDirs: u64,
    pub bytesScanned: u64,
    pub matchesFound: u32,
    pub durationMs: u64,
    pub truncated: bool,
    pub skippedCount: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultsPage {
    pub items: Vec<SearchResultItem>,
    pub total: u64,
}

// ============ 内部状态 ============

#[derive(Debug)]
pub struct SearchResults {
    pub search_id: String,
    pub root_path: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub status: SearchStatus,
    pub cancel_flag: Arc<AtomicBool>,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_path: Option<String>,
    pub skipped_count: u32,
    pub results: Vec<SearchResultItem>,
    pub truncated: bool,
}

// ============ 常量 ============

const MAX_RESULTS: u32 = 1000;
const MAX_PREVIEW_LINES: usize = 3;
const MAX_MATCHES_PER_LINE: usize = 5;
const MAX_LINE_TEXT_CHARS: usize = 500;
const CANCEL_CHECK_INTERVAL: u64 = 1000;
const BINARY_DETECT_WINDOW: usize = 8 * 1024;

// ============ 纯函数 ============

/// 当前毫秒时间戳
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 判断文件名是否隐藏（沿用 disk_analyzer 的 is_hidden 逻辑）
fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// 解析扩展名过滤输入："ts, .js" -> (["ts","js"], [])
/// "!exe, dll" -> ([], ["exe","dll"])
/// 空字符串 -> ([], [])
fn parse_extension_filter(text: &str) -> (Vec<String>, Vec<String>) {
    let tokens: Vec<String> = text
        .split(',')
        .map(|s| s.trim().trim_start_matches('.').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.is_empty() {
        return (Vec::new(), Vec::new());
    }
    // 任一 token 以 ! 开头则整体视为排除模式
    let is_exclude = tokens.iter().any(|t| t.starts_with('!'));
    if is_exclude {
        let exc: Vec<String> = tokens
            .iter()
            .map(|t| t.trim_start_matches('!').to_string())
            .collect();
        (Vec::new(), exc)
    } else {
        (tokens, Vec::new())
    }
}

/// 二进制检测：BOM 优先（UTF-8/UTF-16 BOM 直接判为非二进制），
/// 无 BOM 时检查前 8KB 是否含 \0 字节
fn is_binary(bytes: &[u8]) -> bool {
    // BOM 优先
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return false;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        return false;
    }
    // 无 BOM：检查前 8KB
    let window = if bytes.len() > BINARY_DETECT_WINDOW {
        &bytes[..BINARY_DETECT_WINDOW]
    } else {
        bytes
    };
    window.contains(&0u8)
}

/// 构建正则：caseSensitive=false 时注入 (?i) 前缀
fn build_regex(query: &str, case_sensitive: bool) -> Result<regex::Regex, String> {
    let pattern = if case_sensitive {
        query.to_string()
    } else {
        format!("(?i){}", query)
    };
    regex::Regex::new(&pattern).map_err(|e| format!("正则表达式无效: {}", e))
}

/// 扫描文本内容，返回 (总命中数, 预览行列表)
fn scan_content(text: &str, re: &regex::Regex, max_lines: usize) -> (u32, Vec<MatchedLine>) {
    let mut total_matches: u32 = 0;
    let mut previews: Vec<MatchedLine> = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line_no = (idx as u32) + 1;
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for m in re.find_iter(line) {
            // 字符偏移（前端 slice 友好）
            let start = line[..m.start()].chars().count() as u32;
            let end = line[..m.end()].chars().count() as u32;
            ranges.push((start, end));
            total_matches += 1;
            if ranges.len() >= MAX_MATCHES_PER_LINE {
                break;
            }
        }
        if !ranges.is_empty() && previews.len() < max_lines {
            let truncated_text = if line.chars().count() > MAX_LINE_TEXT_CHARS {
                let mut s: String = line.chars().take(MAX_LINE_TEXT_CHARS).collect();
                s.push('…');
                s
            } else {
                line.to_string()
            };
            previews.push(MatchedLine {
                lineNumber: line_no,
                lineText: truncated_text,
                matchRanges: ranges,
            });
        }
    }
    (total_matches, previews)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ext_filter_include_and_exclude() {
        let (inc, exc) = parse_extension_filter("ts, .js, .vue");
        assert_eq!(inc, vec!["ts", "js", "vue"]);
        assert!(exc.is_empty());

        let (inc, exc) = parse_extension_filter("!exe, dll");
        assert!(inc.is_empty());
        assert_eq!(exc, vec!["exe", "dll"]);

        // 空输入
        let (inc, exc) = parse_extension_filter("");
        assert!(inc.is_empty());
        assert!(exc.is_empty());
    }

    #[test]
    fn is_binary_respects_bom_and_null() {
        assert!(!is_binary(&[0xEF, 0xBB, 0xBF, b'h', b'i']), "UTF-8 BOM 不是二进制");
        assert!(!is_binary(&[0xFF, 0xFE, b'A', 0x00]), "UTF-16 LE BOM 不是二进制");
        assert!(!is_binary(&[0xFE, 0xFF, 0x00, b'A']), "UTF-16 BE BOM 不是二进制");
        assert!(is_binary(&[b'A', 0x00, b'B', 0x01]), "无 BOM 含 \\0 是二进制");
        assert!(!is_binary(b"plain ascii text"), "纯 ASCII 不是二进制");
    }

    #[test]
    fn scan_content_char_offsets_for_multibyte() {
        let re = regex::Regex::new("World").unwrap();
        let text = "你好 World 继续\n第二行 World";
        let (total, previews) = scan_content(text, &re, MAX_PREVIEW_LINES);
        assert_eq!(total, 2, "应命中 2 次");
        assert_eq!(previews.len(), 2, "应有 2 行预览");
        // "你好 World" 中 World 的字符偏移是 3..8（中文占 2 字符 + 空格 1）
        assert_eq!(previews[0].matchRanges[0], (3, 8));
        assert_eq!(previews[0].lineNumber, 1);
        assert_eq!(previews[1].lineNumber, 2);
    }

    #[test]
    fn build_regex_case_insensitive_flag() {
        let re_ci = build_regex("foo", false).unwrap();
        assert!(re_ci.is_match("FOO Bar"));
        let re_cs = build_regex("foo", true).unwrap();
        assert!(!re_cs.is_match("FOO Bar"));
        assert!(re_cs.is_match("a foo b"));

        // 无效正则应返回错误
        assert!(build_regex("[", false).is_err());
    }
}
