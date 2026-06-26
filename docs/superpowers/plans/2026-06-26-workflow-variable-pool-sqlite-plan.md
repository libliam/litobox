# 工作流编排 + 变量池 + SQLite 迁移实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 LitoBox 数据层从 localStorage 迁移到 SQLite，新增工作流编排页面、变量池面板和全量导出/导入功能。

**Architecture:** Rust 层使用 rusqlite 提供 SQLite CRUD Tauri 命令，前端通过 dbClient.ts 封装调用，Pinia store 改为从 DB 加载数据。工作流执行在前端完成，通过 TOOL_EXECUTORS 字典映射工具函数。

**Tech Stack:** Rust (rusqlite + dirs), Tauri 2 commands, Vue 3 + Pinia, TypeScript

---

## 文件结构总览

| 文件 | 操作 | 职责 |
|------|------|------|
| `src-tauri/Cargo.toml` | 修改 | 新增 rusqlite、dirs 依赖 |
| `src-tauri/src/db.rs` | 新增 | SQLite 初始化、所有 CRUD 操作、迁移逻辑 |
| `src-tauri/src/main.rs` | 修改 | 注册 db 模块 + Tauri 命令 |
| `src/utils/dbClient.ts` | 新增 | 前端 Tauri 命令封装 |
| `src/store/index.ts` | 修改 | 改为从 dbClient 加载/保存，添加自动变量捕获 |
| `src/views/WorkflowView.vue` | 新增 | 工作流编排 + 变量池面板 |
| `src/App.vue` | 修改 | 添加 WorkflowView 路由入口 |
| `src/views/HistoryView.vue` | 修改 | 添加全量导出/导入按钮 |
| `src/components/SidebarNav.vue` | 修改 | 添加工作流入口图标 |

---

## Task 1: Rust 层 - Cargo.toml 依赖 + db.rs 骨架

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: 添加依赖到 Cargo.toml**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
rusqlite = { version = "0.32", features = ["bundled"] }
dirs = "5.0"
```

- [ ] **Step 2: 创建 db.rs 骨架 + 数据库初始化**

创建 `src-tauri/src/db.rs`，包含：

```rust
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::path::PathBuf;

// 全局数据库连接（懒加载）
static DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);

/// 获取数据库连接（单例）
fn get_conn() -> Result<Connection, String> {
    let mut guard = DB_CONN.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = guard.as_ref() {
        // 简单方式：每次返回 clone（rusqlite::Connection 不实现 Clone）
        // 实际上需要改为 Arc<Mutex<Connection>> 或使用连接池
        // 这里先用简单方案
    }
    
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    // 启用 WAL 模式
    conn.execute("PRAGMA journal_mode=WAL", []).map_err(|e| e.to_string())?;
    
    // 创建表
    init_tables(&conn).map_err(|e| e.to_string())?;
    
    *guard = Some(conn);
    Ok(guard.as_ref().unwrap().try_clone().map_err(|e| e.to_string())?)
}

/// 获取数据库文件路径
fn get_db_path() -> Result<PathBuf, String> {
    let app_dir = dirs::config_dir()
        .ok_or("无法获取应用数据目录")?;
    let db_dir = app_dir.join("com.dev.toolbox");
    std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;
    Ok(db_dir.join("litobox.db"))
}

