use encoding_rs::{Encoding, UTF_8, GBK, GB18030, WINDOWS_1252};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use regex::Regex;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销），沿用项目惯例
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

fn get_encoding(name: &str) -> &'static Encoding {
    match name.to_lowercase().as_str() {
        "utf-8" | "utf8" => UTF_8,
        "gbk" => GBK,
        "gb2312" | "gb18030" => GB18030,
        "iso-8859-1" | "latin1" | "iso8859-1" => WINDOWS_1252,
        _ => UTF_8,
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileResult {
    pub path: String,
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BatchConvertResult {
    pub path: String,
    pub output_path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn read_file_with_encoding(path: String, encoding: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    let enc = get_encoding(&encoding);
    let (decoded, _, _) = enc.decode(&bytes);
    Ok(decoded.into_owned())
}

#[tauri::command]
pub async fn batch_read_txt_files(paths: Vec<String>, encoding: String) -> Result<Vec<FileResult>, String> {
    let enc = get_encoding(&encoding);
    let mut results = Vec::new();
    
    for path in paths {
        match fs::read(&path) {
            Ok(bytes) => {
                let (decoded, _, _) = enc.decode(&bytes);
                results.push(FileResult {
                    path,
                    success: true,
                    content: Some(decoded.into_owned()),
                    error: None,
                });
            }
            Err(e) => {
                results.push(FileResult {
                    path,
                    success: false,
                    content: None,
                    error: Some(format!("读取失败: {}", e)),
                });
            }
        }
    }
    
    Ok(results)
}

#[tauri::command]
pub async fn batch_replace_in_files(
    paths: Vec<String>,
    search: String,
    replacement: String,
    encoding: String,
    output_dir: Option<String>,
) -> Result<Vec<FileResult>, String> {
    let enc = get_encoding(&encoding);
    let mut results = Vec::new();
    
    for path in paths {
        match fs::read(&path) {
            Ok(bytes) => {
                let (decoded, _, _) = enc.decode(&bytes);
                let replaced = decoded.replace(&search, &replacement);
                
                let output_path = if let Some(ref dir) = output_dir {
                    let file_name = Path::new(&path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    format!("{}/{}", dir.trim_end_matches('/').trim_end_matches('\\'), file_name)
                } else {
                    path.clone()
                };
                
                let (encoded, _, _) = enc.encode(&replaced);
                if let Err(e) = fs::write(&output_path, encoded.as_ref()) {
                    results.push(FileResult {
                        path,
                        success: false,
                        content: None,
                        error: Some(format!("写入失败: {}", e)),
                    });
                    continue;
                }
                
                results.push(FileResult {
                    path,
                    success: true,
                    content: Some(replaced),
                    error: None,
                });
            }
            Err(e) => {
                results.push(FileResult {
                    path,
                    success: false,
                    content: None,
                    error: Some(format!("读取失败: {}", e)),
                });
            }
        }
    }
    
    Ok(results)
}

#[tauri::command]
pub async fn batch_convert_encoding(
    paths: Vec<String>,
    from_encoding: String,
    to_encoding: String,
    output_dir: String,
) -> Result<Vec<BatchConvertResult>, String> {
    let from_enc = get_encoding(&from_encoding);
    let to_enc = get_encoding(&to_encoding);
    let mut results = Vec::new();
    
    // 确保输出目录存在
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("创建输出目录失败: {}", e));
    }
    
    for path in paths {
        let file_name = Path::new(&path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let output_path = format!("{}/{}", output_dir.trim_end_matches('/').trim_end_matches('\\'), file_name);
        
        match fs::read(&path) {
            Ok(bytes) => {
                let (decoded, _, _) = from_enc.decode(&bytes);
                let (encoded, _, _) = to_enc.encode(&decoded);
                
                if let Err(e) = fs::write(&output_path, encoded.as_ref()) {
                    results.push(BatchConvertResult {
                        path,
                        output_path,
                        success: false,
                        error: Some(format!("写入失败: {}", e)),
                    });
                } else {
                    results.push(BatchConvertResult {
                        path,
                        output_path,
                        success: true,
                        error: None,
                    });
                }
            }
            Err(e) => {
                results.push(BatchConvertResult {
                    path,
                    output_path,
                    success: false,
                    error: Some(format!("读取失败: {}", e)),
                });
            }
        }
    }
    
    Ok(results)
}

#[tauri::command]
pub async fn convert_file_encoding(
    path: String,
    from_encoding: String,
    to_encoding: String,
    output_path: String,
) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    
    let from_enc = get_encoding(&from_encoding);
    let (decoded, _, _) = from_enc.decode(&bytes);
    
    let to_enc = get_encoding(&to_encoding);
    let (encoded, _, _) = to_enc.encode(&decoded);
    
    fs::write(&output_path, encoded.as_ref())
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(output_path)
}

#[tauri::command]
pub async fn detect_file_encoding(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok("UTF-8-BOM".to_string());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok("UTF-16-LE".to_string());
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok("UTF-16-BE".to_string());
    }
    
    let (decoded_utf8, _, had_errors) = UTF_8.decode(&bytes);
    if !had_errors && decoded_utf8.chars().all(|c| !c.is_control() || c == '\n' || c == '\r' || c == '\t') {
        return Ok("UTF-8".to_string());
    }
    
    let (_, _, had_errors_gbk) = GBK.decode(&bytes);
    if !had_errors_gbk {
        return Ok("GBK".to_string());
    }
    
    Ok("UTF-8".to_string())
}

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

// ============ 批量内容替换（正则 + 自动备份，写回保留原编码） ============

/// 文本编码种类（替换后写回时保留原编码与 BOM）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,    // 无 BOM
    Utf8Bom, // EF BB BF
    Utf16Le, // FF FE
    Utf16Be, // FE FF
    Gbk,     // 无 BOM 且非 UTF-8
}

