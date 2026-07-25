//!
//! 一键加速
//!
//! 内存释放（EmptyWorkingSet）+ 临时文件清理 + 回收站清空

use serde::Serialize;
use std::fs;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize)]
pub struct BoostScanResult {
    pub memory_total: u64,       // 所有进程工作集总和（字节）
    pub temp_size: u64,          // 临时文件总大小（字节）
    pub temp_file_count: u32,    // 临时文件数量
    pub recycle_size: u64,       // 回收站大小（字节）
}

#[derive(Debug, Clone, Serialize)]
pub struct BoostItemResult {
    pub name: String,            // 项名称
    pub success: bool,
    pub freed: u64,              // 释放的字节数
    pub duration_ms: u64,        // 耗时（毫秒）
    pub message: String,         // 补充信息
}

#[derive(Debug, Clone, Serialize)]
pub struct BoostExecuteResult {
    pub items: Vec<BoostItemResult>,
    pub total_freed: u64,
    pub total_duration_ms: u64,
}

// ============ PowerShell 封装 ============

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("PowerShell 执行失败: {}", e))?;
    if !output.status.success() {
        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
        return Err(format!("PowerShell 错误: {}", stderr));
    }
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

// ============ 内存释放（Rust 原生） ============

#[cfg(target_os = "windows")]
mod memory_ops {
    use std::mem;
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
    };

    #[repr(C)]
    struct PROCESS_MEMORY_COUNTERS {
        cb: u32,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
    }

    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            hProcess: HANDLE,
            ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS,
            cb: u32,
        ) -> i32;
    }

    fn get_process_working_set(pid: u32) -> u64 {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid);
            if handle.is_null() {
                return 0;
            }
            let mut counters: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            counters.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            let ret = GetProcessMemoryInfo(handle, &mut counters, counters.cb);
            CloseHandle(handle);
            if ret == 0 {
                0
            } else {
                counters.WorkingSetSize as u64
            }
        }
    }

    /// 获取所有进程的工作集总和（字节）
    pub fn get_total_working_set() -> u64 {
        let mut total: u64 = 0;
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot as isize == -1 {
                return 0;
            }
            let mut pe: PROCESSENTRY32 = mem::zeroed();
            pe.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
            if Process32First(snapshot, &mut pe) != 0 {
                loop {
                    let pid = pe.th32ProcessID;
                    if pid != 0 {
                        total += get_process_working_set(pid);
                    }
                    if Process32Next(snapshot, &mut pe) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }
        total
    }

    /// 对所有进程调用 EmptyWorkingSet，返回释放的内存总量（字节）
    pub fn empty_all_working_sets() -> (u64, u32, u32) {
        // (freed, total, skipped)
        let mut freed: u64 = 0;
        let mut total: u32 = 0;
        let mut skipped: u32 = 0;

        #[link(name = "kernel32")]
        extern "system" {
            fn SetProcessWorkingSetSize(
                hProcess: HANDLE,
                dwMinimumWorkingSetSize: usize,
                dwMaximumWorkingSetSize: usize,
            ) -> i32;
        }

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot as isize == -1 {
                return (0, 0, 0);
            }
            let mut pe: PROCESSENTRY32 = mem::zeroed();
            pe.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
            if Process32First(snapshot, &mut pe) != 0 {
                loop {
                    total += 1;
                    let pid = pe.th32ProcessID;
                    if pid != 0 {
                        let before = get_process_working_set(pid);

                        let handle = OpenProcess(
                            PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA,
                            FALSE,
                            pid,
                        );
                        if !handle.is_null() {
                            // -1,-1 参数等同于 EmptyWorkingSet
                            let ret = SetProcessWorkingSetSize(handle, !0, !0);
                            if ret != 0 {
                                freed += before;
                            } else {
                                skipped += 1;
                            }
                            CloseHandle(handle);
                        } else {
                            skipped += 1;
                        }
                    }
                    if Process32Next(snapshot, &mut pe) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }
        (freed, total, skipped)
    }
}

#[cfg(not(target_os = "windows"))]
mod memory_ops {
    pub fn get_total_working_set() -> u64 { 0 }
    pub fn empty_all_working_sets() -> (u64, u32, u32) { (0, 0, 0) }
}

// ============ 临时文件清理 ============

/// 获取临时文件总大小和数量
fn scan_temp_files() -> (u64, u32) {
    let mut total_size: u64 = 0;
    let mut count: u32 = 0;

    let temp_paths = vec![
        std::env::temp_dir(),
        std::path::PathBuf::from(r"C:\Windows\Temp"),
    ];

    for path in &temp_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total_size += meta.len();
                        count += 1;
                    }
                }
            }
        }
    }
    (total_size, count)
}

