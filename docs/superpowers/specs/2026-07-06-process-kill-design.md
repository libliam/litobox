# 进程 kill 与端口释放工具设计文档

## 概述

为栗的百宝箱（LitoBox）的系统工具补上"可操作"能力：在 ProcessListView 加"结束进程"按钮、在 NetworkInfoView 监听端口表加"释放端口"按钮。两者共享一个 Rust 后端命令 `kill_process(pid)`，通过 `taskkill /F` 强制结束进程。覆盖两个高频开发痛点：进程卡死需结束、端口被占用需释放。

## 需求背景

- **场景一**：开发中进程卡死（如 electron 子进程、node 服务），需要从进程列表直接结束，不必切到任务管理器
- **场景二**：端口被占用（"Address already in use"），需要快速找到占用进程并释放，不必记忆 `netstat -ano | findstr` 命令
- **现状**：ProcessListView 和 NetworkInfoView 都是只读，用户看到进程/端口后无法操作，必须切到任务管理器或命令行
- **约束**：纯本地离线；kill 前二次确认防误点；记录到历史可审计；不引入新依赖

## 架构设计

### 文件结构

```
src-tauri/src/
  └── system_info.rs            # 修改：新增 kill_process 命令 + parse_taskkill_output 纯函数
src-tauri/src/
  └── main.rs                   # 修改：invoke_handler 注册 kill_process
src/utils/
  └── systemInfoClient.ts       # 修改：新增 killProcess 封装 + KillResult 类型
src/views/
  ├── ProcessListView.vue       # 修改：表格加"操作"列 + handleKill
  └── NetworkInfoView.vue       # 修改：监听端口表加"释放"列 + handleReleasePort
```

**不新建模块**：`kill_process` 加到 `system_info.rs`，与现有 `get_process_list` / `get_network_info` 同属系统信息域，代码量小（~30 行），独立模块反而碎片化。

### 新增依赖

**无新增依赖**。复用 Cargo.toml 已有：
- `encoding_rs` —— GBK 解码 taskkill 输出
- `sysinfo = "0.31"` —— 预查进程名

### 后端命令

只新增 1 个命令：

| 命令 | 签名 | 作用 |
|---|---|---|
| `kill_process` | `(pid: u32) -> Result<KillResult, String>` | 强制结束指定 PID 进程，返回结构化结果 |

### 数据结构

```rust
#[derive(Serialize)]
pub struct KillResult {
    pub success: bool,           // 是否成功
    pub pid: u32,                // 目标 PID
    pub process_name: String,    // 进程名（成功时填充，预查失败则空）
    pub message: String,         // 友好消息，直接展示给用户
}
```

### 实现流程（system_info.rs::kill_process）

```
1. debug_log!("kill_process: pid={}", pid)
2. 用 sysinfo best-effort 预查进程名（查不到也继续，name 留空）：
   - sys.refresh_processes(); sys.process(Pid::from_u32(pid)) → name
3. Command::new("taskkill")
     .args(["/PID", &pid.to_string(), "/F"])
     .creation_flags(CREATE_NO_WINDOW)   // 0x08000000，避免弹黑框
     .output()
4. 用 encoding_rs::GBK.decode() 解码 stdout + stderr
5. 调 parse_taskkill_output(decoded) 纯函数解析（taskkill 是成功/失败的唯一真相源）：
   - exit_code == 0 → (true, "已结束 {name 或 PID} (PID: {pid})")
   - 含"拒绝访问" → (false, "拒绝访问，可能需要管理员权限")
   - 含"没有找到" / "找不到" → (false, "进程不存在或已退出")
   - 其他 → (false, 原始输出截断 200 字符)
6. debug_log!("kill_process result: {:?}", result)
7. Ok(result)
```

**关键**：sysinfo 预查仅用于获取进程名（成功消息需要），不作为提前返回依据。taskkill 是进程存在与否的唯一真相源——避免 sysinfo 快照过期导致误判。

### parse_taskkill_output 纯函数

抽取为独立函数便于单元测试：

```rust
fn parse_taskkill_output(
    exit_code: i32,
    stdout: &str,
    stderr: &str,
    pid: u32,
    process_name: &str,
) -> KillResult
```

## 数据流

