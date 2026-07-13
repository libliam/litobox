# 全文搜索工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 V4.6 全文搜索工具，支持文件名/内容双模式正则搜索，镜像磁盘分析器架构（不建索引、walkdir 实时遍历、进度+取消）。

**Architecture:** 后端新建 `file_searcher.rs`（沿用 disk_analyzer 的 `OnceLock<Mutex<HashMap>>` + `thread::spawn` + `emit` 模式），在 `file_encoding.rs` 末尾加 `read_file_auto` 复用解码；前端新建 `FileSearcher.vue` 单页 + types/client；复用 `disk_locate_in_explorer` 命令。

**Tech Stack:** Rust (Tauri 2.0, walkdir, encoding_rs, regex, uuid) + Vue 3 (Composition API, Element Plus, TypeScript)

**Spec:** `docs/superpowers/specs/2026-07-07-file-searcher-design.md`

---

## 文件结构

| 文件 | 责任 | 改动类型 |
|------|------|----------|
| `src-tauri/Cargo.toml` | 加 `regex = "1.10"`，版本 4.4.0 → 4.6.0 | 修改 |
| `src-tauri/src/file_encoding.rs` | 末尾新增 `read_file_auto` 函数 + 测试 | 修改 |
| `src-tauri/src/file_searcher.rs` | 搜索核心：结构 + 纯函数 + run_search + 6 命令 + 测试 | 新增 |
| `src-tauri/src/main.rs` | `mod file_searcher` + `generate_handler!` 注册 6 命令 | 修改 |
| `src/utils/fileSearcherTypes.ts` | TS 类型定义 | 新增 |
| `src/utils/fileSearcherClient.ts` | invoke 封装 | 新增 |
| `src/views/FileSearcher.vue` | 搜索页面 | 新增 |
| `src/App.vue` | `toolComponentMap` 加 `fileSearcher` | 修改 |
| `src/store/index.ts` | `TOOL_LIST` 末尾加条目 | 修改 |
| `package.json` | 版本 4.5.0 → 4.6.0 | 修改 |
| `README.md` | 版本表加 V4.6 行 | 修改 |
| `docs/superpowers/plans/feature-backlog.md` | B1 移到已完成版本 | 修改 |

---

## Task 1: Cargo.toml 加 regex 依赖 + 版本号同步

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 的 [dependencies] 末尾加 regex，并更新版本号**

打开 `src-tauri/Cargo.toml`，将 `version = "4.4.0"` 改为 `version = "4.6.0"`，并在 `[dependencies]` 段末尾（`uuid = ...` 之后）添加 `regex`：

```toml
[package]
name = "litobox"
version = "4.6.0"
description = "栗的百宝箱"
authors = ["developer"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0.0", features = [] }

[dependencies]
tauri = { version = "2.0.0", features = [] }
tauri-plugin-shell = "2.0.0"
tauri-plugin-dialog = "2.7"
tauri-plugin-global-shortcut = "2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
encoding_rs = "0.8"
boa_engine = "0.21"
arboard = "3.3"
reqwest = { version = "0.12", features = ["json"] }
serde_urlencoded = "0.7"
base64 = "0.22"
rusqlite = { version = "0.32", features = ["bundled"] }
dirs = "5.0"
chardetng = "0.1"
sysinfo = "0.31"
walkdir = "2.5"
trash = "5.1"
sha2 = "0.10"
uuid = { version = "1.10", features = ["v4"] }
regex = "1.10"

[dev-dependencies]
tempfile = "3.10"

[features]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 2: 验证依赖能编译**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 编译通过（可能首次下载 regex crate 较慢），无错误。

- [ ] **Step 3: 暂不提交（后续任务一起提交）**

---

## Task 2: file_encoding.rs 新增 read_file_auto（TDD）

**Files:**
- Modify: `src-tauri/src/file_encoding.rs`（末尾追加函数 + 测试模块）

- [ ] **Step 1: 在 file_encoding.rs 末尾添加测试模块（含失败测试）**

在 `src-tauri/src/file_encoding.rs` 文件最末尾追加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_auto_handles_gbk_and_utf16() {
        let dir = tempfile::TempDir::new().unwrap();

        // GBK 文件："中文" 的 GBK 字节是 D6 D0 CE C4
        let gbk_path = dir.path().join("gbk.txt");
        std::fs::write(&gbk_path, [0xD6, 0xD0, 0xCE, 0xC4]).unwrap();
        let s = read_file_auto(&gbk_path).unwrap();
        assert_eq!(s, "中文", "GBK 文件应解码为中文");

        // UTF-16 LE with BOM: "Hi" -> FF FE 48 00 69 00
        let u16_path = dir.path().join("u16.txt");
        let u16_bytes: Vec<u8> = vec![0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
        std::fs::write(&u16_path, u16_bytes).unwrap();
        let s2 = read_file_auto(&u16_path).unwrap();
        assert_eq!(s2, "Hi", "UTF-16 LE BOM 文件应正确解码");

        // UTF-8 BOM
        let u8_path = dir.path().join("u8.txt");
        std::fs::write(&u8_path, [0xEF, 0xBB, 0xBF, b'h', b'i']).unwrap();
        let s3 = read_file_auto(&u8_path).unwrap();
        assert_eq!(s3, "hi", "UTF-8 BOM 文件应正确解码");

        // 纯 ASCII（无 BOM）
        let ascii_path = dir.path().join("ascii.txt");
        std::fs::write(&ascii_path, b"plain ascii").unwrap();
        let s4 = read_file_auto(&ascii_path).unwrap();
        assert_eq!(s4, "plain ascii", "纯 ASCII 应原样返回");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo test read_file_auto
```
Expected: 编译失败，错误 `cannot find function read_file_auto in this scope`。

