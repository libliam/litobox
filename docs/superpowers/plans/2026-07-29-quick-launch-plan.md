# 快速启动 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 基于 SQLite FTS5 实现全盘文件名快速搜索工具，支持即时搜索和一键打开文件

**Architecture:** Rust 后端负责全盘扫描构建 FTS5 索引、搜索和打开文件；Vue 前端提供工具页面和全局浮层两种交互入口。索引持久化到 SQLite，首次全盘扫描后增量更新。

**Tech Stack:** Rust (rusqlite + FTS5 + walkdir), Vue 3 (Composition API), Element Plus, Tauri v2 (tauri-plugin-shell 打开文件, tauri-plugin-global-shortcut 快捷键)

---

### Task 1: 后端 — db.rs 添加快速启动表结构和辅助函数

**Files:**
- Modify: `src-tauri/src/db.rs` (在 `init_tables` 中添加建表语句 + 新增辅助函数)

- [ ] **Step 1: 在 init_tables 中添加快速启动三张表的创建语句**

在 `d:\work\litobox\src-tauri\src\db.rs` 的 `init_tables` 函数末尾（其他 `CREATE TABLE` 之后）添加：

```sql
CREATE TABLE IF NOT EXISTS quick_launch_files(
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL,
  path        TEXT NOT NULL UNIQUE,
  extension   TEXT DEFAULT '',
  size_bytes  INTEGER DEFAULT 0,
  modified_at INTEGER DEFAULT 0,
  drive       TEXT NOT NULL
);
CREATE VIRTUAL TABLE IF NOT EXISTS quick_launch_fts USING fts5(
  name,
  content='quick_launch_files',
  content_rowid='id',
  tokenize='unicode61'
);
CREATE TABLE IF NOT EXISTS quick_launch_meta(
  drive        TEXT PRIMARY KEY,
  last_scanned INTEGER NOT NULL DEFAULT 0,
  file_count   INTEGER NOT NULL DEFAULT 0,
  status       TEXT NOT NULL DEFAULT 'pending'
);
```

- [ ] **Step 2: 添加 db 辅助函数**

在 `db.rs` 末尾（其他函数之后）添加以下函数：

