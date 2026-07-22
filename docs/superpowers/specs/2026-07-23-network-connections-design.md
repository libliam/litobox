# 网络连接查看器设计文档

> **版本**: V6.1  
> **日期**: 2026-07-23  
> **Backlog**: A10 网络连接查看器

## 概述

新增独立工具页「网络连接」，列出所有 TCP/UDP 连接（listening/established/time_wait 等），关联进程名/PID/路径，支持按端口/状态/协议筛选、自动刷新、结束进程/释放端口、导出 CSV。与 V4.4 端口释放（进程 kill）互补，提供网络连接全貌视图。

## 技术架构

```
┌─ Frontend (Vue 3) ───────────────────────────────────────────┐
│  NetworkConnections.vue                                       │
│  ├─ 统计概览卡片 (总数/协议分布/状态分布)                       │
│  ├─ 搜索筛选栏 (端口/进程名/PID/协议/状态下拉)                  │
│  ├─ 连接表格 (el-table, 可排序)                                │
│  │   └─ 操作列: 结束进程 / 释放端口 / 复制地址                 │
│  └─ 底部栏: 自动刷新开关 + 间隔选择 + 导出CSV + 刷新时间        │
├───────────────────────────────────────────────────────────────┤
│  systemInfoClient.ts (扩展)                                    │
│  invoke('get_network_connections') → NetworkConnection[]      │
│  invoke('kill_process') → 复用 V4.4                            │
└───────────────────────────────────────────────────────────────┘
                              │
┌─ Backend (Rust) ─────────────────────────────────────────────┐
│  network_connections.rs (新建, ~200 行)                       │
│  get_network_connections() → Vec<NetworkConnection>           │
│  ├─ 主路径: netstat -ano (解析文本, CREATE_NO_WINDOW)         │
│  ├─ fallback: Get-NetTCPConnection + Get-NetUDPEndpoint       │
│  └─ 进程路径: sysinfo::Process::exe() best-effort             │
└──────────────────────────────────────────────────────────────┘
```

## 数据结构

### Rust 后端

```rust
#[derive(Debug, Clone, Serialize)]
pub struct NetworkConnection {
    pub protocol: String,       // "TCP" / "UDP"
    pub local_addr: String,     // "0.0.0.0:8080"
    pub remote_addr: String,    // "192.168.1.5:443" (UDP 时为 "*:*")
    pub state: String,          // "LISTENING"/"ESTABLISHED"/"TIME_WAIT"/"CLOSE_WAIT"/"SYN_SENT"/"" (UDP 无状态)
    pub pid: u32,
    pub process_name: String,   // "node.exe"
    pub process_path: String,   // "C:\Program Files\nodejs\node.exe" (权限不足时为空)
}
```

### 前端 TypeScript

```ts
export interface NetworkConnection {
  protocol: string
  local_addr: string
  remote_addr: string
  state: string
  pid: number
  process_name: string
  process_path: string
}
```

## 功能详述

### 1. 数据采集（后端）

**主路径：`netstat -ano`**

```
netstat -ano
```

输出格式（中文 Windows）:
```
活动连接
  协议  本地地址          外部地址         状态           PID
  TCP    0.0.0.0:135      0.0.0.0:0       LISTENING      1234
  TCP    192.168.1.5:54321 13.107.42.14:443 ESTABLISHED   5678
  UDP    0.0.0.0:5353      *:*                            9012
```

解析规则：
- 跳过前 4 行（表头）
- 按空白字符分割（`split_whitespace`）
- TCP: 协议 + 本地地址 + 外部地址 + 状态 + PID（5 列）
- UDP: 协议 + 本地地址 + `*:*` + PID（4 列，无状态）
- 状态映射：LISTENING→Listen, ESTABLISHED→Established, TIME_WAIT→TimeWait, CLOSE_WAIT→CloseWait, SYN_SENT→SynSent, FIN_WAIT_1→FinWait1, FIN_WAIT_2→FinWait2, LAST_ACK→LastAck

**Fallback：PowerShell cmdlet**

```powershell
Get-NetTCPConnection | Select-Object State,LocalAddress,LocalPort,RemoteAddress,RemotePort,OwningProcess | ConvertTo-Json
Get-NetUDPEndpoint | Select-Object LocalAddress,LocalPort,OwningProcess | ConvertTo-Json
```

- TCP 状态数字码映射同现有 `tcp_state_name()` 函数
- 两个 cmdlet 输出合并为统一 `Vec<NetworkConnection>`

**进程信息补全：**

- 采集完成后，用 `sysinfo::System::new_all()` 构建进程快照
- `process_name`：`sys.process(Pid::from_u32(pid)).map(|p| p.name())` → 进程名
- `process_path`：`sys.process(Pid::from_u32(pid)).and_then(|p| p.exe())` → 完整路径
- 进程已退出：`process_name` = "(已退出)"，`process_path` = ""，pid 保留
- 路径获取失败（权限不足）：`process_path` = ""

### 2. 统计概览（前端）

顶部卡片行，三列：

| 统计项 | 说明 |
|--------|------|
| 连接总数 | `filteredConnections.length` |
| 按协议分布 | `TCP: N / UDP: M` |
| 按状态分布 | `LISTENING: N / ESTABLISHED: M / TIME_WAIT: K / ...` |

统计基于当前筛选后的数据。

### 3. 搜索筛选（前端）

筛选栏控件：

