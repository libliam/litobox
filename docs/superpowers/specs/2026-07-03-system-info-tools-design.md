# 系统信息工具集 设计文档

> 日期：2026-07-03
> 状态：待审核

## 1. 概述

为 LitoBox 新增 5 个系统级信息展示工具，作为独立菜单项归入新增的「系统工具」分类。工具为只读展示型，打开时采集一次快照，手动点击刷新重新采集。

### 工具清单

| ID | 名称 | 说明 |
|----|------|------|
| `systemInfo` | 系统信息 | 操作系统、CPU、内存、磁盘 |
| `networkInfo` | 网络信息 | 网络接口、IP/MAC、活动连接、监听端口、WiFi |
| `processList` | 进程列表 | 运行中的进程及 CPU/内存占用 |
| `hardwareInfo` | 硬件外设 | GPU、显示器、音频设备 |
| `softwareEnv` | 软件环境 | 已安装软件、环境变量、启动项 |

### 设计约束

- 纯本地离线运行，无网络请求
- 静态快照模式（不实时监控），手动刷新
- 遵循科技风 UI 规范，使用 CSS 变量，不硬编码颜色
- 新增依赖仅 `sysinfo` crate

## 2. 架构

### 文件变更

**新增文件：**

| 文件 | 说明 |
|------|------|
| `src-tauri/src/system_info.rs` | Rust 后端模块，5 个 Tauri 命令 |
| `src/utils/systemInfoClient.ts` | 前端调用层，封装 invoke + TS 类型定义 |
| `src/views/SystemInfoView.vue` | 系统信息页面 |
| `src/views/NetworkInfoView.vue` | 网络信息页面 |
| `src/views/ProcessListView.vue` | 进程列表页面 |
| `src/views/HardwareInfoView.vue` | 硬件外设页面 |
| `src/views/SoftwareEnvView.vue` | 软件环境页面 |

**修改文件：**

| 文件 | 改动 |
|------|------|
| `src-tauri/Cargo.toml` | 新增 `sysinfo = "0.32"` |
| `src-tauri/src/main.rs` | `mod system_info;` + 注册 5 个命令到 `invoke_handler` |
| `src/store/index.ts` | `TOOL_LIST` 新增 5 个工具项，category 为 `system` |
| `src/components/SidebarNav.vue` | `categoryNames` 新增 `system: '系统工具'` |
| `src/App.vue` | 新增 5 个 `v-else-if` 分支 + import |

### 数据采集策略

| 数据源 | 覆盖范围 | 方式 |
|--------|----------|------|
| `sysinfo` crate | CPU、内存、磁盘、网络接口、进程 | Rust 库直接调用 |
| `std::env` | 环境变量 | 标准库 `env::vars()` |
| `netstat -ano` | 活动 TCP 连接、监听端口 | `std::process::Command` + 输出解析 |
| `netsh wlan show interface` | WiFi SSID | `std::process::Command` + 输出解析 |
| `ipconfig /all` | 默认网关、DNS 服务器 | `std::process::Command` + 输出解析 |
| PowerShell `Get-CimInstance` | GPU、音频设备、已安装软件、启动项 | `std::process::Command` + JSON 输出解析 |

### 菜单分类

新增 `system` 分类，显示名「系统工具」。避免 `utility` 分类膨胀（当前已有 9 项）。

## 3. 后端设计

### 3.1 模块结构

`src-tauri/src/system_info.rs` 单文件，内部按数据域分函数，对外暴露 5 个 `#[tauri::command]`。

### 3.2 命令与数据结构

#### 命令 1：`get_system_info`

返回 `SystemInfo`，包含 OS、CPU、内存、磁盘概览。

```rust
#[derive(Serialize)]
struct SystemInfo {
    os_name: String,
    os_version: String,
    os_arch: String,
    hostname: String,
    uptime_secs: u64,
    cpu: CpuInfo,
    memory: MemoryInfo,
    disks: Vec<DiskInfo>,
}

#[derive(Serialize)]
struct CpuInfo {
    brand: String,
    core_count: usize,
    thread_count: usize,
    frequency_mhz: u64,
    usage_percent: f32,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
}

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_bytes: u64,
    used_bytes: u64,
    file_system: String,
    drive_type: String,
}
```

