use std::fs;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};

/// 文件条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub ext: String,
}

/// 重命名规则
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameRules {
    pub mode: String, // "replace" | "regex" | "prefix_suffix" | "sequence"
    // replace
    pub find_text: Option<String>,
    pub replace_text: Option<String>,
    pub case_sensitive: Option<bool>,
    // regex
    pub pattern: Option<String>,
    pub replacement: Option<String>,
    // prefix_suffix
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    // sequence
    pub seq_prefix: Option<String>,
    pub seq_suffix: Option<String>,
    pub start_number: Option<i32>,
    pub padding: Option<i32>,
}

/// 重命名预览结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenamePreview {
    pub old_name: String,
    pub new_name: String,
    pub error: Option<String>,
}

/// 重命名对
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenamePair {
    pub old_path: String,
    pub new_path: String,
}

/// 重命名执行结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenameExecuteResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<(String, String)>, // (old_name, error)
    pub backups: Vec<RenamePair>,        // for undo
}

/// 列出指定目录下的文件（不含子目录）
#[tauri::command]
pub fn rename_list_files(path: String) -> Result<Vec<FileEntry>, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err("路径不是一个有效的目录".to_string());
    }

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let metadata = entry.metadata().map_err(|e| format!("获取文件信息失败: {}", e))?;

        let file_path = entry.path();
        let name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let ext = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        entries.push(FileEntry {
            name,
            path: file_path.to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            ext,
        });
    }

    // 目录排前面，文件按名称排序
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

/// 预览重命名结果
#[tauri::command]
pub fn rename_preview(files: Vec<String>, rules: RenameRules) -> Result<Vec<RenamePreview>, String> {
    let mut previews = Vec::new();

    for (i, file_name) in files.iter().enumerate() {
        let result = apply_rule(file_name, &rules, i);
        match result {
            Ok(new_name) => {
                let error = if new_name == *file_name {
                    Some("文件名未变化".to_string())
                } else {
                    None
                };
                previews.push(RenamePreview {
                    old_name: file_name.clone(),
                    new_name,
                    error,
                });
            }
            Err(e) => {
                previews.push(RenamePreview {
                    old_name: file_name.clone(),
                    new_name: file_name.clone(),
                    error: Some(e),
                });
            }
        }
    }

    // 检查重名冲突
    let mut name_count: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (i, p) in previews.iter().enumerate() {
        if p.error.is_none() {
            name_count.entry(p.new_name.clone()).or_default().push(i);
        }
    }
    for (name, indices) in &name_count {
        if indices.len() > 1 {
            for &idx in indices {
                let existing = &mut previews[idx];
                let conflict_msg = if existing.error.as_deref() == Some("文件名未变化") {
                    format!("与 \"{}\" 重名（共 {} 个）", name, indices.len())
                } else {
                    format!("与 \"{}\" 冲突（共 {} 个）", name, indices.len())
                };
                existing.error = Some(conflict_msg);
            }
        }
    }

    Ok(previews)
}

/// 执行重命名
#[tauri::command]
pub fn rename_execute(renames: Vec<RenamePair>) -> Result<RenameExecuteResult, String> {
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut failures = Vec::new();
    let mut backups = Vec::new();

    for pair in &renames {
        let old_path = Path::new(&pair.old_path);
        let new_path = Path::new(&pair.new_path);

        if !old_path.exists() {
            failed_count += 1;
            failures.push((
                old_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                "文件不存在".to_string(),
            ));
            continue;
        }

        // 如果新路径已存在且不是同一个文件，跳过
        if new_path.exists() && old_path.canonicalize().ok() != new_path.canonicalize().ok() {
            failed_count += 1;
            failures.push((
                old_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                format!(
                    "目标文件已存在: {}",
                    new_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                ),
            ));
            continue;
        }

        match fs::rename(&pair.old_path, &pair.new_path) {
            Ok(_) => {
                success_count += 1;
                // 备份用于撤销：反向映射
                backups.push(RenamePair {
                    old_path: pair.new_path.clone(),
                    new_path: pair.old_path.clone(),
                });
            }
            Err(e) => {
                failed_count += 1;
                failures.push((
                    old_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    e.to_string(),
                ));
            }
        }
    }

    Ok(RenameExecuteResult {
        success_count,
        failed_count,
        failures,
        backups,
    })
}

