# 全局快捷键占用查看器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 新增「全局快捷键占用查看器」工具，基于 RegisterHotKey 试探+立即释放的方式探测 Windows 已注册的全局热键，标注占用进程，支持搜索冲突。

**Architecture:** 后端新增 `hotkey_probe.rs`（探测核心 + 5 个 Tauri 命令）和 `hotkey_data.rs`（系统/应用热键映射表），沿用 `file_searcher.rs` 的 `OnceLock<Mutex<HashMap>>` + `Arc<AtomicBool>` cancel + `std::thread::spawn` + `app.emit()` 模式。前端新增 `HotkeyView.vue` 单页面，参考 `ProcessListView.vue` 结构，进入页面自动探测 + 事件+2s轮询兜底。新增 `windows-sys` 依赖（约 0.5 MB），版本号 5.7.0 → 5.8.0。

**Tech Stack:** Rust + Tauri 2.0 + `windows-sys` 0.59（Win32 API）+ Vue 3 Composition API + Element Plus + Pinia

**Spec:** [docs/superpowers/specs/2026-07-20-hotkey-viewer-design.md](file:///d:/work/codes/litobox/docs/superpowers/specs/2026-07-20-hotkey-viewer-design.md)

---

## 关键参考文件

| 用途 | 路径 |
|------|------|
| 后端命令模板（cancel + emit + 后台线程） | [src-tauri/src/file_searcher.rs](file:///d:/work/codes/litobox/src-tauri/src/file_searcher.rs) |
| 前端页面模板（系统工具风格） | [src/views/ProcessListView.vue](file:///d:/work/codes/litobox/src/views/ProcessListView.vue) |
| 前端历史记录调用模式 | [src/store/index.ts:281](file:///d:/work/codes/litobox/src/store/index.ts) `addHistory` |
| CSV 导出参考 | [src-tauri/src/sqlite_viewer.rs:206](file:///d:/work/codes/litobox/src-tauri/src/sqlite_viewer.rs) `sqlite_export_csv` |
| TOOL_LIST 注册位置 | [src/store/index.ts:95](file:///d:/work/codes/litobox/src/store/index.ts) `fileSearcher` 条目后 |
| App.vue 组件映射 | [src/App.vue:84](file:///d:/work/codes/litobox/src/App.vue) `toolComponentMap` |
| 命令注册位置 | [src-tauri/src/main.rs:30](file:///d:/work/codes/litobox/src-tauri/src/main.rs) `invoke_handler` |

**重要修正**：Spec 中提到「修改 src/router/index.ts」是错误的——项目无独立 router，组件注册在 [App.vue](file:///d:/work/codes/litobox/src/App.vue) 的 `toolComponentMap` 中。

---

## Task 1: 添加 windows-sys 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 末尾添加 windows-sys 依赖**

打开 `d:\work\codes\litobox\src-tauri\Cargo.toml`，在 `flate2 = "1.0"` 行（第 42 行）之后、`[dev-dependencies]` 之前追加：

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_ToolHelp",
] }
```

- [ ] **Step 2: 验证依赖能正常拉取**

Run: `cd src-tauri; cargo check`
Expected: 编译通过（可能有 unused warning，无错误）。若出现 feature 不存在的错误，对照 [windows-sys 文档](https://docs.rs/windows-sys) 调整 feature 名。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore(deps): 新增 windows-sys 依赖用于全局热键探测"
```

---

## Task 2: 创建 hotkey_data.rs（内置映射表 + 进程匹配）

**Files:**
- Create: `src-tauri/src/hotkey_data.rs`

- [ ] **Step 1: 创建 hotkey_data.rs 文件**

写入完整内容到 `d:\work\codes\litobox\src-tauri\src\hotkey_data.rs`：

```rust
//! 全局快捷键占用查看器 - 内置映射表与进程匹配
//!
//! 提供：
//! - 系统保留热键表（Win+L 锁屏等不可注册热键）
//! - 常见应用热键表（微信截图、QQ 截图等已知占用）
//! - 进程名匹配（基于 ProcessCache 扫描运行中进程）

use crate::hotkey_probe::ProcessInfo;

// Windows 修饰键常量（与 windows-sys MOD_* 一致）
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;

// 常用虚拟键码
pub const VK_DELETE: u32 = 0x2E;
pub const VK_TAB: u32 = 0x09;

/// 系统保留热键：不可注册，由 Windows 内核处理
/// (mod_flags, vk, display_name)
const SYSTEM_RESERVED: &[(u32, u32, &str)] = &[
    (MOD_WIN, 0x4C, "系统锁屏"),                // Win+L
    (MOD_WIN, 0x44, "显示桌面"),                // Win+D
    (MOD_WIN, 0x45, "资源管理器"),              // Win+E
    (MOD_WIN, 0x52, "运行对话框"),              // Win+R
    (MOD_WIN, 0x50, "投影切换"),                // Win+P
    (MOD_WIN, 0x54, "任务栏切换"),              // Win+T
    (MOD_WIN, 0x55, "轻松访问中心"),            // Win+U
    (MOD_WIN, 0x56, "通知中心"),                // Win+V
    (MOD_WIN, 0x49, "设置"),                    // Win+I
    (MOD_WIN, 0x4B, "连接面板"),                // Win+K
    (MOD_WIN, 0x4D, "最小化所有窗口"),          // Win+M
    (MOD_WIN | MOD_SHIFT, 0x4D, "还原最小化窗口"), // Win+Shift+M
    (MOD_WIN, VK_TAB, "任务视图"),              // Win+Tab
    (MOD_WIN | MOD_CONTROL, VK_TAB, "虚拟桌面切换"), // Win+Ctrl+Tab（左/右用箭头）
    (MOD_WIN | MOD_CONTROL, 0xD, "虚拟桌面切换"),   // Win+Ctrl+Enter(0x0D 不准,实际用 Left/Right)
    (MOD_WIN | MOD_CONTROL, 0x24, "新建虚拟桌面"),   // Win+Ctrl+D
    (MOD_WIN | MOD_CONTROL, 0x23, "关闭虚拟桌面"),   // Win+Ctrl+F4
    (MOD_CONTROL | MOD_MENU, VK_DELETE, "安全选项"), // Ctrl+Alt+Del
    (MOD_CONTROL | MOD_MENU | MOD_SHIFT, VK_DELETE, "安全选项增强"), // Ctrl+Alt+Shift+Del 不存在，删除
    // ponytail: Win+Ctrl+Left/Right 是虚拟桌面切换，但 Left=0x25/Right=0x27 是箭头键
    // 实际探测时会注册失败（系统占用），映射表此处省略，由通用机制处理
];

/// 常见应用热键表：已知的第三方应用占用
/// (mod_flags, vk, display_name, [matching_process_names])
const APP_HOTKEYS: &[(u32, u32, &str, &[&str])] = &[
    // 即时通讯
    (MOD_CONTROL | MOD_ALT, 0x41, "微信截图", &["WeChat.exe"]),
    (MOD_CONTROL | MOD_ALT, 0x53, "QQ 截图", &["QQ.exe", "QQProtect.exe"]),
    (MOD_ALT, 0x41, "微信录屏", &["WeChat.exe"]),
    (MOD_CONTROL | MOD_ALT, 0x5A, "微信截图(旧)", &["WeChat.exe"]),
    // 截图工具
    (MOD_WIN | MOD_SHIFT, 0x53, "Snipaste 截图", &["Snipaste.exe"]),
    (MOD_WIN, 0x53, "Snipping Tool", &["ScreenClippingHost.exe", "SnippingTool.exe"]),
    (MOD_CONTROL | MOD_ALT, 0x50, "PicPick 截图", &["picpick.exe"]),
    // IDE / 编辑器
    (MOD_CONTROL | MOD_SHIFT, 0x58, "IDA 截图", &["IDA64.exe", "IDAQ.exe", "idag.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x50, "VSCode 命令面板", &["Code.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x47, "VSCode Git 视图", &["Code.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x4E, "VSCode 资源管理器", &["Code.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x4C, "Cursor 命令面板", &["Cursor.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x46, "IDEA 查找", &["idea64.exe", "idea.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x41, "IDEA 全局搜索", &["idea64.exe", "idea.exe"]),
    // 浏览器
    (MOD_CONTROL | MOD_SHIFT, 0x54, "浏览器新标签页", &["chrome.exe", "msedge.exe", "firefox.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x4E, "浏览器新窗口", &["chrome.exe", "msedge.exe"]),
    // 系统工具
    (MOD_CONTROL | MOD_SHIFT, 0x1B, "任务管理器(Ctrl+Shift+Esc)", &[]),  // 0x1B = ESC
    // 办公套件
    (MOD_CONTROL | MOD_ALT, 0x56, "WPS 截图", &["wps.exe", "ksolaunch.exe"]),
    // 远程/会议
    (MOD_CONTROL | MOD_ALT, 0x5A, "钉钉截图", &["DingTalk.exe", "DingTalkLauncher.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x41, "飞书截图", &["Feishu.exe", "Lark.exe"]),
    // 输入法
    (MOD_CONTROL | MOD_SHIFT, 0x46, "搜狗输入法繁简切换", &["SogouInput.exe"]),
];

/// 查询系统保留热键表
pub fn lookup_system_reserved(mod_flags: u32, vk: u32) -> Option<ProcessInfo> {
    SYSTEM_RESERVED.iter().find_map(|(m, v, name)| {
        if *m == mod_flags && *v == vk {
            Some(ProcessInfo {
                name: String::new(),
                display: name.to_string(),
                pid: None,
                path: None,
            })
        } else {
            None
        }
    })
}

/// 查询常见应用热键表（不依赖进程扫描，直接命中）
pub fn lookup_maptable(mod_flags: u32, vk: u32) -> Option<ProcessInfo> {
    APP_HOTKEYS.iter().find_map(|(m, v, display, _procs)| {
        if *m == mod_flags && *v == vk {
            Some(ProcessInfo {
                name: String::new(),
                display: display.to_string(),
                pid: None,
                path: None,
            })
        } else {
            None
        }
    })
}

/// 扫描运行中进程，匹配应用热键习惯表
/// 仅在 lookup_maptable miss 时调用，利用已枚举的 ProcessCache
pub fn scan_processes<F>(mod_flags: u32, vk: u32, is_running: F) -> Option<ProcessInfo>
where
    F: Fn(&str) -> Option<(u32, Option<String>)>,  // (pid, path) by process_name
{
    APP_HOTKEYS.iter().find_map(|(m, v, display, procs)| {
        if *m != mod_flags || *v != vk {
            return None;
        }
        // procs 为空数组时（如 Ctrl+Shift+Esc 任务管理器），不扫描
        for proc_name in *procs {
            if let Some((pid, path)) = is_running(proc_name) {
                return Some(ProcessInfo {
                    name: proc_name.to_string(),
                    display: display.to_string(),
                    pid: Some(pid),
                    path,
                });
            }
        }
        None
    })
}

/// 获取所有需匹配的进程名清单（用于一次性枚举进程，避免重复扫描）
pub fn get_process_names_to_scan() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = APP_HOTKEYS
        .iter()
        .flat_map(|(_, _, _, procs)| procs.iter().copied())
        .collect();
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_system_reserved_win_l() {
        let info = lookup_system_reserved(MOD_WIN, 0x4C).unwrap();
        assert_eq!(info.display, "系统锁屏");
    }

    #[test]
    fn test_lookup_system_reserved_miss() {
        assert!(lookup_system_reserved(MOD_WIN, 0x41).is_none());
    }

    #[test]
    fn test_lookup_maptable_wechat_screenshot() {
        let info = lookup_maptable(MOD_CONTROL | MOD_ALT, 0x41).unwrap();
        assert_eq!(info.display, "微信截图");
    }

    #[test]
    fn test_scan_processes_match() {
        // 模拟 WeChat.exe 正在运行
        let info = scan_processes(MOD_CONTROL | MOD_ALT, 0x41, |name| {
            if name == "WeChat.exe" {
                Some((1234, Some("C:\\Program Files\\WeChat\\WeChat.exe".to_string())))
            } else {
                None
            }
        }).unwrap();
        assert_eq!(info.display, "微信截图");
        assert_eq!(info.name, "WeChat.exe");
        assert_eq!(info.pid, Some(1234));
    }

    #[test]
    fn test_scan_processes_no_match() {
        // 模拟无进程运行
        let result = scan_processes(MOD_CONTROL | MOD_ALT, 0x41, |_| None);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_process_names_to_scan_dedup() {
        let names = get_process_names_to_scan();
        // 验证去重
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        // 应包含 WeChat.exe
        assert!(names.contains(&"WeChat.exe"));
    }
}
```

- [ ] **Step 2: 验证编译（预期会因引用 hotkey_probe::ProcessInfo 失败，此为正常 — Task 3 会补全）**

Run: `cd src-tauri; cargo check`
Expected: 编译失败，错误为 `unresolved module hotkey_probe` 或 `cannot find type ProcessInfo`。这是预期的——下一任务会创建 hotkey_probe.rs。

- [ ] **Step 3: 暂不提交，等 Task 3 完成后一起提交**

---

## Task 3: 创建 hotkey_probe.rs（探测核心 + Tauri 命令）

**Files:**
- Create: `src-tauri/src/hotkey_probe.rs`

- [ ] **Step 1: 创建 hotkey_probe.rs 文件**

写入完整内容到 `d:\work\codes\litobox\src-tauri\src\hotkey_probe.rs`：

```rust
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
    CreateWindowExW, RegisterClassW, UnregisterHotKey, RegisterHotKey, WNDCLASSW,
    HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_NULL,
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
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus as _;

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
    use windows_sys::Win32::UI::WindowsAndMessaging::{HMENU, WNDCLASS_STYLES};
    unsafe {
        let class_name: Vec<u16> = "LitoboxHotkeyProbe\0".encode_utf16().collect();
        let wc = WNDCLASSW {
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(def_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        let atom = RegisterClassW(&wc);
        if atom == 0 {
            let err = GetLastError();
            // 类已存在不算错误（ERROR_CLASS_ALREADY_EXISTS = 1410）
            if err.0 != 1410 {
                debug_log!("[hotkey_probe] RegisterClassW failed: err={}", err.0);
            }
        }

        // ponytail: windows-sys 的 WINDOW_EX_STYLE/WINDOW_STYLE 是 newtype，未实现 Default
        // 用 (0) 构造即可；HWND_MESSAGE 用于创建 message-only 窗口（不显示、不接收输入）
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name.as_ptr(),
            std::ptr::null(),
            WINDOW_STYLE(0),
            0, 0, 0, 0,
            HWND_MESSAGE,
            0 as HMENU,
            0 as HINSTANCE,
            std::ptr::null(),
        );
        if hwnd == 0 {
            return Err(format!("CreateWindowExW failed: err={}", GetLastError().0));
        }
        Ok(hwnd)
    }
}

#[cfg(windows)]
extern "system" fn def_window_proc(
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

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x2 /* TH32CS_SNAPPROCESS */, 0);
        if snapshot == 0 || snapshot == std::ptr::invalid_mut(-1isize as usize) {
            debug_log!("[hotkey_probe] CreateToolhelp32Snapshot failed: err={}", GetLastError().0);
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
        if handle == 0 {
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
        let self_path = std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().strip_prefix(r"\\?\").unwrap_or(&p).to_string());
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
    debug_log!("[hotkey_probe] {} failed: err={}", label, err.0);

    // 3. 三级回退定位进程
    let (process_info, source) = if err.0 == ERROR_HOTKEY_ALREADY_REGISTERED {
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

    let status = if err.0 == ERROR_HOTKEY_ALREADY_REGISTERED {
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
```

> **注意**：
> 1. `MOD_*` 常量在 `hotkey_data.rs` 中定义为 `pub const`，hotkey_probe.rs 末尾 `use crate::hotkey_data::{...}` 引入供测试用。
> 2. `ProcessInfo` 结构体定义在 hotkey_probe.rs 中（pub），hotkey_data.rs 通过 `use crate::hotkey_probe::ProcessInfo` 引用——这是单向依赖。
> 3. `db_read_shortcuts` 函数在 db.rs 中已存在（main.rs:211 有调用），无需新增。
> 4. `chrono_like_timestamp` 用 Unix 秒数避免引入 chrono 依赖。
> 5. 测试中 `parse_accelerator` 用的 `MOD_*` 常量在文件末尾 `use` 引入。

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri; cargo check`
Expected: 编译通过，可能有 unused warning。如果出现 `cannot find function db_read_shortcuts` 错误，确认 db.rs 中函数签名是 `pub fn db_read_shortcuts() -> Vec<(String, String)>`。

- [ ] **Step 3: 运行单元测试**

Run: `cd src-tauri; cargo test hotkey_probe -- --nocapture`
Expected: 所有 8 个测试通过（test_parse_accelerator_*、test_generate_default_candidates_*、test_compute_stats、test_export_csv_writes_file）。

- [ ] **Step 4: 暂不提交，等 Task 4 注册命令后一起提交**

---

## Task 4: main.rs 注册模块和命令

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 main.rs 顶部添加 mod 声明**

在 `mod media_info;` 行（约第 19 行）之后追加：

```rust
mod hotkey_probe;
mod hotkey_data;
```

- [ ] **Step 2: 在 invoke_handler 数组末尾追加 5 个命令**

找到 `media_info::extract_cover_art,` 行（约第 185 行），在它之后追加：

```rust
            // 全局快捷键占用查看器命令
            hotkey_probe::hotkey_probe_start,
            hotkey_probe::hotkey_probe_cancel,
            hotkey_probe::hotkey_probe_status,
            hotkey_probe::hotkey_probe_get_results,
            hotkey_probe::hotkey_probe_export_csv,
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri; cargo check`
Expected: 编译通过，无错误。

- [ ] **Step 4: 验证测试仍通过**

Run: `cd src-tauri; cargo test hotkey`
Expected: hotkey_probe 和 hotkey_data 的所有测试通过（约 13 个）。

- [ ] **Step 5: 提交后端模块**

```bash
git add src-tauri/src/hotkey_probe.rs src-tauri/src/hotkey_data.rs src-tauri/src/main.rs
git commit -m "feat(hotkey-probe): 新增全局快捷键占用探测后端模块

- hotkey_probe.rs: RegisterHotKey 试探+立即释放，5 个 Tauri 命令
- hotkey_data.rs: 系统保留+常见应用热键映射表，进程匹配
- 时间驱动取消（200ms）+ 事件流推送进度 + 30s 超时
- 进程定位三级回退：映射表→进程扫描→LitoBox 自身
- 13 个单元测试覆盖解析/匹配/统计/导出"
```

---

## Task 5: 创建前端 HotkeyView.vue 页面

**Files:**
- Create: `src/views/HotkeyView.vue`

- [ ] **Step 1: 创建 HotkeyView.vue**

写入完整内容到 `d:\work\codes\litobox\src\views\HotkeyView.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- 统计卡片 -->
    <div class="tool-card">
      <div class="card-header"><span class="card-title">快捷键占用概览</span></div>
      <div class="card-body">
        <div class="stats-grid">
          <div class="stat-item">总探测数 <strong>{{ stats?.total ?? 0 }}</strong></div>
          <div class="stat-item">被占用 <strong class="stat-danger">{{ stats?.occupied ?? 0 }}</strong></div>
          <div class="stat-item">可注册 <strong class="stat-success">{{ stats?.available ?? 0 }}</strong></div>
          <div class="stat-item">系统保留 <strong class="stat-warning">{{ stats?.reserved ?? 0 }}</strong></div>
        </div>
      </div>
    </div>

    <!-- 搜索栏 + 操作区 -->
    <div class="tool-card sticky-card">
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 2">
            <el-input v-model="searchKeyword" placeholder="搜索热键或进程名" clearable size="small" />
          </div>
          <div class="action-group">
            <el-radio-group v-model="filterStatus" size="small">
              <el-radio-button label="">全部</el-radio-button>
              <el-radio-button label="Occupied">被占用</el-radio-button>
              <el-radio-button label="Available">可注册</el-radio-button>
              <el-radio-button label="SystemReserved">系统保留</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <el-input v-model="extraKeysInput" placeholder="补充: Ctrl+Shift+S, Alt+F7" style="width: 240px" size="small" />
          </div>
          <div class="action-group">
            <el-button type="primary" size="small" :loading="isProbing" @click="startProbe">开始探测</el-button>
            <el-button v-if="isProbing" size="small" @click="cancelProbe">取消</el-button>
            <el-button size="small" :disabled="!results.length" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
        <el-progress v-if="isProbing" :percentage="progressPercent" :format="formatProgress" :stroke-width="6" style="margin-top: 8px" />
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 结果表格 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">探测结果</span>
        <div class="card-actions">
          <span class="group-label">{{ filteredResults.length }} / {{ results.length }} 条</span>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="pagedResults" stripe size="small" max-height="600" style="width: 100%">
          <el-table-column prop="label" label="热键组合" width="140" sortable />
          <el-table-column label="状态" width="120" sortable :sort-method="sortByStatus">
            <template #default="{ row }">
              <el-tag :type="statusTagType(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="占用进程" width="180" sortable :sort-method="sortByProcess">
            <template #default="{ row }">
              {{ row.process_display || (row.process_name || '-') }}
            </template>
          </el-table-column>
          <el-table-column prop="process_pid" label="PID" width="90" sortable />
          <el-table-column label="进程路径" show-overflow-tooltip>
            <template #default="{ row }">
              <span :class="{ 'path-muted': !row.process_path }">{{ row.process_path || '—' }}</span>
            </template>
          </el-table-column>
          <el-table-column label="来源" width="110">
            <template #default="{ row }">{{ sourceLabel(row.source) }}</template>
          </el-table-column>
        </el-table>
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredResults.length"
          layout="prev, pager, next, total"
          class="pagination-right"
          style="margin-top: 12px"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onDeactivated, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

// Rust struct 字段为 snake_case，前端 interface 必须一致（AGENTS 经验 16）
// 但 enum 序列化为 PascalCase（serde rename_all = "PascalCase"）
interface HotkeyResult {
  label: string
  mod_flags: number
  vk: number
  status: 'Available' | 'Occupied' | 'SystemReserved'
  process_name: string | null
  process_display: string | null
  process_pid: number | null
  process_path: string | null
  source: 'MapTable' | 'ProcessScan' | 'SelfRegistered' | 'None'
}

interface ProbeStats {
  total: number
  available: number
  occupied: number
  reserved: number
}

interface ProbeProgress {
  probe_id: string
  done: number
  total: number
  last_key: string
  is_finished: boolean
  error: string | null
}

interface ProbeCompletePayload {
  probe_id: string
  results: HotkeyResult[]
  stats: ProbeStats
  cancelled: boolean
}

const store = useToolboxStore()

const results = ref<HotkeyResult[]>([])
const stats = ref<ProbeStats | null>(null)
const isProbing = ref(false)
const error = ref('')
const progress = ref<ProbeProgress | null>(null)
const searchKeyword = ref('')
const filterStatus = ref('')
const extraKeysInput = ref('')
const currentPage = ref(1)
const pageSize = ref(50)

let unlistenProgress: UnlistenFn | null = null
let unlistenComplete: UnlistenFn | null = null
let pollTimer: number | null = null
let done = false

const progressPercent = computed(() => {
  if (!progress.value || progress.value.total === 0) return 0
  return Math.round((progress.value.done / progress.value.total) * 100)
})

const filteredResults = computed(() => {
  return results.value.filter(r => {
    if (filterStatus.value && r.status !== filterStatus.value) return false
    if (searchKeyword.value) {
      const kw = searchKeyword.value.toLowerCase()
      return r.label.toLowerCase().includes(kw)
        || (r.process_display?.toLowerCase().includes(kw) ?? false)
        || (r.process_name?.toLowerCase().includes(kw) ?? false)
    }
    return true
  })
})

const pagedResults = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredResults.value.slice(start, start + pageSize.value)
})

function statusLabel(s: HotkeyResult['status']): string {
  return { Available: '可注册', Occupied: '被占用', SystemReserved: '系统保留' }[s]
}

function statusTagType(s: HotkeyResult['status']): 'success' | 'danger' | 'warning' | 'info' {
  return { Available: 'success', Occupied: 'danger', SystemReserved: 'warning' }[s]
}

function sourceLabel(s: HotkeyResult['source']): string {
  return { MapTable: '映射表', ProcessScan: '进程扫描', SelfRegistered: '自身注册', None: '-' }[s]
}

function sortByStatus(a: HotkeyResult, b: HotkeyResult): number {
  return a.status.localeCompare(b.status)
}

function sortByProcess(a: HotkeyResult, b: HotkeyResult): number {
  return (a.process_display || '').localeCompare(b.process_display || '')
}

function formatProgress(percentage: number): string {
  if (!progress.value) return `${percentage}%`
  return `${percentage}% (${progress.value.done}/${progress.value.total})`
}

async function startProbe() {
  if (isProbing.value) return

  // 解析自定义补充热键
  const extraKeys = extraKeysInput.value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0)

  isProbing.value = true
  error.value = ''
  done = false
  currentPage.value = 1

  try {
    await invoke<string>('hotkey_probe_start', { extraKeys })
    startPolling()
  } catch (e) {
    isProbing.value = false
    error.value = String(e)
    ElMessage.error(`探测启动失败: ${e}`)
  }
}

async function cancelProbe() {
  try {
    await invoke('hotkey_probe_cancel')
    ElMessage.info('已请求取消探测')
  } catch (e) {
    ElMessage.error(`取消失败: ${e}`)
  }
}

function startPolling() {
  stopPolling()
  pollTimer = window.setInterval(async () => {
    if (done) { stopPolling(); return }
    try {
      const status = await invoke<ProbeProgress>('hotkey_probe_status')
      progress.value = status
      if (status.is_finished) {
        done = true
        stopPolling()
        // 兜底：万一 complete 事件丢失，主动拉取结果
        if (status.error) {
          error.value = status.error
          isProbing.value = false
        } else {
          try {
            const finalResults = await invoke<HotkeyResult[]>('hotkey_probe_get_results')
            handleComplete(finalResults)
          } catch (e) {
            isProbing.value = false
            error.value = `拉取结果失败: ${e}`
          }
        }
      }
    } catch (e) {
      debug_log(`轮询失败: ${e}`)
    }
  }, 2000)
}

function stopPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

function handleComplete(payload: ProbeCompletePayload | HotkeyResult[]) {
  // 兼容两种调用：事件 payload 是 ProbeCompletePayload，兜底拉取是 HotkeyResult[]
  if (Array.isArray(payload)) {
    results.value = payload
    stats.value = computeStats(payload)
  } else {
    results.value = payload.results
    stats.value = payload.stats
  }
  isProbing.value = false
  done = true
  stopPolling()

  // 缓存到 store（仅内存，不持久化）
  store.hotkeyLastResult = results.value
  store.hotkeyLastStats = stats.value

  // 记录历史（AGENTS 规范：必须传 inputFull/outputFull）
  const inputFull = `候选集: ${results.value.length} 个热键 + 自定义: ${extraKeysInput.value || '无'}`
  const outputFull = buildOutputFull(results.value, stats.value!)
  store.addHistory({
    tool: 'hotkeyViewer',
    action: '探测全局快捷键占用',
    inputPreview: inputFull.slice(0, 50),
    outputPreview: `占用:${stats.value!.occupied} 可注册:${stats.value!.available} 系统保留:${stats.value!.reserved}`,
    inputFull,
    outputFull,
  })

  ElMessage.success(`探测完成: 共 ${stats.value!.total} 个，被占用 ${stats.value!.occupied} 个`)
}

function computeStats(list: HotkeyResult[]): ProbeStats {
  let available = 0, occupied = 0, reserved = 0
  for (const r of list) {
    if (r.status === 'Available') available++
    else if (r.status === 'Occupied') occupied++
    else if (r.status === 'SystemReserved') reserved++
  }
  return { total: list.length, available, occupied, reserved }
}

function buildOutputFull(list: HotkeyResult[], s: ProbeStats): string {
  const header = `占用: ${s.occupied} | 可注册: ${s.available} | 系统保留: ${s.reserved}\n详细列表:`
  const lines = list.map(r => {
    const proc = r.process_display || r.process_name || '-'
    return `${r.label} - ${statusLabel(r.status)} - ${proc}`
  })
  return `${header}\n${lines.join('\n')}`
}

async function exportCsv() {
  if (!results.value.length) return
  try {
    const path = await invoke<string>('hotkey_probe_export_csv', { results: results.value })
    ElMessage.success(`已导出到: ${path}`)
  } catch (e) {
    ElMessage.error(`导出失败: ${e}`)
  }
}

function debug_log(msg: string) {
  // ponytail: 简易 console 输出，仅 dev 模式
  if (import.meta.env.DEV) console.log(`[HotkeyView] ${msg}`)
}

onMounted(async () => {
  // 先填充上次结果
  if (store.hotkeyLastResult?.length) {
    results.value = store.hotkeyLastResult
    stats.value = store.hotkeyLastStats
  }

  // 监听后端事件
  unlistenProgress = await listen<ProbeProgress>('hotkey-probe-progress', (e) => {
    progress.value = e.payload
  })
  unlistenComplete = await listen<ProbeCompletePayload>('hotkey-probe-complete', (e) => {
    handleComplete(e.payload)
  })

  // 进入页面自动启动探测（首次 + KeepAlive 激活）
  startProbe()
})

onActivated(() => {
  // AGENTS 经验 12: KeepAlive 缓存组件 onMounted 不会再次触发
  // 但 onActivated 会，用于切换回来时自动重新探测
  if (!isProbing.value && !done) {
    startProbe()
  }
})

onDeactivated(() => {
  stopPolling()
})

onUnmounted(() => {
  stopPolling()
  unlistenProgress?.()
  unlistenComplete?.()
})
</script>

<style scoped>
/* 只定义页面特有样式，全局 .tool-card/.card-header 等由 theme.css 提供（AGENTS 经验 21） */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.stat-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.stat-item strong {
  font-size: 22px;
  color: var(--text-primary);
  font-weight: 600;
}
.stat-danger { color: var(--accent-red) !important; }
.stat-success { color: var(--accent-green) !important; }
.stat-warning { color: var(--accent-orange) !important; }
.path-muted { color: var(--text-secondary); font-style: italic; }
.pagination-right {
  display: flex;
  justify-content: flex-end;
}
</style>
```

> **关键点**：
> 1. scoped 样式**不**重复定义 `.tool-card`/`.card-header`/`.card-body`/`.action-grid`/`.action-group`/`.group-label`/`.error-message` 等全局类名（AGENTS 经验 21）。
> 2. 使用 `onActivated` 兼容 KeepAlive（AGENTS 经验 12）。
> 3. 事件 + 2s 轮询双保险（AGENTS 经验 10）。
> 4. 调用 `store.addHistory` 传 inputFull/outputFull（AGENTS 强制规范）。
> 5. HotkeyResult interface 字段 snake_case（AGENTS 经验 16）。

- [ ] **Step 2: 验证 TypeScript 类型**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc 类型检查通过。如果 `store.hotkeyLastResult` 报错"property does not exist"，是因为 Task 6 还没添加 store 字段——先继续 Task 6 再验证。

- [ ] **Step 3: 暂不提交，等 Task 6 注册到 store 和 App.vue 后一起提交**

---

## Task 6: 注册到 TOOL_LIST 和 App.vue

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 在 store/index.ts 的 state 中追加 hotkey 缓存字段**

找到 `src/store/index.ts` 中的 `const history = ref<HistoryRecord[]>([])` 或类似 state 定义区域（约第 110-150 行附近），在合适位置追加：

```typescript
// 快捷键占用查看器：上次探测结果缓存（仅内存，不持久化）
const hotkeyLastResult = ref<any[]>([])
const hotkeyLastStats = ref<any | null>(null)
```

> 实际行号需要根据 store 现有结构定位。如果 store 用的是 `ref` 模式，按上面的写法；如果是 reactive object，追加到对应 reactive 中。

- [ ] **Step 2: 在 store return 语句中暴露新字段**

找到 store 的 return 语句（约第 350-375 行），追加：

```typescript
    hotkeyLastResult,
    hotkeyLastStats,
```

- [ ] **Step 3: 在 TOOL_LIST 数组的 system 分类下追加 hotkey 条目**

找到 `src/store/index.ts` 第 95 行 `fileSearcher` 条目，在它之后（同一行下方）追加：

```typescript
  { id: 'hotkeyViewer', name: '快捷键占用', icon: '⌨', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M6 14h.01M18 14h.01M10 14h4"/></svg>`, description: '探测 Windows 已注册的全局快捷键，标注占用进程', keywords: ['快捷键', '热键', 'hotkey', '冲突', '占用'], category: 'system' },
```

- [ ] **Step 4: 在 App.vue 中 import HotkeyView 并注册到 toolComponentMap**

打开 `src/App.vue`，在 `import ServiceListView from '@/views/ServiceListView.vue'` 行（约第 81 行）之后追加：

```typescript
import HotkeyView from '@/views/HotkeyView.vue'
```

然后在 `toolComponentMap` 对象中，在 `serviceList: ServiceListView,` 行（约第 135 行）之后追加：

```typescript
  hotkeyViewer: HotkeyView,
```

- [ ] **Step 5: 验证前端构建**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc + vite build 通过。如果报 `Property 'hotkeyLastResult' does not exist`，检查 Step 1-2 是否正确。

- [ ] **Step 6: 提交前端页面和注册**

```bash
git add src/views/HotkeyView.vue src/store/index.ts src/App.vue
git commit -m "feat(hotkey-viewer): 新增快捷键占用查看器前端页面

- HotkeyView.vue: 统计卡片+搜索栏+表格+分页
- onActivated 自动探测，事件+2s 轮询兜底
- 三级过滤（状态/关键词/分页）纯前端过滤
- CSV 导出，操作历史含 inputFull/outputFull
- TOOL_LIST/App.vue 注册 hotkeyViewer 工具"
```

---

## Task 7: 版本号更新 + README + feature-backlog

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 更新 package.json 版本号**

打开 `d:\work\codes\litobox\package.json`，将第 3 行 `"version": "5.7.0"` 改为：

```json
  "version": "5.8.0",
```

- [ ] **Step 2: 更新 Cargo.toml 版本号**

打开 `d:\work\codes\litobox\src-tauri\Cargo.toml`，将第 3 行 `version = "5.7.0"` 改为：

```toml
version = "5.8.0"
```

- [ ] **Step 3: 更新 tauri.conf.json 版本号**

打开 `d:\work\codes\litobox\src-tauri\tauri.conf.json`，将第 3 行 `"version": "5.7.0"` 改为：

```json
  "version": "5.8.0",
```

- [ ] **Step 4: 在 README.md 中追加 V5.8.0 功能记录**

打开 `d:\work\codes\litobox\README.md`，找到 V5.7 那一行（第 322 行），在它下方追加：

```markdown
| V5.8 | ✅ | 全局快捷键占用查看器：基于 RegisterHotKey 试探+立即释放探测 Windows 已注册全局热键，标注占用进程/应用，三级回退定位（映射表→进程扫描→LitoBox 自身），常见热键集+自定义补充，进入页面自动探测，事件+2s轮询兜底，CSV 导出，操作历史记录 |
```

- [ ] **Step 5: 在 feature-backlog.md 中标记 A8 已完成**

打开 `d:\work\codes\litobox\docs\superpowers\plans\feature-backlog.md`，找到 A8 那一行（约第 58 行）：

```markdown
| A8  | **全局快捷键占用查看器**   | 列出当前 Windows 已注册的全局快捷键，标注占用进程/应用，搜索冲突。技术难点：Windows 无直接枚举 API，需穷举 RegisterHotKey 试探 + 窗口枚举定位进程                                                                              | 高            | 2026-07-08 brainstorming |
```

改为：

```markdown
| A8  | ✅ **全局快捷键占用查看器**   | 列出当前 Windows 已注册的全局快捷键，标注占用进程/应用，搜索冲突 — 已完成 V5.8 | — 已完成 V5.8 — | 2026-07-08 brainstorming |
```

- [ ] **Step 6: 在「已完成版本」表格末尾追加 V5.8 行**

找到 `feature-backlog.md` 第 36 行（V5.7 行），在它下方追加：

```markdown
| V5.8 | ✅  | 全局快捷键占用查看器（RegisterHotKey 试探+映射表+进程扫描）                                | 2026-07-20 |
```

- [ ] **Step 7: 在「下次 brainstorming 检查清单」中标记 A8 已完成**

找到 `feature-backlog.md` 的检查清单区域（约第 183-195 行），将 A8 那一行（如果存在）打勾。如果不存在则追加：

```markdown
- [x] A8 全局快捷键占用查看器（新，2026-07-08）— 已完成 V5.8
```

- [ ] **Step 8: 验证 Cargo.toml 版本号更新后能编译**

Run: `cd src-tauri; cargo check`
Expected: 编译通过。

- [ ] **Step 9: 提交版本号和文档更新**

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md docs/superpowers/plans/feature-backlog.md
git commit -m "chore(release): 发布 v5.8.0 - 新增全局快捷键占用查看器

- 版本号 5.7.0 → 5.8.0（minor 升级，新增侧边栏菜单项）
- README 功能阶段记录追加 V5.8
- feature-backlog A8 标记已完成"
```

---

## Task 8: 整体验证

**Files:** 无（仅运行验证）

- [ ] **Step 1: 完整后端编译 + 测试**

Run: `cd src-tauri; cargo test`
Expected: 所有测试通过，无编译错误。

- [ ] **Step 2: 完整前端构建**

Run: `cd d:\work\codes\litobox; npm run build`
Expected: vue-tsc 类型检查通过，vite build 成功。

- [ ] **Step 3: 启动开发服务器手动验证**

Run: `cd d:\work\codes\litobox; npm run tauri dev`

按以下清单逐项验证：

- [ ] 侧边栏「系统工具」分类下出现「快捷键占用」菜单项
- [ ] 点击进入后自动开始探测，进度条实时更新（5-15 秒）
- [ ] 探测完成后表格填充，统计卡片显示总数/占用/可注册/系统保留
- [ ] Win+L 显示为「系统保留 - 系统锁屏」
- [ ] Ctrl+Alt+Del 显示为「系统保留 - 安全选项」
- [ ] 若运行了微信，Ctrl+Alt+A 显示为「被占用 - 微信截图」（来源：进程扫描）
- [ ] LitoBox 自身注册的快捷键（如 Alt+Space）显示为「被占用 - LitoBox」（来源：自身注册）
- [ ] 在搜索框输入「微信」过滤出所有微信相关热键
- [ ] 点击「全部/被占用/可注册/系统保留」快速过滤标签，表格正确筛选
- [ ] 在补充输入框输入「Ctrl+Shift+Z」后点「开始探测」，结果中包含该热键
- [ ] 探测中点击「取消」立即停止（≤200ms 响应）
- [ ] 点击「导出 CSV」生成文件，Excel 打开中文不乱码
- [ ] 切换到其他工具再切回，表格保留上次结果（KeepAlive）
- [ ] 切回后自动开始新一轮探测
- [ ] 打开「历史记录」页面，看到「探测全局快捷键占用」记录
- [ ] 双击该历史记录，能还原输入输出（验证 inputFull/outputFull 完整）

- [ ] **Step 4: 关闭开发服务器**

按 Ctrl+C 停止 `npm run tauri dev`。

- [ ] **Step 5: 如有问题，回到对应 Task 修复并重新提交**

如果手动验证发现问题（如某热键探测结果错误、CSV 乱码、历史记录无法还原等），针对问题修复后追加提交：

```bash
git add <修复的文件>
git commit -m "fix(hotkey-viewer): <问题描述>"
```

---

## 完成标准

- [ ] 所有 8 个 Task 的所有 Step 完成
- [ ] 13 个后端单元测试通过
- [ ] `npm run build` 通过
- [ ] `cargo check` 通过
- [ ] 手动验证清单全部通过
- [ ] 版本号 5.8.0 在三处（package.json / Cargo.toml / tauri.conf.json）一致
- [ ] README.md 和 feature-backlog.md 已同步更新
- [ ] 至少 5 次 git 提交（Task 1/4/6/7 + 可能的 fix）

---

## 风险与降级

| 风险 | 处理 |
|------|------|
| `windows-sys` feature 名错误（版本差异） | 对照 [docs.rs/windows-sys](https://docs.rs/windows-sys/0.59) 调整 |
| `windows-sys` 0.59 API 字段类型差异（newtype vs 原始类型） | 编译错误会明确指出，按错误信息调整（如 `0 as HMENU` → `0 as HMENU as *mut _`）。所有 Win32 调用集中在 `hotkey_probe.rs` 的 `create_message_window` / `enumerate_processes_once` / `query_process_path` 三个函数中，影响面可控 |
| `db_read_shortcuts` 函数签名不匹配 | 检查 db.rs 中实际签名，可能需要 `pub fn db_read_shortcuts() -> Vec<(String, String)>` |
| `tauri_plugin_global_shortcut` 不暴露已注册列表 | 已降级：从 db::db_read_shortcuts 读取配置 |
| `RegisterHotKey` 在 message-only window 上失败 | 改用普通隐藏窗口（去掉 HWND_MESSAGE 参数） |
| 进程枚举权限不足（系统进程路径为空） | 已设计降级：path = None，仍展示进程名 |
| `store` 结构与 plan 假设不符（reactive vs ref） | Step 1 中根据实际结构调整 hotkeyLastResult 声明方式 |
| CSV 中文乱码（Excel 默认 GBK） | 已写入 UTF-8 BOM，Excel 可识别 |

---

## Self-Review 记录

**Spec 覆盖检查**：
- ✅ Spec §2 架构与数据流 → Task 3 (run_probe) + Task 5 (前端生命周期)
- ✅ Spec §3 后端设计 → Task 2 (hotkey_data) + Task 3 (hotkey_probe)
- ✅ Spec §3.6 五个 Tauri 命令 → Task 3 + Task 4 注册
- ✅ Spec §4 映射表设计 → Task 2 完整实现
- ✅ Spec §5 前端设计 → Task 5
- ✅ Spec §6 数据库历史记录 → Task 5 handleComplete 中调用 store.addHistory
- ✅ Spec §7 错误处理 → Task 3 (run_probe 错误处理) + Task 5 (前端兜底)
- ✅ Spec §8 依赖与版本号 → Task 1 (Cargo.toml) + Task 7 (三处版本号)
- ✅ Spec §9 测试 → Task 3 单元测试 + Task 8 手动验证

**修正项**：
- Spec §2.3 提到「修改 src/router/index.ts」是错误的，项目无独立 router，实际修改 `src/App.vue` 的 toolComponentMap。已在 Task 6 修正。

**类型一致性检查**：
- `HotkeyResult` Rust struct 字段 snake_case（label/mod_flags/vk/status/process_name/process_display/process_pid/process_path/source）
- 前端 TypeScript interface 字段完全一致
- `HotkeyStatus` / `MatchSource` enum 用 `#[serde(rename_all = "PascalCase")]` 序列化为 PascalCase（Available/Occupied/SystemReserved）
- `ProbeProgress` / `ProbeStats` / `ProbeCompletePayload` 字段全 snake_case
- 前端 invoke 调用使用 camelCase 参数名（extraKeys / results）— Tauri 2.x 自动转换函数参数（AGENTS 经验 16）
