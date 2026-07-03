# SQLite 查看器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 新增一个只读 SQLite 查看器工具，用于开发调试时浏览本地 .db 文件的表结构和数据。

**Architecture:** 新增 Rust 后端模块 `sqlite_viewer.rs`（复用 rusqlite 依赖，只读模式打开），提供 5 个 Tauri 命令；新增 Vue 前端页面 `SqliteViewerView.vue`（三栏布局：表列表 + SQL 编辑器 + 结果表格）；新增 TypeScript 客户端 `sqliteClient.ts` 封装 invoke 调用。

**Tech Stack:** Rust + rusqlite 0.32 (bundled) + Tauri 2.0 / Vue 3 + Element Plus + @tauri-apps/plugin-dialog

---

## 文件结构

| 文件 | 职责 | 操作 |
|------|------|------|
| `src-tauri/src/sqlite_viewer.rs` | SQLite 只读查询命令（5 个 Tauri 命令 + 1 个 test） | 新建 |
| `src-tauri/src/main.rs` | 注册新命令到 invoke_handler | 修改 |
| `src/utils/sqliteClient.ts` | TypeScript 类型定义 + invoke 封装 | 新建 |
| `src/views/SqliteViewerView.vue` | 查看器页面（三栏布局） | 新建 |
| `src/store/index.ts` | TOOL_LIST 添加 sqlite-viewer 条目 | 修改 |
| `src/App.vue` | 添加 SqliteViewerView 路由分支 | 修改 |

---

### Task 1: 后端 — 数据结构与核心辅助函数

**Files:**
- Create: `src-tauri/src/sqlite_viewer.rs`

- [ ] **Step 1: 创建 sqlite_viewer.rs，定义数据结构和辅助函数**

创建 `src-tauri/src/sqlite_viewer.rs`：

```rust
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use serde::Serialize;
use std::time::Instant;

// ============ 数据结构 ============

#[derive(Serialize)]
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,
}

#[derive(Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: usize,
    pub execution_ms: u64,
}

// ============ 辅助函数 ============

/// 以只读模式打开数据库文件
fn open_db(path: &str) -> Result<Connection, String> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("无法打开数据库文件: {}", e))
}

/// 将 rusqlite 的值转为 serde_json::Value
fn value_to_json(value: ValueRef) -> serde_json::Value {
    match value {
        ValueRef::Null => serde_json::Value::Null,
        ValueRef::Integer(i) => serde_json::json!(i),
        ValueRef::Real(f) => serde_json::json!(f),
        ValueRef::Text(bytes) => {
            serde_json::Value::String(String::from_utf8_lossy(bytes).to_string())
        }
        ValueRef::Blob(bytes) => {
            // ponytail: BLOB 转 hex 字符串展示，避免二进制数据破坏 JSON
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            serde_json::json!(format!("BLOB:{}bytes:{}", bytes.len(), hex.chars().take(100).collect::<String>()))
        }
    }
}

/// 检查 SQL 是否为 SELECT 语句（只允许查询）
fn is_select_sql(sql: &str) -> bool {
    let trimmed = sql.trim();
    // ponytail: 简单前缀检查，覆盖 SELECT / WITH (CTE)；SQLite 不支持 EXPLAIN 外的其他读操作入口
    trimmed.to_uppercase().starts_with("SELECT") || trimmed.to_uppercase().starts_with("WITH")
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过（模块还没注册到 main.rs，但文件本身应能编译）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/sqlite_viewer.rs
git commit -m "feat(sqlite-viewer): 添加后端数据结构与辅助函数"
```

---

### Task 2: 后端 — 实现 5 个 Tauri 命令

**Files:**
- Modify: `src-tauri/src/sqlite_viewer.rs`

- [ ] **Step 1: 在 sqlite_viewer.rs 末尾追加 5 个命令实现**

在 `src-tauri/src/sqlite_viewer.rs` 末尾追加：

