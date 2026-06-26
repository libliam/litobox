# Task 1: Rust 层 - Cargo.toml 依赖 + db.rs 骨架

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

创建 `src-tauri/src/db.rs`。

**重要修正**：计划中的 `get_conn()` 初版代码有注释说 `Connection` 不实现 `Clone`，后面给出了正确方案。请直接使用 `OnceLock<Mutex<Connection>>` 方案，不要用 `Mutex<Option<Connection>>` + `try_clone()`。

完整实现要求：

1. 使用 `OnceLock<Mutex<Connection>>` 作为全局单例
2. `get_conn()` 返回 `&'static Mutex<Connection>`，通过 `get_or_try_init` 懒加载
3. `with_conn<T, F>` 辅助函数：获取 Mutex 锁，调用闭包
4. `get_db_path()` 返回 `%APPDATA%/com.dev.toolbox/litobox.db`
5. `init_tables()` 创建所有表（config, history, recent_tools, workflows, variable_pool, snippets, migration_status）
6. 启用 WAL 模式

完整代码：

```rust
use rusqlite::{Connection, Result};
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

fn get_db_path() -> Result<PathBuf, String> {
    let app_dir = dirs::config_dir()
        .ok_or("无法获取应用数据目录")?;
    let db_dir = app_dir.join("com.dev.toolbox");
    std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;
    Ok(db_dir.join("litobox.db"))
}

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

- [ ] **Step 3: 验证编译**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过（可能有未使用警告，忽略即可）

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/db.rs
git commit -m "feat: 添加 SQLite 数据库层骨架（rusqlite + 表初始化）"
```
