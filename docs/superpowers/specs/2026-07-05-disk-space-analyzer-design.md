# 磁盘空间分析工具设计文档

## 概述

为栗的百宝箱（LitoBox）新增一个磁盘空间分析工具，帮助用户回答两个高频问题：「我的空间都被什么占了？」和「有哪些重复文件可以清理？」。一次扫描，四个视角（文件夹大小 / 大文件 Top N / 按类型统计 / 重复文件检测）共同展示，并支持从工具内勾选文件送入回收站。

## 需求背景

- **场景**：磁盘空间不足时定位元凶；清理重复文件释放空间
- **核心功能**：
  1. 扫描指定路径，递归计算每个子文件夹大小，支持点击下钻
  2. 列出 Top N 大文件（默认 100，可加载更多）
  3. 按文件扩展名分组统计总大小与数量
  4. 检测重复文件（按大小分组 → 前 64KB SHA-256 指纹比对），展示可回收空间
  5. 勾选文件后入回收站删除（不删文件夹）
- **约束**：纯本地离线；扫描异步执行带进度反馈；删除前二次确认；不依赖网络

## 架构设计

### 文件结构

```
src-tauri/src/
  └── disk_analyzer.rs          # 新增：磁盘扫描与查询命令
src/views/
  └── DiskSpaceAnalyzer.vue     # 新增：分析页面（4 Tab）
src/utils/
  └── diskAnalyzerTypes.ts      # 新增：前端类型定义
```

### 扫描执行模型

异步 + 事件流，避免单次 IPC 传 100MB+ 数据：

1. 前端调 `disk_scan_start(path, opts)` → 命令立即返回 `scan_id`（UUID）
2. 后端 `tokio::spawn` 后台任务，递归 walk 目录
3. 任务每 500ms 通过 `app.emit("disk-scan-progress", { scan_id, files_scanned, bytes_scanned, current_path })` 推送进度
4. 完成时 `app.emit("disk-scan-complete", { scan_id, summary })` 推送汇总
5. 扫描结果暂存在 Rust 端 `AppState` 的 `Arc<Mutex<HashMap<String, ScanResults>>>`，按 scan_id 索引
6. 前端按需调分页查询命令拉取切片，避免一次传整棵树
7. 用户离开页面或显式调 `disk_clear_scan(scan_id)` 释放内存

### 新增依赖

| crate | 版本 | 用途 | 体积 |
|---|---|---|---|
| `walkdir` | 2.x | 递归目录遍历，处理符号链接环路 | ~30KB |
| `trash` | 2.x | 跨平台入回收站（Windows 走 COM IFileOperation） | ~50KB |
| `sha2` | 0.10 | 重复文件指纹哈希（SHA-256） | ~50KB |
| `uuid` | 1.x | scan_id 生成（v4） | ~20KB |

不复用 `sysinfo`：sysinfo 的磁盘信息是分区级，不递归到文件级，必须自己 walk。

### 后端命令

新增模块 `disk_analyzer.rs`，注册到 `main.rs` 的 `generate_handler!`。共 11 个命令：

| 命令 | 签名 | 作用 |
|---|---|---|
| `disk_scan_start` | `(path, opts: ScanOptions) -> Result<String>` | 启动扫描，返回 scan_id |
| `disk_scan_cancel` | `(scan_id) -> Result<()>` | 通过 AtomicBool 取消 |
| `disk_scan_status` | `(scan_id) -> Result<ScanStatus>` | 查询 running/completed/failed/cancelled |
| `disk_get_summary` | `(scan_id) -> Result<ScanSummary>` | 总文件数、总大小、耗时、跳过目录数 |
| `disk_get_folders` | `(scan_id, parent: Option<String>, limit, offset) -> Result<FolderPage>` | 文件夹大小分析（按下钻层级返回） |
| `disk_get_top_files` | `(scan_id, limit, offset) -> Result<FilePage>` | 大文件 Top N（按 size desc） |
| `disk_get_extension_stats` | `(scan_id, limit, offset) -> Result<ExtStatPage>` | 按扩展名聚合 |
| `disk_get_duplicates` | `(scan_id, limit, offset) -> Result<DupPage>` | 重复文件分组（按 wasted_bytes desc） |
| `disk_delete_files` | `(paths: Vec<String>) -> Result<DeleteResult>` | 入回收站，返回每条结果 |
| `disk_clear_scan` | `(scan_id) -> Result<()>` | 释放 Rust 端结果内存 |
| `disk_locate_in_explorer` | `(path) -> Result<()>` | 用 `explorer.exe /select,"path"` 定位 |

