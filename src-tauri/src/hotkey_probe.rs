//! 全局快捷键占用查看器 - 探测核心
//!
//! 通过 RegisterHotKey 试探 + 立即释放的方式探测热键占用情况：
//! - RegisterHotKey 成功 → 该热键未被占用（可注册）
//! - 失败 + GetLastError == ERROR_HOTKEY_ALREADY_REGISTERED (1409) → 已被占用
//! - 失败 + 其他错误码 → 系统保留
//!
//! 进程定位三级回退：
//! 1. 内置映射表（hotkey_data::lookup_maptable）
//! 2. 进程扫描（hotkey_data::scan_processes，匹配运行中进程）
//! 3. LitoBox 自身已注册热键（从 db::db_read_shortcuts 读取）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, RegisterClassW, WNDCLASSW,
    HWND_MESSAGE,
};
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey,
};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, CloseHandle};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
};

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 常量 ============

const PROGRESS_INTERVAL_MS: u64 = 200;
const PROBE_TIMEOUT_MS: u64 = 30_000;
const PROBE_HOTKEY_ID: i32 = 0xBEEF;  // 探测用的热键 ID（任意值，避免与 hotkey.rs 冲突）
const ERROR_HOTKEY_ALREADY_REGISTERED: u32 = 1409;

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,           // 进程名（如 "WeChat.exe"），映射表命中时为空
    pub display: String,        // 显示名（如 "微信截图"）
    pub pid: Option<u32>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HotkeyStatus {
    Available,
    Occupied,
    SystemReserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MatchSource {
    MapTable,
    ProcessScan,
    SelfRegistered,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyResult {
    pub label: String,
    pub mod_flags: u32,
    pub vk: u32,
    pub status: HotkeyStatus,
    pub process_name: Option<String>,
    pub process_display: Option<String>,
    pub process_pid: Option<u32>,
    pub process_path: Option<String>,
    pub source: MatchSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeProgress {
    pub probe_id: String,
    pub done: usize,
    pub total: usize,
    pub last_key: String,
    pub is_finished: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeStats {
    pub total: usize,
    pub available: usize,
    pub occupied: usize,
    pub reserved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCompletePayload {
    pub probe_id: String,
    pub results: Vec<HotkeyResult>,
    pub stats: ProbeStats,
    pub cancelled: bool,
}

// ============ 内部状态 ============

#[derive(Debug)]
pub struct ProbeState {
    pub cancel_flag: Arc<AtomicBool>,
    pub progress: Arc<Mutex<ProbeProgress>>,
    pub last_results: Arc<Mutex<Vec<HotkeyResult>>>,
    pub current_probe_id: Arc<Mutex<Option<String>>>,
}

impl Default for ProbeState {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(Mutex::new(ProbeProgress {
                probe_id: String::new(),
                done: 0,
                total: 0,
                last_key: String::new(),
                is_finished: false,
                error: None,
            })),
            last_results: Arc::new(Mutex::new(Vec::new())),
            current_probe_id: Arc::new(Mutex::new(None)),
        }
    }
}

// 全局状态：单例探测任务（同时只允许一个探测运行）
static PROBE_STATE: OnceLock<ProbeState> = OnceLock::new();

fn state() -> &'static ProbeState {
    PROBE_STATE.get_or_init(ProbeState::default)
}

// ============ 候选热键集生成 ============

/// 生成默认候选热键集
/// 返回 Vec<(mod_flags, vk, label)>
pub fn generate_default_candidates() -> Vec<(u32, u32, String)> {
    use crate::hotkey_data::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};

    let mut result: Vec<(u32, u32, String)> = Vec::with_capacity(260);

    // 辅助：生成 (mod, vk, label)
    let mods_label = |m: u32| -> String {
        let mut parts: Vec<&str> = Vec::new();
        if m & MOD_CONTROL != 0 { parts.push("Ctrl"); }
        if m & MOD_ALT != 0 { parts.push("Alt"); }
        if m & MOD_SHIFT != 0 { parts.push("Shift"); }
        if m & MOD_WIN != 0 { parts.push("Win"); }
        parts.join("+")
    };

    let add = |result: &mut Vec<_>, m: u32, vk: u32, key: &str| {
        let label = if mods_label(m).is_empty() {
            key.to_string()
        } else {
            format!("{}+{}", mods_label(m), key)
        };
        result.push((m, vk, label));
    };

    // Win + 0-9 / A-Z
    for c in b'0'..=b'9' {
        add(&mut result, MOD_WIN, c as u32, &(c as char).to_string());
    }
    for c in b'A'..=b'Z' {
        add(&mut result, MOD_WIN, c as u32, &(c as char).to_string());
    }
    // Ctrl+Shift + A-Z
    for c in b'A'..=b'Z' {
        add(&mut result, MOD_CONTROL | MOD_SHIFT, c as u32, &(c as char).to_string());
    }
    // Ctrl+Alt + A-Z / 0-9
    for c in b'A'..=b'Z' {
        add(&mut result, MOD_CONTROL | MOD_ALT, c as u32, &(c as char).to_string());
    }
    for c in b'0'..=b'9' {
        add(&mut result, MOD_CONTROL | MOD_ALT, c as u32, &(c as char).to_string());
    }
    // Alt+Shift + A-Z
    for c in b'A'..=b'Z' {
        add(&mut result, MOD_ALT | MOD_SHIFT, c as u32, &(c as char).to_string());
    }
    // Ctrl+Win + A-Z
    for c in b'A'..=b'Z' {
        add(&mut result, MOD_CONTROL | MOD_WIN, c as u32, &(c as char).to_string());
    }
    // F1-F12 × {Ctrl, Alt, Shift, Win}
    let f_mods = [MOD_CONTROL, MOD_ALT, MOD_SHIFT, MOD_WIN];
    let f_mod_names = ["Ctrl", "Alt", "Shift", "Win"];
    for (idx, &m) in f_mods.iter().enumerate() {
        for n in 1..=12u32 {
            let vk = 0x6F + n;  // VK_F1 = 0x70, VK_F12 = 0x7B
            result.push((m, vk, format!("{}+F{}", f_mod_names[idx], n)));
        }
    }

    result
}

/// 解析自定义热键字符串（如 "Ctrl+Shift+S"）为 (mod_flags, vk)
/// 解析失败返回 None，并打印 debug 日志
pub fn parse_accelerator(s: &str) -> Option<(u32, u32)> {
    use crate::hotkey_data::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};

    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut mod_flags = 0u32;
    let mut vk: Option<u32> = None;

    for (i, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        let is_last = i == parts.len() - 1;
        match lower.as_str() {
            "ctrl" | "control" => mod_flags |= MOD_CONTROL,
            // Tauri 加速器语法：CmdOrCtrl 在 Windows 上等价于 Ctrl
            "cmdorctrl" | "cmd" => mod_flags |= MOD_CONTROL,
            "alt" => mod_flags |= MOD_ALT,
            "shift" => mod_flags |= MOD_SHIFT,
            "win" | "super" | "meta" => mod_flags |= MOD_WIN,
            _ if is_last => {
                // 主键
                vk = parse_vk(part);
                if vk.is_none() {
                    debug_log!("[hotkey_probe] 无法解析主键: {}", part);
                    return None;
                }
            }
            _ => {
                debug_log!("[hotkey_probe] 未知修饰键: {}", part);
                return None;
            }
        }
    }

    vk.map(|v| (mod_flags, v))
}