以 ProcessListView 为例（NetworkInfoView 监听端口"释放"流程相同，仅文案与 tool/action 字段不同）：

```
用户点【结束】
  ↓
ElMessageBox.confirm(`确定结束进程 "${name}" (PID: ${pid})？\n强制结束可能导致未保存的数据丢失。`)
  ↓ 用户确认
const result = await killProcess(row.pid)
  ↓
store.addHistory({
  tool: 'processList',
  action: '结束进程',
  inputPreview: `${row.name} (PID: ${row.pid})`,
  outputPreview: result.message,
  inputFull: JSON.stringify({pid: row.pid, name: row.name}),
  outputFull: JSON.stringify(result),    // 符合 AGENTS.md 完整数据存储规范
})
  ↓
result.success
  ? ElMessage.success(result.message)
  : result.message.includes("管理员")
    ? ElMessage.error(result.message)      // 权限不足 → 红色
    : ElMessage.warning(result.message)    // 进程不存在等 → 黄色
  ↓
loadData()   // 自动刷新列表，让用户看到进程已消失
```

**NetworkInfoView 释放端口**：
- `tool: 'networkInfo'`，`action: '释放端口'`
- confirm 文案：`确定释放端口 ${local_addr}？\n将强制结束占用进程 "${process_name}" (PID: ${pid})。`

## 错误处理与边界情况

| 场景 | 后端行为 | 前端 Toast |
|------|----------|------------|
| 进程不存在（已退出） | taskkill 输出"没有找到" → `success=false, message="进程不存在或已退出"` | `ElMessage.warning` |
| 权限不足（系统进程） | taskkill 输出"拒绝访问" → `success=false, message="拒绝访问，可能需要管理员权限"` | `ElMessage.error` |
| PID 无效（0 / 超范围） | taskkill 输出"找不到" → `success=false` | `ElMessage.warning` |
| taskkill 命令本身失败 | `Command::output()` 返回 Err → `Err(String)` 抛给前端 | `ElMessage.error(String)` |
| taskkill 成功但进程名预查为空 | `process_name=""`，message 用 PID 兜底 `"已结束 PID: {pid}"` | `ElMessage.success` |
| 用户取消二次确认 | 不调后端 | 不提示 |

**Toast 类型选择规则**（前端）：
- `result.success === true` → `ElMessage.success`
- `result.success === false` 且 `result.message` 含"管理员" → `ElMessage.error`（权限问题，红色醒目）
- `result.success === false` 其他情况 → `ElMessage.warning`（进程不存在等，黄色提示）
- 后端抛 `Err(String)` → `ElMessage.error`（命令本身失败）

**安全护栏**（最小化，不过度）：
- 不做"系统进程白名单"——taskkill 本身会拒绝杀关键系统进程（csrss.exe 等会蓝屏的进程 taskkill 直接拒绝）
- 二次确认框已防止误点
- 不限制 PID 范围——用户责任，符合 lazy 原则

## UI 细节

### ProcessListView 表格新增列

```vue
<el-table-column label="操作" width="100" fixed="right">
  <template #default="{ row }">
    <el-button type="danger" size="small" link
      :loading="killingPids.has(row.pid)"
      @click="handleKill(row)">
      结束
    </el-button>
  </template>
</el-table-column>
```

### NetworkInfoView 监听端口表新增列

```vue
<el-table-column label="操作" width="100" fixed="right">
  <template #default="{ row }">
    <el-button type="danger" size="small" link
      :loading="killingPids.has(row.pid)"
      @click="handleReleasePort(row)">
      释放
    </el-button>
  </template>
</el-table-column>
```

### 按钮状态管理

用 `Set<number>` 记录正在 kill 的 PID，防止重复点击：

```typescript
const killingPids = ref(new Set<number>())

// handleKill 内：
killingPids.value.add(row.pid)
try {
  const result = await killProcess(row.pid)
  // ... Toast + 历史 + 刷新
} finally {
  killingPids.value.delete(row.pid)
}
```

## 设计决策

1. **taskkill 子进程而非 windows-sys API**：符合 AGENTS.md lazy 阶梯——系统命令已能解决则用它；零新依赖；项目已有成熟的子进程模式（CREATE_NO_WINDOW + encoding_rs::GBK.decode()，system_info.rs 的 PowerShell 调用就是这套）；taskkill 输出的中文错误信息可直接展示，无需翻译错误码。代价是 ~50ms 的子进程开销，单次操作可接受。

