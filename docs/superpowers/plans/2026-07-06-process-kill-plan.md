# 进程 kill 与端口释放工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 ProcessListView 和 NetworkInfoView 监听端口表加可操作按钮，共享 Rust 后端命令 `kill_process(pid)`，通过 taskkill /F 强制结束进程。

**Architecture:** 后端在 system_info.rs 新增 `kill_process` 命令 + `parse_taskkill_output` 纯函数（taskkill 子进程 + GBK 解码），前端在 systemInfoClient.ts 加 invoke 封装，两个 Vue 页面各加一列操作按钮。零新依赖，复用项目现有子进程模式。

**Tech Stack:** Rust (Tauri 2.0, sysinfo, encoding_rs, std::process::Command) + Vue 3 (Composition API, Element Plus, TypeScript)

**Spec:** `docs/superpowers/specs/2026-07-06-process-kill-design.md`

---

## 文件结构

| 文件 | 责任 | 改动类型 |
|------|------|----------|
| `src-tauri/src/system_info.rs` | 新增 KillResult 结构 + parse_taskkill_output 纯函数 + kill_process 命令 + 单元测试 | 修改 |
| `src-tauri/src/main.rs` | invoke_handler 注册 kill_process | 修改 |
| `src/utils/systemInfoClient.ts` | 新增 killProcess 封装 + KillResult 类型 | 修改 |
| `src/views/ProcessListView.vue` | 表格加"操作"列 + handleKill | 修改 |
| `src/views/NetworkInfoView.vue` | 监听端口表加"释放"列 + handleReleasePort | 修改 |
| `src-tauri/Cargo.toml` | 版本号 4.3.0 → 4.4.0 | 修改 |
| `package.json` | 版本号 4.3.0 → 4.4.0 | 修改 |
| `README.md` | 功能阶段记录加 V4.4 条目 | 修改 |

---

## Task 1: KillResult 结构 + parse_taskkill_output 纯函数（TDD）

**Files:**
- Modify: `src-tauri/src/system_info.rs` (在 ProcessItem 结构后 ~line 105 加 KillResult；在 run_powershell_json 后 ~line 320 加 parse_taskkill_output；在文件末尾 ~line 974 加 test 模块)

- [ ] **Step 1: 在 system_info.rs 的 ProcessItem 结构后添加 KillResult 结构**

在 `src-tauri/src/system_info.rs` 找到 ProcessItem 结构定义（约 line 95-104）：

```rust
#[derive(Serialize)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub status: String,
    pub command: String,
}
```

在其后添加（注意 `Debug` derive 用于 debug_log 的 `{:?}` 格式化）：

```rust
#[derive(Serialize, Debug)]
pub struct KillResult {
    pub success: bool,
    pub pid: u32,
    pub process_name: String,
    pub message: String,
}
```

- [ ] **Step 2: 在文件末尾添加 test 模块（含 4 个失败测试）**

在 `src-tauri/src/system_info.rs` 文件最末尾（line 973 之后）追加：

```rust
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
```

- [ ] **Step 3: 运行测试验证失败（函数未定义）**

Run:
```bash
cd src-tauri && cargo test parse_taskkill -- --nocapture
```
Expected: 编译失败，错误信息含 `cannot find function 'parse_taskkill_output' in this scope`

- [ ] **Step 4: 实现 parse_taskkill_output 纯函数**

在 `src-tauri/src/system_info.rs` 的 `run_powershell_json` 函数之后（约 line 320，`is_admin` 函数之前）添加：

```rust
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
```

- [ ] **Step 5: 运行测试验证通过**

