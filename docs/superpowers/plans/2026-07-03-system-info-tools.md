# 系统信息工具集 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 新增 5 个系统级信息展示工具（系统信息、网络信息、进程列表、硬件外设、软件环境）

**Architecture:** Rust 后端新增 `system_info.rs` 单模块，使用 `sysinfo` crate + PowerShell 命令采集数据，通过 5 个 Tauri 命令暴露给前端。前端新增 5 个 Vue 视图 + 1 个 TypeScript 调用层。

**Tech Stack:** Rust (sysinfo 0.31+, serde, serde_json), Vue 3 (Composition API), TypeScript, Element Plus, Tauri 2.0

---

## 文件结构

### 新增文件

| 文件 | 职责 |
|------|------|
| `src-tauri/src/system_info.rs` | Rust 后端：5 个 Tauri 命令 + 数据结构 + PowerShell 辅助函数 |
| `src/utils/systemInfoClient.ts` | 前端调用层：TypeScript 类型定义 + invoke 封装 + 格式化工具函数 |
| `src/views/SystemInfoView.vue` | 系统信息页面（OS/CPU/内存/磁盘） |
| `src/views/NetworkInfoView.vue` | 网络信息页面（接口/连接/端口/WiFi） |
| `src/views/ProcessListView.vue` | 进程列表页面（搜索/排序/表格） |
| `src/views/HardwareInfoView.vue` | 硬件外设页面（GPU/显示器/音频） |
| `src/views/SoftwareEnvView.vue` | 软件环境页面（已装软件/环境变量/启动项） |

### 修改文件

| 文件 | 改动 |
|------|------|
| `src-tauri/Cargo.toml` | 新增 `sysinfo` 依赖 |
| `src-tauri/src/main.rs` | `mod system_info;` + 注册 5 个命令 |
| `src/store/index.ts` | `TOOL_LIST` 新增 5 个工具项 |
| `src/components/SidebarNav.vue` | `categoryNames` 新增 `system` |
| `src/App.vue` | 新增 5 个 `v-else-if` 分支 + import |

---

## Task 1: 后端基础 — Cargo.toml + system_info.rs 骨架 + main.rs 注册

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/system_info.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 Cargo.toml 添加 sysinfo 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 末尾添加：

```toml
sysinfo = "0.31"
```

- [ ] **Step 2: 创建 system_info.rs 骨架（结构体 + 辅助函数）**

创建 `src-tauri/src/system_info.rs`，包含所有数据结构定义和 PowerShell 辅助函数：

```rust
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
    Err("尚未实现".to_string())
}

#[tauri::command]
pub fn get_network_info() -> Result<NetworkInfo, String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub fn get_process_list() -> Result<Vec<ProcessItem>, String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub fn get_hardware_info() -> Result<HardwareInfo, String> {
    Err("尚未实现".to_string())
}

#[tauri::command]
pub fn get_software_env() -> Result<SoftwareEnv, String> {
    Err("尚未实现".to_string())
}
```

- [ ] **Step 3: 在 main.rs 注册模块和命令**

在 `src-tauri/src/main.rs` 的 `mod` 声明区（第 3-9 行附近）添加：

```rust
mod system_info;
```

在 `invoke_handler` 宏的命令列表末尾（第 91 行 `db::cmd_db_register_shortcuts,` 之后）添加：

```rust
            system_info::get_system_info,
            system_info::get_network_info,
            system_info::get_process_list,
            system_info::get_hardware_info,
            system_info::get_software_env,
```

- [ ] **Step 4: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过（首次会下载 sysinfo crate，可能需要几分钟）

- [ ] **Step 5: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/system_info.rs src-tauri/src/main.rs
git commit -m "feat: 添加 system_info 后端模块骨架和 sysinfo 依赖"
```

---

## Task 2: 实现 get_system_info 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs`

- [ ] **Step 1: 实现 get_system_info 函数**

