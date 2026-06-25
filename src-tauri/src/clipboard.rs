use arboard::Clipboard;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter};

static MONITORING: AtomicBool = AtomicBool::new(true);
static LAST_TEXT: Mutex<String> = Mutex::new(String::new());

#[derive(Clone, Serialize)]
pub struct ClipboardEntry {
    pub text: String,
    pub timestamp: u64,
}

#[tauri::command]
pub fn start_clipboard_monitor(app: AppHandle) {
    MONITORING.store(true, Ordering::SeqCst);

    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };

        loop {
            if !MONITORING.load(Ordering::SeqCst) {
                thread::sleep(std::time::Duration::from_millis(500));
                continue;
            }

            let text = clipboard.get_text().unwrap_or_default();

            if !text.is_empty() {
                let last = LAST_TEXT.lock().unwrap();
                if text != *last {
                    drop(last);
                    *LAST_TEXT.lock().unwrap() = text.clone();

                    let entry = ClipboardEntry {
                        text: text.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    let _ = app.emit("clipboard://new-entry", entry);
                }
            }

            thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

#[tauri::command]
pub fn stop_clipboard_monitor() {
    MONITORING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn is_monitoring() -> bool {
    MONITORING.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}
