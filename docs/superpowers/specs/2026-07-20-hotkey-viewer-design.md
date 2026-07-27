# 全局快捷键占用查看器设计规格

**版本**: 1.0
**日期**: 2026-07-20
**状态**: 待实现
**Backlog 编号**: A8
**目标版本**: V5.8.0

---

## 1. 概述

### 1.1 目标
创建一个系统工具，列出当前 Windows 已注册的全局快捷键，标注占用进程/应用，支持搜索冲突。基于 `RegisterHotKey` 试探 + 立即释放的方式探测热键占用情况。

### 1.2 核心价值
- **冲突排查**：注册新全局热键前快速发现冲突，避免热键失效
- **占用可视化**：清楚知道哪些热键被哪些应用占用，方便清理冗余热键
- **快速查询**：搜索指定热键是否被占用，定位占用者

### 1.3 技术约束
- 纯本地离线运行，无网络请求
- 遵循 LitoBox 轻量级原则，新增依赖仅 `windows-sys`（约 0.3-0.5 MB）
- 探测过程不能影响其他应用已注册的热键（毫秒级 register-unregister）
- 严格遵守 AGENTS.md 规范：CREATE_NO_WINDOW + GBK 解码 + 时间驱动取消 + 事件+轮询兜底

---

## 2. 架构设计

### 2.1 独立工具页面
- **位置**：侧边栏 `system` 分类
- **菜单项**：`hotkey` - 快捷键占用
- **页面文件**：`src/views/HotkeyView.vue`
- **后端模块**：`src-tauri/src/hotkey_probe.rs`、`src-tauri/src/hotkey_data.rs`

### 2.2 数据流
```
进入页面 (onActivated)
  ↓
检查 store.hotkeyLastResult
  ├─ 有 → 立即填充表格
  └─ 无 → 显示空状态
  ↓
自动调用 invoke('hotkey_probe_start', { extraKeys })
  ↓
后端 spawn_blocking 启动探测线程：
  - 创建隐藏窗口 + 消息循环
  - 遍历候选集，每 200ms 检查取消 + emit 进度（时间驱动）
  - 占用热键立即查映射表 + 进程扫描
  - 完成 emit('hotkey-probe-complete', { results, stats })
  ↓
前端 listen 进度 + 2s 轮询 invoke('hotkey_probe_status') 兜底
  ↓
探测完成：填充表格 + 更新统计 + 记录历史
```

### 2.3 文件变更清单
1. **新增** `src-tauri/src/hotkey_probe.rs` - 探测核心（RegisterHotKey 试探 + 进程枚举）
2. **新增** `src-tauri/src/hotkey_data.rs` - 内置映射表（系统热键 + 应用热键 + 进程匹配规则）
3. **新增** `src/views/HotkeyView.vue` - 前端页面
4. **修改** `src-tauri/src/main.rs` - 注册 5 个 Tauri 命令
5. **修改** `src-tauri/Cargo.toml` - 添加 `windows-sys` 依赖
6. **修改** `src/store/index.ts` - 在 `TOOL_LIST` `system` 分类下追加 `hotkey` 条目
7. **修改** `src/router/index.ts` - 添加路由
8. **修改** `src/views/WorkflowView.vue` - 工作流集成（可选，本工具非典型工作流工具，记录但不强集成）

---

## 3. 后端设计

### 3.1 候选热键集生成

**默认常见集（约 250 个）**：
- Win + 0-9 / A-Z（36 个）
- Ctrl+Shift + A-Z（26 个）
- Ctrl+Alt + A-Z / 0-9（36 个）
- Alt+Shift + A-Z（26 个）
- Ctrl+Win + A-Z（26 个）
- F1-F12 × {Ctrl, Alt, Shift, Win}（48 个）
- 数字键盘 Alt+0-9 系列（~10 个，可选）

**自定义补充**：前端传入 `extraKeys: Vec<String>`（如 `["Ctrl+Shift+S", "Alt+F7"]`），后端解析为 `(mod, vk)` 合并到候选集。