将 `system_info.rs` 中的 `get_system_info` 占位函数替换为：

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat: 实现 get_system_info 命令（OS/CPU/内存/磁盘）"
```

---

## Task 3: 实现 get_network_info 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs`

- [ ] **Step 1: 实现 get_network_info 函数**

将 `get_network_info` 占位函数替换为：

```rust
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

    // 活动连接和监听端口（通过 PowerShell Get-NetTCPConnection）
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat: 实现 get_network_info 命令（网络接口/连接/端口/WiFi）"
```

---

## Task 4: 实现 get_process_list 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs`

- [ ] **Step 1: 实现 get_process_list 函数**

将 `get_process_list` 占位函数替换为：

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat: 实现 get_process_list 命令"
```

---

## Task 5: 实现 get_hardware_info 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs`

- [ ] **Step 1: 实现 get_hardware_info 函数**

将 `get_hardware_info` 占位函数替换为：

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat: 实现 get_hardware_info 命令（GPU/显示器/音频）"
```

---

## Task 6: 实现 get_software_env 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs`

- [ ] **Step 1: 实现 get_software_env 函数**

将 `get_software_env` 占位函数替换为：

```rust
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
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat: 实现 get_software_env 命令（已装软件/环境变量/启动项）"
```

---

## Task 7: 前端调用层 — systemInfoClient.ts

**Files:**
- Create: `src/utils/systemInfoClient.ts`

- [ ] **Step 1: 创建 systemInfoClient.ts**

```typescript
import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface SystemInfo {
  os_name: string
  os_version: string
  os_arch: string
  hostname: string
  uptime_secs: number
  cpu: CpuInfo
  memory: MemoryInfo
  disks: DiskInfo[]
}

export interface CpuInfo {
  brand: string
  core_count: number
  thread_count: number
  frequency_mhz: number
  usage_percent: number
}

export interface MemoryInfo {
  total_bytes: number
  used_bytes: number
  available_bytes: number
}

export interface DiskInfo {
  name: string
  total_bytes: number
  used_bytes: number
  file_system: string
  is_removable: boolean
}

export interface NetworkInfo {
  hostname: string
  interfaces: NetInterface[]
  default_gateway: string
  dns_servers: string[]
  wifi_name: string | null
  active_connections: TcpConnection[]
  listening_ports: ListeningPort[]
}

export interface NetInterface {
  name: string
  mac: string
  ipv4: string[]
  ipv6: string[]
  status: string
}

export interface TcpConnection {
  protocol: string
  local_addr: string
  remote_addr: string
  state: string
  pid: number
}

export interface ListeningPort {
  protocol: string
  local_addr: string
  pid: number
  process_name: string
}

export interface ProcessItem {
  pid: number
  name: string
  cpu_usage: number
  memory_bytes: number
  status: string
  command: string
}

export interface HardwareInfo {
  gpus: GpuInfo[]
  displays: DisplayInfo[]
  audio_devices: AudioDevice[]
}

export interface GpuInfo {
  name: string
  driver_version: string
  vram_mb: number
}

export interface DisplayInfo {
  name: string
  resolution: string
}

export interface AudioDevice {
  name: string
  status: string
}

export interface SoftwareEnv {
  installed_software: SoftwareItem[]
  environment_variables: EnvVar[]
  startup_items: StartupItem[]
}

export interface SoftwareItem {
  name: string
  version: string
  publisher: string
  install_date: string
}

export interface EnvVar {
  key: string
  value: string
}

export interface StartupItem {
  name: string
  command: string
  location: string
}

// ============ invoke 封装 ============

export function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info')
}

export function getNetworkInfo(): Promise<NetworkInfo> {
  return invoke<NetworkInfo>('get_network_info')
}

export function getProcessList(): Promise<ProcessItem[]> {
  return invoke<ProcessItem[]>('get_process_list')
}

export function getHardwareInfo(): Promise<HardwareInfo> {
  return invoke<HardwareInfo>('get_hardware_info')
}

export function getSoftwareEnv(): Promise<SoftwareEnv> {
  return invoke<SoftwareEnv>('get_software_env')
}

// ============ 格式化工具函数 ============

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const idx = Math.min(i, units.length - 1)
  return (bytes / Math.pow(1024, idx)).toFixed(idx === 0 ? 0 : 1) + ' ' + units[idx]
}

export function formatUptime(secs: number): string {
  const days = Math.floor(secs / 86400)
  const hours = Math.floor((secs % 86400) / 3600)
  const mins = Math.floor((secs % 3600) / 60)
  const parts: string[] = []
  if (days > 0) parts.push(`${days}天`)
  if (hours > 0) parts.push(`${hours}小时`)
  parts.push(`${mins}分钟`)
  return parts.join('')
}

export function formatTimestamp(): string {
  const d = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

// ============ 自检 ============
// ponytail: 纯函数自检，确保格式化逻辑正确
console.assert(formatBytes(0) === '0 B', 'formatBytes(0)')
console.assert(formatBytes(1024) === '1.0 KB', 'formatBytes(1024)')
console.assert(formatBytes(1073741824) === '1.0 GB', 'formatBytes(1GB)')
console.assert(formatUptime(3661) === '1小时1分钟', 'formatUptime(3661)')
console.assert(formatUptime(90061) === '1天1小时1分钟', 'formatUptime(90061)')
```