采集方式：`sysinfo::System`（CPU、内存）、`sysinfo::Disks`（磁盘）。CPU 使用率需先 `system.refresh_cpu_usage()` 后短暂等待（约 200ms）再读取，以获得非零值。

#### 命令 2：`get_network_info`

返回 `NetworkInfo`，包含网络接口、连接、端口、WiFi。

```rust
#[derive(Serialize)]
struct NetworkInfo {
    hostname: String,
    interfaces: Vec<NetInterface>,
    default_gateway: String,
    dns_servers: Vec<String>,
    wifi_name: Option<String>,
    active_connections: Vec<TcpConnection>,
    listening_ports: Vec<ListeningPort>,
}

#[derive(Serialize)]
struct NetInterface {
    name: String,
    mac: String,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
    subnet: String,
    is_up: bool,
}

#[derive(Serialize)]
struct TcpConnection {
    protocol: String,
    local_addr: String,
    remote_addr: String,
    state: String,
    pid: u32,
}

#[derive(Serialize)]
struct ListeningPort {
    protocol: String,
    local_addr: String,
    pid: u32,
    process_name: String,
}
```

采集方式：
- 网络接口：`sysinfo::Networks`
- 活动连接 + 监听端口：执行 `netstat -ano`，按行解析。TCP 行格式为 `协议  本地地址  远程地址  状态  PID`（tab/空格分隔）。**注意**：Windows 中文系统下 `netstat` 输出表头为中文，需按列位置（空格分割后的索引）而非列名解析。状态为 `LISTENING` 的归入 `listening_ports`，其余归入 `active_connections`。
- WiFi 名称：执行 `netsh wlan show interface`，匹配 `SSID` 行。未连接 WiFi 时返回 `None`。
- 网关/DNS：执行 `ipconfig /all`，解析默认网关和 DNS 服务器行。如解析失败则留空。
- 监听端口的进程名：通过 PID 用 `sysinfo::System::process(pid)` 查询。

#### 命令 3：`get_process_list`

返回 `Vec<ProcessItem>`，按 CPU 占用降序排列。

```rust
#[derive(Serialize)]
struct ProcessItem {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_bytes: u64,
    status: String,
    command: String,
}
```

采集方式：`sysinfo::System::processes()`。需先 `refresh_processes()` 刷新。CPU 使用率同上需短暂等待。`command` 字段取进程可执行文件路径，无权限时为空字符串。

#### 命令 4：`get_hardware_info`

返回 `HardwareInfo`，包含 GPU、显示器、音频设备。

```rust
#[derive(Serialize)]
struct HardwareInfo {
    gpus: Vec<GpuInfo>,
    displays: Vec<DisplayInfo>,
    audio_devices: Vec<AudioDevice>,
}

#[derive(Serialize)]
struct GpuInfo {
    name: String,
    driver_version: String,
    vram_mb: u64,
}

#[derive(Serialize)]
struct DisplayInfo {
    name: String,
    resolution: String,
    refresh_rate: u32,
}

#[derive(Serialize)]
struct AudioDevice {
    name: String,
    status: String,
}
```

采集方式：通过 PowerShell `Get-CimInstance` 查询 WMI，输出格式设为 JSON 便于解析：
- GPU：`Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM | ConvertTo-Json`
- 显示器：`Get-CimInstance Win32_DesktopMonitor | Select-Object Name,ScreenWidth,ScreenHeight | ConvertTo-Json`
- 音频：`Get-CimInstance Win32_SoundDevice | Select-Object Name,Status | ConvertTo-Json`

`AdapterRAM` 为 `u32` 类型（WMI 限制），超过 4GB 的显存会溢出，此时 VRAM 显示为「未知」。显示分辨率可由前端 `window.screen.width/height` 补充。

#### 命令 5：`get_software_env`

