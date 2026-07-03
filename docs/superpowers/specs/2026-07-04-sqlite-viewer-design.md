# SQLite 查看器设计文档

## 概述

为栗的百宝箱（LitoBox）新增一个 SQLite 数据库查看器工具，用于开发调试时快速查看本地 `.db` / `.sqlite` 文件的表结构和数据。定位为**只读查看器**，不涉及数据编辑，从根源上避免误操作损坏数据库。

## 需求背景

- **场景**：开发调试时查看本地 SQLite 数据库文件（如 App 本地数据库、缓存数据库等）
- **核心功能**：打开文件、浏览表结构、预览数据、执行 SELECT 查询、导出 CSV
- **约束**：纯本地离线运行；只读模式打开数据库；查询结果强制 LIMIT 保护

## 架构设计

### 文件结构

```
src-tauri/src/
  └── sqlite_viewer.rs          # 新增：SQLite 查询命令（只读）
src/views/
  └── SqliteViewerView.vue      # 新增：查看器页面
src/utils/
  └── sqliteTypes.ts            # 新增：类型定义
```

### 后端命令

新增模块 `sqlite_viewer.rs`，复用已有的 `rusqlite` 依赖（项目 db.rs 已使用）。提供 5 个 Tauri 命令：

| 命令 | 作用 |
|------|------|
| `sqlite_list_tables(db_path)` | 打开指定路径的 .db 文件，返回表名 + 预估行数 |
| `sqlite_get_schema(db_path, table_name)` | 返回指定表的字段结构 |
| `sqlite_query(db_path, sql, limit?)` | 执行 SELECT 查询，返回结果集 |
| `sqlite_table_preview(db_path, table_name)` | 快速预览某表前 100 条 |
| `sqlite_export_csv(db_path, sql, save_path)` | 将查询结果导出为 CSV 文件 |

### 设计决策

1. **无状态连接**：每次命令调用都传入文件路径，不持有连接对象。rusqlite 打开本地文件是毫秒级，性能足够，且避免连接泄漏。
2. **只读模式**：用 `OpenFlags::SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` 打开，从根本上防止误操作改坏数据。
3. **强制 LIMIT**：`sqlite_query` 默认限制 1000 行，若用户 SQL 无 LIMIT 则自动追加。
4. **仅允许 SELECT**：`sqlite_query` 对 SQL 做前缀检查，非 SELECT 语句拒绝执行。

## 数据结构

### Rust 后端

```rust
pub struct TableInfo {
    pub name: String,
    pub row_count: i64,        // 预估行数
}

pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,     // INTEGER/TEXT/REAL/BLOB/...
    pub not_null: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
}

pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,  // 每个单元格用 JSON 值，支持 null/数字/字符串
    pub affected_rows: usize,
    pub execution_ms: u64,
}
```

### 命令签名

```rust
#[tauri::command]
pub fn sqlite_list_tables(db_path: String) -> Result<Vec<TableInfo>, String>

#[tauri::command]
pub fn sqlite_get_schema(db_path: String, table_name: String) -> Result<Vec<ColumnInfo>, String>

#[tauri::command]
pub fn sqlite_query(db_path: String, sql: String, limit: Option<usize>) -> Result<QueryResult, String>

#[tauri::command]
pub fn sqlite_table_preview(db_path: String, table_name: String) -> Result<QueryResult, String>

#[tauri::command]
pub fn sqlite_export_csv(db_path: String, sql: String, save_path: String) -> Result<usize, String>
```

## 前端页面设计

### 布局（三栏）

```
┌─────────────────────────────────────────────────┐
│  文件路径栏 [选择.db文件] [当前: C:/xxx.db]      │
├──────────┬──────────────────────────────────────┤
│          │  SQL 编辑器                           │
│  表列表   │  ┌──────────────────────────────┐   │
│          │  │ SELECT * FROM users LIMIT 10 │   │
│  users   │  │                              │   │
│  orders  │  └──────────────────────────────┘   │
│  products│  [执行查询] [清除] [导出CSV]          │
│  ...     ├──────────────────────────────────────┤
│          │  结果表格                            │
│          │  ┌────┬──────┬────────┬─────────┐   │
│          │  │ id │ name │ email  │ created │   │
│          │  ├────┼──────┼────────┼─────────┤   │
│          │  │ 1  │ 张三 │ a@b.c  │ 2024... │   │
│          │  └────┴──────┴────────┴─────────┘   │
│          │  耗时: 3ms | 返回: 10行              │
├──────────┴──────────────────────────────────────┤
│  表结构面板（点击表名时展开）                     │
│  字段名 | 类型 | 主键 | 可空 | 默认值            │
└─────────────────────────────────────────────────┘
```

### 交互流程

1. 打开页面 → 显示"请选择数据库文件"提示
2. 选择文件 → 左侧加载表列表
3. 点击表名 → 右下方显示前 100 条数据 + 底部展开表结构
4. 输入 SQL → 点击执行 → 结果显示在表格
5. 点击导出 → 弹出保存对话框 → 生成 CSV

### 复用现有组件

- 文件选择：用 Tauri dialog 插件的 `open` 对话框
- 表格展示：Element Plus 的 `el-table`，支持虚拟滚动处理大结果集
- SQL 编辑器：`el-input textarea`（暂不引入 CodeMirror，保持简单）
- 遵循 `.tool-card` 卡片式布局规范，颜色用 `theme.css` 变量

## 错误处理

| 场景 | 处理方式 |
|------|----------|
| 文件不存在/不是 SQLite | 返回中文错误提示，前端红色边框展示 |
| SQL 语法错误 | 捕获 rusqlite 错误，返回原始错误信息 |
| 非 SELECT 语句 | 后端拒绝，返回"仅支持 SELECT 查询" |
| 查询超时 | 设置 5 秒 `busy_timeout` |
| 结果集过大 | 强制 LIMIT 1000，超出提示"结果已截断" |

## 集成规范

### 历史记录集成

每次执行查询，调用 `store.addHistory()`：
- `tool: 'sqlite-viewer'`
- `action: '执行查询'`
- `inputFull: SQL 语句`
- `outputFull: 结果摘要（前 50 行的 JSON 字符串）`
- `inputPreview` / `outputPreview`：截断 50 字符用于列表展示

### 工作流集成（后续迭代）

当前版本先实现独立工具页。后续可在 `WorkflowView.vue` 的 `executeStep()` 添加 `sqlite-viewer` 分支：
- 输入：数据库路径 + SQL（支持变量池）
- 输出：查询结果 JSON，可存入变量池

### 注册到工具箱

在前端工具列表配置中注册新工具（参照现有工具的注册方式，如在 `HomeView.vue` 或工具配置文件中添加条目）：
- 工具 ID: `sqlite-viewer`
- 名称: SQLite 查看器
- 分类: 开发工具
- 图标: 数据库相关图标

## 测试策略

按 AGENTS.md 要求，非平凡逻辑留一个可运行的检查。后端在 `sqlite_viewer.rs` 中写一个 `#[test]`：

- 创建临时 SQLite 文件，建一张测试表，插入几条数据
- 验证 `sqlite_list_tables` 返回正确的表名
- 验证 `sqlite_query` 返回正确的列和行数
- 验证非 SELECT 语句被拒绝

## 不做的事情（YAGNI）

- 不做数据编辑（新增/修改/删除行）
- 不做建表/改表/删表的可视化操作
- 不做索引/视图管理
- 不做导入功能
- 不做多数据库标签页（单文件操作即可）
- 不引入 CodeMirror 等 SQL 高亮编辑器