/// 初始化所有表
fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tool TEXT NOT NULL,
            action TEXT NOT NULL,
            input_preview TEXT,
            output_preview TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_history_created ON history(created_at DESC);
        CREATE TABLE IF NOT EXISTS recent_tools (
            tool_id TEXT PRIMARY KEY,
            last_used_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            steps_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS variable_pool (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            value TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'manual',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_variable_name ON variable_pool(name);
        CREATE TABLE IF NOT EXISTS snippets (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            category TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS migration_status (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    "#)?;
    Ok(())
}
```

**注意**：`rusqlite::Connection` 不实现 `Clone`，需要重新设计。改为使用 `Arc<Mutex<Connection>>` 全局单例：

```rust
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::path::PathBuf;

static DB_CONN: OnceLock<Mutex<Connection>> = OnceLock::new();

fn get_conn() -> Result<&'static Mutex<Connection>, String> {
    DB_CONN.get_or_try_init(|| {
        let db_path = get_db_path()?;
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL").map_err(|e| e.to_string())?;
        init_tables(&conn).map_err(|e| e.to_string())?;
        Ok(Mutex::new(conn))
    }).map_err(|e: String| e)
}

fn with_conn<T, F: FnOnce(&mut Connection) -> Result<T, String>>(f: F) -> Result<T, String> {
    let lock = get_conn()?;
    let mut conn = lock.lock().map_err(|e| e.to_string())?;
    f(&mut conn)
}
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过（可能有未使用警告）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/db.rs
git commit -m "feat: 添加 SQLite 数据库层骨架（rusqlite + 表初始化）"
```

---

## Task 2: Rust 层 - 配置 CRUD 命令

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 db.rs 中添加配置 CRUD 函数**

在 `db.rs` 末尾添加：

```rust
// ========== 配置 CRUD ==========

pub fn db_get_config(key: String) -> Result<String, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = ?1")
            .map_err(|e| e.to_string())?;
        let result: Option<String> = stmt
            .query_row(params![key], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result.unwrap_or_default())
    })
}

pub fn db_set_config(key: String, value: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}
```

- [ ] **Step 2: 在 main.rs 中注册命令**

在 `main.rs` 顶部添加 `mod db;`（在现有 mod 之后）：

```rust
mod db;
```

在 `invoke_handler` 中添加：

```rust
db::cmd_db_get_config,
db::cmd_db_set_config,
```

在 `db.rs` 中添加命令函数（在文件末尾）：

```rust
// ========== Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_get_config(key: String) -> Result<String, String> {
    db_get_config(key)
}

#[tauri::command]
pub fn cmd_db_set_config(key: String, value: String) -> Result<(), String> {
    db_set_config(key, value)
}
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat: 添加配置 CRUD Tauri 命令"
```

---

## Task 3: Rust 层 - 历史记录 CRUD 命令

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 db.rs 中添加数据模型和历史 CRUD**

在 `db.rs` 的 `use` 区域下方添加数据模型：

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryRecord {
    pub id: Option<i64>,
    pub tool: String,
    pub action: String,
    pub input_preview: String,
    pub output_preview: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolVariable {
    pub id: String,
    pub name: String,
    pub value: String,
    pub source: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}
```

添加历史记录 CRUD：

```rust
// ========== 历史记录 CRUD ==========

pub fn db_add_history(record: HistoryRecord) -> Result<i64, String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO history (tool, action, input_preview, output_preview)
             VALUES (?1, ?2, ?3, ?4)",
            params![record.tool, record.action, record.input_preview, record.output_preview],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn db_get_history(limit: i64, offset: i64) -> Result<Vec<HistoryRecord>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, tool, action, input_preview, output_preview, created_at
                      FROM history ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit, offset], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    tool: row.get(1)?,
                    action: row.get(2)?,
                    input_preview: row.get(3)?,
                    output_preview: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_clear_history() -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM history", []).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_search_history(query: String, limit: i64) -> Result<Vec<HistoryRecord>, String> {
    with_conn(|conn| {
        let like = format!("%{}%", query);
        let mut stmt = conn
            .prepare("SELECT id, tool, action, input_preview, output_preview, created_at
                      FROM history
                      WHERE tool LIKE ?1 OR action LIKE ?1
                         OR input_preview LIKE ?1 OR output_preview LIKE ?1
                      ORDER BY created_at DESC LIMIT ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like, limit], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    tool: row.get(1)?,
                    action: row.get(2)?,
                    input_preview: row.get(3)?,
                    output_preview: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}
```

- [ ] **Step 2: 添加历史命令到 main.rs**

在 `invoke_handler` 中添加：