/// 读取文件并检测编码（BOM 优先；无 BOM 先严格 UTF-8，失败回退 GBK）
pub fn read_file_detected(path: &Path) -> Result<(String, TextEncoding), String> {
    let bytes = fs::read(path).map_err(|e| format!("读取失败: {}", e))?;
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok((
            String::from_utf8_lossy(&bytes[3..]).into_owned(),
            TextEncoding::Utf8Bom,
        ));
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return Ok((String::from_utf16_lossy(&utf16), TextEncoding::Utf16Le));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return Ok((String::from_utf16_lossy(&utf16), TextEncoding::Utf16Be));
    }
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok((s.to_string(), TextEncoding::Utf8)),
        Err(_) => {
            let (decoded, _, _) = GBK.decode(&bytes);
            Ok((decoded.into_owned(), TextEncoding::Gbk))
        }
    }
}

/// 按原编码写回（保留 BOM / UTF-16 字节序）
fn encode_with_encoding(text: &str, enc: TextEncoding) -> Vec<u8> {
    match enc {
        TextEncoding::Utf8 => text.as_bytes().to_vec(),
        TextEncoding::Utf8Bom => {
            let mut v = vec![0xEF, 0xBB, 0xBF];
            v.extend_from_slice(text.as_bytes());
            v
        }
        TextEncoding::Utf16Le => {
            let mut v = vec![0xFF, 0xFE];
            for u in text.encode_utf16() {
                v.extend_from_slice(&u.to_le_bytes());
            }
            v
        }
        TextEncoding::Utf16Be => {
            let mut v = vec![0xFE, 0xFF];
            for u in text.encode_utf16() {
                v.extend_from_slice(&u.to_be_bytes());
            }
            v
        }
        TextEncoding::Gbk => GBK.encode(text).0.into_owned(),
    }
}

/// 单个文件的替换结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchReplaceFileResult {
    pub path: String,
    pub success: bool,
    pub match_count: u32,
    pub backup_path: Option<String>,
    pub error: Option<String>,
}