/// 解析主键名为虚拟键码
fn parse_vk(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let lower = s.to_lowercase();
    // 单字符 0-9
    if let Some(c) = s.chars().next() {
        if s.len() == 1 && c.is_ascii_digit() {
            return Some(c as u32);
        }
        // 单字符 A-Z（不区分大小写）
        if s.len() == 1 && c.is_ascii_alphabetic() {
            return Some(c.to_ascii_uppercase() as u32);
        }
    }
    // F1-F12
    if let Some(rest) = lower.strip_prefix('f') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=12).contains(&n) {
                return Some(0x6F + n);  // VK_F1 = 0x70
            }
        }
    }
    None
}

// ============ Windows API 封装 ============

#[cfg(windows)]
fn create_message_window() -> Result<HWND, String> {
    use windows_sys::Win32::Foundation::HINSTANCE;
    use windows_sys::Win32::UI::WindowsAndMessaging::HMENU;
    // ponytail: windows-sys 0.59 中 WNDCLASS_STYLES/WINDOW_EX_STYLE/WINDOW_STYLE 都是 u32 的类型别名（非 newtype），
    // 直接用字面量 0 即可；HINSTANCE/HICON/HCURSOR/HBRUSH 都是 *mut c_void，用 null_mut 构造空指针。
    unsafe {
        let class_name: Vec<u16> = "LitoboxHotkeyProbe\0".encode_utf16().collect();
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(def_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: std::ptr::null_mut(),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        let atom = RegisterClassW(&wc);
        if atom == 0 {
            let err = GetLastError();
            // 类已存在不算错误（ERROR_CLASS_ALREADY_EXISTS = 1410）
            if err != 1410 {
                debug_log!("[hotkey_probe] RegisterClassW failed: err={}", err);
            }
        }

        // ponytail: WINDOW_EX_STYLE/WINDOW_STYLE 是 u32 类型别名，直接传 0；
        // HWND_MESSAGE 用于创建 message-only 窗口（不显示、不接收输入）
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            0, 0, 0, 0,
            HWND_MESSAGE,
            0 as HMENU,
            0 as HINSTANCE,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return Err(format!("CreateWindowExW failed: err={}", GetLastError()));
        }
        Ok(hwnd)
    }
}

