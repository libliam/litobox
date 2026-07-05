// ponytail: debug 模式输出日志到 stderr，release 模式编译时移除（零开销）
// 用法: debug_log!("查询失败: {}", err)
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*)
    };
}

use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ============ 数据结构 ============

#[derive(Serialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub hostname: String,
    pub uptime_secs: u64,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub brand: String,
    pub core_count: usize,
    pub thread_count: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub file_system: String,
    pub is_removable: bool,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub hostname: String,
    pub interfaces: Vec<NetInterface>,
    pub default_gateway: String,
    pub dns_servers: Vec<String>,
    pub wifi_name: Option<String>,
    pub active_connections: Vec<TcpConnection>,
    pub listening_ports: Vec<ListeningPort>,
}

#[derive(Serialize)]
pub struct NetInterface {
    pub name: String,
    pub mac: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct TcpConnection {
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub pid: u32,
}

#[derive(Serialize)]
pub struct ListeningPort {
    pub protocol: String,
    pub local_addr: String,
    pub pid: u32,
    pub process_name: String,
}

#[derive(Serialize)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub command: String,
}

#[derive(Serialize, Debug)]
pub struct KillResult {
    pub success: bool,
    pub pid: u32,
    pub process_name: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HardwareInfo {
    pub cpu: CpuSummary,
    pub memory: MemorySummary,
    pub disks: Vec<DiskSummary>,
    pub gpus: Vec<GpuInfo>,
    pub displays: Vec<DisplayInfo>,
    pub audio_devices: Vec<AudioDevice>,
    pub motherboard: MotherboardInfo,
    pub battery: Option<BatteryInfo>,
    pub usb_devices: Vec<UsbDevice>,
}

#[derive(Serialize)]
pub struct CpuSummary {
    pub name: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_mhz: u64,
}

#[derive(Serialize)]
pub struct MemorySummary {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
}

#[derive(Serialize)]
pub struct DiskSummary {
    pub name: String,
    pub model: String,
    pub size_gb: f64,
    pub free_gb: f64,
    pub fs_type: String,
}

#[derive(Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub driver_version: String,
    pub vram_mb: u64,
}

#[derive(Serialize)]
pub struct DisplayInfo {
    pub name: String,
    pub resolution: String,
}

#[derive(Serialize)]
pub struct AudioDevice {
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
}

#[derive(Serialize)]
pub struct BatteryInfo {
    pub status: String,
    pub charge_percent: u32,
    pub estimated_time: String,
}

#[derive(Serialize)]
pub struct UsbDevice {
    pub name: String,
    pub device_id: String,
}

#[derive(Serialize)]
pub struct SoftwareEnv {
    pub installed_software: Vec<SoftwareItem>,
    pub environment_variables: Vec<EnvVar>,
    pub startup_items: Vec<StartupItem>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SoftwareItem {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_date: String,
}

#[derive(Serialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location: String,
}

// ============ PowerShell 辅助函数 ============