### 设计决策

1. **结果暂存 Rust 端**：扫描百万级文件产生的 `Vec<FileInfo>` 可达 100MB+，一次 IPC 传回会卡死前端。改为后端保存、前端分页拉取。
2. **top_files 用最小堆限 5000**：避免保存全量文件列表导致内存爆炸。5000 条 FileInfo ≈ 1MB。
3. **重复检测用前 64KB + 大小做指纹**：不做全文件 SHA-256。前 64KB + 大小相同，碰撞概率极低（ponytail: 极端构造场景可能误判，升级路径是全文件 SHA-256）。
4. **取消信号用 `Arc<AtomicBool>`**：跨线程共享，walk 循环每 1000 个文件检查一次，cancel 后保留已扫数据可查询。`disk_scan_cancel` 命令本身立即返回（仅置位），扫描任务在下一轮检查点（最多再扫 1000 个文件）后才真正停止，status 才变为 Cancelled。
5. **并发扫描策略**：允许多个 scan_id 并发扫描（不同路径），但内存开销线性增长。建议前端在已有扫描进行时禁用"开始扫描"按钮，或先 `disk_clear_scan(old_id)` 释放旧结果。
6. **不删文件夹**：即使发现整文件夹可删也只删文件，YAGNI + 安全。
7. **trash crate 而非 PowerShell**：原生 COM 调用，不弹黑框、不需要 GBK 解码、无 Tauri 沙箱风险（项目记忆中 PowerShell 沙箱踩过多次坑）。

## 数据结构

### Rust 后端

```rust
pub struct ScanOptions {
    pub include_hidden: bool,        // 是否包含隐藏文件/文件夹
    pub detect_duplicates: bool,     // 是否做重复检测（耗时）
    pub max_files: Option<u64>,      // 软上限，超过则警告但继续
    pub follow_symlinks: bool,       // 默认 false，避免环路
}

pub enum ScanStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

pub struct ScanResults {
    pub scan_id: String,
    pub root_path: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub status: ScanStatus,
    pub cancel_flag: Arc<AtomicBool>,

    // 流式累积（扫描中）
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_path: Option<String>,
    pub skipped_dirs: Vec<String>,   // 无权限等被跳过的目录，上限 1000 条（ponytail: 超出则只保留前 1000，summary 中显示总数）

    // 完成后填充
    pub folders: Vec<FolderInfo>,         // 所有文件夹扁平化
    pub top_files: Vec<FileInfo>,         // Top 5000，按 size desc
    pub ext_stats: Vec<ExtensionStat>,    // 按 total_size desc
    pub duplicates: Vec<DuplicateGroup>,  // 按 wasted_bytes desc，仅当 detect_duplicates=true
}

pub struct FolderInfo {
    pub path: String,
    pub parent: Option<String>,    // 下钻用
    pub name: String,
    pub depth: u32,
    pub file_count: u64,
    pub size_bytes: u64,
    pub percent_of_root: f32,
}

pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_ms: i64,
    pub extension: String,
}

pub struct ExtensionStat {
    pub extension: String,         // "" 表示无扩展名
    pub file_count: u64,
    pub total_size: u64,
    pub percent: f32,
}

pub struct DuplicateGroup {
    pub group_id: u32,
    pub file_size: u64,
    pub file_count: u32,
    pub wasted_bytes: u64,         // file_size * (file_count - 1)
    pub files: Vec<FileInfo>,
}

pub struct ScanSummary {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub skipped_count: u32,
    pub duration_ms: u64,
    pub duplicates_wasted_bytes: Option<u64>,  // None 表示未启用重复检测
}

pub struct DeleteResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<DeleteFailure>,
}

pub struct DeleteFailure {
    pub path: String,
    pub error: String,
}

// 分页响应
pub struct FolderPage { pub items: Vec<FolderInfo>, pub total: u64 }
pub struct FilePage { pub items: Vec<FileInfo>, pub total: u64 }
pub struct ExtStatPage { pub items: Vec<ExtensionStat>, pub total: u64 }
pub struct DupPage { pub items: Vec<DuplicateGroup>, pub total: u64 }
```