- [ ] **Step 2: 验证编译**

Run: `npx tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 3: 提交**

```bash
git add src/utils/systemInfoClient.ts
git commit -m "feat: 添加 systemInfoClient 前端调用层和类型定义"
```

---

## Task 8: SystemInfoView.vue — 系统信息页面

**Files:**
- Create: `src/views/SystemInfoView.vue`

- [ ] **Step 1: 创建 SystemInfoView.vue**

```vue
<template>
  <div class="tool-container">
    <!-- 操作栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">系统信息</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ error }}</div>
      </div>
    </div>

    <template v-if="data">
      <!-- 操作系统 + CPU -->
      <div class="info-row">
        <div class="tool-card">
          <div class="card-header"><span class="card-title">操作系统</span></div>
          <div class="card-body">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">系统</span><span class="kv-value">{{ data.os_name }}</span></div>
              <div class="kv-item"><span class="kv-label">版本</span><span class="kv-value">{{ data.os_version }}</span></div>
              <div class="kv-item"><span class="kv-label">架构</span><span class="kv-value">{{ data.os_arch }}</span></div>
              <div class="kv-item"><span class="kv-label">主机名</span><span class="kv-value">{{ data.hostname }}</span></div>
              <div class="kv-item"><span class="kv-label">运行时长</span><span class="kv-value">{{ formatUptime(data.uptime_secs) }}</span></div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header"><span class="card-title">CPU</span></div>
          <div class="card-body">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">型号</span><span class="kv-value">{{ data.cpu.brand }}</span></div>
              <div class="kv-item"><span class="kv-label">物理核心</span><span class="kv-value">{{ data.cpu.core_count }}</span></div>
              <div class="kv-item"><span class="kv-label">逻辑线程</span><span class="kv-value">{{ data.cpu.thread_count }}</span></div>
              <div class="kv-item"><span class="kv-label">频率</span><span class="kv-value">{{ data.cpu.frequency_mhz }} MHz</span></div>
            </div>
            <div class="progress-row">
              <span class="progress-label">使用率</span>
              <el-progress :percentage="Math.round(data.cpu.usage_percent)" :stroke-width="10" />
            </div>
          </div>
        </div>
      </div>

      <!-- 内存 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">内存</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">总计</span><span class="kv-value">{{ formatBytes(data.memory.total_bytes) }}</span></div>
            <div class="kv-item"><span class="kv-label">已用</span><span class="kv-value">{{ formatBytes(data.memory.used_bytes) }}</span></div>
            <div class="kv-item"><span class="kv-label">可用</span><span class="kv-value">{{ formatBytes(data.memory.available_bytes) }}</span></div>
          </div>
          <div class="progress-row">
            <span class="progress-label">使用率</span>
            <el-progress :percentage="Math.round(data.memory.used_bytes / data.memory.total_bytes * 100)" :stroke-width="10" />
          </div>
        </div>
      </div>

      <!-- 磁盘 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">磁盘</span></div>
        <div class="card-body">
          <el-table :data="data.disks" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" width="100" />
            <el-table-column label="总量">
              <template #default="{ row }">{{ formatBytes(row.total_bytes) }}</template>
            </el-table-column>
            <el-table-column label="已用">
              <template #default="{ row }">{{ formatBytes(row.used_bytes) }}</template>
            </el-table-column>
            <el-table-column label="使用率" width="180">
              <template #default="{ row }">
                <el-progress :percentage="Math.round(row.used_bytes / row.total_bytes * 100)" :stroke-width="8" />
              </template>
            </el-table-column>
            <el-table-column prop="file_system" label="文件系统" width="100" />
            <el-table-column label="类型" width="80">
              <template #default="{ row }">{{ row.is_removable ? '可移动' : '固定' }}</template>
            </el-table-column>
          </el-table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getSystemInfo, formatBytes, formatUptime, formatTimestamp, type SystemInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<SystemInfo | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getSystemInfo()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'systemInfo',
      action: '查看系统信息',
      inputPreview: '',
      outputPreview: `${data.value.cpu.brand} | ${formatBytes(data.value.memory.total_bytes)}`,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 12px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.info-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }
