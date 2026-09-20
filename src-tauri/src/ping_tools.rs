//! Ping / Traceroute 工具 — 封装 Windows ping/tracert 命令，实时推送进度
//!
//! 功能：
//! - Ping：可视化 ICMP 回显，实时显示每次回复的延迟/TTL，统计丢包率与平均延迟
//! - Traceroute：路由跟踪，显示每跳 IP 与三次延迟
//!
//! 实现：用 std::process::Command spawn 子进程，独立线程逐行读取 stdout，
//! 通过 tauri 事件推送进度，结束时推送 complete 事件。

use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式日志
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

/// 单次 Ping 回复
#[derive(Debug, Clone, Serialize)]
pub struct PingReply {
    pub seq: u32,
    pub bytes: u32,
    pub time_ms: f64,
    pub ttl: u32,
    pub from: String,
    pub timeout: bool,
}

/// Ping 统计
#[derive(Debug, Clone, Serialize)]
pub struct PingStats {
    pub sent: u32,
    pub received: u32,
    pub lost: u32,
    pub loss_percent: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
}

/// Ping 进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum PingEvent {
    Reply { reply: PingReply },
    Stats { stats: PingStats },
    Error { message: String },
    Complete,
}

/// Traceroute 单跳
#[derive(Debug, Clone, Serialize)]
pub struct TraceHop {
    pub hop: u32,
    pub rtt1: String,
    pub rtt2: String,
    pub rtt3: String,
    pub ip: String,
    pub hostname: String,
    pub timeout: bool,
}

/// Traceroute 进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum TraceEvent {
    Hop { hop: TraceHop },
    Error { message: String },
    Complete,
}

/// 任务状态
#[derive(Debug, Clone, Serialize)]
pub struct TaskStatus {
    pub running: bool,
    pub task_type: String, // "ping" | "tracert"
}

// ============ 全局任务状态 ============

struct TaskState {
    cancel_flag: Arc<AtomicBool>,
    task_type: String,
}

static TASKS: OnceLock<Mutex<Option<TaskState>>> = OnceLock::new();

fn tasks() -> &'static Mutex<Option<TaskState>> {
    TASKS.get_or_init(|| Mutex::new(None))
}

// ============ 工具函数 ============

fn decode_gbk(bytes: &[u8]) -> String {
    let (text, _, _) = encoding_rs::GBK.decode(bytes);
    text.into_owned()
}

/// 从 ping 回复行中解析字段
/// 例：来自 110.242.68.66 的回复: 字节=32 时间=23ms TTL=52
fn parse_ping_reply(line: &str, seq: u32) -> Option<PingReply> {
    if !line.contains("回复") && !line.contains("bytes=") && !line.contains("time=") {
        return None;
    }
    let mut from = String::new();
    let mut bytes: u32 = 32;
    let mut time_ms: f64 = 0.0;
    let mut ttl: u32 = 0;

    // 提取来源 IP（注意：find 返回字节索引，偏移量要用字节长度）
    const FROM_CN: &str = "来自 ";
    const FROM_EN: &str = "Reply from ";
    if let Some(idx) = line.find(FROM_CN) {
        let rest = &line[idx + FROM_CN.len()..];
        if let Some(end) = rest.find(' ') {
            from = rest[..end].trim().to_string();
        }
    } else if let Some(idx) = line.find(FROM_EN) {
        let rest = &line[idx + FROM_EN.len()..];
        if let Some(end) = rest.find([':', ' ']) {
            from = rest[..end].trim().to_string();
        }
    }

    // 提取 bytes
    const BYTES_CN: &str = "字节=";
    const BYTES_EN: &str = "bytes=";
    if let Some(idx) = line.find(BYTES_CN) {
        let rest = &line[idx + BYTES_CN.len()..];
        if let Some(end) = rest.find(' ') {
            bytes = rest[..end].trim().parse().unwrap_or(32);
        }
    } else if let Some(idx) = line.find(BYTES_EN) {
        let rest = &line[idx + BYTES_EN.len()..];
        if let Some(end) = rest.find(' ') {
            bytes = rest[..end].trim().parse().unwrap_or(32);
        }
    }

    // 提取 time
    const TIME_CN: &str = "时间=";
    const TIME_EN: &str = "time=";
    const TIME_LT: &str = "time<";
    if let Some(idx) = line.find(TIME_CN) {
        let rest = &line[idx + TIME_CN.len()..];
        if let Some(end) = rest.find("ms") {
            time_ms = rest[..end].trim().parse().unwrap_or(0.0);
        }
    } else if let Some(idx) = line.find(TIME_EN) {
        let rest = &line[idx + TIME_EN.len()..];
        if let Some(end) = rest.find("ms") {
            time_ms = rest[..end].trim().parse().unwrap_or(0.0);
        }
    } else if let Some(idx) = line.find(TIME_LT) {
        let rest = &line[idx + TIME_LT.len()..];
        if let Some(end) = rest.find("ms") {
            time_ms = rest[..end].trim().parse().unwrap_or(0.0);
        }
    }

    // 提取 TTL
    if let Some(idx) = line.find("TTL=") {
        let rest = &line[idx + "TTL=".len()..];
        ttl = rest.trim().parse().unwrap_or(0);
    }

    Some(PingReply {
        seq,
        bytes,
        time_ms,
        ttl,
        from,
        timeout: false,
    })
}