```rust
// ============ Tauri 命令 ============

#[tauri::command]
pub fn sqlite_list_tables(db_path: String) -> Result<Vec<TableInfo>, String> {
    let conn = open_db(&db_path)?;
    // ponytail: sqlite_sequence 记录 AUTOINCREMENT 表的行数，比 COUNT(*) 快得多
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| e.to_string())?;
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut tables = Vec::new();
    for name in table_names {
        // 用 MAX(rowid) 估算行数，比 COUNT(*) 快，对调试足够
        let row_count: i64 = conn
            .query_row(
                &format!("SELECT MAX(rowid) FROM \"{}\"", name.replace('"', "\"\"")),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0)
            .unwrap_or(0);
        tables.push(TableInfo { name, row_count });
    }
    Ok(tables)
}

#[tauri::command]
pub fn sqlite_get_schema(db_path: String, table_name: String) -> Result<Vec<ColumnInfo>, String> {
    let conn = open_db(&db_path)?;
    // 用 PRAGMA table_info 获取字段信息
    let pragma = format!("PRAGMA table_info(\"{}\")", table_name.replace('"', "\"\""));
    let mut stmt = conn.prepare(&pragma).map_err(|e| e.to_string())?;
    let columns: Vec<ColumnInfo> = stmt
        .query_map([], |row| {
            // PRAGMA table_info 返回: cid, name, type, notnull, dflt_value, pk
            let not_null: i32 = row.get(3)?;
            let pk: i32 = row.get(5)?;
            let default_val: Option<String> = row.get(4).ok().flatten();
            Ok(ColumnInfo {
                name: row.get(1)?,
                data_type: row.get::<_, String>(2).unwrap_or_default(),
                not_null: not_null != 0,
                is_primary_key: pk != 0,
                default_value: default_val,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(columns)
}

#[tauri::command]
pub fn sqlite_query(
    db_path: String,
    sql: String,
    limit: Option<usize>,
) -> Result<QueryResult, String> {
    if !is_select_sql(&sql) {
        return Err("仅支持 SELECT 或 WITH 查询".to_string());
    }
    let conn = open_db(&db_path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;

    let max_rows = limit.unwrap_or(1000);
    let start = Instant::now();

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("SQL 错误: {}", e))?;
    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows_iter = stmt.query([]).map_err(|e| format!("查询执行失败: {}", e))?;
    let mut rows = Vec::new();
    let mut affected_rows = 0;
    while let Some(row) = rows_iter.next().map_err(|e| e.to_string())? {
        if rows.len() >= max_rows {
            break;
        }
        let mut row_values = Vec::with_capacity(column_count);
        for i in 0..column_count {
            let val = row.get_ref(i).map_err(|e| e.to_string())?;
            row_values.push(value_to_json(val));
        }
        rows.push(row_values);
        affected_rows += 1;
    }

    Ok(QueryResult {
        columns: column_names,
        rows,
        affected_rows,
        execution_ms: start.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub fn sqlite_table_preview(db_path: String, table_name: String) -> Result<QueryResult, String> {
    let sql = format!("SELECT * FROM \"{}\"", table_name.replace('"', "\"\""));
    sqlite_query(db_path, sql, Some(100))
}

#[tauri::command]
pub fn sqlite_export_csv(
    db_path: String,
    sql: String,
    save_path: String,
) -> Result<usize, String> {
    if !is_select_sql(&sql) {
        return Err("仅支持 SELECT 或 WITH 查询".to_string());
    }
    let result = sqlite_query(db_path, sql, Some(100000))?;

    let mut csv_content = String::new();
    // 表头
    csv_content.push_str(&result.columns.join(","));
    csv_content.push('\n');
    // 数据行
    for row in &result.rows {
        let line: Vec<String> = row
            .iter()
            .map(|v| {
                match v {
                    serde_json::Value::Null => String::new(),
                    serde_json::Value::String(s) => {
                        // CSV 转义：含逗号/引号/换行的用双引号包裹，内部引号翻倍
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s.clone()
                        }
                    }
                    other => other.to_string(),
                }
            })
            .collect();
        csv_content.push_str(&line.join(","));
        csv_content.push('\n');
    }

    std::fs::write(&save_path, csv_content.as_bytes())
        .map_err(|e| format!("文件写入失败: {}", e))?;
    Ok(result.rows.len())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/sqlite_viewer.rs
git commit -m "feat(sqlite-viewer): 实现5个Tauri查询命令"
```

---

### Task 3: 后端 — 注册命令到 main.rs

**Files:**
- Modify: `src-tauri/src/main.rs:3-10` (添加 mod 声明)
- Modify: `src-tauri/src/main.rs:21-99` (添加到 invoke_handler)

- [ ] **Step 1: 在 main.rs 添加模块声明**

在 `src-tauri/src/main.rs` 第 10 行 `mod system_info;` 后添加：