2. **强制结束（/F）而非优雅关闭**：用户痛点就是"进程卡死"，优雅关闭（WM_CLOSE）对卡死进程无效。强制结束覆盖 95% 场景。代价是进程没机会保存数据，但二次确认已警示。

3. **sysinfo 预查为 best-effort**：kill 后再查进程名就没了，必须先查。但预查不作为提前返回依据——sysinfo 快照可能过期，taskkill 才是进程存在与否的真相源。预查失败时 name 留空，taskkill 仍执行，成功消息用 PID 兜底。

4. **不新建 process_manager.rs 模块**：kill_process 与 get_process_list 同属系统信息域，代码量小（~30 行），独立模块反而碎片化。符合"最少文件"原则。

5. **KillResult 同时返回 success 和 message**：前端按 success 决定 Toast 类型（success/warning/error），message 直接展示——避免前端再翻译错误码，后端一次性把友好文案准备好。

6. **不集成工作流/变量池**：kill 是即时操作，无输入输出流转需求，YAGNI。

## 测试策略

按 AGENTS.md 规范——非平凡逻辑留一个最小自检，不上框架。

### 后端单元测试（system_info.rs `#[cfg(test)]` 模块）

不测 taskkill 真实调用（会真杀进程，危险），只测 `parse_taskkill_output` 纯函数：

```rust
#[test]
fn parse_taskkill_success() {
    let r = parse_taskkill_output(0, "成功: 已终止 PID 1234 的进程...", "", 1234, "notepad.exe");
    assert!(r.success);
    assert_eq!(r.process_name, "notepad.exe");
    assert!(r.message.contains("notepad.exe"));
}

#[test]
fn parse_taskkill_access_denied() {
    let r = parse_taskkill_output(1, "", "错误: 拒绝访问进程 PID 1234", 1234, "");
    assert!(!r.success);
    assert!(r.message.contains("管理员"));
}

#[test]
fn parse_taskkill_not_found() {
    let r = parse_taskkill_output(128, "", "错误: 没有找到进程 PID 9999", 9999, "");
    assert!(!r.success);
    assert!(r.message.contains("不存在"));
}

#[test]
fn parse_taskkill_unknown_error() {
    let r = parse_taskkill_output(1, "", "未知错误输出", 1234, "");
    assert!(!r.success);
    assert!(r.message.contains("未知错误"));
}
```

### 前端

无需测试，UI 行为简单（确认→调用→Toast→刷新）。

## 验收清单

实现完成后需手动验证以下场景：

1. **kill 普通进程**：开 notepad → ProcessListView 找到 notepad → 点【结束】→ 确认 → 成功 Toast → 列表刷新 → notepad 消失
2. **kill 系统进程失败**：选一个系统进程（如 winlogon.exe）→ 点【结束】→ 确认 → 失败 Toast 提示权限不足
3. **kill 已退出进程**：手动关闭某进程后立即对其 PID 点【结束】→ 提示"进程不存在或已退出"
4. **释放被占用端口**：`python -m http.server 8000` → NetworkInfoView 刷新 → 监听端口表找到 8000 → 点【释放】→ 确认 → 成功 Toast → 列表刷新 → 8000 消失
5. **取消二次确认**：点【结束】→ 取消 → 不调后端，无 Toast，按钮不 loading
6. **历史记录**：完成 kill 后去历史记录页 → 看到"结束进程"记录，inputFull/outputFull 完整
7. **防重复点击**：kill 进行中按钮 loading，不可再点
8. **端口释放联动**：释放端口后，对应 PID 在进程列表中也消失（刷新后验证）

## 不在本期范围

以下功能已识别但不在本 spec，留待后续：

- **G2 系统文件编辑**（环境变量编辑 + hosts 编辑器）—— 共享 admin + 文件写入基础设施
- **G3 全盘文件搜索**（Everything 简化版）—— 独立 spec
- **批量 kill**——YAGNI，单 kill 覆盖 95% 场景
- **UAC 提权重启**——复杂度高，当前用错误提示引导即可
- **进程详情查看**（句柄/模块/线程）——超出 kill 痛点范围
