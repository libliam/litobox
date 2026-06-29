# Task 2: 注册新 Tauri 命令

**Files:**
- Modify: `src-tauri/src/main.rs`

## Steps

### Step 1: 在 invoke_handler 中注册新命令

在 `db::cmd_db_search_history,` 之后添加：

```rust
            db::cmd_db_add_history_detail,
            db::cmd_db_get_history_detail,
            db::cmd_db_delete_history_details_for_history,
```

### Step 2: 编译验证

Run: `cd src-tauri && cargo check`
Expected: 编译通过