- [ ] **Step 3: 在 file_encoding.rs 末尾（test 模块之前）实现 read_file_auto**

在 `src-tauri/src/file_encoding.rs` 的 `detect_file_encoding` 函数之后、`#[cfg(test)]` 之前添加：

```rust
/// 读取文件并自动检测编码解码为字符串（单次读盘，供内容搜索复用）
/// ponytail: 与 detect_file_encoding 逻辑一致但单次读盘，避免搜索时双读
pub fn read_file_auto(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("读取失败: {}", e))?;
    // BOM 优先（与 detect_file_encoding 一致）
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(String::from_utf8_lossy(&bytes[3..]).into_owned());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| {
                if bytes[1] == 0xFE {
                    u16::from_le_bytes([c[0], c[1]])
                } else {
                    u16::from_be_bytes([c[0], c[1]])
                }
            })
            .collect();
        return Ok(String::from_utf16_lossy(&utf16));
    }
    // 无 BOM：先试 UTF-8 严格，失败回退 GBK（与项目惯例一致）
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => {
            let (decoded, _, _) = GBK.decode(&bytes);
            Ok(decoded.into_owned())
        }
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo test read_file_auto
```
Expected: 1 个测试通过。

- [ ] **Step 5: 提交**

```powershell
cd d:\work\codes\litobox
git add src-tauri/Cargo.toml src-tauri/src/file_encoding.rs
git commit -m "feat(file-searcher): 新增 read_file_auto 自动解码函数 + regex 依赖"
```

---

## Task 3: file_searcher.rs 数据结构 + 常量

**Files:**
- Create: `src-tauri/src/file_searcher.rs`

- [ ] **Step 1: 创建 file_searcher.rs，写入数据结构 + 常量 + debug_log 宏**

创建 `src-tauri/src/file_searcher.rs`：

```rust
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
```

- [ ] **Step 2: 暂不编译（main.rs 未注册模块，且后续任务会补全）**

---

## Task 4: file_searcher.rs 纯函数（TDD）

**Files:**
- Modify: `src-tauri/src/file_searcher.rs`（在常量后追加纯函数 + 测试模块）

- [ ] **Step 1: 在 file_searcher.rs 末尾添加测试模块（含 4 个失败测试）**

在 `src-tauri/src/file_searcher.rs` 末尾追加：

```rust
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
```

- [ ] **Step 2: 在 main.rs 临时注册模块以支持编译测试**

在 `src-tauri/src/main.rs` 第 12 行 `mod disk_analyzer;` 之后添加：

```rust
mod file_searcher;
```

- [ ] **Step 3: 运行测试验证失败**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo test file_searcher
```
Expected: 编译失败，`cannot find function parse_extension_filter / is_binary / scan_content / build_regex`。

- [ ] **Step 4: 在 file_searcher.rs 的常量后、`#[cfg(test)]` 之前实现 4 个纯函数**

```rust
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
    let tokens: Vec<&str> = text
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
        (tokens.into_iter().map(String::from).collect(), Vec::new())
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
```

- [ ] **Step 5: 运行测试验证通过**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo test file_searcher
```
Expected: 4 个测试通过。

- [ ] **Step 6: 提交**

```powershell
cd d:\work\codes\litobox
git add src-tauri/src/file_searcher.rs src-tauri/src/main.rs
git commit -m "feat(file-searcher): 数据结构 + 4 个纯函数（parse_ext_filter/is_binary/build_regex/scan_content）含单测"
```

---

## Task 5: file_searcher.rs 全局状态 + run_search

**Files:**
- Modify: `src-tauri/src/file_searcher.rs`（在纯函数后追加状态管理 + run_search）

- [ ] **Step 1: 在 file_searcher.rs 的纯函数后添加全局状态管理**

在 `src-tauri/src/file_searcher.rs` 的 `scan_content` 函数之后、`#[cfg(test)]` 之前添加：

```rust
// ============ 全局状态（沿用 disk_analyzer 的 OnceLock 模式） ============

// ponytail: 全局搜索结果存储。OnceLock + Mutex 模式，沿用 disk_analyzer 惯例
// 升级路径：若并发搜索数经常 >10，可改用 Tauri managed state + Arc
static SEARCHES: OnceLock<Mutex<HashMap<String, Arc<Mutex<SearchResults>>>>> = OnceLock::new();

fn searches() -> &'static Mutex<HashMap<String, Arc<Mutex<SearchResults>>>> {
    SEARCHES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_search(search_id: &str) -> Option<Arc<Mutex<SearchResults>>> {
    let s = searches().lock().unwrap();
    s.get(search_id).cloned()
}

fn insert_search(search_id: String, results: Arc<Mutex<SearchResults>>) {
    let mut s = searches().lock().unwrap();
    s.insert(search_id, results);
}

fn remove_search(search_id: &str) -> bool {
    let mut s = searches().lock().unwrap();
    s.remove(search_id).is_some()
}
```

- [ ] **Step 2: 在全局状态后添加 run_search 核心逻辑**

继续在 `src-tauri/src/file_searcher.rs` 添加：

