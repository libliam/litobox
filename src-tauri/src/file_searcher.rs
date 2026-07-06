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
            let mut r = results_arc.lock().unwrap();
            if r.cancel_flag.load(Ordering::SeqCst) {
                debug_log!("file_searcher: 搜索被取消");
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
                                        matchedLines: matched_lines,
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
