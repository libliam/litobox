use encoding_rs::{Encoding, UTF_8, GBK, GB18030, WINDOWS_1252};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

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
}
