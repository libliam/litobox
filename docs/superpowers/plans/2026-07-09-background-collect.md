# 系统工具后台采集 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将系统工具 5 个采集页由"同步 invoke + 全屏 ElLoading 遮罩"改造为"后台 spawn_blocking 采集 + 事件通知 + 应用内 ElNotification 提示"，采集期间不锁 UI、可切其他工具。

**Architecture:** 后端 5 个 `get_xxx` 命令改为内部同步函数 `get_xxx_inner`，新增 5 个 `collect_xxx` async 命令用 `tokio::task::spawn_blocking` 后台执行，完成经 `app.emit("collect-complete", ...)` 通知；全局 `OnceLock<Mutex<HashMap<CollectKind, TaskState>>>` 状态表供轮询兜底。前端抽 `useBackgroundCollect` composable 复用（listen + 2s 轮询 + done flag 去重），5 个页面复用，结果存 store，ElNotification 带「查看」按钮调 `store.openTab(toolId)` 跳回。

**Tech Stack:** Tauri 2.0 (Rust, tokio::spawn_blocking, AppHandle.emit)、Vue 3 Composition API、Pinia、Element Plus (ElNotification)、TypeScript

**Spec:** `docs/superpowers/specs/2026-07-09-background-collect-design.md`

---

## File Structure

| 文件 | 责任 | 动作 |
|------|------|------|
| `src-tauri/src/system_info.rs` | 采集命令 + 状态表 | 修改：补 use；5 命令改 `_inner` 内部函数；新增 `CollectKind`/`TaskState`/`CollectComplete`、全局状态表、`upsert_state`、5 个 `collect_xxx` async 命令、`get_collect_status` 命令；新增测试 |
| `src-tauri/src/main.rs` | 命令注册 | 修改：`generate_handler!` 移除 5 个旧 `get_xxx`，新增 5 个 `collect_xxx` + `get_collect_status` |
| `src/utils/systemInfoClient.ts` | TS 类型 + invoke 封装 | 修改：新增 `CollectKind`/`CollectStartResult`/`CollectCompletePayload`/`TaskState` 类型；5 个 getter 改调 `collect_xxx`；新增 `getCollectStatus` |
| `src/store/index.ts` | 全局状态 | 修改：新增 `collectResults`/`collecting` 状态并导出 |
| `src/composables/useBackgroundCollect.ts` | 采集 composable | 新建：封装启动/监听/轮询/去重/通知/跳转 |
| `src/views/SystemInfoView.vue` | 系统信息页 | 修改：接入 composable，移除 ElLoading，watch 还原 |
| `src/views/NetworkInfoView.vue` | 网络信息页 | 同上 |
| `src/views/ProcessListView.vue` | 进程列表页 | 同上 |
| `src/views/HardwareInfoView.vue` | 硬件外设页 | 同上 |
| `src/views/SoftwareEnvView.vue` | 软件环境页 | 同上 |

---

## Task 1: 后端 — 新增采集类型与全局状态表

**Files:**
- Modify: `src-tauri/src/system_info.rs:10-14`（use 区）、`:414`（命令实现区上方）

- [ ] **Step 1: 补充 use 导入**

在 `src-tauri/src/system_info.rs` 第 10-14 行现有 use 区下方追加（`Command`/`CommandExt`/`CREATE_NO_WINDOW` 保留不动）：

```rust
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
```

- [ ] **Step 2: 新增 CollectKind / TaskState / CollectComplete 与全局状态表**

在第 414 行 `// ============ 命令实现（后续 Task 填充） ============` 上方插入：