```rust
mod sqlite_viewer;
```

- [ ] **Step 2: 在 invoke_handler 注册 5 个命令**

在 `src-tauri/src/main.rs` 的 `invoke_handler` 宏中，在 `system_info::get_software_env,` 之后添加：

```rust
            // SQLite 查看器命令
            sqlite_viewer::sqlite_list_tables,
            sqlite_viewer::sqlite_get_schema,
            sqlite_viewer::sqlite_query,
            sqlite_viewer::sqlite_table_preview,
            sqlite_viewer::sqlite_export_csv,
```

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无警告

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(sqlite-viewer): 注册SQLite查看器命令到main.rs"
```

---

### Task 4: 后端 — 添加单元测试

**Files:**
- Modify: `src-tauri/src/sqlite_viewer.rs`

- [ ] **Step 1: 在 sqlite_viewer.rs 末尾添加测试模块**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(file.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT);
             INSERT INTO users (name, email) VALUES ('张三', 'zhang@san.com'), ('李四', 'li@si.com');
             CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount REAL);",
        )
        .unwrap();
        file
    }

    #[test]
    fn test_list_tables() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let tables = sqlite_list_tables(path.to_string()).unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "orders"); // 字母序
        assert_eq!(tables[1].name, "users");
    }

    #[test]
    fn test_get_schema() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let cols = sqlite_get_schema(path.to_string(), "users".to_string()).unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "id");
        assert!(cols[0].is_primary_key);
        assert_eq!(cols[1].name, "name");
        assert!(cols[1].not_null);
        assert_eq!(cols[2].name, "email");
        assert!(!cols[2].not_null);
    }

    #[test]
    fn test_query_select() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let result = sqlite_query(
            path.to_string(),
            "SELECT name, email FROM users ORDER BY id".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(result.columns, vec!["name", "email"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], serde_json::json!("张三"));
        assert_eq!(result.rows[1][0], serde_json::json!("李四"));
    }

    #[test]
    fn test_query_rejects_non_select() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let result = sqlite_query(
            path.to_string(),
            "DELETE FROM users".to_string(),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("仅支持 SELECT"));
    }

    #[test]
    fn test_query_limit() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let result = sqlite_query(
            path.to_string(),
            "SELECT * FROM users".to_string(),
            Some(1),
        )
        .unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_table_preview() {
        let db = create_test_db();
        let path = db.path().to_str().unwrap();
        let result = sqlite_table_preview(path.to_string(), "users".to_string()).unwrap();
        assert_eq!(result.columns, vec!["id", "name", "email"]);
        assert_eq!(result.rows.len(), 2);
    }
}
```

- [ ] **Step 2: 添加 tempfile 开发依赖到 Cargo.toml**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 之后添加：

```toml
[dev-dependencies]
tempfile = "3.10"
```

- [ ] **Step 3: 运行测试**

Run: `cd src-tauri && cargo test sqlite_viewer`
Expected: 6 个测试全部通过

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/sqlite_viewer.rs src-tauri/Cargo.toml
git commit -m "test(sqlite-viewer): 添加后端单元测试"
```

---

### Task 5: 前端 — 创建 TypeScript 客户端

**Files:**
- Create: `src/utils/sqliteClient.ts`

- [ ] **Step 1: 创建 sqliteClient.ts**

创建 `src/utils/sqliteClient.ts`：

```typescript
import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface TableInfo {
  name: string
  row_count: number
}

export interface ColumnInfo {
  name: string
  data_type: string
  not_null: boolean
  is_primary_key: boolean
  default_value: string | null
}

export interface QueryResult {
  columns: string[]
  rows: unknown[][]
  affected_rows: number
  execution_ms: number
}

// ============ invoke 封装 ============

export function sqliteListTables(dbPath: string): Promise<TableInfo[]> {
  return invoke<TableInfo[]>('sqlite_list_tables', { dbPath })
}

export function sqliteGetSchema(dbPath: string, tableName: string): Promise<ColumnInfo[]> {
  return invoke<ColumnInfo[]>('sqlite_get_schema', { dbPath, tableName })
}

export function sqliteQuery(dbPath: string, sql: string, limit?: number): Promise<QueryResult> {
  return invoke<QueryResult>('sqlite_query', { dbPath, sql, limit: limit ?? null })
}