**解析规则**：
- 修饰键不区分大小写：`ctrl` / `control` → `MOD_CONTROL`，`alt` → `MOD_ALT`，`shift` → `MOD_SHIFT`，`win` / `super` → `MOD_WIN`
- 主键：`A-Z` → `0x41-0x5A`，`0-9` → `0x30-0x39`，`F1-F12` → `VK_F1-VK_F12`
- 解析失败的字段跳过并记录 debug 日志

### 3.2 探测核心算法

```rust
// hotkey_probe.rs 核心结构
pub struct HotkeyProbe {
    cancel: Arc<AtomicBool>,
    results: Arc<Mutex<Vec<HotkeyResult>>>,
    progress: Arc<Mutex<ProbeProgress>>,
}

pub struct HotkeyResult {
    pub label: String,          // "Ctrl+Alt+A"
    pub mod_flags: u32,         // MOD_CONTROL | MOD_ALT
    pub vk: u32,                // 0x41
    pub status: HotkeyStatus,   // Available / Occupied / SystemReserved
    pub process_name: Option<String>,  // "WeChat.exe"
    pub process_display: Option<String>, // "微信"
    pub process_pid: Option<u32>,
    pub process_path: Option<String>,
    pub source: MatchSource,    // MapTable / ProcessScan / SelfRegistered / None
}

pub enum HotkeyStatus {
    Available,        // RegisterHotKey 成功
    Occupied,         // GetLastError == ERROR_HOTKEY_ALREADY_REGISTERED (1409)
    SystemReserved,   // 其他错误码（不可注册）
}

pub enum MatchSource {
    MapTable,         // 内置映射表命中
    ProcessScan,      // 进程扫描匹配
    SelfRegistered,   // LitoBox 自己注册的
    None,             // 未匹配到
}

pub struct ProbeProgress {
    pub done: usize,
    pub total: usize,
    pub last_key: String,
    pub is_finished: bool,
}

pub struct ProbeStats {
    pub total: usize,
    pub available: usize,
    pub occupied: usize,
    pub reserved: usize,
}
```

