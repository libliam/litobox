# SDD Progress Ledger

Plan: docs/superpowers/plans/2026-07-20-hotkey-viewer.md
Spec: docs/superpowers/specs/2026-07-20-hotkey-viewer-design.md
Base commit: bcfb453

## Pre-Flight Plan Review 修正

- 修复 hotkey_data.rs SYSTEM_RESERVED 表 MOD_MENU 未定义（→ MOD_ALT）
- 移除"安全选项增强"矛盾条目（注释写"删除"但条目仍在）
- 修正虚拟桌面 VK 码（0x24→0x44 'D', 0x23→0x73 VK_F4, 0xD→0x25/0x27 Left/Right）
- 移除 hotkey_probe.rs 未使用的 ProcessStatus import

## Task Progress

| Task | Status | Commit(s) | Notes |
|------|--------|-----------|-------|
| 1: 添加 windows-sys 依赖 | ✅ | f376b8c | review clean; 1 file +9 lines; Minor: Cargo.lock 未提交（项目 .gitignore 策略） |
| 2: 创建 hotkey_data.rs | ⏳ | — | — |
| 3: 创建 hotkey_probe.rs | ⏳ | — | — |
| 4: main.rs 注册 | ⏳ | — | — |
| 5: 创建 HotkeyView.vue | ⏳ | — | — |
| 6: 注册到 TOOL_LIST 和 App.vue | ⏳ | — | — |
| 7: 版本号 + README | ⏳ | — | — |
| 8: 整体验证 | ⏳ | — | — |

## Minor Findings (for final review triage)

- Task 1: Cargo.lock 被 .gitignore 忽略未提交，符合项目策略；Tauri 二进制应用业内通常建议提交 Cargo.lock，但项目级决策不阻塞本任务 → defer


### Task 4 Review Minor Findings (defer to final review)
- Minor #1: extra_keys 参数在 run_probe 内未使用（已折叠进 candidates）— 删除该参数即可
- Minor #2: CSV 路径字符串构造冗余（两次 to_string_lossy）— 用 trim_start_matches 统一
- Minor #3: test_generate_default_candidates_count 断言过松（>=190 应改为 ==198）
- Minor #4: 错误路径持锁调用 app.emit（应释放锁后再 emit）
- Minor #5: ProbeCompletePayload 缺少 error 字段（超时无法通过事件感知，需轮询）
- Minor #6: 文件末尾 use 声明位置怪异（应放进 mod tests）
- Minor #7: test_parse_accelerator_invalid 覆盖不全（缺 "+", "Ctrl+Shift+", "Foo" 等边界）
- Dismissed: Important #1 rename_all 是误报（file_searcher.rs 约定无 rename_all，前端用 camelCase）