```rust
// ============ 搜索核心逻辑 ============

/// 同步搜索指定路径，更新 results
fn run_search(
    results_arc: Arc<Mutex<SearchResults>>,
    app: AppHandle,
    opts: SearchOptions,
) -> Result<(), String> {
    let root_path = {
        let r = results_arc.lock().unwrap();
        r.root_path.clone()
    };

    // 编译正则
    let re = build_regex(&opts.query, opts.caseSensitive)?;

    // 解析扩展名过滤
    let (ext_include, ext_exclude) = (
        opts.extensions.clone(),
        opts.excludeExtensions.clone(),
    );

    let mut total_files: u64 = 0;
    let mut total_dirs: u64 = 0;
    let mut bytes_scanned: u64 = 0;
    let mut skipped_count: u32 = 0;
    let mut files_since_check: u64 = 0;
    let mut last_progress_emit = std::time::Instant::now();

    let mut walker = walkdir::WalkDir::new(&root_path)
        .follow_links(false)
        .into_iter();

    while let Some(entry) = walker.next() {
        // 取消检查（每 CANCEL_CHECK_INTERVAL 项查一次）
        if files_since_check >= CANCEL_CHECK_INTERVAL {
            files_since_check = 0;
            let r = results_arc.lock().unwrap();
            if r.cancel_flag.load(Ordering::SeqCst) {
                debug_log!("file_searcher: 搜索被取消");
                let mut r = results_arc.lock().unwrap();
                r.status = SearchStatus::Cancelled;
                r.finished_at = Some(now_ms());
                return Ok(());
            }
        }
        files_since_check += 1;

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug_log!("file_searcher: 跳过无权限项: {}", e);
                skipped_count += 1;
                continue;
            }
        };

        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        {
            let mut r = results_arc.lock().unwrap();
            r.current_path = Some(path_str.clone());
        }

        if entry.file_type().is_dir() {
            total_dirs += 1;
            if !opts.includeHidden && is_hidden(entry.file_name()) {
                walker.skip_current_dir();
                continue;
            }
        } else if entry.file_type().is_file() {
            if !opts.includeHidden && is_hidden(entry.file_name()) {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            // 扩展名过滤
            if !ext_include.is_empty() && !ext_include.contains(&extension) {
                continue;
            }
            if ext_exclude.contains(&extension) {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => {
                    skipped_count += 1;
                    continue;
                }
            };
            let size = meta.len();
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            total_files += 1;
            bytes_scanned += size;

            // 文件名模式匹配
            let name_matched = re.is_match(&name);
            let mut item: Option<SearchResultItem> = None;

            if opts.mode == "content" {
                let should_read_content = size <= opts.maxContentFileBytes;
                if should_read_content {
                    match crate::file_encoding::read_file_auto(path) {
                        Ok(content) => {
                            if !is_binary(content.as_bytes()) {
                                let (match_count, matched_lines) =
                                    scan_content(&content, &re, MAX_PREVIEW_LINES);
                                if match_count > 0 {
                                    item = Some(SearchResultItem {
                                        path: path_str.clone(),
                                        name: name.clone(),
                                        extension: extension.clone(),
                                        sizeBytes: size,
                                        modifiedMs: modified_ms,
                                        matchCount: match_count,
                                        matchedLines,
                                    });
                                }
                            } else {
                                // 二进制：跳过内容，仅文件名匹配
                                if name_matched {
                                    item = Some(filename_only_item(
                                        &path_str, &name, &extension, size, modified_ms,
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            debug_log!("file_searcher: 读取文件失败 {}: {}", path_str, e);
                            skipped_count += 1;
                        }
                    }
                } else {
                    // 超大文件：降级为文件名匹配
                    if name_matched {
                        item = Some(filename_only_item(
                            &path_str, &name, &extension, size, modified_ms,
                        ));
                    }
                }
            } else {
                // 文件名模式
                if name_matched {
                    item = Some(filename_only_item(
                        &path_str, &name, &extension, size, modified_ms,
                    ));
                }
            }

            if let Some(it) = item {
                let mut r = results_arc.lock().unwrap();
                if r.results.len() < MAX_RESULTS as usize {
                    r.results.push(it);
                } else if !r.truncated {
                    r.truncated = true;
                    debug_log!("file_searcher: 命中 MAX_RESULTS 上限，截断");
                    let _ = app.emit(
                        "file-search-warning",
                        serde_json::json!({
                            "searchId": r.search_id,
                            "message": format!("已达结果上限 {}，后续命中已截断", MAX_RESULTS),
                        }),
                    );
                }
            }

            {
                let mut r = results_arc.lock().unwrap();
                r.files_scanned = total_files;
                r.bytes_scanned = bytes_scanned;
                r.skipped_count = skipped_count;
            }
        }

        // 进度事件：每 200ms 或每 1000 文件
        if last_progress_emit.elapsed() >= std::time::Duration::from_millis(200) {
            last_progress_emit = std::time::Instant::now();
            let r = results_arc.lock().unwrap();
            let _ = app.emit(
                "file-search-progress",
                SearchProgress {
                    searchId: r.search_id.clone(),
                    filesScanned: r.files_scanned,
                    bytesScanned: r.bytes_scanned,
                    matchesFound: r.results.len() as u32,
                    currentPath: r.current_path.clone().unwrap_or_default(),
                },
            );
        }
    }

    // 完成
    {
        let mut r = results_arc.lock().unwrap();
        r.status = SearchStatus::Completed;
        r.finished_at = Some(now_ms());
        let summary = SearchSummary {
            totalFiles: r.files_scanned,
            totalDirs: total_dirs,
            bytesScanned: r.bytes_scanned,
            matchesFound: r.results.len() as u32,
            durationMs: r
                .finished_at
                .map(|f| (f - r.started_at) as u64)
                .unwrap_or(0),
            truncated: r.truncated,
            skippedCount: r.skipped_count,
        };
        let _ = app.emit("file-search-complete", summary);
    }
    debug_log!("file_searcher: 搜索完成 id={}", root_path);
    Ok(())
}

/// 构造仅文件名匹配的结果项
fn filename_only_item(
    path: &str,
    name: &str,
    extension: &str,
    size: u64,
    modified_ms: i64,
) -> SearchResultItem {
    SearchResultItem {
        path: path.to_string(),
        name: name.to_string(),
        extension: extension.to_string(),
        sizeBytes: size,
        modifiedMs: modified_ms,
        matchCount: 1,
        matchedLines: Vec::new(),
    }
}
```

