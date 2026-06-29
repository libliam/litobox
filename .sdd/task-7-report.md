# Task 7 Report: 导入导出适配

## What was implemented

Modified `src-tauri/src/db.rs` to include `history_details` data in export/import:

1. **`db_export_all`** (line 728-751): Changed the history export SQL from a simple SELECT to a LEFT JOIN with `history_details` on `h.detail_id = d.id`. The exported JSON now includes `input_full`, `output_full`, and `options_json` fields (as `Option<String>` to handle NULL from the LEFT JOIN).

2. **`db_import_all`** (line 844-878): Changed the history import to:
   - Capture the `history_id` from the INSERT
   - If `input_full` and `output_full` are present in the record, insert a `history_details` row
   - UPDATE the history row to set `detail_id` linking back to the newly created detail

## What was tested

- `cargo check` passed successfully (Finished `dev` profile in 3.71s)
- No compilation errors or warnings

## Files changed

- `d:\work\litobox\src-tauri\src\db.rs` (+31 lines, -4 lines)

## Self-review findings

- The export uses `Option<String>` for the detail fields, which correctly handles records without details (LEFT JOIN produces NULL)
- The import only creates a detail record when both `input_full` and `output_full` are present, which is a reasonable heuristic
- The `history_id as i64` cast is correct since `conn.execute` returns `usize` and SQLite rowids fit in i64
- Backward compatibility: old exports without detail fields will still import correctly (the `and_then` chain will return None and skip detail creation)

## Issues or concerns

- None identified. The implementation matches the task brief exactly.
