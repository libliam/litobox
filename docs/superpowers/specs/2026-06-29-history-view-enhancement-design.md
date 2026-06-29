# 历史记录功能扩展 - 设计文档

**日期**: 2026-06-29
**状态**: 待用户审批

## 概述

扩展历史记录功能，支持双击跳转到对应工具页面，自动填充当时的完整输入/输出和操作配置。为此需要扩展数据库结构，将大文本内容分离到独立附件表，避免主表膨胀。

## 架构设计

### 1. 数据库变更

#### 1.1 新建 `history_details` 表

```sql
CREATE TABLE IF NOT EXISTS history_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id INTEGER NOT NULL,
    input_full TEXT,
    output_full TEXT,
    options_json TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (history_id) REFERENCES history(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_history_details_history_id ON history_details(history_id);
```

#### 1.2 history 表扩展

```sql
ALTER TABLE history ADD COLUMN detail_id INTEGER;
```

#### 1.3 数据写入流程

```
工具执行 → INSERT history (tool/action/input_preview/output_preview) → last_insert_rowid()
         → INSERT history_details (history_id/input_full/output_full/options_json) → last_insert_rowid()
         → UPDATE history SET detail_id = ? WHERE id = ?
```

#### 1.4 导入导出兼容

- 导出：LEFT JOIN `history_details`，将完整数据合并到 JSON
- 导入：先恢复 history 记录，再恢复 details，最后关联 `detail_id`

### 2. Pinia Store 状态传递

#### 2.1 新增状态

```typescript
interface HistoryRestoreState {
  tool: string
  input: string
  output: string
  options: Record<string, any>
  timestamp: string
}

// Store 新增
const pendingHistoryRestore = ref<HistoryRestoreState | null>(null)
const triggerHistoryRestore = (data: HistoryRestoreState) => { pendingHistoryRestore.value = data }
const clearHistoryRestore = () => { pendingHistoryRestore.value = null }
```

#### 2.2 传递流程

```
HistoryView 双击
  → dbClient.getHistoryDetail(id) 获取完整数据
  → store.triggerHistoryRestore(data)
  → store.activeTool = tool（切换页面）
  → 目标页面 onMounted 检测 pendingHistoryRestore → 填充 → clear
```

#### 2.3 过期清理

Store 中添加 watcher：如果 `pendingHistoryRestore` 超过 30 秒未被消费，自动清除。

### 3. UI 交互

#### 3.1 HistoryView 列表

- 历史列表项增加 `cursor: pointer` 和 hover 高亮
- Tooltip 提示"双击跳转到对应工具"
- 双击事件：`@dblclick="handleJumpToTool(record)"`
- 大文本记录 preview 显示 `[大文本 · 双击查看]`

#### 3.2 跳转后提示条

工具页面顶部显示临时提示条（3 秒自动消失）：
```
ℹ️ 已加载历史记录（2026-06-29 14:32 的操作）
   输入和输出已填充，配置已还原
```
样式：浅青色背景，与工具卡片风格一致。

#### 3.3 多 Tab 工具页面

`options_json` 中记录 `activeTab`，跳转时自动切换到对应 Tab。

#### 3.4 工具不存在

弹出提示："该工具当前不可用"。

### 4. 错误处理与边界

| 场景 | 处理方式 |
|------|----------|
| 大文本 | 不限制大小，preview 超过 10KB 显示 `[大文本 · 双击查看]` |
| 清空历史 | CASCADE 删除关联 details |
| 目标页面无 restore 逻辑 | 30 秒后自动清除状态 |
| options_json 解析失败 | 忽略 options 还原，只还原 input/output |
| 工具不存在 | 提示"该工具当前不可用" |

## 数据模型

### HistoryRecord（扩展后）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| tool | TEXT | 工具标识 |
| action | TEXT | 操作名称 |
| input_preview | TEXT | 输入预览（截断） |
| output_preview | TEXT | 输出预览（截断） |
| detail_id | INTEGER | 关联 details 表 |
| created_at | DATETIME | 时间戳 |

### HistoryDetail（新表）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| history_id | INTEGER | 关联 history.id |
| input_full | TEXT | 完整输入 |
| output_full | TEXT | 完整输出 |
| options_json | TEXT | 操作配置（JSON） |
| created_at | DATETIME | 时间戳 |

## 影响范围

### 后端（Rust）

- `db.rs`：新增 `history_details` 表创建、CRUD 函数
- 导入导出逻辑适配

### 前端

- `src/utils/dbClient.ts`：新增 `getHistoryDetail` 等函数
- `src/store/index.ts`：新增 `pendingHistoryRestore` 状态
- `src/views/HistoryView.vue`：双击跳转逻辑
- 各工具页面：按需添加 `restoreFromHistory` 逻辑（可逐步实现）

## 实施优先级

1. **P0**：数据库结构变更 + Rust CRUD + dbClient 封装
2. **P0**：HistoryView 双击跳转 + Store 状态传递
3. **P1**：核心工具页面 restore 逻辑（JSON/字符串/编码/正则/SQL/JS）
4. **P2**：其余工具页面 restore 逻辑