.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.progress-row { display: flex; align-items: center; gap: 12px; margin-top: 16px; }
.progress-label { font-size: 13px; color: var(--text-secondary); white-space: nowrap; min-width: 50px; }
.error-message {
  padding: 12px; background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red); border-radius: 4px;
  color: var(--accent-red); font-size: 13px;
}
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
:deep(.el-progress__text) { color: var(--text-primary); }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/SystemInfoView.vue
git commit -m "feat: 添加系统信息页面"
```

---

## Task 9: NetworkInfoView.vue — 网络信息页面

**Files:**
- Create: `src/views/NetworkInfoView.vue`

- [ ] **Step 1: 创建 NetworkInfoView.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">网络信息</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <template v-if="data">
      <!-- 概览 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">概览</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">主机名</span><span class="kv-value">{{ data.hostname }}</span></div>
            <div class="kv-item"><span class="kv-label">WiFi</span><span class="kv-value">{{ data.wifi_name || '未连接' }}</span></div>
            <div class="kv-item"><span class="kv-label">默认网关</span><span class="kv-value">{{ data.default_gateway || '—' }}</span></div>
            <div class="kv-item"><span class="kv-label">DNS</span><span class="kv-value">{{ data.dns_servers.join(', ') || '—' }}</span></div>
          </div>
        </div>
      </div>

      <!-- 网络接口 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">网络接口</span></div>
        <div class="card-body">
          <el-table :data="data.interfaces" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" min-width="120" />
            <el-table-column prop="mac" label="MAC 地址" width="160" />
            <el-table-column label="IPv4" min-width="140">
              <template #default="{ row }">{{ row.ipv4.join(', ') || '—' }}</template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="80" />
          </el-table>
        </div>
      </div>

      <!-- 活动连接 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">活动连接 ({{ data.active_connections.length }})</span></div>
        <div class="card-body">
          <el-table :data="data.active_connections" border size="small" max-height="400" style="width: 100%">
            <el-table-column prop="protocol" label="协议" width="60" />
            <el-table-column prop="local_addr" label="本地地址" min-width="160" />
            <el-table-column prop="remote_addr" label="远程地址" min-width="160" />
            <el-table-column prop="state" label="状态" width="100" />
            <el-table-column prop="pid" label="PID" width="70" />
          </el-table>
        </div>
      </div>

      <!-- 监听端口 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">监听端口 ({{ data.listening_ports.length }})</span></div>
        <div class="card-body">
          <el-table :data="data.listening_ports" border size="small" max-height="400" style="width: 100%">
            <el-table-column prop="protocol" label="协议" width="60" />
            <el-table-column prop="local_addr" label="地址" min-width="160" />
            <el-table-column prop="pid" label="PID" width="70" />
            <el-table-column prop="process_name" label="进程" min-width="120" />
          </el-table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getNetworkInfo, formatTimestamp, type NetworkInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<NetworkInfo | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getNetworkInfo()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'networkInfo',
      action: '查看网络信息',
      inputPreview: '',
      outputPreview: `${data.value.interfaces.length} 个接口 | ${data.value.listening_ports.length} 个监听端口`,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => { loadData() })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 12px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/NetworkInfoView.vue
git commit -m "feat: 添加网络信息页面"
```

