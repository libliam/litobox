use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chardetng::EncodingDetector;

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteFileContent {
    pub content: String,
    pub encoding: String,
    pub size: usize,
}

pub fn note_read(file_path: &str) -> Result<NoteFileContent, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let bytes = fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let size = bytes.len();

    // 检测编码
    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let encoding_name = encoding.name();

    // 解码
    let (content, _, _) = encoding.decode(&bytes);

    Ok(NoteFileContent {
        content: content.into_owned(),
        encoding: encoding_name.to_string(),
        size,
    })
}

pub fn note_write(file_path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))
}

#[tauri::command]
pub fn cmd_note_read(file_path: String) -> Result<NoteFileContent, String> {
    note_read(&file_path)
}

#[tauri::command]
pub fn cmd_note_write(file_path: String, content: String) -> Result<(), String> {
    note_write(&file_path, &content)
}