/// 从 ping 统计行解析
fn parse_ping_stats(lines: &[String]) -> Option<PingStats> {
    let mut sent = 0u32;
    let mut received = 0u32;
    let mut lost = 0u32;
    let mut loss_percent = 0.0f64;
    let mut min_ms = 0.0f64;
    let mut max_ms = 0.0f64;
    let mut avg_ms = 0.0f64;

    // 中文标签（用 .len() 取字节长度，避免字符/字节偏移错误）
    const SENT_CN: &str = "已发送 = ";
    const RECEIVED_CN: &str = "已接收 = ";
    const LOST_CN: &str = "丢失 = ";
    const MIN_CN: &str = "最短 = ";
    const MAX_CN: &str = "最长 = ";
    const AVG_CN: &str = "平均 = ";

    for line in lines {
        let l = line.trim();
        // 数据包统计行
        if l.contains("已发送") || l.contains("Sent") {
            if let Some(idx) = l.find(SENT_CN) {
                let rest = &l[idx + SENT_CN.len()..];
                if let Some(end) = rest.find('，') {
                    sent = rest[..end].trim().parse().unwrap_or(0);
                }
            } else if let Some(idx) = l.find("Sent = ") {
                let rest = &l[idx + "Sent = ".len()..];
                if let Some(end) = rest.find(',') {
                    sent = rest[..end].trim().parse().unwrap_or(0);
                }
            }
            if let Some(idx) = l.find(RECEIVED_CN) {
                let rest = &l[idx + RECEIVED_CN.len()..];
                if let Some(end) = rest.find('，') {
                    received = rest[..end].trim().parse().unwrap_or(0);
                }
            } else if let Some(idx) = l.find("Received = ") {
                let rest = &l[idx + "Received = ".len()..];
                if let Some(end) = rest.find(',') {
                    received = rest[..end].trim().parse().unwrap_or(0);
                }
            }
            if let Some(idx) = l.find(LOST_CN) {
                let rest = &l[idx + LOST_CN.len()..];
                if let Some(end) = rest.find('(') {
                    lost = rest[..end].trim().parse().unwrap_or(0);
                }
            } else if let Some(idx) = l.find("Lost = ") {
                let rest = &l[idx + "Lost = ".len()..];
                if let Some(end) = rest.find('(') {
                    lost = rest[..end].trim().parse().unwrap_or(0);
                }
            }
            if let Some(idx) = l.find("(") {
                let rest = &l[idx + 1..];
                if let Some(end) = rest.find("%") {
                    loss_percent = rest[..end].trim().parse().unwrap_or(0.0);
                }
            }
        }
        // 延迟统计行
        if l.contains("最短") || l.contains("Minimum") {
            if let Some(idx) = l.find(MIN_CN) {
                let rest = &l[idx + MIN_CN.len()..];
                if let Some(end) = rest.find("ms") {
                    min_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            } else if let Some(idx) = l.find("Minimum = ") {
                let rest = &l[idx + "Minimum = ".len()..];
                if let Some(end) = rest.find("ms") {
                    min_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            }
            if let Some(idx) = l.find(MAX_CN) {
                let rest = &l[idx + MAX_CN.len()..];
                if let Some(end) = rest.find("ms") {
                    max_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            } else if let Some(idx) = l.find("Maximum = ") {
                let rest = &l[idx + "Maximum = ".len()..];
                if let Some(end) = rest.find("ms") {
                    max_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            }
            if let Some(idx) = l.find(AVG_CN) {
                let rest = &l[idx + AVG_CN.len()..];
                if let Some(end) = rest.find("ms") {
                    avg_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            } else if let Some(idx) = l.find("Average = ") {
                let rest = &l[idx + "Average = ".len()..];
                if let Some(end) = rest.find("ms") {
                    avg_ms = rest[..end].trim().parse().unwrap_or(0.0);
                }
            }
        }
    }

    if sent > 0 {
        Some(PingStats {
            sent,
            received,
            lost,
            loss_percent,
            min_ms,
            max_ms,
            avg_ms,
        })
    } else {
        None
    }
}

/// 从 tracert 行解析单跳
/// 例：  1    <1 ms    <1 ms    <1 ms  192.168.1.1
/// 超时：  2     *        *        *     请求超时。
fn parse_trace_hop(line: &str) -> Option<TraceHop> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }
    // 第一个 token 是跳数
    let hop: u32 = match tokens[0].parse() {
        Ok(h) => h,
        Err(_) => return None,
    };

    let mut rtt1 = String::new();
    let mut rtt2 = String::new();
    let mut rtt3 = String::new();
    let mut ip = String::new();
    let mut hostname = String::new();
    let mut timeout = false;

    if tokens.len() >= 8 {
        // 正常：hop rtt1 ms rtt2 ms rtt3 ms ip [hostname]
        rtt1 = format!("{} {}", tokens[1], tokens[2]);
        rtt2 = format!("{} {}", tokens[3], tokens[4]);
        rtt3 = format!("{} {}", tokens[5], tokens[6]);
        ip = tokens[7].to_string();
        if tokens.len() > 8 {
            hostname = tokens[8..].join(" ");
        }
    } else if tokens.len() >= 5 {
        // 可能是超时或其他格式
        if tokens.iter().any(|t| *t == "*") {
            timeout = true;
            rtt1 = "*".to_string();
            rtt2 = "*".to_string();
            rtt3 = "*".to_string();
            // 尝试找 IP
            for t in &tokens[1..] {
                if t.contains('.') && !t.contains('*') {
                    ip = t.to_string();
                    break;
                }
            }
        } else {
            // 兼容其他格式
            rtt1 = tokens[1].to_string();
            rtt2 = tokens.get(2).map(|s| s.to_string()).unwrap_or_default();
            rtt3 = tokens.get(3).map(|s| s.to_string()).unwrap_or_default();
            ip = tokens.get(4).map(|s| s.to_string()).unwrap_or_default();
        }
    }

    Some(TraceHop {
        hop,
        rtt1,
        rtt2,
        rtt3,
        ip,
        hostname,
        timeout,
    })
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn ping_echo(host: String, count: u32, timeout_ms: u32, size: u32) -> String {
    format!("host={}, count={}, timeout_ms={}, size={}", host, count, timeout_ms, size)
}

#[tauri::command]
pub fn ping_get_status() -> TaskStatus {
    let guard = tasks().lock().unwrap();
    match guard.as_ref() {
        Some(s) => TaskStatus {
            running: !s.cancel_flag.load(Ordering::Relaxed),
            task_type: s.task_type.clone(),
        },
        None => TaskStatus {
            running: false,
            task_type: String::new(),
        },
    }
}

#[tauri::command]
pub async fn ping_start(
    app: AppHandle,
    host: String,
    count: u32,
    timeout_ms: u32,
    size: u32,
) -> Result<(), String> {
    eprintln!("[ping_tools] ping_start 被调用: host={}, count={}, timeout_ms={}, size={}", host, count, timeout_ms, size);
    // 检查是否已有任务在运行
    {
        let guard = tasks().lock().unwrap();
        if guard.is_some() {
            return Err("已有任务正在运行，请先停止".to_string());
        }
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut guard = tasks().lock().unwrap();
        *guard = Some(TaskState {
            cancel_flag: cancel_flag.clone(),
            task_type: "ping".to_string(),
        });
    }

    let host_clone = host.clone();
    let cancel_clone = cancel_flag.clone();
    let app_clone = app.clone();

    thread::spawn(move || {
        eprintln!("[ping_tools] 启动 ping 子进程: {} -n {} -w {} -l {}", host_clone, count, timeout_ms, size);
        let mut cmd = Command::new("ping");
        cmd.args([
            "-n",
            &count.to_string(),
            "-w",
            &timeout_ms.to_string(),
            "-l",
            &size.to_string(),
            &host_clone,
        ]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = match cmd.spawn() {
            Ok(c) => {
                eprintln!("[ping_tools] ping 子进程已启动, pid={:?}", c.id());
                c
            }
            Err(e) => {
                eprintln!("[ping_tools] ping 子进程启动失败: {}", e);
                let _ = app_clone.emit(
                    "ping-event",
                    PingEvent::Error {
                        message: format!("启动 ping 失败: {}", e),
                    },
                );
                let _ = app_clone.emit("ping-event", PingEvent::Complete);
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut seq: u32 = 0;
        let mut stats_lines: Vec<String> = Vec::new();
        let mut in_stats = false;
        let mut buf: Vec<u8> = Vec::new();

        loop {
            if cancel_clone.load(Ordering::Relaxed) {
                let _ = child.kill();
                break;
            }
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(_) => break,
            }
            let line = decode_gbk(&buf);
            debug_log!("[ping] {}", line);

            let trimmed = line.trim();

            // 检测统计段开始
            if trimmed.contains("统计信息") || trimmed.contains("Ping statistics") {
                in_stats = true;
                continue;
            }

            if in_stats {
                stats_lines.push(trimmed.to_string());
                continue;
            }

            // 超时行
            if trimmed.contains("请求超时") || trimmed.contains("Request timed out") {
                seq += 1;
                let _ = app_clone.emit(
                    "ping-event",
                    PingEvent::Reply {
                        reply: PingReply {
                            seq,
                            bytes: 0,
                            time_ms: 0.0,
                            ttl: 0,
                            from: String::new(),
                            timeout: true,
                        },
                    },
                );
                continue;
            }

            // 回复行
            if let Some(reply) = parse_ping_reply(trimmed, seq + 1) {
                seq += 1;
                let _ = app_clone.emit("ping-event", PingEvent::Reply { reply });
            }
        }

        // 等待子进程结束
        let _ = child.wait();

        // 推送统计
        if let Some(stats) = parse_ping_stats(&stats_lines) {
            let _ = app_clone.emit("ping-event", PingEvent::Stats { stats });
        }

        let _ = app_clone.emit("ping-event", PingEvent::Complete);

        // 清理任务状态
        let mut guard = tasks().lock().unwrap();
        *guard = None;
    });

    Ok(())
}

#[tauri::command]
pub fn ping_cancel() -> Result<(), String> {
    let guard = tasks().lock().unwrap();
    if let Some(s) = guard.as_ref() {
        s.cancel_flag.store(true, Ordering::Relaxed);
    }
    drop(guard);
    Ok(())
}

#[tauri::command]
pub async fn tracert_start(app: AppHandle, host: String, max_hops: u32, timeout_ms: u32) -> Result<(), String> {
    {
        let guard = tasks().lock().unwrap();
        if guard.is_some() {
            return Err("已有任务正在运行，请先停止".to_string());
        }
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut guard = tasks().lock().unwrap();
        *guard = Some(TaskState {
            cancel_flag: cancel_flag.clone(),
            task_type: "tracert".to_string(),
        });
    }

    let host_clone = host.clone();
    let cancel_clone = cancel_flag.clone();
    let app_clone = app.clone();

    thread::spawn(move || {
        let mut cmd = Command::new("tracert");
        cmd.args([
            "-d",
            "-h",
            &max_hops.to_string(),
            "-w",
            &timeout_ms.to_string(),
            &host_clone,
        ]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = app_clone.emit(
                    "tracert-event",
                    TraceEvent::Error {
                        message: format!("启动 tracert 失败: {}", e),
                    },
                );
                let _ = app_clone.emit("tracert-event", TraceEvent::Complete);
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut buf: Vec<u8> = Vec::new();

        loop {
            if cancel_clone.load(Ordering::Relaxed) {
                let _ = child.kill();
                break;
            }
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
            let line = decode_gbk(&buf);
            debug_log!("[tracert] {}", line);

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 跳行：以数字开头
            if let Some(hop) = parse_trace_hop(trimmed) {
                let _ = app_clone.emit("tracert-event", TraceEvent::Hop { hop });
            }
        }

        let _ = child.wait();
        let _ = app_clone.emit("tracert-event", TraceEvent::Complete);

        let mut guard = tasks().lock().unwrap();
        *guard = None;
    });

    Ok(())
}

#[tauri::command]
pub fn tracert_cancel() -> Result<(), String> {
    let guard = tasks().lock().unwrap();
    if let Some(s) = guard.as_ref() {
        s.cancel_flag.store(true, Ordering::Relaxed);
    }
    drop(guard);
    Ok(())
}

// ============ 单元测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_reply_cn() {
        let line = "来自 110.242.68.66 的回复: 字节=32 时间=23ms TTL=52";
        let reply = parse_ping_reply(line, 1).unwrap();
        assert_eq!(reply.from, "110.242.68.66");
        assert_eq!(reply.bytes, 32);
        assert_eq!(reply.time_ms, 23.0);
        assert_eq!(reply.ttl, 52);
        assert!(!reply.timeout);
    }

    #[test]
    fn test_parse_ping_reply_en() {
        let line = "Reply from 110.242.68.66: bytes=32 time=24ms TTL=52";
        let reply = parse_ping_reply(line, 1).unwrap();
        assert_eq!(reply.from, "110.242.68.66");
        assert_eq!(reply.time_ms, 24.0);
        assert_eq!(reply.ttl, 52);
    }

    #[test]
    fn test_parse_ping_stats() {
        let lines = vec![
            "110.242.68.66 的 Ping 统计信息:".to_string(),
            "    数据包: 已发送 = 4，已接收 = 4，丢失 = 0 (0% 丢失)，".to_string(),
            "往返行程的估计时间(以毫秒为单位):".to_string(),
            "    最短 = 22ms，最长 = 25ms，平均 = 23ms".to_string(),
        ];
        let stats = parse_ping_stats(&lines).unwrap();
        assert_eq!(stats.sent, 4);
        assert_eq!(stats.received, 4);
        assert_eq!(stats.lost, 0);
        assert_eq!(stats.loss_percent, 0.0);
        assert_eq!(stats.min_ms, 22.0);
        assert_eq!(stats.max_ms, 25.0);
        assert_eq!(stats.avg_ms, 23.0);
    }

    #[test]
    fn test_parse_trace_hop_normal() {
        let line = "  1    <1 ms    <1 ms    <1 ms  192.168.1.1";
        let hop = parse_trace_hop(line).unwrap();
        assert_eq!(hop.hop, 1);
        assert_eq!(hop.ip, "192.168.1.1");
        assert!(!hop.timeout);
    }

    #[test]
    fn test_parse_trace_hop_timeout() {
        let line = "  2     *        *        *     请求超时。";
        let hop = parse_trace_hop(line).unwrap();
        assert_eq!(hop.hop, 2);
        assert!(hop.timeout);
    }
}
