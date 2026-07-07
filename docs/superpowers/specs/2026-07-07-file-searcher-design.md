# 全文搜索工具设计文档（B1）

## 概述

为栗的百宝箱（LitoBox）新增"全文搜索"工具（V4.6），支持在指定目录下按**文件名**或**文件内容**搜索，支持正则表达式。定位类似 Everything + grep 的轻量组合：文件名模式用于快速定位文件，内容模式用于在代码/文本中查找匹配行。

采用与磁盘分析器（V4.3）同构的实时遍历架构：不建索引、`walkdir` 遍历、`thread::spawn` 异步、`emit` 进度事件、`AtomicBool` 可取消。复用 `disk_locate_in_explorer` 命令实现"在资源管理器中定位"动作。

## 需求背景

- **场景一**：开发中只记得文件名片段或某段代码内容，想在项目目录快速定位文件
- **场景二**：在大量日志/配置中按正则找匹配行（如 `\d{4}-\d{2}-\d{2}` 找日期），不想开 IDE 等待索引
- **现状**：工具箱有磁盘分析器（看占用）但无搜索能力；Windows 自带搜索慢且不支持正则；Everything 强但需另装、不搜内容
- **约束**：纯本地离线；不建索引（工具箱场景偶尔用，建索引维护成本高）；支持进度反馈与取消；中文 Windows 多编码文件混存

## 架构设计

### 整体方案：镜像磁盘分析器架构

新建 `src-tauri/src/file_searcher.rs`，完全复用磁盘分析器的成熟模式：
- `OnceLock<Mutex<HashMap<search_id, Arc<Mutex<SearchResults>>>>>` 全局状态
- `std::thread::spawn` 异步执行
- `AppHandle.emit()` 进度事件
- `AtomicBool` 取消标志
- `search_id` 生命周期管理

### 文件结构

```
src-tauri/src/
  ├── file_searcher.rs          # 新增：搜索核心 + 6 个 Tauri 命令 + 测试
  ├── file_encoding.rs          # 修改：末尾新增 read_file_auto 函数（约 25 行）
  └── main.rs                   # 修改：mod file_searcher + generate_handler 注册 6 命令
src/utils/
  ├── fileSearcherTypes.ts      # 新增：TS 类型定义
  └── fileSearcherClient.ts     # 新增：invoke 封装
src/views/
  └── FileSearcher.vue          # 新增：搜索页面（单页，文件名/内容 radio 切换）
src/
  ├── App.vue                   # 修改：toolComponentMap 加 fileSearcher
  └── store/index.ts            # 修改：TOOL_LIST 末尾加条目（category: 'system'）
src-tauri/Cargo.toml            # 修改：加 regex = "1.10"，版本 4.4.0 → 4.6.0
package.json                    # 修改：版本 4.5.0 → 4.6.0
README.md                       # 修改：版本表加 V4.6 行
```

### 新增依赖

| 依赖 | 用途 |
|------|------|
| `regex = "1.10"`（Cargo.toml） | 文件名/内容正则匹配，Rust 原生引擎 |

复用 Cargo.toml 已有：`walkdir`（遍历）、`encoding_rs`（解码）、`uuid`（search_id）、`tempfile`（测试）。

### 关键复用

- **walkdir 遍历** — 与 disk_analyzer 同构
- **`disk_locate_in_explorer` 命令** — 前端直接 `invoke('disk_locate_in_explorer', { path })`，0 新代码
- **`file_encoding::read_file_auto`** — 新增小函数，内容搜索时复用解码（处理 GBK/UTF-8/UTF-16）
- **`debug_log!` 宏** — 模块内复制一份（与 disk_analyzer 一致，项目惯例）

### 不集成（与 disk_analyzer 保持一致）

- **工作流 / 变量池 / 历史记录** — 系统工具非文本转换，磁盘分析器也未集成

## 数据结构