---

## Task 10: ProcessListView.vue — 进程列表页面

**Files:**
- Create: `src/views/ProcessListView.vue`

- [ ] **Step 1: 创建 ProcessListView.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">进程列表</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索进程名/PID..." style="width: 200px" clearable />
          <el-select v-model="sortBy" size="small" style="width: 120px">
            <el-option label="按 CPU 排序" value="cpu" />
            <el-option label="按内存排序" value="memory" />
          </el-select>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <div v-if="data" class="tool-card">
      <div class="card-header">
        <span class="card-title">进程 ({{ filteredData.length }} / {{ data.length }})</span>
      </div>
      <div class="card-body">
        <el-table :data="filteredData" border size="small" max-height="600" style="width: 100%">
          <el-table-column prop="pid" label="PID" width="80" sortable />
          <el-table-column prop="name" label="名称" min-width="160" sortable />
          <el-table-column label="CPU%" width="100" sortable :sort-method="sortByCpu">
            <template #default="{ row }">{{ row.cpu_usage.toFixed(1) }}%</template>
          </el-table-column>
          <el-table-column label="内存" width="120" sortable :sort-method="sortByMemory">
            <template #default="{ row }">{{ formatBytes(row.memory_bytes) }}</template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="80" />
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { ElLoading } from 'element-plus'
import { getProcessList, formatBytes, formatTimestamp, type ProcessItem } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<ProcessItem[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const sortBy = ref('cpu')

let searchTimer: ReturnType<typeof setTimeout> | null = null
const searchTrigger = ref('')

watch(searchQuery, (val) => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { searchTrigger.value = val }, 300)
})

const filteredData = computed(() => {
  let result = [...data.value]
  const q = searchTrigger.value.toLowerCase().trim()
  if (q) {
    result = result.filter(p =>
      p.name.toLowerCase().includes(q) || p.pid.toString().includes(q)
    )
  }
  if (sortBy.value === 'cpu') {
    result.sort((a, b) => b.cpu_usage - a.cpu_usage)
  } else {
    result.sort((a, b) => b.memory_bytes - a.memory_bytes)
  }
  return result
})

const sortByCpu = (a: ProcessItem, b: ProcessItem) => b.cpu_usage - a.cpu_usage
const sortByMemory = (a: ProcessItem, b: ProcessItem) => b.memory_bytes - a.memory_bytes

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getProcessList()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'processList',
      action: '查看进程列表',
      inputPreview: '',
      outputPreview: `${data.value.length} 个进程`,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => { loadData() })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 8px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/ProcessListView.vue