返回 `SoftwareEnv`，包含已安装软件、环境变量、启动项。

```rust
#[derive(Serialize)]
struct SoftwareEnv {
    installed_software: Vec<SoftwareItem>,
    environment_variables: Vec<EnvVar>,
    startup_items: Vec<StartupItem>,
}

#[derive(Serialize)]
struct SoftwareItem {
    name: String,
    version: String,
    publisher: String,
    install_date: String,
}

#[derive(Serialize)]
struct EnvVar {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct StartupItem {
    name: String,
    command: String,
    location: String,
}
```

采集方式：
- 已安装软件：PowerShell 查询注册表 `HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*` 和 `HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*`，取 `DisplayName`、`DisplayVersion`、`Publisher`、`InstallDate`。过滤无名称的条目。
- 环境变量：`std::env::vars()` 直接收集。
- 启动项：PowerShell 查询 `HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`、`HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`，以及 `[Environment]::GetFolderPath('Startup')` 目录下的文件。

### 3.3 错误处理

每个命令内部用 `Result<T, String>`，采集失败时返回错误字符串。前端 catch 后在错误提示卡片中展示。部分子项采集失败（如 WiFi 未连接、GPU 查询失败）时返回空列表或 `None`，不影响整体。

### 3.4 性能注意

- CPU 使用率采集需要 `refresh_cpu_usage()` 后等待约 200ms 再读取，否则返回 0
- 进程列表采集需 `refresh_processes()`，同样有 CPU 使用率等待
- PowerShell 命令执行可能有 1-3 秒延迟，硬件信息和软件环境页面需显示加载提示
- `sysinfo::System` 实例在每次命令调用时新建，不跨命令复用（避免全局状态）

## 4. 前端设计

### 4.1 统一交互模式

5 个页面共享以下模式：

- **顶部 sticky 操作栏**：`.tool-card.sticky-card`，含「刷新」按钮 + 采集时间戳
- **加载状态**：`ElLoading.service()` + `finally` 确保关闭
- **初始加载**：`onMounted` 自动采集一次
- **数据展示**：`.tool-card` 卡片，青色大写标题
- **错误提示**：红色边框卡片
- **表格**：`el-table`，深色主题，使用 CSS 变量

### 4.2 页面布局

#### SystemInfoView.vue

4 个卡片区域：
1. **操作系统**：名称、版本、架构、主机名、运行时长 — key-value 网格
2. **CPU**：型号、核心数、线程数、频率、使用率 — key-value + `el-progress` 进度条
3. **内存**：总计、已用、可用 — `el-progress` 进度条
4. **磁盘**：每个磁盘一行，含名称、总量、已用、文件系统 — `el-table` 或列表

运行时长格式化：`X天X小时X分钟`

#### NetworkInfoView.vue

4 个卡片区域：
1. **概览**：主机名、WiFi、默认网关、DNS — key-value 网格
2. **网络接口**：`el-table`，列：名称、MAC、IPv4、子网、状态
3. **活动连接**：`el-table`，列：协议、本地地址、远程地址、状态、PID
4. **监听端口**：`el-table`，列：协议、地址、PID、进程名

#### ProcessListView.vue

1 个操作栏 + 1 个表格：
- **操作栏**：搜索框（过滤进程名/PID）+ 排序切换（CPU/内存）+ 刷新按钮
- **表格**：`el-table`，列：PID、名称、CPU%、内存（格式化为 MB/GB）
- 默认按 CPU 降序，搜索框 300ms 防抖过滤
- 表格底部显示总进程数

#### HardwareInfoView.vue

3 个卡片区域：
1. **GPU**：名称、驱动版本、显存 — key-value 网格（支持多 GPU）
2. **显示器**：名称、分辨率、刷新率 — key-value 网格（支持多显示器）
3. **音频设备**：`el-table`，列：名称、状态

#### SoftwareEnvView.vue