Run:
```bash
cd src-tauri && cargo test parse_taskkill -- --nocapture
```
Expected: 5 个测试全部 PASS（parse_taskkill_success, parse_taskkill_success_without_name, parse_taskkill_access_denied, parse_taskkill_not_found, parse_taskkill_unknown_error）

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat(process-kill): 添加 KillResult 结构与 parse_taskkill_output 纯函数及单元测试"
```

---

## Task 2: kill_process Tauri 命令

**Files:**
- Modify: `src-tauri/src/system_info.rs` (在 get_process_list 函数后 ~line 629 添加 kill_process)

- [ ] **Step 1: 在 get_process_list 函数后添加 kill_process 命令**

在 `src-tauri/src/system_info.rs` 找到 `get_process_list` 函数结束位置（约 line 629，`Ok(processes)` 之后），添加：

```rust
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<KillResult, String> {
    debug_log!("kill_process: pid={}", pid);

    // 1. best-effort 预查进程名（查不到也继续，taskkill 是真相源）
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let process_name = sys
        .process(sysinfo::Pid::from_u32(pid))
        .map(|p| p.name().to_string_lossy().to_string())
        .unwrap_or_default();
    debug_log!("kill_process: 预查进程名 = {:?}", process_name);

    // 2. 调用 taskkill /PID <pid> /F 强制结束
    let mut cmd = Command::new("taskkill");
    cmd.args(["/PID", &pid.to_string(), "/F"]);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .map_err(|e| format!("taskkill 执行失败: {}", e))?;

    // 3. GBK 解码输出（中文 Windows taskkill 输出为 GBK 编码）
    let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
    let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);
    debug_log!(
        "kill_process: exit_code={}, stdout={}, stderr={}",
        exit_code,
        stdout,
        stderr
    );

    // 4. 解析输出构造结果
    let result = parse_taskkill_output(
        exit_code,
        &stdout,
        &stderr,
        pid,
        &process_name,
    );
    debug_log!("kill_process result: {:?}", result);

    Ok(result)
}
```

- [ ] **Step 2: 运行 cargo check 验证编译**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过，无错误（可能有 warning，忽略）

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/system_info.rs
git commit -m "feat(process-kill): 实现 kill_process Tauri 命令（taskkill + GBK 解码）"
```

---

## Task 3: 注册 kill_process 到 main.rs

**Files:**
- Modify: `src-tauri/src/main.rs` (在 system_info::get_software_env 后添加，约 line 100)

- [ ] **Step 1: 在 invoke_handler 中注册 kill_process**

在 `src-tauri/src/main.rs` 找到（约 line 95-100）：

```rust
            system_info::is_admin,
            system_info::get_system_info,
            system_info::get_network_info,
            system_info::get_process_list,
            system_info::get_hardware_info,
            system_info::get_software_env,
```

在 `system_info::get_software_env,` 之后添加一行：

```rust
            system_info::get_software_env,
            system_info::kill_process,
```

- [ ] **Step 2: 运行 cargo check 验证**

Run:
```bash
cd src-tauri && cargo check
```
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(process-kill): 注册 kill_process 命令到 invoke_handler"
```

---

## Task 4: 前端 killProcess 封装 + KillResult 类型

**Files:**
- Modify: `src/utils/systemInfoClient.ts` (在 getProcessList 后添加，约 line 181)

- [ ] **Step 1: 添加 KillResult 接口和 killProcess 封装**

在 `src/utils/systemInfoClient.ts` 找到 `getProcessList` 函数（约 line 179-181）：

```typescript
export function getProcessList(): Promise<ProcessItem[]> {
  return invoke<ProcessItem[]>('get_process_list')
}
```

在其后添加：

```typescript
export interface KillResult {
  success: boolean
  pid: number
  process_name: string
  message: string
}

export function killProcess(pid: number): Promise<KillResult> {
  return invoke<KillResult>('kill_process', { pid })
}
```

- [ ] **Step 2: 运行 vue-tsc 验证类型**

Run:
```bash
npx vue-tsc --noEmit
```
Expected: 无新增类型错误

- [ ] **Step 3: 提交**

```bash
git add src/utils/systemInfoClient.ts
git commit -m "feat(process-kill): 添加 killProcess 前端封装与 KillResult 类型"
```

---

## Task 5: ProcessListView 加"结束"按钮

**Files:**
- Modify: `src/views/ProcessListView.vue`

- [ ] **Step 1: 更新 import，添加 ElMessageBox、ElMessage、killProcess**

在 `src/views/ProcessListView.vue` 找到 import 区（约 line 44-47）：

```typescript
import { ref, computed, onMounted, watch } from 'vue'
import { ElLoading } from 'element-plus'
import { getProcessList, formatBytes, formatTimestamp, type ProcessItem } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
```

改为：

```typescript
import { ref, computed, onMounted, watch } from 'vue'
import { ElLoading, ElMessageBox, ElMessage } from 'element-plus'
import { getProcessList, killProcess, formatBytes, formatTimestamp, type ProcessItem } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
```

- [ ] **Step 2: 在表格添加"操作"列**

在 `src/views/ProcessListView.vue` 找到进程表格（约 line 27-37），在 `状态` 列之后、`</el-table>` 之前添加操作列：

```vue
          <el-table-column prop="status" label="状态" width="80" />
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

