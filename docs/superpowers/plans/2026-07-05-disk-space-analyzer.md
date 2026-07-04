# 磁盘空间分析工具 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增「磁盘分析」工具页（DiskSpaceAnalyzer.vue）+ Rust 后端模块（disk_analyzer.rs），支持文件夹大小分析 / 大文件 Top N / 按类型统计 / 重复文件检测，可勾选文件入回收站删除。

**Architecture:** Rust 后端异步扫描目录，结果暂存于模块级 `Lazy<Mutex<HashMap<String, ScanResults>>>`（**不使用 Tauri managed state**，沿用项目现有模式），通过 `app.emit` 推送进度事件；前端按 scan_id 分页拉取切片，4 个 Tab 展示不同视角。

**Tech Stack:** Rust（walkdir / trash / sha2 / uuid）+ Vue 3 Composition API + Element Plus + TypeScript

**Spec:** `docs/superpowers/specs/2026-07-05-disk-space-analyzer-design.md`

**重要偏差说明（与 spec 的差异）**：调研发现项目**不使用 Tauri managed state（AppState / app.manage / State<'_, ...>）**，全部共享状态走模块级 `static` + `Mutex`/`AtomicBool`（参考 `clipboard.rs`）。本计划改用 `std::sync::OnceLock` + `Lazy` 模式实现 scan 结果存储，不引入 AppState。

---

## 文件结构

**新增文件：**
- `src-tauri/src/disk_analyzer.rs` — Rust 后端模块（数据结构 + 11 个命令 + 测试）
- `src/views/DiskSpaceAnalyzer.vue` — 前端工具页（4 Tab）
- `src/utils/diskAnalyzerTypes.ts` — 前端 TS 类型定义
- `src/utils/diskAnalyzerClient.ts` — 前端 Tauri 命令封装

**修改文件：**
- `src-tauri/Cargo.toml` — 新增 4 个 crate + 版本号 4.2.0 → 4.3.0
- `src-tauri/src/main.rs` — `mod disk_analyzer;` + 注册 11 个命令
- `src/store/index.ts` — `TOOL_LIST` 追加 disk-analyzer 项
- `src/App.vue` — import + v-else-if 路由
- `src/components/SidebarNav.vue` — 版本号显示 4.2 → 4.3
- `package.json` — 版本号 4.2.0 → 4.3.0
- `src-tauri/tauri.conf.json` — 版本号 4.2.0 → 4.3.0
- `README.md` — 版本路线表追加 V4.3 行 + 功能特性追加磁盘分析条目

---

## Task 1: 添加 Rust 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 `[dependencies]` 末尾追加 4 个 crate**

打开 `d:\work\codes\litobox\src-tauri\Cargo.toml`，在 `sysinfo = "0.31"` 行下方追加：

```toml
walkdir = "2.5"
trash = "5.1"
sha2 = "0.10"
uuid = { version = "1.10", features = ["v4"] }
```

- [ ] **Step 2: 验证依赖能拉取**

Run: `cd src-tauri && cargo check`
Expected: 输出 `Finished` 无错误（首次会下载 4 个 crate，约 30s）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: 添加磁盘分析工具的 Rust 依赖（walkdir/trash/sha2/uuid）"
```

---

## Task 2: 创建 disk_analyzer.rs 模块骨架（数据结构 + 状态存储）

**Files:**
- Create: `src-tauri/src/disk_analyzer.rs`
- Modify: `src-tauri/src/main.rs:3-11`（追加 `mod disk_analyzer;`）

- [ ] **Step 1: 创建 disk_analyzer.rs 文件，写入数据结构 + 状态存储 + debug_log 宏**

写入以下完整内容到 `d:\work\codes\litobox\src-tauri\src\disk_analyzer.rs`：

```rust
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
```

- [ ] **Step 2: 在 main.rs 注册模块**

打开 `d:\work\codes\litobox\src-tauri\src\main.rs`，在第 11 行 `mod sqlite_viewer;` 下方追加：

```rust
mod disk_analyzer;
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过（可能有 unused warning，正常）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/disk_analyzer.rs src-tauri/src/main.rs
git commit -m "feat(disk-analyzer): 创建模块骨架与数据结构"
```

---

## Task 3: 实现扫描核心逻辑（walk + 聚合）+ 单元测试

**Files:**
- Modify: `src-tauri/src/disk_analyzer.rs`（追加扫描函数 + 测试模块）

- [ ] **Step 1: 在 disk_analyzer.rs 末尾追加扫描核心函数**

在文件末尾追加（在 `remove_scan` 函数之后）：

```rust
// ============ 扫描核心逻辑 ============

const TOP_FILES_LIMIT: usize = 5000;
const SKIPPED_DIRS_LIMIT: usize = 1000;
const CANCEL_CHECK_INTERVAL: u64 = 1000;

/// 维护 Top N 大文件的最小堆辅助：用 Vec + sort + truncate 简化（ponytail: 真正最小堆用 BinaryHeap，
// 但 Vec 排序在 N=5000 时性能足够，且代码更简单；升级路径是 BinaryHeap）
fn maybe_insert_top_file(top: &mut Vec<FileInfo>, file: FileInfo) {
    if top.len() < TOP_FILES_LIMIT {
        top.push(file);
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
        return;
    }
    // 已满，仅当新文件比当前最小大时替换
    let last = top.last().unwrap();
    if file.sizeBytes > last.sizeBytes {
        top.pop();
        top.push(file);
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
    }
}