- [ ] **Step 3: 验证编译**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 编译通过（可能有未使用函数警告，Tauri 命令未注册前正常）。

- [ ] **Step 4: 运行已有测试确保未破坏**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo test file_searcher
```
Expected: 4 个测试仍通过。

- [ ] **Step 5: 暂不提交（与下个任务一起）**

---

## Task 6: file_searcher.rs Tauri 命令

**Files:**
- Modify: `src-tauri/src/file_searcher.rs`（在 run_search 后、test 模块前追加 6 个命令）

- [ ] **Step 1: 在 file_searcher.rs 的 filename_only_item 后、`#[cfg(test)]` 前添加 6 个 Tauri 命令**

```rust
// ============ Tauri 命令 ============

#[tauri::command]
pub async fn file_search_start(
    app: AppHandle,
    path: String,
    opts: SearchOptions,
) -> Result<String, String> {
    let path_canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|e| format!("路径无法访问: {}", e))?
        .to_string_lossy()
        .to_string();

    let search_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let results = SearchResults {
        search_id: search_id.clone(),
        root_path: path_canonical.clone(),
        started_at: now_ms(),
        finished_at: None,
        status: SearchStatus::Running,
        cancel_flag: cancel_flag.clone(),
        files_scanned: 0,
        bytes_scanned: 0,
        current_path: None,
        skipped_count: 0,
        results: Vec::new(),
        truncated: false,
    };
    let results_arc = Arc::new(Mutex::new(results));
    insert_search(search_id.clone(), results_arc.clone());

    debug_log!(
        "file_searcher: start id={} path={} mode={}",
        search_id, path_canonical, opts.mode
    );

    let app_clone = app.clone();
    let search_id_clone = search_id.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_search(results_arc.clone(), app_clone, opts) {
            debug_log!("file_searcher: 失败 id={} err={}", search_id_clone, e);
            let mut r = results_arc.lock().unwrap();
            r.status = SearchStatus::Failed { error: e };
            r.finished_at = Some(now_ms());
        }
    });

    Ok(search_id)
}

#[tauri::command]
pub fn file_search_cancel(search_id: String) -> Result<(), String> {
    debug_log!("file_searcher: cancel id={}", search_id);
    if let Some(arc) = get_search(&search_id) {
        let r = arc.lock().unwrap();
        r.cancel_flag.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("search not found or expired".into())
    }
}

#[tauri::command]
pub fn file_search_status(search_id: String) -> Result<SearchStatus, String> {
    let arc = get_search(&search_id).ok_or("search not found or expired")?;
    let r = arc.lock().unwrap();
    Ok(r.status.clone())
}

#[tauri::command]
pub fn file_search_get_summary(search_id: String) -> Result<SearchSummary, String> {
    let arc = get_search(&search_id).ok_or("search not found or expired")?;
    let r = arc.lock().unwrap();
    Ok(SearchSummary {
        totalFiles: r.files_scanned,
        totalDirs: 0, // ponytail: totalDirs 在 run_search 内未存入结构，简化为 0
        bytesScanned: r.bytes_scanned,
        matchesFound: r.results.len() as u32,
        durationMs: r
            .finished_at
            .map(|f| (f - r.started_at) as u64)
            .unwrap_or(0),
        truncated: r.truncated,
        skippedCount: r.skipped_count,
    })
}

#[tauri::command]
pub fn file_search_get_results(
    search_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchResultsPage, String> {
    let arc = get_search(&search_id).ok_or("search not found or expired")?;
    let r = arc.lock().unwrap();
    let limit = limit.unwrap_or(100) as usize;
    let offset = offset.unwrap_or(0) as usize;
    let total = r.results.len() as u64;
    let items = r
        .results
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    Ok(SearchResultsPage { items, total })
}

#[tauri::command]
pub fn file_search_clear(search_id: String) -> Result<(), String> {
    debug_log!("file_searcher: clear id={}", search_id);
    remove_search(&search_id);
    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 编译通过。

- [ ] **Step 3: 提交**

```powershell
cd d:\work\codes\litobox
git add src-tauri/src/file_searcher.rs
git commit -m "feat(file-searcher): 全局状态 + run_search + 6 个 Tauri 命令"
```

---

## Task 7: main.rs 注册命令

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 main.rs 的 disk_analyzer 命令注册后添加 file_searcher 命令**

在 `src-tauri/src/main.rs` 的 `generate_handler!` 宏中，找到 `disk_analyzer::disk_locate_in_explorer,`（约 line 121）这一行，在其后添加：

```rust
            disk_analyzer::disk_locate_in_explorer,
            // 全文搜索命令
            file_searcher::file_search_start,
            file_searcher::file_search_cancel,
            file_searcher::file_search_status,
            file_searcher::file_search_get_summary,
            file_searcher::file_search_get_results,
            file_searcher::file_search_clear,
```

注：`mod file_searcher;` 已在 Task 4 Step 2 添加。

- [ ] **Step 2: 验证编译 + 重启 dev server**

Run:
```powershell
cd d:\work\codes\litobox\src-tauri
cargo check
```
Expected: 编译通过。

- [ ] **Step 3: 提交**

```powershell
cd d:\work\codes\litobox
git add src-tauri/src/main.rs
git commit -m "feat(file-searcher): main.rs 注册 6 个搜索命令"
```

---

## Task 8: 前端类型定义 + client 封装

**Files:**
- Create: `src/utils/fileSearcherTypes.ts`
- Create: `src/utils/fileSearcherClient.ts`

- [ ] **Step 1: 创建 fileSearcherTypes.ts**

创建 `src/utils/fileSearcherTypes.ts`：

```typescript
export interface SearchOptions {
  mode: 'filename' | 'content'
  query: string
  caseSensitive: boolean
  extensions: string[]
  excludeExtensions: string[]
  includeHidden: boolean
  maxContentFileBytes: number
}