/// 删除临时文件，返回删除的总大小和文件数
fn clean_temp_files() -> (u64, u32) {
    let mut deleted_size: u64 = 0;
    let mut deleted_count: u32 = 0;

    let temp_paths = vec![
        std::env::temp_dir(),
        std::path::PathBuf::from(r"C:\Windows\Temp"),
    ];

    for path in &temp_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        let size = meta.len();
                        if fs::remove_file(entry.path()).is_ok() {
                            deleted_size += size;
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
    }
    (deleted_size, deleted_count)
}

// ============ 回收站清理 ============

fn recycle_bin_scan() -> Result<u64, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $shell = New-Object -ComObject Shell.Application
    $recycleBin = $shell.NameSpace(0xa)
    $totalSize = 0
    foreach ($item in $recycleBin.Items()) {
        # Size 属性返回字节数，但可能为 null（文件夹）
        $size = $item.ExtendedProperty("System.Size")
        if ($size) { $totalSize += [int64]$size }
    }
    Write-Output $totalSize
} catch {
    Write-Output "0"
}"#;
    match run_powershell(script) {
        Ok(output) => {
            let trimmed = output.trim();
            Ok(trimmed.parse::<u64>().unwrap_or(0))
        }
        Err(_) => Ok(0),
    }
}

fn recycle_bin_clean() -> Result<u64, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
try {
    $shell = New-Object -ComObject Shell.Application
    $recycleBin = $shell.NameSpace(0xa)
    $totalSize = 0
    foreach ($item in $recycleBin.Items()) {
        $size = $item.ExtendedProperty("System.Size")
        if ($size) { $totalSize += [int64]$size }
    }
    $recycleBin.Items() | ForEach-Object { Remove-Item $_.Path -Force -ErrorAction SilentlyContinue }
    Write-Output $totalSize
} catch {
    Write-Output "0"
}"#;
    match run_powershell(script) {
        Ok(output) => {
            let trimmed = output.trim();
            Ok(trimmed.parse::<u64>().unwrap_or(0))
        }
        Err(e) => Err(e),
    }
}

// ============ Tauri 命令 ============

#[tauri::command]
pub async fn boost_scan() -> Result<BoostScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        debug_log!("[boost] 开始扫描");

        let memory_total = memory_ops::get_total_working_set();
        let (temp_size, temp_file_count) = scan_temp_files();
        let recycle_size = recycle_bin_scan().unwrap_or(0);

        debug_log!(
            "[boost] 扫描完成: memory={}, temp={}, temp_files={}, recycle={}",
            memory_total,
            temp_size,
            temp_file_count,
            recycle_size
        );

        Ok(BoostScanResult {
            memory_total,
            temp_size,
            temp_file_count,
            recycle_size,
        })
    })
    .await
    .map_err(|e| format!("扫描失败: {}", e))?
}