fn get_interfaces_from_ipconfig() -> Vec<NetInterface> {
    let mut cmd = Command::new("ipconfig");
    cmd.args(["/all"]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            debug_log!("ipconfig 执行失败: {}", e);
            return Vec::new();
        }
    };
    
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    let lines: Vec<&str> = text.lines().collect();
    
    let mut interfaces = Vec::new();
    let mut current: Option<NetInterface> = None;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed.starts_with("以太网适配器") || trimmed.starts_with("无线局域网适配器") {
            if let Some(iface) = current.take() {
                if !iface.ipv4.is_empty() || !iface.ipv6.is_empty() {
                    interfaces.push(iface);
                }
            }
            
            let name_start = trimmed.find(' ').map_or(0, |i| i + 1);
            let name = trimmed[name_start..].trim_end_matches(':').trim().to_string();
            current = Some(NetInterface {
                name,
                mac: String::new(),
                ipv4: Vec::new(),
                ipv6: Vec::new(),
                status: "Up".to_string(),
            });
        } else if let Some(ref mut iface) = current {
            if trimmed.starts_with("物理地址") || trimmed.starts_with("Physical Address") {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    iface.mac = parts[1].trim().replace('-', "-").to_string();
                }
            } else if trimmed.starts_with("IPv4 地址") || trimmed.starts_with("IPv4 Address") {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let ip = parts[1].trim().split_whitespace().next().unwrap_or("");
                    if !ip.is_empty() && ip != "(首选)" && !ip.starts_with('(') {
                        iface.ipv4.push(ip.to_string());
                    }
                }
            } else if trimmed.starts_with("IPv6 地址") || trimmed.starts_with("IPv6 Address") {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let ip = parts[1].trim().split_whitespace().next().unwrap_or("");
                    if !ip.is_empty() && !ip.starts_with('%') && !ip.starts_with('(') {
                        iface.ipv6.push(ip.to_string());
                    }
                }
            }
        }
    }
    
    if let Some(iface) = current {
        if !iface.ipv4.is_empty() || !iface.ipv6.is_empty() {
            interfaces.push(iface);
        }
    }
    
    interfaces
}

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
    // ponytail: 中文 Windows PowerShell 输出为 GBK/CP936 编码，必须用 encoding_rs 解码
    // 不能用 String::from_utf8()，否则遇到中文设备名会失败
    let (text, _, _) = encoding_rs::GBK.decode(&output.stdout);
    Ok(text.into_owned())
}

fn run_powershell_json<T: for<'de> Deserialize<'de>>(script: &str) -> Result<Vec<T>, String> {
    let output = run_powershell(script)?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    let json_val: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("JSON 解析失败: {} - 输出: {}", e, &trimmed[..200.min(trimmed.len())]))?;
    match json_val {
        serde_json::Value::Array(_) => {
            serde_json::from_value(json_val).map_err(|e| e.to_string())
        }
        serde_json::Value::Object(_) => {
            Ok(serde_json::from_value(serde_json::Value::Array(vec![json_val]))
                .map_err(|e| e.to_string())?)
        }
        _ => Ok(vec![]),
    }
}

// ============ 进程 kill 辅助函数 ============

/// 解析 taskkill 命令输出，构造友好的 KillResult
/// ponytail: 依赖中文 Windows taskkill 输出关键词匹配，英文系统需额外适配
fn parse_taskkill_output(
    exit_code: i32,
    stdout: &str,
    stderr: &str,
    pid: u32,
    process_name: &str,
) -> KillResult {
    // taskkill 成功时 exit_code == 0
    if exit_code == 0 {
        let message = if process_name.is_empty() {
            format!("已结束 PID: {}", pid)
        } else {
            format!("已结束 {} (PID: {})", process_name, pid)
        };
        return KillResult {
            success: true,
            pid,
            process_name: process_name.to_string(),
            message,
        };
    }

    // 失败时合并 stdout + stderr 做关键词匹配
    let combined = format!("{}\n{}", stdout, stderr);

    let message = if combined.contains("拒绝访问") || combined.contains("Access is denied") {
        "拒绝访问，可能需要管理员权限".to_string()
    } else if combined.contains("没有找到") || combined.contains("找不到") || combined.contains("not found") {
        "进程不存在或已退出".to_string()
    } else {
        // 未知错误，返回原始输出（截断 200 字符避免过长）
        let raw = combined.trim();
        let truncated = if raw.len() > 200 { &raw[..200] } else { raw };
        format!("未知错误: {}", truncated)
    };

    KillResult {
        success: false,
        pid,
        process_name: process_name.to_string(),
        message,
    }
}

