# 工作流编排 + 变量池 + SQLite 迁移设计文档

**日期**: 2026-06-26
**状态**: 待实现

---

## 一、需求概述

在 LitoBox 项目中新增四个核心功能：

1. **链式数据流转**：支持工具间数据传递，上一步输出作为下一步输入
2. **自定义工作流**：保存连续处理步骤，下次一键执行
3. **全局变量缓存池**：跨工具共享临时变量，支持手动命名和自动捕获
4. **全量数据导出/导入**：导出所有数据为 JSON 备份，换电脑可导入恢复

同时将所有数据从 localStorage 迁移到 SQLite，为后续大数据量功能（如大量历史记录、大批量文件处理结果等）做准备。

---

## 二、架构设计

### 2.1 整体架构

```
┌──────────────────────────────────────────────────┐
│  前端 Vue (TypeScript)                            │
│  ├── store/index.ts          # Pinia 状态         │
│  ├── utils/dbClient.ts       # Tauri 命令封装      │
│  └── views/                                      │
│      ├── WorkflowView.vue    # 工作流 + 变量池     │
│      └── HistoryView.vue     # 导出/导入按钮       │
├──────────────────────────────────────────────────┤
│  Tauri Bridge (invoke)                            │
├──────────────────────────────────────────────────┤
│  Rust 后端 (src-tauri/)                           │
│  ├── src/db.rs               # SQLite 操作层       │
│  └── src/main.rs             # 命令注册            │
└──────────────────────────────────────────────────┘
```

### 2.2 数据流向

```
工具操作 → addHistory() → db_add_history() → SQLite
         → captureVariable() → db_set_variable() → SQLite

工作流执行 → resolveInput(step) → 获取上一步输出/变量池 → 执行工具 → 捕获输出
```

---

## 三、数据库设计

### 3.1 数据库文件

路径：`%APPDATA%/com.dev.toolbox/litobox.db`

### 3.2 表结构

```sql
-- 配置表（KV 存储）
CREATE TABLE IF NOT EXISTS config (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- 操作历史表
CREATE TABLE IF NOT EXISTS history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tool TEXT NOT NULL,
  action TEXT NOT NULL,
  input_preview TEXT,
  output_preview TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_history_created ON history(created_at DESC);

-- 最近工具表
CREATE TABLE IF NOT EXISTS recent_tools (
  tool_id TEXT PRIMARY KEY,
  last_used_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 工作流表
CREATE TABLE IF NOT EXISTS workflows (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  steps_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 变量池表
CREATE TABLE IF NOT EXISTS variable_pool (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  value TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'manual',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_used_at TEXT
);
CREATE INDEX idx_variable_name ON variable_pool(name);

-- 代码片段表
CREATE TABLE IF NOT EXISTS snippets (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  category TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 3.3 数据模型

**WorkflowStep**
```typescript
interface WorkflowStep {
  id: string
  toolId: string          // 对应 TOOL_LIST 中的工具 ID
  params: Record<string, any>
  inputSource: 'manual' | 'prev' | 'variable' | 'expression'
  inputRef: string        // 变量名或表达式（inputSource 为 variable/expression 时使用）
}
```

**Workflow**
```typescript
interface Workflow {
  id: string
  name: string
  description: string
  steps: WorkflowStep[]
  createdAt: string
  updatedAt: string
}
```

**PoolVariable**
```typescript
interface PoolVariable {
  id: string
  name: string
  value: string
  source: 'manual' | 'auto'
  createdAt: string
  lastUsedAt?: string
}
```

---

## 四、Tauri 命令设计

### 4.1 配置

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_get_config` | `key: String` | `String` | 获取配置值 |
| `db_set_config` | `key: String, value: String` | `()` | 设置配置值 |

### 4.2 历史记录

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_add_history` | `record: HistoryRecord` | `i64` | 添加历史记录，返回 ID |
| `db_get_history` | `limit: i64, offset: i64` | `Vec<HistoryRecord>` | 分页查询历史 |
| `db_clear_history` | `()` | `()` | 清空历史 |
| `db_search_history` | `query: String, limit: i64` | `Vec<HistoryRecord>` | 搜索历史 |

### 4.3 工作流

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_list_workflows` | `()` | `Vec<Workflow>` | 列出所有工作流 |
| `db_save_workflow` | `workflow: Workflow` | `()` | 保存/更新工作流 |
| `db_delete_workflow` | `id: String` | `()` | 删除工作流 |