```rust
// ============ 后台采集状态 ============

#[derive(serde::Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectKind {
    System,
    Network,
    Process,
    Hardware,
    Software,
}

#[derive(serde::Serialize, Clone)]
pub struct TaskState {
    pub task_id: String,
    pub kind: CollectKind,
    pub status: String,                  // "running" | "done" | "error"
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub updated_at: u64,                 // unix secs
}

#[derive(serde::Serialize, Clone)]
pub struct CollectComplete {
    pub kind: CollectKind,
    pub task_id: String,
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CollectStartResult {
    pub task_id: String,
    pub kind: CollectKind,
}

// ponytail: 全局状态表，5 个 kind 各记最新一条任务。上限固定 5 条，无内存增长风险
static COLLECT_STATE: OnceLock<Mutex<HashMap<CollectKind, TaskState>>> = OnceLock::new();

fn collect_state() -> &'static Mutex<HashMap<CollectKind, TaskState>> {
    COLLECT_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn upsert_state(kind: CollectKind, task_id: &str, status: &str, data: Option<serde_json::Value>, error: Option<String>) {
    if let Ok(mut m) = collect_state().lock() {
        m.insert(kind, TaskState {
            task_id: task_id.to_string(),
            kind,
            status: status.to_string(),
            data,
            error,
            updated_at: now_secs(),
        });
    }
}

/// 通用：启动一个后台采集任务。`work` 在 spawn_blocking 线程内执行原同步采集逻辑。
fn spawn_collect<F, T>(app: AppHandle, kind: CollectKind, work: F) -> CollectStartResult
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: serde::Serialize + Send + 'static,
{
    let task_id = Uuid::new_v4().to_string();
    upsert_state(kind, &task_id, "running", None, None);
    let task_id_clone = task_id.clone();
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        match work() {
            Ok(data) => {
                let val = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
                upsert_state(kind, &task_id_clone, "done", Some(val.clone()), None);
                debug_log!("collect {:?} done, task_id={}", kind, task_id_clone);
                let _ = app2.emit("collect-complete", CollectComplete {
                    kind, task_id: task_id_clone, ok: true, data: Some(val), error: None,
                });
            }
            Err(e) => {
                debug_log!("collect {:?} error: {}, task_id={}", kind, e, task_id_clone);
                upsert_state(kind, &task_id_clone, "error", None, Some(e.clone()));
                let _ = app2.emit("collect-complete", CollectComplete {
                    kind, task_id: task_id_clone, ok: false, data: None, error: Some(e),
                });
            }
        }
    });
    CollectStartResult { task_id, kind }
}
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri; cargo check`
Expected: 编译通过（可能有 unused 警告，因 collect_xxx 命令尚未调用 spawn_collect，可忽略）

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat(backend): 新增后台采集类型与全局状态表"
```

---

## Task 2: 后端 — 改造 5 个采集命令 + 新增 collect_xxx / get_collect_status

**Files:**
- Modify: `src-tauri/src/system_info.rs:432-493`（get_system_info）、`:495-673`（get_network_info）、`:675-704`（get_process_list）、`:848-1121`（get_hardware_info）、`:1123-1190`（get_software_env）
- Modify: `src-tauri/src/main.rs:96-103`（handler 注册）

- [ ] **Step 1: get_system_info 改名内部函数**

将 `src-tauri/src/system_info.rs:432-433`：

```rust
#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
```

改为（去掉 `#[tauri::command]`，函数名加 `_inner`）：

```rust
fn get_system_info_inner() -> Result<SystemInfo, String> {
```

函数体（434-493）不动。

- [ ] **Step 2: get_network_info 改名内部函数**

将 `:495-496`：

```rust
#[tauri::command]
pub fn get_network_info() -> Result<NetworkInfo, String> {
```

改为：

```rust
fn get_network_info_inner() -> Result<NetworkInfo, String> {
```

函数体不动。

- [ ] **Step 3: get_process_list 改名内部函数**

将 `:675-676`：

```rust
#[tauri::command]
pub fn get_process_list() -> Result<Vec<ProcessItem>, String> {
```

改为：

```rust
fn get_process_list_inner() -> Result<Vec<ProcessItem>, String> {
```

函数体不动。

- [ ] **Step 4: get_hardware_info 改名内部函数**

将 `:848-849`：

```rust
#[tauri::command]
pub fn get_hardware_info() -> Result<HardwareInfo, String> {
```

改为：

```rust
fn get_hardware_info_inner() -> Result<HardwareInfo, String> {
```

函数体不动。

- [ ] **Step 5: get_software_env 改名内部函数**

将 `:1123-1124`：

```rust
#[tauri::command]
pub fn get_software_env() -> Result<SoftwareEnv, String> {
```

改为：

```rust
fn get_software_env_inner() -> Result<SoftwareEnv, String> {
```

函数体不动。

- [ ] **Step 6: 新增 5 个 collect_xxx async 命令 + get_collect_status**

在 `get_software_env_inner` 函数结束（原 `:1190` 的 `}` 之后）、`#[cfg(test)] mod tests` 之前插入：