**探测循环**：
```rust
pub fn run_probe(
    candidates: Vec<(u32, u32, String)>,  // (mod, vk, label)
    extra_keys: Vec<String>,
    cancel: Arc<AtomicBool>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<HotkeyResult>, String> {
    // 1. 创建隐藏窗口 + 消息循环（独立线程处理 WM_HOTKEY）
    let hwnd = create_message_window()?;
    
    // 2. 查询 LitoBox 自身已注册热键
    let self_keys = query_self_registered_hotkeys();
    
    // 3. 启动进程枚举缓存（一次枚举，多次查询）
    let process_cache = enumerate_processes_once()?;
    
    // 4. 时间驱动探测循环
    let mut last_check = Instant::now();
    let mut results = Vec::with_capacity(candidates.len());
    for (i, (mod_, vk, label)) in candidates.iter().enumerate() {
        if cancel.load(Ordering::Acquire) {
            debug_log!("[hotkey_probe] cancelled at {}/{}", i, candidates.len());
            break;
        }
        
        let result = probe_one(hwnd, *mod_, *vk, label, &self_keys, &process_cache);
        results.push(result);
        
        // 200ms 时间驱动检查（AGENTS 经验 11）
        if last_check.elapsed() > Duration::from_millis(200) {
            emit_progress(&app_handle, i + 1, candidates.len(), label);
            last_check = Instant::now();
        }
    }
    
    emit_complete(&app_handle, &results);
    Ok(results)
}

fn probe_one(hwnd: HWND, mod_: u32, vk: u32, label: &str, 
             self_keys: &[(u32, u32)], process_cache: &ProcessCache) -> HotkeyResult {
    // 先检查是否 LitoBox 自身注册
    if self_keys.iter().any(|(m, v)| *m == mod_ && *v == vk) {
        return HotkeyResult {
            label: label.to_string(), mod_flags: mod_, vk,
            status: HotkeyStatus::Occupied,
            process_name: Some("LitoBox".to_string()),
            process_display: Some("LitoBox".to_string()),
            process_pid: Some(std::process::id()),
            process_path: Some(std::env::current_exe().unwrap().to_string_lossy().to_string()),
            source: MatchSource::SelfRegistered,
        };
    }
    
    // 尝试注册
    let ok = unsafe { RegisterHotKey(hwnd, PROBE_ID, mod_, vk) };
    if ok {
        unsafe { UnregisterHotKey(hwnd, PROBE_ID); }
        return HotkeyResult {
            label: label.to_string(), mod_flags: mod_, vk,
            status: HotkeyStatus::Available,
            process_name: None, process_display: None,
            process_pid: None, process_path: None,
            source: MatchSource::None,
        };
    }
    
    let err = unsafe { GetLastError() };
    debug_log!("[hotkey_probe] {} failed: err={}", label, err.0);
    
    // 三级回退定位进程
    let (process, source) = if err == ERROR_HOTKEY_ALREADY_REGISTERED {
        // 1. 映射表查询
        if let Some(p) = hotkey_data::lookup_maptable(mod_, vk) {
            (Some(p), MatchSource::MapTable)
        } 
        // 2. 进程扫描匹配
        else if let Some(p) = hotkey_data::scan_processes(mod_, vk, process_cache) {
            (Some(p), MatchSource::ProcessScan)
        }
        else {
            (None, MatchSource::None)
        }
    } else {
        // 系统保留
        if let Some(p) = hotkey_data::lookup_system_reserved(mod_, vk) {
            (Some(p), MatchSource::MapTable)
        } else {
            (None, MatchSource::None)
        }
    };
    
    let status = if err == ERROR_HOTKEY_ALREADY_REGISTERED {
        HotkeyStatus::Occupied
    } else {
        HotkeyStatus::SystemReserved
    };
    
    HotkeyResult {
        label: label.to_string(), mod_flags: mod_, vk, status,
        process_name: process.as_ref().map(|p| p.name.clone()),
        process_display: process.as_ref().map(|p| p.display.clone()),
        process_pid: process.as_ref().and_then(|p| p.pid),
        process_path: process.as_ref().and_then(|p| p.path.clone()),
        source,
    }
}
```

### 3.3 隐藏消息窗口

```rust
fn create_message_window() -> Result<HWND, String> {
    // 注册一个空窗口类，创建一个不显示的窗口用于接收 WM_HOTKEY
    // ponytail: 用 message-only window（HWND_MESSAGE）更轻量，但需要父窗口为 HWND_MESSAGE
    //           普通隐藏窗口也能工作，简单优先
    unsafe {
        let class_name: Vec<u16> = "LitoboxHotkeyProbe\0".encode_utf16().collect();
        let wc = WNDCLASSW {
            lpfnWndProc: Some(def_window_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hInstance: GetModuleHandleW(PCWSTR::null()),
            ..Default::default()
        };
        RegisterClassW(&wc);
        
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0, 0, 0, 0,
            HWND_MESSAGE,  // message-only window
            HMENU::null(),
            wc.hInstance,
            None,
        );
        if hwnd.0 == 0 {
            return Err(format!("CreateWindowExW failed: {}", GetLastError().0));
        }
        Ok(hwnd)
    }
}
```

> **注意**：`RegisterHotKey` 不需要消息循环也能成功注册（API 本身只检查冲突）。消息循环仅在需要处理热键触发时才需要。本工具 register 后立即 unregister，不会触发 WM_HOTKEY，因此**可以省略消息循环**。但 `HWND` 必须有效（不能为 null），所以仍需创建一个 message-only 窗口。

### 3.4 进程枚举缓存

```rust
pub struct ProcessInfo {
    pub name: String,        // "WeChat.exe"
    pub display: String,     // "微信"
    pub pid: u32,
    pub path: Option<String>,
}

pub struct ProcessCache {
    pub processes: Vec<ProcessInfo>,
}

pub fn enumerate_processes_once() -> Result<ProcessCache, String> {
    // 使用 EnumProcesses + OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW
    // 一次枚举所有进程，后续在 hotkey_data::scan_processes 中匹配
    // 路径获取失败时降级为 None（权限不足），不影响进程名匹配
}
```