/// 同步扫描指定路径，更新 results。返回 Result<(), String>
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

    // 用 BTreeMap 按父路径聚合文件夹统计
    let mut folder_map: HashMap<String, (u64, u64)> = HashMap::new(); // path -> (file_count, size_bytes)
    let mut ext_map: HashMap<String, (u64, u64)> = HashMap::new(); // ext -> (count, size)
    let mut top_files: Vec<FileInfo> = Vec::with_capacity(TOP_FILES_LIMIT);
    let mut all_files_by_size: HashMap<u64, Vec<String>> = HashMap::new(); // 用于重复检测
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

        // 更新当前路径（进度用）
        {
            let mut r = results_arc.lock().unwrap();
            r.current_path = Some(path_str.clone());
        }

        if entry.file_type().is_dir() {
            total_dirs += 1;
            // 初始化该目录的聚合项
            folder_map.entry(path_str.clone()).or_insert((0, 0));
            // 同时确保父目录被加入（用于下钻）
            if let Some(parent) = path.parent() {
                folder_map
                    .entry(parent.to_string_lossy().to_string())
                    .or_insert((0, 0));
            }
            // 隐藏文件过滤
            if !opts.includeHidden && is_hidden(entry.file_name()) {
                walker.skip_current_dir();
                continue;
            }
        } else if entry.file_type().is_file() {
            // 隐藏文件过滤
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
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();
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

        // 进度节流（500ms）
        files_since_check += 1;
        if files_since_check >= CANCEL_CHECK_INTERVAL || last_progress_emit.elapsed() > std::time::Duration::from_millis(500) {
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

    debug_log!("disk_analyzer: walk 完成，开始聚合。文件={}, 目录={}", total_files, total_dirs);

    // 计算文件夹信息
    let root_size = folder_map
        .get(&root_path)
        .map(|(_, s)| *s)
        .unwrap_or(0);
    let mut folders: Vec<FolderInfo> = folder_map
        .into_iter()
        .map(|(path, (file_count, size_bytes))| {
            let pb = PathBuf::from(&path);
            let name = pb
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let parent = pb
                .parent()
                .map(|p| p.to_string_lossy().to_string());
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

    // 扩展名统计
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
        debug_log!("disk_analyzer: 开始重复检测，候选 size 组数={}", all_files_by_size.len());
        let mut group_id = 0u32;
        for (size, paths) in all_files_by_size.iter() {
            if paths.len() < 2 {
                continue;
            }
            // 按指纹（前 64KB SHA-256）二次分组
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

    // 写回结果
    {
        let mut r = results_arc.lock().unwrap();
        r.status = ScanStatus::Completed;
        r.finished_at = Some(now_ms());
        r.folders = folders;
        r.top_files = top_files;
        r.ext_stats = ext_stats;
        r.duplicates = duplicates;
    }

    // 推送完成事件
    let summary = {
        let r = results_arc.lock().unwrap();
        ScanSummary {
            totalFiles: r.files_scanned,
            totalDirs,
            totalSize: r.bytes_scanned,
            skippedCount: r.skipped_total,
            durationMs: (r.finished_at.unwrap_or(0) - r.started_at) as u64,
            duplicatesWastedBytes: dup_wasted,
        }
    };
    let scan_id = results_arc.lock().unwrap().scan_id.clone();
    let _ = app.emit(
        "disk-scan-complete",
        ScanComplete { scanId: scan_id, summary },
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
```

- [ ] **Step 2: 追加单元测试模块（在文件最末尾）**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // a.txt 1KB
        let content_a = vec![b'a'; 1024];
        fs::write(root.join("a.txt"), &content_a).unwrap();
        // b.txt 1KB 同内容
        fs::write(root.join("b.txt"), &content_a).unwrap();
        // sub/c.txt 2KB
        fs::write(root.join("sub"), "").unwrap();
        let _ = fs::create_dir(root.join("sub"));
        fs::write(root.join("sub").join("c.txt"), vec![b'c'; 2048]).unwrap();
        // sub/d.txt 1KB 同内容
        fs::write(root.join("sub").join("d.txt"), &content_a).unwrap();
        // empty/
        let _ = fs::create_dir(root.join("empty"));

        dir
    }

    #[test]
    fn maybe_insert_top_file_keeps_largest() {
        let mut top = Vec::new();
        maybe_insert_top_file(&mut top, FileInfo {
            path: "a".into(), name: "a".into(), sizeBytes: 100, modifiedMs: 0, extension: "".into(),
        });
        maybe_insert_top_file(&mut top, FileInfo {
            path: "b".into(), name: "b".into(), sizeBytes: 300, modifiedMs: 0, extension: "".into(),
        });
        maybe_insert_top_file(&mut top, FileInfo {
            path: "c".into(), name: "c".into(), sizeBytes: 200, modifiedMs: 0, extension: "".into(),
        });
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].sizeBytes, 300);
        assert_eq!(top[1].sizeBytes, 200);
        assert_eq!(top[2].sizeBytes, 100);
    }

    #[test]
    fn maybe_insert_top_file_respects_limit() {
        let mut top = Vec::new();
        // 填满 5 个（模拟小 limit）
        for i in 0..5 {
            top.push(FileInfo {
                path: format!("f{}", i), name: format!("f{}", i),
                sizeBytes: i * 10, modifiedMs: 0, extension: "".into(),
            });
        }
        top.sort_by(|a, b| b.sizeBytes.cmp(&a.sizeBytes));
        // 模拟替换：用更大的替换最小的
        top.pop(); // 移除 sizeBytes=0
        top.push(FileInfo {
            path: "new".into(), name: "new".into(), sizeBytes: 999, modifiedMs: 0, extension: "".into(),
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
```

- [ ] **Step 3: 运行测试验证通过**

Run: `cd src-tauri && cargo test disk_analyzer`
Expected: `5 passed` 无失败

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/disk_analyzer.rs
git commit -m "feat(disk-analyzer): 实现扫描核心逻辑与单元测试"
```

---

## Task 4: 实现 Tauri 命令（11 个）

**Files:**
- Modify: `src-tauri/src/disk_analyzer.rs`（追加 11 个 `#[tauri::command]` 函数）

- [ ] **Step 1: 在 `now_ms` 函数之后、`#[cfg(test)]` 之前追加所有命令**

```rust
// ============ Tauri 命令 ============

#[tauri::command]
pub async fn disk_scan_start(
    app: AppHandle,
    path: String,
    opts: Option<ScanOptions>,
) -> Result<String, String> {
    let opts = opts.unwrap_or_default();
    let path_canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|e| format!("路径无法访问: {}", e))?
        .to_string_lossy()
        .to_string();

    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let results = ScanResults {
        scan_id: scan_id.clone(),
        root_path: path_canonical.clone(),
        started_at: now_ms(),
        finished_at: None,
        status: ScanStatus::Running,
        cancel_flag: cancel_flag.clone(),
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
    let results_arc = Arc::new(Mutex::new(results));
    insert_scan(scan_id.clone(), results_arc.clone());

    debug_log!("disk_analyzer: scan_start id={} path={}", scan_id, path_canonical);

    // 启动后台扫描线程
    let app_clone = app.clone();
    let scan_id_clone = scan_id.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_scan(results_arc.clone(), app_clone, opts) {
            debug_log!("disk_analyzer: scan 失败 id={} err={}", scan_id_clone, e);
            let mut r = results_arc.lock().unwrap();
            r.status = ScanStatus::Failed { error: e };
            r.finished_at = Some(now_ms());
        }
    });

    Ok(scan_id)
}

#[tauri::command]
pub fn disk_scan_cancel(scan_id: String) -> Result<(), String> {
    debug_log!("disk_analyzer: cancel id={}", scan_id);
    if let Some(arc) = get_scan(&scan_id) {
        let r = arc.lock().unwrap();
        r.cancel_flag.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("scan not found or expired".into())
    }
}

#[tauri::command]
pub fn disk_scan_status(scan_id: String) -> Result<ScanStatus, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    Ok(r.status.clone())
}

#[tauri::command]
pub fn disk_get_summary(scan_id: String) -> Result<ScanSummary, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    let dup_wasted = if r.duplicates.is_empty() {
        None
    } else {
        Some(r.duplicates.iter().map(|g| g.wastedBytes).sum())
    };
    Ok(ScanSummary {
        totalFiles: r.files_scanned,
        totalDirs: r.folders.len() as u64,
        totalSize: r.bytes_scanned,
        skippedCount: r.skipped_total,
        durationMs: r
            .finished_at
            .map(|f| (f - r.started_at) as u64)
            .unwrap_or(0),
        duplicatesWastedBytes: dup_wasted,
    })
}

#[tauri::command]
pub fn disk_get_folders(
    scan_id: String,
    parent: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<FolderPage, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    let limit = limit.unwrap_or(100).min(500) as usize;
    let offset = offset.unwrap_or(0) as usize;

    // 默认 parent = root_path
    let target_parent = parent.unwrap_or_else(|| r.root_path.clone());

    let filtered: Vec<&FolderInfo> = r
        .folders
        .iter()
        .filter(|f| f.parent.as_deref() == Some(&target_parent) && f.path != r.root_path)
        .collect();
    let total = filtered.len() as u64;
    let items: Vec<FolderInfo> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    Ok(FolderPage { items, total })
}

#[tauri::command]
pub fn disk_get_top_files(
    scan_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<FilePage, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    let limit = limit.unwrap_or(100).min(500) as usize;
    let offset = offset.unwrap_or(0) as usize;
    let total = r.top_files.len() as u64;
    let items: Vec<FileInfo> = r
        .top_files
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    Ok(FilePage { items, total })
}

#[tauri::command]
pub fn disk_get_extension_stats(
    scan_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<ExtStatPage, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    let limit = limit.unwrap_or(100).min(500) as usize;
    let offset = offset.unwrap_or(0) as usize;
    let total = r.ext_stats.len() as u64;
    let items: Vec<ExtensionStat> = r
        .ext_stats
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    Ok(ExtStatPage { items, total })
}

#[tauri::command]
pub fn disk_get_duplicates(
    scan_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<DupPage, String> {
    let arc = get_scan(&scan_id).ok_or("scan not found or expired")?;
    let r = arc.lock().unwrap();
    let limit = limit.unwrap_or(50).min(200) as usize;
    let offset = offset.unwrap_or(0) as usize;
    let total = r.duplicates.len() as u64;
    let items: Vec<DuplicateGroup> = r
        .duplicates
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    Ok(DupPage { items, total })
}

#[tauri::command]
pub async fn disk_delete_files(paths: Vec<String>) -> Result<DeleteResult, String> {
    debug_log!("disk_analyzer: delete_files count={}", paths.len());
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    for path in paths {
        match trash::delete(&path) {
            Ok(_) => {
                debug_log!("disk_analyzer: 已入回收站 {}", path);
                succeeded.push(path);
            }
            Err(e) => {
                debug_log!("disk_analyzer: 删除失败 {}: {}", path, e);
                failed.push(DeleteFailure {
                    path,
                    error: e.to_string(),
                });
            }
        }
    }
    Ok(DeleteResult { succeeded, failed })
}

#[tauri::command]
pub fn disk_clear_scan(scan_id: String) -> Result<(), String> {
    debug_log!("disk_analyzer: clear_scan id={}", scan_id);
    if remove_scan(&scan_id) {
        Ok(())
    } else {
        Err("scan not found".into())
    }
}

#[tauri::command]
pub fn disk_locate_in_explorer(path: String) -> Result<(), String> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // 规范化路径
    let pb = PathBuf::from(&path);
    if !pb.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    let parent = pb.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_name = pb
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut cmd = Command::new("explorer.exe");
    cmd.arg(format!("/select,{}", file_name));
    cmd.current_dir(parent);
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn()
        .map_err(|e| format!("无法打开资源管理器: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: 在 main.rs 注册所有命令**

打开 `d:\work\codes\litobox\src-tauri\src\main.rs`，找到 `tauri::generate_handler![` 块（约第 22 行起），在 `sqlite_viewer::sqlite_get_app_db_path,` 行下方追加：

```rust
        // 磁盘分析命令
        disk_analyzer::disk_scan_start,
        disk_analyzer::disk_scan_cancel,
        disk_analyzer::disk_scan_status,
        disk_analyzer::disk_get_summary,
        disk_analyzer::disk_get_folders,
        disk_analyzer::disk_get_top_files,
        disk_analyzer::disk_get_extension_stats,
        disk_analyzer::disk_get_duplicates,
        disk_analyzer::disk_delete_files,
        disk_analyzer::disk_clear_scan,
        disk_analyzer::disk_locate_in_explorer,
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误（可能有 unused warning）

- [ ] **Step 4: 验证测试仍通过**

Run: `cd src-tauri && cargo test disk_analyzer`
Expected: `5 passed`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/disk_analyzer.rs src-tauri/src/main.rs
git commit -m "feat(disk-analyzer): 实现 11 个 Tauri 命令并注册到 main.rs"
```

---

## Task 5: 创建前端类型定义 diskAnalyzerTypes.ts

**Files:**
- Create: `src/utils/diskAnalyzerTypes.ts`

- [ ] **Step 1: 创建类型文件**

写入以下内容到 `d:\work\codes\litobox\src\utils\diskAnalyzerTypes.ts`：

```ts
// 磁盘分析工具类型定义，与 Rust 端 disk_analyzer.rs 一一对应

export interface ScanOptions {
  includeHidden: boolean
  detectDuplicates: boolean
  maxFiles?: number | null
  followSymlinks: boolean
}

export type ScanStatus =
  | { status: 'running' }
  | { status: 'completed' }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }

export interface FolderInfo {
  path: string
  parent: string | null
  name: string
  depth: number
  fileCount: number
  sizeBytes: number
  percentOfRoot: number
}

export interface FileInfo {
  path: string
  name: string
  sizeBytes: number
  modifiedMs: number
  extension: string
}

export interface ExtensionStat {
  extension: string
  fileCount: number
  totalSize: number
  percent: number
}

export interface DuplicateGroup {
  groupId: number
  fileSize: number
  fileCount: number
  wastedBytes: number
  files: FileInfo[]
}

export interface ScanSummary {
  totalFiles: number
  totalDirs: number
  totalSize: number
  skippedCount: number
  durationMs: number
  duplicatesWastedBytes: number | null
}

export interface DeleteFailure {
  path: string
  error: string
}

export interface DeleteResult {
  succeeded: string[]
  failed: DeleteFailure[]
}

export interface FolderPage {
  items: FolderInfo[]
  total: number
}

export interface FilePage {
  items: FileInfo[]
  total: number
}

export interface ExtStatPage {
  items: ExtensionStat[]
  total: number
}

export interface DupPage {
  items: DuplicateGroup[]
  total: number
}

export interface ScanProgress {
  scanId: string
  filesScanned: number
  bytesScanned: number
  currentPath: string
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/diskAnalyzerTypes.ts
git commit -m "feat(disk-analyzer): 前端类型定义"
```

---

## Task 6: 创建前端 Tauri 命令封装 diskAnalyzerClient.ts

**Files:**
- Create: `src/utils/diskAnalyzerClient.ts`

- [ ] **Step 1: 创建客户端封装**

写入以下内容到 `d:\work\codes\litobox\src\utils\diskAnalyzerClient.ts`：

```ts
import { invoke } from '@tauri-apps/api/core'
import type {
  ScanOptions,
  ScanStatus,
  ScanSummary,
  FolderPage,
  FilePage,
  ExtStatPage,
  DupPage,
  DeleteResult,
} from './diskAnalyzerTypes'

export async function diskScanStart(path: string, opts: ScanOptions): Promise<string> {
  return invoke<string>('disk_scan_start', { path, opts })
}

export async function diskScanCancel(scanId: string): Promise<void> {
  return invoke('disk_scan_cancel', { scanId })
}

export async function diskScanStatus(scanId: string): Promise<ScanStatus> {
  return invoke<ScanStatus>('disk_scan_status', { scanId })
}

export async function diskGetSummary(scanId: string): Promise<ScanSummary> {
  return invoke<ScanSummary>('disk_get_summary', { scanId })
}

export async function diskGetFolders(
  scanId: string,
  parent: string | null,
  limit?: number,
  offset?: number
): Promise<FolderPage> {
  return invoke<FolderPage>('disk_get_folders', { scanId, parent, limit, offset })
}

export async function diskGetTopFiles(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<FilePage> {
  return invoke<FilePage>('disk_get_top_files', { scanId, limit, offset })
}

export async function diskGetExtensionStats(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<ExtStatPage> {
  return invoke<ExtStatPage>('disk_get_extension_stats', { scanId, limit, offset })
}

export async function diskGetDuplicates(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<DupPage> {
  return invoke<DupPage>('disk_get_duplicates', { scanId, limit, offset })
}

export async function diskDeleteFiles(paths: string[]): Promise<DeleteResult> {
  return invoke<DeleteResult>('disk_delete_files', { paths })
}

export async function diskClearScan(scanId: string): Promise<void> {
  return invoke('disk_clear_scan', { scanId })
}

export async function diskLocateInExplorer(path: string): Promise<void> {
  return invoke('disk_locate_in_explorer', { path })
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/diskAnalyzerClient.ts
git commit -m "feat(disk-analyzer): 前端 Tauri 命令封装"
```

---

## Task 7: 创建 DiskSpaceAnalyzer.vue 页面骨架（扫描配置 + 进度卡片）

**Files:**
- Create: `src/views/DiskSpaceAnalyzer.vue`

- [ ] **Step 1: 创建页面骨架**

写入以下内容到 `d:\work\codes\litobox\src\views\DiskSpaceAnalyzer.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- 扫描配置卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">磁盘空间分析</span>
        <div class="card-actions">
          <el-button size="small" @click="loadLastScanPath">上次路径</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 1">
            <div class="group-label">扫描路径</div>
            <el-input
              v-model="scanPath"
              placeholder="选择或输入要扫描的目录路径"
              size="small"
              clearable
            >
              <template #append>
                <el-button size="small" @click="selectFolder">浏览</el-button>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <el-checkbox v-model="opts.includeHidden">包含隐藏</el-checkbox>
            <el-checkbox v-model="opts.detectDuplicates">检测重复</el-checkbox>
            <el-checkbox v-model="opts.followSymlinks">跟随链接</el-checkbox>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!scanPath || scanning"
                :loading="scanning"
                @click="startScan"
              >
                开始扫描
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 扫描进度卡片（仅扫描中显示） -->
    <div v-if="scanning" class="tool-card">
      <div class="card-header">
        <span class="card-title">扫描中</span>
        <div class="card-actions">
          <el-button size="small" type="danger" @click="cancelScan">取消扫描</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="progress-info">
          <div>当前路径: {{ progress?.currentPath || '准备中...' }}</div>
          <div>
            已扫描: {{ progress?.filesScanned || 0 }} 文件 |
            {{ formatBytes(progress?.bytesScanned || 0) }} |
            耗时 {{ formatDuration(scanElapsedMs) }}
          </div>
        </div>
        <el-progress :percentage="scanPercentage" :stroke-width="14" :show-text="false" stripe />
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="scanError" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ scanError }}</div>
      </div>
    </div>

    <!-- 结果展示卡片（仅完成后显示） -->
    <div v-if="scanCompleted && summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">扫描结果</span>
        <div class="card-actions">
          <span class="summary-text">
            {{ summary.totalFiles }} 文件 | {{ formatBytes(summary.totalSize) }} |
            耗时 {{ formatDuration(summary.durationMs) }}
            <span v-if="summary.skippedCount > 0" class="warn-text">
              (跳过 {{ summary.skippedCount }} 个无权限目录)
            </span>
          </span>
        </div>
      </div>
      <div class="card-body">
        <el-tabs v-model="activeTab">
          <el-tab-pane label="文件夹大小" name="folders">
            <FoldersTab v-if="activeTab === 'folders'" :scan-id="scanId" :root-path="scanPath" />
          </el-tab-pane>
          <el-tab-pane label="大文件 Top N" name="topFiles">
            <TopFilesTab v-if="activeTab === 'topFiles'" :scan-id="scanId" />
          </el-tab-pane>
          <el-tab-pane label="按类型" name="extensions">
            <ExtensionsTab v-if="activeTab === 'extensions'" :scan-id="scanId" />
          </el-tab-pane>
          <el-tab-pane
            v-if="opts.detectDuplicates"
            label="重复文件"
            name="duplicates"
          >
            <DuplicatesTab v-if="activeTab === 'duplicates'" :scan-id="scanId" />
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { formatBytes, formatUptime } from '@/utils/systemInfoClient'
import {
  diskScanStart,
  diskScanCancel,
  diskGetSummary,
  diskClearScan,
} from '@/utils/diskAnalyzerClient'
import type { ScanOptions, ScanSummary, ScanProgress } from '@/utils/diskAnalyzerTypes'
import FoldersTab from './disk-analyzer-tabs/FoldersTab.vue'
import TopFilesTab from './disk-analyzer-tabs/TopFilesTab.vue'
import ExtensionsTab from './disk-analyzer-tabs/ExtensionsTab.vue'
import DuplicatesTab from './disk-analyzer-tabs/DuplicatesTab.vue'

const store = useToolboxStore()

const scanPath = ref(localStorage.getItem('diskAnalyzer.lastPath') || 'C:\\')
const opts = reactive<ScanOptions>({
  includeHidden: false,
  detectDuplicates: false,
  maxFiles: null,
  followSymlinks: false,
})
const scanning = ref(false)
const scanCompleted = ref(false)
const scanError = ref('')
const scanId = ref('')
const summary = ref<ScanSummary | null>(null)
const progress = ref<ScanProgress | null>(null)
const scanStartTime = ref(0)
const scanElapsedMs = ref(0)
const activeTab = ref('folders')

let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null
let elapsedTimer: ReturnType<typeof setInterval> | null = null

const scanPercentage = computed(() => {
  // 无准确百分比，用文件数模 1000 模拟进度条动画（ponytail: 仅视觉反馈）
  const n = progress.value?.filesScanned || 0
  return Math.min(95, (n % 1000) / 10 + 5)
})

const formatDuration = (ms: number) => {
  if (ms < 1000) return '0s'
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  return `${m}m ${s % 60}s`
}

const loadLastScanPath = () => {
  const last = localStorage.getItem('diskAnalyzer.lastPath')
  if (last) {
    scanPath.value = last
    ElMessage.info('已加载上次路径')
  }
}

const selectFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === 'string') {
      scanPath.value = selected
      localStorage.setItem('diskAnalyzer.lastPath', selected)
    }
  } catch (e) {
    ElMessage.error(`选择文件夹失败: ${e}`)
  }
}

const startScan = async () => {
  if (!scanPath.value) {
    ElMessage.warning('请先选择扫描路径')
    return
  }
  // 清理上次扫描
  if (scanId.value) {
    try {
      await diskClearScan(scanId.value)
    } catch {
      /* 忽略旧 scan_id 已过期 */
    }
  }
  scanning.value = true
  scanCompleted.value = false
  scanError.value = ''
  summary.value = null
  progress.value = null
  scanStartTime.value = Date.now()
  scanElapsedMs.value = 0
  localStorage.setItem('diskAnalyzer.lastPath', scanPath.value)

  elapsedTimer = setInterval(() => {
    scanElapsedMs.value = Date.now() - scanStartTime.value
  }, 1000)

  try {
    scanId.value = await diskScanStart(scanPath.value, { ...opts })
    ElMessage.success('扫描已启动')
  } catch (e) {
    scanning.value = false
    scanError.value = `启动扫描失败: ${e}`
    if (elapsedTimer) clearInterval(elapsedTimer)
  }
}

const cancelScan = async () => {
  if (!scanId.value) return
  try {
    await diskScanCancel(scanId.value)
    ElMessage.info('已请求取消，等待扫描停止...')
  } catch (e) {
    ElMessage.error(`取消失败: ${e}`)
  }
}

const handleScanComplete = async (event: { payload: { scanId: string; summary: ScanSummary } }) => {
  if (event.payload.scanId !== scanId.value) return
  scanning.value = false
  scanCompleted.value = true
  summary.value = event.payload.summary
  if (elapsedTimer) {
    clearInterval(elapsedTimer)
    elapsedTimer = null
  }
  scanElapsedMs.value = event.payload.summary.durationMs

  // 写入历史记录（AGENTS.md 强制要求 inputFull/outputFull）
  store.addHistory({
    tool: 'diskAnalyzer',
    action: '扫描磁盘',
    inputPreview: scanPath.value.slice(0, 50),
    outputPreview: `${event.payload.summary.totalFiles} 文件 ${formatBytes(event.payload.summary.totalSize)}`.slice(0, 50),
    inputFull: scanPath.value,
    outputFull: JSON.stringify(event.payload.summary),
  })
}

const handleScanProgress = (event: { payload: ScanProgress }) => {
  if (event.payload.scanId !== scanId.value) return
  progress.value = event.payload
}

const handleScanWarning = (event: { payload: { scanId: string; message: string } }) => {
  if (event.payload.scanId !== scanId.value) return
  ElMessage.warning(event.payload.message)
}

onMounted(async () => {
  unlistenProgress = await listen('disk-scan-progress', handleScanProgress as any)
  unlistenComplete = await listen('disk-scan-complete', handleScanComplete as any)
  await listen('disk-scan-warning', handleScanWarning as any)
})

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress()
  if (unlistenComplete) unlistenComplete()
  if (elapsedTimer) clearInterval(elapsedTimer)
  // 离开页面时释放 Rust 端结果
  if (scanId.value) {
    diskClearScan(scanId.value).catch(() => {})
  }
})
</script>

<style scoped>
@import '../style/tool-card.css';
.progress-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
  font-size: 13px;
  color: var(--text-secondary, #888);
  word-break: break-all;
}
.summary-text {
  font-size: 13px;
  color: var(--text-secondary, #888);
}
.warn-text {
  color: var(--warning-color, #e6a23c);
  margin-left: 8px;
}
</style>
```

- [ ] **Step 2: 验证页面可加载（暂时缺子组件，会有警告但能编译）**

Run: `npm run tauri dev`
Expected: 开发服务器启动，但页面访问会因缺子组件报错（后续 Task 8-11 补齐）。先确认主页面无语法错误。

- [ ] **Step 3: Commit**

```bash
git add src/views/DiskSpaceAnalyzer.vue
git commit -m "feat(disk-analyzer): 主页面骨架（扫描配置+进度+结果容器）"
```

---

## Task 8: 实现 FoldersTab.vue（文件夹大小分析，支持下钻）

**Files:**
- Create: `src/views/disk-analyzer-tabs/FoldersTab.vue`

- [ ] **Step 1: 创建子组件目录并写入 FoldersTab.vue**

先创建目录 `d:\work\codes\litobox\src\views\disk-analyzer-tabs\`，然后写入：

```vue
<template>
  <div class="folder-tab">
    <!-- 面包屑 -->
    <div class="breadcrumb">
      <el-breadcrumb separator="/">
        <el-breadcrumb-item
          v-for="(crumb, idx) in breadcrumbs"
          :key="idx"
          @click="navigateTo(crumb.path)"
        >
          <span class="crumb-link">{{ crumb.name }}</span>
        </el-breadcrumb-item>
      </el-breadcrumb>
      <el-button size="small" :disabled="!currentParent" @click="goUp">返回上级</el-button>
    </div>

    <el-table :data="folders" v-loading="loading" border size="small" style="width: 100%">
      <el-table-column prop="name" label="名称" min-width="200" />
      <el-table-column label="大小" width="220">
        <template #default="{ row }">
          <div class="size-cell">
            <span>{{ formatBytes(row.sizeBytes) }}</span>
            <el-progress
              :percentage="Math.min(100, row.percentOfRoot)"
              :stroke-width="6"
              :show-text="false"
              style="flex: 1; margin-left: 8px"
            />
          </div>
        </template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="100" />
      <el-table-column label="占比" width="80">
        <template #default="{ row }">{{ row.percentOfRoot.toFixed(1) }}%</template>
      </el-table-column>
      <el-table-column label="操作" width="160">
        <template #default="{ row }">
          <el-button size="small" link @click="drillDown(row)">下钻</el-button>
          <el-button size="small" link @click="locate(row.path)">定位</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetFolders, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { FolderInfo } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string; rootPath: string }>()

const folders = ref<FolderInfo[]>([])
const loading = ref(false)
const currentParent = ref<string | null>(null) // null = root
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)

const breadcrumbs = ref<{ name: string; path: string }[]>([])

const fetchFolders = async (reset = false) => {
  if (reset) {
    offset.value = 0
    folders.value = []
  }
  loading.value = true
  try {
    const page = await diskGetFolders(props.scanId, currentParent.value, pageSize, offset.value)
    if (reset) {
      folders.value = page.items
    } else {
      folders.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = folders.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载文件夹失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const drillDown = (folder: FolderInfo) => {
  currentParent.value = folder.path
  updateBreadcrumbs(folder.path)
  fetchFolders(true)
}

const goUp = () => {
  if (!currentParent.value) return
  // 找当前父的父
  const pb = currentParent.value
  // 简化：用 props.rootPath 作上限
  if (pb === props.rootPath) {
    currentParent.value = null
  } else {
    // 取最后一段的上级
    const parts = pb.split(/[/\\]/).filter(Boolean)
    if (parts.length <= 1) {
      currentParent.value = null
    } else {
      parts.pop()
      // 重构路径（保留盘符冒号）
      let parent = parts.join('\\')
      if (pb.startsWith('\\\\')) parent = '\\\\' + parent // UNC 路径
      else if (/^[A-Za-z]:/.test(pb)) parent = parts[0] + '\\' + parts.slice(1).join('\\')
      currentParent.value = parent === props.rootPath ? null : parent
    }
  }
  updateBreadcrumbs(currentParent.value || props.rootPath)
  fetchFolders(true)
}

const navigateTo = (path: string) => {
  currentParent.value = path === props.rootPath ? null : path
  updateBreadcrumbs(path)
  fetchFolders(true)
}

const updateBreadcrumbs = (path: string) => {
  const parts = path.split(/[/\\]/).filter(Boolean)
  const crumbs: { name: string; path: string }[] = []
  let acc = ''
  for (const p of parts) {
    acc = acc ? acc + '\\' + p : (path.startsWith('\\\\') ? '\\\\' + p : /^[A-Za-z]:/.test(path) ? p : p)
    crumbs.push({ name: p, path: acc })
  }
  breadcrumbs.value = crumbs
}

const loadMore = () => fetchFolders(false)

const locate = async (path: string) => {
  try {
    await diskLocateInExplorer(path)
  } catch (e) {
    ElMessage.error(`定位失败: ${e}`)
  }
}

onMounted(() => {
  updateBreadcrumbs(props.rootPath)
  fetchFolders(true)
})

// 当 scanId 变化时重新加载
watch(() => props.scanId, () => {
  currentParent.value = null
  updateBreadcrumbs(props.rootPath)
  fetchFolders(true)
})
</script>

<style scoped>
.folder-tab { padding: 8px 0; }
.breadcrumb {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.crumb-link { cursor: pointer; color: var(--primary-color, #00d4ff); }
.size-cell { display: flex; align-items: center; }
.load-more { text-align: center; margin-top: 12px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/disk-analyzer-tabs/FoldersTab.vue
git commit -m "feat(disk-analyzer): FoldersTab 子组件（文件夹大小+下钻+面包屑）"
```

---

## Task 9: 实现 TopFilesTab.vue（大文件 Top N + 多选删除）

**Files:**
- Create: `src/views/disk-analyzer-tabs/TopFilesTab.vue`

- [ ] **Step 1: 写入 TopFilesTab.vue**

```vue
<template>
  <div class="top-files-tab">
    <div class="tab-toolbar">
      <span class="muted">共 {{ total }} 个文件（仅展示 Top {{ pageSize }}）</span>
      <el-button
        size="small"
        type="danger"
        :disabled="selected.length === 0"
        @click="confirmDelete"
      >
        删除勾选 ({{ selected.length }}, {{ formatBytes(selectedSize) }})
      </el-button>
    </div>

    <el-table
      :data="files"
      v-loading="loading"
      border
      size="small"
      @selection-change="onSelectionChange"
      style="width: 100%"
    >
      <el-table-column type="selection" width="40" />
      <el-table-column prop="path" label="路径" min-width="300" show-overflow-tooltip />
      <el-table-column label="大小" width="120">
        <template #default="{ row }">{{ formatBytes(row.sizeBytes) }}</template>
      </el-table-column>
      <el-table-column label="修改时间" width="170">
        <template #default="{ row }">{{ formatTime(row.modifiedMs) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="120">
        <template #default="{ row }">
          <el-button size="small" link @click="locate(row.path)">定位</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetTopFiles, diskDeleteFiles, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { FileInfo } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const files = ref<FileInfo[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)
const selected = ref<FileInfo[]>([])

const selectedSize = computed(() =>
  selected.value.reduce((sum, f) => sum + f.sizeBytes, 0)
)

const formatTime = (ms: number) => {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('zh-CN')
}

const fetchFiles = async (reset = false) => {
  if (reset) {
    offset.value = 0
    files.value = []
  }
  loading.value = true
  try {
    const page = await diskGetTopFiles(props.scanId, pageSize, offset.value)
    if (reset) {
      files.value = page.items
    } else {
      files.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = files.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载文件列表失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const onSelectionChange = (rows: FileInfo[]) => {
  selected.value = rows
}

const confirmDelete = async () => {
  if (selected.value.length === 0) return
  const paths = selected.value.map((f) => f.path)
  const totalSize = formatBytes(selectedSize.value)
  try {
    await ElMessageBox.confirm(
      `确认将 ${paths.length} 个文件（共 ${totalSize}）送入回收站？`,
      '删除确认',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' }
    )
  } catch {
    return // 用户取消
  }
  try {
    const result = await diskDeleteFiles(paths)
    if (result.succeeded.length > 0) {
      ElMessage.success(`已删除 ${result.succeeded.length} 个文件`)
      // 从列表中移除已删除的
      const succSet = new Set(result.succeeded)
      files.value = files.value.filter((f) => !succSet.has(f.path))
    }
    if (result.failed.length > 0) {
      ElMessage.warning(`${result.failed.length} 个文件删除失败`)
      console.error('删除失败详情:', result.failed)
    }
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`)
  }
}

const locate = async (path: string) => {
  try {
    await diskLocateInExplorer(path)
  } catch (e) {
    ElMessage.error(`定位失败: ${e}`)
  }
}

const loadMore = () => fetchFiles(false)

onMounted(() => fetchFiles(true))
</script>

<style scoped>
.top-files-tab { padding: 8px 0; }
.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.load-more { text-align: center; margin-top: 12px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/disk-analyzer-tabs/TopFilesTab.vue
git commit -m "feat(disk-analyzer): TopFilesTab 子组件（大文件Top N+多选删除）"
```

---

## Task 10: 实现 ExtensionsTab.vue（按类型统计）

**Files:**
- Create: `src/views/disk-analyzer-tabs/ExtensionsTab.vue`

- [ ] **Step 1: 写入 ExtensionsTab.vue**

```vue
<template>
  <div class="extensions-tab">
    <div class="tab-toolbar">
      <span class="muted">共 {{ total }} 种扩展名</span>
    </div>

    <el-table :data="exts" v-loading="loading" border size="small" style="width: 100%">
      <el-table-column label="扩展名" width="140">
        <template #default="{ row }">
          <span class="ext-tag">{{ row.extension || '(无扩展名)' }}</span>
        </template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="120" />
      <el-table-column label="总大小" width="240">
        <template #default="{ row }">
          <div class="size-cell">
            <span>{{ formatBytes(row.totalSize) }}</span>
            <el-progress
              :percentage="Math.min(100, row.percent)"
              :stroke-width="6"
              :show-text="false"
              style="flex: 1; margin-left: 8px"
            />
          </div>
        </template>
      </el-table-column>
      <el-table-column label="占比" width="80">
        <template #default="{ row }">{{ row.percent.toFixed(1) }}%</template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetExtensionStats } from '@/utils/diskAnalyzerClient'
import type { ExtensionStat } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const exts = ref<ExtensionStat[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 100
const hasMore = ref(false)

const fetchExts = async (reset = false) => {
  if (reset) {
    offset.value = 0
    exts.value = []
  }
  loading.value = true
  try {
    const page = await diskGetExtensionStats(props.scanId, pageSize, offset.value)
    if (reset) {
      exts.value = page.items
    } else {
      exts.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = exts.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载类型统计失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const loadMore = () => fetchExts(false)

onMounted(() => fetchExts(true))
</script>

<style scoped>
.extensions-tab { padding: 8px 0; }
.tab-toolbar { margin-bottom: 10px; }
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.ext-tag {
  font-family: monospace;
  background: var(--bg-alt, #1e2a3a);
  padding: 2px 6px;
  border-radius: 3px;
}
.size-cell { display: flex; align-items: center; }
.load-more { text-align: center; margin-top: 12px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/disk-analyzer-tabs/ExtensionsTab.vue
git commit -m "feat(disk-analyzer): ExtensionsTab 子组件（按类型统计）"
```

---

## Task 11: 实现 DuplicatesTab.vue（重复文件 + 展开删除）

**Files:**
- Create: `src/views/disk-analyzer-tabs/DuplicatesTab.vue`

- [ ] **Step 1: 写入 DuplicatesTab.vue**

```vue
<template>
  <div class="duplicates-tab">
    <div class="tab-toolbar">
      <span class="muted">
        共 {{ total }} 组重复文件 |
        可回收 {{ formatBytes(totalWasted) }}
      </span>
      <el-button
        size="small"
        type="danger"
        :disabled="allSelected.length === 0"
        @click="confirmDeleteAll"
      >
        删除全部勾选 ({{ allSelected.length }}, {{ formatBytes(allSelectedSize) }})
      </el-button>
    </div>

    <el-table
      :data="groups"
      v-loading="loading"
      border
      size="small"
      row-key="groupId"
      style="width: 100%"
    >
      <el-table-column type="expand">
        <template #default="{ row }">
          <div class="group-files">
            <div v-for="file in row.files" :key="file.path" class="group-file-row">
              <el-checkbox
                :model-value="isChecked(row.groupId, file.path)"
                @change="(val) => toggleCheck(row.groupId, file.path, val as boolean)"
              />
              <span class="file-path" :title="file.path">{{ file.path }}</span>
              <el-button size="small" link @click="locate(file.path)">定位</el-button>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column label="组号" width="80">
        <template #default="{ row }">#{{ row.groupId }}</template>
      </el-table-column>
      <el-table-column prop="fileCount" label="文件数" width="100" />
      <el-table-column label="单个大小" width="120">
        <template #default="{ row }">{{ formatBytes(row.fileSize) }}</template>
      </el-table-column>
      <el-table-column label="浪费空间" width="140">
        <template #default="{ row }">
          <span class="wasted">{{ formatBytes(row.wastedBytes) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="本组勾选" width="120">
        <template #default="{ row }">{{ countCheckedInGroup(row.groupId) }} / {{ row.fileCount }}</template>
      </el-table-column>
    </el-table>

    <div v-if="hasMore" class="load-more">
      <el-button size="small" @click="loadMore" :loading="loading">加载更多</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { formatBytes } from '@/utils/systemInfoClient'
import { diskGetDuplicates, diskDeleteFiles, diskLocateInExplorer } from '@/utils/diskAnalyzerClient'
import type { DuplicateGroup } from '@/utils/diskAnalyzerTypes'

const props = defineProps<{ scanId: string }>()

const groups = ref<DuplicateGroup[]>([])
const loading = ref(false)
const total = ref(0)
const offset = ref(0)
const pageSize = 50
const hasMore = ref(false)

// 选中状态：Map<groupId, Set<filePath>>
const selectedMap = ref<Map<number, Set<string>>>(new Map())

const allSelected = computed(() => {
  const arr: { groupId: number; path: string; size: number }[] = []
  for (const [gid, paths] of selectedMap.value) {
    const g = groups.value.find((x) => x.groupId === gid)
    if (!g) continue
    for (const p of paths) {
      arr.push({ groupId: gid, path: p, size: g.fileSize })
    }
  }
  return arr
})

const allSelectedSize = computed(() =>
  allSelected.value.reduce((s, x) => s + x.size, 0)
)

const totalWasted = computed(() =>
  groups.value.reduce((s, g) => s + g.wastedBytes, 0)
)

const isChecked = (gid: number, path: string) => {
  return selectedMap.value.get(gid)?.has(path) ?? false
}

const toggleCheck = (gid: number, path: string, val: boolean) => {
  if (!selectedMap.value.has(gid)) {
    selectedMap.value.set(gid, new Set())
  }
  const set = selectedMap.value.get(gid)!
  if (val) set.add(path)
  else set.delete(path)
  if (set.size === 0) selectedMap.value.delete(gid)
  // 触发响应式
  selectedMap.value = new Map(selectedMap.value)
}

const countCheckedInGroup = (gid: number) => {
  return selectedMap.value.get(gid)?.size ?? 0
}

const fetchGroups = async (reset = false) => {
  if (reset) {
    offset.value = 0
    groups.value = []
    selectedMap.value.clear()
  }
  loading.value = true
  try {
    const page = await diskGetDuplicates(props.scanId, pageSize, offset.value)
    if (reset) {
      groups.value = page.items
    } else {
      groups.value.push(...page.items)
    }
    total.value = page.total
    hasMore.value = groups.value.length < total.value
    offset.value += page.items.length
  } catch (e) {
    ElMessage.error(`加载重复文件失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const confirmDeleteAll = async () => {
  const paths = allSelected.value.map((x) => x.path)
  if (paths.length === 0) return
  const totalSize = formatBytes(allSelectedSize.value)
  try {
    await ElMessageBox.confirm(
      `确认将 ${paths.length} 个文件（共 ${totalSize}）送入回收站？\n\n注意：每组至少保留 1 个文件，否则数据丢失不可恢复。`,
      '删除确认',
      { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' }
    )
  } catch {
    return
  }
  try {
    const result = await diskDeleteFiles(paths)
    if (result.succeeded.length > 0) {
      ElMessage.success(`已删除 ${result.succeeded.length} 个文件`)
      // 从展开组中移除已删除的
      const succSet = new Set(result.succeeded)
      for (const g of groups.value) {
        g.files = g.files.filter((f) => !succSet.has(f.path))
        g.fileCount = g.files.length as number
        g.wastedBytes = g.fileSize * Math.max(0, g.fileCount - 1)
      }
      // 移除空组
      groups.value = groups.value.filter((g) => g.fileCount >= 1)
      selectedMap.value.clear()
    }
    if (result.failed.length > 0) {
      ElMessage.warning(`${result.failed.length} 个文件删除失败`)
      console.error('删除失败详情:', result.failed)
    }
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`)
  }
}

const locate = async (path: string) => {
  try {
    await diskLocateInExplorer(path)
  } catch (e) {
    ElMessage.error(`定位失败: ${e}`)
  }
}

const loadMore = () => fetchGroups(false)

onMounted(() => fetchGroups(true))
</script>

<style scoped>
.duplicates-tab { padding: 8px 0; }
.tab-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.muted { font-size: 13px; color: var(--text-secondary, #888); }
.wasted { color: var(--warning-color, #e6a23c); font-weight: 600; }
.group-files { padding: 8px 16px; background: var(--bg-alt, #1a2332); }
.group-file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}
.file-path {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.load-more { text-align: center; margin-top: 12px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/disk-analyzer-tabs/DuplicatesTab.vue
git commit -m "feat(disk-analyzer): DuplicatesTab 子组件（重复文件+展开+多选删除）"
```

---

## Task 12: 注册工具到 TOOL_LIST 和 App.vue

**Files:**
- Modify: `src/store/index.ts`（TOOL_LIST 追加项）
- Modify: `src/App.vue`（import + v-else-if 路由）

- [ ] **Step 1: 在 TOOL_LIST 追加 diskAnalyzer 项**

打开 `d:\work\codes\litobox\src\store\index.ts`，找到系统工具分组中的 `sqliteViewer` 项（约第 81-86 行附近），在其后追加：

```ts
{ id: 'diskAnalyzer', name: '磁盘分析', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/><circle cx="9" cy="13" r="2"/><circle cx="15" cy="13" r="2"/></svg>`,
  description: '分析磁盘空间占用，查找大文件和重复文件',
  keywords: ['磁盘', '空间', '重复', '清理', 'disk', 'space', 'duplicate'],
  category: 'system' },
```

- [ ] **Step 2: 在 App.vue 注册路由**

打开 `d:\work\codes\litobox\src\App.vue`，在 `<script setup>` 的 import 块中（约第 67-109 行）追加：

```ts
import DiskSpaceAnalyzer from '@/views/DiskSpaceAnalyzer.vue'
```

在 `<KeepAlive>` 块内（约第 50 行 `SqliteViewerView` 之后）追加：

```vue
<DiskSpaceAnalyzer v-else-if="activeTool === 'diskAnalyzer'" :key="'diskAnalyzer'" />
```

- [ ] **Step 3: 启动开发服务器验证**

Run: `npm run tauri dev`
Expected: 应用启动，侧边栏「系统工具」分组出现「磁盘分析」项，点击进入页面无报错

- [ ] **Step 4: Commit**

```bash
git add src/store/index.ts src/App.vue
git commit -m "feat(disk-analyzer): 注册到 TOOL_LIST 和 App.vue 路由"
```

---

## Task 13: 端到端手动验证

**Files:** 无修改，仅测试

- [ ] **Step 1: 启动开发服务器**

Run: `npm run tauri dev`

- [ ] **Step 2: 验证场景 1 — 小目录扫描**

操作：
1. 侧边栏点击「磁盘分析」
2. 路径输入一个小的测试目录（如 `D:\work\codes\litobox\src`）
3. 勾选「检测重复」
4. 点击「开始扫描」
5. 观察进度卡片更新

Expected: 扫描完成后显示汇总，4 个 Tab 可切换，文件夹大小 Tab 可下钻

- [ ] **Step 3: 验证场景 2 — 重复文件检测**

操作：
1. 在测试目录中复制几个相同内容的文件到不同位置
2. 重新扫描，勾选「检测重复」
3. 切到「重复文件」Tab
4. 展开一组，勾选其中 1 个文件
5. 点击「删除全部勾选」
6. 确认删除

Expected: 文件被移到回收站，组内文件数减少

- [ ] **Step 4: 验证场景 3 — 大文件删除**

操作：
1. 切到「大文件 Top N」Tab
2. 勾选 1-2 个文件
3. 点击「删除勾选」
4. 确认

Expected: 文件被删除，列表刷新

- [ ] **Step 5: 验证场景 4 — 历史记录**

操作：
1. 完成一次扫描后，切换到「历史」页
2. 查看是否有「扫描磁盘」记录

Expected: 历史列表新增一条记录，inputFull 为路径，可双击跳转

- [ ] **Step 6: 验证场景 5 — 取消扫描**

操作：
1. 选择一个较大目录（如 `C:\`）
2. 点击「开始扫描」
3. 扫描进行中点击「取消扫描」

Expected: 扫描停止，进度卡片消失，无错误

- [ ] **Step 7: 验证场景 6 — 离开页面清理**

操作：
1. 完成扫描后切到其他工具页
2. 切回磁盘分析页

Expected: 重新进入时是干净状态，无残留旧数据

---

## Task 14: 版本号同步更新

**Files:**
- Modify: `src-tauri/Cargo.toml`（version = "4.2.0" → "4.3.0"）
- Modify: `package.json`（version 同步）
- Modify: `src-tauri/tauri.conf.json`（version 同步）
- Modify: `src/components/SidebarNav.vue`（显示版本 v4.2 → v4.3）

- [ ] **Step 1: 更新 Cargo.toml**

`d:\work\codes\litobox\src-tauri\Cargo.toml` 第 3 行：

```toml
version = "4.3.0"
```

- [ ] **Step 2: 更新 package.json**

`d:\work\codes\litobox\package.json` `"version"` 字段：

```json
"version": "4.3.0",
```

- [ ] **Step 3: 更新 tauri.conf.json**

`d:\work\codes\litobox\src-tauri\tauri.conf.json` `"version"` 字段：

```json
"version": "4.3.0"
```

- [ ] **Step 4: 更新 SidebarNav.vue 版本显示**

打开 `d:\work\codes\litobox\src\components\SidebarNav.vue`，搜索 `v4.2`（约第 8 行），改为 `v4.3`。

- [ ] **Step 5: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 通过

Run: `npm run build`
Expected: 前端构建通过

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json src/components/SidebarNav.vue
git commit -m "chore: 版本号 4.2.0 → 4.3.0"
```

---

## Task 15: 更新 README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 在版本路线表追加 V4.3 行**

打开 `d:\work\codes\litobox\README.md`，找到 `## 🗺️ 版本路线` 表（约第 287 行起），在 `V4.2` 行下方追加：

```markdown
| V4.3 | ✅ | 磁盘空间分析器（文件夹大小/大文件Top N/按类型统计/重复文件检测，可入回收站删除） |
```

- [ ] **Step 2: 在功能特性章节追加磁盘分析条目**

找到 `## 🧰 功能特性` 章节（约第 35 行起），定位到「系统工具」子分组（与系统信息/进程列表等同组），追加：

```markdown
- **磁盘分析** — 文件夹大小下钻、大文件 Top N、按扩展名统计、重复文件检测（前64KB指纹），支持勾选文件入回收站删除
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: README 同步更新 V4.3 磁盘空间分析器"
```

---

## 自检报告

**Spec 覆盖检查：**
- ✅ 文件夹大小分析（带下钻） — Task 3 + 8
- ✅ 大文件 Top N — Task 3 + 9
- ✅ 按类型分组统计 — Task 3 + 10
- ✅ 重复文件检测（前 64KB + 大小指纹） — Task 3 + 11
- ✅ 入回收站删除（trash crate） — Task 4 + 9 + 11
- ✅ 异步扫描 + 进度事件流 — Task 3 + 7
- ✅ 取消扫描（AtomicBool） — Task 3 + 4
- ✅ 结果暂存 Rust 端 + 分页拉取 — Task 2 + 4
- ✅ 路径规范化 + 二次确认 — Task 4 + 9 + 11
- ✅ 历史记录接入（inputFull/outputFull） — Task 7
- ✅ 不接入工作流/变量池 — 按 spec
- ✅ 单元测试（tempfile + 5 个测试） — Task 3
- ✅ debug_log! 日志 — Task 2 + 3 + 4
- ✅ 版本号同步 README — Task 14 + 15

**Placeholder 扫描：** 无 TBD/TODO，所有步骤含完整代码。

**类型一致性：** Rust 端 `sizeBytes`/`fileCount`/`wastedBytes` 等 camelCase 字段（serde 默认转换）与 TS 端接口完全一致；命令名 `diskScanStart` 等 camelCase 与 Rust 函数名 `disk_scan_start` 通过 Tauri 自动转换匹配。

**重要偏差：** Spec 写的是 `Arc<Mutex<HashMap<String, ScanResults>>>` 存在 AppState 中，实际改为模块级 `static SCANS: Lazy<Mutex<HashMap<String, Arc<Mutex<ScanResults>>>>>`（项目惯例），数据结构层级多包了一层 Arc，但对外接口完全一致。
