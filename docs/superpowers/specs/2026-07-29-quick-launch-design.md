# 快速启动 - 设计文档

## 1. 功能概述

全盘文件名快速搜索工具，基于 SQLite FTS5 持久化文件名索引，实现类似 Everything 的即时搜索体验。搜索到文件后回车一键打开。

与旧有全文搜索（FileSearcher）的区别：全文搜索是按指定目录搜文件名+文件内容，定位文件位置；快速启动是全盘搜文件名，找到就打开。

## 2. 核心功能

| 功能 | 说明 |
|------|------|
| 全盘文件名搜索 | 基于 FTS5 持久化索引，输入即搜，毫秒级返回 |
| 一键打开 | 选中结果回车/双击，调用 `open::that()` 打开文件/文件夹 |
| 持久化索引 | 首次全盘扫描后索引存 SQLite，重启不用重扫 |
| 增量更新 | 启动时检查修改时间，增量刷新索引 |
| 全局浮层 | 快捷键呼出浮层，在当前应用上层快速搜索打开 |
| 重建索引 | 手动触发全盘重新扫描 |

## 3. 后端设计

### 3.1 新增文件

`src-tauri/src/quick_launch.rs` — 索引构建、搜索、打开、增量更新

### 3.2 SQLite 表结构

在 `db.rs` 中新增三张表（在 `init_db` 中创建）：

```sql
-- 文件元数据表
CREATE TABLE IF NOT EXISTS quick_launch_files(
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL,
  path        TEXT NOT NULL UNIQUE,
  extension   TEXT DEFAULT '',
  size_bytes  INTEGER DEFAULT 0,
  modified_at INTEGER DEFAULT 0,
  drive       TEXT NOT NULL
);

-- FTS5 全文搜索索引（仅索引文件名）
-- unicode61 分词器支持中文和 Unicode 字符
CREATE VIRTUAL TABLE IF NOT EXISTS quick_launch_fts USING fts5(
  name,
  content='quick_launch_files',
  content_rowid='id',
  tokenize='unicode61'
);

-- 索引元数据（按驱动器记录扫描状态）
CREATE TABLE IF NOT EXISTS quick_launch_meta(
  drive        TEXT PRIMARY KEY,
  last_scanned INTEGER NOT NULL DEFAULT 0,
  file_count   INTEGER NOT NULL DEFAULT 0,
  status       TEXT NOT NULL DEFAULT 'pending'
);
```

### 3.3 Tauri 命令

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `ql_search` | `query: string` | `QuickLaunchResult[]` | 搜索文件名，FTS5 前缀匹配 |
| `ql_build_index` | 无 | `string` (search_id) | 开始全盘扫描建索引，异步事件通知进度 |
| `ql_index_status` | 无 | `IndexStatus` | 查询各驱动器索引状态 |
| `ql_rebuild_index` | 无 | `string` (search_id) | 清空全表重新扫描 |
| `ql_open_file` | `path: string` | `()` | 调用 `open::that()` 打开文件/目录 |

**前端类型定义：**

```typescript
interface QuickLaunchResult {
  id: number
  name: string
  path: string
  extension: string
  sizeBytes: number
  modifiedAt: number
  drive: string
}

interface IndexStatus {
  drives: DriveIndexInfo[]
  builtAt: number | null   // 最近一次全盘索引时间戳
  isBuilding: boolean       // 是否正在构建
}

interface DriveIndexInfo {
  drive: string
  lastScanned: number
  fileCount: number
  status: 'pending' | 'indexing' | 'ready' | 'failed'
}
```

### 3.4 搜索算法

- 前端 300ms 防抖后调用 `ql_search`
- 后端将用户输入按字符拆分，每个非空字符追加 `*` 后以空格拼接（FTS5 隐式 AND）
  - 例：`excel` → `excel*`（ASCII 前缀匹配）
  - 例：`my excel` → `my* excel*`（多词 AND 匹配）
  - 例：`笔记` → `笔* 记*`（中文逐字 AND 匹配，unicode61 将每个汉字视为独立 token）
  - 例：`财务报告` → `财* 务* 报* 告*`（多字 AND 匹配）
- 搜索结果排序：按修改时间倒序（最近修改的文件优先展示）
- 返回 Top 100 结果

### 3.5 索引构建