### 重复检测算法

1. walk 时收集所有文件元组 `(size_bytes, path)`
2. 按 `size_bytes` 分组，过滤出 `count > 1` 的组
3. 对每个 size 组内文件，计算快速指纹 `fingerprint = SHA-256(前 64KB)`
4. 按 fingerprint 二次分组，子组 `count > 1` 即为重复组
5. 按 `wasted_bytes = file_size * (file_count - 1)` 倒序排序

性能：1M 文件元组 ≈ 80MB 内存；哈希只对 size 重复的文件做，多数文件不参与哈希。

### 前端类型（diskAnalyzerTypes.ts）

TS 接口与 Rust 结构一一对应，仅命名改为 camelCase（`scanId` / `startedAt` / `filesScanned` 等），符合项目 Tauri 命令参数 camelCase 约定。

## 前端 UI 设计

### 页面结构

单页面 `DiskSpaceAnalyzer.vue`，三层卡片结构：

```
┌─ 扫描配置卡片 ─────────────────────────────────┐
│ 标题: 磁盘空间分析                              │
│ 路径: [_________________________] [浏览]        │
│ ☐ 包含隐藏文件  ☐ 检测重复文件  ☐ 跟随符号链接 │
│ [开始扫描]                                      │
└────────────────────────────────────────────────┘

┌─ 扫描进度卡片（仅扫描中显示）──────────────────┐
│ 当前路径: D:\work\...                           │
│ 已扫描: 23,456 文件 | 12.3 GB | 耗时 0:23       │
│ ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░       │
│ [取消扫描]                                      │
└────────────────────────────────────────────────┘

┌─ 结果展示卡片（仅完成后显示）──────────────────┐
│ 汇总: 235,687 文件 | 412.3 GB | 耗时 1:24       │
│                                                │
│ [文件夹大小] [大文件 Top N] [按类型] [重复文件]│ ← el-tabs
│ Tab1: 文件夹大小                                │
│   当前路径: D:\work\  [返回上级] [面包屑]       │
│   表格: 名称 | 大小 | 文件数 | 占比 | 操作       │
│   占比列内嵌 el-progress 可视化                 │
│                                                │
│ Tab2: 大文件 Top N                             │
│   表格: ☐ | 路径 | 大小 | 修改时间 | 操作       │
│   底部: [加载更多] [删除勾选 (N, X GB)]         │
│                                                │
│ Tab3: 按类型                                    │
│   表格: 扩展名 | 文件数 | 总大小 | 占比 | 可视化│
│                                                │
│ Tab4: 重复文件 (仅当 detect_duplicates=true)   │
│   表格: 组号 | 文件数 | 单个大小 | 浪费空间 | 操作│
│   展开后: 组内文件列表 + 勾选框                 │
│   底部: [删除勾选 (N, X GB)]                    │
└────────────────────────────────────────────────┘
```

### 关键 UI 决策

- **路径选择**：用 `tauri-plugin-dialog` 的 `dialog.open({ directory: true })`，与 NoteEditor 的目录选择一致
- **进度更新**：`listen('disk-scan-progress', ...)`，后端已 500ms 节流
- **文件夹下钻**：点击"下钻"切换 `currentParent`，重新调 `disk_get_folders(scan_id, parent, ...)`，维护面包屑导航
- **大文件列表**：el-table 默认 Top 100，"加载更多"按钮拉取下 100 条
- **删除流程**：勾选 → `ElMessageBox.confirm` 列出待删文件总大小 → 调 `disk_delete_files` → 刷新列表
- **定位**：调 `disk_locate_in_explorer` 打开资源管理器并选中
- **错误处理**：扫描中权限不足的文件夹跳过累计（不中断），完成后在汇总里显示"跳过 X 个无法访问的文件夹"
- **样式**：复用项目 `theme.css` 变量，按 `AGENTS.md` 卡片式布局规范

### 侧边栏归类

归入"系统工具"分组（与 SystemInfoView / NetworkInfoView / ProcessListView / HardwareInfoView / SoftwareEnvView 同组），命名"磁盘分析"。

## 错误处理与边界

### 错误路径