`el-tabs` 分 3 个 Tab：
1. **已安装软件**：`el-table`，列：名称、版本、发布者、安装日期。带搜索框。
2. **环境变量**：`el-table`，列：变量名、值（可复制）。带搜索框。
3. **启动项**：`el-table`，列：名称、命令、位置。

### 4.3 工具注册

#### store/index.ts — TOOL_LIST 新增

```typescript
{ id: 'systemInfo', name: '系统信息', icon: '', iconSvg: '<svg>...</svg>', description: '查看操作系统、CPU、内存、磁盘信息', keywords: ['系统', 'cpu', '内存', '磁盘', 'system'], category: 'system' },
{ id: 'networkInfo', name: '网络信息', icon: '', iconSvg: '<svg>...</svg>', description: '查看网络接口、IP、连接、端口', keywords: ['网络', 'ip', 'mac', '端口', 'netstat'], category: 'system' },
{ id: 'processList', name: '进程列表', icon: '', iconSvg: '<svg>...</svg>', description: '查看运行中的进程及资源占用', keywords: ['进程', 'process', '任务管理器'], category: 'system' },
{ id: 'hardwareInfo', name: '硬件外设', icon: '', iconSvg: '<svg>...</svg>', description: '查看GPU、显示器、音频设备', keywords: ['硬件', 'gpu', '显卡', '显示器', '音频'], category: 'system' },
{ id: 'softwareEnv', name: '软件环境', icon: '', iconSvg: '<svg>...</svg>', description: '已安装软件、环境变量、启动项', keywords: ['软件', '环境变量', '启动项', 'env'], category: 'system' },
```

#### SidebarNav.vue — categoryNames 新增

```typescript
system: '系统工具',
```

#### App.vue — 新增分支

```vue
<SystemInfoView v-else-if="activeTool === 'systemInfo'" :key="'systemInfo'" />
<NetworkInfoView v-else-if="activeTool === 'networkInfo'" :key="'networkInfo'" />
<ProcessListView v-else-if="activeTool === 'processList'" :key="'processList'" />
<HardwareInfoView v-else-if="activeTool === 'hardwareInfo'" :key="'hardwareInfo'" />
<SoftwareEnvView v-else-if="activeTool === 'softwareEnv'" :key="'softwareEnv'" />
```

### 4.4 工作流集成

这 5 个工具为只读信息展示型，不涉及输入→输出的数据转换：
- 工作流集成：不适用
- 变量池集成：不适用
- 历史记录：刷新时记录一条操作历史（action: "查看系统信息"，outputPreview: 摘要信息）

### 4.5 前端调用层

新增 `src/utils/systemInfoClient.ts`，封装 5 个 `invoke()` 调用：

```typescript
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info')
}
export async function getNetworkInfo(): Promise<NetworkInfo> {
  return invoke<NetworkInfo>('get_network_info')
}
export async function getProcessList(): Promise<ProcessItem[]> {
  return invoke<ProcessItem[]>('get_process_list')
}
export async function getHardwareInfo(): Promise<HardwareInfo> {
  return invoke<HardwareInfo>('get_hardware_info')
}
export async function getSoftwareEnv(): Promise<SoftwareEnv> {
  return invoke<SoftwareEnv>('get_software_env')
}
```

TypeScript 类型定义与 Rust 结构体一一对应。

## 5. 测试

非核心逻辑（Rust 数据采集）依赖系统环境，不适合单元测试。以下为验证检查项：

- **后端编译检查**：`cargo check` 通过
- **前端编译检查**：`npm run build` 通过
- **运行时验证**：`npm run tauri dev` 启动后，逐一手动验证每个页面：
  1. 系统信息页：CPU 型号、内存总量、磁盘列表正确显示
  2. 网络信息页：本机 IP、MAC、活动连接列表正确
  3. 进程列表页：进程数量合理，CPU/内存占用非零
  4. 硬件外设页：GPU 型号、显示器分辨率正确
  5. 软件环境页：已安装软件列表非空，环境变量完整
- **刷新功能**：每个页面刷新按钮可重新采集
- **错误处理**：模拟 PowerShell 不可用时，页面显示错误提示而非崩溃
