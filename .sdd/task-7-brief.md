# Task 7: 导入导出适配

**Files:**
- Modify: `src-tauri/src/db.rs`

## Steps

### Step 1: 修改 `db_export_all` — 导出时 JOIN details

In `db_export_all`, replace the history export SQL query (around line 649) with:

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

### Step 2: 修改 `db_import_all` — 导入时恢复 details

Replace the history import section in `db_import_all` with:

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

### Step 3: 编译验证

Run: `cd src-tauri && cargo check`
Expected: 编译通过