export interface MatchedLine {
  lineNumber: number
  lineText: string
  matchRanges: [number, number][]
}

export interface SearchResultItem {
  path: string
  name: string
  extension: string
  sizeBytes: number
  modifiedMs: number
  matchCount: number
  matchedLines: MatchedLine[]
}

export type SearchStatus =
  | { status: 'running' }
  | { status: 'completed' }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }

export interface SearchProgress {
  searchId: string
  filesScanned: number
  bytesScanned: number
  matchesFound: number
  currentPath: string
}

export interface SearchSummary {
  totalFiles: number
  totalDirs: number
  bytesScanned: number
  matchesFound: number
  durationMs: number
  truncated: boolean
  skippedCount: number
}

export interface SearchResultsPage {
  items: SearchResultItem[]
  total: number
}
```

- [ ] **Step 2: 创建 fileSearcherClient.ts**

创建 `src/utils/fileSearcherClient.ts`：

```typescript
import { invoke } from '@tauri-apps/api/core'
import type {
  SearchOptions,
  SearchStatus,
  SearchSummary,
  SearchResultsPage,
} from './fileSearcherTypes'

export async function fileSearchStart(path: string, opts: SearchOptions): Promise<string> {
  return invoke<string>('file_search_start', { path, opts })
}

export async function fileSearchCancel(searchId: string): Promise<void> {
  return invoke('file_search_cancel', { searchId })
}

export async function fileSearchStatus(searchId: string): Promise<SearchStatus> {
  return invoke<SearchStatus>('file_search_status', { searchId })
}

export async function fileSearchGetSummary(searchId: string): Promise<SearchSummary> {
  return invoke<SearchSummary>('file_search_get_summary', { searchId })
}

export async function fileSearchGetResults(
  searchId: string,
  limit?: number,
  offset?: number
): Promise<SearchResultsPage> {
  return invoke<SearchResultsPage>('file_search_get_results', { searchId, limit, offset })
}

export async function fileSearchClear(searchId: string): Promise<void> {
  return invoke('file_search_clear', { searchId })
}
```

- [ ] **Step 3: 暂不提交（与下个任务一起）**

---

## Task 9: FileSearcher.vue 主页面

**Files:**
- Create: `src/views/FileSearcher.vue`

- [ ] **Step 1: 创建 FileSearcher.vue**

创建 `src/views/FileSearcher.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- 1. 搜索配置卡片（sticky） -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">全文搜索</span>
        <div class="card-actions">
          <el-button size="small" @click="loadLastPath">上次路径</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 2">
            <div class="group-label">搜索路径</div>
            <el-input
              v-model="searchPath"
              placeholder="选择或输入要搜索的目录路径"
              size="small"
              clearable
            >
              <template #append>
                <el-button size="small" @click="selectFolder">浏览</el-button>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">模式</div>
            <el-radio-group v-model="opts.mode" size="small">
              <el-radio-button value="filename">文件名</el-radio-button>
              <el-radio-button value="content">内容</el-radio-button>
            </el-radio-group>
          </div>
        </div>
        <div class="action-grid" style="margin-top: 8px">
          <div class="action-group" style="flex: 2">
            <div class="group-label">搜索词（正则）</div>
            <el-input
              v-model="opts.query"
              placeholder="例如 \d{4}-\d{2}-\d{2} 或 TODO"
              size="small"
              clearable
              @keyup.enter="startSearch"
            />
          </div>
          <div class="action-group">
            <div class="group-label">扩展名</div>
            <el-input
              v-model="extFilterText"
              placeholder="ts,js 或 !exe,dll"
              size="small"
            />
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!searchPath || !opts.query || searching"
                :loading="searching"
                @click="startSearch"
              >
                搜索
              </el-button>
            </div>
          </div>
        </div>
        <div class="action-grid" style="margin-top: 8px">
          <el-checkbox v-model="opts.caseSensitive">区分大小写</el-checkbox>
          <el-checkbox v-model="opts.includeHidden">包含隐藏</el-checkbox>
          <div class="action-group" v-if="opts.mode === 'content'">
            <div class="group-label">内容最大文件</div>
            <el-input-number
              v-model="maxContentMb"
              :min="1"
              :max="500"
              size="small"
              controls-position="right"
              style="width: 110px"
            />
            <span class="hint">MB</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 2. 进度卡片 -->
    <div v-if="searching" class="tool-card">
      <div class="card-header">
        <span class="card-title">搜索中</span>
        <el-button size="small" type="danger" @click="cancelSearch">取消</el-button>
      </div>
      <div class="card-body">
        <div class="progress-info">
          <div>当前: {{ progress?.currentPath || '准备中...' }}</div>
          <div>
            已扫描 {{ progress?.filesScanned || 0 }} 文件 |
            命中 {{ progress?.matchesFound || 0 }} |
            耗时 {{ formatDuration(elapsedMs) }}
          </div>
        </div>
        <el-progress
          :percentage="100"
          :show-text="false"
          :stroke-width="14"
          stripe
          status="success"
          :indeterminate="true"
        />
      </div>
    </div>

    <!-- 3. 错误卡片 -->
    <div v-if="searchError" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ searchError }}</div>
      </div>
    </div>

    <!-- 4. 结果卡片 -->
    <div v-if="completed && summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">结果</span>
        <div class="card-actions">
          <span class="summary-text">
            {{ summary.matchesFound }} 命中 |
            {{ summary.totalFiles }} 文件 |
            耗时 {{ formatDuration(summary.durationMs) }}
            <span v-if="summary.truncated" class="warn-text">
              (已达上限 1000，结果截断)
            </span>
            <span v-if="summary.skippedCount > 0" class="warn-text">
              (跳过 {{ summary.skippedCount }} 个文件)
            </span>
          </span>
        </div>
      </div>
      <div class="card-body">
        <el-table
          :data="resultItems"
          stripe
          size="small"
          @row-dblclick="locateInExplorer"
        >
          <el-table-column label="文件" min-width="300">
            <template #default="{ row }">
              <div class="file-name"><strong>{{ row.name }}</strong></div>
              <div class="file-path">{{ row.path }}</div>
              <div class="file-meta">
                {{ formatBytes(row.sizeBytes) }} · {{ formatTime(row.modifiedMs) }}
              </div>
            </template>
          </el-table-column>
          <el-table-column
            v-if="opts.mode === 'content'"
            label="命中行"
            min-width="400"
          >
            <template #default="{ row }">
              <div
                v-for="ml in row.matchedLines"
                :key="ml.lineNumber"
                class="match-line"
              >
                <span class="line-no">L{{ ml.lineNumber }}:</span>
                <span class="line-text" v-html="highlightLine(ml)"></span>
              </div>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="120">
            <template #default="{ row }">
              <el-button size="small" link @click="locateInExplorer(row)">
                定位
              </el-button>
            </template>
          </el-table-column>
        </el-table>
        <el-pagination
          v-if="totalResults > pageSize"
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="totalResults"
          layout="prev, pager, next, total"
          @current-change="loadResults"
          style="margin-top: 12px; justify-content: flex-end"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import {
  fileSearchStart,
  fileSearchCancel,
  fileSearchGetSummary,
  fileSearchGetResults,
  fileSearchClear,
} from '@/utils/fileSearcherClient'
import type {
  SearchOptions,
  SearchProgress,
  SearchSummary,
  SearchResultItem,
  MatchedLine,
} from '@/utils/fileSearcherTypes'

