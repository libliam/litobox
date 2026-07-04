use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
// 用法: debug_log!("查询失败: {}", err)
// 注：用 cfg!() 而非 #[cfg] 属性，避免 experimental feature
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Deserialize)]
pub struct ScanOptions {
    pub includeHidden: bool,
    pub detectDuplicates: bool,
    pub maxFiles: Option<u64>,
    pub followSymlinks: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            includeHidden: false,
            detectDuplicates: false,
            maxFiles: None,
            followSymlinks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ScanStatus {
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderInfo {
    pub path: String,
    pub parent: Option<String>,
    pub name: String,
    pub depth: u32,
    pub fileCount: u64,
    pub sizeBytes: u64,
    pub percentOfRoot: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub sizeBytes: u64,
    pub modifiedMs: i64,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub fileCount: u64,
    pub totalSize: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub groupId: u32,
    pub fileSize: u64,
    pub fileCount: u32,
    pub wastedBytes: u64,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanSummary {
    pub totalFiles: u64,
    pub totalDirs: u64,
    pub totalSize: u64,
    pub skippedCount: u32,
    pub durationMs: u64,
    pub duplicatesWastedBytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteFailure {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<DeleteFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderPage {
    pub items: Vec<FolderInfo>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FilePage {
    pub items: Vec<FileInfo>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtStatPage {
    pub items: Vec<ExtensionStat>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DupPage {
    pub items: Vec<DuplicateGroup>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub scanId: String,
    pub filesScanned: u64,
    pub bytesScanned: u64,
    pub currentPath: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanComplete {
    pub scanId: String,
    pub summary: ScanSummary,
}

// ============ 内部状态（不暴露给前端） ============

#[derive(Debug)]
pub struct ScanResults {
    pub scan_id: String,
    pub root_path: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub status: ScanStatus,
    pub cancel_flag: Arc<AtomicBool>,

    // 流式累积
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_path: Option<String>,
    pub skipped_dirs: Vec<String>,
    pub skipped_total: u32,

    // 完成后填充
    pub folders: Vec<FolderInfo>,
    pub top_files: Vec<FileInfo>,
    pub ext_stats: Vec<ExtensionStat>,
    pub duplicates: Vec<DuplicateGroup>,
}

// ponytail: 全局扫描结果存储。OnceLock + Mutex 模式，沿用项目惯例（clipboard.rs 也用模块级 static）
// 升级路径：若并发扫描数经常 >10，可改用 Tauri managed state + Arc
static SCANS: OnceLock<Mutex<HashMap<String, Arc<Mutex<ScanResults>>>>> = OnceLock::new();

fn scans() -> &'static Mutex<HashMap<String, Arc<Mutex<ScanResults>>>> {
    SCANS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 获取指定 scan_id 的扫描结果 Arc（克隆 Arc，不持有锁）
fn get_scan(scan_id: &str) -> Option<Arc<Mutex<ScanResults>>> {
    let scans = scans().lock().unwrap();
    scans.get(scan_id).cloned()
}

/// 插入新扫描
fn insert_scan(scan_id: String, results: Arc<Mutex<ScanResults>>) {
    let mut s = scans().lock().unwrap();
    s.insert(scan_id, results);
}

/// 移除扫描（释放内存）
fn remove_scan(scan_id: &str) -> bool {
    let mut s = scans().lock().unwrap();
    s.remove(scan_id).is_some()
}

// ============ 扫描核心逻辑 ============

const TOP_FILES_LIMIT: usize = 5000;
const SKIPPED_DIRS_LIMIT: usize = 1000;
const CANCEL_CHECK_INTERVAL: u64 = 1000;

/// 维护 Top N 大文件的最小堆辅助：用 Vec + sort + truncate 简化
/// ponytail: 真正最小堆用 BinaryHeap，但 Vec 排序在 N=5000 时性能足够，且代码更简单
/// 升级路径：BinaryHeap
fn maybe_insert_top_file(top: &mut Vec<FileInfo>, file: FileInfo) {
    if top.len() < TOP_FILES_LIMIT {
        top.push(file);
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
        return;
    }
    let last = top.last().unwrap();
    if file.sizeBytes > last.sizeBytes {
        top.pop();
        top.push(file);
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
    }
}

/// 同步扫描指定路径，更新 results
fn run_scan(
    results_arc: Arc<Mutex<ScanResults>>,
    app: AppHandle,
    opts: ScanOptions,
) -> Result<(), String> {
    let root_path = {
        let r = results_arc.lock().unwrap();
        r.root_path.clone()
    };
    let root = PathBuf::from(&root_path);
    if !root.exists() {
        let err = format!("路径不存在: {}", root_path);
        let mut r = results_arc.lock().unwrap();
        r.status = ScanStatus::Failed { error: err.clone() };
        r.finished_at = Some(now_ms());
        return Err(err);
    }

    debug_log!("disk_analyzer: 开始扫描 {}", root_path);

    let mut folder_map: HashMap<String, (u64, u64)> = HashMap::new();
    let mut ext_map: HashMap<String, (u64, u64)> = HashMap::new();
    let mut top_files: Vec<FileInfo> = Vec::with_capacity(TOP_FILES_LIMIT);
    let mut all_files_by_size: HashMap<u64, Vec<String>> = HashMap::new();
    let mut total_files: u64 = 0;
    let mut total_dirs: u64 = 0;
    let mut files_since_check: u64 = 0;
    let mut last_progress_emit = std::time::Instant::now();

    let mut walker = walkdir::WalkDir::new(&root)
        .follow_links(opts.followSymlinks)
        .into_iter();

    while let Some(entry) = walker.next() {
        // 取消检查
        {
            let r = results_arc.lock().unwrap();
            if r.cancel_flag.load(Ordering::SeqCst) {
                debug_log!("disk_analyzer: 扫描被取消");
                let mut r = results_arc.lock().unwrap();
                r.status = ScanStatus::Cancelled;
                r.finished_at = Some(now_ms());
                return Ok(());
            }
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                debug_log!("disk_analyzer: 跳过无权限目录: {}", e);
                let mut r = results_arc.lock().unwrap();
                r.skipped_total += 1;
                if r.skipped_dirs.len() < SKIPPED_DIRS_LIMIT {
                    r.skipped_dirs.push(format!("{}", e));
                }
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
            folder_map.entry(path_str.clone()).or_insert((0, 0));
            if let Some(parent) = path.parent() {
                folder_map
                    .entry(parent.to_string_lossy().to_string())
                    .or_insert((0, 0));
            }
            if !opts.includeHidden && is_hidden(entry.file_name()) {
                walker.skip_current_dir();
                continue;
            }
        } else if entry.file_type().is_file() {
            if !opts.includeHidden && is_hidden(entry.file_name()) {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = meta.len();
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let name = entry.file_name().to_string_lossy().to_string();
            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            // 累加到所有父目录
            let mut p = path.parent();
            while let Some(parent) = p {
                let parent_str = parent.to_string_lossy().to_string();
                if let Some((cnt, sz)) = folder_map.get_mut(&parent_str) {
                    *cnt += 1;
                    *sz += size;
                }
                p = parent.parent();
            }

            // 扩展名聚合
            let ext_entry = ext_map.entry(extension.clone()).or_insert((0, 0));
            ext_entry.0 += 1;
            ext_entry.1 += size;

            // Top N
            maybe_insert_top_file(
                &mut top_files,
                FileInfo {
                    path: path_str.clone(),
                    name: name.clone(),
                    sizeBytes: size,
                    modifiedMs: modified_ms,
                    extension,
                },
            );

            // 重复检测：按 size 分组
            if opts.detectDuplicates {
                all_files_by_size
                    .entry(size)
                    .or_insert_with(Vec::new)
                    .push(path_str.clone());
            }

            total_files += 1;
            {
                let mut r = results_arc.lock().unwrap();
                r.files_scanned = total_files;
                r.bytes_scanned += size;
            }
        }

        // 软上限警告
        if let Some(max) = opts.maxFiles {
            if total_files == max {
                let _ = app.emit(
                    "disk-scan-warning",
                    serde_json::json!({
                        "scanId": results_arc.lock().unwrap().scan_id,
                        "message": format!("已达软上限 {} 文件，继续扫描但仅保留 Top {}", max, TOP_FILES_LIMIT)
                    }),
                );
            }
        }

        // 进度节流（500ms 或每 1000 文件）
        files_since_check += 1;
        if files_since_check >= CANCEL_CHECK_INTERVAL
            || last_progress_emit.elapsed() > std::time::Duration::from_millis(500)
        {
            let r = results_arc.lock().unwrap();
            let _ = app.emit(
                "disk-scan-progress",
                ScanProgress {
                    scanId: r.scan_id.clone(),
                    filesScanned: r.files_scanned,
                    bytesScanned: r.bytes_scanned,
                    currentPath: r.current_path.clone().unwrap_or_default(),
                },
            );
            last_progress_emit = std::time::Instant::now();
            files_since_check = 0;
        }
    }

    debug_log!(
        "disk_analyzer: walk 完成，开始聚合。文件={}, 目录={}",
        total_files,
        total_dirs
    );

    let root_size = folder_map.get(&root_path).map(|(_, s)| *s).unwrap_or(0);
    let mut folders: Vec<FolderInfo> = folder_map
        .into_iter()
        .map(|(path, (file_count, size_bytes))| {
            let pb = PathBuf::from(&path);
            let name = pb
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let parent = pb.parent().map(|p| p.to_string_lossy().to_string());
            let depth = pb.components().count() as u32;
            let percent = if root_size > 0 {
                size_bytes as f32 / root_size as f32 * 100.0
            } else {
                0.0
            };
            FolderInfo {
                path,
                parent,
                name,
                depth,
                fileCount: file_count,
                sizeBytes: size_bytes,
                percentOfRoot: percent,
            }
        })
        .collect();
    folders.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));

    let mut ext_stats: Vec<ExtensionStat> = ext_map
        .into_iter()
        .map(|(extension, (file_count, total_size))| {
            let percent = if root_size > 0 {
                total_size as f32 / root_size as f32 * 100.0
            } else {
                0.0
            };
            ExtensionStat {
                extension,
                fileCount: file_count,
                totalSize: total_size,
                percent,
            }
        })
        .collect();
    ext_stats.sort_by(|a, b| b.totalSize.cmp(&a.totalSize));

    // 重复检测
    let mut duplicates: Vec<DuplicateGroup> = Vec::new();
    let mut dup_wasted: Option<u64> = None;
    if opts.detectDuplicates {
        debug_log!(
            "disk_analyzer: 开始重复检测，候选 size 组数={}",
            all_files_by_size.len()
        );
        let mut group_id = 0u32;
        for (size, paths) in all_files_by_size.iter() {
            if paths.len() < 2 {
                continue;
            }
            let mut fp_map: HashMap<Vec<u8>, Vec<String>> = HashMap::new();
            for path in paths {
                match compute_fingerprint(path, *size) {
                    Ok(fp) => {
                        fp_map.entry(fp).or_insert_with(Vec::new).push(path.clone());
                    }
                    Err(e) => debug_log!("disk_analyzer: 指纹计算失败 {}: {}", path, e),
                }
            }
            for (_fp, group_files) in fp_map {
                if group_files.len() < 2 {
                    continue;
                }
                group_id += 1;
                let file_count = group_files.len() as u32;
                let wasted = size * (file_count as u64 - 1);
                let files: Vec<FileInfo> = group_files
                    .iter()
                    .map(|p| {
                        let pb = PathBuf::from(p);
                        FileInfo {
                            path: p.clone(),
                            name: pb
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            sizeBytes: *size,
                            modifiedMs: 0,
                            extension: pb
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_default(),
                        }
                    })
                    .collect();
                duplicates.push(DuplicateGroup {
                    groupId: group_id,
                    fileSize: *size,
                    fileCount: file_count,
                    wastedBytes: wasted,
                    files,
                });
            }
        }
        duplicates.sort_by(|a, b| b.wastedBytes.cmp(&a.wastedBytes));
        dup_wasted = Some(duplicates.iter().map(|g| g.wastedBytes).sum());
        debug_log!("disk_analyzer: 重复检测完成，组数={}", duplicates.len());
    }

    {
        let mut r = results_arc.lock().unwrap();
        r.status = ScanStatus::Completed;
        r.finished_at = Some(now_ms());
        r.folders = folders;
        r.top_files = top_files;
        r.ext_stats = ext_stats;
        r.duplicates = duplicates;
    }

    let summary = {
        let r = results_arc.lock().unwrap();
        ScanSummary {
            totalFiles: r.files_scanned,
            totalDirs: total_dirs,
            totalSize: r.bytes_scanned,
            skippedCount: r.skipped_total,
            durationMs: (r.finished_at.unwrap_or(0) - r.started_at) as u64,
            duplicatesWastedBytes: dup_wasted,
        }
    };
    let scan_id = results_arc.lock().unwrap().scan_id.clone();
    let _ = app.emit(
        "disk-scan-complete",
        ScanComplete {
            scanId: scan_id,
            summary,
        },
    );

    Ok(())
}

/// 计算文件指纹：前 64KB 的 SHA-256
fn compute_fingerprint(path: &str, _size: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 64 * 1024];
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(n);
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&buf);
    Ok(hasher.finalize().to_vec())
}

/// 判断文件名是否隐藏（Windows: 以 . 开头，或带 Hidden 属性；简化版只查 . 开头）
fn is_hidden(name: &std::ffi::OsStr) -> bool {
    name.to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// 当前毫秒时间戳
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let content_a = vec![b'a'; 1024];
        fs::write(root.join("a.txt"), &content_a).unwrap();
        fs::write(root.join("b.txt"), &content_a).unwrap();
        let _ = fs::create_dir(root.join("sub"));
        fs::write(root.join("sub").join("c.txt"), vec![b'c'; 2048]).unwrap();
        fs::write(root.join("sub").join("d.txt"), &content_a).unwrap();
        let _ = fs::create_dir(root.join("empty"));

        dir
    }

    #[test]
    fn maybe_insert_top_file_keeps_largest() {
        let mut top = Vec::new();
        maybe_insert_top_file(
            &mut top,
            FileInfo { path: "a".into(), name: "a".into(), sizeBytes: 100, modifiedMs: 0, extension: "".into() },
        );
        maybe_insert_top_file(
            &mut top,
            FileInfo { path: "b".into(), name: "b".into(), sizeBytes: 300, modifiedMs: 0, extension: "".into() },
        );
        maybe_insert_top_file(
            &mut top,
            FileInfo { path: "c".into(), name: "c".into(), sizeBytes: 200, modifiedMs: 0, extension: "".into() },
        );
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].sizeBytes, 300);
        assert_eq!(top[1].sizeBytes, 200);
        assert_eq!(top[2].sizeBytes, 100);
    }

    #[test]
    fn maybe_insert_top_file_respects_limit() {
        let mut top = Vec::new();
        for i in 0..5 {
            top.push(FileInfo {
                path: format!("f{}", i),
                name: format!("f{}", i),
                sizeBytes: i * 10,
                modifiedMs: 0,
                extension: "".into(),
            });
        }
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
        top.pop();
        top.push(FileInfo {
            path: "new".into(),
            name: "new".into(),
            sizeBytes: 999,
            modifiedMs: 0,
            extension: "".into(),
        });
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
        assert_eq!(top[0].sizeBytes, 999);
        assert_eq!(top.len(), 5);
    }

    #[test]
    fn compute_fingerprint_same_content_same_hash() {
        let dir = make_test_tree();
        let root = dir.path();
        let fp_a = compute_fingerprint(root.join("a.txt").to_str().unwrap(), 1024).unwrap();
        let fp_b = compute_fingerprint(root.join("b.txt").to_str().unwrap(), 1024).unwrap();
        let fp_c = compute_fingerprint(root.join("sub").join("c.txt").to_str().unwrap(), 2048).unwrap();
        assert_eq!(fp_a, fp_b, "相同内容应有相同指纹");
        assert_ne!(fp_a, fp_c, "不同内容应有不同指纹");
    }

    #[test]
    fn is_hidden_detects_dot_prefix() {
        assert!(is_hidden(std::ffi::OsStr::new(".gitignore")));
        assert!(!is_hidden(std::ffi::OsStr::new("a.txt")));
    }

    #[test]
    fn scan_results_default_status_running() {
        let r = ScanResults {
            scan_id: "test".into(),
            root_path: "/tmp".into(),
            started_at: 0,
            finished_at: None,
            status: ScanStatus::Running,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            files_scanned: 0,
            bytes_scanned: 0,
            current_path: None,
            skipped_dirs: Vec::new(),
            skipped_total: 0,
            folders: Vec::new(),
            top_files: Vec::new(),
            ext_stats: Vec::new(),
            duplicates: Vec::new(),
        };
        match r.status {
            ScanStatus::Running => (),
            _ => panic!("默认状态应为 Running"),
        }
    }
}
