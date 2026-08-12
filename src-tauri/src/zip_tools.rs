use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use zip::unstable::write::FileOptionsExt;
use zip::write::FileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

// ============ 返回结构（snake_case 字段） ============

#[derive(serde::Serialize)]
pub struct ZipEntry {
    name: String,
    is_dir: bool,
    size: u64,
    compressed_size: u64,
    modified: Option<String>,
    crc32: u32,
}

#[derive(serde::Serialize)]
pub struct ZipCreateResult {
    path: String,
    file_count: usize,
    dir_count: usize,
    total_size: u64,
    compressed_size: u64,
    ratio: f64, // 压缩率 = 压缩后 / 原始（0~1）
}

#[derive(serde::Serialize)]
pub struct ZipExtractResult {
    path: String,
    file_count: usize,
    extracted_bytes: u64,
    skipped_existing: usize,
    skipped_unsafe: usize,
}

// ============ 工具函数 ============

/// 收集文件，返回 (根路径, 文件完整路径) 列表；目录递归展开
fn collect_files(paths: &[String]) -> (Vec<(PathBuf, PathBuf)>, usize) {
    let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut dirs = 0usize;
    for p in paths {
        let p = Path::new(p);
        if p.is_dir() {
            for entry in WalkDir::new(p).follow_links(false).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_dir() {
                    dirs += 1;
                } else if entry.file_type().is_file() {
                    files.push((p.to_path_buf(), entry.path().to_path_buf()));
                }
            }
        } else if p.is_file() {
            // 单文件：根取父目录，ZIP 内仅含文件名
            let root = p.parent().map(|q| q.to_path_buf()).unwrap_or_default();
            files.push((root, p.to_path_buf()));
        }
    }
    // 按相对路径排序保证可重复性
    files.sort_by(|a, b| rel_path(&a.0, &a.1).cmp(&rel_path(&b.0, &b.1)));
    (files, dirs)
}

/// 计算相对路径（以传入的根为基准），Windows 分隔符转 /
fn rel_path(root: &Path, full: &Path) -> String {
    full.strip_prefix(root)
        .unwrap_or(full)
        .to_string_lossy()
        .replace('\\', "/")
}

fn compression_from_level(level: Option<u8>) -> (CompressionMethod, Option<i32>) {
    match level {
        Some(0) => (CompressionMethod::Stored, None),
        Some(l) => (CompressionMethod::Deflated, Some(l.clamp(1, 9) as i32)),
        None => (CompressionMethod::Deflated, Some(6)),
    }
}

/// zip::DateTime 无 Display，手动格式化 YYYY-MM-DD HH:MM:SS
fn format_modified(t: DateTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )
}

// ============ 压缩 ============

#[tauri::command]
pub fn zip_create(
    app: AppHandle,
    files: Vec<String>,
    dest: String,
    level: Option<u8>,
    password: Option<String>,
) -> Result<ZipCreateResult, String> {
    if files.is_empty() {
        return Err("请至少选择一个文件或文件夹".to_string());
    }
    if dest.trim().is_empty() {
        return Err("请指定输出 ZIP 文件路径".to_string());
    }
    let (files_list, dir_count) = collect_files(&files);
    if files_list.is_empty() {
        return Err("所选内容中没有可压缩的文件".to_string());
    }
    let total = files_list.len();
    let total_size: u64 = files_list
        .iter()
        .map(|(_, p)| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();

    let out_file = File::create(&dest).map_err(|e| format!("创建输出文件失败: {}", e))?;
    let mut writer = ZipWriter::new(BufWriter::new(out_file));
    let (method, clevel) = compression_from_level(level);

    let mut size_src_total: u64 = 0;

    for (i, (root, path)) in files_list.iter().enumerate() {
        let name = rel_path(root, path);

        let mut opts = FileOptions::default()
            .compression_method(method)
            .compression_level(clevel);
        // zip 0.6.6 写入加密仅支持 ZipCrypto（unstable API，Windows/7-Zip/WinRAR 均可解压）
        if let Some(pw) = &password {
            opts = opts.with_deprecated_encryption(pw.as_bytes());
        }

        let data = std::fs::read(path).map_err(|e| format!("读取 {} 失败: {}", name, e))?;
        let raw_len = data.len() as u64;
        writer
            .start_file(&name, opts)
            .map_err(|e| format!("写入条目 {} 失败: {}", name, e))?;
        writer
            .write_all(&data)
            .map_err(|e| format!("写入 {} 失败: {}", name, e))?;
        size_src_total += raw_len;

        let percent = (i + 1) as f64 / total as f64 * 100.0;
        let _ = app.emit(
            "zip-progress",
            serde_json::json!({
                "stage": "create",
                "percent": percent,
                "current": i + 1,
                "total": total,
                "file": name,
            }),
        );
    }

    let final_writer = writer
        .finish()
        .map_err(|e| format!("完成 ZIP 写入失败: {}", e))?;
    drop(final_writer);

    let compressed_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    let ratio = if size_src_total > 0 {
        compressed_size as f64 / size_src_total as f64
    } else {
        0.0
    };

    let _ = app.emit(
        "zip-progress",
        serde_json::json!({ "stage": "done", "percent": 100.0 }),
    );

    Ok(ZipCreateResult {
        path: dest,
        file_count: total,
        dir_count,
        total_size,
        compressed_size,
        ratio,
    })
}

// ============ 列出内容 ============

#[tauri::command]
pub fn zip_list(path: String, password: Option<String>) -> Result<Vec<ZipEntry>, String> {
    let file = File::open(&path).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;
    let mut entries = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        // 有密码：解密读取（校验密码是否正确）；无密码：仅读取元数据（加密条目返回需要密码）
        let entry = match &password {
            Some(pw) => match archive.by_index_decrypt(i, pw.as_bytes()) {
                Ok(Ok(f)) => f,
                Ok(Err(_)) => return Err("密码错误，无法解密该压缩包".to_string()),
                Err(_) => return Err("读取条目失败".to_string()),
            },
            None => match archive.by_index(i) {
                Ok(f) => f,
                Err(zip::result::ZipError::UnsupportedArchive(msg))
                    if msg.contains("Password required") =>
                {
                    return Err("该压缩包已加密，请输入密码后重试".to_string());
                }
                Err(e) => return Err(format!("读取条目失败: {}", e)),
            },
        };
        entries.push(ZipEntry {
            name: entry.name().to_string(),
            is_dir: entry.is_dir(),
            size: entry.size(),
            compressed_size: entry.compressed_size(),
            modified: Some(format_modified(entry.last_modified())),
            crc32: entry.crc32(),
        });
    }
    Ok(entries)
}