### 3.5 取消机制

使用 `Arc<AtomicBool>` cancel flag：
- `hotkey_probe_cancel` 命令设置 `cancel.store(true, Ordering::Release)`
- 探测循环每 200ms 检查一次（时间驱动，AGENTS 经验 11）
- 取消后立即返回已探测的结果（部分结果也展示）

### 3.6 Tauri 命令

```rust
#[tauri::command]
pub async fn hotkey_probe_start(
    extra_keys: Vec<String>,
    state: tauri::State<'_, ProbeState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 返回 probe_id（UUID），启动后台线程
    // state 中保存 cancel flag + 当前 probe_id
}

#[tauri::command]
pub async fn hotkey_probe_cancel(
    state: tauri::State<'_, ProbeState>,
) -> Result<(), String> {
    // 设置 cancel flag
}

#[tauri::command]
pub async fn hotkey_probe_status(
    state: tauri::State<'_, ProbeState>,
) -> Result<ProbeProgress, String> {
    // 返回当前进度（轮询兜底用）
}

#[tauri::command]
pub async fn hotkey_probe_export_csv(
    results: Vec<HotkeyResult>,
) -> Result<String, String> {
    // 导出 CSV 到临时目录，返回路径
    // 复用项目现有 CSV 导出工具
}

#[tauri::command]
pub async fn hotkey_probe_get_results(
    state: tauri::State<'_, ProbeState>,
) -> Result<Vec<HotkeyResult>, String> {
    // 返回上次探测的完整结果
    // 兜底用：万一 complete 事件丢失，前端通过 status 轮询发现 is_finished 后调用此命令拉取
}
```

> 共 **5 个命令**：start / cancel / status / export_csv / get_results。其中 `get_results` 是兜底命令，正常流程通过 complete 事件拿到结果，仅在事件丢失时使用。

**ProbeState**：
```rust
pub struct ProbeState {
    pub cancel: Arc<AtomicBool>,
    pub current_probe_id: Arc<Mutex<Option<String>>>,
    pub progress: Arc<Mutex<ProbeProgress>>,
    pub last_results: Arc<Mutex<Vec<HotkeyResult>>>,
}
```

---

## 4. 内置映射表设计（hotkey_data.rs）

### 4.1 系统保留热键表

```rust
// 不可注册的系统热键（Win+L 锁屏、Win+P 投影等）
const SYSTEM_RESERVED: &[(u32, u32, &str)] = &[
    (MOD_WIN, 0x4C, "系统锁屏"),                // Win+L
    (MOD_WIN, 0x44, "显示桌面"),                // Win+D
    (MOD_WIN, 0x45, "资源管理器"),              // Win+E
    (MOD_WIN, 0x52, "运行对话框"),              // Win+R
    (MOD_WIN, 0x50, "投影切换"),                // Win+P
    (MOD_WIN, 0x54, "任务管理器"),              // Win+T (任务栏)
    (MOD_WIN, 0x55, "轻松访问中心"),            // Win+U
    (MOD_WIN, 0x56, "通知中心"),                // Win+V
    (MOD_WIN, 0x49, "设置"),                    // Win+I
    (MOD_WIN, VK_TAB, "任务视图"),              // Win+Tab
    (MOD_WIN, 0x4B, "剪贴板历史"),              // Win+K (Connect)
    (MOD_CONTROL | MOD_MENU | MOD_SHIFT, 0x2E, "安全选项"), // Ctrl+Alt+Shift+Del 等价
    (MOD_CONTROL | MOD_MENU, VK_DELETE, "安全选项"),        // Ctrl+Alt+Del
];
```

### 4.2 常见应用热键表