- [ ] **Step 3: 添加 killingPids 状态和 handleKill 函数**

在 `src/views/ProcessListView.vue` 的 `<script setup>` 中，找到 `sortBy` 定义之后（约 line 55）：

```typescript
const sortBy = ref('cpu')
```

在其后添加：

```typescript
const killingPids = ref(new Set<number>())

const handleKill = async (row: ProcessItem) => {
  try {
    await ElMessageBox.confirm(
      `确定结束进程 "${row.name}" (PID: ${row.pid})？\n强制结束可能导致未保存的数据丢失。`,
      '结束进程确认',
      { type: 'warning', confirmButtonText: '结束', cancelButtonText: '取消' }
    )
  } catch {
    return  // 用户取消
  }

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    store.addHistory({
      tool: 'processList',
      action: '结束进程',
      inputPreview: `${row.name} (PID: ${row.pid})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ pid: row.pid, name: row.name }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }
    await loadData()  // 刷新列表
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}
```

- [ ] **Step 4: 运行 vue-tsc 验证类型**

Run:
```bash
npx vue-tsc --noEmit
```
Expected: 无新增类型错误

- [ ] **Step 5: 提交**

```bash
git add src/views/ProcessListView.vue
git commit -m "feat(process-kill): ProcessListView 添加结束进程按钮"
```

---

## Task 6: NetworkInfoView 监听端口表加"释放"按钮

**Files:**
- Modify: `src/views/NetworkInfoView.vue`

- [ ] **Step 1: 更新 import，添加 ElMessageBox、ElMessage、killProcess**

在 `src/views/NetworkInfoView.vue` 找到 import 区（约 line 73-76）：

```typescript
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getNetworkInfo, formatTimestamp, type NetworkInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
```

改为：

```typescript
import { ref, onMounted } from 'vue'
import { ElLoading, ElMessageBox, ElMessage } from 'element-plus'
import { getNetworkInfo, killProcess, formatTimestamp, type NetworkInfo, type ListeningPort } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
```

- [ ] **Step 2: 在监听端口表添加"操作"列**

在 `src/views/NetworkInfoView.vue` 找到监听端口表（约 line 60-66）：

```vue
          <el-table-column prop="protocol" label="协议" width="60" />
          <el-table-column prop="local_addr" label="地址" min-width="160" />
          <el-table-column prop="pid" label="PID" width="70" />
          <el-table-column prop="process_name" label="进程" min-width="120" />
```

在 `process_name` 列之后、`</el-table>` 之前添加：

```vue
          <el-table-column prop="process_name" label="进程" min-width="120" />
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

- [ ] **Step 3: 添加 killingPids 状态和 handleReleasePort 函数**

在 `src/views/NetworkInfoView.vue` 的 `<script setup>` 中，找到 `lastRefresh` 定义之后（约 line 82）：

```typescript
const lastRefresh = ref('')
```

在其后添加：

```typescript
const killingPids = ref(new Set<number>())

const handleReleasePort = async (row: ListeningPort) => {
  try {
    await ElMessageBox.confirm(
      `确定释放端口 ${row.local_addr}？\n将强制结束占用进程 "${row.process_name}" (PID: ${row.pid})。`,
      '释放端口确认',
      { type: 'warning', confirmButtonText: '释放', cancelButtonText: '取消' }
    )
  } catch {
    return  // 用户取消
  }

  killingPids.value.add(row.pid)
  try {
    const result = await killProcess(row.pid)
    store.addHistory({
      tool: 'networkInfo',
      action: '释放端口',
      inputPreview: `${row.local_addr} (${row.process_name} PID: ${row.pid})`,
      outputPreview: result.message,
      inputFull: JSON.stringify({ local_addr: row.local_addr, pid: row.pid, process_name: row.process_name }),
      outputFull: JSON.stringify(result),
    })
    if (result.success) {
      ElMessage.success(result.message)
    } else if (result.message.includes('管理员')) {
      ElMessage.error(result.message)
    } else {
      ElMessage.warning(result.message)
    }
    await loadData()  // 刷新列表
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    killingPids.value.delete(row.pid)
  }
}
```

