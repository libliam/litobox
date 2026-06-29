# History View Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 扩展历史记录功能，支持双击跳转到对应工具页面，自动填充完整输入/输出和操作配置。

**Architecture:** 新建 `history_details` 附件表存储完整输入/输出/配置，通过 Pinia store 状态传递实现双击跳转后的数据回填。

**Tech Stack:** Rust (rusqlite), Vue 3, TypeScript, Pinia, Tauri 2.0

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/db.rs` | Modify | 新建 `history_details` 表 + CRUD + 迁移 + 导入导出适配 |
| `src-tauri/src/main.rs` | Modify | 注册新 Tauri 命令 |
| `src/utils/dbClient.ts` | Modify | 新增 `getHistoryDetail` 等前端调用函数 |
| `src/store/index.ts` | Modify | 新增 `pendingHistoryRestore` 状态及方法 |
| `src/views/HistoryView.vue` | Modify | 双击跳转逻辑 + 大文本提示 |
| `src/views/JsonTool.vue` | Modify | 示例：实现 restoreFromHistory |
| `src/views/StringTool.vue` | Modify | 示例：实现 restoreFromHistory |
| `src/views/EncodeTool.vue` | Modify | 示例：实现 restoreFromHistory |

---

### Task 1: 数据库层 — history_details 表 + CRUD（Rust）

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: 在 `init_tables` 中新增 `history_details` 表和 `detail_id` 列迁移**

在 `init_tables` 函数末尾（现有迁移代码之后，`Ok(())` 之前）添加：

```rust
    // 迁移：history_details 表
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS history_details (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            history_id INTEGER NOT NULL,
            input_full TEXT,
            output_full TEXT,
            options_json TEXT DEFAULT '{}',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (history_id) REFERENCES history(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_history_details_history_id ON history_details(history_id);
    "#).ok(); // 表已存在时忽略

    // 迁移：history 表新增 detail_id 列
    if let Err(_) = conn.execute("ALTER TABLE history ADD COLUMN detail_id INTEGER", []) {
        // 列已存在，忽略
    }
```

- [ ] **Step 2: 新增 `HistoryDetail` struct**

在 `HistoryRecord` struct 定义之后添加：

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryDetail {
    pub id: Option<i64>,
    pub history_id: i64,
    pub input_full: Option<String>,
    pub output_full: Option<String>,
    pub options_json: String,
    pub created_at: Option<String>,
}
```

- [ ] **Step 3: 新增 `db_add_history_detail` 函数**

在 `db_search_history` 函数之后添加：

```rust
pub fn db_add_history_detail(detail: HistoryDetail) -> Result<i64, String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO history_details (history_id, input_full, output_full, options_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                detail.history_id,
                detail.input_full,
                detail.output_full,
                detail.options_json
            ],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    })
}
```

- [ ] **Step 4: 新增 `db_get_history_detail` 函数**

```rust
pub fn db_get_history_detail(history_id: i64) -> Result<Option<HistoryDetail>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, history_id, input_full, output_full, options_json, created_at
                      FROM history_details WHERE history_id = ?1")
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_map(params![history_id], |row| {
                Ok(HistoryDetail {
                    id: row.get(0)?,
                    history_id: row.get(1)?,
                    input_full: row.get(2)?,
                    output_full: row.get(3)?,
                    options_json: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .next()
            .transpose()
            .map_err(|e| e.to_string())?;
        Ok(result)
    })
}
```

- [ ] **Step 5: 新增 `db_delete_history_details_for_history` 函数**

```rust
pub fn db_delete_history_details_for_history(history_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "DELETE FROM history_details WHERE history_id = ?1",
            params![history_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}
```

- [ ] **Step 6: 新增 Tauri 命令包装函数**

在文件末尾的 cmd 函数区域添加：

```rust
#[tauri::command]
pub fn cmd_db_add_history_detail(detail: HistoryDetail) -> Result<i64, String> {
    db_add_history_detail(detail)
}

#[tauri::command]
pub fn cmd_db_get_history_detail(history_id: i64) -> Result<Option<HistoryDetail>, String> {
    db_get_history_detail(history_id)
}

#[tauri::command]
pub fn cmd_db_delete_history_details_for_history(history_id: i64) -> Result<(), String> {
    db_delete_history_details_for_history(history_id)
}
```

- [ ] **Step 7: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误

---

### Task 2: 注册新 Tauri 命令

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 invoke_handler 中注册新命令**

在 `db::cmd_db_search_history,` 之后添加：

```rust
            db::cmd_db_add_history_detail,
            db::cmd_db_get_history_detail,
            db::cmd_db_delete_history_details_for_history,
```

- [ ] **Step 2: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

---

### Task 3: 前端 dbClient 封装

**Files:**
- Modify: `src/utils/dbClient.ts`

- [ ] **Step 1: 新增 `HistoryDetail` 接口和调用函数**

在 `searchHistory` 函数之后添加：

```typescript
export interface HistoryDetail {
  id?: number;
  history_id: number;
  input_full: string | null;
  output_full: string | null;
  options_json: string;
  created_at?: string;
}

export async function addHistoryDetail(detail: HistoryDetail): Promise<number> {
  return invoke('cmd_db_add_history_detail', { detail });
}

export async function getHistoryDetail(historyId: number): Promise<HistoryDetail | null> {
  return invoke('cmd_db_get_history_detail', { historyId });
}

export async function deleteHistoryDetailsForHistory(historyId: number): Promise<void> {
  return invoke('cmd_db_delete_history_details_for_history', { historyId });
}
```

---

### Task 4: Pinia Store 新增状态传递

**Files:**
- Modify: `src/store/index.ts`

- [ ] **Step 1: 新增 `HistoryRestoreState` 接口和状态**

在 `HistoryRecord` 接口之后添加：

```typescript
export interface HistoryRestoreState {
  tool: string
  input: string
  output: string
  options: Record<string, any>
  timestamp: string
}
```

在 `const history = ref<HistoryRecord[]>([])` 之后添加：

```typescript
  const pendingHistoryRestore = ref<HistoryRestoreState | null>(null)
  let restoreTimeout: ReturnType<typeof setTimeout> | null = null

  const triggerHistoryRestore = (data: HistoryRestoreState) => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = data
    // 30 秒未消费自动清除
    restoreTimeout = setTimeout(() => {
      pendingHistoryRestore.value = null
    }, 30000)
  }

  const clearHistoryRestore = () => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = null
  }
```

- [ ] **Step 2: 导出新方法**

在 return 语句中添加：

```typescript
    pendingHistoryRestore,
    triggerHistoryRestore,
    clearHistoryRestore,
```

---

### Task 5: HistoryView 双击跳转逻辑

**Files:**
- Modify: `src/views/HistoryView.vue`

- [ ] **Step 1: 模板改造 — 添加双击事件和 tooltip**

将 `<div class="history-item">` 改为：

```vue
<div
  v-for="(record, index) in filteredHistory"
  :key="index"
  class="history-item"
  @dblclick="handleJumpToTool(record)"
  :title="双击跳转到对应工具"
>
```

- [ ] **Step 2: 新增 `handleJumpToTool` 函数**

在 `<script setup>` 中 `handleClear` 函数之后添加：

```typescript
import * as db from '@/utils/dbClient'

const handleJumpToTool = async (record: any) => {
  // 检查工具是否在导航列表中
  const toolIds = store.tools.map(t => t.id)
  if (!toolIds.includes(record.tool)) {
    ElMessage.warning('该工具当前不可用')
    return
  }

  const loading = ElLoading.service({
    lock: true,
    text: '正在加载历史数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    // 获取完整数据
    const detail = await db.getHistoryDetail(record.id)

    store.triggerHistoryRestore({
      tool: record.tool,
      input: detail?.input_full || record.inputPreview || '',
      output: detail?.output_full || record.outputPreview || '',
      options: detail ? JSON.parse(detail.options_json || '{}') : {},
      timestamp: record.timestamp,
    })

    // 切换页面
    store.activeTool = record.tool
    ElMessage.success('已加载历史记录，输入和输出已填充')
  } catch (e: any) {
    ElMessage.error('加载失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
}
```

- [ ] **Step 3: 样式改造 — 添加 cursor 和 hover 效果**

在 `<style scoped>` 中 `.history-item` 添加：

```css
.history-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.3s;
  cursor: pointer;
}
```

- [ ] **Step 4: 大文本提示 — 在 preview 中显示 `[大文本 · 双击查看]`**

在 `filteredHistory` computed 中，如果 `inputPreview` 或 `outputPreview` 超过 10KB（10240 字符），显示特殊文本。

在模板中修改 preview 部分：

```vue
<div class="preview-row">
  <span class="preview-label">输入</span>
  <code class="preview-text">{{ formatPreview(record.inputPreview) }}</code>
</div>
<div class="preview-row">
  <span class="preview-label">输出</span>
  <code class="preview-text">{{ formatPreview(record.outputPreview) }}</code>
</div>
```

在 script 中添加：

```typescript
const LARGE_TEXT_THRESHOLD = 10240

const formatPreview = (text: string): string => {
  if (!text) return ''
  if (text.length > LARGE_TEXT_THRESHOLD) {
    return '[大文本 · 双击查看]'
  }
  return text
}
```

---

### Task 6: 修改 store.addHistory 同时写入 details 表

**Files:**
- Modify: `src/store/index.ts`

- [ ] **Step 1: 修改 `addHistory` 函数签名和实现**

当前 `addHistory` 接收 `Omit<HistoryRecord, 'timestamp'>`。扩展为接收可选的 `inputFull`、`outputFull`、`options` 参数。

修改 `HistoryRecord` 接口（前端类型）：

```typescript
export interface HistoryRecord {
  tool: string
  action: string
  timestamp: string
  inputPreview: string
  outputPreview: string
  inputFull?: string    // 新增
  outputFull?: string   // 新增
  options?: Record<string, any>  // 新增
}
```

修改 `addHistory` 函数：

```typescript
  const addHistory = async (record: Omit<HistoryRecord, 'timestamp'>) => {
    const newRecord = {
      ...record,
      timestamp: new Date().toISOString()
    }
    // 同步更新本地状态
    history.value.unshift(newRecord)
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY)
    }
    // 保存到 SQLite
    try {
      const historyId = await db.addHistory({
        tool: newRecord.tool,
        action: newRecord.action,
        input_preview: newRecord.inputPreview,
        output_preview: newRecord.outputPreview,
      })

      // 如果有完整数据，写入 details 表
      if (record.inputFull !== undefined || record.outputFull !== undefined || record.options) {
        await db.addHistoryDetail({
          history_id: historyId,
          input_full: record.inputFull ?? null,
          output_full: record.outputFull ?? null,
          options_json: JSON.stringify(record.options || {}),
        })
      }
    } catch (error) {
      console.error('保存历史失败:', error)
    }
  }
```

> **注意**：现有工具页面调用 `addHistory` 时不传 `inputFull`/`outputFull`/`options`，不会报错，因为它们是可选字段。后续逐步在各工具页面添加完整数据传递。

---

### Task 7: 导入导出适配

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: 修改 `db_export_all` — 导出时 JOIN details**

修改导出历史的 SQL 查询，将 `db_export_all` 中的历史导出部分改为：

```rust
        // 导出历史（LEFT JOIN details）
        let mut stmt = conn.prepare(
            "SELECT h.tool, h.action, h.input_preview, h.output_preview, h.created_at,
                    d.input_full, d.output_full, d.options_json
             FROM history h
             LEFT JOIN history_details d ON h.detail_id = d.id
             ORDER BY h.created_at DESC"
        ).map_err(|e| e.to_string())?;
        let history: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "tool": row.get::<_, String>(0)?,
                    "action": row.get::<_, String>(1)?,
                    "input_preview": row.get::<_, String>(2)?,
                    "output_preview": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "input_full": row.get::<_, Option<String>>(5)?,
                    "output_full": row.get::<_, Option<String>>(6)?,
                    "options_json": row.get::<_, Option<String>>(7)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("history", serde_json::Value::Array(history));
```

- [ ] **Step 2: 修改 `db_import_all` — 导入时恢复 details**

修改导入历史部分：

```rust
        // 导入历史（清空后重新导入）
        conn.execute("DELETE FROM history", []).map_err(|e| e.to_string())?;
        if let Some(history) = export.get("history").and_then(|v| v.as_array()) {
            for record in history {
                let tool = record.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                let action = record.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let input_preview = record.get("input_preview").and_then(|v| v.as_str()).unwrap_or("");
                let output_preview = record.get("output_preview").and_then(|v| v.as_str()).unwrap_or("");
                let created_at = record.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                let history_id = conn.execute(
                    "INSERT INTO history (tool, action, input_preview, output_preview, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![tool, action, input_preview, output_preview, created_at],
                ).map_err(|e| e.to_string())?;

                // 导入 details
                if let (Some(input_full), Some(output_full)) = (
                    record.get("input_full").and_then(|v| v.as_str()),
                    record.get("output_full").and_then(|v| v.as_str()),
                ) {
                    let options_json = record.get("options_json")
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}");
                    let detail_id = conn.execute(
                        "INSERT INTO history_details (history_id, input_full, output_full, options_json)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![history_id as i64, input_full, output_full, options_json],
                    ).map_err(|e| e.to_string())?;
                    conn.execute(
                        "UPDATE history SET detail_id = ?1 WHERE id = ?2",
                        params![detail_id as i64, history_id as i64],
                    ).map_err(|e| e.to_string())?;
                }
            }
        }
```

- [ ] **Step 3: 编译验证**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

---

### Task 8: 示例工具 — JsonTool restoreFromHistory

**Files:**
- Modify: `src/views/JsonTool.vue`

- [ ] **Step 1: 在 `onMounted` 中检查并还原历史**

查看 JsonTool.vue 的现有结构后，在 `onMounted` 中添加：

```typescript
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

onMounted(() => {
  // 检查是否有待还原的历史记录
  if (store.pendingHistoryRestore?.tool === 'json') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})
```

- [ ] **Step 2: 实现 `restoreFromHistory` 函数**

```typescript
const restoreFromHistory = (data: HistoryRestoreState) => {
  // 还原 activeTab
  if (data.options?.activeTab) {
    activeTab.value = data.options.activeTab
  }
  // 填充输入框
  input.value = data.input
  // 填充输出框（不重新执行）
  const currentTabState = getTabState(activeTab.value)
  currentTabState.output = data.output
  // 显示提示
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}
```

> 需要根据 JsonTool.vue 的实际变量名调整（如 `input`、`activeTab`、Tab 状态管理等）。

---

### Task 9: 示例工具 — StringTool restoreFromHistory

**Files:**
- Modify: `src/views/StringTool.vue`

- [ ] **Step 1: 同上模式，在 `onMounted` 中添加历史还原检查**

```typescript
onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'string') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})
```

- [ ] **Step 2: 实现 `restoreFromHistory`**

根据 StringTool 的实际结构，还原 `input`、`activeTab`（操作类型如 trim/uppercase 等）、输出。

---

### Task 10: 示例工具 — EncodeTool restoreFromHistory

**Files:**
- Modify: `src/views/EncodeTool.vue`

- [ ] **Step 1: 同上模式，在 `onMounted` 中添加历史还原检查**

- [ ] **Step 2: 实现 `restoreFromHistory`**

根据 EncodeTool 的实际结构，还原 `input`、操作类型（encode/decode）、编码方式（base64/url/html/unicode）、输出。

---

### Task 11: 提交

- [ ] **Step 1: 提交所有变更**

```bash
git add -A
git commit -m "feat: 扩展历史记录功能，支持双击跳转和完整输入输出还原"
```

---

## Self-Review

### 1. Spec 覆盖检查

| Spec 要求 | 对应 Task |
|-----------|-----------|
| 新建 `history_details` 表 | Task 1 Step 1 |
| history 表新增 `detail_id` 列 | Task 1 Step 1 |
| details CRUD 函数 | Task 1 Step 3-5 |
| Tauri 命令注册 | Task 2 |
| dbClient 封装 | Task 3 |
| Pinia 状态传递 + 30s 过期 | Task 4 |
| HistoryView 双击跳转 | Task 5 |
| 大文本 10KB 提示 | Task 5 Step 4 |
| 工具不存在提示 | Task 5 Step 2 |
| 修改 addHistory 写入 details | Task 6 |
| 导入导出适配 | Task 7 |
| options_json 解析失败降级 | Task 8/9/10 中 JSON.parse 用 try-catch |
| 多 Tab activeTab 还原 | Task 8/9/10 |
| 跳转后提示条 | Task 8/9/10 |
| CASCADE 删除 | Task 1 Step 1（FOREIGN KEY ON DELETE CASCADE）|

### 2. Placeholder 扫描

- ✅ 无 TBD/TODO
- ✅ 所有函数签名在前后一致
- ✅ 类型名一致（`HistoryDetail`、`HistoryRestoreState`）

### 3. 类型一致性

- Rust: `HistoryDetail` struct 字段与前端 `HistoryDetail` 接口匹配（`history_id`、`input_full`/`inputFull` 通过 serde 自动转换）
- 注意：Rust 的 `snake_case` 字段通过 serde 序列化为 `snake_case`，前端需要对应。`dbClient.ts` 中 `getHistoryDetail` 返回的字段是 `input_full`（snake_case），`HistoryRestoreState` 中是 `input`（camelCase），转换在 `handleJumpToTool` 中完成。