// ============ 状态 ============
type SearchState = 'idle' | 'searching' | 'completed' | 'failed' | 'cancelled'
const state = ref<SearchState>('idle')
const searching = computed(() => state.value === 'searching')
const completed = computed(() => state.value === 'completed')

const searchPath = ref('')
const opts = reactive<SearchOptions>({
  mode: 'filename',
  query: '',
  caseSensitive: false,
  extensions: [],
  excludeExtensions: [],
  includeHidden: false,
  maxContentFileBytes: 10 * 1024 * 1024,
})
const extFilterText = ref('')
const maxContentMb = ref(10)

const searchId = ref('')
const progress = ref<SearchProgress | null>(null)
const summary = ref<SearchSummary | null>(null)
const searchError = ref('')
const elapsedMs = ref(0)
const resultItems = ref<SearchResultItem[]>([])
const totalResults = ref(0)
const currentPage = ref(1)
const pageSize = 100

let timerId: ReturnType<typeof setInterval> | null = null
let unlistenFns: UnlistenFn[] = []
const startTime = ref(0)

// ============ 工具函数 ============
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  return `${m}m${s % 60}s`
}

function formatTime(ms: number): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function highlightLine(ml: MatchedLine): string {
  const escaped = escapeHtml(ml.lineText)
  // 反向应用 matchRanges 以避免偏移变化（先替换后面的）
  const sorted = [...ml.matchRanges].sort((a, b) => b[0] - a[0])
  let result = ''
  // 转成数组操作字符
  const chars = Array.from(escaped)
  // ponytail: escapeHtml 后偏移会变，这里简化为对原始 lineText 做标记再 escape
  // 改用：对原始文本按 ranges 反向插入 <mark> 标签，再整体 escape（标记不转义）
  const origChars = Array.from(ml.lineText)
  for (const [start, end] of sorted) {
    origChars.splice(end, 0, ...Array.from('</mark>'))
    origChars.splice(start, 0, ...Array.from('<mark>'))
  }
  // 对非标签部分 escape：简单方案是先 join 再 escape，但会转义 <mark>
  // 正确方案：分段 escape
  result = origChars.join('')
  // escape 非标签文本（保留 <mark></mark>）
  result = result.replace(/(<mark>|<\/mark>)|([^<]+)/g, (_, tag, text) => {
    if (tag) return tag
    return escapeHtml(text)
  })
  return result
}

// ============ 持久化 ============
const STORAGE_KEY_PATH = 'litobox.fileSearcher.lastPath'
const STORAGE_KEY_OPTS = 'litobox.fileSearcher.lastOpts'

function saveOpts() {
  localStorage.setItem(
    STORAGE_KEY_OPTS,
    JSON.stringify({
      mode: opts.mode,
      caseSensitive: opts.caseSensitive,
      extFilterText: extFilterText.value,
      includeHidden: opts.includeHidden,
      maxContentMb: maxContentMb.value,
    })
  )
}

function loadLastPath() {
  const last = localStorage.getItem(STORAGE_KEY_PATH)
  if (last) {
    searchPath.value = last
    ElMessage.success('已加载上次路径')
  } else {
    ElMessage.info('无上次路径记录')
  }
}

function loadOpts() {
  const raw = localStorage.getItem(STORAGE_KEY_OPTS)
  if (!raw) return
  try {
    const saved = JSON.parse(raw)
    opts.mode = saved.mode ?? 'filename'
    opts.caseSensitive = saved.caseSensitive ?? false
    extFilterText.value = saved.extFilterText ?? ''
    opts.includeHidden = saved.includeHidden ?? false
    maxContentMb.value = saved.maxContentMb ?? 10
  } catch {
    // 忽略损坏的配置
  }
}