// ponytail: 解析 reg query 输出行，提取指定字段值
// 格式: "    DisplayName    REG_SZ    Value"
fn parse_reg_line(line: &str, field: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(field) {
        return None;
    }
    // 跳过字段名和类型（REG_SZ/REG_EXPAND_SZ 等），取剩余部分作为值
    let rest = &trimmed[field.len()..].trim();
    // 跳过 REG_XXX 类型标识
    let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].trim().to_string())
}

// ============ 命令实现（后续 Task 填充） ============

#[tauri::command]
pub fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = run_powershell("[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)");
        match output {
            Ok(s) => s.trim().to_lowercase() == "true",
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::{System, Disks};

    let mut sys = System::new_all();
    // CPU 使用率需要两次刷新间隔才有非零值
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_usage();

    let os_name = System::name().unwrap_or_default();
    let os_version = System::os_version().unwrap_or_default();
    let os_arch = std::env::consts::ARCH.to_string();
    let hostname = System::host_name().unwrap_or_default();
    let uptime_secs = System::uptime();

    // CPU 信息
    let cpus = sys.cpus();
    let cpu = CpuInfo {
        brand: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
        core_count: sys.physical_core_count().unwrap_or(0),
        thread_count: cpus.len(),
        frequency_mhz: cpus.first().map(|c| c.frequency()).unwrap_or(0),
        usage_percent: if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        },
    };

    // 内存信息
    let memory = MemoryInfo {
        total_bytes: sys.total_memory(),
        used_bytes: sys.used_memory(),
        available_bytes: sys.available_memory(),
    };

    // 磁盘信息
    let disks_list = Disks::new_with_refreshed_list();
    let disks: Vec<DiskInfo> = disks_list.list().iter().map(|disk| {
        let name = disk.name().to_string_lossy().to_string();
        let total = disk.total_space();
        let available = disk.available_space();
        DiskInfo {
            name,
            total_bytes: total,
            used_bytes: total.saturating_sub(available),
            file_system: disk.file_system().to_string_lossy().to_string(),
            is_removable: disk.is_removable(),
        }
    }).collect();

    Ok(SystemInfo {
        os_name,
        os_version,
        os_arch,
        hostname,
        uptime_secs,
        cpu,
        memory,
        disks,
    })
}