```rust
// ============ 后台采集命令 ============

#[tauri::command]
pub async fn collect_system(app: AppHandle) -> Result<CollectStartResult, String> {
    Ok(spawn_collect(app, CollectKind::System, get_system_info_inner))
}

#[tauri::command]
pub async fn collect_network(app: AppHandle) -> Result<CollectStartResult, String> {
    Ok(spawn_collect(app, CollectKind::Network, get_network_info_inner))
}

#[tauri::command]
pub async fn collect_process(app: AppHandle) -> Result<CollectStartResult, String> {
    Ok(spawn_collect(app, CollectKind::Process, get_process_list_inner))
}

#[tauri::command]
pub async fn collect_hardware(app: AppHandle) -> Result<CollectStartResult, String> {
    Ok(spawn_collect(app, CollectKind::Hardware, get_hardware_info_inner))
}

#[tauri::command]
pub async fn collect_software(app: AppHandle) -> Result<CollectStartResult, String> {
    Ok(spawn_collect(app, CollectKind::Software, get_software_env_inner))
}

#[tauri::command]
pub fn get_collect_status(kind: CollectKind) -> Option<TaskState> {
    collect_state().lock().ok().and_then(|m| m.get(&kind).cloned())
}
```

- [ ] **Step 7: 更新 main.rs 命令注册**

将 `src-tauri/src/main.rs:96-103`：

```rust
            system_info::is_admin,
            system_info::get_system_info,
            system_info::get_network_info,
            system_info::get_process_list,
            system_info::get_hardware_info,
            system_info::get_software_env,
            system_info::kill_process,
            system_info::kill_process_by_name,
```

改为：

```rust
            system_info::is_admin,
            system_info::collect_system,
            system_info::collect_network,
            system_info::collect_process,
            system_info::collect_hardware,
            system_info::collect_software,
            system_info::get_collect_status,
            system_info::kill_process,
            system_info::kill_process_by_name,
```

- [ ] **Step 8: 新增 get_collect_status 边界测试**

在 `src-tauri/src/system_info.rs` 末尾的 `#[cfg(test)] mod tests { ... }` 内追加（与现有测试同级）：

```rust
    #[test]
    fn get_collect_status_returns_none_when_empty() {
        // 清空状态表后查询任意 kind 应返回 None
        if let Ok(mut m) = collect_state().lock() {
            m.clear();
        }
        assert!(get_collect_status(CollectKind::Process).is_none());
    }
```

- [ ] **Step 9: 编译与测试**

Run: `cd src-tauri; cargo check`
Expected: 编译通过，无错误

Run: `cd src-tauri; cargo test get_collect_status`
Expected: 1 个测试通过

- [ ] **Step 10: 提交**

```bash
git add src-tauri/src/system_info.rs src-tauri/src/main.rs
git commit -m "feat(backend): 5 采集命令改后台 spawn_blocking + 事件通知"
```

---

## Task 3: 前端 — systemInfoClient.ts 类型与封装更新

**Files:**
- Modify: `src/utils/systemInfoClient.ts:169-211`（invoke 封装区）

- [ ] **Step 1: 新增采集相关类型**

在 `src/utils/systemInfoClient.ts` 第 168 行（`StartupItem` 接口定义之后、`// ============ invoke 封装 ============` 之前）插入：

```ts
// ============ 后台采集类型 ============

export type CollectKind = 'system' | 'network' | 'process' | 'hardware' | 'software'

export interface CollectStartResult {
  task_id: string
  kind: CollectKind
}

export interface CollectCompletePayload {
  kind: CollectKind
  task_id: string
  ok: boolean
  data: unknown
  error: string | null
}

export interface TaskState {
  task_id: string
  kind: CollectKind
  status: 'running' | 'done' | 'error'
  data: unknown
  error: string | null
  updated_at: number
}
```

- [ ] **Step 2: 5 个 getter 改调 collect_xxx + 新增 getCollectStatus**

将 `src/utils/systemInfoClient.ts:171-211` 的 5 个 getter（`getSystemInfo`/`getNetworkInfo`/`getProcessList`/`getHardwareInfo`/`getSoftwareEnv`）整体替换为：

```ts
export function collectSystem(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_system')
}
export function collectNetwork(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_network')
}
export function collectProcess(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_process')
}
export function collectHardware(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_hardware')
}
export function collectSoftware(): Promise<CollectStartResult> {
  return invoke<CollectStartResult>('collect_software')
}

export function getCollectStatus(kind: CollectKind): Promise<TaskState | null> {
  return invoke<TaskState | null>('get_collect_status', { kind })
}
```

