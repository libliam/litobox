# SDD Progress Ledger

Plan: docs/superpowers/plans/2026-07-07-file-searcher-plan.md
Spec: docs/superpowers/specs/2026-07-07-file-searcher-design.md
Base commit: 39c316a

## Task Progress

| Task | Status | Commit(s) | Notes |
|------|--------|-----------|-------|
| 1: Cargo.toml regex + version | ✅ | 07612ac | 4.4.0→4.6.0, +regex |
| 2: read_file_auto TDD | ✅ | 07612ac | GBK/UTF-16/UTF-8 BOM |
| 3: file_searcher structs | ✅ | dd25dd4 | 数据结构 + 常量 |
| 4: 4 pure functions TDD | ✅ | dd25dd4 | parse_ext/is_binary/build_regex/scan_content |
| 5+6: run_search + 6 cmds | ✅ | 504b7bd + e2956dc | fix: 嵌套锁死锁 |
| 7: main.rs register cmds | ✅ | d8e49e6 | 6 commands registered |
| 8+9+10: frontend | ✅ | 6d2a9ff + cf71683 | fix: XSS + cancel/failed event emit |
| 11: version sync + docs | ✅ | 0d72c9a | package.json 4.6.0 + README + backlog |
| 12: final review | ✅ | 07a5a83 | searchId 包装 complete 事件防陈旧竞态；finding #4 顺带修复 |

## Minor Findings (for final review triage)

1. **日志标签误导**（file_searcher.rs run_search 末尾）：`debug_log!("file_searcher: 搜索完成 id={}", root_path)` 用 `id=` 标签打印了 `root_path`（路径）。应改为 `path={}` 或打印 `search_id`。不影响功能。
2. **废弃搜索资源驻留**：若前端未调用 `file_search_clear`（如页面刷新），SEARCHES HashMap 条目永久驻留。与 disk_analyzer 模式一致，记录备查。
3. **file_search_clear 静默忽略不存在条目**：返回 `Ok(())` 即使条目不存在。与 brief 一致。
4. ~~**fileSearchStatus invoke<any>**~~：✅ 已在 07a5a83 中改为 `invoke<{ status: string; error?: string }>`，补类型安全。
5. **startTime 用 ref 但无模板依赖**：`startTime` 不需要响应式，可改为普通变量。风格问题。

## Commits on dev branch (39c316a..HEAD)

- 07612ac feat(file-searcher): 新增 read_file_auto 自动解码函数 + regex 依赖
- dd25dd4 feat(file-searcher): 数据结构 + 4 个纯函数
- 504b7bd feat(file-searcher): 全局状态 + run_search + 6 个 Tauri 命令
- e2956dc fix(file-searcher): 修复 run_search 取消检查中的嵌套锁死锁
- d8e49e6 feat(file-searcher): main.rs 注册 6 个搜索命令
- 6d2a9ff feat(file-searcher): 前端 types + client + FileSearcher.vue 页面 + 注册
- cf71683 fix(file-searcher): 修复 highlightLine XSS + cancel/failed 路径未发 complete 事件
- 0d72c9a docs: V4.6 版本号同步 + README + backlog 更新
- 07a5a83 fix(file-searcher): complete 事件加 searchId 防陈旧事件竞态