#[cfg(windows)]
unsafe extern "system" fn def_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    // ponytail: 我们不处理任何消息（register 后立即 unregister，不会触发 WM_HOTKEY）
    // 直接调用 DefWindowProcW 即可
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(windows)]
fn register_hotkey_probe(hwnd: HWND, mod_flags: u32, vk: u32) -> bool {
    unsafe {
        // MOD_NOREPEAT (0x4000) 避免重复触发，但本工具不实际响应热键，可省略
        RegisterHotKey(hwnd, PROBE_HOTKEY_ID, mod_flags, vk) != 0
    }
}

#[cfg(windows)]
fn unregister_hotkey_probe(hwnd: HWND) {
    unsafe {
        UnregisterHotKey(hwnd, PROBE_HOTKEY_ID);
    }
}

#[cfg(not(windows))]
fn create_message_window() -> Result<u64, String> {
    Err("hotkey probe only supported on Windows".to_string())
}

#[cfg(not(windows))]
fn register_hotkey_probe(_hwnd: u64, _mod_flags: u32, _vk: u32) -> bool {
    false
}

#[cfg(not(windows))]
fn unregister_hotkey_probe(_hwnd: u64) {}

// ============ 进程枚举缓存 ============

/// 一次性枚举所有进程，返回进程名 → (pid, path) 映射
/// 路径获取失败时 path = None（权限不足等）
#[cfg(windows)]
pub fn enumerate_processes_once() -> HashMap<String, (u32, Option<String>)> {
    let mut result: HashMap<String, (u32, Option<String>)> = HashMap::new();

    // ponytail: windows-sys 0.59 中 HANDLE 是 *mut c_void，与 0 比较 用 is_null()；
    // std::ptr::invalid_mut 在 stable Rust 上不稳定，改用 windows-sys 提供的 INVALID_HANDLE_VALUE 常量
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x2 /* TH32CS_SNAPPROCESS */, 0);
        if snapshot.is_null() || snapshot == INVALID_HANDLE_VALUE {
            debug_log!("[hotkey_probe] CreateToolhelp32Snapshot failed: err={}", GetLastError());
            return result;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile
                );
                // 截断到第一个 null
                let name = name.split('\0').next().unwrap_or("").to_string();
                if !name.is_empty() {
                    let pid = entry.th32ProcessID;
                    let path = query_process_path(pid);
                    result.entry(name).or_insert((pid, path));
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    debug_log!("[hotkey_probe] enumerated {} processes", result.len());
    result
}