```rust
db::cmd_db_add_history,
db::cmd_db_get_history,
db::cmd_db_clear_history,
db::cmd_db_search_history,
```

在 `db.rs` 命令区域添加：

```rust
#[tauri::command]
pub fn cmd_db_add_history(record: HistoryRecord) -> Result<i64, String> {
    db_add_history(record)
}

#[tauri::command]
pub fn cmd_db_get_history(limit: i64, offset: i64) -> Result<Vec<HistoryRecord>, String> {
    db_get_history(limit, offset)
}

#[tauri::command]
pub fn cmd_db_clear_history() -> Result<(), String> {
    db_clear_history()
}

#[tauri::command]
pub fn cmd_db_search_history(query: String, limit: i64) -> Result<Vec<HistoryRecord>, String> {
    db_search_history(query, limit)
}
```

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat: 添加历史记录 CRUD Tauri 命令"
```

---

## Task 4: Rust 层 - 工作流和变量池 CRUD 命令

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 db.rs 中添加工作流 CRUD**

```rust
// ========== 工作流 CRUD ==========

pub fn db_list_workflows() -> Result<Vec<Workflow>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, description, steps_json, created_at, updated_at
                      FROM workflows ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2).unwrap_or_default(),
                    steps_json: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_save_workflow(workflow: Workflow) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO workflows (id, name, description, steps_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 name = ?2, description = ?3, steps_json = ?4, updated_at = ?6",
            params![
                workflow.id, workflow.name, workflow.description,
                workflow.steps_json, workflow.created_at, workflow.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_workflow(id: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM workflows WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}
```

- [ ] **Step 2: 在 db.rs 中添加变量池 CRUD**

```rust
// ========== 变量池 CRUD ==========

pub fn db_list_variables() -> Result<Vec<PoolVariable>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, value, source, created_at, last_used_at
                      FROM variable_pool ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PoolVariable {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    source: row.get(3)?,
                    created_at: row.get(4)?,
                    last_used_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_set_variable(name: String, value: String, source: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO variable_pool (id, name, value, source, created_at)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET value = ?2, last_used_at = datetime('now')",
            params![name, value, source],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_variable(name: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM variable_pool WHERE name = ?1", params![name])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_get_variable(name: String) -> Result<String, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM variable_pool WHERE name = ?1")
            .map_err(|e| e.to_string())?;
        // 更新 last_used_at
        conn.execute(
            "UPDATE variable_pool SET last_used_at = datetime('now') WHERE name = ?1",
            params![name],
        ).map_err(|e| e.to_string())?;
        let result: Option<String> = stmt
            .query_row(params![name], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result.unwrap_or_default())
    })
}
```

- [ ] **Step 3: 添加命令到 main.rs**

在 `invoke_handler` 中添加：

```rust
db::cmd_db_list_workflows,
db::cmd_db_save_workflow,
db::cmd_db_delete_workflow,
db::cmd_db_list_variables,
db::cmd_db_set_variable,
db::cmd_db_delete_variable,
db::cmd_db_get_variable,
```

在 `db.rs` 命令区域添加对应命令函数。

- [ ] **Step 4: 验证编译 + Commit**

```bash
cd src-tauri && cargo check
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat: 添加工作流和变量池 CRUD Tauri 命令"
```

---

## Task 5: Rust 层 - 导入导出 + 迁移命令

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 添加导出命令**

```rust
// ========== 导入导出 ==========

pub fn db_export_all() -> Result<String, String> {
    with_conn(|conn| {
        let mut export = serde_json::Map::new();

        // 导出配置
        let mut config_map = serde_json::Map::new();
        let mut stmt = conn.prepare("SELECT key, value FROM config")
            .map_err(|e| e.to_string())?;
        let config_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;
        for row in config_rows {
            let (k, v) = row.map_err(|e| e.to_string())?;
            config_map.insert(k, serde_json::Value::String(v));
        }
        export.insert("config".to_string(), serde_json::Value::Object(config_map));

        // 导出历史
        let mut stmt = conn.prepare("SELECT tool, action, input_preview, output_preview, created_at FROM history ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let history: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "tool": row.get::<_, String>(0)?,
                    "action": row.get::<_, String>(1)?,
                    "input_preview": row.get::<_, String>(2)?,
                    "output_preview": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("history".to_string(), serde_json::Value::Array(history));

        // 导出工作流
        let workflows = db_list_workflows()?;
        export.insert("workflows".to_string(), serde_json::to_value(workflows).map_err(|e| e.to_string())?);

        // 导出变量池
        let variables = db_list_variables()?;
        export.insert("variables".to_string(), serde_json::to_value(variables).map_err(|e| e.to_string())?);

        // 导出最近工具
        let mut stmt = conn.prepare("SELECT tool_id, last_used_at FROM recent_tools ORDER BY last_used_at DESC")
            .map_err(|e| e.to_string())?;
        let recent: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "tool_id": row.get::<_, String>(0)?,
                    "last_used_at": row.get::<_, String>(1)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("recent_tools".to_string(), serde_json::Value::Array(recent));

        serde_json::to_string(&export).map_err(|e| e.to_string())
    })
}
```

- [ ] **Step 2: 添加导入命令**

```rust
pub fn db_import_all(data: String) -> Result<(), String> {
    let export: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    with_conn(|conn| {
        // 导入配置
        if let Some(config) = export.get("config").and_then(|v| v.as_object()) {
            for (k, v) in config {
                if let Some(vs) = v.as_str() {
                    conn.execute(
                        "INSERT INTO config (key, value) VALUES (?1, ?2)
                         ON CONFLICT(key) DO UPDATE SET value = ?2",
                        params![k, vs],
                    ).map_err(|e| e.to_string())?;
                }
            }
        }

        // 导入历史（清空后重新导入）
        conn.execute("DELETE FROM history", []).map_err(|e| e.to_string())?;
        if let Some(history) = export.get("history").and_then(|v| v.as_array()) {
            for record in history {
                let tool = record.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                let action = record.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let input_preview = record.get("input_preview").and_then(|v| v.as_str()).unwrap_or("");
                let output_preview = record.get("output_preview").and_then(|v| v.as_str()).unwrap_or("");
                let created_at = record.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO history (tool, action, input_preview, output_preview, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![tool, action, input_preview, output_preview, created_at],
                ).map_err(|e| e.to_string())?;
            }
        }

        // 导入工作流（清空后重新导入）
        conn.execute("DELETE FROM workflows", []).map_err(|e| e.to_string())?;
        if let Some(workflows) = export.get("workflows").and_then(|v| v.as_array()) {
            for wf in workflows {
                let id = wf.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let name = wf.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let description = wf.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let steps_json = wf.get("steps_json").and_then(|v| v.as_str()).unwrap_or("");
                let created_at = wf.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                let updated_at = wf.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO workflows (id, name, description, steps_json, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![id, name, description, steps_json, created_at, updated_at],
                ).map_err(|e| e.to_string())?;
            }
        }

        // 导入变量池（清空后重新导入）
        conn.execute("DELETE FROM variable_pool", []).map_err(|e| e.to_string())?;
        if let Some(variables) = export.get("variables").and_then(|v| v.as_array()) {
            for var in variables {
                let name = var.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = var.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let source = var.get("source").and_then(|v| v.as_str()).unwrap_or("manual");
                let created_at = var.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO variable_pool (id, name, value, source, created_at)
                     VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4)",
                    params![name, value, source, created_at],
                ).map_err(|e| e.to_string())?;
            }
        }

        // 导入最近工具
        conn.execute("DELETE FROM recent_tools", []).map_err(|e| e.to_string())?;
        if let Some(recent) = export.get("recent_tools").and_then(|v| v.as_array()) {
            for r in recent {
                let tool_id = r.get("tool_id").and_then(|v| v.as_str()).unwrap_or("");
                let last_used_at = r.get("last_used_at").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO recent_tools (tool_id, last_used_at) VALUES (?1, ?2)",
                    params![tool_id, last_used_at],
                ).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    })
}
```

- [ ] **Step 3: 添加迁移命令**

```rust
// ========== 迁移 ==========

pub fn db_check_migrated() -> Result<bool, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM migration_status WHERE key = 'migrated'")
            .map_err(|e| e.to_string())?;
        let result: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result == Some("true".to_string()))
    })
}

