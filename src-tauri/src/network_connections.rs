//! 网络连接查看器 — netstat 解析 + PowerShell fallback
//!
//! 采集所有 TCP/UDP 连接，关联进程名/路径，供前端 NetworkConnections.vue 使用。

use serde::Serialize;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*)
        }
    };
}

// ============ 数据结构 ============

#[derive(Debug, Clone, Serialize)]
pub struct NetworkConnection {
    pub protocol: String,       // "TCP" / "UDP"
    pub local_addr: String,     // "0.0.0.0:8080"
    pub remote_addr: String,    // "192.168.1.5:443" (UDP 时为 "*:*")
    pub state: String,          // "LISTENING"/"ESTABLISHED"/"TIME_WAIT"/... (UDP 为空)
    pub pid: u32,
    pub process_name: String,   // "node.exe" / "(已退出)"
    pub process_path: String,   // "C:\Program Files\nodejs\node.exe" (权限不足时为空)
}

// ============ netstat 解析 ============

/// 解析 netstat -ano 输出
fn parse_netstat_output(output: &str) -> Vec<NetworkConnection> {
    let mut connections = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();

        // 需要至少 4 个 token（UDP 最小: UDP local *:* PID）
        if tokens.len() < 4 {
            continue;
        }

        let protocol = tokens[0].to_uppercase();

        match protocol.as_str() {
            "TCP" => {
                if tokens.len() < 5 {
                    continue;
                }
                // tokens: TCP, local_addr, remote_addr, STATE, PID
                let pid: u32 = match tokens[tokens.len() - 1].parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let state = tokens[tokens.len() - 2].to_string();
                let local_addr = tokens[1].to_string();
                let remote_addr = tokens[2].to_string();

                connections.push(NetworkConnection {
                    protocol: "TCP".to_string(),
                    local_addr,
                    remote_addr,
                    state,
                    pid,
                    process_name: String::new(),
                    process_path: String::new(),
                });
            }
            "UDP" => {
                // tokens: UDP, local_addr, *:*, PID
                let pid: u32 = match tokens[tokens.len() - 1].parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let local_addr = tokens[1].to_string();
                let remote_addr = tokens[2].to_string();

                connections.push(NetworkConnection {
                    protocol: "UDP".to_string(),
                    local_addr,
                    remote_addr,
                    state: String::new(),
                    pid,
                    process_name: String::new(),
                    process_path: String::new(),
                });
            }
            _ => continue,
        }
    }

    connections
}

// ============ PowerShell fallback ============

/// 解析 PowerShell Get-NetTCPConnection JSON 输出
fn parse_ps_tcp(json: &str) -> Vec<NetworkConnection> {
    #[derive(serde::Deserialize)]
    struct PsTcp {
        #[serde(rename = "State")]
        state: u32,
        #[serde(rename = "LocalAddress")]
        local_address: String,
        #[serde(rename = "LocalPort")]
        local_port: u32,
        #[serde(rename = "RemoteAddress")]
        remote_address: String,
        #[serde(rename = "RemotePort")]
        remote_port: u32,
        #[serde(rename = "OwningProcess")]
        owning_process: u32,
    }

    let items: Vec<PsTcp> = serde_json::from_str(json).unwrap_or_default();

    items
        .into_iter()
        .map(|c| {
            let state_name = match c.state {
                1 => "CLOSED",
                2 => "LISTENING",
                3 => "SYN_SENT",
                4 => "SYN_RECEIVED",
                5 => "ESTABLISHED",
                6 => "FIN_WAIT1",
                7 => "FIN_WAIT2",
                8 => "CLOSE_WAIT",
                9 => "LAST_ACK",
                10 => "CLOSING",
                11 => "TIME_WAIT",
                12 => "DELETE_TCB",
                _ => "UNKNOWN",
            };
            NetworkConnection {
                protocol: "TCP".to_string(),
                local_addr: format!("{}:{}", c.local_address, c.local_port),
                remote_addr: format!("{}:{}", c.remote_address, c.remote_port),
                state: state_name.to_string(),
                pid: c.owning_process,
                process_name: String::new(),
                process_path: String::new(),
            }
        })
        .collect()
}

