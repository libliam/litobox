use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销），沿用项目惯例
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum DiffState {
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffEntry {
    pub path: String,
    /// added | removed | modified | same
    pub status: String,
    pub left_size: Option<u64>,
    pub right_size: Option<u64>,
    pub left_modified: Option<i64>,
    pub right_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffSummary {
    pub added: u64,
    pub removed: u64,
    pub modified: u64,
    pub same: u64,
    pub total: u64,
    pub skipped: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffProgress {
    pub scan_id: String,
    pub scanned: u64,
    pub current_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffComplete {
    pub scan_id: String,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffPage {
    pub items: Vec<DiffEntry>,
    pub total: u64,
}

#[derive(Debug)]
struct FileMeta {
    size: u64,
    modified_ms: i64,
}

#[derive(Debug)]
pub struct DiffResults {
    pub scan_id: String,
    pub left_path: String,
    pub right_path: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub state: DiffState,
    pub cancel_flag: Arc<AtomicBool>,
    pub scanned: u64,
    pub current_path: String,
    pub skipped_total: u32,
    pub entries: Vec<DiffEntry>,
    pub summary: Option<DiffSummary>,
}

// ponytail: 全局对比结果存储，沿用 disk_analyzer.rs 的 OnceLock + Mutex 惯例
static DIFFS: OnceLock<Mutex<HashMap<String, Arc<Mutex<DiffResults>>>>> = OnceLock::new();

fn diffs() -> &'static Mutex<HashMap<String, Arc<Mutex<DiffResults>>>> {
    DIFFS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_diff(scan_id: &str) -> Option<Arc<Mutex<DiffResults>>> {
    diffs().lock().unwrap().get(scan_id).cloned()
}

fn insert_diff(scan_id: String, results: Arc<Mutex<DiffResults>>) {
    diffs().lock().unwrap().insert(scan_id, results);
}

fn remove_diff(scan_id: &str) -> bool {
    diffs().lock().unwrap().remove(scan_id).is_some()
}

// ============ 忽略规则匹配 ============

/// 简单 glob 匹配：支持 `*` 和 `?`。
/// 含路径分隔符的规则匹配相对路径；不含的规则匹配文件名（或路径中任意一段），大小写不敏感。
struct IgnoreMatcher {
    regexes: Vec<regex::Regex>,
}

impl IgnoreMatcher {
    fn new(patterns: &[String]) -> Self {
        let regexes = patterns
            .iter()
            .filter_map(|raw| {
                // 去掉尾部斜杠：`dist/` 与 `dist` 等价，否则 `^dist/$` 永远匹配不到路径
                let p = raw.trim().replace('\\', "/");
                let p = p.trim_end_matches('/');
                if p.is_empty() {
                    return None;
                }
                let has_sep = p.contains('/');
                let mut re = String::new();
                for ch in p.chars() {
                    match ch {
                        '*' => re.push_str(".*"),
                        '?' => re.push('.'),
                        c => re.push_str(&regex::escape(&c.to_string())),
                    }
                }
                let full = if has_sep {
                    format!("^{}$", re)
                } else {
                    format!("(^|/){}$", re)
                };
                regex::Regex::new(&format!("(?i){}", full)).ok()
            })
            .collect();
        Self { regexes }
    }

    fn matches(&self, rel_path: &str, name: &str) -> bool {
        self.regexes
            .iter()
            .any(|re| re.is_match(rel_path) || re.is_match(name))
    }
}

// ============ 对比核心逻辑 ============

fn modified_ms(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// 递归采集一棵树。返回 Ok(false) 表示已被取消（状态已置为 Cancelled）。
fn collect_tree(
    root: &Path,
    ignore: &IgnoreMatcher,
    state: &Arc<Mutex<DiffResults>>,
    app: &AppHandle,
    side: &str,
    out: &mut HashMap<String, FileMeta>,
) -> Result<bool, String> {
    let mut last_emit = Instant::now();
    let mut walker = WalkDir::new(root).into_iter();

    while let Some(entry) = walker.next() {
        if state.lock().unwrap().cancel_flag.load(Ordering::SeqCst) {
            debug_log!("dir_diff: 对比被取消");
            let mut s = state.lock().unwrap();
            s.state = DiffState::Cancelled;
            s.finished_at = Some(now_ms());
            return Ok(false);
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug_log!("dir_diff: 跳过无权限目录: {}", e);
                state.lock().unwrap().skipped_total += 1;
                continue;
            }
        };

        let path = entry.path();
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let name = entry.file_name().to_string_lossy().to_string();

        if ignore.matches(&rel_str, &name) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }

        if entry.file_type().is_file() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            out.insert(
                rel_str.clone(),
                FileMeta {
                    size: meta.len(),
                    modified_ms: modified_ms(&meta),
                },
            );
        }

        {
            let mut s = state.lock().unwrap();
            s.scanned += 1;
            s.current_path = format!("[{}] {}", side, rel_str);
        }

        if last_emit.elapsed() > Duration::from_millis(400) {
            let s = state.lock().unwrap();
            let _ = app.emit(
                "dir-diff-progress",
                DiffProgress {
                    scan_id: s.scan_id.clone(),
                    scanned: s.scanned,
                    current_path: s.current_path.clone(),
                },
            );
            drop(s);
            last_emit = Instant::now();
        }
    }

    Ok(true)
}

/// 计算文件 sha256（仅 compare_content 开启时用于同尺寸文件的精确比对）
fn hash_file(path: &Path) -> Option<Vec<u8>> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut f = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hasher.finalize().to_vec())
}

fn run_diff(
    state: Arc<Mutex<DiffResults>>,
    app: AppHandle,
    ignore_patterns: Vec<String>,
    compare_content: bool,
) -> Result<(), String> {
    let (left_root, right_root) = {
        let s = state.lock().unwrap();
        (PathBuf::from(&s.left_path), PathBuf::from(&s.right_path))
    };

    for (label, root) in [("左侧", &left_root), ("右侧", &right_root)] {
        if !root.exists() {
            let err = format!("{}路径不存在: {}", label, root.display());
            let mut s = state.lock().unwrap();
            s.state = DiffState::Failed { error: err.clone() };
            s.finished_at = Some(now_ms());
            return Err(err);
        }
    }

    let ignore = IgnoreMatcher::new(&ignore_patterns);
    debug_log!(
        "dir_diff: 开始对比 {} <-> {} (忽略 {} 条规则, 内容比对={})",
        left_root.display(),
        right_root.display(),
        ignore_patterns.len(),
        compare_content
    );

    let mut left_map: HashMap<String, FileMeta> = HashMap::new();
    let mut right_map: HashMap<String, FileMeta> = HashMap::new();

    if !collect_tree(&left_root, &ignore, &state, &app, "左", &mut left_map)? {
        return Ok(());
    }
    if !collect_tree(&right_root, &ignore, &state, &app, "右", &mut right_map)? {
        return Ok(());
    }

    let mut entries: Vec<DiffEntry> = Vec::new();
    let mut added: u64 = 0;
    let mut removed: u64 = 0;
    let mut modified: u64 = 0;
    let mut same: u64 = 0;

    for (path, lm) in left_map.iter() {
        match right_map.get(path) {
            None => {
                removed += 1;
                entries.push(DiffEntry {
                    path: path.clone(),
                    status: "removed".into(),
                    left_size: Some(lm.size),
                    right_size: None,
                    left_modified: Some(lm.modified_ms),
                    right_modified: None,
                });
            }
            Some(rm) => {
                // 尺寸不同直接判为已修改；尺寸相同则仅在开启内容比对时用 sha256 精确判定
                let is_modified = if lm.size != rm.size {
                    true
                } else if compare_content {
                    let lh = hash_file(&left_root.join(path));
                    let rh = hash_file(&right_root.join(path));
                    match (lh, rh) {
                        (Some(a), Some(b)) => a != b,
                        _ => true,
                    }
                } else {
                    false
                };
                let status = if is_modified {
                    modified += 1;
                    "modified"
                } else {
                    same += 1;
                    "same"
                };
                entries.push(DiffEntry {
                    path: path.clone(),
                    status: status.into(),
                    left_size: Some(lm.size),
                    right_size: Some(rm.size),
                    left_modified: Some(lm.modified_ms),
                    right_modified: Some(rm.modified_ms),
                });
            }
        }
    }

    for (path, rm) in right_map.iter() {
        if !left_map.contains_key(path) {
            added += 1;
            entries.push(DiffEntry {
                path: path.clone(),
                status: "added".into(),
                left_size: None,
                right_size: Some(rm.size),
                left_modified: None,
                right_modified: Some(rm.modified_ms),
            });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let (scan_id, skipped, started_at) = {
        let s = state.lock().unwrap();
        (s.scan_id.clone(), s.skipped_total, s.started_at)
    };
    let summary = DiffSummary {
        added,
        removed,
        modified,
        same,
        total: entries.len() as u64,
        skipped,
        duration_ms: (now_ms() - started_at).max(0) as u64,
    };

    debug_log!(
        "dir_diff: 完成 新增={} 删除={} 修改={} 相同={}",
        added,
        removed,
        modified,
        same
    );

    {
        let mut s = state.lock().unwrap();
        s.entries = entries;
        s.summary = Some(summary.clone());
        s.state = DiffState::Completed;
        s.finished_at = Some(now_ms());
    }

    let _ = app.emit(
        "dir-diff-complete",
        DiffComplete {
            scan_id,
            summary,
        },
    );

    Ok(())
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn dir_diff_start(
    app: AppHandle,
    left: String,
    right: String,
    ignore: Option<Vec<String>>,
    compare_content: Option<bool>,
) -> Result<String, String> {
    let left_canonical = PathBuf::from(&left)
        .canonicalize()
        .map_err(|e| format!("左侧路径无法访问: {}", e))?
        .to_string_lossy()
        .to_string();
    let right_canonical = PathBuf::from(&right)
        .canonicalize()
        .map_err(|e| format!("右侧路径无法访问: {}", e))?
        .to_string_lossy()
        .to_string();

    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let results = DiffResults {
        scan_id: scan_id.clone(),
        left_path: left_canonical.clone(),
        right_path: right_canonical.clone(),
        started_at: now_ms(),
        finished_at: None,
        state: DiffState::Running,
        cancel_flag,
        scanned: 0,
        current_path: String::new(),
        skipped_total: 0,
        entries: Vec::new(),
        summary: None,
    };
    let results_arc = Arc::new(Mutex::new(results));
    insert_diff(scan_id.clone(), results_arc.clone());

    let ignore_patterns = ignore.unwrap_or_default();
    let compare = compare_content.unwrap_or(false);

    debug_log!(
        "dir_diff: start id={} {} <-> {}",
        scan_id,
        left_canonical,
        right_canonical
    );

    let app_clone = app.clone();
    let scan_id_clone = scan_id.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_diff(results_arc.clone(), app_clone, ignore_patterns, compare) {
            debug_log!("dir_diff: 对比失败 id={} err={}", scan_id_clone, e);
            let mut r = results_arc.lock().unwrap();
            r.state = DiffState::Failed { error: e };
            r.finished_at = Some(now_ms());
        }
    });

    Ok(scan_id)
}

#[tauri::command]
pub fn dir_diff_cancel(scan_id: String) -> Result<(), String> {
    debug_log!("dir_diff: cancel id={}", scan_id);
    if let Some(arc) = get_diff(&scan_id) {
        arc.lock().unwrap().cancel_flag.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("diff not found or expired".into())
    }
}

#[tauri::command]
pub fn dir_diff_status(scan_id: String) -> Result<DiffState, String> {
    let arc = get_diff(&scan_id).ok_or("diff not found or expired")?;
    let state = arc.lock().unwrap().state.clone();
    Ok(state)
}

#[tauri::command]
pub fn dir_diff_get_summary(scan_id: String) -> Result<DiffSummary, String> {
    let arc = get_diff(&scan_id).ok_or("diff not found or expired")?;
    let s = arc.lock().unwrap();
    s.summary
        .clone()
        .ok_or_else(|| "对比尚未完成".to_string())
}

#[tauri::command]
pub fn dir_diff_get_entries(
    scan_id: String,
    status: Option<String>,
    keyword: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<DiffPage, String> {
    let arc = get_diff(&scan_id).ok_or("diff not found or expired")?;
    let s = arc.lock().unwrap();
    let limit = limit.unwrap_or(200).min(2000) as usize;
    let offset = offset.unwrap_or(0) as usize;
    let status_filter = status.filter(|v| !v.is_empty() && v != "all");
    let kw = keyword.unwrap_or_default().to_lowercase();

    let filtered: Vec<&DiffEntry> = s
        .entries
        .iter()
        .filter(|e| {
            if let Some(st) = &status_filter {
                if &e.status != st {
                    return false;
                }
            }
            if !kw.is_empty() && !e.path.to_lowercase().contains(&kw) {
                return false;
            }
            true
        })
        .collect();

    let total = filtered.len() as u64;
    let items = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();

    Ok(DiffPage { items, total })
}

#[tauri::command]
pub fn dir_diff_clear(scan_id: String) -> Result<(), String> {
    debug_log!("dir_diff: clear id={}", scan_id);
    remove_diff(&scan_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(path: &str, status: &str) -> DiffEntry {
        DiffEntry {
            path: path.into(),
            status: status.into(),
            left_size: Some(1),
            right_size: Some(1),
            left_modified: Some(0),
            right_modified: Some(0),
        }
    }

    #[test]
    fn ignore_matcher_globs() {
        let m = IgnoreMatcher::new(&[
            "node_modules".into(),
            "*.log".into(),
            "dist/".into(),
            "src/target".into(),
        ]);
        // 无分隔符规则匹配任意层级的同名项
        assert!(m.matches("node_modules", "node_modules"));
        assert!(m.matches("a/node_modules", "node_modules"));
        assert!(m.matches("logs/app.log", "app.log"));
        // 尾部斜杠规则等价于无斜杠规则
        assert!(m.matches("dist", "dist"));
        assert!(m.matches("a/dist", "dist"));
        // 带路径的规则只匹配该相对路径
        assert!(m.matches("src/target", "target"));
        assert!(!m.matches("other/target", "target"));
        assert!(!m.matches("main.rs", "main.rs"));
    }

    #[test]
    fn entries_filter_and_page() {
        let scan_id = "test-scan".to_string();
        let results = DiffResults {
            scan_id: scan_id.clone(),
            left_path: "L".into(),
            right_path: "R".into(),
            started_at: 0,
            finished_at: Some(0),
            state: DiffState::Completed,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            scanned: 4,
            current_path: String::new(),
            skipped_total: 0,
            entries: vec![
                test_entry("a.txt", "modified"),
                test_entry("b.log", "added"),
                test_entry("dir/c.txt", "same"),
                test_entry("z.txt", "removed"),
            ],
            summary: None,
        };
        insert_diff(scan_id.clone(), Arc::new(Mutex::new(results)));

        let all = dir_diff_get_entries(scan_id.clone(), None, None, None, None).unwrap();
        assert_eq!(all.total, 4);

        let added = dir_diff_get_entries(scan_id.clone(), Some("added".into()), None, None, None).unwrap();
        assert_eq!(added.total, 1);
        assert_eq!(added.items[0].path, "b.log");

        let kw = dir_diff_get_entries(scan_id.clone(), None, Some("TXT".into()), None, None).unwrap();
        assert_eq!(kw.total, 3);

        let page = dir_diff_get_entries(scan_id.clone(), None, None, Some(2), Some(1)).unwrap();
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].path, "b.log");

        remove_diff(&scan_id);
        assert!(dir_diff_get_entries(scan_id, None, None, None, None).is_err());
    }
}
