# LitoBox 开发待办

> 后续主要开发方向。按批次依次推进，每批完成后更新状态。

## 批次 0：已有工具补注册菜单（快速收尾） ✅ 已完成

经核查，`entityGen`/`folderDiff`/`totp` 三个工具此前已注册菜单；实际缺失的是以下 2 个，已补注册：

- [x] **Hex 十六进制查看器** — `HexViewerTool.vue`，category: dev
- [x] **文件编码转换** — `FileEncodingTool.vue`，category: utility

修改内容：`src/App.vue` 添加组件 import 与路由映射；`src/store/index.ts` 添加 `TOOL_LIST` 条目。类型检查通过。

## 批次 1：纯前端工具（快速上线） ✅ 已完成

无需后端命令，前端纯逻辑即可完成。

- [x] **文本排序工具** — 在 StringTool 批量处理 Tab 内新增「行排序」操作组。新增：升序、降序、忽略大小写、数字排序、自然排序、随机打乱、去重排序、按列排序（自定义分隔符/列索引/降序）。底层函数在 `src/utils/stringUtils.ts`，UI 在 `src/views/StringTool.vue`。
- [x] **表情符号面板** — 独立工具页 `EmojiPanelTool.vue`，8 大分类（笑脸/动物/食物/活动/旅行/物品/符号/旗帜）、搜索过滤、最近使用记录、点击复制。emoji 数据在 `src/utils/emojiData.ts`。
  - 增强：扩展至 26 分类（新增植物、颜文字、emoji 组合、数学、几何、货币、星座、棋类、技术、标点等），emoji 名称映射 `emojiNames.ts`（400+）。hover 显示名称用纯 CSS `:hover::after` 实现（零组件开销，避免 el-tooltip 导致分类切换卡顿）。搜索同时匹配 emoji 字符和名称。
- [x] **字体预览工具** — 独立工具页 `FontPreviewTool.vue`，内置 100+ 常用字体列表，前端检测字体可用性（渲染宽度对比法），自定义预览文本/字号/粗细，搜索过滤，点击复制字体名。

修改内容：新增 3 个文件（emojiData.ts、EmojiPanelTool.vue、FontPreviewTool.vue），修改 stringUtils.ts、StringTool.vue、App.vue、store/index.ts。`vue-tsc --noEmit` 类型检查通过。

## 批次 2：系统命令封装工具

需要后端调用 Windows 系统命令 / 操作注册表。

- [ ] **Ping / Traceroute** — 可视化 ICMP Ping（延迟、丢包率、TTL）与 Traceroute（路由跳数、每跳延迟）。后端封装 `ping` / `tracert` 命令，实时输出进度。category: system。
- [ ] **右键菜单管理** — 管理 Windows 右键菜单：文件/文件夹/桌面右键的"新建"、"发送到"、自定义项的增删启用。通过注册表操作，自动备份。category: system。

## 批次 3：需要新依赖的工具

- [ ] **Redis 客户端** — 本地 Redis 连接管理（地址/端口/密码/DB），支持 Key 浏览（按类型图标）、GET/SET/DEL、TTL 查看、执行自定义命令。需引入 Rust `redis` crate。category: dev。

---

## 备注

- 文本排序工具：StringTool 已有基础字典序排序，本项为增强，避免重复造轮子。
- 每个新工具需遵循 `AGENTS.md` 规范：`_ToolTemplate.vue` 模板、`fill-height`、CSS 变量、历史记录传 `inputFull`/`outputFull`、下载走 `saveFileWithDialog`。
- 版本号更新时四文件同步（package.json / Cargo.toml / tauri.conf.json / README.md）。