// ============ 扩展名解析 ============
function parseExtFilter(text: string): { inc: string[]; exc: string[] } {
  const tokens = text
    .split(',')
    .map((s) => s.trim().trimStartMatches('.').toLowerCase())
    .filter((s) => s.length > 0)
  if (tokens.length === 0) return { inc: [], exc: [] }
  const isExclude = tokens.some((t) => t.startsWith('!'))
  if (isExclude) {
    return { inc: [], exc: tokens.map((t) => t.replace(/^!/, '')) }
  }
  return { inc: tokens, exc: [] }
}

// ============ 文件夹选择 ============
async function selectFolder() {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    searchPath.value = selected as string
  }
}

// ============ 搜索流程 ============
async function startSearch() {
  if (!searchPath.value || !opts.query) return

  // 扩展名过滤解析
  const { inc, exc } = parseExtFilter(extFilterText.value)
  if (inc.length > 0 && exc.length > 0) {
    ElMessage.warning('扩展名不能同时包含和排除，请只用一种模式')
    return
  }
  opts.extensions = inc
  opts.excludeExtensions = exc
  opts.maxContentFileBytes = maxContentMb.value * 1024 * 1024

  // 重置状态
  searchError.value = ''
  summary.value = null
  resultItems.value = []
  totalResults.value = 0
  currentPage.value = 1
  elapsedMs.value = 0
  progress.value = null

  // 持久化
  localStorage.setItem(STORAGE_KEY_PATH, searchPath.value)
  saveOpts()

  try {
    const id = await fileSearchStart(searchPath.value, opts)
    searchId.value = id
    state.value = 'searching'
    startTime.value = Date.now()
    startTimer()
  } catch (e: any) {
    searchError.value = String(e)
    state.value = 'failed'
  }
}

async function cancelSearch() {
  if (!searchId.value) return
  try {
    await fileSearchCancel(searchId.value)
  } catch (e: any) {
    ElMessage.error('取消失败: ' + String(e))
  }
}

async function loadResults(page: number) {
  if (!searchId.value) return
  const offset = (page - 1) * pageSize
  try {
    const page_data = await fileSearchGetResults(searchId.value, pageSize, offset)
    resultItems.value = page_data.items
    totalResults.value = page_data.total
  } catch (e: any) {
    ElMessage.error('加载结果失败: ' + String(e))
  }
}

async function locateInExplorer(row: SearchResultItem) {
  try {
    await invoke('disk_locate_in_explorer', { path: row.path })
  } catch (e: any) {
    ElMessage.error('定位失败: ' + String(e))
  }
}

// ============ 计时器 ============
function startTimer() {
  stopTimer()
  timerId = setInterval(() => {
    elapsedMs.value = Date.now() - startTime.value
  }, 200)
}
function stopTimer() {
  if (timerId) {
    clearInterval(timerId)
    timerId = null
  }
}

// ============ 事件监听 ============
onMounted(async () => {
  loadOpts()

  unlistenFns.push(
    await listen<SearchProgress>('file-search-progress', (e) => {
      progress.value = e.payload
    })
  )

  unlistenFns.push(
    await listen<SearchSummary>('file-search-complete', async (e) => {
      stopTimer()
      summary.value = e.payload
      elapsedMs.value = e.payload.durationMs
      // 状态查询：判断成功/失败/取消
      if (searchId.value) {
        try {
          const status = await invoke<any>('file_search_status', { searchId: searchId.value })
          if (status.status === 'failed') {
            state.value = 'failed'
            searchError.value = status.error || '搜索失败'
          } else if (status.status === 'cancelled') {
            state.value = 'cancelled'
          } else {
            state.value = 'completed'
            await loadResults(1)
          }
        } catch {
          state.value = 'completed'
          await loadResults(1)
        }
      }
    })
  )

  unlistenFns.push(
    await listen<{ searchId: string; message: string }>('file-search-warning', (e) => {
      ElMessage.warning(e.payload.message)
    })
  )
})

onUnmounted(() => {
  stopTimer()
  unlistenFns.forEach((fn) => fn())
  unlistenFns = []
  // 释放后端内存
  if (searchId.value) {
    fileSearchCancel(searchId.value).catch(() => {})
    fileSearchClear(searchId.value).catch(() => {})
  }
})
</script>

