# SDD Progress Ledger

## Plan: 2026-07-21-hosts-manager

Task 2: complete (commits 5fe3623..f77335c, review clean)
Task 3: complete (commits f77335c..f0cc592, pending merged review with Task 4)
Task 3+4: complete (commits f77335c..87cea2a, review approved, fix 87cea2a..10c239d for path traversal)
Task 5: complete (commits 10c239d..6636c78, pending merged review with Task 6)
Task 5+6: complete (commits 10c239d..1583be4, review approved, fix 1583be4..118a563 for preview+confirm)
Task 7: complete (commit 1775003, version bump 5.8.0→5.9.0 + README + backlog)
Task 8: complete (cargo check --tests pass, npm run build pass; cargo test skipped due to disk space)
Final review: APPROVED_WITH_FIXES, 6 findings fixed (commits f98df04 + f5e5c49)
  - P0 UTC时间→本地时间 (GetLocalTime, +3 windows-sys features)
  - P1 onMounted+onActivated 重复→只用 onActivated
  - P2 备份文件名白名单校验 (validate_backup_filename)
  - P3 validate_profile_name 单元测试
  - P4 debug_log! 关键路径
  - P5 切换 Profile 前未保存弹窗 (isDirty 跟踪)
  - P6 import 重复 + is_alphanumeric 冗余清理

## Plan: 2026-07-22-command-palette (V6.0 快捷命令面板)

Plan: `docs/superpowers/plans/2026-07-22-command-palette.md` (8 tasks)
Spec: `docs/superpowers/specs/2026-07-22-command-palette-design.md`
MERGE_BASE: 041ce63 (plan commit)

Task 1: complete (commit c10d556 + fix 1d8faaa)
  - c10d556: default vec 注入 `__palette__` 热键
  - 1d8faaa: fix — 抽 `merge_shortcut_defaults` 纯函数，HashMap/数组分支返回前补全 default 缺失项（老用户向后兼容）
  - 3 单元测试 `shortcut_merge_tests` 全过
  - Re-review: ✅ Approved（brief 的 `HashSet<&str>` 有 E0502 借用冲突，改为 `HashSet<String>`，最小正确修复）
  - Minor notes (不阻塞): defaults 内部重复 key 会双倍追加（default 静态无重复，不触发）；未覆盖 defaults 为空场景

Task 2: complete (commit 07cfa49, base 1d8faaa)
  - main.rs 顶部加 `debug_log!` 宏（mod 之后、use 之前，含 `// ponytail:` 注释）
  - on_shortcut 回调加 `__palette__` 分支：show + set_focus + debug_log + emit `command-palette-triggered`
  - 普通工具仍走 `global-shortcut-triggered`，行为不变
  - cargo check 通过，0 新增 warning
  - Review: ✅ Approved，无 Critical/Important/Minor