export function sqliteTablePreview(dbPath: string, tableName: string): Promise<QueryResult> {
  return invoke<QueryResult>('sqlite_table_preview', { dbPath, tableName })
}

export function sqliteExportCsv(dbPath: string, sql: string, savePath: string): Promise<number> {
  return invoke<number>('sqlite_export_csv', { dbPath, sql, savePath })
}
```

- [ ] **Step 2: 验证类型检查**

Run: `npx tsc --noEmit`
Expected: 无错误

- [ ] **Step 3: Commit**

```bash
git add src/utils/sqliteClient.ts
git commit -m "feat(sqlite-viewer): 添加前端TypeScript客户端"
```

---

### Task 6: 前端 — 创建查看器页面

**Files:**
- Create: `src/views/SqliteViewerView.vue`

- [ ] **Step 1: 创建 SqliteViewerView.vue**

创建 `src/views/SqliteViewerView.vue`：

```vue
<template>
  <div class="tool-container sqlite-viewer">
    <!-- 文件选择栏 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">数据库文件</span>
        <div class="card-actions">
          <el-button size="small" @click="handleSelectFile">选择 .db 文件</el-button>
          <el-button v-if="dbPath" size="small" @click="handleRefresh">刷新</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="dbPath" class="file-path">{{ dbPath }}</div>
        <div v-else class="empty-hint">请选择一个 SQLite 数据库文件</div>
      </div>
    </div>

    <div v-if="dbPath" class="viewer-body">
      <!-- 左侧：表列表 -->
      <div class="tool-card table-list-card">
        <div class="card-header">
          <span class="card-title">表 ({{ tables.length }})</span>
        </div>
        <div class="card-body table-list-body">
          <div
            v-for="table in tables"
            :key="table.name"
            class="table-item"
            :class="{ active: selectedTable === table.name }"
            @click="handleSelectTable(table.name)"
          >
            <span class="table-name">{{ table.name }}</span>
            <span class="table-rows">{{ table.row_count }}</span>
          </div>
          <div v-if="tables.length === 0" class="empty-hint">无表</div>
        </div>
      </div>

      <!-- 右侧：SQL 编辑器 + 结果 -->
      <div class="viewer-main">
        <!-- SQL 编辑器 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">SQL 查询</span>
            <div class="card-actions">
              <el-button type="primary" size="small" @click="handleExecuteQuery">执行</el-button>
              <el-button size="small" @click="handleClearSql">清空</el-button>
              <el-button size="small" @click="handleExportCsv" :disabled="!lastResult">导出CSV</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="sqlText"
              type="textarea"
              :rows="5"
              placeholder="SELECT * FROM table_name LIMIT 100"
              resize="vertical"
              @keydown.ctrl.enter="handleExecuteQuery"
            />
          </div>
        </div>

        <!-- 结果表格 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">查询结果</span>
            <div class="result-meta" v-if="lastResult">
              耗时: {{ lastResult.execution_ms }}ms | 返回: {{ lastResult.rows.length }}行
              <span v-if="lastResult.rows.length >= 1000" class="truncated-hint">（已截断）</span>
            </div>
          </div>
          <div class="card-body result-body">
            <div v-if="queryError" class="error-message">{{ queryError }}</div>
            <el-table
              v-else-if="lastResult && lastResult.rows.length > 0"
              :data="tableData"
              border
              stripe
              size="small"
              height="100%"
            >
              <el-table-column
                v-for="(col, idx) in lastResult.columns"
                :key="idx"
                :prop="String(idx)"
                :label="col"
                min-width="120"
                show-overflow-tooltip
              />
            </el-table>
            <div v-else-if="lastResult" class="empty-hint">查询结果为空</div>
            <div v-else class="empty-hint">执行查询后在此显示结果</div>
          </div>
        </div>

        <!-- 表结构面板 -->
        <div v-if="schema.length > 0" class="tool-card">
          <div class="card-header">
            <span class="card-title">表结构: {{ selectedTable }}</span>
          </div>
          <div class="card-body">
            <el-table :data="schema" border stripe size="small">
              <el-table-column prop="name" label="字段名" min-width="120" />
              <el-table-column prop="data_type" label="类型" width="120" />
              <el-table-column label="主键" width="60" align="center">
                <template #default="{ row }">
                  <span v-if="row.is_primary_key">是</span>
                </template>
              </el-table-column>
              <el-table-column label="非空" width="60" align="center">
                <template #default="{ row }">
                  <span v-if="row.not_null">是</span>
                </template>
              </el-table-column>
              <el-table-column prop="default_value" label="默认值" min-width="100" />
            </el-table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElLoading } from 'element-plus'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  sqliteListTables,
  sqliteGetSchema,
  sqliteQuery,
  sqliteTablePreview,
  sqliteExportCsv,
  type TableInfo,
  type ColumnInfo,
  type QueryResult,
} from '@/utils/sqliteClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const dbPath = ref('')
const tables = ref<TableInfo[]>([])
const selectedTable = ref('')
const sqlText = ref('')
const lastResult = ref<QueryResult | null>(null)
const queryError = ref('')
const schema = ref<ColumnInfo[]>([])