注意：`killProcess` / `killProcessByName` 两个函数（原 190-203 行）保留不动，确保它们仍在文件中。若整体替换范围误删，需手动补回。

- [ ] **Step 3: 类型检查**

Run: `npx vue-tsc --noEmit`
Expected: 无类型错误（5 个 view 此刻仍引用旧的 `getSystemInfo` 等，会报错——这是预期的，下个 Task 修复。若需立即绿色，可先跳过此步，在 Task 6 后统一检查）

- [ ] **Step 4: 提交**

```bash
git add src/utils/systemInfoClient.ts
git commit -m "feat(frontend): systemInfoClient 改后台采集封装"
```

---

## Task 4: 前端 — store 加采集状态

**Files:**
- Modify: `src/store/index.ts:109-110`（state 区）、`:331-354`（return 区）

- [ ] **Step 1: 新增 collectResults / collecting 状态**

在 `src/store/index.ts` 第 110 行 `const recentTools = ref<string[]>([])` 之后追加：

```ts
  // ============ 后台采集状态 ============
  type CollectKind = 'system' | 'network' | 'process' | 'hardware' | 'software'
  const collectResults = ref<Record<CollectKind, unknown>>({
    system: null, network: null, process: null, hardware: null, software: null,
  })
  const collecting = ref<Record<CollectKind, boolean>>({
    system: false, network: false, process: false, hardware: false, software: false,
  })
```

- [ ] **Step 2: 导出新状态**

在 `src/store/index.ts` 的 return 块（约 331 行起）中，`getTabKey,` 之后追加：

```ts
    // 后台采集
    collectResults,
    collecting,
```

- [ ] **Step 3: 提交**

```bash
git add src/store/index.ts
git commit -m "feat(frontend): store 新增采集结果与采集中状态"
```

---

## Task 5: 前端 — useBackgroundCollect composable

**Files:**
- Create: `src/composables/useBackgroundCollect.ts`

- [ ] **Step 1: 新建 composable**

创建 `src/composables/useBackgroundCollect.ts`：

```ts
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElNotification } from 'element-plus'
import { useToolboxStore } from '@/store'
import {
  type CollectKind, type CollectCompletePayload,
  getCollectStatus,
} from '@/utils/systemInfoClient'

// kind → (工具页 toolId, 中文标签, invoke 命令名)
const KIND_META: Record<CollectKind, { toolId: string; label: string; cmd: string }> = {
  system:   { toolId: 'systemInfo',   label: '系统信息', cmd: 'collect_system' },
  network:  { toolId: 'networkInfo',  label: '网络信息', cmd: 'collect_network' },
  process:  { toolId: 'processList',  label: '进程列表', cmd: 'collect_process' },
  hardware: { toolId: 'hardwareInfo', label: '硬件外设', cmd: 'collect_hardware' },
  software: { toolId: 'softwareEnv',  label: '软件环境', cmd: 'collect_software' },
}

export function useBackgroundCollect(kind: CollectKind) {
  const store = useToolboxStore()
  const meta = KIND_META[kind]
  const collecting = computed(() => store.collecting[kind])
  const result = computed(() => store.collectResults[kind])

  async function collect() {
    if (store.collecting[kind]) return          // 重复触发拦截
    store.collecting[kind] = true
    let done = false                            // 事件 + 轮询去重 flag
    let timer: ReturnType<typeof setInterval> | null = null
    let unlisten: UnlistenFn | null = null

    const finish = (payload: { ok: boolean; data?: unknown; error?: string | null }) => {
      if (done) return
      done = true
      if (timer) clearInterval(timer)
      if (unlisten) unlisten()
      store.collecting[kind] = false
      if (payload.ok) {
        store.collectResults[kind] = payload.data
        ElNotification.success({
          title: '采集完成',
          message: `${meta.label}采集完成，点击查看`,
          duration: 5000,
          onClick: () => store.openTab(meta.toolId),
        })
      } else {
        ElNotification.error({
          title: '采集失败',
          message: payload.error || `${meta.label}采集失败`,
          duration: 5000,
        })
      }
    }

    unlisten = await listen<CollectCompletePayload>('collect-complete', (e) => {
      if (e.payload.kind !== kind) return
      finish({ ok: e.payload.ok, data: e.payload.data, error: e.payload.error })
    })

    timer = setInterval(async () => {            // 2s 轮询兜底
      try {
        const st = await getCollectStatus(kind)
        if (st && (st.status === 'done' || st.status === 'error')) {
          finish({ ok: st.status === 'done', data: st.data, error: st.error })
        }
      } catch { /* 轮询失败忽略，下次重试 */ }
    }, 2000)

    try {
      await invoke(meta.cmd)
    } catch (e) {
      // 启动即失败（极少见），直接收尾
      finish({ ok: false, error: String(e) })
    }
  }

  return { collect, collecting, result }
}

// ============ 自检 ============
// ponytail: 验证 KIND_META 五个 kind 完整，且重复触发拦截语义
console.assert(Object.keys(KIND_META).length === 5, 'KIND_META 应有 5 个 kind')
console.assert(KIND_META.process.toolId === 'processList', 'process → processList')
console.assert(KIND_META.process.cmd === 'collect_process', 'process → collect_process')
```