- **路径不存在/无权限**：`disk_scan_start` 立即返回错误，不进入后台任务
- **扫描中部分文件夹无权限**：walkdir 用 `filter_entry` 跳过，累计 `skipped_dirs`，完成后 summary 显示
- **scan_id 不存在的查询**：返回 `Err("scan not found or expired")`
- **超过 max_files 软上限**：继续扫描，emit `disk-scan-warning` 事件，前端提示"文件数超限，仅统计 Top 5000 大文件"
- **取消扫描**：cancel_flag 置位，停止后保留已扫数据可查询，status=Cancelled
- **删除失败**：trash crate 返回 `Error`，按文件粒度返回 `Vec<(path, Result)>`，前端逐条显示成功/失败

### 安全考虑

- 删除命令二次确认（`ElMessageBox.confirm`），列出待删文件总大小
- 删除前再校验路径存在（避免 UI 显示与磁盘状态不一致）
- 路径用 `PathBuf::canonicalize` 规范化，防止 `..` 注入
- **不提供"删除文件夹"能力**（即使重复检测发现整文件夹可删，也只删文件）

### 性能边界（ponytail）

- 文件数无硬上限，但 top_files 用最小堆限制 5000
- 重复检测：内存中保存所有文件的 `(size, path)` 元组，1M 文件 ≈ 80MB
- 大文件哈希：单文件 IO 受磁盘速度限制，无法并发加速
- 升级路径：超大数据集（>10M 文件）需要分块落 SQLite，MVP 不做

## 与其他工具的集成

### 历史记录（接入）

扫描完成时调 `store.addHistory`：
- `toolId: 'disk-analyzer'`
- `inputPreview: scanPath`（截断 50 字符）
- `inputFull: scanPath`
- `outputPreview: '发现 N 文件，X GB，可回收 Y GB'`（截断 50 字符）
- `outputFull: 完整 summary JSON`

用户可在 HistoryView 双击历史条目用相同路径重新扫描。

### 工作流集成（不接入）

磁盘分析输出是结构化数据，不是典型工作流步骤，YAGNI。

### 变量池集成（不接入）

输出非字符串，不适合变量池模型，YAGNI。

## 测试策略

按 AGENTS.md「ONE runnable check」规则，在 `disk_analyzer.rs` 末尾加 `#[cfg(test)] mod tests`，用 `tempfile` crate（已是 dev-dependency）造测试目录树：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn make_test_tree() -> TempDir {
        // 造一棵已知结构的目录树
        // root/
        //   ├─ a.txt (1KB)
        //   ├─ b.txt (1KB, 内容同 a.txt)  ← 重复
        //   ├─ sub/
        //   │   ├─ c.txt (2KB)
        //   │   └─ d.txt (1KB, 内容同 a.txt)  ← 重复
        //   └─ empty/
    }

    #[test]
    fn folder_size_aggregation_correct() { /* 断言 root 总大小 = 1+1+2+1 KB */ }

    #[test]
    fn top_files_sorted_desc() { /* 断言 top_files[0] 是 c.txt (2KB) */ }

    #[test]
    fn duplicate_detection_groups_by_content() {
        /* 断言 a/b/d 三文件分为一组，wasted_bytes = 2KB */
    }

    #[test]
    fn cancel_flag_stops_scan() { /* 启动后立即 cancel，断言 status=Cancelled */ }

    #[test]
    fn extension_stats_correct() { /* 断言 .txt 4 个，总大小 5KB */ }
}
```

前端 UI 逻辑无独立测试（项目无前端测试框架）；手动验证扫描/下钻/删除流程。

debug 日志：所有命令入口、扫描开始/结束、错误路径加 `debug_log!()`，符合 AGENTS.md 后端新增功能必须加 debug 日志规则。

## 版本与文档

- 版本号：4.2.0 → 4.3.0
- 同步更新 README.md 功能阶段记录：「v4.3.0 新增磁盘空间分析工具（文件夹大小 / 大文件 Top N / 按类型统计 / 重复文件检测，可入回收站删除）」
- 同步更新 `SidebarNav.vue`、`tauri.conf.json`、`Cargo.toml`、`package.json`、`package-lock.json` 版本号

## 后续子项目

本文档仅为「文件系统操作」方向的第 1 个子项目。后续三个子项目将各自走 brainstorming → spec → plan 流程：

1. ✅ 磁盘空间分析（本文档）
2. ⏳ 批量重命名
3. ⏳ 压缩解压
4. ⏳ 文件名搜索