```rust
// 已知应用占用热键（无需进程扫描即可标注）
const APP_HOTKEYS: &[(u32, u32, &str, &[&str])] = &[
    // (mod, vk, display_name, [matching_process_names])
    (MOD_CONTROL | MOD_ALT, 0x41, "微信截图", &["WeChat.exe"]),
    (MOD_CONTROL | MOD_ALT, 0x53, "QQ 截图", &["QQ.exe", "QQProtect.exe"]),
    (MOD_ALT, 0x41, "微信录屏", &["WeChat.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x58, "IDA 截图", &["IDA64.exe", "IDAQ.exe"]),
    (MOD_WIN | MOD_SHIFT, 0x53, "Snipaste 截图", &["Snipaste.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x50, "VSCode 命令面板", &["Code.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x47, "VSCode Git", &["Code.exe"]),
    (MOD_WIN, 0x53, "Snipping Tool", &["ScreenClippingHost.exe"]),
    (MOD_CONTROL | MOD_SHIFT, 0x4C, "Cursor 命令面板", &["Cursor.exe"]),
    // ... 更多映射
];

pub fn lookup_maptable(mod_: u32, vk: u32) -> Option<ProcessInfo> { ... }
pub fn scan_processes(mod_: u32, vk: u32, cache: &ProcessCache) -> Option<ProcessInfo> { ... }
pub fn lookup_system_reserved(mod_: u32, vk: u32) -> Option<ProcessInfo> { ... }
```

### 4.3 LitoBox 自身热键查询

```rust
fn query_self_registered_hotkeys() -> Vec<(u32, u32)> {
    // 通过 tauri_plugin_global_shortcut 的 GlobalShortcutExt::all() 获取
    // 解析每个 Accelerator 的 mods + key
    // 注意：插件版本可能不直接暴露 all()，如不可用则降级为从 db 读取 shortcut 配置
}
```

> **降级**：若 `tauri_plugin_global_shortcut` 不暴露已注册列表，从 `litobox.db` 的 `shortcut_config` 表读取配置（与 hotkey.rs 注册时一致）。

---

## 5. 前端设计（HotkeyView.vue）

### 5.1 页面布局

```vue
<template>
  <div class="tool-container">
    <!-- 统计卡片 -->
    <div class="tool-card">
      <div class="card-header"><span class="card-title">快捷键占用概览</span></div>
      <div class="card-body">
        <div class="stats-grid">
          <div class="stat-item">总探测数 <strong>{{ stats.total }}</strong></div>
          <div class="stat-item">被占用 <strong class="danger">{{ stats.occupied }}</strong></div>
          <div class="stat-item">可注册 <strong class="success">{{ stats.available }}</strong></div>
          <div class="stat-item">系统保留 <strong class="warning">{{ stats.reserved }}</strong></div>
        </div>
      </div>
    </div>
    
    <!-- 搜索栏 + 操作区 -->
    <div class="tool-card sticky-card">
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 2">
            <el-input v-model="searchKeyword" placeholder="搜索热键或进程名" clearable />
          </div>
          <div class="action-group">
            <el-radio-group v-model="filterStatus" size="small">
              <el-radio-button label="">全部</el-radio-button>
              <el-radio-button label="occupied">被占用</el-radio-button>
              <el-radio-button label="available">可注册</el-radio-button>
              <el-radio-button label="reserved">系统保留</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <el-input v-model="extraKeysInput" placeholder="补充探测: Ctrl+Shift+S, Alt+F7" style="width: 240px" />
          </div>
          <div class="action-group">
            <el-button type="primary" :loading="isProbing" @click="startProbe">开始探测</el-button>
            <el-button v-if="isProbing" @click="cancelProbe">取消</el-button>
            <el-button @click="exportCsv" :disabled="!results.length">导出 CSV</el-button>
          </div>
        </div>
        <el-progress v-if="isProbing" :percentage="progressPercent" :format="formatProgress" />
      </div>
    </div>
    
    <!-- 结果表格 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">探测结果</span>
        <div class="card-actions">
          <span class="group-label">{{ filteredResults.length }} 条记录</span>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="pagedResults" stripe>
          <el-table-column prop="label" label="热键组合" width="140" />
          <el-table-column label="状态" width="120">
            <template #default="{ row }">
              <el-tag :type="statusTagType(row.status)">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="process_display" label="占用进程" width="160" />
          <el-table-column prop="process_path" label="进程路径" show-overflow-tooltip />
          <el-table-column label="来源" width="120">
            <template #default="{ row }">{{ sourceLabel(row.source) }}</template>
          </el-table-column>
        </el-table>
        <el-pagination
          v-model:current-page="currentPage"
          :page-size="pageSize"
          :total="filteredResults.length"
          layout="prev, pager, next, total"
          class="pagination-right"
        />
      </div>
    </div>
  </div>
</template>
```

