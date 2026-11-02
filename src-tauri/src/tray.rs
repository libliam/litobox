// 系统托盘：关闭按钮 / Alt+F4 隐藏到托盘，托盘左键切换显示，双击强制显示。
// 右键菜单：显示/隐藏窗口、分隔线、快捷唤回提示（读实际配置）、版本号、分隔线、退出。
// ponytail: 复用 tauri 自带 tray/menu 能力 + 打包已有的 32x32 PNG，零新增依赖。
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const TRAY_ID: &str = "main";
const MENU_TOGGLE: &str = "toggle";
const MENU_SHORTCUT_HINT: &str = "shortcut_hint";
const MENU_VERSION_HINT: &str = "version_hint";
const MENU_QUIT: &str = "quit";

/// 托盘是否创建成功。失败时关闭按钮回退为"确认退出"，避免窗口隐藏后无法唤回。
static TRAY_READY: AtomicBool = AtomicBool::new(false);
/// 用户是否主动隐藏到托盘。用于区分"启动时还没显示"和"用户已隐藏"，供 3 秒兜底 show 判断。
static HIDDEN_BY_USER: AtomicBool = AtomicBool::new(false);

pub fn is_ready() -> bool {
    TRAY_READY.load(Ordering::Relaxed)
}

pub fn has_hidden_by_user() -> bool {
    HIDDEN_BY_USER.load(Ordering::Relaxed)
}

/// 读取用户实际配置的全局唤回快捷键（命令面板 / 快速启动），未配置时给出提示文案。
/// ponytail: 每次重建菜单只查一次 SQLite 配置，菜单仅在显示/隐藏时重建，开销可忽略。
fn shortcut_hint_label() -> String {
    let shortcuts = crate::db::db_read_shortcuts();
    let normalize = |s: &str| s.replace("CmdOrCtrl", "Ctrl");
    let parts: Vec<String> = ["__palette__", "__quick_launch__"]
        .iter()
        .filter_map(|id| shortcuts.iter().find(|(k, _)| k == id).map(|(_, v)| normalize(v)))
        .collect();
    if parts.is_empty() {
        debug_log!("[tray] no global wake shortcut configured");
        "快捷唤回：未设置（可在「快捷键设置」中配置）".to_string()
    } else {
        format!("快捷唤回：{}", parts.join(" / "))
    }
}

/// 菜单文案随窗口可见状态变化，每次显示/隐藏后重建菜单（菜单项句柄不可跨线程共享存储）。
fn build_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let toggle_label = if visible { "隐藏窗口" } else { "显示窗口" };
    let toggle = MenuItem::with_id(app, MENU_TOGGLE, toggle_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep_top = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    // 提示类条目：禁用态，避免看起来能点却没反应
    let shortcut_hint = MenuItem::with_id(
        app,
        MENU_SHORTCUT_HINT,
        shortcut_hint_label(),
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let version_hint = MenuItem::with_id(
        app,
        MENU_VERSION_HINT,
        format!("版本 v{}", app.package_info().version),
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let sep_bottom = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    Menu::with_items(
        app,
        &[
            &toggle,
            &sep_top,
            &shortcut_hint,
            &version_hint,
            &sep_bottom,
            &quit,
        ],
    )
    .map_err(|e| e.to_string())
}

pub fn refresh_menu(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match build_menu(app) {
        Ok(menu) => {
            if let Err(e) = tray.set_menu(Some(menu)) {
                debug_log!("[tray] set_menu failed: {}", e);
            }
        }
        Err(e) => debug_log!("[tray] rebuild menu failed: {}", e),
    }
}

/// 创建托盘图标与菜单。返回 Err 时调用方应保留旧的关闭行为（退出确认）。
pub fn setup(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app)?;
    // 优先 32x32 PNG（避免 ico 在部分缩放下发虚），解码失败再退回打包默认图标
    let icon = match tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")) {
        Ok(icon) => icon,
        Err(e) => {
            debug_log!("[tray] 32x32 png decode failed: {}, fallback to default icon", e);
            app.default_window_icon()
                .cloned()
                .ok_or_else(|| "默认窗口图标不可用".to_string())?
        }
    };
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("栗的百宝箱")
        .menu(&menu)
        // 左键单击用于切换显示/隐藏，右键才弹菜单
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_TOGGLE => toggle_main_window(app),
            MENU_QUIT => {
                debug_log!("[tray] quit from menu");
                crate::shutdown_app(app);
            }
            other => debug_log!("[tray] unknown menu id: {}", other),
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => toggle_main_window(tray.app_handle()),
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_main_window(tray.app_handle()),
            _ => {}
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    TRAY_READY.store(true, Ordering::Relaxed);
    debug_log!("[tray] created");
    Ok(())
}