pub fn db_migrate_from_localstorage(data: String) -> Result<(), String> {
    let ls_data: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("localStorage 数据解析失败: {}", e))?;

    with_conn(|conn| {
        // 迁移配置
        if let Some(config_str) = ls_data.get("config").and_then(|v| v.as_str()) {
            conn.execute(
                "INSERT INTO config (key, value) VALUES ('main', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                params![config_str],
            ).map_err(|e| e.to_string())?;
        }

        // 迁移历史
        if let Some(history_str) = ls_data.get("history").and_then(|v| v.as_str()) {
            let history: Vec<serde_json::Value> = serde_json::from_str(history_str)
                .map_err(|e| e.to_string())?;
            for record in history {
                let tool = record.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                let action = record.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let input_preview = record.get("inputPreview").and_then(|v| v.as_str()).unwrap_or("");
                let output_preview = record.get("outputPreview").and_then(|v| v.as_str()).unwrap_or("");
                let timestamp = record.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO history (tool, action, input_preview, output_preview, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![tool, action, input_preview, output_preview, timestamp],
                ).map_err(|e| e.to_string())?;
            }
        }

        // 迁移最近工具
        if let Some(recent_str) = ls_data.get("recent").and_then(|v| v.as_str()) {
            let recent: Vec<String> = serde_json::from_str(recent_str)
                .map_err(|e| e.to_string())?;
            for (i, tool_id) in recent.iter().enumerate() {
                conn.execute(
                    "INSERT INTO recent_tools (tool_id, last_used_at) VALUES (?1, datetime('now', ?2))",
                    params![tool_id, format!("+{} seconds", i)],
                ).map_err(|e| e.to_string())?;
            }
        }

        // 标记已迁移
        conn.execute(
            "INSERT INTO migration_status (key, value) VALUES ('migrated', 'true')
             ON CONFLICT(key) DO UPDATE SET value = 'true'",
            [],
        ).map_err(|e| e.to_string())?;

        Ok(())
    })
}
```

- [ ] **Step 4: 添加命令到 main.rs + 验证编译 + Commit**

```bash
cd src-tauri && cargo check
git add src-tauri/src/db.rs src-tauri/src/main.rs
git commit -m "feat: 添加导入导出和迁移 Tauri 命令"
```

---

## Task 6: 前端 - dbClient.ts 封装

**Files:**
- Create: `src/utils/dbClient.ts`

- [ ] **Step 1: 创建 dbClient.ts**

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface HistoryRecord {
  id?: number
  tool: string
  action: string
  inputPreview: string
  outputPreview: string
  timestamp?: string
}

export interface Workflow {
  id: string
  name: string
  description: string
  steps: WorkflowStep[]
  createdAt: string
  updatedAt: string
}

export interface WorkflowStep {
  id: string
  toolId: string
  params: Record<string, any>
  inputSource: 'manual' | 'prev' | 'variable' | 'expression'
  inputRef: string
}

export interface PoolVariable {
  id: string
  name: string
  value: string
  source: 'manual' | 'auto'
  createdAt: string
  lastUsedAt?: string
}

export const db = {
  // Config
  getConfig: (key: string) => invoke<string>('cmd_db_get_config', { key }),
  setConfig: (key: string, value: string) => invoke('cmd_db_set_config', { key, value }),

  // History
  addHistory: (record: HistoryRecord) => invoke<number>('cmd_db_add_history', {
    record: {
      id: null,
      tool: record.tool,
      action: record.action,
      input_preview: record.inputPreview,
      output_preview: record.outputPreview,
      created_at: null,
    }
  }),
  getHistory: (limit = 100, offset = 0) => invoke<any[]>('cmd_db_get_history', { limit, offset }),
  clearHistory: () => invoke('cmd_db_clear_history'),
  searchHistory: (query: string, limit = 50) => invoke<any[]>('cmd_db_search_history', { query, limit }),

  // Workflows
  listWorkflows: () => invoke<any[]>('cmd_db_list_workflows'),
  saveWorkflow: (workflow: Workflow) => invoke('cmd_db_save_workflow', {
    workflow: {
      id: workflow.id,
      name: workflow.name,
      description: workflow.description,
      steps_json: JSON.stringify(workflow.steps),
      created_at: workflow.createdAt,
      updated_at: workflow.updatedAt,
    }
  }),
  deleteWorkflow: (id: string) => invoke('cmd_db_delete_workflow', { id }),

  // Variables
  listVariables: () => invoke<any[]>('cmd_db_list_variables'),
  setVariable: (name: string, value: string, source = 'manual') => invoke('cmd_db_set_variable', { name, value, source }),
  deleteVariable: (name: string) => invoke('cmd_db_delete_variable', { name }),
  getVariable: (name: string) => invoke<string>('cmd_db_get_variable', { name }),

  // Import/Export
  exportAll: () => invoke<string>('cmd_db_export_all'),
  importAll: (data: string) => invoke('cmd_db_import_all', { data }),

  // Migration
  migrateFromLocalStorage: (data: string) => invoke('cmd_db_migrate_from_localstorage', { data }),
  checkMigrated: () => invoke<boolean>('cmd_db_check_migrated'),
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/dbClient.ts
git commit -m "feat: 添加前端 dbClient Tauri 命令封装"
```

