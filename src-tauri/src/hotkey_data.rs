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
    // Windows 系统热键
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
    (MOD_WIN | MOD_CONTROL, VK_TAB, "虚拟桌面切换"),      // Win+Ctrl+Tab
    (MOD_WIN | MOD_CONTROL, 0x44, "新建虚拟桌面"),        // Win+Ctrl+D ('D' = 0x44)
    (MOD_WIN | MOD_CONTROL, 0x73, "关闭虚拟桌面"),        // Win+Ctrl+F4 (VK_F4 = 0x73)
    (MOD_WIN | MOD_CONTROL, 0x25, "虚拟桌面左切"),        // Win+Ctrl+Left (VK_LEFT = 0x25)
    (MOD_WIN | MOD_CONTROL, 0x27, "虚拟桌面右切"),        // Win+Ctrl+Right (VK_RIGHT = 0x27)
    (MOD_CONTROL | MOD_ALT, VK_DELETE, "安全选项"),       // Ctrl+Alt+Del
    // ponytail: 修正：MOD_MENU 是 Windows 别名同值 0x0001，统一用本文件定义的 MOD_ALT
    // 移除原"安全选项增强"条目（Ctrl+Alt+Shift+Del 非真实系统热键）
    
    // 常用应用内快捷键（不建议注册为全局热键）
    (MOD_CONTROL, 0x41, "全选"),                 // Ctrl+A
    (MOD_CONTROL, 0x43, "复制"),                 // Ctrl+C
    (MOD_CONTROL, 0x56, "粘贴"),                 // Ctrl+V
    (MOD_CONTROL, 0x58, "剪切"),                 // Ctrl+X
    (MOD_CONTROL, 0x5A, "撤销"),                 // Ctrl+Z
    (MOD_CONTROL | MOD_SHIFT, 0x5A, "重做"),      // Ctrl+Shift+Z
    (MOD_CONTROL, 0x53, "保存"),                 // Ctrl+S
    (MOD_CONTROL, 0x46, "查找"),                 // Ctrl+F
    (MOD_CONTROL, 0x47, "查找下一个"),           // Ctrl+G
    (MOD_CONTROL, 0x59, "替换"),                 // Ctrl+H
    (MOD_CONTROL, 0x57, "关闭"),                 // Ctrl+W
    (MOD_CONTROL, 0x54, "新建标签"),             // Ctrl+T
    (MOD_CONTROL, VK_TAB, "切换标签"),           // Ctrl+Tab
    (MOD_CONTROL | MOD_SHIFT, VK_TAB, "反向切换标签"), // Ctrl+Shift+Tab
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
                name: "system".to_string(),
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
    APP_HOTKEYS.iter().find_map(|(m, v, display, procs)| {
        if *m == mod_flags && *v == vk {
            let name = if procs.is_empty() {
                display.to_string()
            } else {
                procs[0].to_string()
            };
            Some(ProcessInfo {
                name,
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
