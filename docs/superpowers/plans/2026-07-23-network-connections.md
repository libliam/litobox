# 网络连接查看器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 LitoBox 新增「网络连接」工具页，列出所有 TCP/UDP 连接并关联进程信息，支持筛选/自动刷新/结束进程/端口释放/导出 CSV。

**Architecture:** 新建 `network_connections.rs` 模块，`netstat -ano` 主路径 + PowerShell cmdlet fallback 双轨采集，`sysinfo` crate 补进程名/路径；前端 `NetworkConnections.vue` 纯前端筛选排序，复用现有 `kill_process` 命令。

**Tech Stack:** Tauri 2.0 + Rust + Vue 3 (Composition API) + TypeScript + Element Plus + Pinia

**Spec:** `docs/superpowers/specs/2026-07-23-network-connections-design.md`

---

## 文件结构

| 文件 | 动作 | 职责 |
|------|------|------|
| `src-tauri/src/network_connections.rs` | 新建 | `get_network_connections` 命令 |
| `src-tauri/src/main.rs` | 改 | `mod` 声明 + `invoke_handler` 注册 |
| `src/utils/systemInfoClient.ts` | 改 | 类型 + `getNetworkConnections()` invoke 封装 |
| `src/store/index.ts` | 改 | TOOL_LIST 加 `networkConnections` 条目 |
| `src/views/NetworkConnections.vue` | 新建 | 前端工具页 |
| `package.json` | 改 | version 6.0.0 → 6.1.0 |
| `src-tauri/tauri.conf.json` | 改 | version 同步 |
| `src-tauri/Cargo.toml` | 改 | version 同步 |
| `README.md` | 改 | 版本路线表加 V6.1 |
| `docs/superpowers/plans/feature-backlog.md` | 改 | A10 标记完成 + 已完成版本表 |

---

## Task 1: Rust 后端 — network_connections.rs 新建

**Files:**
- Create: `src-tauri/src/network_connections.rs`

- [ ] **Step 1: 创建文件，写入完整代码**

创建 `src-tauri/src/network_connections.rs`：

```rust
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
```

- [ ] **Step 2: cargo check 验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/network_connections.rs
git commit -m "feat(network): 新建 network_connections.rs - netstat 解析 + PowerShell fallback"
```

---

## Task 2: main.rs 注册模块 + 命令

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 加 mod 声明**

在 `src-tauri/src/main.rs` 的 `mod hosts_manager;` 之后（line 22），加一行：

```rust
mod network_connections;
```

- [ ] **Step 2: 加 invoke_handler 注册**

在 `src-tauri/src/main.rs` 的 `invoke_handler` 列表末尾（`system_info::restart_service,` 之后，line 127），加一行：

```rust
            network_connections::get_network_connections,
```

- [ ] **Step 3: cargo check 验证**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: 编译通过，无错误

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(network): main.rs 注册 network_connections 模块与命令"
```

---

## Task 3: 前端 — systemInfoClient.ts 扩展类型 + invoke

**Files:**
- Modify: `src/utils/systemInfoClient.ts`

- [ ] **Step 1: 在文件中加类型定义和 invoke 函数**

在 `src/utils/systemInfoClient.ts` 末尾（`formatTimestamp` 函数之后，line 297 前），插入：

```ts
// ============ 网络连接查看器 ============

export interface NetworkConnection {
  protocol: string
  local_addr: string
  remote_addr: string
  state: string
  pid: number
  process_name: string
  process_path: string
}

export function getNetworkConnections(): Promise<NetworkConnection[]> {
  return invoke<NetworkConnection[]>('get_network_connections')
}
```

- [ ] **Step 2: npm run build 验证**

Run: `npm run build`
Expected: vue-tsc 类型检查通过，vite build 成功

- [ ] **Step 3: Commit**

```bash
git add src/utils/systemInfoClient.ts
git commit -m "feat(network): systemInfoClient 加 NetworkConnection 类型 + getNetworkConnections"
```

---

## Task 4: store — TOOL_LIST 注册新工具

**Files:**
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 TOOL_LIST 中加 networkConnections 条目**

在 `src/store/index.ts` 的 `hostsManager` 条目之后（line 97 之后，`];` 之前），插入：

```ts
  { id: 'networkConnections', name: '网络连接', icon: '🔌', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M8 8h8v8H8z"/><line x1="12" y1="4" x2="12" y2="8"/><line x1="12" y1="16" x2="12" y2="20"/><line x1="4" y1="12" x2="8" y2="12"/><line x1="16" y1="12" x2="20" y2="12"/></svg>`, description: '查看所有 TCP/UDP 连接，关联进程，支持筛选/结束进程/释放端口/导出', keywords: ['网络连接', 'tcp', 'udp', 'netstat', '端口', '连接', 'network'], category: 'system' },