**首次构建流程：**
1. `ql_build_index` 被调用
2. 调用 `GetLogicalDrives` 获取所有驱动器，筛选 `DRIVE_FIXED`
3. 排除 `C:\Windows`、`C:\Program Files\WindowsApps` 目录
4. 跳过隐藏文件和系统文件属性目录
5. 启动后台线程，walkdir 逐文件遍历
6. 插入数据使用事务分批写入（每 500 条提交一次事务）
7. 每遍历 1000 个文件或每 200ms 触发一次进度事件
8. 遍历完一个驱动器后更新 `quick_launch_meta`
9. 全部遍历完成后执行 `INSERT INTO quick_launch_fts(quick_launch_fts) VALUES('rebuild')` 同步 FTS5 外部内容索引

**进度事件 (`ql-index-progress`)：**

```typescript
interface QLIndexProgress {
  searchId: string
  filesScanned: number
  currentDrive: string
  currentPath: string
  status: 'indexing' | 'completed' | 'failed'
  error?: string
}
```

### 3.6 增量更新

- 每次 `ql_search` 调用时检查 `quick_launch_meta` 中状态
- 如果有驱动器 `status = 'pending'` 或距离上次扫描超过 24h，触发后台增量扫描
- 增量扫描：遍历驱动器，对比 `modified_at`，有变化的文件 upsert，已删除的文件移除
- 增量扫描同样通过进度事件通知前端

### 3.7 打开文件

`ql_open_file` 调用 `open::that()`（已在项目中依赖），打开文件或目录。

## 4. 前端设计

### 4.1 新增文件

| 文件 | 说明 |
|------|------|
| `src/views/QuickLaunchTool.vue` | 快速启动工具页面 |
| `src/components/QuickLaunchOverlay.vue` | 全局浮层组件 |
| `src/utils/quickLaunchClient.ts` | 后端命令封装 |

### 4.2 工具页面布局

```
┌─ 快速启动 ───────────────────────────────────────┐
│  [🔍 输入文件名…              ] [重建索引]        │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │ 📄 report-excel-2024.xlsx                   │  │
│  │    D:\work\reports\                         │  │
│  │    修改: 2024-12-20 14:30   15 KB           │  │
│  ├─────────────────────────────────────────────┤  │
│  │ 📄 my_document.docx                         │  │
│  │    C:\Users\me\Documents\                   │  │
│  │    修改: 2024-12-19 09:15   45 KB           │  │
│  └─────────────────────────────────────────────┘  │
│  共 12 条结果         索引: C:✓ D:✓              │
└───────────────────────────────────────────────────┘
```

- 输入框：大字输入，placeholder "搜索文件名…"
- 结果列表：每项显示**文件图标（按扩展名）+ 文件名 + 路径 + 修改时间 + 大小**
- 快捷键：方向键 ↑↓ 选择，Enter 打开
- 索引状态：在底部显示各驱动器索引状态（✓ 已就绪 / ⟳ 索引中 / ⚠ 待建）

### 4.3 全局浮层

- 全局快捷键 `Alt+Space` 呼出浮层（在 `main.rs` 注册，类似命令面板）
- 浮层 UI 类似命令面板，在应用上方覆盖显示
- 焦点自动进入输入框
- 结果列表同工具页面
- Enter 打开文件后自动关闭浮层
- Escape 关闭浮层

### 4.4 增量更新状态交互

- 工具页面首次加载时，自动调用 `ql_index_status`
- 如果有驱动器 `status = 'pending'`，显示提示条："首次使用需要建立索引，点击开始"
- 索引过程中显示进度条（文件数 + 当前路径）
- 索引完成后自动开始搜索

### 4.5 TOOL_LIST 注册

```typescript
{
  id: 'quickLaunch',
  name: '快速启动',
  icon: '⚡',
  iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>`,
  description: '全盘文件名快速搜索，一键打开',
  keywords: ['快速启动', '文件搜索', 'Everything', '启动', '搜索文件', '打开'],
  category: 'utility'
}
```

## 5. 工作流集成

在 `WorkflowView.vue` 的 `executeStep()` 中添加 `quickLaunch` 分支：
- 输入源：执行输入（搜索词）
- 输出：搜索结果列表（JSON 格式，包含文件名和路径）
- 结果可保存到变量池

## 6. 变量池集成

- 输入区添加 `VariablePicker` 组件
- 搜索词支持从变量池插入

## 7. 历史记录

- 每次搜索操作记录到 SQLite 历史
- `inputFull`：搜索词
- `outputFull`：搜索结果条数 + 搜索结果摘要

## 8. 排除考虑

- 不扫描网络盘/移动盘/光驱（仅 `DRIVE_FIXED`）
- 排除 `C:\Windows` 目录（太大且对日常打开没帮助）
- 不支持内容搜索、正则搜索（那是 FileSearcher 的职责）
- 不支持 Everything 的 USN Journal 直接读取（需要 `everything-sdk`，增加外部依赖）
- 不提供搜索结果的文件预览（定位就是快速打开）