- [ ] **Step 4: 运行 vue-tsc 验证类型**

Run:
```bash
npx vue-tsc --noEmit
```
Expected: 无新增类型错误

- [ ] **Step 5: 提交**

```bash
git add src/views/NetworkInfoView.vue
git commit -m "feat(process-kill): NetworkInfoView 监听端口表添加释放端口按钮"
```

---

## Task 7: 版本号更新 + README 同步

**Files:**
- Modify: `src-tauri/Cargo.toml` (version 4.3.0 → 4.4.0)
- Modify: `package.json` (version 4.3.0 → 4.4.0)
- Modify: `README.md` (版本路线表加 V4.4 + 系统工具功能表更新)

- [ ] **Step 1: 更新 Cargo.toml 版本号**

在 `src-tauri/Cargo.toml` 找到（line 3）：

```toml
version = "4.3.0"
```

改为：

```toml
version = "4.4.0"
```

- [ ] **Step 2: 更新 package.json 版本号**

在 `package.json` 找到 `"version": "4.3.0"` 改为：

```json
"version": "4.4.0",
```

- [ ] **Step 3: README.md 版本路线表添加 V4.4**

在 `README.md` 找到版本路线表（约 line 303）：

```markdown
| V4.3 | ✅ | 磁盘空间分析器（文件夹大小/大文件Top N/按类型统计/重复文件检测，可入回收站删除） |
```

在其后添加一行：

```markdown
| V4.4 | ✅ | 进程 kill 与端口释放（ProcessListView 加结束按钮、NetworkInfoView 监听端口加释放按钮，共享 kill_process 后端命令） |
```

- [ ] **Step 4: README.md 系统工具表更新**

在 `README.md` 找到系统工具表（约 line 88-95），找到"进程列表"行：

```markdown
| **进程列表** | 运行中进程查看，CPU/内存占用排序，搜索过滤 |
```

改为：

```markdown
| **进程列表** | 运行中进程查看，CPU/内存占用排序，搜索过滤，支持结束进程 |
```

找到"网络信息"行：

```markdown
| **网络信息** | 网络接口/IP/MAC、活动 TCP 连接、监听端口、WiFi |
```

改为：

```markdown
| **网络信息** | 网络接口/IP/MAC、活动 TCP 连接、监听端口、WiFi，监听端口支持释放（kill 占用进程） |
```

- [ ] **Step 5: 提交**

```bash
git add src-tauri/Cargo.toml package.json README.md
git commit -m "chore: 版本号 4.3.0 → 4.4.0，README 同步更新进程 kill 与端口释放功能"
```

---

## Task 8: 启动开发服务器 + 手动验收

**Files:** 无（纯验证）

- [ ] **Step 1: 启动 Tauri 开发服务器**

Run:
```bash
npm run tauri dev
```
Expected: 应用启动，无编译错误

- [ ] **Step 2: 验收清单 - kill 普通进程**

1. 打开记事本（notepad）
2. 在应用中进入"进程列表"页面
3. 搜索 notepad，找到对应行
4. 点击【结束】按钮
5. 二次确认框出现，点击【结束】
6. 验证：成功 Toast 显示"已结束 notepad.exe (PID: xxx)"
7. 验证：列表自动刷新，notepad 行消失
8. 验证：记事本窗口已关闭

- [ ] **Step 3: 验收清单 - kill 系统进程失败**

1. 在进程列表找到一个系统进程（如 winlogon.exe 或 explorer.exe）
2. 点击【结束】
3. 二次确认后验证：红色 error Toast 显示"拒绝访问，可能需要管理员权限"
4. 验证：列表刷新后该进程仍在

- [ ] **Step 4: 验收清单 - kill 已退出进程**