/// 备份文件到 backup_dir 下，保持相对 root_dir 的目录结构
/// ponytail: Windows 大小写规范化可能使 strip_prefix 失败，降级为平铺文件名
/// （backup_dir 带时间戳，同名覆盖风险可忽略）
fn create_backup(path: &str, root_dir: &str, backup_dir: &str) -> Result<String, String> {
    let file_abs = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("文件无效: {}", e))?;
    let root = Path::new(root_dir)
        .canonicalize()
        .map_err(|e| format!("搜索根目录无效: {}", e))?;
    let dest = match file_abs.strip_prefix(&root) {
        Ok(rel) => Path::new(backup_dir).join(rel),
        Err(_) => {
            let name = Path::new(path)
                .file_name()
                .ok_or_else(|| "文件名无效".to_string())?;
            Path::new(backup_dir).join(name)
        }
    };
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建备份目录失败: {}", e))?;
    }
    fs::copy(path, &dest).map_err(|e| format!("复制备份失败: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

/// 同步批量替换核心逻辑（命令与测试共用）
/// 字面量替换时自动转义正则元字符；不区分大小写注入 (?i)
pub fn batch_replace_files(
    paths: Vec<String>,
    root_dir: &str,
    search: &str,
    replacement: &str,
    use_regex: bool,
    case_sensitive: bool,
    backup_dir: &str,
) -> Vec<BatchReplaceFileResult> {
    let pattern = if use_regex {
        search.to_string()
    } else {
        regex::escape(search)
    };
    let pattern = if case_sensitive {
        pattern
    } else {
        format!("(?i){}", pattern)
    };
    let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            // 正则无效：全部失败
            return paths
                .into_iter()
                .map(|path| BatchReplaceFileResult {
                    path,
                    success: false,
                    match_count: 0,
                    backup_path: None,
                    error: Some(format!("正则表达式无效: {}", e)),
                })
                .collect();
        }
    };

    let mut results = Vec::with_capacity(paths.len());
    for path in paths {
        // 读取 + 编码检测
        let (content, enc) = match read_file_detected(Path::new(&path)) {
            Ok(v) => v,
            Err(e) => {
                results.push(BatchReplaceFileResult {
                    path: path.clone(),
                    success: false,
                    match_count: 0,
                    backup_path: None,
                    error: Some(e),
                });
                continue;
            }
        };

        // 替换 + 计数
        let match_count = re.find_iter(&content).count() as u32;
        if match_count == 0 {
            results.push(BatchReplaceFileResult {
                path,
                success: true,
                match_count: 0,
                backup_path: None,
                error: None,
            });
            continue;
        }
        let replaced = re.replace_all(&content, replacement).to_string();

        // 先备份原文件，再写回
        let backup_path = match create_backup(&path, root_dir, backup_dir) {
            Ok(bp) => Some(bp),
            Err(e) => {
                results.push(BatchReplaceFileResult {
                    path: path.clone(),
                    success: false,
                    match_count,
                    backup_path: None,
                    error: Some(format!("备份失败: {}", e)),
                });
                continue;
            }
        };

        let encoded = encode_with_encoding(&replaced, enc);
        if let Err(e) = fs::write(&path, &encoded) {
            results.push(BatchReplaceFileResult {
                path,
                success: false,
                match_count,
                backup_path,
                error: Some(format!("写入失败: {}", e)),
            });
            continue;
        }

        results.push(BatchReplaceFileResult {
            path,
            success: true,
            match_count,
            backup_path,
            error: None,
        });
    }
    results
}

