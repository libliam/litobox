// 窗口尺寸/位置/最大化状态持久化。
// ponytail: 不引入 tauri-plugin-window-state，手写 JSON 文件读写（零新增依赖）。
// 存储位置与 litobox.db 同目录（%APPDATA%\com.dev.toolbox\window_state.json）。
//
// 关键设计：恢复必须在 Rust setup 阶段（窗口首次显示前）完成，
// 不能等前端 onMounted —— 前端加载完再 setSize 会先显示默认尺寸再跳变（闪烁）。
use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1100,
            height: 720,
            x: 0,
            y: 0,
            maximized: false,
        }
    }
}

fn state_file() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("com.dev.toolbox").join("window_state.json"))
}

pub fn load() -> Option<WindowState> {
    let raw = std::fs::read_to_string(state_file()?).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save(state: &WindowState) {
    let Some(path) = state_file() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(raw) = serde_json::to_string(state) {
        let _ = std::fs::write(path, raw);
    }
}

/// 保存当前窗口状态。供前端 onResized/onMoved 调用，也供关闭确认时兜底调用。
#[tauri::command]
pub fn save_window_state(window: WebviewWindow) -> Result<(), String> {
    // 最大化：只记录 maximized，保留上次非最大化尺寸（最大化尺寸不可信）
    if window.is_maximized().unwrap_or(false) {
        let mut st = load().unwrap_or_default();
        st.maximized = true;
        save(&st);
        return Ok(());
    }
    // 最小化：不保存（避免最小化竞态期间读到垃圾值）
    if window.is_minimized().unwrap_or(false) {
        return Ok(());
    }
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let pos = window.inner_position().map_err(|e| e.to_string())?;
    // ponytail: 过滤最小化竞态垃圾值（Windows 最小化为 160×28、-32000,-32000）。
    // 直接丢弃异常值，保留上次正常值。
    if size.width < 400 || size.height < 300 || pos.x < -10000 || pos.y < -10000 {
        return Ok(());
    }
    save(&WindowState {
        width: size.width,
        height: size.height,
        x: pos.x,
        y: pos.y,
        maximized: false,
    });
    Ok(())
}

/// setup 阶段立即恢复窗口状态（窗口首次显示前执行，避免先默认尺寸再跳变）。
pub fn restore_window(window: &WebviewWindow) {
    let Some(st) = load() else { return };
    if st.maximized {
        let _ = window.maximize();
        return;
    }
    if st.width < 400 || st.height < 300 {
        return;
    }
    // 尺寸恢复与位置校验解耦：即使 available_monitors 异常也不阻断 set_size
    let _ = window.set_size(tauri::PhysicalSize::new(st.width, st.height));
    if st.x >= -10000 && st.y >= -10000 {
        let on_screen = window
            .available_monitors()
            .map(|ms| {
                ms.iter().any(|m| {
                    let mp = m.position();
                    let ms = m.size();
                    let overlap_x = (st.x + st.width as i32).min(mp.x + ms.width as i32)
                        - st.x.max(mp.x);
                    let overlap_y = (st.y + st.height as i32).min(mp.y + ms.height as i32)
                        - st.y.max(mp.y);
                    overlap_x >= 100 && overlap_y >= 100
                })
            })
            .unwrap_or(false);
        if on_screen {
            let _ = window.set_position(tauri::PhysicalPosition::new(st.x, st.y));
        }
    }
}

/// 前端 Vue 挂载完成后通知显示主窗口。
/// 窗口配置 visible:false 启动，就绪后再显示，彻底消除启动白屏与尺寸跳变。
#[tauri::command]
pub fn app_ready(window: WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}
