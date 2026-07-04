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
            // 天花板：hex 截断到前 100 字符（约 50 字节），大 BLOB 仅显示前缀；
            //         且 hex 后体积翻倍，MB 级 BLOB 会显著放大 JSON 体积
            // 升级路径：如需完整 BLOB 查看或下载，应改为流式导出或单独的 BLOB 端点
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            serde_json::json!(format!("BLOB:{}bytes:{}", bytes.len(), hex.chars().take(100).collect::<String>()))
        }
    }
}

/// 检查 SQL 是否为 SELECT 语句（只允许查询）
fn is_select_sql(sql: &str) -> bool {
    let trimmed = sql.trim();
    // ponytail: 简单前缀检查，覆盖 SELECT / WITH (CTE)；SQLite 不支持 EXPLAIN 外的其他读操作入口
    // 升级路径：如需更严格的语句类型校验（防止注释绕过、多语句注入），应改用 sqlparser-rs 解析 AST
    let upper = trimmed.to_uppercase();
    upper.starts_with("SELECT") || upper.starts_with("WITH")
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
        .filter_map(|r| match r {
            Ok(name) => Some(name),
            Err(e) => {
                eprintln!("[sqlite_viewer] 表名行读取失败: {}", e);
                None
            }
        })
        .collect();

    let mut tables = Vec::new();
    for name in table_names {
        // MAX(rowid) 在空表上返回 NULL，因此取 Option<i64>；两层 unwrap_or 即对应此情形
        let row_count: i64 = match conn.query_row(
            &format!("SELECT MAX(rowid) FROM \"{}\"", name.replace('"', "\"\"")),
            [],
            |row| row.get::<_, Option<i64>>(0),
        ) {
            Ok(Some(v)) => v,
            Ok(None) => 0,
            Err(e) => {
                eprintln!("[sqlite_viewer] 表 {} 行数查询失败: {}", name, e);
                0
            }
        };
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
        .filter_map(|r| match r {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("[sqlite_viewer] schema 行读取失败: {}", e);
                None
            }
        })
        .collect();
    Ok(columns)
}

/// 查询核心逻辑（不带 limit 钳制，由调用方决定上限）
/// 调用方负责：1) 校验 SQL 是否为 SELECT/WITH；2) 钳制 max_rows 到合理上限
fn execute_query_internal(conn: &Connection, sql: &str, max_rows: usize) -> Result<QueryResult, String> {
    let start = Instant::now();

    let mut stmt = conn.prepare(sql).map_err(|e| format!("SQL 错误: {}", e))?;
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
    // ponytail: 强制钳制到 1000 上限，防止前端传入大值导致内存爆炸
    // 升级路径：如需更大结果集，应走 sqlite_export_csv 流式导出而非 JSON 直返
    let max_rows = limit.unwrap_or(1000).min(1000);
    execute_query_internal(&conn, &sql, max_rows)
}

#[tauri::command]
pub fn sqlite_table_preview(db_path: String, table_name: String) -> Result<QueryResult, String> {
    let conn = open_db(&db_path)?;
    let sql = format!("SELECT * FROM \"{}\"", table_name.replace('"', "\"\""));
    execute_query_internal(&conn, &sql, 100)
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
    let conn = open_db(&db_path)?;
    // ponytail: 导出放宽到 100000 行，绕过 sqlite_query 的 1000 上限；
    // 天花板：仍一次性载入内存，超大数据集会撑爆；
    // 升级路径：如需 GB 级导出，应改为流式写入（stmt 迭代器直写文件，不积攒 rows）
    let result = execute_query_internal(&conn, &sql, 100000)?;

    // UTF-8 BOM，让 Excel 正确识别中文编码（默认按 GBK 解析会乱码）
    let mut csv_content = "\u{FEFF}".to_string();
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
