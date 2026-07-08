# 系统工具后台采集设计文档（A13）

## 概述

将系统工具下 5 个采集类页面（系统信息、网络信息、进程列表、硬件信息、软件环境）由"同步 invoke + 全屏 ElLoading 遮罩"改造为"后台线程采集 + 事件通知 + 应用内浮层提示"模式。采集期间不阻塞 UI、不弹遮罩，用户可自由切换到其他工具；采集完成后通过 ElNotification 提示，点击可跳回原页面查看结果。

架构上镜像文件搜索器（file_searcher）已验证的范式：`tokio::task::spawn_blocking` 后台执行 + `AppHandle.emit()` 事件通知 + 前端 `listen` + 定时轮询兜底 + `done` flag 去重。前端抽 `useBackgroundCollect` composable 供 5 个页面复用，消除重复。

## 需求背景

- **场景**：用户在系统工具页点"刷新"采集系统信息/进程列表/硬件信息等，期间程序卡住未响应，无法切到其他工具
- **现状**：5 个采集命令（`get_system_info` / `get_network_info` / `get_process_list` / `get_hardware_info` / `get_software_env`）均为同步 `pub fn`，内部调用 PowerShell 子进程 + WMI 查询，单次耗时数秒到十几秒；前端 `await invoke` + `ElLoading.service` 全屏遮罩锁住整个工具区
- **根因**：全屏遮罩 + 同步等待导致工具区被锁，并非 UI 主线程真被阻塞（Tauri 同步命令实际跑在阻塞线程池），但用户体验等价于"未响应"
- **约束**：纯本地离线；不引入新依赖；复用既有 file_searcher 范式；保持采集逻辑本身不变

## 架构设计

### 整体方案：镜像文件搜索器范式

后端 `system_info.rs`：
- 5 个命令签名由 `pub fn` 改为 `pub async fn`，函数体用 `tokio::task::spawn_blocking` 包裹原同步采集逻辑（原逻辑作为内部同步函数保留，不改采集内容）
- 命令立即返回 `{ taskId, kind }`，`taskId = Uuid::new_v4()`，`kind` ∈ `system | network | process | hardware | software`
- 采集线程完成后通过 `app_handle.emit("collect-complete", { kind, taskId, ok, data?, error? })` 发事件
- 全局状态表 `Arc<Mutex<HashMap<CollectKind, TaskState>>>` 记录每个 kind 的最新任务状态（taskId / 状态 / 结果 / 时间戳），供轮询兜底查询

前端新增 `src/composables/useBackgroundCollect.ts`：
- 通用 composable，5 个页面复用
- 封装：启动采集（invoke）、监听 `collect-complete` 事件、2s 定时轮询 `get_collect_status(kind)` 兜底、`done` flag 去重
- 暴露：`collect(kind)`、`collecting`（按 kind 的进行中状态）、`result`（按 kind 的最新结果）

### 数据流

```
用户点「刷新」
 → invoke('collect_process') 立即返回 {taskId, kind:'process'}
 → 按钮变「采集中…」禁用，不弹遮罩，用户可切其他工具
 → 后端 spawn_blocking 跑原 get_process_list 同步逻辑
 → 完成 app.emit('collect-complete', {kind, taskId, ok, data})
 → 前端 listen 收到（或 2s 轮询兜底命中）→ done flag 去重
 → store.collectResults['process'] = data
 → ElNotification.success('进程列表采集完成') 带「查看」按钮
 → 用户点查看 → router 跳回该页
 → 页面 watch(() => store.collectResults['process']) 自动填充 + lastRefresh 更新
```

### 文件结构

