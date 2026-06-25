# MyBatis 日志解析功能设计

## 功能概述

新增第 9 个 Tab：**日志解析**，用于从 MyBatis 日志中提取 SQL 和参数，替换 `?` 占位符生成可执行的完整 SQL。

## 支持的 SQL 类型

- **SELECT** — `Preparing: SELECT ... WHERE ... = ?` + `Parameters: xxx(String), yyy(Integer)`
- **INSERT** — `Preparing: INSERT INTO ... VALUES (?, ?, ?)` + `Parameters: ...`
- **UPDATE** — `Preparing: UPDATE ... SET ... = ? WHERE ... = ?` + `Parameters: ...`
- **DELETE** — `Preparing: DELETE FROM ... WHERE ... = ?` + `Parameters: ...`

## 参数类型自动识别规则

| 日志中的类型标记 | 处理方式 |
|---|---|
| `String` / `VARCHAR` / `TEXT` | 加单引号 `'value'` |
| `Integer` / `Long` / `Short` / `Byte` | 不加引号，直接输出 |
| `Double` / `Float` / `BigDecimal` | 不加引号，直接输出 |
| `Boolean` | 转为 `true` / `false`（不加引号） |
| `null` | 转为 `NULL` |
| `Date` / `Timestamp` / `LocalDateTime` | 加单引号 `'yyyy-MM-dd HH:mm:ss'` |
| 未知类型 | 默认加单引号 |

## 日志解析逻辑

1. 按行分割输入文本
2. 提取 `Preparing:` 开头的行 → 获取 SQL 模板
3. 提取 `Parameters:` 开头的行 → 解析参数列表
4. 过滤掉时间戳、线程名、类路径等噪声行
5. 按顺序将参数替换到 SQL 的 `?` 占位符中

## UI 布局

- **输入框**：粘贴完整的多行日志
- **输出框**：显示替换后的完整 SQL（可复制）
- **操作按钮**：清空、粘贴、复制

## 新建文件

- `src/utils/mybatisLogParser.ts` — 日志解析核心逻辑

## 修改文件

- `src/views/SqlTool.vue` — 新增第 9 个 Tab