#[tauri::command]
pub fn get_network_info() -> Result<NetworkInfo, String> {
    use sysinfo::System;

    let hostname = System::host_name().unwrap_or_default();

    #[derive(Deserialize)]
    struct CimAdapter {
        #[serde(rename = "Description")]
        description: String,
        #[serde(rename = "MACAddress")]
        mac_address: Option<String>,
        #[serde(rename = "IPAddress")]
        ip_address: Vec<String>,
    }

    let cim_adapters: Vec<CimAdapter> = match run_powershell_json(
        "Get-CimInstance -ClassName Win32_NetworkAdapterConfiguration -Filter 'IPEnabled=True' | Select-Object Description,MACAddress,IPAddress,DefaultIPGateway | ConvertTo-Json"
    ) {
        Ok(v) => {
            if v.is_empty() {
                debug_log!("Win32_NetworkAdapterConfiguration 返回空，尝试 ipconfig");
            }
            v
        }
        Err(e) => {
            debug_log!("Win32_NetworkAdapterConfiguration 查询失败: {}", e);
            Vec::new()
        }
    };

    let interfaces: Vec<NetInterface> = if !cim_adapters.is_empty() {
        cim_adapters.into_iter()
            .map(|adapter| {
                let mut ipv4 = Vec::new();
                let mut ipv6 = Vec::new();
                for ip in &adapter.ip_address {
                    if ip.contains(':') {
                        ipv6.push(ip.clone());
                    } else {
                        ipv4.push(ip.clone());
                    }
                }
                NetInterface {
                    name: adapter.description,
                    mac: adapter.mac_address.unwrap_or_default().replace(':', "-"),
                    ipv4,
                    ipv6,
                    status: "Up".to_string(),
                }
            })
            .collect()
    } else {
        get_interfaces_from_ipconfig()
    };

    // 默认网关 — ponytail: IPv4DefaultGateway 返回 WMI 对象字符串，改用 Get-NetRoute 直接取 NextHop
    #[derive(Deserialize)]
    struct PsGateway {
        #[serde(rename = "NextHop")]
        next_hop: String,
    }
    let gateways: Vec<PsGateway> = run_powershell_json(
        "Get-NetRoute -DestinationPrefix '0.0.0.0/0' -AddressFamily IPv4 | Select-Object InterfaceAlias,NextHop | ConvertTo-Json"
    ).unwrap_or_default();
    let default_gateway = gateways.iter()
        .find(|g| !g.next_hop.is_empty() && g.next_hop != "0.0.0.0")
        .map(|g| g.next_hop.clone())
        .unwrap_or_default();

    // DNS 服务器
    #[derive(Deserialize)]
    struct PsDns {
        #[serde(rename = "ServerAddresses")]
        server_addresses: Vec<String>,
    }
    let dns_entries: Vec<PsDns> = run_powershell_json(
        "Get-DnsClientServerAddress -AddressFamily IPv4 | Select-Object InterfaceAlias,ServerAddresses | ConvertTo-Json"
    ).unwrap_or_default();
    let dns_servers: Vec<String> = dns_entries.iter()
        .flat_map(|d| d.server_addresses.iter().cloned())
        .filter(|s| !s.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // WiFi 名称
    let wifi_name = run_powershell("(Get-NetConnectionProfile).Name")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // 活动连接和监听端口
    // ponytail: Get-NetTCPConnection.State 返回数字码，需要手动映射为字符串名称
    // MibTcpState: 1=Closed, 2=Listen, 3=SynSent, 4=SynReceived, 5=Established,
    //              6=FinWait1, 7=FinWait2, 8=CloseWait, 9=LastAck, 10=Closing,
    //              11=TimeWait, 12=DeleteTCB
    #[derive(Deserialize)]
    struct PsTcpConnection {
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

    fn tcp_state_name(code: u32) -> &'static str {
        match code {
            1 => "Closed",
            2 => "Listen",
            3 => "SynSent",
            4 => "SynReceived",
            5 => "Established",
            6 => "FinWait1",
            7 => "FinWait2",
            8 => "CloseWait",
            9 => "LastAck",
            10 => "Closing",
            11 => "TimeWait",
            12 => "DeleteTCB",
            _ => "Unknown",
        }
    }

    let tcp_connections: Vec<PsTcpConnection> = run_powershell_json(
        "Get-NetTCPConnection | Select-Object State,LocalAddress,LocalPort,RemoteAddress,RemotePort,OwningProcess | ConvertTo-Json"
    ).unwrap_or_default();

    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut active_connections = Vec::new();
    let mut listening_ports = Vec::new();

    for conn in &tcp_connections {
        let local_addr = format!("{}:{}", conn.local_address, conn.local_port);
        let remote_addr = format!("{}:{}", conn.remote_address, conn.remote_port);
        let pid = conn.owning_process;
        let state_name = tcp_state_name(conn.state);

        if conn.state == 2 {
            // Listen
            let process_name = sys.process(sysinfo::Pid::from_u32(pid))
                .map(|p| p.name().to_string_lossy().to_string())
                .unwrap_or_default();
            listening_ports.push(ListeningPort {
                protocol: "TCP".to_string(),
                local_addr,
                pid,
                process_name,
            });
        } else {
            active_connections.push(TcpConnection {
                protocol: "TCP".to_string(),
                local_addr,
                remote_addr,
                state: state_name.to_string(),
                pid,
            });
        }
    }

    Ok(NetworkInfo {
        hostname,
        interfaces,
        default_gateway,
        dns_servers,
        wifi_name,
        active_connections,
        listening_ports,
    })
}

#[tauri::command]
pub fn get_process_list() -> Result<Vec<ProcessItem>, String> {
    use sysinfo::System;

    let mut sys = System::new_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut processes: Vec<ProcessItem> = sys.processes().iter().map(|(pid, process)| {
        let status = match process.status() {
            sysinfo::ProcessStatus::Run => "Run",
            sysinfo::ProcessStatus::Sleep => "Sleep",
            sysinfo::ProcessStatus::Idle => "Idle",
            _ => "Unknown",
        };
        ProcessItem {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            cpu_usage: process.cpu_usage(),
            memory_bytes: process.memory(),
            status: status.to_string(),
            command: process.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        }
    }).collect();

    // 按 CPU 使用率降序排列
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

    Ok(processes)
}

#[tauri::command]
pub fn get_hardware_info() -> Result<HardwareInfo, String> {
    // === CPU 信息 ===
    #[derive(Deserialize)]
    struct PsCpu {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "NumberOfCores")]
        number_of_cores: Option<u32>,
        #[serde(rename = "NumberOfLogicalProcessors")]
        number_of_logical_processors: Option<u32>,
        #[serde(rename = "MaxClockSpeed")]
        max_clock_speed: Option<u32>,
    }
    let cpus_raw: Vec<PsCpu> = run_powershell_json(
        "Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed | ConvertTo-Json"
    ).unwrap_or_default();
    let cpu = cpus_raw.first().map(|c| CpuSummary {
        name: c.name.clone().unwrap_or_default(),
        cores: c.number_of_cores.unwrap_or(0) as usize,
        threads: c.number_of_logical_processors.unwrap_or(0) as usize,
        frequency_mhz: c.max_clock_speed.unwrap_or(0) as u64,
    }).unwrap_or(CpuSummary {
        name: String::new(), cores: 0, threads: 0, frequency_mhz: 0,
    });

    // === 内存信息 ===
    #[derive(Deserialize)]
    struct PsMemory {
        #[serde(rename = "TotalPhysicalMemory")]
        total_physical_memory: Option<u64>,
    }
    #[derive(Deserialize)]
    struct PsOsMemory {
        #[serde(rename = "FreePhysicalMemory")]
        free_physical_memory: Option<u64>,
    }
    let mem_total: u64 = run_powershell_json::<PsMemory>(
        "Get-CimInstance Win32_ComputerSystem | Select-Object TotalPhysicalMemory | ConvertTo-Json"
    ).ok().and_then(|v| v.first().and_then(|m| m.total_physical_memory)).unwrap_or(0);
    let mem_free: u64 = run_powershell_json::<PsOsMemory>(
        "Get-CimInstance Win32_OperatingSystem | Select-Object FreePhysicalMemory | ConvertTo-Json"
    ).ok().and_then(|v| v.first().and_then(|m| m.free_physical_memory)).unwrap_or(0) * 1024;
    let mem_used = mem_total.saturating_sub(mem_free);
    let gb = 1024.0 * 1024.0 * 1024.0;
    let memory = MemorySummary {
        total_gb: mem_total as f64 / gb,
        used_gb: mem_used as f64 / gb,
        available_gb: mem_free as f64 / gb,
    };

    // === 磁盘信息 ===
    #[derive(Deserialize)]
    struct PsDisk {
        #[serde(rename = "DeviceID")]
        device_id: Option<String>,
        #[serde(rename = "Model")]
        model: Option<String>,
        #[serde(rename = "Size")]
        size: Option<u64>,
        #[serde(rename = "MediaType")]
        media_type: Option<String>,
    }
    let disks_raw: Vec<PsDisk> = run_powershell_json(
        "Get-CimInstance Win32_DiskDrive | Select-Object DeviceID,Model,Size,MediaType | ConvertTo-Json"
    ).unwrap_or_default();
    let disks: Vec<DiskSummary> = disks_raw.into_iter().map(|d| {
        let size_gb = d.size.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
        DiskSummary {
            name: d.device_id.unwrap_or_default(),
            model: d.model.unwrap_or_default(),
            size_gb,
            free_gb: 0.0,
            fs_type: d.media_type.unwrap_or_default(),
        }
    }).collect();

    // === GPU 信息 ===
    #[derive(Deserialize)]
    struct PsGpu {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "DriverVersion")]
        driver_version: Option<String>,
        #[serde(rename = "AdapterRAM")]
        adapter_ram: Option<u64>,
    }
    let gpus_raw: Vec<PsGpu> = run_powershell_json(
        "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM | ConvertTo-Json"
    ).unwrap_or_default();
    let gpus: Vec<GpuInfo> = gpus_raw.iter().map(|g| {
        // ponytail: AdapterRAM 是 u32，超过 4GB 显存会溢出，此时显示 0
        let vram_mb = g.adapter_ram.unwrap_or(0) / 1024 / 1024;
        GpuInfo {
            name: g.name.clone().unwrap_or_default(),
            driver_version: g.driver_version.clone().unwrap_or_default(),
            vram_mb,
        }
    }).collect();

    // === 显示器信息 ===
    #[derive(Deserialize)]
    struct PsMonitor {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "ScreenWidth")]
        screen_width: Option<u32>,
        #[serde(rename = "ScreenHeight")]
        screen_height: Option<u32>,
    }
    let monitors_raw: Vec<PsMonitor> = run_powershell_json(
        "Get-CimInstance Win32_DesktopMonitor | Select-Object Name,ScreenWidth,ScreenHeight | ConvertTo-Json"
    ).unwrap_or_default();
    
    let mut displays: Vec<DisplayInfo> = monitors_raw.iter().filter_map(|m| {
        match (m.screen_width, m.screen_height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => {
                Some(DisplayInfo {
                    name: m.name.clone().unwrap_or_else(|| "显示器".to_string()),
                    resolution: format!("{}x{}", w, h),
                })
            }
            _ => None,
        }
    }).collect();
    
    if displays.is_empty() {
        #[derive(Deserialize)]
        struct PsVideoCurrentMode {
            #[serde(rename = "Name")]
            name: Option<String>,
            #[serde(rename = "CurrentHorizontalResolution")]
            width: Option<u32>,
            #[serde(rename = "CurrentVerticalResolution")]
            height: Option<u32>,
        }
        let video_modes: Vec<PsVideoCurrentMode> = run_powershell_json(
            "Get-CimInstance Win32_VideoController | Where-Object CurrentHorizontalResolution -ne $null | Select-Object Name,CurrentHorizontalResolution,CurrentVerticalResolution | ConvertTo-Json"
        ).unwrap_or_default();
        displays = video_modes.iter().map(|m| {
            DisplayInfo {
                name: m.name.clone().unwrap_or_else(|| "显示器".to_string()),
                resolution: match (m.width, m.height) {
                    (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
                    _ => String::new(),
                },
            }
        }).collect();
    }

    // === 音频设备 ===
    // ponytail: Where-Object 在 Tauri 子进程沙箱中可能失效，改用 WMI -Filter (WQL) 在服务端过滤
    #[derive(Deserialize)]
    struct PsAudio {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "Status")]
        status: Option<String>,
    }
    let audio_raw: Vec<PsAudio> = match run_powershell_json(
        "Get-CimInstance Win32_SoundDevice | Select-Object Name,Status | ConvertTo-Json"
    ) {
        Ok(v) => v,
        Err(e) => {
            debug_log!("音频设备查询失败: {}", e);
            Vec::new()
        }
    };
    
    let mut audio_devices: Vec<AudioDevice> = audio_raw.iter().map(|a| AudioDevice {
        name: a.name.clone().unwrap_or_default(),
        status: a.status.clone().unwrap_or_default(),
    }).collect();
    
    if audio_devices.is_empty() {
        let pnp_audio: Vec<PsAudio> = match run_powershell_json(
            "Get-CimInstance Win32_PnPEntity -Filter \"PNPClass='MEDIA'\" | Select-Object Name,Status | ConvertTo-Json"
        ) {
            Ok(v) => v,
            Err(e) => {
                debug_log!("音频设备 PnP 查询失败: {}", e);
                Vec::new()
            }
        };
        audio_devices = pnp_audio.iter().map(|a| AudioDevice {
            name: a.name.clone().unwrap_or_default(),
            status: a.status.clone().unwrap_or_default(),
        }).collect();
    }

    // === 主板信息 ===
    #[derive(Deserialize)]
    struct PsMotherboard {
        #[serde(rename = "Manufacturer")]
        manufacturer: Option<String>,
        #[serde(rename = "Product")]
        product: Option<String>,
        #[serde(rename = "SerialNumber")]
        serial_number: Option<String>,
    }
    let mb_raw: Vec<PsMotherboard> = run_powershell_json(
        "Get-CimInstance Win32_BaseBoard | Select-Object Manufacturer,Product,SerialNumber | ConvertTo-Json"
    ).unwrap_or_default();
    let motherboard = mb_raw.first().map(|m| MotherboardInfo {
        manufacturer: m.manufacturer.clone().unwrap_or_default(),
        product: m.product.clone().unwrap_or_default(),
        serial: m.serial_number.clone().unwrap_or_default(),
    }).unwrap_or(MotherboardInfo {
        manufacturer: String::new(), product: String::new(), serial: String::new(),
    });

    // === 电池信息（仅笔记本有） ===
    #[derive(Deserialize)]
    struct PsBattery {
        #[serde(rename = "Status")]
        status: Option<String>,
        #[serde(rename = "EstimatedChargeRemaining")]
        estimated_charge_remaining: Option<u32>,
        #[serde(rename = "EstimatedRunTime")]
        estimated_run_time: Option<u32>,
    }
    let battery_raw: Vec<PsBattery> = run_powershell_json(
        "Get-CimInstance Win32_Battery | Select-Object Status,EstimatedChargeRemaining,EstimatedRunTime | ConvertTo-Json"
    ).unwrap_or_default();
    let battery = battery_raw.first().map(|b| {
        let time_str = match b.estimated_run_time {
            Some(mins) if mins > 0 => format!("{} 分钟", mins),
            _ => "未知".to_string(),
        };
        BatteryInfo {
            status: b.status.clone().unwrap_or_default(),
            charge_percent: b.estimated_charge_remaining.unwrap_or(0),
            estimated_time: time_str,
        }
    });

    // === USB 设备 ===
    // ponytail: Where-Object 在 Tauri 子进程沙箱中可能失效，改用 WMI -Filter (WQL) 在服务端过滤
    #[derive(Deserialize)]
    struct PsUsb {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "DeviceID")]
        device_id: Option<String>,
    }
    let usb_raw: Vec<PsUsb> = match run_powershell_json(
        "Get-CimInstance Win32_PnPEntity -Filter \"DeviceID LIKE '%USB%'\" | Select-Object Name,DeviceID | ConvertTo-Json"
    ) {
        Ok(v) => v,
        Err(e) => {
            debug_log!("USB 查询失败: {}", e);
            Vec::new()
        }
    };
    
    let usb_devices: Vec<UsbDevice> = usb_raw.into_iter().filter(|u| {
        u.name.as_ref().map_or(false, |n| !n.is_empty())
    }).map(|u| UsbDevice {
        name: u.name.unwrap_or_default(),
        device_id: u.device_id.unwrap_or_default(),
    }).collect();

    Ok(HardwareInfo {
        cpu,
        memory,
        disks,
        gpus,
        displays,
        audio_devices,
        motherboard,
        battery,
        usb_devices,
    })
}