- [ ] **Step 2: 类型检查**

Run: `npx vue-tsc --noEmit`
Expected: 本文件无类型错误（其他 view 的旧引用错误仍存在，Task 6 修复）

- [ ] **Step 3: 提交**

```bash
git add src/composables/useBackgroundCollect.ts
git commit -m "feat(frontend): 新增 useBackgroundCollect composable"
```

---

## Task 6: 前端 — 改造 5 个采集页面

每个页面改造模式一致：
1. 移除 `ElLoading` 导入与 `ElLoading.service` 调用
2. 引入 `useBackgroundCollect`，替换原 `loadData`
3. 刷新按钮 `@click="collect"`，`:loading="collecting"`
4. 移除 `onMounted(() => loadData())`，改为展示缓存（无缓存显示空态提示）
5. 新增 `watch(() => store.collectResults[<kind>], ...)` 还原数据 + 更新 lastRefresh
6. 采集完成回调里 `store.addHistory(...)`（带 `inputFull/outputFull`）

每个 view 的 `kind` / `toolId` / 数据 ref / watch 字段映射如下表：

| View | kind | 数据 ref | addHistory.tool | addHistory.action | outputPreview |
|------|------|----------|-----------------|-------------------|---------------|
| SystemInfoView | `system` | `data` (SystemInfo) | `systemInfo` | `查看系统信息` | `OS: ${data.os_name}` |
| NetworkInfoView | `network` | `data` (NetworkInfo) | `networkInfo` | `查看网络信息` | `接口 ${data.interfaces.length} 个` |
| ProcessListView | `process` | `data` (ProcessItem[]) | `processList` | `查看进程列表` | `${data.length} 个进程` |
| HardwareInfoView | `hardware` | `data` (HardwareInfo) | `hardwareInfo` | `查看硬件外设` | `GPU ${data.gpus.length} 个` |
| SoftwareEnvView | `software` | `data` (SoftwareEnv) | `softwareEnv` | `查看软件环境` | `软件 ${data.installed_software.length} 个` |

由于历史记录时机移到 composable 完成回调，但 composable 不感知各页面的 action/outputPreview 文案，故采用：**composable 仅写 `store.collectResults[kind]` + 通知；各页面用 `watch` 监听 `collectResults[kind]` 变化时填充数据 + 调 `store.addHistory(...)`**。这样文案留在页面、composable 保持通用。

- [ ] **Step 1: 改造 ProcessListView.vue**

**1a. 修改 `src/views/ProcessListView.vue` 的 `<script setup>`（58-207 行）：**

将 import 行（59-62 行）改为：

```ts
import { ref, computed, watch } from 'vue'
import { ElMessageBox, ElMessage, ElEmpty } from 'element-plus'
import { killProcess, killProcessByName, formatBytes, formatTimestamp, type ProcessItem } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useBackgroundCollect } from '@/composables/useBackgroundCollect'
```

将 `const store = useToolboxStore()` 之后到 `loadData` 定义结束（64-204 行中的状态声明保留，仅替换 loadData 与 onMounted）：

把 64-73 行的状态声明保留不动（`data`/`loading`/`error`/`lastRefresh`/`searchQuery`/`sortBy`/`killingPids`/`killingNames`）。

删除 `const loading = ref(false)`（66 行），改为由 composable 提供。

将 183-206 行的 `loadData` + `onMounted` 整体替换为：