/// 撤销重命名
#[tauri::command]
pub fn rename_undo(backups: Vec<RenamePair>) -> Result<RenameExecuteResult, String> {
    rename_execute(backups)
}

/// 选择文件夹对话框
#[tauri::command]
pub async fn rename_pick_folder(app: AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc::sync_channel;
    let (tx, rx) = sync_channel(0);

    app.dialog()
        .file()
        .set_title("选择文件夹")
        .pick_folder(move |file_path: Option<FilePath>| {
            let result = match file_path {
                Some(FilePath::Path(p)) => Some(p.to_string_lossy().to_string()),
                Some(FilePath::Url(url)) => url.to_file_path()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                None => None,
            };
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("接收结果失败: {}", e))
}

// ============ 规则应用 ============

fn apply_rule(file_name: &str, rules: &RenameRules, index: usize) -> Result<String, String> {
    match rules.mode.as_str() {
        "replace" => apply_replace(file_name, rules),
        "regex" => apply_regex(file_name, rules),
        "prefix_suffix" => apply_prefix_suffix(file_name, rules),
        "sequence" => apply_sequence(file_name, rules, index),
        _ => Err(format!("未知的重命名模式: {}", rules.mode)),
    }
}

fn apply_replace(file_name: &str, rules: &RenameRules) -> Result<String, String> {
    let find_text = rules.find_text.as_deref().unwrap_or("");
    let replace_text = rules.replace_text.as_deref().unwrap_or("");
    let case_sensitive = rules.case_sensitive.unwrap_or(true);

    if find_text.is_empty() {
        return Ok(file_name.to_string());
    }

    if case_sensitive {
        Ok(file_name.replace(find_text, replace_text))
    } else {
        // 大小写不敏感：用正则替换
        let re = Regex::new(&regex::escape(find_text))
            .map_err(|e| format!("正则编译失败: {}", e))?;
        Ok(re.replace_all(file_name, replace_text).to_string())
    }
}

fn apply_regex(file_name: &str, rules: &RenameRules) -> Result<String, String> {
    let pattern = rules.pattern.as_deref().unwrap_or("");
    let replacement = rules.replacement.as_deref().unwrap_or("");

    if pattern.is_empty() {
        return Ok(file_name.to_string());
    }

    let re = Regex::new(pattern).map_err(|e| format!("正则表达式无效: {}", e))?;
    Ok(re.replace_all(file_name, replacement).to_string())
}

fn apply_prefix_suffix(file_name: &str, rules: &RenameRules) -> Result<String, String> {
    let prefix = rules.prefix.as_deref().unwrap_or("");
    let suffix = rules.suffix.as_deref().unwrap_or("");

    // 分离文件名和扩展名
    let (stem, ext) = split_name_ext(file_name);

    if ext.is_empty() {
        Ok(format!("{}{}{}", prefix, stem, suffix))
    } else {
        Ok(format!("{}{}{}.{}", prefix, stem, suffix, ext))
    }
}

fn apply_sequence(file_name: &str, rules: &RenameRules, index: usize) -> Result<String, String> {
    let start = rules.start_number.unwrap_or(1) as usize;
    let pad = rules.padding.unwrap_or(2) as usize;
    let seq_prefix = rules.seq_prefix.as_deref().unwrap_or("");
    let seq_suffix = rules.seq_suffix.as_deref().unwrap_or("");

    let (_, ext) = split_name_ext(file_name);
    let num = start + index;
    let padded = format!("{:0width$}", num, width = pad);

    if ext.is_empty() {
        Ok(format!("{}{}{}", seq_prefix, padded, seq_suffix))
    } else {
        Ok(format!("{}{}{}.{}", seq_prefix, padded, seq_suffix, ext))
    }
}

/// 分离文件名（不含扩展名）和扩展名
fn split_name_ext(file_name: &str) -> (&str, &str) {
    let dot_idx = file_name.rfind('.');
    match dot_idx {
        Some(i) if i > 0 => {
            let (stem, ext) = file_name.split_at(i);
            // 跳过点号
            (stem, &ext[1..])
        }
        _ => (file_name, ""),
    }
}