```
src-tauri/src/
  ├── system_info.rs            # 修改：5 命令改 async + spawn_blocking；新增 CollectKind 枚举、
  │                              #       get_collect_status 命令、全局 Arc<Mutex<HashMap>> 状态表
  └── main.rs                   # 修改：generate_handler 注册 get_collect_status
src/
  ├── composables/
  │   └── useBackgroundCollect.ts   # 新增：通用采集 composable（约 80 行）
  ├── store/
  │   └── index.ts              # 修改：加 collectResults、collecting 状态
  ├── utils/
  │   └── systemInfoClient.ts   # 修改：5 个 getter 改返回 {taskId, kind}；新增事件类型、getCollectStatus
  └── views/
      ├── SystemInfoView.vue       # 修改：loadData 改用 composable；移除 ElLoading；watch 还原
      ├── NetworkInfoView.vue      # 同上
      ├── ProcessListView.vue      # 同上
      ├── HardwareInfoView.vue     # 同上
      └── SoftwareEnvView.vue      # 同上
```

## 详细设计

### 后端（system_info.rs）

**新增类型与状态**

```rust
#[derive(serde::Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectKind { System, Network, Process, Hardware, Software }

#[derive(serde::Serialize, Clone)]
pub struct TaskState {
    pub task_id: String,
    pub kind: CollectKind,
    pub status: String,        // "running" | "done" | "error"
    pub data: Option<serde_json::Value>,   // 完成时携带结果
    pub error: Option<String>,
    pub updated_at: u64,       // unix secs
}

// ponytail: 全局状态表，5 个 kind 各记最新一条任务。上限固定 5 条，无内存增长风险
static COLLECT_STATE: OnceLock<Mutex<HashMap<CollectKind, TaskState>>> = OnceLock::new();
```

**命令改造模板**（以 `get_process_list` 为例）

```rust
#[tauri::command]
pub async fn collect_process(app: AppHandle) -> Result<{ task_id: String, kind: CollectKind }, String> {
    let kind = CollectKind::Process;
    let task_id = uuid::Uuid::new_v4().to_string();
    upsert_state(kind, task_id.clone(), "running", None, None);
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        let res = get_process_list_inner();   // 原 pub fn 逻辑改名移入
        match res {
            Ok(data) => {
                let val = serde_json::to_value(&data).unwrap();
                upsert_state(kind, task_id.clone(), "done", Some(val.clone()), None);
                let _ = app2.emit("collect-complete", CollectComplete { kind, task_id, ok: true, data: Some(val), error: None });
            }
            Err(e) => {
                upsert_state(kind, task_id.clone(), "error", None, Some(e.clone()));
                let _ = app2.emit("collect-complete", CollectComplete { kind, task_id, ok: false, data: None, error: Some(e) });
            }
        }
    });
    Ok({ task_id, kind })
}

#[tauri::command]
pub fn get_collect_status(kind: CollectKind) -> Option<TaskState> {
    // 轮询兜底用，返回该 kind 最新任务状态
}
```

- 原 5 个 `pub fn get_xxx` 改名为 `get_xxx_inner` 作为内部同步函数，被 `spawn_blocking` 调用
- 5 个新命令命名：`collect_system` / `collect_network` / `collect_process` / `collect_hardware` / `collect_software`
- `kill_process` / `kill_process_by_name` / `is_admin` 不变
- 关键分支加 `debug_log!()`

### 前端

**`useBackgroundCollect.ts`（核心骨架）**

```ts
type CollectKind = 'system' | 'network' | 'process' | 'hardware' | 'software'

export function useBackgroundCollect(kind: CollectKind) {
  const store = useToolboxStore()
  const collecting = computed(() => store.collecting[kind])

  async function collect() {
    if (collecting.value) return                  // 重复触发拦截
    store.collecting[kind] = true
    let done = false                              // 事件+轮询去重 flag
    const unlisten = await listen('collect-complete', (e) => {
      if (e.payload.kind !== kind || done) return
      done = true; handleResult(e.payload)
    })
    const timer = setInterval(async () => {        // 2s 轮询兜底
      const st = await invoke('get_collect_status', { kind })
      if (st && (st.status === 'done' || st.status === 'error') && !done) {
        done = true; handleResult(st)
      }
    }, 2000)
    await invoke(`collect_${kind}`)
    // cleanup 在 handleResult 内：clearInterval、unlisten、collecting=false

    function handleResult(payload) {
      clearInterval(timer); unlisten()
      store.collecting[kind] = false
      if (payload.ok) {
        store.collectResults[kind] = payload.data
        ElNotification.success({ title: '采集完成', message: `${labelOf(kind)}采集完成`, duration: 5000, onClick: () => router.push(routeOf(kind)) })
        // store.addHistory 按原页面规则填写：tool/action/inputPreview/outputPreview/inputFull/outputFull
        // outputFull 用 JSON.stringify(payload.data)，inputFull 为空串
      } else {
        ElNotification.error({ title: '采集失败', message: payload.error })
      }
    }
  }
  return { collect, collecting, result: computed(() => store.collectResults[kind]) }
}
```