| 控件 | 类型 | 说明 |
|------|------|------|
| 搜索框 | `el-input` | 按端口号、进程名、PID、本地地址模糊匹配 |
| 协议筛选 | `el-select` | 全部 / TCP / UDP |
| 状态筛选 | `el-select` | 全部 / LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / SYN_SENT / 其他 |

所有筛选纯前端计算，`computed` 派生 `filteredConnections`。

### 4. 连接表格

`el-table` 列定义：

| 列名 | 宽度 | 说明 |
|------|------|------|
| 协议 | 70px | `el-tag` 展示，TCP 蓝色/UDP 绿色 |
| 本地地址 | 160px | `ip:port` 格式，可复制 |
| 远程地址 | 160px | `ip:port` 格式，UDP 显示 `*:*` |
| 状态 | 110px | `el-tag` 展示，颜色按状态区分 |
| PID | 70px | 纯数字 |
| 进程名 | 140px | `process_name`，已退出显示 "(已退出)" |
| 进程路径 | 200px | `process_path`，`show-overflow-tooltip` |
| 操作 | 160px | 固定右侧 |

**状态颜色映射：**
- LISTENING → `success`（绿色）
- ESTABLISHED → `primary`（蓝色）
- TIME_WAIT → `warning`（橙色）
- CLOSE_WAIT → `danger`（红色）
- 其他 → `info`（灰色）

**操作按钮：**
- 结束进程：`el-button type="danger" size="small" link`，pid 为 0 或进程已退出时禁用
- 释放端口：同结束进程，弹窗措辞不同
- 复制地址：`el-button size="small" link`，复制 `local_addr` 到剪贴板

**表格排序：** 支持按 PID、协议、状态列表头排序。

### 5. 自动刷新

- 开关：`el-switch` 
- 间隔选择：`el-select`（5s / 10s / 30s），默认 5s
- 实现：`setInterval` + `ref` 持有定时器 ID
- 手动刷新时重置定时器（避免重复请求）
- 组件 `onUnmounted` 清除定时器

### 6. 结束进程 / 释放端口

- 复用 V4.4 的 `killProcess(pid)` 命令
- 结束进程弹窗：`确定结束进程 "xxx.exe" (PID: 1234)？`
- 释放端口弹窗：`确定释放端口 8080？将结束占用进程 "xxx.exe" (PID: 1234)`
- 使用 `ElMessageBox.confirm` + `type: 'warning'`
- 操作后等待 300ms 自动刷新列表

### 7. 导出 CSV

- 前端生成 CSV 内容（BOM + 表头 + 数据行）
- 列：协议,本地地址,远程地址,状态,PID,进程名,进程路径
- 通过 `save_text_with_dialog` 保存文件
- 文件名默认：`网络连接_2026-07-23_143000.csv`

### 8. 错误处理

| 场景 | 处理 |
|------|------|
| 后端采集失败 | 显示错误卡片，提示"无法获取网络连接信息" |
| netstat 路径失败 | 自动 fallback 到 PowerShell |
| 两个路径都失败 | 返回空列表 + 错误消息 |
| 进程已退出 | 进程名显示 "(已退出)"，路径为空，操作按钮禁用 |
| 进程路径权限不足 | 路径显示为空，不影响其他字段 |
| 非管理员结束系统进程 | `kill_process` 返回友好错误提示 |
| 子进程弹窗 | 所有 Command 加 `CREATE_NO_WINDOW` |
| 网络连接数极大 | `el-table max-height="600"`，内置虚拟滚动 |

## 文件变更清单

| 文件 | 动作 | 职责 |
|------|------|------|
| `src-tauri/src/network_connections.rs` | 新建 | 后端命令 `get_network_connections` |
| `src-tauri/src/main.rs` | 改 | 注册 `mod` + `.invoke_handler` 加命令 |
| `src/views/NetworkConnections.vue` | 新建 | 前端工具页 |
| `src/utils/systemInfoClient.ts` | 改 | 加类型 + `getNetworkConnections()` |
| `src/store/index.ts` | 改 | TOOL_LIST 加 networkConnections 条目 |
| `src/components/SidebarNav.vue` | 不改 | 自动按 TOOL_LIST 顺序渲染 |
| `package.json` | 改 | 版本 6.0.0 → 6.1.0 |
| `src-tauri/tauri.conf.json` | 改 | 版本同步 |
| `src-tauri/Cargo.toml` | 改 | 版本同步 |
| `README.md` | 改 | 版本路线表加 V6.1 |
| `docs/superpowers/plans/feature-backlog.md` | 改 | A10 标记完成 + 已完成版本表 |

## 验收清单

1. 打开网络连接工具 → 显示 TCP+UDP 连接列表，含进程名/路径
2. 按端口号搜索 → 表格实时过滤
3. 按协议/状态下拉筛选 → 表格实时过滤
4. 按 PID 排序 → 表格正确排序
5. 点击"结束进程" → 确认弹窗 → 成功后进程被结束
6. 点击"释放端口" → 确认弹窗 → 成功后端口释放
7. 开启自动刷新 → 每 5s 自动刷新列表
8. 切换自动刷新间隔 → 刷新频率变化
9. 关闭自动刷新 → 停止自动刷新
10. 组件销毁（切换工具） → 自动刷新定时器清除
11. 点击"导出 CSV" → 保存文件，内容正确（BOM + 数据行）
12. 进程已退出 → 显示 "(已退出)"，操作按钮禁用
13. netstat 失败 → 自动 fallback 到 PowerShell
14. 深色/浅色主题 → 表格样式正常