/// 显示并前置主窗口。热键唤起与托盘唤醒共用，隐藏（非最小化）时 IsIconic 为 0，走 SW_SHOW。
pub fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        debug_log!("[tray] main window not found when showing");
        return;
    };
    #[cfg(target_os = "windows")]
    if let Ok(hwnd) = window.hwnd() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
        };
        unsafe {
            // 仅最小化时 SW_RESTORE；最大化仅失焦时 restore 会把窗口缩小回默认尺寸。
            if IsIconic(hwnd.0) != 0 {
                let _ = ShowWindow(hwnd.0, SW_RESTORE);
            } else {
                let _ = ShowWindow(hwnd.0, SW_SHOW);
            }
            let _ = SetForegroundWindow(hwnd.0);
        }
    }
    // hide 之后必须显式 show 再置前，否则 hwnd 仍处于 SW_HIDE
    if let Err(e) = window.show() {
        debug_log!("[tray] show failed: {}", e);
    }
    let _ = window.set_focus();
    HIDDEN_BY_USER.store(false, Ordering::Relaxed);
    refresh_menu(app);
}

pub fn hide_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        debug_log!("[tray] main window not found when hiding");
        return;
    };
    if let Err(e) = window.hide() {
        debug_log!("[tray] hide failed: {}", e);
        return;
    }
    HIDDEN_BY_USER.store(true, Ordering::Relaxed);
    debug_log!("[tray] main window hidden");
    refresh_menu(app);
}

fn toggle_main_window(app: &AppHandle) {
    let visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    debug_log!("[tray] toggle, visible={}", visible);
    if visible {
        hide_main_window(app);
    } else {
        show_main_window(app);
    }
}

/// 首次隐藏到托盘时弹一次原生提示，并把标记写回 ToolboxConfig（与前端同一份 config JSON）。
pub fn maybe_show_hint(app: &AppHandle) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder, MessageDialogButtons, MessageDialogKind};

    let raw = crate::db::db_get_config("main".to_string()).unwrap_or_default();
    let mut cfg: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
    if cfg.get("trayHintShown").and_then(|v| v.as_bool()).unwrap_or(false) {
        return;
    }
    cfg["trayHintShown"] = serde_json::Value::Bool(true);
    if let Err(e) = crate::db::db_set_config("main".to_string(), cfg.to_string()) {
        debug_log!("[tray] persist trayHintShown failed: {}", e);
    }
    // 通知前端同步内存中的配置，避免下次 saveConfig 用旧值覆盖掉标记
    if let Err(e) = tauri::Emitter::emit(app, "tray-hint-shown", ()) {
        debug_log!("[tray] emit tray-hint-shown failed: {}", e);
    }
    // 窗口已隐藏，用 AppHandle 级对话框（无父窗口）确保提示能正常显示
    MessageDialogBuilder::new(
        app.dialog().clone(),
        "已隐藏到托盘",
        "栗的百宝箱已隐藏到系统托盘。\n双击托盘图标恢复窗口，右键菜单可退出应用。",
    )
    .buttons(MessageDialogButtons::Ok)
    .kind(MessageDialogKind::Info)
    .show(|_| {});
    debug_log!("[tray] first-hide hint shown");
}