### 4.4 变量池

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_list_variables` | `()` | `Vec<PoolVariable>` | 列出所有变量 |
| `db_set_variable` | `name: String, value: String, source: String` | `()` | 设置变量 |
| `db_delete_variable` | `name: String` | `()` | 删除变量 |
| `db_get_variable` | `name: String` | `String` | 获取变量值 |

### 4.5 导入导出

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_export_all` | `()` | `String` | 导出全量 JSON |
| `db_import_all` | `data: String` | `()` | 导入全量 JSON |

### 4.6 迁移

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `db_migrate_from_localstorage` | `data: String` | `()` | 从 localStorage 数据迁移 |
| `db_check_migrated` | `()` | `bool` | 检查是否已迁移 |

---

## 五、前端实现

### 5.1 dbClient.ts

封装所有 Tauri 调用，提供 Promise API：

```typescript
import { invoke } from '@tauri-apps/api/core'

export const db = {
  // Config
  getConfig: (key: string) => invoke<string>('db_get_config', { key }),
  setConfig: (key: string, value: string) => invoke('db_set_config', { key, value }),

  // History
  addHistory: (record: HistoryRecord) => invoke('db_add_history', { record }),
  getHistory: (limit = 100, offset = 0) => invoke<HistoryRecord[]>('db_get_history', { limit, offset }),
  clearHistory: () => invoke('db_clear_history'),
  searchHistory: (query: string, limit = 50) => invoke<HistoryRecord[]>('db_search_history', { query, limit }),

  // Workflows
  listWorkflows: () => invoke<Workflow[]>('db_list_workflows'),
  saveWorkflow: (workflow: Workflow) => invoke('db_save_workflow', { workflow }),
  deleteWorkflow: (id: string) => invoke('db_delete_workflow', { id }),

  // Variables
  listVariables: () => invoke<PoolVariable[]>('db_list_variables'),
  setVariable: (name: string, value: string, source = 'manual') => invoke('db_set_variable', { name, value, source }),
  deleteVariable: (name: string) => invoke('db_delete_variable', { name }),
  getVariable: (name: string) => invoke<string>('db_get_variable', { name }),

  // Import/Export
  exportAll: () => invoke<string>('db_export_all'),
  importAll: (data: string) => invoke('db_import_all', { data }),

  // Migration
  migrateFromLocalStorage: (data: string) => invoke('db_migrate_from_localstorage', { data }),
  checkMigrated: () => invoke<boolean>('db_check_migrated'),
}
```

### 5.2 Store 改造

`src/store/index.ts` 改为从 `dbClient` 加载数据：

```typescript
// 初始化
const loadFromDB = async () => {
  const migrated = await db.checkMigrated()
  if (!migrated) {
    // 从 localStorage 迁移
    const lsData = {
      config: localStorage.getItem(STORAGE_KEY_CONFIG),
      history: localStorage.getItem(STORAGE_KEY_HISTORY),
      recent: localStorage.getItem(STORAGE_KEY_RECENT),
    }
    await db.migrateFromLocalStorage(JSON.stringify(lsData))
  }

  // 从 DB 加载
  config.value = parseConfig(await db.getConfig('main'))
  history.value = await db.getHistory(MAX_HISTORY)
  // ...
}

// 操作时同步写 DB
const addHistory = async (record) => {
  await db.addHistory(record)
  history.value.unshift(record)
  // 自动捕获到变量池
  captureAutoVariable(record)
}
```

### 5.3 WorkflowView.vue 页面布局

```
┌─────────────────────────────────────────────────────┐
│  [工作流列表]        │ [步骤编排区]      │ [变量池]   │
│                     │                   │           │
│  + 新建工作流        │  步骤 1:          │  手动变量  │
│  ────────────       │    [工具下拉]      │  + 添加   │
│  工作流卡片 1        │    [输入来源]      │  ─────── │
│  工作流卡片 2        │    [参数配置]      │  自动缓存 │
│                     │    [删除]          │  auto_01  │
│  [搜索]             │                   │  auto_02  │
│                     │  步骤 2:          │           │
│                     │    ...            │  [清空]    │
│                     │                   │           │
│  [执行全部] [保存]   │  + 添加步骤        │           │
└─────────────────────────────────────────────────────┘
```

**核心交互：**
1. 左侧列表选择/新建工作流
2. 中间添加步骤，每步选择工具、配置输入来源（上一步/变量/手动/表达式）
3. 点击「执行全部」顺序执行所有步骤，每步输出显示在步骤下方
4. 右侧变量池显示所有变量，支持复制变量名、手动添加、删除
5. 工作流保存时，步骤以 JSON 格式存入 `workflows.steps_json`