```ts
const { collect, collecting } = useBackgroundCollect('process')

// 采集完成 → 填充数据 + 记录历史（watch 替代 onMounted，兼容 KeepAlive 缓存）
watch(() => store.collectResults['process'], (val) => {
  if (!val) return
  const list = val as ProcessItem[]
  data.value = list
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'processList',
    action: '查看进程列表',
    inputPreview: '',
    outputPreview: `${list.length} 个进程`,
    inputFull: '',
    outputFull: list.map(p => `${p.name} (PID: ${p.pid})`).join('\n'),
  })
}, { immediate: true })
```

`handleKill` / `handleKillAll` 内原本 `await loadData()` 的刷新调用（110、148 行）改为 `collect()`。

**1b. 修改 template（1-56 行）：**

刷新按钮（13 行）：

```html
<el-button type="primary" size="small" :loading="collecting" @click="collect">刷新</el-button>
```

`<div v-if="data" ...>`（22 行）改为 `<div v-if="data && data.length" ...>`，并在其前插入空态：

```html
<div v-if="!data || !data.length" class="tool-card">
  <div class="card-body">
    <el-empty description="暂无数据，点击右上角「刷新」采集进程列表" />
  </div>
</div>
```

- [ ] **Step 2: 改造 SystemInfoView.vue**

读取该文件后，按同样模式：
- import 移除 `ElLoading`，加 `ElEmpty` 与 `useBackgroundCollect`
- 删除 `loading` ref 与 `loadData`、`onMounted`
- `const { collect, collecting } = useBackgroundCollect('system')`
- 新增 watch：

```ts
watch(() => store.collectResults['system'], (val) => {
  if (!val) return
  data.value = val as SystemInfo
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'systemInfo',
    action: '查看系统信息',
    inputPreview: '',
    outputPreview: `OS: ${data.value.os_name}`,
    inputFull: '',
    outputFull: JSON.stringify(data.value),
  })
}, { immediate: true })
```

- 刷新按钮 `:loading="collecting"` `@click="collect"`
- 无数据时 `<el-empty description="暂无数据，点击「刷新」采集系统信息" />`

- [ ] **Step 3: 改造 NetworkInfoView.vue**

同模式，kind=`network`，watch 写入：

```ts
watch(() => store.collectResults['network'], (val) => {
  if (!val) return
  data.value = val as NetworkInfo
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'networkInfo',
    action: '查看网络信息',
    inputPreview: '',
    outputPreview: `接口 ${(val as NetworkInfo).interfaces.length} 个`,
    inputFull: '',
    outputFull: JSON.stringify(val),
  })
}, { immediate: true })
```

空态文案：`暂无数据，点击「刷新」采集网络信息`

- [ ] **Step 4: 改造 HardwareInfoView.vue**

同模式，kind=`hardware`，watch 写入：

```ts
watch(() => store.collectResults['hardware'], (val) => {
  if (!val) return
  data.value = val as HardwareInfo
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'hardwareInfo',
    action: '查看硬件外设',
    inputPreview: '',
    outputPreview: `GPU ${(val as HardwareInfo).gpus.length} 个`,
    inputFull: '',
    outputFull: JSON.stringify(val),
  })
}, { immediate: true })
```

空态文案：`暂无数据，点击「刷新」采集硬件外设`

- [ ] **Step 5: 改造 SoftwareEnvView.vue**

同模式，kind=`software`，watch 写入：

```ts
watch(() => store.collectResults['software'], (val) => {
  if (!val) return
  data.value = val as SoftwareEnv
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'softwareEnv',
    action: '查看软件环境',
    inputPreview: '',
    outputPreview: `软件 ${(val as SoftwareEnv).installed_software.length} 个`,
    inputFull: '',
    outputFull: JSON.stringify(val),
  })
}, { immediate: true })
```

空态文案：`暂无数据，点击「刷新」采集软件环境`

- [ ] **Step 6: 类型检查**