// 将结果行转为 el-table 可用的对象数组
const tableData = computed(() => {
  if (!lastResult.value) return []
  return lastResult.value.rows.map((row) => {
    const obj: Record<string, unknown> = {}
    row.forEach((val, idx) => {
      obj[String(idx)] = val
    })
    return obj
  })
})

const handleSelectFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'SQLite 数据库', extensions: ['db', 'sqlite', 'sqlite3'] }],
  })
  if (typeof selected !== 'string') return

  dbPath.value = selected
  await loadTables()
}

const loadTables = async () => {
  const loading = ElLoading.service({ text: '加载表列表...' })
  try {
    tables.value = await sqliteListTables(dbPath.value)
    selectedTable.value = ''
    schema.value = []
    lastResult.value = null
    queryError.value = ''
    ElMessage.success(`已加载 ${tables.value.length} 个表`)
  } catch (e) {
    ElMessage.error(String(e))
    tables.value = []
  } finally {
    loading.close()
  }
}

const handleSelectTable = async (tableName: string) => {
  selectedTable.value = tableName
  sqlText.value = `SELECT * FROM "${tableName}" LIMIT 100`
  queryError.value = ''

  const loading = ElLoading.service({ text: '加载数据...' })
  try {
    // 并行加载预览数据和表结构
    const [preview, schemaResult] = await Promise.all([
      sqliteTablePreview(dbPath.value, tableName),
      sqliteGetSchema(dbPath.value, tableName),
    ])
    lastResult.value = preview
    schema.value = schemaResult
  } catch (e) {
    lastResult.value = null
    queryError.value = String(e)
    schema.value = []
  } finally {
    loading.close()
  }
}

const handleExecuteQuery = async () => {
  if (!sqlText.value.trim()) {
    ElMessage.warning('请输入 SQL 语句')
    return
  }
  const loading = ElLoading.service({ text: '执行查询...' })
  try {
    lastResult.value = await sqliteQuery(dbPath.value, sqlText.value)
    queryError.value = ''
    store.addHistory({
      tool: 'sqlite-viewer',
      action: '执行查询',
      inputPreview: sqlText.value.slice(0, 50),
      outputPreview: `${lastResult.value.rows.length}行结果`,
      inputFull: sqlText.value,
      outputFull: JSON.stringify(lastResult.value.rows.slice(0, 50)),
    })
    ElMessage.success(`查询完成，返回 ${lastResult.value.rows.length} 行`)
  } catch (e) {
    lastResult.value = null
    queryError.value = String(e)
    ElMessage.error('查询失败')
  } finally {
    loading.close()
  }
}

const handleClearSql = () => {
  sqlText.value = ''
  lastResult.value = null
  queryError.value = ''
}

const handleExportCsv = async () => {
  if (!lastResult.value || !sqlText.value.trim()) return
  const savePath = await save({
    filters: [{ name: 'CSV 文件', extensions: ['csv'] }],
    defaultPath: 'query_result.csv',
  })
  if (!savePath) return

  const loading = ElLoading.service({ text: '导出中...' })
  try {
    const count = await sqliteExportCsv(dbPath.value, sqlText.value, savePath)
    ElMessage.success(`已导出 ${count} 行到 ${savePath}`)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    loading.close()
  }
}

const handleRefresh = async () => {
  await loadTables()
}
</script>

<style scoped>
.sqlite-viewer {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  padding: 16px 20px;
}