### 5.4 HistoryView 导出/导入

在现有操作栏添加：

```vue
<el-button size="small" @click="handleExport">导出备份</el-button>
<el-button size="small" @click="handleImport">导入恢复</el-button>
```

- **导出**：调用 `db.exportAll()`，生成 `litobox-backup-YYYYMMDD.json` 下载
- **导入**：选择 JSON 文件 → 弹窗确认（覆盖/合并）→ 调用 `db.importAll()`

---

## 六、文件清单

### 6.1 新增文件

| 文件 | 说明 |
|------|------|
| `src-tauri/src/db.rs` | SQLite 初始化 + 所有 CRUD 操作 + 迁移逻辑 |
| `src/utils/dbClient.ts` | 前端 Tauri 命令封装 |
| `src/views/WorkflowView.vue` | 工作流编排 + 变量池面板 |

### 6.2 修改文件

| 文件 | 说明 |
|------|------|
| `src-tauri/Cargo.toml` | 新增 `rusqlite = { version = "0.32", features = ["bundled"] }`、`dirs = "5.0"` |
| `src-tauri/src/main.rs` | 注册 `mod db;` + 所有 Tauri 命令 |
| `src/store/index.ts` | 改为从 `dbClient` 加载/保存，添加自动变量捕获 |
| `src/views/HistoryView.vue` | 添加全量导出/导入按钮 |
| `src/App.vue` | 添加 WorkflowView 路由/入口 |

---

## 七、性能与安全

### 7.1 性能

- SQLite 使用默认 WAL 模式，读写不阻塞
- 历史记录表 `created_at` 降序索引，分页查询高效
- 变量池 `name` UNIQUE 索引，查询 O(1)
- 工作流步骤以 JSON 存储，避免多表 JOIN
- 大文本（>10KB）的输入输出只存预览，完整内容存变量池

### 7.2 安全

- 数据库文件存储在 `%APPDATA%`，仅当前用户可访问
- 所有 SQL 使用参数化查询，无 SQL 注入风险
- 导入数据时校验 JSON 格式和字段类型

### 7.3 边界情况

- 变量池自动缓存最多保留 20 条，超出自动清理最旧的
- 历史记录最多 100 条（与现有逻辑一致）
- 工作流步骤最多 20 步（防止循环过长导致性能问题）
- 导入时如果变量名冲突，弹窗让用户选择覆盖/跳过/重命名

---

## 八、迁移策略

### 8.1 首次启动检测

1. 检查 `litobox.db` 是否存在
2. 如果不存在，检查 localStorage 是否有数据
3. 如果有 → 执行迁移 → 标记 `db_migrated = true`
4. 后续启动直接读 SQLite

### 8.2 迁移流程

```
前端收集 localStorage 数据
  → JSON 序列化
  → 调用 db_migrate_from_localstorage(data)
  → Rust 解析 JSON → 写入 SQLite 各表
  → 返回成功
  → 前端清除 localStorage（可选）
```

### 8.3 回滚方案

迁移完成后，localStorage 数据保留 7 天不清除，用户可手动恢复。

---

## 八、工作流执行引擎

工作流执行在**前端**完成，不在 Rust 层。原因：
- 各工具的逻辑已在前端 utils 中实现（如 `jsonUtils.ts`、`stringUtils.ts`）
- 前端按步骤顺序调用对应工具函数，将上一步输出作为下一步输入
- Rust 层只负责数据持久化，不参与执行逻辑

执行流程：
```
用户点击「执行全部」
  → 读取工作流步骤列表
  → for each step:
      → resolveInput(step, prevOutput)  // 解析输入来源
      → 调用对应工具函数（如 formatJSON(input)）
      → 捕获输出到 prevOutput
      → 更新 UI 显示当前步骤结果
  → 最终输出 = 最后一步的 prevOutput
```

工具映射：通过 `toolId` 映射到前端工具函数，维护一个 `TOOL_EXECUTORS` 字典：
```typescript
const TOOL_EXECUTORS: Record<string, (input: string, params: any) => string | Promise<string>> = {
  json: (input, params) => formatJSON(input, params),
  string: (input, params) => transformString(input, params),
  encode: (input, params) => encodeData(input, params),
  // ... 按需注册
}
```

## 九、未来扩展

- 工作流支持条件分支（if/else）和循环（for each line）
- 变量池支持类型系统（string/number/json/array）
- 历史记录支持全文搜索（FTS5 虚拟表）
- 数据同步支持（多设备间 SQLite 同步）