```

- [ ] **Step 2: npm run build 验证**

Run: `npm run build`
Expected: 编译通过（此时 `NetworkConnections.vue` 还不存在，但 `SidebarNav` 会尝试导入 — 需要先创建占位文件，或等 Task 5 一起验证）

**注意：** 此步骤创建 `src/views/NetworkConnections.vue` 占位文件以通过 build：

```vue
<template><div class="tool-container"><div class="tool-card"><div class="card-body"><el-empty description="加载中..." /></div></div></div></template>
```

- [ ] **Step 3: Commit**

```bash
git add src/store/index.ts src/views/NetworkConnections.vue
git commit -m "feat(network): store TOOL_LIST 注册网络连接工具"
```

---

## Task 5: NetworkConnections.vue 工具页

**Files:**
- Modify: `src/views/NetworkConnections.vue`（替换 Task 4 的占位文件）

- [ ] **Step 1: 创建完整组件**

用以下代码替换 `src/views/NetworkConnections.vue`：

```vue
<template>
  <div class="tool-container">
    <!-- 统计概览 -->
    <div v-if="!error && connections.length" class="stats-row">
      <div class="stat-card">
        <span class="stat-number">{{ filteredConnections.length }}</span>
        <span class="stat-label">连接总数</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ tcpCount }}</span>
        <span class="stat-label">TCP</span>
      </div>
      <div class="stat-card">
        <span class="stat-number">{{ udpCount }}</span>
        <span class="stat-label">UDP</span>
      </div>
      <div class="stat-card" v-for="s in stateStats" :key="s.state">
        <span class="stat-number">{{ s.count }}</span>
        <span class="stat-label">{{ stateLabel(s.state) }}</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">网络连接</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索端口/PID/进程名/地址..." style="width: 240px" clearable />
          <el-select v-model="protocolFilter" size="small" style="width: 90px">
            <el-option label="全部" value="all" />
            <el-option label="TCP" value="TCP" />
            <el-option label="UDP" value="UDP" />
          </el-select>
          <el-select v-model="stateFilter" size="small" style="width: 120px">
            <el-option label="全部状态" value="all" />
            <el-option label="LISTENING" value="LISTENING" />
            <el-option label="ESTABLISHED" value="ESTABLISHED" />
            <el-option label="TIME_WAIT" value="TIME_WAIT" />
            <el-option label="CLOSE_WAIT" value="CLOSE_WAIT" />
            <el-option label="SYN_SENT" value="SYN_SENT" />
          </el-select>
          <span v-if="lastRefresh" class="refresh-time">{{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="refresh">刷新</el-button>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && !connections.length" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击「刷新」获取网络连接" />
      </div>
    </div>

    <!-- 连接表格 -->
    <div v-if="connections.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">连接列表 ({{ filteredConnections.length }} / {{ connections.length }})</span>
      </div>
      <div class="card-body">
        <el-table :data="filteredConnections" border size="small" max-height="600" style="width: 100%" v-loading="loading">
          <el-table-column label="协议" width="70">
            <template #default="{ row }">
              <el-tag :type="row.protocol === 'TCP' ? '' : 'success'" size="small">{{ row.protocol }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="本地地址" width="170">
            <template #default="{ row }">
              <span class="mono-text">{{ row.local_addr }}</span>
            </template>
          </el-table-column>
          <el-table-column label="远程地址" width="170">
            <template #default="{ row }">
              <span class="mono-text">{{ row.remote_addr }}</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="120">
            <template #default="{ row }">
              <el-tag v-if="row.state" :type="stateTagType(row.state)" size="small">{{ row.state }}</el-tag>
              <span v-else class="text-secondary">—</span>
            </template>
          </el-table-column>
          <el-table-column prop="pid" label="PID" width="70" sortable />
          <el-table-column prop="process_name" label="进程名" width="140" sortable>
            <template #default="{ row }">
              <span :class="{ 'text-secondary': row.process_name === '(已退出)' }">{{ row.process_name }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="process_path" label="进程路径" min-width="200" show-overflow-tooltip />
          <el-table-column label="操作" width="180" fixed="right">
            <template #default="{ row }">
              <el-button
                type="danger" size="small" link
                :disabled="row.pid === 0 || row.process_name === '(已退出)'"
                :loading="killingPids.has(row.pid)"
                @click="handleKill(row)">
                结束进程
              </el-button>
              <el-button
                type="warning" size="small" link
                :disabled="row.pid === 0 || row.process_name === '(已退出)'"
                :loading="killingPids.has(row.pid)"
                @click="handleReleasePort(row)">
                释放端口
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 底部栏 -->
    <div v-if="connections.length" class="tool-card">
      <div class="card-body">
        <div class="bottom-bar">
          <div class="auto-refresh">
            <span class="group-label">自动刷新</span>
            <el-switch v-model="autoRefresh" size="small" @change="toggleAutoRefresh" />
            <el-select v-if="autoRefresh" v-model="refreshInterval" size="small" style="width: 80px" @change="restartAutoRefresh">
              <el-option label="5s" :value="5" />
              <el-option label="10s" :value="10" />
              <el-option label="30s" :value="30" />
            </el-select>
          </div>
          <div class="bottom-actions">
            <el-button size="small" @click="exportCsv">导出 CSV</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { getNetworkConnections, killProcess, formatTimestamp, type NetworkConnection } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const connections = ref<NetworkConnection[]>([])
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const searchQuery = ref('')
const protocolFilter = ref('all')
const stateFilter = ref('all')
const killingPids = ref(new Set<number>())

// 自动刷新
const autoRefresh = ref(false)
const refreshInterval = ref(5)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ============ 筛选 ============

const filteredConnections = computed(() => {
  let result = connections.value

  // 协议筛选
  if (protocolFilter.value !== 'all') {
    result = result.filter(c => c.protocol === protocolFilter.value)
  }

  // 状态筛选
  if (stateFilter.value !== 'all') {
    result = result.filter(c => c.state === stateFilter.value)
  }

  // 搜索
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    result = result.filter(c =>
      c.local_addr.toLowerCase().includes(q) ||
      c.remote_addr.toLowerCase().includes(q) ||
      c.process_name.toLowerCase().includes(q) ||
      c.pid.toString().includes(q) ||
      c.state.toLowerCase().includes(q)
    )
  }

  return result
})

// ============ 统计 ============

const tcpCount = computed(() => filteredConnections.value.filter(c => c.protocol === 'TCP').length)
const udpCount = computed(() => filteredConnections.value.filter(c => c.protocol === 'UDP').length)

const stateStats = computed(() => {
  const map = new Map<string, number>()
  for (const c of filteredConnections.value) {
    if (c.state) {
      map.set(c.state, (map.get(c.state) || 0) + 1)
    }
  }
  return Array.from(map.entries())
    .map(([state, count]) => ({ state, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 5)
})

const stateLabel = (state: string) => {
  const labels: Record<string, string> = {
    LISTENING: '监听', ESTABLISHED: '已建立', TIME_WAIT: '等待',
    CLOSE_WAIT: '关闭等待', SYN_SENT: '同步发送',
  }
  return labels[state] || state
}

const stateTagType = (state: string): 'success' | 'primary' | 'warning' | 'danger' | 'info' => {
  const map: Record<string, 'success' | 'primary' | 'warning' | 'danger' | 'info'> = {
    LISTENING: 'success', ESTABLISHED: 'primary', TIME_WAIT: 'warning',
    CLOSE_WAIT: 'danger', SYN_SENT: 'info',
  }
  return map[state] || 'info'
}

// ============ 数据采集 ============

const refresh = async () => {
  loading.value = true
  error.value = ''
  try {
    connections.value = await getNetworkConnections()
    lastRefresh.value = formatTimestamp()
  } catch (e) {
    error.value = '无法获取网络连接信息: ' + String(e)
  } finally {
    loading.value = false
  }
  // 重置自动刷新计时器
  if (autoRefresh.value && refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  }
}

// ============ 操作 ============

const handleKill = async (row: NetworkConnection) => {
  try {
    await ElMessageBox.confirm(
      `确定结束进程 "${row.process_name}" (PID: ${row.pid})？\n强制结束可能导致未保存的数据丢失。`,
      '结束进程确认',
      { type: 'warning', confirmButtonText: '结束', cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    if (result.success) {
      ElMessage.success(result.message)
    } else {
      ElMessage.error(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    refresh()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}

const handleReleasePort = async (row: NetworkConnection) => {
  const port = row.local_addr.split(':').pop() || ''
  try {
    await ElMessageBox.confirm(
      `确定释放端口 ${port}？将结束占用进程 "${row.process_name}" (PID: ${row.pid})。`,
      '释放端口确认',
      { type: 'warning', confirmButtonText: '释放', cancelButtonText: '取消' }
    )
  } catch {
    return
  }

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    if (result.success) {
      ElMessage.success(`端口 ${port} 已释放：${result.message}`)
    } else {
      ElMessage.error(result.message)
    }
    await new Promise(r => setTimeout(r, 300))
    refresh()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}

const exportCsv = async () => {
  const BOM = '\uFEFF'
  const header = '协议,本地地址,远程地址,状态,PID,进程名,进程路径'
  const rows = filteredConnections.value.map(c =>
    `${c.protocol},${c.local_addr},${c.remote_addr},${c.state},${c.pid},"${c.process_name}","${c.process_path}"`
  )
  const csv = BOM + header + '\n' + rows.join('\n')

  const now = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  const filename = `网络连接_${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.csv`

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const savedPath = await invoke<string>('save_text_with_dialog', { content: csv, filename })
    if (savedPath) {
      ElMessage.success('CSV 已导出')
    }
  } catch (e) {
    ElMessage.error('导出失败: ' + String(e))
  }
}

// ============ 自动刷新 ============

const toggleAutoRefresh = (val: boolean) => {
  if (val) {
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  } else {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  }
}

const restartAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = setInterval(refresh, refreshInterval.value * 1000)
  }
}

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

// 初始加载
refresh()
</script>

<style scoped>
.stats-row {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.stat-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 80px;
}

.stat-number {
  font-size: 22px;
  font-weight: 700;
  color: var(--accent-cyan);
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.mono-text {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
}

.text-secondary {
  color: var(--text-secondary);
}

.bottom-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.auto-refresh {
  display: flex;
  align-items: center;
  gap: 8px;
}

.bottom-actions {
  display: flex;
  gap: 8px;
}
</style>
```

- [ ] **Step 2: npm run build 验证**

Run: `npm run build`
Expected: vue-tsc 类型检查通过，vite build 成功

- [ ] **Step 3: Commit**

```bash
git add src/views/NetworkConnections.vue
git commit -m "feat(network): NetworkConnections.vue 工具页 - 连接列表/筛选/统计/自动刷新/结束进程/导出CSV"
```

---

## Task 6: 版本号 + README + backlog

**Files:**
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/feature-backlog.md`

- [ ] **Step 1: 修改 package.json version**

`package.json` line 3: `"version": "6.0.0"` → `"version": "6.1.0"`

- [ ] **Step 2: 修改 tauri.conf.json version**

`src-tauri/tauri.conf.json` line 3: `"version": "6.0.0"` → `"version": "6.1.0"`

- [ ] **Step 3: 修改 Cargo.toml version**

`src-tauri/Cargo.toml` line 3: `version = "6.0.0"` → `version = "6.1.0"`

- [ ] **Step 4: 修改 README.md 版本路线表**

在 `README.md` 的 V6.0 行之后，追加 V6.1 行：

```markdown
| V6.1 | ✅ | 网络连接查看器（TCP/UDP 全量连接，关联进程，筛选/自动刷新/结束进程/释放端口/导出CSV） | 2026-07-23 |
```

- [ ] **Step 5: 修改 feature-backlog.md**

1. 在已完成版本表追加 V6.1 行（同上）
2. 将 A10 条目标记为 `✅ 已完成 V6.1`

- [ ] **Step 6: npm run build 最终验证**

Run: `npm run build`
Expected: 全部通过

- [ ] **Step 7: Commit**

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml README.md docs/superpowers/plans/feature-backlog.md
git commit -m "chore(release): 发布 v6.1.0 - 新增 网络连接查看器"
```

---

## 全局约束

- **禁止硬编码颜色**：所有颜色使用 `theme.css` 中定义的 CSS 变量
- **禁止网络请求**：纯本地离线运行
- **子进程弹窗**：所有 Command 加 `CREATE_NO_WINDOW`
- **GBK 解码**：子进程输出用 `encoding_rs::GBK.decode()`
- **debug_log!**：关键路径加 `debug_log!()` 日志，release 模式自动移除
- **版本号同步**：package.json / tauri.conf.json / Cargo.toml 三处同步
- **snake_case 字段名**：Rust 结构体字段名不会自动转 camelCase，前端必须用 snake_case
- **camelCase 命令参数**：Rust 函数参数用 snake_case，前端 invoke 传 camelCase
- **ToolItem.category 可选**：`category?: string`，不要写成非可选
- **scoped 样式**：不能重复定义全局 `.tool-card` 等类名，只定义页面特有样式
- **addHistory 必须传 inputFull/outputFull**：调用 `store.addHistory()` 时必须同时传入 `inputFull: 完整输入, outputFull: 完整输出`