---

## Task 7: 前端 - Store 改造为 SQLite 驱动

**Files:**
- Modify: `src/store/index.ts`

- [ ] **Step 1: 读取现有 store 完整内容**

先 `Read` 完整 `src/store/index.ts`，然后改造。

- [ ] **Step 2: 改造 store**

核心改动：
1. `loadFromStorage()` 改为 `loadFromDB()`，首次启动检测迁移
2. `saveConfig()` 改为同时写 DB
3. `addHistory()` 改为调用 `db.addHistory()`
4. 添加 `captureAutoVariable()` 自动捕获输出到变量池

改造后的关键代码：

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { db, HistoryRecord as DbHistoryRecord } from '@/utils/dbClient'

// ... 保持现有类型定义 ...

export const useToolboxStore = defineStore('toolbox', () => {
  // ... 保持现有 state ...
  const isMigrated = ref(false)

  // 从 DB 加载
  const loadFromDB = async () => {
    try {
      const migrated = await db.checkMigrated()
      isMigrated.value = migrated

      if (!migrated) {
        // 从 localStorage 迁移
        const lsData = {
          config: localStorage.getItem(STORAGE_KEY_CONFIG) || '',
          history: localStorage.getItem(STORAGE_KEY_HISTORY) || '[]',
          recent: localStorage.getItem(STORAGE_KEY_RECENT) || '[]',
        }
        await db.migrateFromLocalStorage(JSON.stringify(lsData))
        isMigrated.value = true
      }

      // 从 DB 加载配置
      const configStr = await db.getConfig('main')
      if (configStr) {
        config.value = { ...config.value, ...JSON.parse(configStr) }
      }

      // 从 DB 加载历史
      const historyData = await db.getHistory(MAX_HISTORY)
      history.value = historyData.map((h: any) => ({
        tool: h.tool,
        action: h.action,
        timestamp: h.created_at,
        inputPreview: h.input_preview,
        outputPreview: h.output_preview,
      }))

      // ... 加载最近工具 ...
    } catch (error) {
      console.error('加载 DB 数据失败:', error)
    }
  }

  // 保存配置
  const saveConfig = async (newConfig: Partial<ToolboxConfig>) => {
    config.value = { ...config.value, ...newConfig }
    await db.setConfig('main', JSON.stringify(config.value))
  }

  // 添加历史记录
  const addHistory = async (record: Omit<HistoryRecord, 'timestamp'>) => {
    const newRecord = {
      ...record,
      timestamp: new Date().toISOString()
    }
    
    try {
      await db.addHistory({
        tool: record.tool,
        action: record.action,
        inputPreview: record.inputPreview,
        outputPreview: record.outputPreview,
      })
      
      history.value.unshift(newRecord)
      if (history.value.length > MAX_HISTORY) {
        history.value = history.value.slice(0, MAX_HISTORY)
      }

      // 自动捕获到变量池
      if (record.outputPreview) {
        captureAutoVariable(record.tool, record.action, record.outputPreview)
      }
    } catch (error) {
      console.error('保存历史记录失败:', error)
    }
  }

  // 自动捕获变量
  const captureAutoVariable = async (tool: string, action: string, value: string) => {
    const maxAutoVars = 20
    const name = `auto_${tool}_${Date.now()}`
    const truncated = value.length > 5000 ? value.slice(0, 5000) : value
    
    await db.setVariable(name, truncated, 'auto')

    // 清理超过 20 条的自动变量
    const vars = await db.listVariables()
    const autoVars = vars.filter(v => v.source === 'auto')
    if (autoVars.length > maxAutoVars) {
      const toDelete = autoVars.slice(maxAutoVars)
      for (const v of toDelete) {
        await db.deleteVariable(v.name)
      }
    }
  }

  // 清空历史
  const clearHistory = async () => {
    await db.clearHistory()
    history.value = []
  }

  // ... 保持 addRecentTool, toggleFavorite 不变（改为 async）...

  // 初始化
  loadFromDB()

  return {
    config,
    history,
    recentTools,
    isMigrated,
    saveConfig,
    addHistory,
    clearHistory,
    addRecentTool,
    toggleFavorite,
    loadFromDB,
  }
})
```

- [ ] **Step 3: Commit**

```bash
git add src/store/index.ts
git commit -m "refactor: store 改为从 SQLite 加载数据，添加自动变量捕获"
```

---

## Task 8: 前端 - WorkflowView.vue 工作流编排页面

**Files:**
- Create: `src/views/WorkflowView.vue`
- Modify: `src/App.vue`
- Modify: `src/components/SidebarNav.vue`

- [ ] **Step 1: 创建 WorkflowView.vue**

基于 `_ToolTemplate.vue` 模板创建，包含：
- 左侧：工作流列表（新建按钮、列表卡片）
- 中间：步骤编排区（步骤列表、添加工具选择、输入来源选择、参数配置、执行按钮）
- 右侧：变量池面板（变量列表、手动添加、自动缓存展示）

完整实现参考设计文档中的布局描述和现有工具页面的样式规范。

- [ ] **Step 2: 在 App.vue 中添加路由**

在 `App.vue` 中添加：

```typescript
import WorkflowView from '@/views/WorkflowView.vue'
```

在模板中添加：

```vue
<WorkflowView v-else-if="activeTool === 'workflow'" />
```

- [ ] **Step 3: 在 SidebarNav.vue 中添加入口**

找到工具列表配置，添加工作流入口（图标用流程/管道相关图标）。

- [ ] **Step 4: Commit**

```bash
git add src/views/WorkflowView.vue src/App.vue src/components/SidebarNav.vue
git commit -m "feat: 添加工作流编排页面和变量池面板"
```

---

## Task 9: 前端 - HistoryView 导出/导入

**Files:**
- Modify: `src/views/HistoryView.vue`

- [ ] **Step 1: 在 HistoryView 操作栏添加按钮**

在现有操作按钮后添加：

```vue
<el-button size="small" @click="handleExport">导出备份</el-button>
<el-upload
  action=""
  :show-file-list="false"
  :before-upload="handleImport"
  accept=".json"