/// 批量内容替换命令：paths 为命中文件列表，root_dir 为搜索根目录（备份按相对路径归档）
#[tauri::command(rename_all = "snake_case")]
pub async fn batch_replace_execute(
    paths: Vec<String>,
    root_dir: String,
    search: String,
    replacement: String,
    use_regex: bool,
    case_sensitive: bool,
    backup_dir: String,
) -> Result<Vec<BatchReplaceFileResult>, String> {
    debug_log!(
        "batch_replace: files={} root={} regex={} case={} backup={}",
        paths.len(),
        root_dir,
        use_regex,
        case_sensitive,
        backup_dir
    );
    Ok(batch_replace_files(
        paths,
        &root_dir,
        &search,
        &replacement,
        use_regex,
        case_sensitive,
        &backup_dir,
    ))
}

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

    #[test]
    fn encode_roundtrip_preserves_encoding() {
        let dir = tempfile::TempDir::new().unwrap();
        // 单一临时文件，顺序覆盖写入（TempDir 存活到断言结束）
        let p = dir.path().join("t.txt");
        let write = |bytes: &[u8]| {
            std::fs::write(&p, bytes).unwrap();
            p.clone()
        };

        // GBK 往返
        let gbk_enc = encode_with_encoding("中文内容", TextEncoding::Gbk);
        let (decoded, enc) = read_file_detected(&write(&gbk_enc)).unwrap();
        assert_eq!(decoded, "中文内容");
        assert_eq!(enc, TextEncoding::Gbk);

        // UTF-8 BOM 往返
        let bom_bytes = encode_with_encoding("hi", TextEncoding::Utf8Bom);
        assert!(bom_bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let (decoded2, enc2) = read_file_detected(&write(&bom_bytes)).unwrap();
        assert_eq!(decoded2, "hi");
        assert_eq!(enc2, TextEncoding::Utf8Bom);

        // UTF-16 LE 往返
        let le_bytes = encode_with_encoding("你好", TextEncoding::Utf16Le);
        assert!(le_bytes.starts_with(&[0xFF, 0xFE]));
        let (decoded3, enc3) = read_file_detected(&write(&le_bytes)).unwrap();
        assert_eq!(decoded3, "你好");
        assert_eq!(enc3, TextEncoding::Utf16Le);
    }

    #[test]
    fn batch_replace_literal_and_regex_with_backup() {
        let root = tempfile::TempDir::new().unwrap();
        // UTF-8 文件（字面量替换）
        let f1 = root.path().join("a.txt");
        std::fs::write(&f1, "hello world\nhello lito\n").unwrap();
        // GBK 文件（正则替换，中文）
        let f2 = root.path().join("b.txt");
        std::fs::write(&f2, [0xC4, 0xE3, 0xBA, 0xC3, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x0A]).unwrap(); // "你好 world\n"

        let backup_dir = root.path().join(".litobox_backup").join("20260811");

        // 字面量替换
        let results = batch_replace_files(
            vec![f1.to_string_lossy().to_string()],
            &root.path().to_string_lossy(),
            "hello",
            "hi",
            false,
            true,
            &backup_dir.to_string_lossy(),
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].match_count, 2);
        let new_content = std::fs::read_to_string(&f1).unwrap();
        assert_eq!(new_content, "hi world\nhi lito\n", "字面量应全部替换");
        // 备份存在且内容为原文
        let backup_file = backup_dir.join("a.txt");
        assert!(backup_file.exists(), "备份文件应存在");
        assert_eq!(
            std::fs::read_to_string(backup_file).unwrap(),
            "hello world\nhello lito\n",
            "备份应为替换前内容"
        );

        // 正则替换（GBK 文件，验证编码保留 + 正则）
        let results2 = batch_replace_files(
            vec![f2.to_string_lossy().to_string()],
            &root.path().to_string_lossy(),
            r"\bworld\b",
            "LitoBox",
            true,
            true,
            &backup_dir.to_string_lossy(),
        );
        assert_eq!(results2.len(), 1);
        assert!(results2[0].success);
        assert_eq!(results2[0].match_count, 1);
        // GBK 编码保留：新内容应仍以 GBK 编码（含"你好"原字节）
        let bytes = std::fs::read(&f2).unwrap();
        assert!(bytes.starts_with(&[0xC4, 0xE3, 0xBA, 0xC3]), "GBK 中文应保留");
        let (decoded, enc) = read_file_detected(&f2).unwrap();
        assert_eq!(enc, TextEncoding::Gbk);
        assert_eq!(decoded, "你好 LitoBox\n");
    }

    #[test]
    fn batch_replace_case_insensitive_and_no_match() {
        let root = tempfile::TempDir::new().unwrap();
        let f = root.path().join("a.txt");
        std::fs::write(&f, "Hello HELLO hello\n").unwrap();
        let backup_dir = root.path().join(".backup").join("ts");

        // 不区分大小写字面量
        let results = batch_replace_files(
            vec![f.to_string_lossy().to_string()],
            &root.path().to_string_lossy(),
            "hello",
            "X",
            false,
            false,
            &backup_dir.to_string_lossy(),
        );
        assert!(results[0].success);
        assert_eq!(results[0].match_count, 3);
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "X X X\n");

        // 无命中：不写备份不写文件
        let f2 = root.path().join("b.txt");
        std::fs::write(&f2, "nothing here\n").unwrap();
        let results2 = batch_replace_files(
            vec![f2.to_string_lossy().to_string()],
            &root.path().to_string_lossy(),
            "zzz",
            "yyy",
            false,
            true,
            &backup_dir.to_string_lossy(),
        );
        assert!(results2[0].success);
        assert_eq!(results2[0].match_count, 0);
        assert!(results2[0].backup_path.is_none(), "无命中不应创建备份");
        assert_eq!(std::fs::read_to_string(&f2).unwrap(), "nothing here\n");

        // 非法正则：整体失败
        let results3 = batch_replace_files(
            vec![f2.to_string_lossy().to_string()],
            &root.path().to_string_lossy(),
            "[",
            "x",
            true,
            true,
            &backup_dir.to_string_lossy(),
        );
        assert!(!results3[0].success);
        assert!(results3[0].error.as_ref().unwrap().contains("正则表达式无效"));
    }
}