/// 解析 PowerShell Get-NetUDPEndpoint JSON 输出
fn parse_ps_udp(json: &str) -> Vec<NetworkConnection> {
    #[derive(serde::Deserialize)]
    struct PsUdp {
        #[serde(rename = "LocalAddress")]
        local_address: String,
        #[serde(rename = "LocalPort")]
        local_port: u32,
        #[serde(rename = "OwningProcess")]
        owning_process: u32,
    }

    let items: Vec<PsUdp> = serde_json::from_str(json).unwrap_or_default();

    items
        .into_iter()
        .map(|c| NetworkConnection {
            protocol: "UDP".to_string(),
            local_addr: format!("{}:{}", c.local_address, c.local_port),
            remote_addr: "*:*".to_string(),
            state: String::new(),
            pid: c.owning_process,
            process_name: String::new(),
            process_path: String::new(),
        })
        .collect()
}

// ============ 进程信息补全 ============

/// 用 sysinfo 补全进程名和进程路径（best-effort，失败不报错）
fn enrich_process_info(connections: &mut Vec<NetworkConnection>) {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    for conn in connections.iter_mut() {
        if conn.pid == 0 {
            conn.process_name = "系统空闲进程".to_string();
            continue;
        }
        if let Some(process) = sys.process(sysinfo::Pid::from_u32(conn.pid)) {
            conn.process_name = process.name().to_string_lossy().to_string();
            conn.process_path = process
                .exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            // 去掉 Windows 长路径前缀
            if conn.process_path.starts_with(r"\\?\") {
                conn.process_path = conn.process_path[4..].to_string();
            }
        } else {
            conn.process_name = "(已退出)".to_string();
        }
    }
}

// ============ Tauri 命令 ============

#[tauri::command]
pub fn get_network_connections() -> Result<Vec<NetworkConnection>, String> {
    debug_log!("[network_connections] 开始采集");

    // 主路径: netstat -ano
    let mut cmd = Command::new("netstat");
    cmd.args(["-ano"]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut connections = match cmd.output() {
        Ok(output) => {
            let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
            let parsed = parse_netstat_output(&text);
            debug_log!("[network_connections] netstat 解析到 {} 条连接", parsed.len());
            parsed
        }
        Err(e) => {
            debug_log!("[network_connections] netstat 执行失败: {}", e);
            Vec::new()
        }
    };

    // Fallback: PowerShell cmdlet（netstat 无结果时启用）
    if connections.is_empty() {
        debug_log!("[network_connections] netstat 无结果，尝试 PowerShell fallback");

        // TCP
        let mut ps_cmd = Command::new("powershell");
        ps_cmd.args([
            "-NoProfile",
            "-Command",
            "Get-NetTCPConnection | Select-Object State,LocalAddress,LocalPort,RemoteAddress,RemotePort,OwningProcess | ConvertTo-Json",
        ]);
        #[cfg(target_os = "windows")]
        ps_cmd.creation_flags(CREATE_NO_WINDOW);

        if let Ok(output) = ps_cmd.output() {
            let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
            let tcp = parse_ps_tcp(&text);
            debug_log!("[network_connections] PowerShell TCP 解析到 {} 条", tcp.len());
            connections.extend(tcp);
        }

        // UDP
        let mut ps_cmd = Command::new("powershell");
        ps_cmd.args([
            "-NoProfile",
            "-Command",
            "Get-NetUDPEndpoint | Select-Object LocalAddress,LocalPort,OwningProcess | ConvertTo-Json",
        ]);
        #[cfg(target_os = "windows")]
        ps_cmd.creation_flags(CREATE_NO_WINDOW);

        if let Ok(output) = ps_cmd.output() {
            let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
            let udp = parse_ps_udp(&text);
            debug_log!("[network_connections] PowerShell UDP 解析到 {} 条", udp.len());
            connections.extend(udp);
        }
    }

    // 补全进程信息
    enrich_process_info(&mut connections);
    debug_log!("[network_connections] 最终 {} 条连接", connections.len());

    Ok(connections)
}