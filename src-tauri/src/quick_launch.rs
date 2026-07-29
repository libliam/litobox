use crate::db::{self, with_conn};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;
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
#[serde(rename_all = "camelCase")]
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

static INDEXING_JOB: Mutex<Option<IndexingJob>> = Mutex::new(None);

struct IndexingJob {
    search_id: String,
    cancelled: bool,
}

// ============ 工具函数 ============

/// 构建 FTS5 查询字符串
/// - 纯 ASCII 字母数字输入：按空白分词，每个词追加 `*` 做前缀匹配
/// - 包含中文/其他 Unicode 字符：逐字符拆分并 AND 匹配
fn build_fts_query(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // 判断是否全为 ASCII 字母数字
    let is_ascii_alphanum =
        trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_');
    if is_ascii_alphanum {
        // 按空白和连接符拆分（防止 - 被 FTS5 解析为 NOT 运算符或列语法），每个词追加 *
        trimmed
            .split(|c: char| c.is_whitespace() || c == '-')
            .filter(|s| !s.is_empty())
            .map(|s| format!("{}*", s))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        // 包含非 ASCII 字符：过滤 FTS5 特殊字符后处理
        let sanitized: String = trimmed
            .chars()
            .filter(|c| !c.is_whitespace() && !matches!(c, '.' | ':' | '"' | '(' | ')' | '+' | '^' | '!' | '@' | '#' | '$' | '%' | '&' | '=' | '/' | '\\' | '?' | '<' | '>' | '~' | '`' | '{' | '[' | '}' | ']' | '|' | ',' | ';'))
            .collect();
        if sanitized.is_empty() {
            return String::new();
        }
        // 如果过滤后只剩纯 ASCII 字母数字，当作整体搜索（如 "filetxt" 搜 file.txt）
        if sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return format!("{}*", sanitized.trim());
        }
        // 逐字符 AND 匹配
        sanitized
            .chars()
            .map(|c| format!("{}*", c))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// 获取所有固定驱动器列表
fn get_fixed_drives() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let drives = unsafe { windows_sys::Win32::Storage::FileSystem::GetLogicalDrives() };
        let mut result = Vec::new();
        for i in 0..26 {
            if drives & (1 << i) != 0 {
                let letter = (b'A' + i as u8) as char;
                let root = format!("{}:\\", letter);
                // GetDriveTypeA 接受 *const u8 (PCSTR), CString::as_ptr() 返回 *const i8
                let c_root = std::ffi::CString::new(root.as_str()).unwrap();
                let dt = unsafe {
                    windows_sys::Win32::Storage::FileSystem::GetDriveTypeA(
                        c_root.as_ptr() as *const u8,
                    )
                };
                // DRIVE_FIXED = 3 (定义在 windows_sys::Win32::System::WindowsProgramming)
                if dt == 3 {
                    result.push(root);
                }
            }
        }
        result
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec!["/".to_string()]
    }
}

/// 判断路径是否应该被排除扫描
fn should_skip(path: &std::path::Path) -> bool {
    let p = path.to_string_lossy().to_lowercase();
    let excluded = [r"c:\windows", r"c:\program files\windowsapps"];
    for e in &excluded {
        if p.starts_with(e) {
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
        Ok(rows
            .into_iter()
            .map(
                |(id, name, path, extension, size_bytes, modified_at, drive)| QuickLaunchResult {
                    id,
                    name,
                    path,
                    extension,
                    size_bytes,
                    modified_at,
                    drive,
                },
            )
            .collect())
    })
}

#[tauri::command]
pub fn ql_index_status() -> Result<IndexStatus, String> {
    let is_building = {
        let guard = INDEXING_JOB.lock().map_err(|e| e.to_string())?;
        guard.is_some()
    };
    let drives = with_conn(|conn| {
        let rows = db::do_ql_get_meta(conn)?;
        Ok(rows
            .into_iter()
            .map(|(drive, last_scanned, file_count, status)| DriveIndexInfo {
                drive,
                last_scanned,
                file_count,
                status,
            })
            .collect::<Vec<_>>())
    })?;

    if drives.is_empty() {
        let drive_list: Vec<DriveIndexInfo> = get_fixed_drives()
            .into_iter()
            .map(|d| DriveIndexInfo {
                drive: d,
                last_scanned: 0,
                file_count: 0,
                status: "pending".to_string(),
            })
            .collect();
        return Ok(IndexStatus {
            drives: drive_list,
            is_building,
        });
    }

    Ok(IndexStatus { drives, is_building })
}

