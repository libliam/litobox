use serde::{Deserialize, Serialize};
use std::process::Command;

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

#[derive(Serialize)]
pub struct HardwareInfo {
    pub gpus: Vec<GpuInfo>,
    pub displays: Vec<DisplayInfo>,
    pub audio_devices: Vec<AudioDevice>,
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
pub struct SoftwareEnv {
    pub installed_software: Vec<SoftwareItem>,
    pub environment_variables: Vec<EnvVar>,
    pub startup_items: Vec<StartupItem>,
}

#[derive(Serialize)]
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

fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("PowerShell 执行失败: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell 错误: {}", stderr));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("UTF-8 转换失败: {}", e))
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

// ============ 命令实现（后续 Task 填充） ============

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
        brand: cpus.first().map(|c| c.brand_name().to_string()).unwrap_or_default(),
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

    // 网络接口（通过 PowerShell 获取 IP 和 MAC）
    #[derive(Deserialize)]
    struct PsNetAdapter {
        #[serde(rename = "InterfaceAlias")]
        interface_alias: String,
        #[serde(rename = "MacAddress")]
        mac_address: String,
        #[serde(rename = "Status")]
        status: String,
    }
    #[derive(Deserialize)]
    struct PsNetIpAddress {
        #[serde(rename = "InterfaceAlias")]
        interface_alias: String,
        #[serde(rename = "IPAddress")]
        ip_address: String,
        #[serde(rename = "AddressFamily")]
        address_family: String,
    }

    let adapters: Vec<PsNetAdapter> = run_powershell_json(
        "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | Select-Object InterfaceAlias,MacAddress,Status | ConvertTo-Json"
    ).unwrap_or_default();

    let ip_addresses: Vec<PsNetIpAddress> = run_powershell_json(
        "Get-NetIPAddress -AddressFamily IPv4,IPv6 | Where-Object { $_.PrefixOrigin -ne 'WellKnown' } | Select-Object InterfaceAlias,IPAddress,AddressFamily | ConvertTo-Json"
    ).unwrap_or_default();

    let interfaces: Vec<NetInterface> = adapters.iter().map(|adapter| {
        let mut ipv4 = Vec::new();
        let mut ipv6 = Vec::new();
        for ip in &ip_addresses {
            if ip.interface_alias == adapter.interface_alias {
                if ip.address_family == "IPv4" {
                    ipv4.push(ip.ip_address.clone());
                } else if ip.address_family == "IPv6" {
                    ipv6.push(ip.ip_address.clone());
                }
            }
        }
        NetInterface {
            name: adapter.interface_alias.clone(),
            mac: adapter.mac_address.clone(),
            ipv4,
            ipv6,
            status: adapter.status.clone(),
        }
    }).collect();

    // 默认网关
    #[derive(Deserialize)]
    struct PsGateway {
        #[serde(rename = "InterfaceAlias")]
        interface_alias: String,
        #[serde(rename = "IPv4DefaultGateway")]
        ipv4_default_gateway: Option<serde_json::Value>,
    }
    let gateways: Vec<PsGateway> = run_powershell_json(
        "Get-NetIPConfiguration | Select-Object InterfaceAlias,IPv4DefaultGateway | ConvertTo-Json"
    ).unwrap_or_default();
    let default_gateway = gateways.iter()
        .find_map(|g| {
            g.ipv4_default_gateway.as_ref().and_then(|v| {
                v.get("NextHop").and_then(|nh| nh.as_str()).map(|s| s.to_string())
            })
        })
        .unwrap_or_default();

    // DNS 服务器
    #[derive(Deserialize)]
    struct PsDns {
        #[serde(rename = "InterfaceAlias")]
        interface_alias: String,
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
    #[derive(Deserialize)]
    struct PsTcpConnection {
        #[serde(rename = "State")]
        state: String,
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

    let tcp_connections: Vec<PsTcpConnection> = run_powershell_json(
        "Get-NetTCPConnection | Select-Object State,LocalAddress,LocalPort,RemoteAddress,RemotePort,OwningProcess | ConvertTo-Json"
    ).unwrap_or_default();

    let mut sys = System::new();
    sys.refresh_processes();

    let mut active_connections = Vec::new();
    let mut listening_ports = Vec::new();

    for conn in &tcp_connections {
        let local_addr = format!("{}:{}", conn.local_address, conn.local_port);
        let remote_addr = format!("{}:{}", conn.remote_address, conn.remote_port);
        let pid = conn.owning_process;

        if conn.state == "Listen" {
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
                state: conn.state.clone(),
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
    sys.refresh_processes();

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
    // GPU 信息
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

    // 显示器信息
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
    let displays: Vec<DisplayInfo> = monitors_raw.iter().map(|m| {
        let resolution = match (m.screen_width, m.screen_height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => format!("{}x{}", w, h),
            _ => String::new(),
        };
        DisplayInfo {
            name: m.name.clone().unwrap_or_else(|| "显示器".to_string()),
            resolution,
        }
    }).collect();

    // 音频设备
    #[derive(Deserialize)]
    struct PsAudio {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "Status")]
        status: Option<String>,
    }
    let audio_raw: Vec<PsAudio> = run_powershell_json(
        "Get-CimInstance Win32_SoundDevice | Select-Object Name,Status | ConvertTo-Json"
    ).unwrap_or_default();
    let audio_devices: Vec<AudioDevice> = audio_raw.iter().map(|a| AudioDevice {
        name: a.name.clone().unwrap_or_default(),
        status: a.status.clone().unwrap_or_default(),
    }).collect();

    Ok(HardwareInfo {
        gpus,
        displays,
        audio_devices,
    })
}

#[tauri::command]
pub fn get_software_env() -> Result<SoftwareEnv, String> {
    // 已安装软件（查询注册表卸载列表）
    #[derive(Deserialize)]
    struct PsSoftware {
        #[serde(rename = "DisplayName")]
        display_name: Option<String>,
        #[serde(rename = "DisplayVersion")]
        display_version: Option<String>,
        #[serde(rename = "Publisher")]
        publisher: Option<String>,
        #[serde(rename = "InstallDate")]
        install_date: Option<String>,
    }
    let software_raw: Vec<PsSoftware> = run_powershell_json(
        r#"$paths = @(
            'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
            'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
        )
        Get-ItemProperty $paths | Where-Object { $_.DisplayName } | Select-Object DisplayName,DisplayVersion,Publisher,InstallDate | ConvertTo-Json"#
    ).unwrap_or_default();
    let installed_software: Vec<SoftwareItem> = software_raw.iter().map(|s| SoftwareItem {
        name: s.display_name.clone().unwrap_or_default(),
        version: s.display_version.clone().unwrap_or_default(),
        publisher: s.publisher.clone().unwrap_or_default(),
        install_date: s.install_date.clone().unwrap_or_default(),
    }).collect();

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