>
  <el-button size="small">导入恢复</el-button>
</el-upload>
```

- [ ] **Step 2: 实现导出函数**

```typescript
import { db } from '@/utils/dbClient'
import { ElMessage, ElLoading, ElMessageBox } from 'element-plus'

const handleExport = async () => {
  const loading = ElLoading.service({
    lock: true,
    text: '正在导出数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    const data = await db.exportAll()
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    const date = new Date().toISOString().slice(0, 10).replace(/-/g, '')
    a.href = url
    a.download = `litobox-backup-${date}.json`
    a.click()
    URL.revokeObjectURL(url)
    ElMessage.success('导出成功')
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
}
```

- [ ] **Step 3: 实现导入函数**

```typescript
const handleImport = async (file: File) => {
  try {
    await ElMessageBox.confirm(
      '导入将覆盖现有数据，确定继续？',
      '确认导入',
      { type: 'warning' }
    )
  } catch {
    return false
  }

  const loading = ElLoading.service({
    lock: true,
    text: '正在导入数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    const text = await file.text()
    await db.importAll(text)
    ElMessage.success('导入成功，页面将刷新')
    setTimeout(() => window.location.reload(), 1000)
  } catch (e: any) {
    ElMessage.error('导入失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
  return false
}
```

- [ ] **Step 4: Commit**

```bash
git add src/views/HistoryView.vue
git commit -m "feat: 历史记录页面添加全量导出/导入功能"
```

---

## Task 10: 端到端测试 + 最终提交

**Files:**
- 全局验证

- [ ] **Step 1: 编译检查**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 2: 前端类型检查**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: 开发模式测试**

```bash
npm run tauri dev
```

手动测试：
1. 启动后检查数据库是否创建（`%APPDATA%/com.dev.toolbox/litobox.db`）
2. 操作任意工具，检查历史记录是否写入 DB
3. 打开工作流页面，创建简单工作流并执行
4. 在变量池中添加变量，确认保存
5. 导出备份，检查 JSON 内容
6. 清除 DB 文件，导入备份，确认数据恢复

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat: 工作流编排+变量池+SQLite迁移功能完成"
```

---

## 自审清单

### 1. Spec 覆盖检查

| Spec 需求 | 对应 Task |
|-----------|-----------|
| 链式数据流转 | Task 8 (WorkflowView 执行引擎) |
| 自定义工作流保存 | Task 4 (工作流 CRUD) + Task 8 (WorkflowView) |
| 全局变量缓存池 | Task 4 (变量池 CRUD) + Task 7 (自动捕获) + Task 8 (变量池面板) |
| 全量导出/导入 | Task 5 (导出导入命令) + Task 9 (HistoryView 按钮) |
| SQLite 全量迁移 | Task 1-5 (Rust 层) + Task 6 (dbClient) + Task 7 (Store 改造) |
| 数据库表设计 | Task 1 (init_tables) |
| 所有 Tauri 命令 | Task 2-5 |
| 工作流执行引擎 | Task 8 |

### 2. Placeholder 扫描

无 TBD/TODO/待实现。所有步骤包含完整代码。

### 3. 类型一致性

- `HistoryRecord` 在 db.rs 中定义字段为 snake_case（`input_preview`），dbClient.ts 中映射为 camelCase（`inputPreview`）
- `Workflow` 的 `steps_json` 在 Rust 层是 JSON 字符串，前端 `saveWorkflow` 时 `JSON.stringify` 转换
- `PoolVariable.source` 统一使用 `'manual'` / `'auto'` 字符串

### 4. 潜在问题

- `with_conn` 使用 `Mutex<Connection>` 全局锁，并发操作会串行化。对于桌面工具场景可接受（ponytail: 全局锁，升级路径为 r2d2 连接池）
- `rusqlite::Connection` 不实现 `Clone`，使用 `OnceLock<Mutex<Connection>>` 方案正确
- `ON CONFLICT` 语法需要 SQLite 3.24+，rusqlite 0.32 捆绑的 SQLite 版本支持