### 5.2 状态管理

```typescript
// 在 src/store/index.ts 的 state 中追加
// 仅 Pinia 内存缓存，不持久化到 localStorage（关闭应用后丢失，符合"探测结果是实时快照"语义）
hotkeyLastResult: [] as HotkeyResult[],
hotkeyLastStats: null as ProbeStats | null,
```

```typescript
// HotkeyResult 前端类型（与 Rust struct 字段 snake_case 一致，AGENTS 经验 16）
interface HotkeyResult {
  label: string
  mod_flags: number
  vk: number
  status: 'Available' | 'Occupied' | 'SystemReserved'
  process_name: string | null
  process_display: string | null
  process_pid: number | null
  process_path: string | null
  source: 'MapTable' | 'ProcessScan' | 'SelfRegistered' | 'None'
}
```

### 5.3 生命周期与事件监听

```typescript
// 进入页面自动探测（onActivated 兼容 KeepAlive，AGENTS 经验 12）
onActivated(() => {
  // 先填充上次结果
  if (store.hotkeyLastResult.length) {
    results.value = store.hotkeyLastResult
    stats.value = store.hotkeyLastStats
  }
  // 自动启动探测
  startProbe()
})

// 事件监听 + 轮询兜底（AGENTS 经验 10）
let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null
let pollTimer: number | null = null
let done = false

onMounted(() => {
  unlistenProgress = listen('hotkey-probe-progress', (e) => {
    progress.value = e.payload
  })
  unlistenComplete = listen('hotkey-probe-complete', (e) => {
    done = true
    results.value = e.payload.results
    stats.value = e.payload.stats
    store.hotkeyLastResult = e.payload.results
    store.hotkeyLastStats = e.payload.stats
    isProbing.value = false
    stopPolling()
  })
})

function startPolling() {
  done = false
  pollTimer = window.setInterval(async () => {
    if (done) { stopPolling(); return }
    const status = await invoke('hotkey_probe_status')
    progress.value = status
    if (status.is_finished) {
      done = true
      stopPolling()
      // 拉取最终结果（兜底，万一 complete 事件丢失）
      const results = await invoke('hotkey_probe_get_results')
      // ...
    }
  }, 2000)
}

onDeactivated(() => {
  stopPolling()
  unlistenProgress?.()
  unlistenComplete?.()
})
```

### 5.4 过滤与搜索

纯前端过滤：
```typescript
const filteredResults = computed(() => {
  return results.value.filter(r => {
    if (filterStatus.value && r.status.toLowerCase() !== filterStatus.value) return false
    if (searchKeyword.value) {
      const kw = searchKeyword.value.toLowerCase()
      return r.label.toLowerCase().includes(kw) 
        || (r.process_display?.toLowerCase().includes(kw) ?? false)
        || (r.process_name?.toLowerCase().includes(kw) ?? false)
    }
    return true
  })
})
```

---

## 6. 数据库历史记录

按 AGENTS.md 规范，每次探测完成后记录一条历史：

```rust
do_note_create(
    note_type: "operation_history",
    tool: "hotkey_viewer",
    action: "probe",
    input_full: format!("候选集: {} 个 + 自定义: {:?}", total, extras),
    output_full: format!(
        "占用: {} | 可注册: {} | 系统保留: {}\n详细列表:\n{}",
        stats.occupied, stats.available, stats.reserved,
        results.iter().map(|r| format!("{} - {} - {}", 
            r.label, 
            match r.status { Available => "可注册", Occupied => "占用", SystemReserved => "系统保留" },
            r.process_display.as_deref().unwrap_or("-")
        )).collect::<Vec<_>>().join("\n")
    ),
    input_preview: truncated_input.chars().take(50).collect(),
    output_preview: truncated_output.chars().take(50).collect(),
)
```

