# SQL 工具箱设计文档

**日期**: 2026-06-18  
**主题**: SQL 工具箱 — 8 功能 Tab 切换版

## 概述

将现有仅支持 SQL IN 转换的 `SqlTool.vue` 扩展为完整的 SQL 工具箱，通过 `el-tabs` 组件提供 8 个功能 Tab，覆盖 SQL 格式化、压缩、校验、JSON 转 Insert、字段转换、注释操作等常见 SQL 开发场景。所有功能纯前端实现，无需外部 SQL 引擎，保持离线轻量。

## 架构

### 改造文件
- `src/views/SqlTool.vue` — 改造为 Tab 切换布局（8 个 Tab）

### 新增文件
- `src/utils/sqlFormatter.ts` — SQL 格式化美化
- `src/utils/sqlCompressor.ts` — SQL 压缩单行化
- `src/utils/sqlValidator.ts` — 离线语法校验
- `src/utils/jsonToInsert.ts` — JSON 批量生成 Insert
- `src/utils/fieldConverter.ts` — 字段格式互转（下划线↔驼峰）
- `src/utils/sqlComment.ts` — 注释批量操作

### 保留文件
- `src/utils/sqlUtils.ts` — 现有 SQL IN 转换功能，保持不变

### 不修改
- `src/store/index.ts` — 导航项不变
- `src/App.vue` — 路由不变

## Tab 布局

使用 `el-tabs` 组件，`tab-position="top"`，8 个 Tab：

| Tab 名称 | 功能 | 核心操作 |
|----------|------|----------|
| 格式化 | SQL 格式化美化 | 方言选择、缩进设置、关键字大小写、执行格式化 |
| 压缩 | SQL 压缩单行化 | 一键压缩 |
| 校验 | 离线语法校验 | 手动校验，显示错误数和位置 |
| JSON→INSERT | JSON 生成 Insert | 表名输入、执行转换 |
| 字段转换 | 字段格式互转 | 下划线驼峰批量转换 |
| 注释操作 | 注释批量操作 | 添加/移除单行或多行注释 |
| SQL IN | SQL IN 转换（已有） | 引号类型选择、执行转换 |
| 快捷操作 | 模板 + 快捷按钮 | SQL 模板选择、一键清空/复制 |

每个 Tab 内部遵循统一的**操作卡片 + 输入卡片 + 输出卡片**三段式布局。

## 功能详细设计

### 1. SQL 格式化美化 (`sqlFormatter.ts`)

```typescript
export interface FormatOptions {
  dialect: 'mysql' | 'postgresql' | 'sqlserver' | 'oracle'
  indent: 2 | 4
  keywordCase: 'upper' | 'lower'
}

export function formatSql(sql: string, options: FormatOptions): { success: boolean; data?: string; error?: string }
```

**配置项**：
- `dialect`: MySQL / PostgreSQL / SQLServer / Oracle（当前版本各方言输出一致，预留扩展）
- `indent`: 2 空格 / 4 空格
- `keywordCase`: 大写 / 小写

**处理逻辑**：
1. 基于正则分词，识别 SQL 关键字、标识符、字符串、运算符
2. 按关键字层级缩进：SELECT/INSERT/UPDATE/DELETE 为顶级，FROM/JOIN/WHERE/GROUP BY/ORDER BY/HAVING 为二级，AND/OR/ON 为三级
3. 关键字按 `keywordCase` 转换大小写
4. 字符串内容保持不变

**关键字表**（部分）：
```
SELECT, FROM, WHERE, JOIN, LEFT, RIGHT, INNER, OUTER, CROSS, ON,
AND, OR, NOT, IN, BETWEEN, LIKE, IS, NULL, AS, GROUP, BY, ORDER,
HAVING, LIMIT, OFFSET, UNION, ALL, INSERT, INTO, VALUES, UPDATE,
SET, DELETE, CREATE, TABLE, ALTER, DROP, INDEX, VIEW, DISTINCT,
CASE, WHEN, THEN, ELSE, END, EXISTS, WITH, RECURSIVE
```

### 2. SQL 压缩单行化 (`sqlCompressor.ts`)

```typescript
export function compressSql(sql: string): string
```

**处理逻辑**：
1. 删除 `--` 单行注释（行首或行内 `--` 到行尾）
2. 删除 `/* ... */` 多行注释
3. 将连续空白（含换行）替换为单个空格
4. 去除首尾空白

**注意**：字符串字面量内的 `--` 和 `/* */` 不应被当作注释删除（基于简单引号配对判断）。