.file-path {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

.empty-hint {
  color: var(--text-secondary);
  text-align: center;
  padding: 24px;
  font-size: 13px;
}

.viewer-body {
  display: flex;
  gap: 16px;
  min-height: 0;
}

.table-list-card {
  width: 220px;
  flex-shrink: 0;
}

.table-list-body {
  padding: 8px;
  max-height: calc(100vh - 200px);
  overflow-y: auto;
}

.table-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.table-item:hover {
  background: rgba(0, 212, 255, 0.08);
}

.table-item.active {
  background: rgba(0, 212, 255, 0.15);
  color: var(--accent-cyan);
}

.table-name {
  font-weight: 500;
}

.table-rows {
  color: var(--text-secondary);
  font-size: 11px;
}

.viewer-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.result-meta {
  font-size: 12px;
  color: var(--text-secondary);
}

.truncated-hint {
  color: #eab308;
}

.result-body {
  height: 320px;
  overflow: hidden;
}

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}
</style>
```

- [ ] **Step 2: 验证类型检查**

Run: `npx tsc --noEmit`
Expected: 无错误

- [ ] **Step 3: Commit**

```bash
git add src/views/SqliteViewerView.vue
git commit -m "feat(sqlite-viewer): 添加SQLite查看器页面"
```

---

### Task 7: 前端 — 注册工具到 TOOL_LIST 和 App.vue

**Files:**
- Modify: `src/store/index.ts:85` (TOOL_LIST 末尾)
- Modify: `src/App.vue:49` (添加路由分支)
- Modify: `src/App.vue:107` (添加 import)

- [ ] **Step 1: 在 store/index.ts 的 TOOL_LIST 数组末尾添加工具条目**

在 `src/store/index.ts` 的 `softwareEnv` 条目（约第 85 行）之后，`]` 之前添加：

```typescript
  { id: 'sqliteViewer', name: 'SQLite查看器', icon: '', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 5v6c0 1.66-4.03 3-9 3s-9-1.34-9-3V5"/><path d="M21 11v6c0 1.66-4.03 3-9 3s-9-1.34-9-3v-6"/></svg>`, description: '浏览本地SQLite数据库表结构和数据，执行SELECT查询，导出CSV', keywords: ['sqlite', '数据库', 'db', '查询', '查看', 'database'], category: 'dev' },
```

- [ ] **Step 2: 在 App.vue 添加 import 和路由分支**

在 `src/App.vue` 第 107 行 `import SoftwareEnvView from '@/views/SoftwareEnvView.vue'` 之后添加：

```typescript
import SqliteViewerView from '@/views/SqliteViewerView.vue'
```

在 `src/App.vue` 第 49 行 `<SoftwareEnvView v-else-if="activeTool === 'softwareEnv'" :key="'softwareEnv'" />` 之后添加：

```vue
          <SqliteViewerView v-else-if="activeTool === 'sqliteViewer'" :key="'sqliteViewer'" />
```

- [ ] **Step 3: 验证类型检查**

Run: `npx tsc --noEmit`
Expected: 无错误

- [ ] **Step 4: Commit**

```bash
git add src/store/index.ts src/App.vue
git commit -m "feat(sqlite-viewer): 注册SQLite查看器到工具箱"
```

---

### Task 8: 端到端验证

**Files:** 无修改

- [ ] **Step 1: 启动开发服务器**

Run: `npm run tauri dev`
Expected: 应用正常启动，侧边栏出现 "SQLite查看器" 工具

- [ ] **Step 2: 手动验证功能**

1. 点击侧边栏 "SQLite查看器"
2. 点击"选择 .db 文件"，选择项目自带的 `%APPDATA%\com.dev.toolbox\litobox.db`
3. 左侧应显示表列表（如 workflows、variables、history 等）
4. 点击某个表，右下方应显示前 100 条数据，底部显示表结构
5. 在 SQL 编辑器输入 `SELECT * FROM history LIMIT 10`，点击执行
6. 结果应显示查询数据，底部显示耗时和行数
7. 点击"导出CSV"，选择保存路径，验证 CSV 文件内容正确

- [ ] **Step 3: 验证错误处理**

1. 在 SQL 编辑器输入 `DELETE FROM history`，点击执行 → 应提示"仅支持 SELECT 或 WITH 查询"
2. 输入 `SELECT * FROM nonexistent_table` → 应提示 SQL 错误
3. 选择一个非 SQLite 文件 → 应提示"无法打开数据库文件"

- [ ] **Step 4: 最终 Commit（如有遗漏的修复）**

```bash
git add -A
git commit -m "feat(sqlite-viewer): 完成SQLite查看器工具"
```