> **注意**：`input_full` / `output_full` 必须完整传入，`input_preview` / `output_preview` 仅截断 50 字符用于列表展示。这关系到操作历史页面双击跳转还原数据（AGENTS.md 强制规范）。

---

## 7. 错误处理与降级

| 场景 | 处理 |
|------|------|
| `windows-sys` API 调用失败（创建窗口失败等） | 返回错误，前端显示"探测失败：{错误}"，不阻塞页面 |
| 进程枚举失败 | 降级为"未知进程"，仍展示热键被占用状态 |
| 进程路径获取失败（权限不足） | 路径字段留空，标注"权限不足" |
| 探测超时（>30s 未完成） | 自动取消，展示已探测部分结果 |
| KeepAlive 切换回来 | `onActivated` + watch(store.pendingHistoryRestore) 双保险 |
| 事件丢失 | listen + setInterval 2s 轮询 `hotkey_probe_status` 兜底 |
| 解析自定义热键失败 | 跳过该条，记录 debug_log，前端展示"无法解析：{原文}" |
| LitoBox 自身热键查询失败 | 跳过自身检查，仅依赖映射表 + 进程扫描 |

---

## 8. 依赖与体积影响

### 8.1 Cargo.toml 新增

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_ToolHelp",
] }
```

### 8.2 体积增量
- `windows-sys` 仅引入必要 feature，增量约 0.3-0.5 MB
- 不影响 53 MB 基础定位
- 不引入 `windows` crate（更重，含 COM 等不需要的能力）

### 8.3 版本号
- `package.json`: 5.7.0 → **5.8.0**
- `Cargo.toml`: 5.7.0 → **5.8.0**
- `tauri.conf.json`: 5.7.0 → **5.8.0**
- `README.md`: 添加 V5.8.0 功能阶段记录条目
- `docs/superpowers/plans/feature-backlog.md`: A8 标记 ✅ 已完成 V5.8

按 project_memory 规则：新增侧边栏菜单项触发 minor 版本升级。

---

## 9. 测试与验证

按 AGENTS.md "非平凡逻辑必须留一个 runnable check" 规则：

### 9.1 自检脚本
在 `src-tauri/src/hotkey_probe.rs` 末尾添加 `#[cfg(test)]` 模块，包含：
- `test_parse_accelerator` - 验证 "Ctrl+Shift+S" 解析为正确 (mod, vk)
- `test_lookup_maptable` - 验证 Win+L 命中系统保留表
- `test_scan_processes` - 验证进程匹配逻辑（mock ProcessCache）

### 9.2 手动验证清单
- [ ] 进入页面自动探测，进度条实时更新
- [ ] 表格正确显示 Win+L 为系统保留
- [ ] 启动微信后探测，Ctrl+Alt+A 显示占用=微信
- [ ] 点击"取消"立即停止探测
- [ ] 搜索"微信"过滤出所有微信相关热键
- [ ] 导出 CSV 文件内容正确
- [ ] 切换其他工具再回来，表格保留上次结果
- [ ] 操作历史页面双击本条记录能还原（验证 inputFull/outputFull 完整）

---

## 10. 工作流集成

本工具为查询型工具，**不强集成工作流**（不会作为工作流步骤被执行）。但按 AGENTS.md 规范仍记录到 SQLite 操作历史。

> 如果未来需要工作流集成（如"工作流前先检查某热键是否空闲"），可补充 `hotkey_check_single` 命令。

---

## 11. 待定问题（无）

设计阶段所有核心决策已通过 brainstorming 确认：
- ✅ 探测范围：常见集 + 自定义补充
- ✅ 进程定位：映射表 + 进程扫描
- ✅ 探测交互：进入页面自动探测 + 实时进度 + 上次结果缓存
- ✅ 技术方案：RegisterHotKey 试探 + 立即释放
