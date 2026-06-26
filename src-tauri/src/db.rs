use rusqlite::{Connection, Result};
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