git commit -m "feat: 添加进程列表页面"
```

---

## Task 11: HardwareInfoView.vue — 硬件外设页面

**Files:**
- Create: `src/views/HardwareInfoView.vue`

- [ ] **Step 1: 创建 HardwareInfoView.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">硬件外设</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <template v-if="data">
      <!-- GPU -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">GPU</span></div>
        <div class="card-body">
          <div v-for="(gpu, i) in data.gpus" :key="i" class="hw-section">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">名称</span><span class="kv-value">{{ gpu.name || '—' }}</span></div>
              <div class="kv-item"><span class="kv-label">驱动版本</span><span class="kv-value">{{ gpu.driver_version || '—' }}</span></div>
              <div class="kv-item"><span class="kv-label">显存</span><span class="kv-value">{{ gpu.vram_mb > 0 ? gpu.vram_mb + ' MB' : '未知' }}</span></div>
            </div>
          </div>
          <div v-if="data.gpus.length === 0" class="empty-tip">未检测到 GPU</div>
        </div>
      </div>

      <!-- 显示器 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">显示器</span></div>
        <div class="card-body">
          <div v-for="(display, i) in data.displays" :key="i" class="hw-section">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">名称</span><span class="kv-value">{{ display.name }}</span></div>
              <div class="kv-item"><span class="kv-label">分辨率</span><span class="kv-value">{{ display.resolution || '—' }}</span></div>
            </div>
          </div>
          <div v-if="data.displays.length === 0" class="empty-tip">未检测到显示器</div>
        </div>
      </div>

      <!-- 音频设备 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">音频设备</span></div>
        <div class="card-body">
          <el-table :data="data.audio_devices" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" min-width="200" />
            <el-table-column prop="status" label="状态" width="100" />
          </el-table>
          <div v-if="data.audio_devices.length === 0" class="empty-tip">未检测到音频设备</div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getHardwareInfo, formatTimestamp, type HardwareInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<HardwareInfo | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getHardwareInfo()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'hardwareInfo',
      action: '查看硬件外设',
      inputPreview: '',
      outputPreview: `${data.value.gpus.length} GPU | ${data.value.displays.length} 显示器`,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => { loadData() })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 12px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.hw-section { padding: 8px 0; border-bottom: 1px solid var(--border-color); }
.hw-section:last-child { border-bottom: none; }
.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.empty-tip { color: var(--text-muted); font-size: 13px; padding: 8px 0; }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/HardwareInfoView.vue
git commit -m "feat: 添加硬件外设页面"
```

---

## Task 12: SoftwareEnvView.vue — 软件环境页面

**Files:**
- Create: `src/views/SoftwareEnvView.vue`

- [ ] **Step 1: 创建 SoftwareEnvView.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">软件环境</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <template v-if="data">
      <el-tabs v-model="activeTab" class="env-tabs">
        <!-- 已安装软件 -->
        <el-tab-pane :label="`已安装软件 (${data.installed_software.length})`" name="software">
          <div class="tool-card">
            <div class="card-header">
              <span class="card-title">已安装软件</span>
              <el-input v-model="softwareSearch" size="small" placeholder="搜索软件名..." style="width: 200px" clearable />
            </div>
            <div class="card-body">
              <el-table :data="filteredSoftware" border size="small" max-height="500" style="width: 100%">
                <el-table-column prop="name" label="名称" min-width="200" sortable />
                <el-table-column prop="version" label="版本" width="100" />
                <el-table-column prop="publisher" label="发布者" min-width="150" />
                <el-table-column prop="install_date" label="安装日期" width="120" />
              </el-table>
            </div>
          </div>
        </el-tab-pane>

        <!-- 环境变量 -->
        <el-tab-pane :label="`环境变量 (${data.environment_variables.length})`" name="env">
          <div class="tool-card">
            <div class="card-header">
              <span class="card-title">环境变量</span>
              <el-input v-model="envSearch" size="small" placeholder="搜索变量名..." style="width: 200px" clearable />
            </div>
            <div class="card-body">
              <el-table :data="filteredEnv" border size="small" max-height="500" style="width: 100%">
                <el-table-column prop="key" label="变量名" width="200" sortable />
                <el-table-column prop="value" label="值" min-width="300" show-overflow-tooltip />
              </el-table>
            </div>
          </div>
        </el-tab-pane>

        <!-- 启动项 -->
        <el-tab-pane :label="`启动项 (${data.startup_items.length})`" name="startup">
          <div class="tool-card">
            <div class="card-header"><span class="card-title">启动项</span></div>
            <div class="card-body">
              <el-table :data="data.startup_items" border size="small" max-height="500" style="width: 100%">
                <el-table-column prop="name" label="名称" min-width="150" />
                <el-table-column prop="command" label="命令" min-width="300" show-overflow-tooltip />
                <el-table-column prop="location" label="位置" min-width="200" show-overflow-tooltip />
              </el-table>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getSoftwareEnv, formatTimestamp, type SoftwareEnv } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<SoftwareEnv | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const activeTab = ref('software')