### 3. 离线语法校验 (`sqlValidator.ts`)

```typescript
export interface ValidationError {
  line: number
  column: number
  message: string
  type: 'error' | 'warning'
}

export function validateSql(sql: string): { errorCount: number; warningCount: number; errors: ValidationError[] }
```

**检测项**：
- **括号匹配**：使用栈算法检测 `(` `)` 是否成对，不匹配时报告位置
- **引号匹配**：检测单引号 `'` 是否成对（跳过转义 `''`）
- **关键字拼写**：基于编辑距离（Levenshtein）检测常见拼写错误，如 `SELEC` → 建议 `SELECT`
- **分号结尾**：警告（非错误）— SQL 语句未以分号结尾

**实现要点**：
- 逐字符扫描，维护括号栈和引号状态
- 关键字字典约 80 个常见 SQL 关键字
- 编辑距离阈值为 2（差异 ≤2 个字符才提示）

### 4. JSON 批量生成 Insert (`jsonToInsert.ts`)

```typescript
export function jsonToInsert(jsonText: string, tableName: string): { success: boolean; data?: string; error?: string }
```

**处理逻辑**：
1. `JSON.parse` 解析输入，必须为数组
2. 提取第一条记录的键作为列名
3. 遍历每条记录，按列名顺序生成值列表
4. 值类型判断：
   - `number` → 直接输出（如 `42`）
   - `string` → 加单引号，内部 `'` 转义为 `''`（如 `'hello''world'`）
   - `null` → 输出 `NULL`
   - `boolean` → 输出 `TRUE` / `FALSE`
   - 其他类型 → 报错
5. 生成 `INSERT INTO tableName (col1, col2) VALUES (val1, val2);` 语句，每条一行

### 5. 字段格式互转 (`fieldConverter.ts`)

```typescript
export function snakeToCamel(text: string): string
export function camelToSnake(text: string): string
```

**snakeToCamel**：
- `user_name` → `userName`
- `first_name_last` → `firstNameLast`
- 按 `_` 分割，首段小写，后续段首字母大写

**camelToSnake**：
- `userName` → `user_name`
- `firstNameLast` → `first_name_last`
- 在大写字母前插入 `_`，全部转小写

**输入/输出**：每行一个字段名，批量转换

### 6. 注释批量操作 (`sqlComment.ts`)

```typescript
export function addLineComment(text: string): string
export function removeLineComment(text: string): string
export function addBlockComment(text: string): string
export function removeBlockComment(text: string): string
```

**addLineComment**：每行前加 `-- `
**removeLineComment**：移除行首的 `-- ` 或 `--`
**addBlockComment**：全文包裹 `/* ... */`
**removeBlockComment**：移除首尾的 `/*` 和 `*/`

**实现要点**：
- 由于浏览器 textarea 选区 API 限制，改为全量操作模式
- 对输入框全部内容执行注释/去注释

### 7. SQL IN 转换（已有，`sqlUtils.ts`）

保持不变，作为 Tab 之一。

### 8. 快捷操作

**SQL 模板**：
```sql
-- SELECT 模板
SELECT column1, column2
FROM table_name
WHERE condition
ORDER BY column1;

-- INSERT 模板
INSERT INTO table_name (column1, column2)
VALUES (value1, value2);

-- UPDATE 模板
UPDATE table_name
SET column1 = value1, column2 = value2
WHERE condition;

-- CREATE TABLE 模板
CREATE TABLE table_name (
  id INT PRIMARY KEY AUTO_INCREMENT,
  column1 VARCHAR(255) NOT NULL,
  column2 INT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

用户选择模板后自动填入输入框。

## 数据流

```
用户选择 Tab → 输入 SQL/JSON/字段名 → 选择配置项 → 点击执行按钮
  → 调用对应 utils 函数 → 显示结果 → 可选复制/记录历史
```

## 错误处理

- 空输入时提示"请输入内容"
- JSON 解析失败时显示具体错误信息（含行号）
- SQL 校验错误以红色列表形式展示，标注行号和错误描述
- 所有操作记录到历史（`tool: 'sql'`, `action: 具体操作名`）
- 格式化/压缩/校验失败时输出区域显示红色边框 + 错误信息

## 样式

遵循 AGENTS.md 科技风规范：
- 卡片式布局（`.tool-card`）
- CSS 变量配色（`var(--accent-cyan)` 等）
- Tab 组件使用 Element Plus 默认样式，通过 `--el-color-primary` 等变量适配主题