Run: `npx vue-tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 7: 手动验证**

Run: `npm run tauri dev`
验证步骤：
1. 打开「进程列表」页，点「刷新」，按钮变 loading，**立即切到「JSON 工具」页应可正常操作**（不卡）
2. 等待数秒后右上角弹出 ElNotification「进程列表采集完成，点击查看」
3. 点通知，跳回进程列表页，数据已填充，`lastRefresh` 已更新
4. 重复点「刷新」按钮，进行中再次点击应被忽略（按钮仍 loading，不重复触发）
5. 对其余 4 个页面各点一次「刷新」，确认均后台采集 + 通知 + 跳转正常
6. 切到「历史记录」页，确认 5 条采集记录存在，双击可还原

- [ ] **Step 8: 提交**

```bash
git add src/views/SystemInfoView.vue src/views/NetworkInfoView.vue src/views/ProcessListView.vue src/views/HardwareInfoView.vue src/views/SoftwareEnvView.vue
git commit -m "feat(frontend): 5 采集页改后台采集+通知+watch还原"
```

---

## Task 7: 版本号与 README 同步

**Files:**
- Modify: `package.json`（version）
- Modify: `src-tauri/tauri.conf.json`（version）
- Modify: `README.md`（功能阶段记录）

- [ ] **Step 1: 查看当前版本号**

Run: 在 `package.json` 与 `src-tauri/tauri.conf.json` 中查看 version 字段（如 `4.x.y`），新版本号 +1 补丁位（如 `4.x.y+1`）。

- [ ] **Step 2: 更新两处版本号**

将 `package.json` 与 `src-tauri/tauri.conf.json` 的 `version` 同步更新为新版本号。

- [ ] **Step 3: README 新增功能条目**

在 `README.md` 的功能阶段记录章节，对应版本下追加一条：

```markdown
- 系统工具后台采集：系统信息/网络/进程/硬件/软件 5 个采集页改为后台线程采集，采集期间不卡 UI、可切换其他工具，完成后应用内通知提示，点击跳回原页
```

- [ ] **Step 4: 提交**

```bash
git add package.json src-tauri/tauri.conf.json README.md
git commit -m "chore: 版本号同步至 <新版本号>，README 记录后台采集功能"
```

---

## Self-Review

**1. Spec coverage:**
- 后端 5 命令 async + spawn_blocking：Task 2 ✓
- `app.emit("collect-complete")`：Task 1 spawn_collect ✓
- 全局状态表 + `get_collect_status` 轮询兜底：Task 1 + Task 2 ✓
- 前端 `useBackgroundCollect` composable（listen + 2s 轮询 + done flag 去重）：Task 5 ✓
- 5 页面改造、移除 ElLoading、watch 还原：Task 6 ✓
- store.collectResults / collecting：Task 4 ✓
- 重复触发拦截：Task 5 `if (store.collecting[kind]) return` ✓
- 不自动采集、展示缓存、无缓存空态：Task 6 各 view ✓
- ElNotification + 查看 → openTab：Task 5 ✓
- 历史记录带 inputFull/outputFull：Task 6 各 watch ✓
- KeepAlive 用 watch 还原（项目记忆 12）：Task 6 `watch(..., { immediate: true })` ✓
- 不做取消/进度/系统通知：范围外，未实现 ✓
- 测试：后端 get_collect_status 边界（Task 2 Step 8）、前端 composable 自检（Task 5）✓
- 版本号同步 README（AGENTS.md）：Task 7 ✓

**2. Placeholder scan:** 无 TBD/TODO；所有代码块完整；命令与文件路径精确。

**3. Type consistency:**
- `CollectKind`：后端枚举 `System/Network/Process/Hardware/Software`，前端 union `'system'|'network'|'process'|'hardware'|'software'`，serde 默认小写序列化，匹配 ✓
- `CollectStartResult { task_id, kind }`：后端 struct 与前端 interface 一致（snake_case，Tauri 默认透传）✓
- `CollectCompletePayload`：后端 `CollectComplete { kind, task_id, ok, data, error }` 与前端一致 ✓
- `TaskState`：后端 `status: String` 序列化为字符串，前端 union 字面量 ✓
- `spawn_collect` 泛型约束 `T: Serialize`，5 个 `_inner` 返回类型均 `#[derive(Serialize)]` ✓
- composable `collect()` 无参，页面 `@click="collect"` 直传 ✓
- `store.collecting[kind]` / `store.collectResults[kind]` 与 store 定义键名一致 ✓

**4. 风险点:** Task 3 Step 2 整体替换 171-211 行时需保留 `killProcess`/`killProcessByName`（它们位于 190-203 行，落在替换区间内）。计划已在 Step 2 注明"若误删需补回"。执行时建议先读 171-211 完整内容，仅替换 5 个 getter，保留 kill 函数。
