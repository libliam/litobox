# Task 1: 数据库层 — history_details 表 + CRUD（Rust）

**Files:**
- Modify: `src-tauri/src/db.rs`

## Steps

### Step 1: 在 `init_tables` 中新增 `history_details` 表和 `detail_id` 列迁移

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

### Step 2: 新增 `HistoryDetail` struct

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

### Step 3: 新增 `db_add_history_detail` 函数

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

### Step 4: 新增 `db_get_history_detail` 函数

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

### Step 5: 新增 `db_delete_history_details_for_history` 函数

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

### Step 6: 新增 Tauri 命令包装函数

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

### Step 7: 编译验证

Run: `cd src-tauri && cargo check`
Expected: 编译通过，无错误
