use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;

fn file_path_to_string(path: FilePath) -> String {
    match path {
        FilePath::Path(p) => p.to_string_lossy().to_string(),
        FilePath::Url(url) => url.to_string(),
    }
}

#[tauri::command]
pub fn save_file_with_dialog(
    app: AppHandle,
    data_base64: String,
    filename: String,
    default_ext: String,
) -> Result<String, String> {
    // 解码 base64 数据
    let bytes = STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    // 使用同步通道在回调中获取结果
    let (tx, rx) = sync_channel(0);

    app.dialog().file()
        .set_title("保存文件")
        .set_file_name(&filename)
        .add_filter("所有文件", &["*"])
        .add_filter(&format!("{} 文件", default_ext.to_uppercase()), &[&default_ext])
        .save_file(move |file_path: Option<FilePath>| {
            let result = match file_path {
                Some(path) => {
                    let path_buf: PathBuf = match path {
                        FilePath::Path(p) => p,
                        FilePath::Url(url) => {
                            match url.to_file_path() {
                                Ok(p) => p,
                                Err(_) => {
                                    let _ = tx.send(Err("无效的文件路径".to_string()));
                                    return;
                                }
                            }
                        }
                    };
                    match fs::write(&path_buf, &bytes) {
                        Ok(_) => Ok(file_path_to_string(FilePath::Path(path_buf))),
                        Err(e) => Err(format!("写入文件失败: {}", e)),
                    }
                }
                None => Ok("cancelled".to_string()),
            };
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("接收结果失败: {}", e))?
}

/// 通过对话框保存文本内容，并返回文件路径
#[tauri::command]
pub fn save_text_with_dialog(
    app: AppHandle,
    content: String,
    filename: String,
) -> Result<String, String> {
    let bytes = content.into_bytes();
    let (tx, rx) = sync_channel(0);

    app.dialog().file()
        .set_title("保存文件")
        .set_file_name(&filename)
        .add_filter("文本文件", &["txt"])
        .add_filter("所有文件", &["*"])
        .save_file(move |file_path: Option<FilePath>| {
            let result = match file_path {
                Some(path) => {
                    let mut path_buf: PathBuf = match path {
                        FilePath::Path(p) => p,
                        FilePath::Url(url) => {
                            match url.to_file_path() {
                                Ok(p) => p,
                                Err(_) => {
                                    let _ = tx.send(Err("无效的文件路径".to_string()));
                                    return;
                                }
                            }
                        }
                    };
                    // 如果没有扩展名，添加 .txt
                    if path_buf.extension().is_none() {
                        path_buf.set_extension("txt");
                    }
                    match fs::write(&path_buf, &bytes) {
                        Ok(_) => Ok(file_path_to_string(FilePath::Path(path_buf))),
                        Err(e) => Err(format!("写入文件失败: {}", e)),
                    }
                }
                None => Ok("cancelled".to_string()),
            };
            let _ = tx.send(result);
        });

    rx.recv().map_err(|e| format!("接收结果失败: {}", e))?
}