**store/index.ts 改动**

```ts
collectResults: { system: null, network: null, process: null, hardware: null, software: null } as Record<CollectKind, any>,
collecting: { system: false, network: false, process: false, hardware: false, software: false } as Record<CollectKind, boolean>,
```

**5 个 View 改造模板**（以 ProcessListView 为例）

- 移除 `import { ElLoading }`
- `const { collect, collecting, result } = useBackgroundCollect('process')`
- `loadData` 删除，按钮 `@click="collect"`，`:loading="collecting"`
- 移除 `onMounted(() => loadData())`，改为：有缓存展示缓存，无缓存显示空态 + 「点击采集」提示
- 新增 `watch(() => store.collectResults['process'], (val) => { if (val) { data.value = val; lastRefresh.value = formatTimestamp() } })`
- 搜索/排序等纯前端逻辑不变

## 关键决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 取消功能 | 不做 | YAGNI；采集只读，跑完即释放，取消增加状态复杂度 |
| 硬超时 | 不设 | WMI 在部分机器慢是常态，硬超时易误杀；前端 30s 无结果改提示「采集较慢」 |
| 重复触发 | 同 kind 进行中拦截 | 避免并发采集同一资源；`store.collecting[kind]` 守卫 |
| 进入页面自动采集 | 改为展示上次缓存 | 避免切页即触发后台任务；无缓存显示空态+引导按钮 |
| 通知形式 | 应用内 ElNotification | 用户已选；无需系统通知权限 |
| 历史记录时机 | 移到采集完成回调 | 仍带 `inputFull/outputFull`，保证历史页双击还原 |
| KeepAlive 还原 | `watch(store.collectResults[kind])` | 项目记忆第 12 条：KeepAlive 缓存页 `onMounted` 不触发，用 watch |
| 统一任务管理器 | 不抽 | YAGNI；当前仅采集一个场景，未来出现第二个再抽 |

## 性能分析

| 维度 | 开销 | 说明 |
|------|------|------|
| 后台线程 | <5MB 栈内存 | `spawn_blocking` 用 tokio 阻塞池（默认上限 512），5 任务同时跑占 5 线程 |
| CPU/IO | 与现状相同 | 采集逻辑不变，仅从卡 UI 改为卡后台线程 |
| 内存（结果缓存） | 进程列表约 1-3MB，其余几十 KB | 作为缓存保留在 store，符合「展示上次缓存」设计 |
| 通知 | 零开销 | ElNotification 是轻量 DOM 浮层 |
| 结论 | **性能负担可忽略** | 体验提升显著 |

## 错误处理

- 采集失败：emit `ok:false` + `error`，前端 ElNotification.error 提示
- 事件丢失：2s 轮询兜底命中 `done/error` 状态
- 事件早到（监听器注册前）：轮询兜底捕获；`done` flag 防止两者都到时重复处理
- 状态表并发写：`Mutex` 保护，5 个 kind 独立槽位无竞争

## 测试

- `useBackgroundCollect` 自检：`done` flag 去重逻辑（模拟事件+轮询都到达，断言只处理一次）、重复触发拦截逻辑——`console.assert` 自检，无框架
- 后端：`get_collect_status` 在无任务时返回 `None` 的边界

## 范围外

- 不改采集内容/数据结构（仅改执行模型）
- 不做取消、不做进度上报（采集是单次完成，非流式）
- 不改 `kill_process` / `is_admin` 等非采集命令
- 不引入系统通知（用户已选应用内浮层）
- 不抽统一后台任务管理器（YAGNI）
