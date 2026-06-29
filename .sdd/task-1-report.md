# Task 1 Report: 数据库层 — history_details 表 + CRUD（Rust）

## What was implemented

1. **Migration in `init_tables`**: Added `history_details` table creation and `detail_id` column migration for the `history` table.

2. **`HistoryDetail` struct**: New serializable struct with fields: `id`, `history_id`, `input_full`, `output_full`, `options_json`, `created_at`.

3. **CRUD functions**:
   - `db_add_history_detail(detail: HistoryDetail) -> Result<i64, String>` - inserts a new detail record and returns the row ID.
   - `db_get_history_detail(history_id: i64) -> Result<Option<HistoryDetail>, String>` - retrieves a detail by history_id.
   - `db_delete_history_details_for_history(history_id: i64) -> Result<(), String>` - deletes all details for a given history_id.

4. **Tauri command wrappers**:
   - `cmd_db_add_history_detail`
   - `cmd_db_get_history_detail`
   - `cmd_db_delete_history_details_for_history`

5. **Registered new commands** in `main.rs` invoke handler.

## Testing

- `cargo check` passed with no errors or warnings.
- Compilation time: ~21.78s.

## Files changed

- `src-tauri/src/db.rs` - Added HistoryDetail struct, migration SQL, 3 CRUD functions, 3 Tauri commands.
- `src-tauri/src/main.rs` - Registered 3 new Tauri commands in the invoke handler.

## Self-review findings

- All code follows existing patterns: `with_conn()` helper, `params![]` macro, `.map_err(|e| e.to_string())?` error handling.
- The `HistoryDetail` struct is placed immediately after `HistoryRecord` as specified.
- The migration uses `execute_batch` for table creation (with `.ok()` to ignore if exists) and `ALTER TABLE` with error suppression for the column addition, matching existing migration patterns.
- The `history_details` table has a foreign key with `ON DELETE CASCADE` to automatically clean up details when history records are deleted.
- An index on `history_id` is created for efficient lookups.

## Issues or concerns

- None. Implementation matches the task brief exactly.