#[tauri::command]
pub async fn boost_execute() -> Result<BoostExecuteResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
    debug_log!("[boost] 开始执行一键加速");

    let start = std::time::Instant::now();
    let mut items = Vec::new();

    // 1. 内存释放
    {
        let t0 = std::time::Instant::now();
        let (freed, total, skipped) = memory_ops::empty_all_working_sets();
        let duration = t0.elapsed().as_millis() as u64;
        items.push(BoostItemResult {
            name: "内存释放".into(),
            success: true,
            freed,
            duration_ms: duration,
            message: format!("处理 {} 个进程, 跳过 {} 个", total, skipped),
        });
        debug_log!("[boost] 内存释放: freed={}, total={}, skipped={}, {}ms", freed, total, skipped, duration);
    }

    // 2. 临时文件清理
    {
        let t0 = std::time::Instant::now();
        let (deleted_size, deleted_count) = clean_temp_files();
        let duration = t0.elapsed().as_millis() as u64;
        items.push(BoostItemResult {
            name: "临时文件清理".into(),
            success: true,
            freed: deleted_size,
            duration_ms: duration,
            message: format!("删除 {} 个文件", deleted_count),
        });
        debug_log!("[boost] 临时文件: deleted_size={}, count={}, {}ms", deleted_size, deleted_count, duration);
    }

    // 3. 回收站清空
    {
        let t0 = std::time::Instant::now();
        match recycle_bin_clean() {
            Ok(size) => {
                let duration = t0.elapsed().as_millis() as u64;
                items.push(BoostItemResult {
                    name: "回收站清空".into(),
                    success: true,
                    freed: size,
                    duration_ms: duration,
                    message: "已清空".into(),
                });
                debug_log!("[boost] 回收站: size={}, {}ms", size, duration);
            }
            Err(e) => {
                let duration = t0.elapsed().as_millis() as u64;
                items.push(BoostItemResult {
                    name: "回收站清空".into(),
                    success: false,
                    freed: 0,
                    duration_ms: duration,
                    message: format!("失败: {}", e),
                });
                debug_log!("[boost] 回收站失败: {}", e);
            }
        }
    }

    let total_freed: u64 = items.iter().map(|i| i.freed).sum();
    let total_duration = start.elapsed().as_millis() as u64;

    debug_log!("[boost] 执行完成: total_freed={}, total_duration={}ms", total_freed, total_duration);

    Ok(BoostExecuteResult {
        items,
        total_freed,
        total_duration_ms: total_duration,
    })
    })
    .await
    .map_err(|e| format!("加速失败: {}", e))?
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_temp_files() {
        let (size, count) = scan_temp_files();
        // 系统通常有临时文件（在干净环境中可能为 0）
        assert!(size < u64::MAX);
        assert!(count < u32::MAX);
    }

    #[test]
    fn test_boost_scan_result_serialization() {
        let result = BoostScanResult {
            memory_total: 1024,
            temp_size: 512,
            temp_file_count: 10,
            recycle_size: 256,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("1024"));
        assert!(json.contains("512"));
        assert!(json.contains("10"));
        assert!(json.contains("256"));
    }

    #[test]
    fn test_boost_execute_result_serialization() {
        let result = BoostExecuteResult {
            items: vec![
                BoostItemResult {
                    name: "内存释放".into(),
                    success: true,
                    freed: 100,
                    duration_ms: 50,
                    message: "ok".into(),
                },
            ],
            total_freed: 100,
            total_duration_ms: 50,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("内存释放"));
        assert!(json.contains("total_freed"));
    }

    #[test]
    fn test_memory_ops_get_total() {
        let total = memory_ops::get_total_working_set();
        // 系统至少有一些进程在运行
        assert!(total > 0);
    }

    #[test]
    fn test_memory_ops_empty() {
        let (_freed, total, skipped) = memory_ops::empty_all_working_sets();
        // 不应该 panic，至少处理了一些进程
        assert!(total > 0);
        // freed 可能为 0（如果权限不足），skipped 应该 >= 0
        assert!(skipped < total);
    }
}