```rust
// ========== 快速启动 ==========

pub fn do_ql_insert_files(conn: &mut Connection, files: &[(String, String, String, i64, i64, String)]) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO quick_launch_files (name, path, extension, size_bytes, modified_at, drive) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        ).map_err(|e| e.to_string())?;
        for (name, path, ext, size, modified, drive) in files {
            stmt.execute(params![name, path, ext, size, modified, drive]).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn do_ql_delete_file(conn: &mut Connection, path: &str) -> Result<(), String> {
    conn.execute("DELETE FROM quick_launch_files WHERE path = ?1", params![path])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn do_ql_search(conn: &mut Connection, query: &str) -> Result<Vec<(i64, String, String, String, i64, i64, String)>, String> {
    let fts_query = query.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| format!("{}*", c))
        .collect::<Vec<_>>()
        .join(" ");
    let mut stmt = conn.prepare(
        "SELECT f.id, f.name, f.path, f.extension, f.size_bytes, f.modified_at, f.drive
         FROM quick_launch_fts
         JOIN quick_launch_files f ON f.id = quick_launch_fts.rowid
         WHERE quick_launch_fts MATCH ?1
         ORDER BY f.modified_at DESC
         LIMIT 100"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![fts_query], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
        ))
    }).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| e.to_string())?);
    }
    Ok(results)
}

pub fn do_ql_get_meta(conn: &mut Connection) -> Result<Vec<(String, i64, i64, String)>, String> {
    let mut stmt = conn.prepare("SELECT drive, last_scanned, file_count, status FROM quick_launch_meta ORDER BY drive")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| e.to_string())?;
    let mut metas = Vec::new();
    for r in rows {
        metas.push(r.map_err(|e| e.to_string())?);
    }
    Ok(metas)
}

pub fn do_ql_update_meta(conn: &mut Connection, drive: &str, file_count: i64, status: &str) -> Result<(), String> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    conn.execute(
        "INSERT INTO quick_launch_meta (drive, last_scanned, file_count, status) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(drive) DO UPDATE SET last_scanned=?2, file_count=?3, status=?4",
        params![drive, now, file_count, status],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn do_ql_clear_all(conn: &mut Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM quick_launch_files;
         DELETE FROM quick_launch_meta;
         INSERT INTO quick_launch_fts(quick_launch_fts) VALUES('rebuild');"
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn do_ql_rebuild_fts(conn: &mut Connection) -> Result<(), String> {
    conn.execute("INSERT INTO quick_launch_fts(quick_launch_fts) VALUES('rebuild')", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

> 注意：`do_ql_search` 中的 FTS5 查询构造将用户输入的每个字符用空格分隔并追加 `*` 做前缀匹配。例如输入 `excel` → `e* x* c* e* l*`。这样对 ASCII 每个字母独立匹配可能产生过多结果。一个更好的做法是：如果输入全部是 ASCII 字母/数字，直接作为一个整体 `excel*`；否则逐字符拆分。在 Task 2 的 `build_fts_query` 函数中实现这个逻辑。

- [ ] **Step 3: 构建确认**

Run: `cd d:\work\litobox\src-tauri && cargo check 2>&1 | head -20`
Expected: 编译通过，无错误


### Task 2: 后端 — 创建 quick_launch.rs

**Files:**
- Create: `src-tauri/src/quick_launch.rs`
- Modify: `src-tauri/src/main.rs` (新增 `mod quick_launch;` + 注册命令)

- [ ] **Step 1: 创建 quick_launch.rs**

创建 `d:\work\litobox\src-tauri\src\quick_launch.rs`，包含完整模块代码：

```rust
use crate::db::{self, with_conn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

// ============ 类型定义 ============

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickLaunchResult {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub extension: String,
    pub size_bytes: i64,
    pub modified_at: i64,
    pub drive: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DriveIndexInfo {
    pub drive: String,
    pub last_scanned: i64,
    pub file_count: i64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexStatus {
    pub drives: Vec<DriveIndexInfo>,
    pub is_building: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QLIndexProgress {
    pub search_id: String,
    pub files_scanned: i64,
    pub total_files: i64,
    pub current_drive: String,
    pub current_path: String,
    pub status: String,
    pub error: Option<String>,
}

// ============ 全局状态 ============

static INDEXING_STATE: OnceLock<Mutex<Option<IndexingJob>>> = OnceLock::new();

struct IndexingJob {
    search_id: String,
    cancelled: bool,
}

fn indexing_state() -> &'static Mutex<Option<IndexingJob>> {
    INDEXING_STATE.get_or_init(|| Mutex::new(None))
}

// ============ 工具函数 ============

/// 构建 FTS5 查询字符串
/// - 纯 ASCII 字母数字输入：整体追加 `*` 做前缀匹配
/// - 包含中文/其他 Unicode 字符：逐字符拆分并 AND 匹配
fn build_fts_query(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // 判断是否全为 ASCII 字母数字
    let is_ascii_alphanum = trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_');
    if is_ascii_alphanum {
        // 按空白拆分，每个词追加 *
        trimmed
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| format!("{}*", s))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        // 包含中文：逐字符拆分
        trimmed
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| format!("{}*", c))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// 获取所有固定驱动器列表（排除 Windows 系统目录）
fn get_fixed_drives() -> Vec<String> {
    let mut drives = Vec::new();
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let logical_drives = windows_sys::Win32::Storage::FileSystem::GetLogicalDrives();
            for i in 0..26 {
                if logical_drives & (1 << i) != 0 {
                    let drive_letter = format!("{}:\\", (b'A' + i as u8) as char);
                    let drive_type = windows_sys::Win32::Storage::FileSystem::GetDriveTypeA(
                        std::ffi::CString::new(drive_letter.as_str()).unwrap().as_ptr(),
                    );
                    if drive_type == windows_sys::Win32::Storage::FileSystem::DRIVE_FIXED {
                        drives.push(drive_letter);
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        drives.push("/".to_string());
    }
    drives
}

/// 判断路径是否应该被排除扫描
fn should_skip(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    // 排除系统目录
    let excluded = [
        r"c:\windows",
        r"c:\program files\windowsapps",
    ];
    for exclude in &excluded {
        if path_str.starts_with(exclude) {
            return true;
        }
    }
    false
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn ql_search(query: String) -> Result<Vec<QuickLaunchResult>, String> {
    let fts_query = build_fts_query(&query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }
    with_conn(|conn| {
        let rows = db::do_ql_search(conn, &fts_query)?;
        Ok(rows.into_iter().map(|(id, name, path, ext, size, modified, drive)| {
            QuickLaunchResult {
                id, name, path, extension: ext,
                size_bytes: size, modified_at: modified, drive,
            }
        }).collect())
    })
}

#[tauri::command]
pub fn ql_index_status() -> Result<IndexStatus, String> {
    let is_building = {
        let state = indexing_state().lock().map_err(|e| e.to_string())?;
        state.is_some()
    };
    let drives = with_conn(|conn| {
        let rows = db::do_ql_get_meta(conn)?;
        Ok(rows.into_iter().map(|(drive, last, count, status)| {
            DriveIndexInfo { drive, last_scanned: last, file_count: count, status }
        }).collect::<Vec<_>>())
    })?;

    // 如果 meta 表为空，列出未索引的驱动器
    if drives.is_empty() {
        let drive_list: Vec<DriveIndexInfo> = get_fixed_drives().into_iter().map(|d| {
            DriveIndexInfo {
                drive: d,
                last_scanned: 0,
                file_count: 0,
                status: "pending".to_string(),
            }
        }).collect();
        return Ok(IndexStatus { drives: drive_list, is_building });
    }

    Ok(IndexStatus { drives, is_building })
}

#[tauri::command]
pub fn ql_build_index(app: AppHandle) -> Result<String, String> {
    let search_id = uuid::Uuid::new_v4().to_string();

    {
        let mut state = indexing_state().lock().map_err(|e| e.to_string())?;
        if state.is_some() {
            return Err("索引正在构建中".to_string());
        }
        state.replace(IndexingJob { search_id: search_id.clone(), cancelled: false });
    }

    let app_clone = app.clone();
    let sid = search_id.clone();

    std::thread::spawn(move || {
        let drives = get_fixed_drives();
        let mut total_files: i64 = 0;

        for drive in &drives {
            // 检查是否取消
            {
                let state = indexing_state().lock().unwrap();
                if let Some(ref job) = *state {
                    if job.cancelled {
                        let _ = app_clone.emit("ql-index-progress", QLIndexProgress {
                            search_id: sid.clone(),
                            files_scanned: total_files,
                            total_files,
                            current_drive: drive.clone(),
                            current_path: String::new(),
                            status: "cancelled".to_string(),
                            error: None,
                        });
                        return;
                    }
                }
            }

            let _ = app_clone.emit("ql-index-progress", QLIndexProgress {
                search_id: sid.clone(),
                files_scanned: total_files,
                total_files,
                current_drive: drive.clone(),
                current_path: format!("开始扫描 {}...", drive),
                status: "indexing".to_string(),
                error: None,
            });

            let mut batch: Vec<(String, String, String, i64, i64, String)> = Vec::new();
            let mut drive_count: i64 = 0;

            for entry in WalkDir::new(drive)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !should_skip(e.path()))
            {
                // 检查取消
                {
                    let state = indexing_state().lock().unwrap();
                    if let Some(ref job) = *state {
                        if job.cancelled {
                            return;
                        }
                    }
                }

                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();
                let name = entry.file_name().to_string_lossy().to_string();
                let ext = path.extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let size = match entry.metadata() {
                    Ok(m) => m.len() as i64,
                    Err(_) => 0,
                };
                let modified = match entry.metadata().and_then(|m| m.modified()) {
                    Ok(t) => t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                    Err(_) => 0,
                };

                batch.push((name, path_str, ext, size, modified, drive.clone()));
                drive_count += 1;
                total_files += 1;

                // 每 500 条或每 1000 条触发事件
                if batch.len() >= 500 {
                    if let Err(e) = with_conn(|conn| db::do_ql_insert_files(conn, &batch)) {
                        eprintln!("快速启动索引插入错误: {}", e);
                    }
                    batch.clear();

                    // 每 1000 条发送进度事件
                    if total_files % 1000 == 0 {
                        let _ = app_clone.emit("ql-index-progress", QLIndexProgress {
                            search_id: sid.clone(),
                            files_scanned: total_files,
                            total_files,
                            current_drive: drive.clone(),
                            current_path: entry.path().to_string_lossy().to_string(),
                            status: "indexing".to_string(),
                            error: None,
                        });
                    }
                }

                // 200ms 间隔检查
                if total_files % 5000 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }

            // 插入剩余批次
            if !batch.is_empty() {
                if let Err(e) = with_conn(|conn| db::do_ql_insert_files(conn, &batch)) {
                    eprintln!("快速启动索引插入错误: {}", e);
                }
            }

            // 更新驱动器索引元数据
            let _ = with_conn(|conn| db::do_ql_update_meta(conn, drive, drive_count, "ready"));
        }

        // 重建 FTS 索引
        let _ = with_conn(|conn| db::do_ql_rebuild_fts(conn));

        // 清理状态
        {
            let mut state = indexing_state().lock().unwrap();
            *state = None;
        }

        // 发送完成事件
        let _ = app_clone.emit("ql-index-progress", QLIndexProgress {
            search_id: sid.clone(),
            files_scanned: total_files,
            total_files,
            current_drive: String::new(),
            current_path: String::new(),
            status: "completed".to_string(),
            error: None,
        });
    });

    Ok(search_id)
}

#[tauri::command]
pub fn ql_rebuild_index(app: AppHandle) -> Result<String, String> {
    // 清空旧索引
    with_conn(|conn| db::do_ql_clear_all(conn))?;
    // 重新构建
    ql_build_index(app)
}

#[tauri::command]
pub fn ql_cancel_index() -> Result<(), String> {
    let mut state = indexing_state().lock().map_err(|e| e.to_string())?;
    if let Some(ref mut job) = *state {
        job.cancelled = true;
    }
    Ok(())
}

#[tauri::command]
pub async fn ql_open_file(path: String, app: AppHandle) -> Result<(), String> {
    app.shell().open(&path, None).map_err(|e| format!("打开文件失败: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: 在 main.rs 注册模块和命令**

在 `d:\work\litobox\src-tauri\src\main.rs` 的 `mod` 区域末尾（`mod file_renamer;` 之后）添加：
```rust
mod quick_launch;
```

在 `invoke_handler` 的 `generate_handler![]` 末尾（`file_renamer::rename_undo,` 之后）添加：
```rust
            quick_launch::ql_search,
            quick_launch::ql_index_status,
            quick_launch::ql_build_index,
            quick_launch::ql_rebuild_index,
            quick_launch::ql_cancel_index,
            quick_launch::ql_open_file,
```

- [ ] **Step 3: 构建确认**

Run: `cd d:\work\litobox\src-tauri && cargo check 2>&1 | head -40`
Expected: 编译通过，无错误（可能需要添加 `uuid` crate — 先确认是否已存在，若不存在则从 Task 回到 Step 1 调整：用简单字符串替代 uuid）

> 注意：检查 `Cargo.toml` 是否已有 `uuid` crate。如果不存在，`ql_build_index` 中改用 `std::time::SystemTime::now().duration_since(...).unwrap().as_nanos().to_string()` 生成 search_id。


### Task 3: 前端 — 类型定义和命令封装

**Files:**
- Create: `src/utils/quickLaunchClient.ts`

- [ ] **Step 1: 创建 quickLaunchClient.ts**

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface QuickLaunchResult {
  id: number
  name: string
  path: string
  extension: string
  sizeBytes: number
  modifiedAt: number
  drive: string
}

export interface DriveIndexInfo {
  drive: string
  lastScanned: number
  fileCount: number
  status: 'pending' | 'indexing' | 'ready' | 'failed'
}

export interface IndexStatus {
  drives: DriveIndexInfo[]
  isBuilding: boolean
}

export interface QLIndexProgress {
  searchId: string
  filesScanned: number
  totalFiles: number
  currentDrive: string
  currentPath: string
  status: 'indexing' | 'completed' | 'cancelled' | 'failed'
  error?: string
}

export async function qlSearch(query: string): Promise<QuickLaunchResult[]> {
  return invoke<QuickLaunchResult[]>('ql_search', { query })
}

export async function qlIndexStatus(): Promise<IndexStatus> {
  return invoke<IndexStatus>('ql_index_status')
}

export async function qlBuildIndex(): Promise<string> {
  return invoke<string>('ql_build_index')
}

export async function qlRebuildIndex(): Promise<string> {
  return invoke<string>('ql_rebuild_index')
}

export async function qlCancelIndex(): Promise<void> {
  return invoke<void>('ql_cancel_index')
}

export async function qlOpenFile(path: string): Promise<void> {
  return invoke<void>('ql_open_file', { path })
}
```


### Task 4: 前端 — TOOL_LIST 注册 + store 状态

**Files:**
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 TOOL_LIST 中添加快速启动条目**

在 `d:\work\litobox\src\store\index.ts` 中，在 `fileRenamer` 条目之后添加：

```typescript
  { id: 'quickLaunch', name: '快速启动', icon: '⚡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`, description: '全盘文件名快速搜索，一键打开', keywords: ['快速启动', '文件搜索', 'Everything', '启动', '搜索文件', '打开', 'quick', 'launch'], category: 'utility' },
```

- [ ] **Step 2: 确认构建**

Run: `cd d:\work\litobox && npx vue-tsc --noEmit 2>&1 | head -10`
Expected: 类型检查通过


### Task 5: 前端 — QuickLaunchTool.vue 工具页面

**Files:**
- Create: `src/views/QuickLaunchTool.vue`

- [ ] **Step 1: 创建 QuickLaunchTool.vue**

基于 `_ToolTemplate.vue` 模式，创建工具页面：

```vue
<template>
  <div class="tool-container">
    <!-- 索引状态提示 -->
    <div v-if="showIndexPrompt" class="tool-card">
      <div class="card-body">
        <div class="index-prompt">
          <span class="prompt-text">首次使用需要建立文件名索引</span>
          <el-button type="primary" size="small" :loading="isIndexing" @click="startIndex">
            {{ isIndexing ? '索引中...' : '开始建立索引' }}
          </el-button>
        </div>
      </div>
    </div>

    <!-- 索引进度 -->
    <div v-if="isIndexing && indexProgress" class="tool-card">
      <div class="card-body">
        <div class="index-progress">
          <div class="progress-text">
            已扫描 {{ indexProgress.filesScanned }} 个文件
            <span v-if="indexProgress.currentPath" class="progress-path">{{ indexProgress.currentPath }}</span>
          </div>
          <el-progress :percentage="indexPercent" :stroke-width="6" />
        </div>
      </div>
    </div>

    <!-- 搜索卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">快速启动</span>
        <div class="card-actions">
          <el-tooltip content="重建索引" placement="top">
            <el-button size="small" :icon="Refresh" :loading="isIndexing" @click="handleRebuild" />
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="search-input-wrapper">
          <el-input
            v-model="query"
            placeholder="搜索文件名…"
            size="large"
            clearable
            :disabled="isIndexing"
            @input="onQueryInput"
            @keydown="handleKeydown"
          />
        </div>
      </div>
    </div>

    <!-- 搜索结果 -->
    <div v-if="results.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">搜索结果 ({{ results.length }})</span>
      </div>
      <div class="card-body result-list">
        <div
          v-for="(item, idx) in results"
          :key="item.id"
          class="result-row"
          :class="{ selected: selectedIndex === idx }"
          @click="openFile(item.path)"
          @mouseenter="selectedIndex = idx"
        >
          <span class="result-icon">{{ getFileIcon(item.extension) }}</span>
          <div class="result-info">
            <span class="result-name">{{ item.name }}</span>
            <span class="result-path">{{ item.path }}</span>
          </div>
          <div class="result-meta">
            <span class="result-size">{{ formatSize(item.sizeBytes) }}</span>
            <span class="result-date">{{ formatDate(item.modifiedAt) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="searched && !isIndexing" class="tool-card">
      <div class="card-body">
        <el-empty :description="query ? '未找到匹配的文件' : '输入文件名开始搜索'" />
      </div>
    </div>

    <!-- 索引状态栏 -->
    <div v-if="indexStatus && indexStatus.drives.length > 0" class="index-status-bar">
      <span v-for="d in indexStatus.drives" :key="d.drive" class="drive-status">
        {{ d.drive }}
        <span :class="statusClass(d.status)">{{ statusLabel(d.status) }}</span>
        ({{ d.fileCount }} 文件)
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  type QuickLaunchResult,
  type IndexStatus,
  type QLIndexProgress,
  qlSearch,
  qlIndexStatus,
  qlBuildIndex,
  qlRebuildIndex,
  qlOpenFile,
} from '@/utils/quickLaunchClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const query = ref('')
const results = ref<QuickLaunchResult[]>([])
const selectedIndex = ref(0)
const searched = ref(false)
const isIndexing = ref(false)
const showIndexPrompt = ref(false)
const indexProgress = ref<QLIndexProgress | null>(null)
const indexStatus = ref<IndexStatus | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | null = null
let unlistens: UnlistenFn[] = []

onMounted(async () => {
  // 加载索引状态
  await refreshIndexStatus()

  // 监听索引进度
  unlistens.push(await listen<QLIndexProgress>('ql-index-progress', (e) => {
    const p = e.payload
    indexProgress.value = p
    if (p.status === 'completed') {
      isIndexing.value = false
      showIndexPrompt.value = false
      ElMessage.success('索引构建完成')
      refreshIndexStatus()
    } else if (p.status === 'cancelled') {
      isIndexing.value = false
      ElMessage.info('索引已取消')
    } else if (p.status === 'failed') {
      isIndexing.value = false
      ElMessage.error(p.error || '索引构建失败')
    }
  }))

  // 检查历史待还原
  if (store.pendingHistoryRestore?.tool === 'quickLaunch') {
    const data = store.pendingHistoryRestore
    query.value = data.input
    await doSearch()
    store.clearHistoryRestore()
  }
})

onUnmounted(() => {
  unlistens.forEach(fn => fn())
  if (searchTimer) clearTimeout(searchTimer)
})

async function refreshIndexStatus() {
  try {
    const status = await qlIndexStatus()
    indexStatus.value = status
    isIndexing.value = status.isBuilding
    const allReady = status.drives.length > 0 && status.drives.every(d => d.status === 'ready')
    showIndexPrompt.value = !allReady && !status.isBuilding
  } catch {
    showIndexPrompt.value = true
  }
}

async function startIndex() {
  isIndexing.value = true
  showIndexPrompt.value = false
  try {
    await qlBuildIndex()
  } catch (e) {
    isIndexing.value = false
    ElMessage.error(String(e))
  }
}

async function handleRebuild() {
  isIndexing.value = true
  results.value = []
  searched.value = false
  try {
    await qlRebuildIndex()
  } catch (e) {
    isIndexing.value = false
    ElMessage.error(String(e))
  }
}

function onQueryInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
}

async function doSearch() {
  if (!query.value.trim()) {
    results.value = []
    searched.value = false
    return
  }
  try {
    results.value = await qlSearch(query.value.trim())
    searched.value = true
    selectedIndex.value = 0
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = results.value[selectedIndex.value]
    if (item) openFile(item.path)
  }
}

async function openFile(path: string) {
  try {
    await qlOpenFile(path)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function getFileIcon(ext: string): string {
  const iconMap: Record<string, string> = {
    txt: '📄', md: '📝', pdf: '📕',
    doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
    ppt: '📙', pptx: '📙',
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', webp: '🖼️', bmp: '🖼️', svg: '🖼️',
    mp3: '🎵', wav: '🎵', flac: '🎵', m4a: '🎵', ogg: '🎵',
    mp4: '🎬', avi: '🎬', mkv: '🎬', mov: '🎬', webm: '🎬',
    zip: '📦', rar: '📦', '7z': '📦', gz: '📦',
    exe: '⚙️', dll: '⚙️', msi: '⚙️',
    js: '🟨', ts: '🟦', py: '🐍', rs: '🦀', go: '🔵',
    json: '📋', xml: '📋', yaml: '📋', yml: '📋', toml: '📋',
    html: '🌐', css: '🎨',
  }
  return iconMap[ext.toLowerCase()] || '📄'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`
}

function formatDate(ts: number): string {
  if (!ts) return ''
  return new Date(ts * 1000).toLocaleDateString('zh-CN', {
    month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
}

const indexPercent = computed(() => {
  if (!indexProgress.value || indexProgress.value.totalFiles === 0) return 0
  return Math.round((indexProgress.value.filesScanned / indexProgress.value.totalFiles) * 100)
})

function statusClass(s: string): string {
  return s === 'ready' ? 'status-ready' : s === 'indexing' ? 'status-indexing' : s === 'pending' ? 'status-pending' : 'status-failed'
}

function statusLabel(s: string): string {
  return s === 'ready' ? '✓' : s === 'indexing' ? '⟳' : s === 'pending' ? '⚠' : '✗'
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body { padding: 16px 20px; }

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input-wrapper {
  max-width: 600px;
}

.index-prompt {
  display: flex;
  align-items: center;
  gap: 12px;
}

.prompt-text {
  color: var(--text-secondary);
  font-size: 13px;
}

.index-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-text {
  font-size: 12px;
  color: var(--text-secondary);
}

.progress-path {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  word-break: break-all;
}

.result-list {
  padding: 0;
}

.result-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  cursor: pointer;
  transition: background 0.12s;
  border-bottom: 1px solid var(--border-color);
}
.result-row:last-child { border-bottom: none; }
.result-row:hover,
.result-row.selected {
  background: var(--bg-secondary);
}

.result-icon {
  flex-shrink: 0;
  font-size: 20px;
  width: 28px;
  text-align: center;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  display: block;
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.3;
}

.result-path {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-meta {
  flex-shrink: 0;
  text-align: right;
  font-size: 11px;
  color: var(--text-muted);
}

.result-size { display: block; }
.result-date { display: block; margin-top: 2px; }

.index-status-bar {
  display: flex;
  gap: 16px;
  padding: 8px 0;
  font-size: 12px;
  color: var(--text-muted);
}

.drive-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-ready { color: var(--accent-green); }
.status-indexing { color: var(--accent-cyan); }
.status-pending { color: var(--accent-orange); }
.status-failed { color: var(--accent-red); }
</style>
```


### Task 6: 前端 — QuickLaunchOverlay.vue 全局浮层

**Files:**
- Create: `src/components/QuickLaunchOverlay.vue`
- Modify: `src/store/index.ts` (添加 overlay 开关状态)
- Modify: `src/App.vue` (注册组件 + 监听快捷键)

- [ ] **Step 1: 创建 QuickLaunchOverlay.vue**

```vue
<template>
  <Teleport to="body">
    <div v-if="visible" class="ql-overlay" @click.self="close">
      <div class="ql-container" @click.stop>
        <input
          ref="inputRef"
          v-model="query"
          class="ql-input"
          type="text"
          placeholder="搜索文件名…"
          autocomplete="off"
          spellcheck="false"
          @keydown="handleKeydown"
        />
        <div v-if="results.length > 0" class="ql-results">
          <div
            v-for="(item, idx) in results"
            :key="item.id"
            class="ql-result-item"
            :class="{ active: selectedIndex === idx }"
            @click="openFile(item.path)"
            @mouseenter="selectedIndex = idx"
          >
            <span class="ql-result-icon">{{ getFileIcon(item.extension) }}</span>
            <div class="ql-result-info">
              <div class="ql-result-name">{{ item.name }}</div>
              <div class="ql-result-path">{{ item.path }}</div>
            </div>
          </div>
        </div>
        <div v-else-if="searched" class="ql-empty">未找到匹配的文件</div>
        <div class="ql-hint">
          <span>↑↓ 选择</span>
          <span>Enter 打开</span>
          <span>Esc 关闭</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { qlSearch, qlOpenFile, type QuickLaunchResult } from '@/utils/quickLaunchClient'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean] }>()

const query = ref('')
const results = ref<QuickLaunchResult[]>([])
const selectedIndex = ref(0)
const searched = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.visible, (v) => {
  if (v) {
    query.value = ''
    results.value = []
    selectedIndex.value = 0
    searched.value = false
    nextTick(() => inputRef.value?.focus())
  }
})

watch(query, () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
})

function close() {
  emit('update:visible', false)
}

async function doSearch() {
  if (!query.value.trim()) {
    results.value = []
    searched.value = false
    return
  }
  try {
    results.value = await qlSearch(query.value.trim())
    searched.value = true
    selectedIndex.value = 0
  } catch {
    results.value = []
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = results.value[selectedIndex.value]
    if (item) {
      qlOpenFile(item.path).catch(() => {})
      close()
    }
  } else if (e.key === 'Escape') {
    e.preventDefault()
    close()
  }
}

function getFileIcon(ext: string): string {
  const icons: Record<string, string> = {
    txt: '📄', pdf: '📕', doc: '📘', docx: '📘', xls: '📗', xlsx: '📗',
    jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️',
    mp3: '🎵', mp4: '🎬', zip: '📦', exe: '⚙️',
  }
  return icons[ext.toLowerCase()] || '📄'
}
</script>

<style scoped>
.ql-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.ql-container {
  width: 600px;
  max-width: 90vw;
  max-height: 70vh;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ql-input {
  width: 100%;
  padding: 14px 18px;
  background: var(--bg-input);
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 15px;
  outline: none;
  box-sizing: border-box;
}

.ql-input:focus {
  border-bottom-color: var(--accent-cyan);
}

.ql-results {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.ql-result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 9px 18px;
  cursor: pointer;
  transition: background 0.12s;
}

.ql-result-item.active,
.ql-result-item:hover {
  background: var(--bg-secondary);
}

.ql-result-icon {
  flex-shrink: 0;
  font-size: 18px;
  width: 24px;
  text-align: center;
}

.ql-result-info {
  flex: 1;
  min-width: 0;
}

.ql-result-name {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.3;
}

.ql-result-path {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.ql-empty {
  padding: 32px 18px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.ql-hint {
  display: flex;
  gap: 18px;
  padding: 8px 18px;
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
}
</style>
```

- [ ] **Step 2: 在 store 中添加 overlay 状态**

在 `src/store/index.ts` 中，`isCommandPaletteOpen` 相关代码之后添加：

```typescript
  // ============ 快速启动浮层 ============
  const isQuickLaunchOverlayOpen = ref(false)
  const openQuickLaunchOverlay = () => { isQuickLaunchOverlayOpen.value = true }
  const closeQuickLaunchOverlay = () => { isQuickLaunchOverlayOpen.value = false }
```

并 export：`isQuickLaunchOverlayOpen`, `openQuickLaunchOverlay`, `closeQuickLaunchOverlay`

在 `return { ... }` 语句中也加入这三个。


### Task 7: 前端 — App.vue 集成浮层 + 快捷键

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 在 App.vue 中导入并注册 QuickLaunchOverlay 组件**

在 `<script setup>` 的 import 区域添加：

```typescript
import QuickLaunchOverlay from '@/components/QuickLaunchOverlay.vue'
```

在 template 区域（`<CommandPalette />` 附近或 `</div>` 闭合前）添加：

```vue
<QuickLaunchOverlay v-model:visible="store.isQuickLaunchOverlayOpen" />
```

- [ ] **Step 2: 监听全局快捷键触发浮层**

在 `onMounted` 中，`listen('command-palette-triggered', ...)` 块之后，添加对快速启动浮层快捷键的监听。全局快捷键已在 `main.rs` 的 `db_read_shortcuts()` 中读取，我们需要在配置中注册 `__quick_launch__: 'Alt+Space'`。

在 `src/store/index.ts` 中 `shortcuts` 配置添加：
```typescript
__quick_launch__: 'Alt+Space',
```

然后在 `src/App.vue` 的 `onMounted` 中添加监听：
```typescript
// 监听快速启动浮层快捷键（Alt+Space 或其他配的键）
unlistenQuickLaunch = await listen('global-shortcut-triggered', (event) => {
  const toolId = event.payload as string
  if (toolId === '__quick_launch__') {
    if (store.isQuickLaunchOverlayOpen) {
      store.closeQuickLaunchOverlay()
    } else {
      store.openQuickLaunchOverlay()
    }
  }
})
```

在 `onUnmounted` 中清理：
```typescript
if (unlistenQuickLaunch) unlistenQuickLaunch()
```

- [ ] **Step 3: 配置默认快捷键**

在 `db.rs` 中找到 `db_read_shortcuts()` 函数，在 `__palette__` 条目确认后，确保快速启动快捷方式可被读取。

> **实际快捷键注册机制**：`main.rs` 中的 `setup` 从 `db_read_shortcuts()` 读取所有快捷键并注册。用户需要在应用内快捷键设置页面配置 `__quick_launch__` 的快捷键。默认值为 `Alt+Space`。

需要确认快捷键配置页面是否支持添加新的快捷键条目。查看 `shortcuts` 相关代码。

- [ ] **Step 4: 检查并确保构建通过**

Run: `cd d:\work\litobox && npx vue-tsc --noEmit 2>&1 | head -20`


### Task 8: 工作流集成

**Files:**
- Modify: `src/views/WorkflowView.vue`

- [ ] **Step 1: 添加快捷启动到 executeStep 分支**

在 `WorkflowView.vue` 中找到 `executeStep()` 函数（类似其他工具的 switch/case 分支），添加：

```typescript
case 'quickLaunch':
  result = await qlSearch(input)
  outputText = JSON.stringify(result, null, 2)
  break
```

并确保 import 了 `qlSearch` 从 `@/utils/quickLaunchClient`。


### Task 9: 历史记录集成

**Files:**
- Modify: `src/views/QuickLaunchTool.vue`

- [ ] **Step 1: 在搜索完成后记录历史**

在 `doSearch()` 函数中，搜索成功且 query 非空时，调用 `store.addHistory()`：

```typescript
// 在 doSearch 中搜索成功后添加
store.addHistory({
  tool: 'quickLaunch',
  action: '文件名搜索',
  inputPreview: query.value.trim().slice(0, 50),
  outputPreview: `${results.value.length} 条结果`.slice(0, 50),
  inputFull: query.value.trim(),
  outputFull: JSON.stringify(results.value.map(r => ({ name: r.name, path: r.path }))),
})
```

> 注意：只在首次搜索结果变化时记录历史，避免每次防抖搜索都产生新记录。可以使用一个 `lastRecordedQuery` 变量做去重。


### Task 10: 变量池集成

**Files:**
- Modify: `src/views/QuickLaunchTool.vue`

- [ ] **Step 1: 在输入区添加 VariablePicker**

在搜索输入框附近添加 `VariablePicker` 组件（放在 card-header 的 card-actions 中）：

```vue
<VariablePicker @select="(v: string) => { query = v; onQueryInput() }" />
```

并添加 import：`import VariablePicker from '@/components/VariablePicker.vue'`


### 验证

所有任务完成后，执行：

1. `cd d:\work\litobox\src-tauri && cargo build` — Rust 编译通过
2. `cd d:\work\litobox && npm run build` — 前端构建通过
3. `npm run tauri dev` — 启动应用验证功能