1. 记下某个进程的 PID（如 notepad PID 1234）
2. 手动关闭该进程
3. 立即在进程列表对该 PID 点【结束】（如果列表未刷新，进程还在显示）
4. 验证：黄色 warning Toast 显示"进程不存在或已退出"

- [ ] **Step 5: 验收清单 - 释放被占用端口**

1. 在终端运行 `python -m http.server 8000`（或其他方式占用 8000 端口）
2. 在应用中进入"网络信息"页面，点【刷新】
3. 在"监听端口"表找到 8000 端口行
4. 点击【释放】按钮
5. 二次确认框出现，点击【释放】
6. 验证：成功 Toast 显示"已结束 python.exe (PID: xxx)"
7. 验证：列表自动刷新，8000 端口行消失
8. 验证：终端中 python http.server 进程已终止

- [ ] **Step 6: 验收清单 - 取消二次确认**

1. 在进程列表对任意进程点【结束】
2. 二次确认框出现，点击【取消】
3. 验证：无 Toast 提示
4. 验证：按钮不处于 loading 状态
5. 验证：进程未被结束

- [ ] **Step 7: 验收清单 - 历史记录**

1. 完成一次 kill 操作后
2. 进入"历史记录"页面
3. 验证：看到"结束进程"记录
4. 双击该记录，验证 inputFull/outputFull 完整还原

- [ ] **Step 8: 验收清单 - 防重复点击**

1. 对一个大进程点【结束】
2. 在 kill 进行中（loading 状态）再次点击该行的【结束】
3. 验证：按钮处于 loading 不可重复点击

- [ ] **Step 9: 验收清单 - 端口释放联动**

1. 释放一个端口后
2. 切换到"进程列表"页面，点【刷新】
3. 验证：对应 PID 的进程已从进程列表消失

- [ ] **Step 10: 提交验收通过标记**

如果所有验收通过，无需额外提交。如果有修复，提交修复：

```bash
git add -A
git commit -m "fix(process-kill): 验收修复"
```

---

## Self-Review 检查

### Spec 覆盖检查

| Spec 要求 | 对应 Task |
|-----------|-----------|
| kill_process 命令 (pid) -> Result<KillResult, String> | Task 2 |
| KillResult 结构 (success/pid/process_name/message) | Task 1 Step 1 |
| parse_taskkill_output 纯函数 | Task 1 Step 4 |
| taskkill /PID /F + CREATE_NO_WINDOW + GBK 解码 | Task 2 Step 1 |
| sysinfo best-effort 预查进程名 | Task 2 Step 1 |
| debug_log! 关键分支日志 | Task 2 Step 1 |
| 注册到 main.rs invoke_handler | Task 3 |
| 前端 killProcess 封装 + KillResult 类型 | Task 4 |
| ProcessListView 加操作列 + handleKill | Task 5 |
| NetworkInfoView 监听端口表加释放列 + handleReleasePort | Task 6 |
| 二次确认（ElMessageBox.confirm） | Task 5/6 Step 3 |
| Toast 类型选择规则（success/warning/error） | Task 5/6 Step 3 |
| 记录到历史（store.addHistory + inputFull/outputFull） | Task 5/6 Step 3 |
| kill 后自动刷新（loadData） | Task 5/6 Step 3 |
| 防重复点击（killingPids Set + loading） | Task 5/6 Step 2/3 |
| 单元测试（parse_taskkill_output 4+用例） | Task 1 Step 2 |
| 版本号同步 + README 更新 | Task 7 |
| 手动验收清单 8 项 | Task 8 |

### 类型一致性检查

- `KillResult` 字段名：后端 `process_name`（snake_case）→ 前端 `process_name`（Tauri 默认 snake_case 传递，无需 rename）✓
- `killProcess(pid: number)` 前端参数名 `pid` → 后端 `kill_process(pid: u32)` ✓
- `ListeningPort` 类型已在 systemInfoClient.ts 定义（line 64-69），Task 6 import 正确 ✓
- `ProcessItem` 类型已在 systemInfoClient.ts 定义（line 71-78），Task 5 已 import ✓

### 无占位符检查

- 所有代码步骤含完整代码，无 "TODO"/"TBD"/"类似 Task N" ✓
- 所有文件路径为绝对相对路径 ✓
- 所有命令含 expected 输出 ✓