<style scoped>
.progress-info {
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--text-secondary, #909399);
}
.progress-info div {
  margin-bottom: 4px;
}
.progress-info div:last-child {
  color: var(--text-primary, #409eff);
}
.summary-text {
  font-size: 12px;
  color: var(--text-secondary, #909399);
}
.warn-text {
  color: var(--el-color-warning, #e6a23c);
  margin-left: 8px;
}
.file-name {
  font-size: 13px;
}
.file-path {
  font-size: 11px;
  color: var(--text-secondary, #909399);
  margin-top: 2px;
  word-break: break-all;
}
.file-meta {
  font-size: 11px;
  color: var(--text-secondary, #909399);
  margin-top: 2px;
}
.match-line {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  margin-bottom: 4px;
  word-break: break-all;
}
.match-line :deep(mark) {
  background-color: var(--el-color-warning-light-7, #fdf6ec);
  color: var(--el-color-danger, #f56c6c);
  padding: 0 2px;
  border-radius: 2px;
}
.line-no {
  color: var(--text-secondary, #909399);
  margin-right: 4px;
}
.hint {
  font-size: 12px;
  color: var(--text-secondary, #909399);
  margin-left: 4px;
}
</style>
```

- [ ] **Step 2: 暂不提交（与下个任务一起）**

---

## Task 10: 前端注册（App.vue + store）

**Files:**
- Modify: `src/App.vue`
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 App.vue 的 toolComponentMap 加 fileSearcher**

在 `src/App.vue` 找到 `import DiskSpaceAnalyzer from '@/views/DiskSpaceAnalyzer.vue'`（约 line 74），在其后添加：

```typescript
import FileSearcher from '@/views/FileSearcher.vue'
```

然后在 `toolComponentMap` 中找到 `diskAnalyzer: DiskSpaceAnalyzer,`（约 line 121），在其后添加：

```typescript
  diskAnalyzer: DiskSpaceAnalyzer,
  fileSearcher: FileSearcher,
}
```

- [ ] **Step 2: 在 store/index.ts 的 TOOL_LIST 末尾加 fileSearcher 条目**

在 `src/store/index.ts` 找到 `diskAnalyzer` 条目（约 line 87，TOOL_LIST 数组最后一项），在其后、数组结束 `]` 之前添加：

```typescript
  { id: 'diskAnalyzer', name: '磁盘分析', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><line x1="12" y1="2" x2="12" y2="6"/><line x1="12" y1="18" x2="12" y2="22"/><line x1="2" y1="12" x2="6" y2="12"/><line x1="18" y1="12" x2="22" y2="12"/></svg>`, description: '分析磁盘空间占用，查找大文件和重复文件', keywords: ['磁盘', '空间', '重复', '清理', 'disk', 'space', 'duplicate'], category: 'system' },
  { id: 'fileSearcher', name: '全文搜索', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`, description: '按文件名或内容搜索，支持正则表达式，类似 Everything + grep', keywords: ['搜索', '全文', '文件名', '内容', 'grep', 'find', 'search'], category: 'system' },
]
```

- [ ] **Step 3: 验证前端编译**

Run:
```powershell
cd d:\work\codes\litobox
npm run build
```
Expected: vue-tsc 类型检查通过，vite build 成功。

- [ ] **Step 4: 提交**

```powershell
cd d:\work\codes\litobox
git add src/utils/fileSearcherTypes.ts src/utils/fileSearcherClient.ts src/views/FileSearcher.vue src/App.vue src/store/index.ts
git commit -m "feat(file-searcher): 前端 types + client + FileSearcher.vue 页面 + 注册"
```

---

## Task 11: 版本号同步 + README + backlog

**Files:**
- Modify: `package.json`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 更新 package.json 版本号**

在 `package.json` 中将 `"version": "4.5.0"` 改为 `"version": "4.6.0"`。

- [ ] **Step 2: 在 README.md 版本表加 V4.6 行**

在 `README.md` 找到 V4.5 行（约 line 305）：

```
| V4.5 | ✅ | 多 Tab 导航模式（顶部 Tab 栏、同时打开多个工具、KeepAlive 状态保留、右键菜单关闭其他/全部、LRU 上限 8 个） |
```

在其后添加：

```
| V4.6 | ✅ | 全文搜索工具（文件名/内容双模式、正则表达式、进度取消、编码自动识别 UTF-8/GBK/UTF-16） |
```

- [ ] **Step 3: 在 feature-backlog.md 把 B1 移到已完成版本表**

在 `docs/superpowers/plans/feature-backlog.md`：

1. 找到已完成版本表（约 line 24 的 V4.4 行），在其后添加：
```
| V4.6 | ✅ | 全文搜索（文件名/内容双模式、正则、进度取消、编码识别） | 2026-07-07 |
```

2. 在候选方向池的 B 类表格中，把 B1 行标记为已完成（约 line 53）：
将 `| B1 | **全文搜索** | 指定目录下按文件名/内容搜索，支持正则，类似 Everything + grep | 高 | 2026-07-06 brainstorming |`
改为 `| B1 | ✅ 全文搜索 | 指定目录下按文件名/内容搜索，支持正则，类似 Everything + grep | — 已完成 V4.6 — | 2026-07-06 brainstorming |`

- [ ] **Step 4: 提交**

```powershell
cd d:\work\codes\litobox
git add package.json README.md docs/superpowers/plans/feature-backlog.md
git commit -m "docs: V4.6 版本号同步 + README + backlog 更新（全文搜索已完成）"
```

---

## 验收清单（实现完成后手动逐项验证）

参照 spec `## 手动验收清单` 的 12 项：

1. 文件名模式：搜 `.*\.rs` 在 litobox 项目根，能找到所有 .rs 文件
2. 内容模式：搜 `debug_log!` 在 `src-tauri/src/`，结果含命中行预览且高亮位置正确
3. 中文内容：在 GBK 编码的 .txt 中搜中文关键词，能命中（验证 GBK 回退）
4. UTF-16 文件：用记事本另存为"Unicode"的 .txt 搜内容，能命中（验证 BOM 优先于 \0 检测）
5. 二进制跳过：搜 .exe 目录，二进制文件不计入内容命中但可文件名匹配
6. 扩展名过滤：`ts,vue` 只搜这两类；`!exe,dll` 排除这两类
7. 取消：大目录搜索中点取消，能立即停止并切 cancelled 状态
8. 截断：构造 >1000 命中，结果停在 1000 且 summary.truncated=true
9. 定位：点结果"定位"按钮，资源管理器打开并选中该文件
10. 分页：>100 条结果时翻页正常
11. 错误路径：不存在路径 / 无效正则（如 `[`）显示友好错误
12. 释放：离开页面后再次进入，能正常新搜索（验证 `file_search_clear` 清理）

---

## 自审

- **Spec 覆盖**：架构/数据结构/命令接口/前端UI/编码处理/测试策略 6 节均有对应 Task ✓
- **占位符**：无 TBD/TODO，所有步骤含完整代码 ✓
- **类型一致性**：`SearchOptions`/`SearchResultItem`/`MatchedLine` 等在 Rust 与 TS 两侧字段名一致（camelCase）；`file_search_*` 命令名与 client 函数名一致 ✓
- **TDD**：Task 2/4 走"先写失败测试→实现→通过"流程；Task 5/6 的 run_search 与命令层为集成代码，靠手动验收覆盖（与 disk_analyzer 一致）✓
