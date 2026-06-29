use rusqlite::{Connection, Result, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::path::PathBuf;

static DB_CONN: OnceLock<Result<Mutex<Connection>, String>> = OnceLock::new();

fn get_conn() -> Result<&'static Mutex<Connection>, String> {
    DB_CONN.get_or_init(|| {
        match do_init() {
            Ok(conn) => Ok(Mutex::new(conn)),
            Err(e) => Err(e),
        }
    }).as_ref().map_err(|e| e.clone())
}

fn do_init() -> Result<Connection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL").map_err(|e| e.to_string())?;
    init_tables(&conn).map_err(|e| e.to_string())?;
    Ok(conn)
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
pub struct HistoryDetail {
    pub id: Option<i64>,
    pub history_id: i64,
    pub input_full: Option<String>,
    pub output_full: Option<String>,
    pub options_json: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ClipboardRecord {
    pub id: Option<i64>,
    pub text: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpEnvironment {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub variables_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpHistoryRecord {
    pub id: Option<i64>,
    pub method: String,
    pub url: String,
    pub headers_json: String,
    pub body: Option<String>,
    pub body_type: String,
    pub env_name: Option<String>,
    pub status: Option<i64>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpBookmark {
    pub id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers_json: String,
    pub body: Option<String>,
    pub body_type: String,
    pub created_at: String,
    pub updated_at: String,
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
            lang TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL,
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS ocr_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thumbnail TEXT NOT NULL,
            original_url TEXT NOT NULL DEFAULT '',
            text TEXT NOT NULL,
            time TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS clipboard_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS http_environments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                base_url TEXT NOT NULL DEFAULT '',
                variables_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        CREATE TABLE IF NOT EXISTS http_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            headers_json TEXT NOT NULL DEFAULT '{}',
            body TEXT,
            body_type TEXT NOT NULL DEFAULT 'json',
            env_name TEXT,
            status INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS http_bookmarks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            method TEXT NOT NULL DEFAULT 'GET',
            url TEXT NOT NULL,
            headers_json TEXT NOT NULL DEFAULT '{}',
            body TEXT,
            body_type TEXT NOT NULL DEFAULT 'json',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE IF NOT EXISTS migration_status (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    "#)?;

    // 迁移：旧版 snippets 表有 category 列但无 lang/note 列
    if let Err(_) = conn.execute("ALTER TABLE snippets ADD COLUMN lang TEXT NOT NULL DEFAULT ''", []) {
        // 列已存在，忽略
    }
    if let Err(_) = conn.execute("ALTER TABLE snippets ADD COLUMN note TEXT NOT NULL DEFAULT ''", []) {
        // 列已存在，忽略
    }
    if let Err(_) = conn.execute("ALTER TABLE http_environments ADD COLUMN base_url TEXT NOT NULL DEFAULT ''", []) {
        // 列已存在，忽略
    }

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

    Ok(())
}

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

// ========== 代码片段 CRUD ==========

#[derive(Serialize, Deserialize, Debug)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub lang: String,
    pub content: String,
    pub note: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn db_list_snippets() -> Result<Vec<Snippet>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, title, lang, content, note, created_at, updated_at FROM snippets ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Snippet {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    lang: row.get(2)?,
                    content: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_save_snippet(snippet: Snippet) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO snippets (id, title, lang, content, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 title = ?2, lang = ?3, content = ?4, note = ?5, updated_at = ?7",
            params![
                snippet.id, snippet.title, snippet.lang,
                snippet.content, snippet.note,
                snippet.created_at, snippet.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_snippet(id: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== 最近工具 CRUD ==========

pub fn db_list_recent_tools(limit: i64) -> Result<Vec<String>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT tool_id FROM recent_tools ORDER BY last_used_at DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_add_recent_tool(tool_id: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO recent_tools (tool_id, last_used_at) VALUES (?1, datetime('now'))
             ON CONFLICT(tool_id) DO UPDATE SET last_used_at = datetime('now')",
            params![tool_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== OCR 历史 CRUD ==========

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrHistoryRecord {
    pub id: Option<i64>,
    pub thumbnail: String,
    pub original_url: String,
    pub text: String,
    pub time: String,
}

pub fn db_list_ocr_history(limit: i64) -> Result<Vec<OcrHistoryRecord>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, thumbnail, original_url, text, time FROM ocr_history ORDER BY id DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(OcrHistoryRecord {
                    id: Some(row.get(0)?),
                    thumbnail: row.get(1)?,
                    original_url: row.get(2)?,
                    text: row.get(3)?,
                    time: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_add_ocr_history(record: OcrHistoryRecord) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO ocr_history (thumbnail, original_url, text, time) VALUES (?1, ?2, ?3, ?4)",
            params![record.thumbnail, record.original_url, record.text, record.time],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_clear_ocr_history() -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM ocr_history", []).map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== 剪贴板历史 CRUD ==========

pub fn db_list_clipboard_history(limit: i64, offset: i64) -> Result<Vec<ClipboardRecord>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, text, timestamp FROM clipboard_history ORDER BY id DESC LIMIT ?1 OFFSET ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit, offset], |row| {
                Ok(ClipboardRecord {
                    id: Some(row.get(0)?),
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_search_clipboard_history(query: String, limit: i64) -> Result<Vec<ClipboardRecord>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, text, timestamp FROM clipboard_history WHERE text LIKE '%' || ?1 || '%' ORDER BY id DESC LIMIT ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![query, limit], |row| {
                Ok(ClipboardRecord {
                    id: Some(row.get(0)?),
                    text: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
}

pub fn db_add_clipboard_record(text: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO clipboard_history (text, timestamp) VALUES (?1, datetime('now'))",
            params![text],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_clipboard_record(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_clear_clipboard_history() -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM clipboard_history", []).map_err(|e| e.to_string())?;
        Ok(())
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

pub fn db_delete_history_details_for_history(history_id: i64) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "DELETE FROM history_details WHERE history_id = ?1",
            params![history_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

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

// ========== 导入导出 ==========

pub fn db_export_all() -> Result<String, String> {
    use std::collections::HashMap;
    
    with_conn(|conn| {
        let mut export = HashMap::new();

        // 导出配置
        let mut config_map = HashMap::new();
        let mut stmt = conn.prepare("SELECT key, value FROM config")
            .map_err(|e| e.to_string())?;
        let config_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| e.to_string())?;
        for row in config_rows {
            let (k, v) = row.map_err(|e| e.to_string())?;
            config_map.insert(k, v);
        }
        export.insert("config", serde_json::Value::Object(config_map.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect()));

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
        export.insert("history", serde_json::Value::Array(history));

        // 导出工作流（直接查询，避免 db_list_workflows 内部再次 with_conn 导致死锁）
        let mut stmt = conn.prepare("SELECT id, name, description, steps_json, created_at, updated_at FROM workflows ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let workflows: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "steps_json": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "updated_at": row.get::<_, String>(5)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("workflows", serde_json::Value::Array(workflows));

        // 导出变量池（直接查询，避免 db_list_variables 内部再次 with_conn 导致死锁）
        let mut stmt = conn.prepare("SELECT id, name, value, source, created_at, last_used_at FROM variable_pool ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let variables: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "value": row.get::<_, String>(2)?,
                    "source": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, String>(4)?,
                    "last_used_at": row.get::<_, Option<String>>(5)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("variables", serde_json::Value::Array(variables));

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
        export.insert("recent_tools", serde_json::Value::Array(recent));

        // 导出代码片段
        let mut stmt = conn.prepare("SELECT id, title, lang, content, note, created_at, updated_at FROM snippets ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let snippets: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "lang": row.get::<_, String>(2)?,
                    "content": row.get::<_, String>(3)?,
                    "note": row.get::<_, String>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                    "updated_at": row.get::<_, String>(6)?,
                }))
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        export.insert("snippets", serde_json::Value::Array(snippets));

        serde_json::to_string(&export).map_err(|e| e.to_string())
    })
}

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

        // 导入代码片段（清空后重新导入）
        conn.execute("DELETE FROM snippets", []).map_err(|e| e.to_string())?;
        if let Some(snippets) = export.get("snippets").and_then(|v| v.as_array()) {
            for s in snippets {
                let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let title = s.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let lang = s.get("lang").and_then(|v| v.as_str()).unwrap_or("");
                let content = s.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let note = s.get("note").and_then(|v| v.as_str()).unwrap_or("");
                let created_at = s.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                let updated_at = s.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
                conn.execute(
                    "INSERT INTO snippets (id, title, lang, content, note, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![id, title, lang, content, note, created_at, updated_at],
                ).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    })
}

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

// ========== Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_get_config(key: String) -> Result<String, String> {
    db_get_config(key)
}

#[tauri::command]
pub fn cmd_db_set_config(key: String, value: String) -> Result<(), String> {
    db_set_config(key, value)
}

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

#[tauri::command]
pub fn cmd_db_list_workflows() -> Result<Vec<Workflow>, String> {
    db_list_workflows()
}

#[tauri::command]
pub fn cmd_db_save_workflow(workflow: Workflow) -> Result<(), String> {
    db_save_workflow(workflow)
}

#[tauri::command]
pub fn cmd_db_delete_workflow(id: String) -> Result<(), String> {
    db_delete_workflow(id)
}

#[tauri::command]
pub fn cmd_db_list_variables() -> Result<Vec<PoolVariable>, String> {
    db_list_variables()
}

#[tauri::command]
pub fn cmd_db_set_variable(name: String, value: String, source: String) -> Result<(), String> {
    db_set_variable(name, value, source)
}

#[tauri::command]
pub fn cmd_db_delete_variable(name: String) -> Result<(), String> {
    db_delete_variable(name)
}

#[tauri::command]
pub fn cmd_db_get_variable(name: String) -> Result<String, String> {
    db_get_variable(name)
}

#[tauri::command]
pub fn cmd_db_export_all() -> Result<String, String> {
    db_export_all()
}

#[tauri::command]
pub fn cmd_db_import_all(data: String) -> Result<(), String> {
    db_import_all(data)
}

#[tauri::command]
pub fn cmd_db_check_migrated() -> Result<bool, String> {
    db_check_migrated()
}

#[tauri::command]
pub fn cmd_db_migrate_from_localstorage(data: String) -> Result<(), String> {
    db_migrate_from_localstorage(data)
}

// ========== 代码片段 Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_list_snippets() -> Result<Vec<Snippet>, String> {
    db_list_snippets()
}

#[tauri::command]
pub fn cmd_db_save_snippet(snippet: Snippet) -> Result<(), String> {
    db_save_snippet(snippet)
}

#[tauri::command]
pub fn cmd_db_delete_snippet(id: String) -> Result<(), String> {
    db_delete_snippet(id)
}

// ========== 最近工具 Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_list_recent_tools(limit: i64) -> Result<Vec<String>, String> {
    db_list_recent_tools(limit)
}

#[tauri::command]
pub fn cmd_db_add_recent_tool(tool_id: String) -> Result<(), String> {
    db_add_recent_tool(tool_id)
}

// ========== OCR 历史 Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_list_ocr_history(limit: i64) -> Result<Vec<OcrHistoryRecord>, String> {
    db_list_ocr_history(limit)
}

#[tauri::command]
pub fn cmd_db_add_ocr_history(thumbnail: String, original_url: String, text: String, time: String) -> Result<(), String> {
    db_add_ocr_history(OcrHistoryRecord { id: None, thumbnail, original_url, text, time })
}

#[tauri::command]
pub fn cmd_db_clear_ocr_history() -> Result<(), String> {
    db_clear_ocr_history()
}

// ========== 剪贴板历史 Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_list_clipboard_history(limit: i64, offset: i64) -> Result<Vec<ClipboardRecord>, String> {
    db_list_clipboard_history(limit, offset)
}

#[tauri::command]
pub fn cmd_db_search_clipboard_history(query: String, limit: i64) -> Result<Vec<ClipboardRecord>, String> {
    db_search_clipboard_history(query, limit)
}

#[tauri::command]
pub fn cmd_db_add_clipboard_record(text: String) -> Result<(), String> {
    db_add_clipboard_record(text)
}

#[tauri::command]
pub fn cmd_db_delete_clipboard_record(id: i64) -> Result<(), String> {
    db_delete_clipboard_record(id)
}

#[tauri::command]
pub fn cmd_db_clear_clipboard_history() -> Result<(), String> {
    db_clear_clipboard_history()
}

// ========== HTTP 环境 CRUD ==========

pub fn db_list_http_environments() -> Result<Vec<HttpEnvironment>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, base_url, variables_json, created_at, updated_at FROM http_environments ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HttpEnvironment {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_url: row.get(2)?,
                    variables_json: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}

pub fn db_save_http_environment(env: HttpEnvironment) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO http_environments (id, name, base_url, variables_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 name = ?2, base_url = ?3, variables_json = ?4, updated_at = ?6",
            params![env.id, env.name, env.base_url, env.variables_json, env.created_at, env.updated_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_http_environment(id: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM http_environments WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== HTTP 历史 CRUD ==========

pub fn db_list_http_history(limit: i64) -> Result<Vec<HttpHistoryRecord>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, method, url, headers_json, body, body_type, env_name, status, created_at
                      FROM http_history ORDER BY id DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(HttpHistoryRecord {
                    id: Some(row.get(0)?),
                    method: row.get(1)?,
                    url: row.get(2)?,
                    headers_json: row.get(3)?,
                    body: row.get(4)?,
                    body_type: row.get(5)?,
                    env_name: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}

pub fn db_add_http_history(record: HttpHistoryRecord) -> Result<i64, String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO http_history (method, url, headers_json, body, body_type, env_name, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![record.method, record.url, record.headers_json, record.body, record.body_type, record.env_name, record.status],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    })
}

pub fn db_clear_http_history() -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM http_history", []).map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== HTTP 收藏 CRUD ==========

pub fn db_list_http_bookmarks() -> Result<Vec<HttpBookmark>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, method, url, headers_json, body, body_type, created_at, updated_at
                      FROM http_bookmarks ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HttpBookmark {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    method: row.get(2)?,
                    url: row.get(3)?,
                    headers_json: row.get(4)?,
                    body: row.get(5)?,
                    body_type: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}

pub fn db_save_http_bookmark(bookmark: HttpBookmark) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO http_bookmarks (id, name, method, url, headers_json, body, body_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 name = ?2, method = ?3, url = ?4, headers_json = ?5, body = ?6, body_type = ?7, updated_at = ?9",
            params![
                bookmark.id, bookmark.name, bookmark.method, bookmark.url,
                bookmark.headers_json, bookmark.body, bookmark.body_type,
                bookmark.created_at, bookmark.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn db_delete_http_bookmark(id: String) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM http_bookmarks WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

// ========== HTTP Tauri 命令 ==========

#[tauri::command]
pub fn cmd_db_list_http_environments() -> Result<Vec<HttpEnvironment>, String> {
    db_list_http_environments()
}

#[tauri::command]
pub fn cmd_db_save_http_environment(env: HttpEnvironment) -> Result<(), String> {
    db_save_http_environment(env)
}

#[tauri::command]
pub fn cmd_db_delete_http_environment(id: String) -> Result<(), String> {
    db_delete_http_environment(id)
}

#[tauri::command]
pub fn cmd_db_list_http_history(limit: i64) -> Result<Vec<HttpHistoryRecord>, String> {
    db_list_http_history(limit)
}

#[tauri::command]
pub fn cmd_db_add_http_history(record: HttpHistoryRecord) -> Result<i64, String> {
    db_add_http_history(record)
}

#[tauri::command]
pub fn cmd_db_clear_http_history() -> Result<(), String> {
    db_clear_http_history()
}

#[tauri::command]
pub fn cmd_db_list_http_bookmarks() -> Result<Vec<HttpBookmark>, String> {
    db_list_http_bookmarks()
}

#[tauri::command]
pub fn cmd_db_save_http_bookmark(bookmark: HttpBookmark) -> Result<(), String> {
    db_save_http_bookmark(bookmark)
}

#[tauri::command]
pub fn cmd_db_delete_http_bookmark(id: String) -> Result<(), String> {
    db_delete_http_bookmark(id)
}

// 读取快捷键配置
pub fn db_read_shortcuts() -> Vec<(String, String)> {
    let config = db_get_config("shortcuts".to_string()).unwrap_or_default();
    if config.is_empty() {
        return vec![
            ("json".to_string(), "CmdOrCtrl+Alt+J".to_string()),
            ("string".to_string(), "CmdOrCtrl+Alt+S".to_string()),
            ("encode".to_string(), "CmdOrCtrl+Alt+E".to_string()),
            ("regex".to_string(), "CmdOrCtrl+Alt+R".to_string()),
            ("http".to_string(), "CmdOrCtrl+Alt+H".to_string()),
        ];
    }
    serde_json::from_str(&config).unwrap_or_default()
}

#[tauri::command]
pub fn cmd_db_register_shortcuts(shortcuts_json: String) -> Result<(), String> {
    db_set_config("shortcuts".to_string(), shortcuts_json)
}

// ========== 历史详情 Tauri 命令 ==========

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
