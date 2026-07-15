# SDD Progress Ledger

Plan: docs/superpowers/plans/2026-07-15-video-tool.md
Spec: docs/superpowers/specs/2026-07-15-video-tool-design.md
Base commit: b9cfba0

## Task Progress

| Task | Status | Commit(s) | Notes |
|------|--------|-----------|-------|
| 1: 添加 mp4 依赖 | ✅ | 709fe3f | review clean; 1 line added |
| 2: 创建 video_tools.rs | ✅ | 6110dc0 | 559 行，mp4 0.14 API 适配（9 处差异），关键帧裁剪简化策略 |
| 3: 注册 Tauri commands | ✅ | 5c69f27 | 3 commands 注册; cargo check 通过 |
| 4: 创建 VideoTool.vue | ✅ | 8986d70 | 693 行; npm run build 通过; vue-tsc 0 错误 |
| 5: 注册到 App.vue 和 TOOL_LIST | ✅ | be2c08c | 3 insertions; npm run build 通过 |
| 6: 版本号 + README | ✅ | a5fc9cc | 5.1.0→5.2.0; README 功能表 + 版本历史更新 |
| 7: 端到端验证 | ⏳ | — | 需手动运行 npm run tauri dev 测试 |

## Minor Notes
- Task 2: 关键帧裁剪采用"头部复制 + mdat 裁剪"简化策略，不重写 moov 元数据，播放器容错播放可用。如需精确裁剪建议使用 ffmpeg 路径。

---

Plan: docs/superpowers/plans/2026-07-09-background-collect.md
Spec: docs/superpowers/specs/2026-07-09-background-collect-design.md
Base commit: b7d4a4e

## Task Progress

| Task | Status | Commit(s) | Notes |
|------|--------|-----------|-------|
| 1: 后端采集类型与状态表 | ✅ | da542ea | review clean; +Debug derive, tauri::async_runtime::spawn_blocking (brief 笔误修正) |
| 2: 5 采集命令改后台 + 注册 | ✅ | 1475d68 + 2affe5d | fix: CollectKind +serde rename_all=lowercase + 序列化守护测试; Minor: 测试清空全局态有理论竞态(待 final triage) |
| 3: 前端 systemInfoClient 封装 | ✅ | 5553d0f | review clean; kill 函数完整保留 |
| 4: store 采集状态 | ✅ | 25107cf | review clean |
| 5: useBackgroundCollect composable | ✅ | 2826902 | review clean; verbatim from brief |
| 6: 5 采集页改造 | ✅ | db33c64 | review clean; ElEmpty 未 import（全局注册）, 0 ts 错误 |
| 7: 版本号 + README | ✅ | 45721fc | 自验通过; 4.7.0 minor bump（偏离 plan 补丁位措辞，尊重 README 约定）; tauri.conf.json 滞后顺带修复 |
| Final: 整支分支审查 | ✅ | origin/dev..45721fc | APPROVED; 0 Critical/0 Important/3 Minor(全 defer); vue-tsc 0 错误, cargo test 24 通过(含 2 新测试) |

## Minor Findings (for final review triage) — final reviewer 已 triage，均为 defer

- Task 2: `get_collect_status_returns_none_when_empty` 测试清空全局 `COLLECT_STATE`，单线程测试无干扰，若未来并行测试需重新评估 → defer
- Task 5 review-package diff 文件中文乱码（PowerShell Out-File 编码问题），源码 UTF-8 正常，不影响功能 → defer
- 跨切面: post-kill 刷新被 `collect()` 重入拦截跳过（kill 时若已有采集在跑，刷新被忽略，用户可手动点刷新）→ defer，可接受权衡

Plan: docs/superpowers/plans/2026-07-13-audio-crop.md
Spec: docs/superpowers/specs/2026-07-13-audio-crop-design.md
Base commit: 988cdc5

## Task Progress

| Task | Status | Commit(s) | Notes |
|------|--------|-----------|-------|
| 1: ���� Rust ���� | ? | 12c5a84 | review clean; mp3lame-encoder 0.5��0.2.4 (API ������֪��Task 3 ����) |