### Rust 结构（`file_searcher.rs`，字段 camelCase 沿用 disk_analyzer 惯例）

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SearchOptions {
    pub mode: String,              // "filename" | "content"
    pub query: String,             // 正则表达式
    pub caseSensitive: bool,
    pub extensions: Vec<String>,   // 包含列表，如 ["ts","js"]；空=不限
    pub excludeExtensions: Vec<String>, // 排除列表，如 ["exe","dll"]
    pub includeHidden: bool,
    pub maxContentFileBytes: u64,  // 内容模式：超过此大小的文件跳过内容只匹配文件名
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedLine {
    pub lineNumber: u32,
    pub lineText: String,          // 截断 500 字符
    pub matchRanges: Vec<(u32, u32)>, // 字符偏移，单行最多 5 个
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultItem {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub sizeBytes: u64,
    pub modifiedMs: i64,
    pub matchCount: u32,           // 文件总命中数（文件名模式=1）
    pub matchedLines: Vec<MatchedLine>, // 内容模式最多 3 行预览；文件名模式为空
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum SearchStatus {
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchProgress {
    pub searchId: String,
    pub filesScanned: u64,
    pub bytesScanned: u64,
    pub matchesFound: u32,
    pub currentPath: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchSummary {
    pub totalFiles: u64,
    pub totalDirs: u64,
    pub bytesScanned: u64,
    pub matchesFound: u32,
    pub durationMs: u64,
    pub truncated: bool,           // 是否命中 MAX_RESULTS 上限
    pub skippedCount: u32,         // 二进制/过大/无权限
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResultsPage {
    pub items: Vec<SearchResultItem>,
    pub total: u64,
}
```

### 关键常量

```rust
const MAX_RESULTS: u32 = 1000;         // 单次搜索结果上限
const MAX_PREVIEW_LINES: usize = 3;    // 每文件最多预览行
const MAX_MATCHES_PER_LINE: usize = 5; // 单行最多高亮区间
const MAX_LINE_TEXT_CHARS: usize = 500;
const DEFAULT_MAX_CONTENT_BYTES: u64 = 10 * 1024 * 1024; // 10MB
const CANCEL_CHECK_INTERVAL: u64 = 1000; // 每 1000 项检查一次取消
```

### 设计要点

- **按文件分组**而非按行（grep 按行）：一个文件最多 3 行预览，避免大文件刷屏；用户想看全部就"在资源管理器定位"后打开
- **`maxResults` 不暴露为选项**，硬编码 1000（与 disk_analyzer `TOP_FILES_LIMIT` 同思路），降低 UI 复杂度
- **`matchRanges` 用字符偏移**而非字节偏移，前端 `slice()` 直接可用
- **`truncated` 标志**告知用户结果被截断，避免误以为搜索不全

## 命令接口

### Tauri 命令（6 个，注册到 main.rs generate_handler!）

```rust
#[tauri::command]
pub async fn file_search_start(
    app: AppHandle,
    path: String,
    opts: SearchOptions,
) -> Result<String, String>
// 返回 search_id；thread::spawn 执行 run_search；失败返回错误字符串

#[tauri::command]
pub fn file_search_cancel(searchId: String) -> Result<(), String>

#[tauri::command]
pub fn file_search_status(searchId: String) -> Result<SearchStatus, String>

#[tauri::command]
pub fn file_search_get_summary(searchId: String) -> Result<SearchSummary, String>

#[tauri::command]
pub fn file_search_get_results(
    searchId: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchResultsPage, String>
// 默认 limit=100, offset=0

#[tauri::command]
pub fn file_search_clear(searchId: String) -> Result<(), String>
// 释放内存，从 SCANS map 移除
```

### 事件（前端 listen 监听，命名与 disk-scan-* 对齐）

| 事件名 | 载荷 | 时机 |
|--------|------|------|
| `file-search-progress` | `SearchProgress` | 每 200ms 或每 1000 文件 |
| `file-search-complete` | `SearchSummary` | 搜索结束（含成功/失败/取消） |
| `file-search-warning` | `{ searchId, message }` | 命中 MAX_RESULTS 截断 / 跳过大量文件 |

### run_search 核心流程

```
1. canonicalize(path)，失败返回错误
2. 编译正则 build_regex(query, caseSensitive)，失败返回 "正则表达式无效: ..."
   - caseSensitive=false 时用 (?i) 前缀注入
3. 解析扩展名过滤：parse_extension_filter 统一小写、去前导点
4. WalkDir::new(root).into_iter() 遍历：
   ├─ 取消检查（每 1000 项查一次 cancel_flag）
   ├─ 目录：计数；非 includeHidden 且 is_hidden → skip_current_dir
   └─ 文件：
      ├─ 扩展名过滤不通过 → continue
      ├─ 文件名模式：regex.is_match(name) → 加入结果（matchCount=1, matchedLines=[]）
      └─ 内容模式：
         ├─ size > maxContentFileBytes → 仅文件名匹配（降级）
         ├─ 否则 read_file_auto → is_binary 检测 → scan_content 按行扫描
         │   ├─ 二进制（无 BOM 含 \0）→ 跳过内容，仅文件名匹配
         │   ├─ 收集命中行（最多 MAX_PREVIEW_LINES）
         │   └─ matchRanges 算字符偏移
         └─ 加入结果
5. results.len() >= MAX_RESULTS → truncated=true，break + emit warning
6. 完成：设置 status，emit file-search-complete
```

### 关键决策

- **大小写不敏感用 `(?i)` 前缀注入**而非 `RegexBuilder::case_insensitive`——一行实现，避免引入 builder API
- **超大文件降级为文件名匹配**而非跳过——用户仍能看到文件，只是没读内容（避免 5GB 日志卡死）
- **二进制检测用"BOM 优先 + 前 8KB 含 \0"双重判定**——解决 UTF-16 文件含 \0 的误判（详见编码处理节）
- **取消检查每 1000 项一次**——与 disk_analyzer `CANCEL_CHECK_INTERVAL` 一致，平衡响应性与性能
- **不提供 `file_search_locate_in_explorer`**——前端直接 `invoke('disk_locate_in_explorer', { path })` 复用现有命令

### 错误处理

- 路径不存在 / 无权限 → `Err("路径无法访问: ...")`
- 正则编译失败 → `Err("正则表达式无效: ...")`
- 单文件读取失败 → 计入 `skippedCount`，不中断搜索（与 disk_analyzer 跳过无权限目录一致）

## 前端 UI

### 页面结构（`FileSearcher.vue`，沿用 tool-card 模式）

四个卡片，状态机驱动显隐：
1. **搜索配置卡片（sticky）**：搜索路径 + 浏览/上次路径、模式 radio（文件名/内容）、搜索词 input（回车触发）、扩展名过滤 input、高级选项（区分大小写/包含隐藏/内容最大文件 MB）、搜索按钮
2. **进度卡片（仅 searching）**：当前路径、已扫描/命中/耗时、indeterminate 进度条、取消按钮
3. **错误卡片**：错误信息
4. **结果卡片**：摘要（命中数/耗时/截断提示/跳过提示）+ el-table（文件名列 + 命中行列 + 操作列）+ 分页

### 状态机

```ts
type SearchState = 'idle' | 'searching' | 'completed' | 'failed' | 'cancelled'
```

### 交互流

1. 输入查询后回车或点搜索 → `startSearch()`
2. `startSearch`：解析 `extFilterText`（`!` 前缀 → `excludeExtensions`，否则 `extensions`）；`opts.maxContentFileBytes = maxContentMb * 1024 * 1024`；调 `fileSearchStart`；切换 `searching`；启动计时器
3. 监听 3 个事件：
   - `file-search-progress` → 更新 `progress`、`elapsedMs`
   - `file-search-complete` → 调 `fileSearchGetSummary` + `loadResults(1)`；切换 `completed`
   - `file-search-warning` → `ElMessage.warning(message)`
4. 取消 → `fileSearchCancel`，等 `complete` 事件后切 `cancelled`
5. 定位 → `invoke('disk_locate_in_explorer', { path: row.path })`
6. 分页 → `loadResults(page)` 调 `fileSearchGetResults(searchId, pageSize, offset)`
7. `onUnmounted` → 移除事件监听；若仍在搜索调 `cancel`；调 `fileSearchClear` 释放后端内存

### 持久化（localStorage）

- `litobox.fileSearcher.lastPath` — 上次搜索路径
- `litobox.fileSearcher.lastOpts` — 上次选项（mode/caseSensitive/extFilterText/includeHidden/maxContentMb）

### UI 要点

- **进度条用 `indeterminate`**（不像磁盘分析器有 percentage）——文件搜索无已知总量，无法算百分比
- **双击结果行 = 定位**（与 disk_analyzer 双击行为一致）
- **命中行高亮**：`highlightLine(ml)` 把 `matchRanges` 转成 `<mark>` 标签包裹，用 `v-html` 渲染（先 escape HTML 防注入）
- **扩展名输入框语法**：逗号分隔，`!` 前缀切换为排除模式（不能混用，混用时 `ElMessage.warning` 提示）

## 编码与二进制处理

### 复用 file_encoding.rs 的解码能力

现状：`read_file_with_encoding(path, encoding)` 需要前端指定编码；`detect_file_encoding` 单独检测会再读一次盘。内容搜索既不知道编码、又不能双读。

**方案：在 `file_encoding.rs` 末尾新增一个小函数**（约 25 行，复用现有 BOM/UTF-8/GBK 判断逻辑，单次读盘）：

```rust
/// 读取文件并自动检测编码解码为字符串（单次读盘，供内容搜索复用）
pub fn read_file_auto(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("读取失败: {}", e))?;
    // BOM 优先（与 detect_file_encoding 一致）
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(String::from_utf8_lossy(&bytes[3..]).into_owned());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..].chunks_exact(2)
            .map(|c| if bytes[1] == 0xFE { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();
        return Ok(String::from_utf16_lossy(&utf16));
    }
    // 无 BOM：先试 UTF-8 严格，失败回退 GBK（与项目惯例一致）
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => {
            let (decoded, _, _) = GBK.decode(&bytes);
            Ok(decoded.into_owned())
        }
    }
}
```

file_searcher.rs 直接调用 `crate::file_encoding::read_file_auto(&path)`，不经过 Tauri 命令层（避免 async 命令开销 + 避免重复读盘）。

### 二进制检测（必须在解码前）

**核心问题**：UTF-16 文件的 ASCII 字符高位是 `\0`（`0x41 0x00`），"含 \0 即二进制"的 grep 启发式会误判 UTF-16。

**解决顺序**：
```
读 bytes →
  1. 有 UTF-16 BOM (FF FE / FE FF)？→ 非二进制，走 UTF-16 解码
  2. 有 UTF-8 BOM (EF BB BF)？→ 非二进制，走 UTF-8 解码
  3. 无 BOM：检查前 8KB 是否含 \0 字节
     ├─ 含 \0 → 判定为二进制，跳过内容（仅文件名匹配，计入 skippedCount）
     └─ 不含 \0 → 走 read_file_auto 剩余逻辑（UTF-8 严格 → GBK 回退）
```

### 行扫描与匹配

```rust
fn scan_content(text: &str, re: &Regex, max_lines: usize) -> (u32, Vec<MatchedLine>) {
    let mut total_matches: u32 = 0;
    let mut previews: Vec<MatchedLine> = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line_no = (idx as u32) + 1;
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for m in re.find_iter(line) {
            // 字符偏移（前端 slice 友好）
            let start = line[..m.start()].chars().count() as u32;
            let end = line[..m.end()].chars().count() as u32;
            ranges.push((start, end));
            total_matches += 1;
            if ranges.len() >= MAX_MATCHES_PER_LINE { break; }
        }
        if !ranges.is_empty() && previews.len() < max_lines {
            let truncated_text = if line.chars().count() > MAX_LINE_TEXT_CHARS {
                line.chars().take(MAX_LINE_TEXT_CHARS).collect::<String>() + "…"
            } else { line.to_string() };
            previews.push(MatchedLine { lineNumber: line_no, lineText: truncated_text, matchRanges: ranges });
        }
    }
    (total_matches, previews)
}
```

### 关键决策

- **BOM 优先于 \0 检测**——解决 UTF-16 误判，这是 grep 在 UTF-16 环境下的已知短板
- **8KB 探测窗口**——平衡内存与准确性，前 8KB 无 \0 基本可断定是文本
- **字符偏移而非字节偏移**——前端 JS `slice(start, end)` 直接可用，无需处理多字节字符
- **单行最多 5 个高亮区间**——避免一行几百个匹配撑爆结果（如 `.` 匹配整行）
- **`lineText` 截断 500 字符 + `…`**——超长行（如压缩 JS）不撑爆 UI
- **`total_matches` 继续累加即使预览已满 3 行**——用户能看到文件总命中数，知道是否值得打开

### 性能边界

- **10MB 文件 + 正则**：regex crate 约 50-100ms，可接受
- **10 万文件全扫描**：磁盘分析器实测约 30-60 秒，文件搜索因多了读盘+解码会慢 2-3 倍，但有进度反馈+取消，可接受
- **内存**：单文件 10MB String 是临时变量，搜索完即释放；结果只保留最多 1000 个 `SearchResultItem`（每个含 ≤3 行预览），峰值内存 < 50MB

## 测试策略

### 遵循项目惯例

沿用 `disk_analyzer.rs` 的测试模式：`#[cfg(test)] mod tests` 内嵌在模块底部，纯 `assert_eq!`/`assert!`，用 `tempfile::TempDir` 处理文件系统，不引入框架。

### 测试覆盖（聚焦非平凡纯逻辑，5 个测试函数）

1. **扩展名过滤解析**：`parse_extension_filter("ts, .js, .vue")` → 包含列表；`"!exe, dll"` → 排除列表
2. **二进制检测（含 UTF-16 误判防护）**：UTF-8 BOM / UTF-16 LE BOM / UTF-16 BE BOM 均非二进制；无 BOM 含 \0 是二进制；纯 ASCII 非二进制
3. **内容扫描字符偏移正确性**：中文多字节场景，`"你好 World 继续"` 中 `World` 偏移应为 (3, 8)
4. **大小写不敏感正则注入**：`build_regex("foo", false)` 匹配 "FOO"；`build_regex("foo", true)` 不匹配 "FOO"
5. **编码自动解码**：GBK 字节 `[D6 D0 CE C4]` → "中文"；UTF-16 LE with BOM `[FF FE 48 00 69 00]` → "Hi"

### 不测试的部分（理由）

- **Tauri 命令层**（`file_search_start` 等）——薄包装，靠手动验收覆盖（与 disk_analyzer 一致，其命令层无单测）
- **前端 Vue 组件**——项目无前端测试框架，靠手动验收
- **walkdir 遍历**——第三方库已测，集成行为靠端到端验收

### 运行方式

```powershell
cd d:\work\codes\litobox\src-tauri
cargo test file_searcher
cargo test read_file_auto
```

## 手动验收清单

1. 文件名模式：搜 `.*\.rs` 在 litobox 项目根，能找到所有 .rs 文件
2. 内容模式：搜 `debug_log!` 在 `src-tauri/src/`，结果含命中行预览且高亮位置正确
3. 中文内容：在 GBK 编码的 .txt 中搜中文关键词，能命中（验证 GBK 回退）
4. UTF-16 文件：用记事本另存为"Unicode"的 .txt 搜内容，能命中（验证 BOM 优先于 \0 检测）
5. 二进制跳过：搜 .exe 目录，二进制文件不计入内容命中但可文件名匹配
6. 扩展名过滤：`ts,vue` 只搜这两类；`!exe,dll` 排除这两类
7. 取消：大目录搜索中点取消，能立即停止并切 cancelled 状态
8. 截断：构造 >1000 命中，结果停在 1000 且 summary.truncated=true
9. 定位：点结果"定位"按钮，资源管理器打开并选中该文件
10. 分页：>100 条结果时翻页正常
11. 错误路径：不存在路径 / 无效正则（如 `[`）显示友好错误
12. 释放：离开页面后再次进入，能正常新搜索（验证 `file_search_clear` 清理）

## 版本与文档

- `package.json` 4.5.0 → 4.6.0
- `Cargo.toml` 4.4.0 → 4.6.0（顺便同步，V4.5 漏更）
- `README.md` 版本表加 V4.6 行：`| V4.6 | ✅ | 全文搜索工具（文件名/内容双模式、正则、进度取消、编码自动识别） |`
- `docs/superpowers/plans/feature-backlog.md`：B1 从候选池移到已完成版本