#[tauri::command]
pub fn get_software_env() -> Result<SoftwareEnv, String> {
    // ponytail: PowerShell 注册表访问在子进程中可能失败，改用 reg query 直接读取注册表
    // reg query 输出为 OEM 编码（中文 Windows 为 CP936），需用 encoding_rs 解码
    let mut installed_software: Vec<SoftwareItem> = Vec::new();
    for reg_path in &["HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"] {
        let mut cmd = Command::new("reg");
        cmd.args(["query", reg_path, "/s"]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let output = cmd.output();
        if let Ok(out) = output {
            // ponytail: OEM 编码解码，中文 Windows 为 CP936
            let (text, _, _) = encoding_rs::GBK.decode(&out.stdout);
            let mut current = SoftwareItem::default();
            let mut has_display_name = false;
            for line in text.lines() {
                if line.starts_with("HKEY_") {
                    if has_display_name {
                        installed_software.push(current);
                    }
                    current = SoftwareItem::default();
                    has_display_name = false;
                } else if let Some(val) = parse_reg_line(line, "DisplayName") {
                    current.name = val;
                    has_display_name = true;
                } else if let Some(val) = parse_reg_line(line, "DisplayVersion") {
                    current.version = val;
                } else if let Some(val) = parse_reg_line(line, "Publisher") {
                    current.publisher = val;
                } else if let Some(val) = parse_reg_line(line, "InstallDate") {
                    current.install_date = val;
                }
            }
            if has_display_name {
                installed_software.push(current);
            }
        }
    }

    // 环境变量
    let environment_variables: Vec<EnvVar> = std::env::vars().map(|(key, value)| EnvVar { key, value }).collect();

    // 启动项
    #[derive(Deserialize)]
    struct PsStartup {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "Command")]
        command: Option<String>,
        #[serde(rename = "Location")]
        location: Option<String>,
    }
    let startup_raw: Vec<PsStartup> = run_powershell_json(
        r#"Get-CimInstance Win32_StartupCommand | Select-Object Name,Command,Location | ConvertTo-Json"#
    ).unwrap_or_default();
    let startup_items: Vec<StartupItem> = startup_raw.iter().map(|s| StartupItem {
        name: s.name.clone().unwrap_or_default(),
        command: s.command.clone().unwrap_or_default(),
        location: s.location.clone().unwrap_or_default(),
    }).collect();

    Ok(SoftwareEnv {
        installed_software,
        environment_variables,
        startup_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_taskkill_success() {
        let r = parse_taskkill_output(0, "成功: 已终止 PID 为 1234 的进程。", "", 1234, "notepad.exe");
        assert!(r.success);
        assert_eq!(r.pid, 1234);
        assert_eq!(r.process_name, "notepad.exe");
        assert!(r.message.contains("notepad.exe"));
        assert!(r.message.contains("1234"));
    }

    #[test]
    fn parse_taskkill_success_without_name() {
        let r = parse_taskkill_output(0, "成功: 已终止 PID 为 1234 的进程。", "", 1234, "");
        assert!(r.success);
        assert_eq!(r.process_name, "");
        assert!(r.message.contains("1234"));
        assert!(!r.message.contains("notepad"));
    }

    #[test]
    fn parse_taskkill_access_denied() {
        let r = parse_taskkill_output(1, "", "错误: 无法终止 PID 1234 的进程。拒绝访问。", 1234, "");
        assert!(!r.success);
        assert!(r.message.contains("管理员"));
    }

    #[test]
    fn parse_taskkill_not_found() {
        let r = parse_taskkill_output(128, "", "错误: 没有找到进程 \"9999\"。", 9999, "");
        assert!(!r.success);
        assert!(r.message.contains("不存在"));
    }

    #[test]
    fn parse_taskkill_unknown_error() {
        let r = parse_taskkill_output(1, "", "未知错误输出内容", 1234, "");
        assert!(!r.success);
        assert!(r.message.contains("未知错误"));
    }
}
