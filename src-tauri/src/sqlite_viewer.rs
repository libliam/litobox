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

// ============ Tauri 命令 ============

#[tauri::command]
pub fn sqlite_list_tables(db_path: String) -> Result<Vec<TableInfo>, String> {
    let conn = open_db(&db_path)?;
    // ponytail: 用 MAX(rowid) 估算行数，比 COUNT(*) 快，对调试足够
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