const softwareSearch = ref('')
const envSearch = ref('')

const filteredSoftware = computed(() => {
  if (!data.value) return []
  const q = softwareSearch.value.toLowerCase().trim()
  if (!q) return data.value.installed_software
  return data.value.installed_software.filter(s => s.name.toLowerCase().includes(q))
})

const filteredEnv = computed(() => {
  if (!data.value) return []
  const q = envSearch.value.toLowerCase().trim()
  if (!q) return data.value.environment_variables
  return data.value.environment_variables.filter(e => e.key.toLowerCase().includes(q))
})

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getSoftwareEnv()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'softwareEnv',
      action: '查看软件环境',
      inputPreview: '',
      outputPreview: `${data.value.installed_software.length} 软件 | ${data.value.environment_variables.length} 环境变量`,
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => { loadData() })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 12px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
.env-tabs :deep(.el-tabs__header) { margin-bottom: 16px; padding-left: 8px; position: sticky; top: 0; z-index: 20; background: var(--bg-primary); box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3); }
.env-tabs :deep(.el-tabs__item) { color: var(--text-secondary); font-size: 14px; font-weight: 500; }
.env-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.env-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.env-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/SoftwareEnvView.vue
git commit -m "feat: 添加软件环境页面"
```

---

## Task 13: 注册工具 — store / sidebar / App.vue

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/components/SidebarNav.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: 在 store/index.ts 的 TOOL_LIST 末尾添加 5 个工具项**

在 `src/store/index.ts` 的 `TOOL_LIST` 数组中，最后一个工具项（workflow）之后添加：

```typescript
  { id: 'systemInfo', name: '系统信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>`, description: '查看操作系统、CPU、内存、磁盘信息', keywords: ['系统', 'cpu', '内存', '磁盘', 'system'], category: 'system' },
  { id: 'networkInfo', name: '网络信息', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/></svg>`, description: '查看网络接口、IP、连接、端口', keywords: ['网络', 'ip', 'mac', '端口', 'netstat'], category: 'system' },
  { id: 'processList', name: '进程列表', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>`, description: '查看运行中的进程及资源占用', keywords: ['进程', 'process', '任务管理器'], category: 'system' },
  { id: 'hardwareInfo', name: '硬件外设', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>`, description: '查看GPU、显示器、音频设备', keywords: ['硬件', 'gpu', '显卡', '显示器', '音频'], category: 'system' },
  { id: 'softwareEnv', name: '软件环境', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>`, description: '已安装软件、环境变量、启动项', keywords: ['软件', '环境变量', '启动项', 'env'], category: 'system' },
```

- [ ] **Step 2: 在 SidebarNav.vue 的 categoryNames 中添加 system 分类**

在 `src/components/SidebarNav.vue` 的 `categoryNames` 对象中（约第 301 行），添加 `system` 条目：

```typescript
  const categoryNames: Record<string, string> = {
    text: '文本工具',
    dev: '开发工具',
    security: '安全工具',
    utility: '实用工具',
    system: '系统工具'
  }
```

- [ ] **Step 3: 在 App.vue 添加 import 和 v-else-if 分支**

在 `src/App.vue` 的 import 区（CalculatorTool import 之后）添加：

```typescript
import SystemInfoView from '@/views/SystemInfoView.vue'
import NetworkInfoView from '@/views/NetworkInfoView.vue'
import ProcessListView from '@/views/ProcessListView.vue'
import HardwareInfoView from '@/views/HardwareInfoView.vue'
import SoftwareEnvView from '@/views/SoftwareEnvView.vue'
```

在 `<KeepAlive>` 块内，CalculatorTool 分支之后添加：

```vue
          <SystemInfoView v-else-if="activeTool === 'systemInfo'" :key="'systemInfo'" />
          <NetworkInfoView v-else-if="activeTool === 'networkInfo'" :key="'networkInfo'" />
          <ProcessListView v-else-if="activeTool === 'processList'" :key="'processList'" />
          <HardwareInfoView v-else-if="activeTool === 'hardwareInfo'" :key="'hardwareInfo'" />
          <SoftwareEnvView v-else-if="activeTool === 'softwareEnv'" :key="'softwareEnv'" />
```

- [ ] **Step 4: 验证前端编译**

Run: `npx tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 5: 提交**

```bash
git add src/store/index.ts src/components/SidebarNav.vue src/App.vue
git commit -m "feat: 注册 5 个系统信息工具到菜单和路由"
```

---

## Task 14: 集成验证

**Files:** 无代码改动

- [ ] **Step 1: 启动 Tauri 开发服务器**

Run: `npm run tauri dev`
Expected: 应用正常启动，左侧菜单出现「系统工具」分类，包含 5 个工具

- [ ] **Step 2: 验证系统信息页**

点击「系统信息」菜单项，验证：
- OS 名称、版本、架构、主机名显示正确
- CPU 型号、核心数、线程数、频率显示正确
- CPU 使用率非零
- 内存总量、已用、可用显示正确
- 磁盘列表正确显示，使用率进度条正常
- 点击「刷新」按钮可重新采集

- [ ] **Step 3: 验证网络信息页**

点击「网络信息」菜单项，验证：
- 网络接口列表显示名称、MAC、IP
- 活动连接列表非空
- 监听端口列表非空
- WiFi 名称显示（如已连接）

- [ ] **Step 4: 验证进程列表页**

点击「进程列表」菜单项，验证：
- 进程列表非空，按 CPU 降序排列
- 搜索框可过滤进程
- 切换排序方式（CPU/内存）正常

- [ ] **Step 5: 验证硬件外设页**

点击「硬件外设」菜单项，验证：
- GPU 名称和驱动版本显示正确
- 显示器分辨率正确
- 音频设备列表正确

- [ ] **Step 6: 验证软件环境页**

点击「软件环境」菜单项，验证：
- 已安装软件列表非空，搜索功能正常
- 环境变量列表完整，搜索功能正常
- 启动项列表显示

- [ ] **Step 7: 最终提交（如有修复）**

```bash
git add -A
git commit -m "fix: 集成验证修复"
```

---

## 自审检查

### 规格覆盖

| 规格要求 | 对应 Task |
|----------|-----------|
| get_system_info 命令 | Task 2 |
| get_network_info 命令 | Task 3 |
| get_process_list 命令 | Task 4 |
| get_hardware_info 命令 | Task 5 |
| get_software_env 命令 | Task 6 |
| systemInfoClient.ts | Task 7 |
| SystemInfoView.vue | Task 8 |
| NetworkInfoView.vue | Task 9 |
| ProcessListView.vue | Task 10 |
| HardwareInfoView.vue | Task 11 |
| SoftwareEnvView.vue | Task 12 |
| store 注册 | Task 13 Step 1 |
| SidebarNav 分类 | Task 13 Step 2 |
| App.vue 路由 | Task 13 Step 3 |
| 集成验证 | Task 14 |

### 类型一致性

- Rust 结构体字段名（snake_case）与 TypeScript 接口字段名一致
- `invoke` 命令名与 `#[tauri::command]` 函数名一致（`get_system_info` 等）
- 前端 import 的函数名与 `systemInfoClient.ts` 导出的函数名一致

### 已知限制

- `AdapterRAM` 为 WMI u32 限制，超过 4GB 显存显示为「未知」（Task 5 中已标注 ponytail 注释）
- sysinfo API 版本可能略有差异，如 `brand_name()` 在某些版本为 `brand()`，编译时按错误信息调整即可
- `Process::status()` 的 `Display` trait 实现可能因版本不同，使用 match 确保兼容