// ============ 解压 ============

#[tauri::command]
pub fn zip_extract(
    app: AppHandle,
    path: String,
    dest: String,
    entries: Option<Vec<String>>,
    password: Option<String>,
    overwrite: Option<bool>,
) -> Result<ZipExtractResult, String> {
    let file = File::open(&path).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

    std::fs::create_dir_all(&dest).map_err(|e| format!("创建解压目录失败: {}", e))?;
    let dest_path = PathBuf::from(&dest);
    let overwrite = overwrite.unwrap_or(false);

    // 先读取全部条目元数据（不解密），避免闭包借用冲突
    let mut all_names: Vec<String> = Vec::with_capacity(archive.len());
    let mut all_sizes: Vec<u64> = Vec::with_capacity(archive.len());
    let mut all_is_dir: Vec<bool> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let e = archive
            .by_index_raw(i)
            .map_err(|e| format!("读取条目失败: {}", e))?;
        all_names.push(e.name().to_string());
        all_sizes.push(e.size());
        all_is_dir.push(e.is_dir());
    }

    // 计算选中条目
    let selected: Vec<usize> = if let Some(ref wanted) = entries {
        (0..all_names.len())
            .filter(|&i| wanted.iter().any(|w| w == &all_names[i]))
            .collect()
    } else {
        (0..all_names.len()).collect()
    };
    let total_bytes: u64 = selected
        .iter()
        .map(|&i| if all_is_dir[i] { 0 } else { all_sizes[i] })
        .sum();

    let mut file_count = 0usize;
    let mut extracted_bytes = 0u64;
    let mut skipped_existing = 0usize;
    let mut skipped_unsafe = 0usize;

    for (idx, &i) in selected.iter().enumerate() {
        // 有密码：解密读取；无密码：普通读取（加密条目会报需要密码）
        let mut entry = match &password {
            Some(pw) => match archive.by_index_decrypt(i, pw.as_bytes()) {
                Ok(Ok(f)) => f,
                Ok(Err(_)) => return Err("密码错误，无法解密该压缩包".to_string()),
                Err(_) => return Err("读取条目失败".to_string()),
            },
            None => match archive.by_index(i) {
                Ok(f) => f,
                Err(zip::result::ZipError::UnsupportedArchive(msg))
                    if msg.contains("Password required") =>
                {
                    return Err("该压缩包已加密，请输入密码后重试".to_string());
                }
                Err(e) => return Err(format!("读取条目失败: {}", e)),
            },
        };

        // 防路径穿越：enclosed_name 拒绝包含 .. 的名字
        let Some(rel) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            skipped_unsafe += 1;
            continue;
        };
        let out_path = dest_path.join(&rel);

        if entry.is_dir() {
            let _ = std::fs::create_dir_all(&out_path);
            continue;
        }
        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if out_path.exists() && !overwrite {
            skipped_existing += 1;
            continue;
        }

        let mut out_file = File::create(&out_path)
            .map_err(|e| format!("创建 {} 失败: {}", out_path.display(), e))?;
        let copied = std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("解压 {} 失败: {}", entry.name(), e))?;
        file_count += 1;
        extracted_bytes += copied;

        // 进度 = 已解压字节 / 总字节（按原始大小估算）
        let percent = if total_bytes > 0 {
            (extracted_bytes as f64 / total_bytes as f64 * 100.0).min(100.0)
        } else {
            (idx + 1) as f64 / selected.len().max(1) as f64 * 100.0
        };
        let _ = app.emit(
            "zip-progress",
            serde_json::json!({
                "stage": "extract",
                "percent": percent,
                "current": idx + 1,
                "total": selected.len(),
                "file": entry.name(),
            }),
        );
    }

    let _ = app.emit(
        "zip-progress",
        serde_json::json!({ "stage": "done", "percent": 100.0 }),
    );

    Ok(ZipExtractResult {
        path: dest,
        file_count,
        extracted_bytes,
        skipped_existing,
        skipped_unsafe,
    })
}