#[cfg(windows)]
fn query_process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        let _ = CloseHandle(handle);
        if ok != 0 {
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            // 去掉 Windows 长路径前缀 \\?\ （AGENTS 经验 13）
            Some(path.strip_prefix(r"\\?\").unwrap_or(&path).to_string())
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn enumerate_processes_once() -> HashMap<String, (u32, Option<String>)> {
    HashMap::new()
}

// ============ LitoBox 自身热键查询 ============

/// 从 db 读取 LitoBox 已注册的快捷键配置
/// 返回 (mod_flags, vk) 列表
pub fn query_self_registered_hotkeys() -> Vec<(u32, u32)> {
    let shortcuts = crate::db::db_read_shortcuts();
    shortcuts.iter()
        .filter_map(|(_tool_id, shortcut_str)| {
            parse_accelerator(shortcut_str).map(|(m, v)| {
                debug_log!("[hotkey_probe] self shortcut: {} -> mod={:#x} vk={}", shortcut_str, m, v);
                (m, v)
            })
        })
        .collect()
}

// ============ 探测循环 ============

/// 执行探测（在后台线程中调用）
#[cfg(windows)]
fn run_probe(
    app: AppHandle,
    probe_id: String,
    candidates: Vec<(u32, u32, String)>,
    extra_keys: Vec<String>,
) {
    use crate::hotkey_data::{lookup_maptable, lookup_system_reserved, scan_processes};

    let state_ref = state();
    let cancel_flag = state_ref.cancel_flag.clone();
    let progress_arc = state_ref.progress.clone();
    let results_arc = state_ref.last_results.clone();

    // 重置 cancel flag
    cancel_flag.store(false, Ordering::Release);

    // 初始化进度
    {
        let mut p = progress_arc.lock().unwrap();
        *p = ProbeProgress {
            probe_id: probe_id.clone(),
            done: 0,
            total: candidates.len(),
            last_key: "初始化中...".to_string(),
            is_finished: false,
            error: None,
        };
    }
    // 清空上次结果
    results_arc.lock().unwrap().clear();

    // 1. 创建隐藏窗口
    let hwnd = match create_message_window() {
        Ok(h) => h,
        Err(e) => {
            let mut p = progress_arc.lock().unwrap();
            p.is_finished = true;
            p.error = Some(format!("创建窗口失败: {}", e));
            let _ = app.emit("hotkey-probe-complete", ProbeCompletePayload {
                probe_id: probe_id.clone(),
                results: vec![],
                stats: ProbeStats { total: 0, available: 0, occupied: 0, reserved: 0 },
                cancelled: false,
            });
            return;
        }
    };

    // 2. 查询 LitoBox 自身已注册热键
    let self_keys = query_self_registered_hotkeys();
    debug_log!("[hotkey_probe] self_keys count={}", self_keys.len());

    // 3. 一次性枚举进程（只针对映射表中出现的进程名）
    let process_map = enumerate_processes_once();

    // 4. 时间驱动探测循环
    let started = Instant::now();
    let mut last_check = Instant::now();
    let mut last_progress_emit = Instant::now();
    let mut results: Vec<HotkeyResult> = Vec::with_capacity(candidates.len());
    let mut cancelled = false;

    for (i, (mod_flags, vk, label)) in candidates.iter().enumerate() {
        // 时间驱动取消检查
        if last_check.elapsed() >= Duration::from_millis(PROGRESS_INTERVAL_MS) {
            last_check = Instant::now();
            if cancel_flag.load(Ordering::Acquire) {
                debug_log!("[hotkey_probe] cancelled at {}/{}", i, candidates.len());
                cancelled = true;
                break;
            }
            // 超时检查
            if started.elapsed() > Duration::from_millis(PROBE_TIMEOUT_MS) {
                debug_log!("[hotkey_probe] timeout at {}/{}", i, candidates.len());
                let mut p = progress_arc.lock().unwrap();
                p.error = Some(format!("探测超时（{}秒）", PROBE_TIMEOUT_MS / 1000));
                break;
            }
        }

        // 探测单个热键
        let result = probe_one(hwnd, *mod_flags, *vk, label, &self_keys, &process_map);
        results.push(result);

        // 更新进度（每 200ms emit 一次，避免事件风暴）
        if last_progress_emit.elapsed() >= Duration::from_millis(PROGRESS_INTERVAL_MS) {
            last_progress_emit = Instant::now();
            let done = i + 1;
            {
                let mut p = progress_arc.lock().unwrap();
                p.done = done;
                p.last_key = label.clone();
            }
            let _ = app.emit("hotkey-probe-progress", ProbeProgress {
                probe_id: probe_id.clone(),
                done,
                total: candidates.len(),
                last_key: label.clone(),
                is_finished: false,
                error: None,
            });
        }
    }

    // 5. 完成：统计 + emit
    let stats = compute_stats(&results);
    {
        let mut p = progress_arc.lock().unwrap();
        p.done = results.len();
        p.total = candidates.len();
        p.last_key = if cancelled { "已取消".to_string() } else { "完成".to_string() };
        p.is_finished = true;
    }
    *results_arc.lock().unwrap() = results.clone();

    debug_log!(
        "[hotkey_probe] done: total={} avail={} occupied={} reserved={} cancelled={}",
        stats.total, stats.available, stats.occupied, stats.reserved, cancelled
    );

    let _ = app.emit("hotkey-probe-complete", ProbeCompletePayload {
        probe_id: probe_id.clone(),
        results,
        stats: stats.clone(),
        cancelled,
    });

    // 清理 probe_id 标记
    *state_ref.current_probe_id.lock().unwrap() = None;
}

#[cfg(windows)]
fn probe_one(
    hwnd: HWND,
    mod_flags: u32,
    vk: u32,
    label: &str,
    self_keys: &[(u32, u32)],
    process_map: &HashMap<String, (u32, Option<String>)>,
) -> HotkeyResult {
    use crate::hotkey_data;

    // 1. 先检查是否 LitoBox 自身注册
    if self_keys.iter().any(|(m, v)| *m == mod_flags && *v == vk) {
        let self_pid = std::process::id();
        // ponytail: PathBuf 没有 Deref<Target=str>，unwrap_or(&p) 类型不匹配；
        // 改用 trim_start_matches 直接去掉 \\?\ 前缀
        let self_path = std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().trim_start_matches(r"\\?\").to_string());
        return HotkeyResult {
            label: label.to_string(),
            mod_flags,
            vk,
            status: HotkeyStatus::Occupied,
            process_name: Some("LitoBox".to_string()),
            process_display: Some("LitoBox".to_string()),
            process_pid: Some(self_pid),
            process_path: self_path,
            source: MatchSource::SelfRegistered,
        };
    }

    // 2. 尝试注册
    let ok = register_hotkey_probe(hwnd, mod_flags, vk);
    if ok {
        unregister_hotkey_probe(hwnd);
        return HotkeyResult {
            label: label.to_string(),
            mod_flags,
            vk,
            status: HotkeyStatus::Available,
            process_name: None,
            process_display: None,
            process_pid: None,
            process_path: None,
            source: MatchSource::None,
        };
    }

    let err = unsafe { GetLastError() };
    debug_log!("[hotkey_probe] {} failed: err={}", label, err);

    // 3. 三级回退定位进程
    let (process_info, source) = if err == ERROR_HOTKEY_ALREADY_REGISTERED {
        // 已被占用
        // 3.1 映射表查询
        if let Some(p) = hotkey_data::lookup_maptable(mod_flags, vk) {
            (Some(p), MatchSource::MapTable)
        }
        // 3.2 进程扫描
        else if let Some(p) = hotkey_data::scan_processes(mod_flags, vk, |name| {
            process_map.get(name).cloned()
        }) {
            (Some(p), MatchSource::ProcessScan)
        }
        else {
            (None, MatchSource::None)
        }
    } else {
        // 系统保留
        if let Some(p) = hotkey_data::lookup_system_reserved(mod_flags, vk) {
            (Some(p), MatchSource::MapTable)
        } else {
            (None, MatchSource::None)
        }
    };

    let status = if err == ERROR_HOTKEY_ALREADY_REGISTERED {
        HotkeyStatus::Occupied
    } else {
        HotkeyStatus::SystemReserved
    };

    HotkeyResult {
        label: label.to_string(),
        mod_flags,
        vk,
        status,
        process_name: process_info.as_ref().map(|p| p.name.clone()),
        process_display: process_info.as_ref().map(|p| p.display.clone()),
        process_pid: process_info.as_ref().and_then(|p| p.pid),
        process_path: process_info.as_ref().and_then(|p| p.path.clone()),
        source,
    }
}

#[cfg(not(windows))]
fn run_probe(
    app: AppHandle,
    probe_id: String,
    _candidates: Vec<(u32, u32, String)>,
    _extra_keys: Vec<String>,
) {
    let state_ref = state();
    let mut p = state_ref.progress.lock().unwrap();
    p.is_finished = true;
    p.error = Some("仅支持 Windows 平台".to_string());
    let _ = app.emit("hotkey-probe-complete", ProbeCompletePayload {
        probe_id,
        results: vec![],
        stats: ProbeStats { total: 0, available: 0, occupied: 0, reserved: 0 },
        cancelled: false,
    });
}

fn compute_stats(results: &[HotkeyResult]) -> ProbeStats {
    let mut available = 0;
    let mut occupied = 0;
    let mut reserved = 0;
    for r in results {
        match r.status {
            HotkeyStatus::Available => available += 1,
            HotkeyStatus::Occupied => occupied += 1,
            HotkeyStatus::SystemReserved => reserved += 1,
        }
    }
    ProbeStats {
        total: results.len(),
        available,
        occupied,
        reserved,
    }
}

// ============ CSV 导出 ============

/// 导出结果为 CSV（UTF-8 BOM，Excel 友好）
pub fn export_csv(results: &[HotkeyResult]) -> Result<String, String> {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let filename = format!("hotkey_probe_{}.csv", chrono_like_timestamp());
    let csv_path = temp_dir.join(filename);

    let mut content = String::from("\u{FEFF}");  // UTF-8 BOM
    content.push_str("热键组合,状态,占用进程,进程PID,进程路径,来源\n");

    for r in results {
        let status_cn = match r.status {
            HotkeyStatus::Available => "可注册",
            HotkeyStatus::Occupied => "被占用",
            HotkeyStatus::SystemReserved => "系统保留",
        };
        let source_cn = match r.source {
            MatchSource::MapTable => "映射表",
            MatchSource::ProcessScan => "进程扫描",
            MatchSource::SelfRegistered => "自身注册",
            MatchSource::None => "-",
        };
        // CSV 转义：包含逗号/引号/换行的字段用双引号包裹
        let escape = |s: &str| -> String {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        content.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape(&r.label),
            status_cn,
            escape(r.process_display.as_deref().unwrap_or("-")),
            r.process_pid.map(|p| p.to_string()).unwrap_or_default(),
            escape(r.process_path.as_deref().unwrap_or("-")),
            source_cn,
        ));
    }

    let mut file = std::fs::File::create(&csv_path)
        .map_err(|e| format!("创建 CSV 文件失败: {}", e))?;
    // 写入 GBK 兼容？保持 UTF-8 即可（含 BOM）
    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入 CSV 失败: {}", e))?;

    let path_str = csv_path.to_string_lossy()
        .strip_prefix(r"\\?\")
        .unwrap_or(&csv_path.to_string_lossy())
        .to_string();
    Ok(path_str)
}

/// 生成类 ISO 时间戳（避免引入 chrono 依赖）
fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn hotkey_probe_start(
    app: AppHandle,
    extra_keys: Vec<String>,
) -> Result<String, String> {
    let state_ref = state();

    // 检查是否已有探测在运行
    {
        let current = state_ref.current_probe_id.lock().unwrap();
        if current.is_some() {
            return Err("已有探测任务正在运行".to_string());
        }
    }

    let probe_id = uuid::Uuid::new_v4().to_string();
    *state_ref.current_probe_id.lock().unwrap() = Some(probe_id.clone());

    // 生成候选集：默认 + 自定义
    let mut candidates = generate_default_candidates();
    for key_str in &extra_keys {
        if let Some((m, v)) = parse_accelerator(key_str) {
            // 去重
            if !candidates.iter().any(|(em, ev, _)| *em == m && *ev == v) {
                candidates.push((m, v, key_str.clone()));
            }
        } else {
            debug_log!("[hotkey_probe] 无法解析自定义热键: {}", key_str);
        }
    }

    debug_log!(
        "[hotkey_probe] start id={} candidates={} extras={:?}",
        probe_id, candidates.len(), extra_keys
    );

    let app_clone = app.clone();
    let probe_id_clone = probe_id.clone();
    let extra_keys_clone = extra_keys.clone();
    std::thread::spawn(move || {
        run_probe(app_clone, probe_id_clone, candidates, extra_keys_clone);
    });

    Ok(probe_id)
}

#[tauri::command]
pub async fn hotkey_probe_cancel() -> Result<(), String> {
    let state_ref = state();
    state_ref.cancel_flag.store(true, Ordering::Release);
    debug_log!("[hotkey_probe] cancel requested");
    Ok(())
}

#[tauri::command]
pub async fn hotkey_probe_status() -> Result<ProbeProgress, String> {
    let state_ref = state();
    let p = state_ref.progress.lock().unwrap();
    Ok(p.clone())
}

#[tauri::command]
pub async fn hotkey_probe_get_results() -> Result<Vec<HotkeyResult>, String> {
    let state_ref = state();
    let r = state_ref.last_results.lock().unwrap();
    Ok(r.clone())
}

#[tauri::command]
pub async fn hotkey_probe_export_csv(results: Vec<HotkeyResult>) -> Result<String, String> {
    export_csv(&results)
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_accelerator_ctrl_shift_s() {
        let (m, v) = parse_accelerator("Ctrl+Shift+S").unwrap();
        assert_eq!(m, MOD_CONTROL | MOD_SHIFT);
        assert_eq!(v, 0x53);  // 'S'
    }

    #[test]
    fn test_parse_accelerator_win_a() {
        let (m, v) = parse_accelerator("Win+A").unwrap();
        assert_eq!(m, MOD_WIN);
        assert_eq!(v, 0x41);
    }

    #[test]
    fn test_parse_accelerator_case_insensitive() {
        let (m1, v1) = parse_accelerator("ctrl+shift+s").unwrap();
        let (m2, v2) = parse_accelerator("CTRL+SHIFT+S").unwrap();
        assert_eq!(m1, m2);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_parse_accelerator_f5() {
        let (m, v) = parse_accelerator("F5").unwrap();
        assert_eq!(m, 0);  // 无修饰键
        assert_eq!(v, 0x74);  // VK_F5
    }

    #[test]
    fn test_parse_accelerator_ctrl_f12() {
        let (m, v) = parse_accelerator("Ctrl+F12").unwrap();
        assert_eq!(m, MOD_CONTROL);
        assert_eq!(v, 0x7B);  // VK_F12
    }

    #[test]
    fn test_parse_accelerator_invalid() {
        assert!(parse_accelerator("").is_none());
        assert!(parse_accelerator("Ctrl+").is_none());
        assert!(parse_accelerator("Ctrl+Foo").is_none());
    }

    #[test]
    fn test_generate_default_candidates_count() {
        let candidates = generate_default_candidates();
        // 36 + 26 + 36 + 26 + 26 + 48 = 198
        assert!(candidates.len() >= 190, "expected ~200 candidates, got {}", candidates.len());
    }

    #[test]
    fn test_generate_default_candidates_no_duplicates() {
        let candidates = generate_default_candidates();
        let mut seen = std::collections::HashSet::new();
        for (m, v, _) in &candidates {
            let key = (*m, *v);
            assert!(seen.insert(key), "duplicate candidate: mod={:#x} vk={}", m, v);
        }
    }

    #[test]
    fn test_compute_stats() {
        let results = vec![
            HotkeyResult {
                label: "A".into(), mod_flags: 0, vk: 0,
                status: HotkeyStatus::Available,
                process_name: None, process_display: None,
                process_pid: None, process_path: None,
                source: MatchSource::None,
            },
            HotkeyResult {
                label: "B".into(), mod_flags: 0, vk: 0,
                status: HotkeyStatus::Occupied,
                process_name: None, process_display: None,
                process_pid: None, process_path: None,
                source: MatchSource::None,
            },
            HotkeyResult {
                label: "C".into(), mod_flags: 0, vk: 0,
                status: HotkeyStatus::SystemReserved,
                process_name: None, process_display: None,
                process_pid: None, process_path: None,
                source: MatchSource::None,
            },
        ];
        let stats = compute_stats(&results);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.available, 1);
        assert_eq!(stats.occupied, 1);
        assert_eq!(stats.reserved, 1);
    }

    #[test]
    fn test_export_csv_writes_file() {
        let results = vec![
            HotkeyResult {
                label: "Win+L".into(), mod_flags: MOD_WIN, vk: 0x4C,
                status: HotkeyStatus::SystemReserved,
                process_name: None, process_display: Some("系统锁屏".into()),
                process_pid: None, process_path: None,
                source: MatchSource::MapTable,
            },
        ];
        let path = export_csv(&results).unwrap();
        assert!(path.ends_with(".csv"));
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with('\u{FEFF}'));
        assert!(content.contains("Win+L"));
        assert!(content.contains("系统锁屏"));
        // 清理
        let _ = std::fs::remove_file(&path);
    }
}

// 使用 hotkey_data 中定义的常量，避免在测试中重复定义
use crate::hotkey_data::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};