#[tauri::command]
pub fn ql_build_index(app: AppHandle) -> Result<String, String> {
    let search_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();

    {
        let mut guard = INDEXING_JOB.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("索引正在构建中".to_string());
        }
        guard.replace(IndexingJob {
            search_id: search_id.clone(),
            cancelled: false,
        });
    }

    let app_clone = app.clone();
    let sid = search_id.clone();

    std::thread::spawn(move || {
        // ponytail: 降低索引线程优先级，防止系统卡顿
        #[cfg(target_os = "windows")]
        unsafe {
            let handle = windows_sys::Win32::System::Threading::GetCurrentThread();
            windows_sys::Win32::System::Threading::SetThreadPriority(
                handle,
                windows_sys::Win32::System::Threading::THREAD_PRIORITY_BELOW_NORMAL,
            );
        }
        let drives = get_fixed_drives();
        let mut total_files: i64 = 0;

        for drive in &drives {
            // 检查取消
            {
                let guard = INDEXING_JOB.lock().unwrap();
                if let Some(ref job) = *guard {
                    if job.cancelled {
                        let _ = app_clone.emit(
                            "ql-index-progress",
                            QLIndexProgress {
                                search_id: sid.clone(),
                                files_scanned: total_files,
                                total_files,
                                current_drive: drive.clone(),
                                current_path: String::new(),
                                status: "cancelled".to_string(),
                                error: None,
                            },
                        );
                        return;
                    }
                }
            }

            let _ = app_clone.emit(
                "ql-index-progress",
                QLIndexProgress {
                    search_id: sid.clone(),
                    files_scanned: total_files,
                    total_files,
                    current_drive: drive.clone(),
                    current_path: format!("开始扫描 {}...", drive),
                    status: "indexing".to_string(),
                    error: None,
                },
            );

            let mut batch: Vec<(String, String, String, i64, i64, String)> = Vec::new();
            let mut drive_count: i64 = 0;

            for entry in WalkDir::new(drive)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !should_skip(e.path()))
            {
                // 检查取消
                {
                    let guard = INDEXING_JOB.lock().unwrap();
                    if let Some(ref job) = *guard {
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
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let size = match entry.metadata() {
                    Ok(m) => m.len() as i64,
                    Err(_) => 0,
                };
                let modified = match entry.metadata() {
                    Ok(m) => match m.modified() {
                        Ok(t) => t
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                };

                batch.push((name, path_str, ext, size, modified, drive.clone()));
                drive_count += 1;
                total_files += 1;

                // 每 500 条提交一批
                if batch.len() >= 500 {
                    if let Err(e) = with_conn(|conn| db::do_ql_insert_files(conn, &batch)) {
                        eprintln!("快速启动索引插入错误: {}", e);
                    }
                    batch.clear();

                    // ponytail: 每批提交后短暂让出 CPU，防止 IO 饱和导致 UI 卡顿
                    std::thread::sleep(Duration::from_millis(2));

                    // 每 1000 条发送进度事件
                    if total_files % 1000 == 0 {
                        let _ = app_clone.emit(
                            "ql-index-progress",
                            QLIndexProgress {
                                search_id: sid.clone(),
                                files_scanned: total_files,
                                total_files,
                                current_drive: drive.clone(),
                                current_path: entry.path().to_string_lossy().to_string(),
                                status: "indexing".to_string(),
                                error: None,
                            },
                        );
                    }
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
            let mut guard = INDEXING_JOB.lock().unwrap();
            *guard = None;
        }

        // 发送完成事件
        let _ = app_clone.emit(
            "ql-index-progress",
            QLIndexProgress {
                search_id: sid.clone(),
                files_scanned: total_files,
                total_files,
                current_drive: String::new(),
                current_path: String::new(),
                status: "completed".to_string(),
                error: None,
            },
        );
    });

    Ok(search_id)
}

#[tauri::command]
pub fn ql_rebuild_index(app: AppHandle) -> Result<String, String> {
    with_conn(|conn| db::do_ql_clear_all(conn))?;
    ql_build_index(app)
}

#[tauri::command]
pub fn ql_cancel_index() -> Result<(), String> {
    let mut guard = INDEXING_JOB.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut job) = *guard {
        job.cancelled = true;
    }
    Ok(())
}

#[tauri::command]
pub async fn ql_open_file(path: String, app: AppHandle) -> Result<(), String> {
    app.shell()
        .open(&path, None)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    Ok(())
}
