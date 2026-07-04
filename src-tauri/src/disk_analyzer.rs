use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Lazy, Mutex};
use tauri::{AppHandle, Emitter};

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
// 用法: debug_log!("查询失败: {}", err)
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*)
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
static SCANS: Lazy<Mutex<HashMap<String, Arc<Mutex<ScanResults>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 获取指定 scan_id 的扫描结果 Arc（克隆 Arc，不持有锁）
fn get_scan(scan_id: &str) -> Option<Arc<Mutex<ScanResults>>> {
    let scans = SCANS.lock().unwrap();
    scans.get(scan_id).cloned()
}

/// 插入新扫描
fn insert_scan(scan_id: String, results: Arc<Mutex<ScanResults>>) {
    let mut scans = SCANS.lock().unwrap();
    scans.insert(scan_id, results);
}

/// 移除扫描（释放内存）
fn remove_scan(scan_id: &str) -> bool {
    let mut scans = SCANS.lock().unwrap();
    scans.remove(scan_id).is_some()
}
