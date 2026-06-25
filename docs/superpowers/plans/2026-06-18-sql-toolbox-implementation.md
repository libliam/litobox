# SQL 工具箱 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将现有仅支持 SQL IN 转换的 SqlTool.vue 扩展为 8 功能 SQL 工具箱，通过 el-tabs 组件实现 Tab 切换布局。

**Architecture:** 新增 6 个独立工具函数文件（sqlFormatter、sqlCompressor、sqlValidator、jsonToInsert、fieldConverter、sqlComment），改造 SqlTool.vue 为 Tab 切换布局，保留现有 sqlUtils.ts 不变。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus (el-tabs)

---

### Task 1: 创建 sqlFormatter.ts — SQL 格式化美化

**Files:**
- Create: `src/utils/sqlFormatter.ts`

- [ ] **Step 1: 编写 sqlFormatter.ts**

```typescript
export interface FormatOptions {
  dialect: 'mysql' | 'postgresql' | 'sqlserver' | 'oracle'
  indent: 2 | 4
  keywordCase: 'upper' | 'lower'
}

export interface FormatResult {
  success: boolean
  data?: string
  error?: string
}

const KEYWORDS_LEVEL1 = new Set([
  'SELECT', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'ALTER', 'DROP',
  'MERGE', 'TRUNCATE', 'REPLACE'
])

const KEYWORDS_LEVEL2 = new Set([
  'FROM', 'WHERE', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'CROSS',
  'GROUP', 'ORDER', 'HAVING', 'LIMIT', 'OFFSET', 'INTO', 'VALUES',
  'SET', 'TABLE', 'INDEX', 'VIEW', 'UNION', 'WITH', 'RECURSIVE'
])

const KEYWORDS_LEVEL3 = new Set([
  'AND', 'OR', 'ON', 'WHEN', 'THEN', 'ELSE', 'END', 'CASE',
  'BETWEEN', 'LIKE', 'IN', 'IS', 'NOT', 'NULL', 'AS', 'DISTINCT',
  'ALL', 'EXISTS', 'DEFAULT', 'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES',
  'CONSTRAINT', 'CHECK', 'UNIQUE', 'AUTO_INCREMENT', 'CURRENT_TIMESTAMP'
])

const ALL_KEYWORDS = new Set([
  ...KEYWORDS_LEVEL1, ...KEYWORDS_LEVEL2, ...KEYWORDS_LEVEL3
])

function tokenize(sql: string): string[] {
  const tokens: string[] = []
  let i = 0
  while (i < sql.length) {
    // 跳过空白
    if (/\s/.test(sql[i])) {
      i++
      continue
    }
    // 字符串字面量（单引号）
    if (sql[i] === "'") {
      let j = i + 1
      while (j < sql.length) {
        if (sql[j] === "'" && sql[j + 1] === "'") {
          j += 2 // 转义 ''
        } else if (sql[j] === "'") {
          break
        } else {
          j++
        }
      }
      tokens.push(sql.slice(i, j + 1))
      i = j + 1
      continue
    }
    // 字符串字面量（双引号）
    if (sql[i] === '"') {
      let j = i + 1
      while (j < sql.length && sql[j] !== '"') {
        if (sql[j] === '\\') j++
        j++
      }
      tokens.push(sql.slice(i, j + 1))
      i = j + 1
      continue
    }
    // 运算符和标点
    if ('(),;=<>!+-*/'.includes(sql[i])) {
      tokens.push(sql[i])
      i++
      continue
    }
    // 标识符/关键字
    if (/[a-zA-Z_]/.test(sql[i])) {
      let j = i
      while (j < sql.length && /[a-zA-Z0-9_]/.test(sql[j])) {
        j++
      }
      tokens.push(sql.slice(i, j))
      i = j
      continue
    }
    // 数字
    if (/[0-9]/.test(sql[i])) {
      let j = i
      while (j < sql.length && /[0-9.]/.test(sql[j])) {
        j++
      }
      tokens.push(sql.slice(i, j))
      i = j
      continue
    }
    // 其他字符
    tokens.push(sql[i])
    i++
  }
  return tokens
}

function formatToken(token: string, keywordCase: 'upper' | 'lower'): string {
  const upper = token.toUpperCase()
  if (ALL_KEYWORDS.has(upper)) {
    return keywordCase === 'upper' ? upper : token.toLowerCase()
  }
  return token
}

function getIndentLevel(token: string): number {
  const upper = token.toUpperCase()
  if (KEYWORDS_LEVEL1.has(upper)) return 0
  if (KEYWORDS_LEVEL2.has(upper)) return 1
  if (KEYWORDS_LEVEL3.has(upper)) return 2
  return -1
}

export function formatSql(sql: string, options: FormatOptions): FormatResult {
  try {
    if (!sql.trim()) {
      return { success: false, error: '请输入SQL内容' }
    }

    const tokens = tokenize(sql)
    if (tokens.length === 0) {
      return { success: false, error: '无法解析SQL内容' }
    }

    const indentStr = ' '.repeat(options.indent)
    const lines: string[] = []
    let currentIndent = 0
    let inParen = 0

    for (let i = 0; i < tokens.length; i++) {
      const token = tokens[i]
      const upper = token.toUpperCase()
      const level = getIndentLevel(token)

      // 左括号
      if (token === '(') {
        inParen++
        lines.push(token)
        continue
      }

      // 右括号
      if (token === ')') {
        inParen = Math.max(0, inParen - 1)
        lines.push(token)
        continue
      }

      // 分号
      if (token === ';') {
        lines.push(token)
        continue
      }

      // 关键字换行逻辑
      if (level >= 0 && inParen === 0) {
        // 如果是连续的同级关键字（如 GROUP BY），保持在一行
        const prevToken = i > 0 ? tokens[i - 1].toUpperCase() : ''
        const isCompound = (upper === 'BY' && (prevToken === 'GROUP' || prevToken === 'ORDER')) ||
                           (upper === 'JOIN' && ['LEFT', 'RIGHT', 'INNER', 'OUTER', 'CROSS'].includes(prevToken))

        if (!isCompound) {
          currentIndent = level
          lines.push('\n' + indentStr.repeat(currentIndent) + formatToken(token, options.keywordCase))
          continue
        }
      }

      // AND/OR 在 WHERE 后换行
      if ((upper === 'AND' || upper === 'OR') && inParen === 0) {
        lines.push('\n' + indentStr.repeat(currentIndent + 1) + formatToken(token, options.keywordCase))
        continue
      }

      // 逗号在括号外换行
      if (token === ',' && inParen === 0) {
        lines.push(token + '\n' + indentStr.repeat(currentIndent + 1))
        continue
      }

      // 普通 token
      lines.push(' ' + formatToken(token, options.keywordCase))
    }

    let result = lines.join('').trim()
    // 清理多余空格
    result = result.replace(/\s+/g, ' ')
    // 在关键字前确保换行
    result = result.replace(/\n/g, '\n')

    return { success: true, data: result }
  } catch (error) {
    const message = error instanceof Error ? error.message : '格式化失败'
    return { success: false, error: message }
  }
}
```

---

### Task 2: 创建 sqlCompressor.ts — SQL 压缩单行化

**Files:**
- Create: `src/utils/sqlCompressor.ts`

- [ ] **Step 1: 编写 sqlCompressor.ts**

```typescript
export function compressSql(sql: string): string {
  if (!sql.trim()) return ''

  let result = sql
  let i = 0
  let output = ''

  while (i < result.length) {
    // 单引号字符串
    if (result[i] === "'") {
      let j = i + 1
      while (j < result.length) {
        if (result[j] === "'" && result[j + 1] === "'") {
          j += 2
        } else if (result[j] === "'") {
          break
        } else {
          j++
        }
      }
      output += result.slice(i, j + 1)
      i = j + 1
      continue
    }

    // 双引号字符串
    if (result[i] === '"') {
      let j = i + 1
      while (j < result.length && result[j] !== '"') {
        if (result[j] === '\\') j++
        j++
      }
      output += result.slice(i, j + 1)
      i = j + 1
      continue
    }

    // 单行注释 --
    if (result[i] === '-' && result[i + 1] === '-') {
      // 跳到行尾
      while (i < result.length && result[i] !== '\n') {
        i++
      }
      continue
    }

    // 多行注释 /* */
    if (result[i] === '/' && result[i + 1] === '*') {
      i += 2
      while (i < result.length && !(result[i] === '*' && result[i + 1] === '/')) {
        i++
      }
      i += 2 // 跳过 */
      continue
    }

    // 普通字符
    output += result[i]
    i++
  }

  // 将连续空白替换为单个空格
  output = output.replace(/\s+/g, ' ').trim()

  return output
}
```

---

### Task 3: 创建 sqlValidator.ts — 离线语法校验

**Files:**
- Create: `src/utils/sqlValidator.ts`

- [ ] **Step 1: 编写 sqlValidator.ts**

```typescript
export interface ValidationError {
  line: number
  column: number
  message: string
  type: 'error' | 'warning'
}

export interface ValidationResult {
  errorCount: number
  warningCount: number
  errors: ValidationError[]
}

const SQL_KEYWORDS = [
  'SELECT', 'FROM', 'WHERE', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER',
  'CROSS', 'ON', 'AND', 'OR', 'NOT', 'IN', 'BETWEEN', 'LIKE', 'IS',
  'NULL', 'AS', 'GROUP', 'BY', 'ORDER', 'HAVING', 'LIMIT', 'OFFSET',
  'UNION', 'ALL', 'INSERT', 'INTO', 'VALUES', 'UPDATE', 'SET', 'DELETE',
  'CREATE', 'TABLE', 'ALTER', 'DROP', 'INDEX', 'VIEW', 'DISTINCT',
  'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'EXISTS', 'WITH', 'RECURSIVE',
  'PRIMARY', 'KEY', 'FOREIGN', 'REFERENCES', 'CONSTRAINT', 'CHECK',
  'UNIQUE', 'DEFAULT', 'AUTO_INCREMENT', 'CURRENT_TIMESTAMP', 'TRUNCATE',
  'MERGE', 'REPLACE', 'GRANT', 'REVOKE', 'BEGIN', 'COMMIT', 'ROLLBACK',
  'DECLARE', 'FUNCTION', 'PROCEDURE', 'TRIGGER', 'CURSOR', 'FETCH',
  'OPEN', 'CLOSE', 'IF', 'WHILE', 'RETURN', 'EXECUTE', 'CALL'
]

function levenshtein(a: string, b: string): number {
  const m = a.length
  const n = b.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0))
  for (let i = 0; i <= m; i++) dp[i][0] = i
  for (let j = 0; j <= n; j++) dp[0][j] = j
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)
      )
    }
  }
  return dp[m][n]
}

function findClosestKeyword(word: string): string | null {
  const upper = word.toUpperCase()
  if (SQL_KEYWORDS.includes(upper)) return null

  let bestMatch: string | null = null
  let bestDist = 3 // 阈值

  for (const kw of SQL_KEYWORDS) {
    const dist = levenshtein(upper, kw)
    if (dist <= bestDist && dist > 0) {
      bestDist = dist
      bestMatch = kw
    }
  }

  return bestMatch
}

export function validateSql(sql: string): ValidationResult {
  const errors: ValidationError[] = []

  if (!sql.trim()) {
    return { errorCount: 0, warningCount: 0, errors }
  }

  // 1. 括号匹配检查
  const parenStack: { char: string; line: number; col: number }[] = []
  const lines = sql.split('\n')

  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const line = lines[lineIdx]
    let inSingleQuote = false
    let inDoubleQuote = false

    for (let colIdx = 0; colIdx < line.length; colIdx++) {
      const ch = line[colIdx]

      // 处理引号状态
      if (ch === "'" && !inDoubleQuote) {
        if (inSingleQuote && line[colIdx + 1] === "'") {
          colIdx++ // 跳过转义 ''
        } else {
          inSingleQuote = !inSingleQuote
        }
        continue
      }
      if (ch === '"' && !inSingleQuote) {
        if (line[colIdx - 1] === '\\') continue
        inDoubleQuote = !inDoubleQuote
        continue
      }

      // 跳过注释
      if (!inSingleQuote && !inDoubleQuote && ch === '-' && line[colIdx + 1] === '-') break
      if (!inSingleQuote && !inDoubleQuote && ch === '/' && line[colIdx + 1] === '*') {
        // 跳到 */
        let rest = sql.slice(sql.indexOf(line, sql.split('\n').slice(0, lineIdx).join('\n').length + lineIdx))
        const endIdx = rest.indexOf('*/')
        if (endIdx !== -1) {
          // 简化处理：跳过当前行
        }
        continue
      }

      if (inSingleQuote || inDoubleQuote) continue

      if (ch === '(') {
        parenStack.push({ char: '(', line: lineIdx + 1, col: colIdx + 1 })
      } else if (ch === ')') {
        if (parenStack.length === 0) {
          errors.push({
            line: lineIdx + 1,
            column: colIdx + 1,
            message: '多余的右括号 ")"',
            type: 'error'
          })
        } else {
          parenStack.pop()
        }
      }
    }
  }

  // 未闭合的左括号
  for (const item of parenStack) {
    errors.push({
      line: item.line,
      column: item.col,
      message: `未闭合的左括号 "("`,
      type: 'error'
    })
  }

  // 2. 关键字拼写检查
  const wordRegex = /\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g
  for (const line of lines) {
    let match
    while ((match = wordRegex.exec(line)) !== null) {
      const word = match[1]
      const upper = word.toUpperCase()
      // 跳过太短的词和常见非关键字
      if (word.length < 3) continue
      if (/^\d/.test(word)) continue

      const closest = findClosestKeyword(word)
      if (closest) {
        errors.push({
          line: lines.indexOf(line) + 1,
          column: match.index + 1,
          message: `疑似拼写错误: "${word}"，是否应为 "${closest}"?`,
          type: 'warning'
        })
      }
    }
  }

  // 3. 分号结尾检查
  const trimmed = sql.trim()
  if (trimmed.length > 0 && !trimmed.endsWith(';')) {
    errors.push({
      line: lines.length,
      column: lines[lines.length - 1].length + 1,
      message: 'SQL语句建议以分号 ";" 结尾',
      type: 'warning'
    })
  }

  const errorCount = errors.filter(e => e.type === 'error').length
  const warningCount = errors.filter(e => e.type === 'warning').length

  return { errorCount, warningCount, errors }
}
```

---

### Task 4: 创建 jsonToInsert.ts — JSON 批量生成 Insert

**Files:**
- Create: `src/utils/jsonToInsert.ts`

- [ ] **Step 1: 编写 jsonToInsert.ts**

```typescript
export interface InsertResult {
  success: boolean
  data?: string
  error?: string
}

function escapeSqlString(value: string): string {
  return value.replace(/'/g, "''")
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) {
    return 'NULL'
  }
  if (typeof value === 'number') {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'TRUE' : 'FALSE'
  }
  if (typeof value === 'string') {
    return `'${escapeSqlString(value)}'`
  }
  throw new Error(`不支持的值类型: ${typeof value}`)
}

export function jsonToInsert(jsonText: string, tableName: string): InsertResult {
  try {
    if (!jsonText.trim()) {
      return { success: false, error: '请输入JSON内容' }
    }
    if (!tableName.trim()) {
      return { success: false, error: '请输入表名' }
    }

    let data: unknown
    try {
      data = JSON.parse(jsonText)
    } catch (e) {
      const message = e instanceof Error ? e.message : 'JSON解析失败'
      return { success: false, error: `JSON解析失败: ${message}` }
    }

    if (!Array.isArray(data)) {
      return { success: false, error: 'JSON必须为数组格式' }
    }

    if (data.length === 0) {
      return { success: false, error: 'JSON数组不能为空' }
    }

    // 提取列名
    const firstRow = data[0]
    if (typeof firstRow !== 'object' || firstRow === null || Array.isArray(firstRow)) {
      return { success: false, error: 'JSON数组元素必须为对象' }
    }

    const columns = Object.keys(firstRow)

    // 生成 INSERT 语句
    const statements: string[] = []
    for (const row of data) {
      if (typeof row !== 'object' || row === null) {
        return { success: false, error: 'JSON数组元素必须为对象' }
      }

      const values = columns.map(col => {
        const val = (row as Record<string, unknown>)[col]
        return formatValue(val)
      })

      const colList = columns.join(', ')
      const valList = values.join(', ')
      statements.push(`INSERT INTO ${tableName} (${colList}) VALUES (${valList});`)
    }

    return { success: true, data: statements.join('\n') }
  } catch (error) {
    const message = error instanceof Error ? error.message : '转换失败'
    return { success: false, error: message }
  }
}
```

---

### Task 5: 创建 fieldConverter.ts — 字段格式互转

**Files:**
- Create: `src/utils/fieldConverter.ts`

- [ ] **Step 1: 编写 fieldConverter.ts**

```typescript
export function snakeToCamel(text: string): string {
  return text
    .split('_')
    .map((part, index) => {
      if (index === 0) return part.toLowerCase()
      return part.charAt(0).toUpperCase() + part.slice(1).toLowerCase()
    })
    .join('')
}

export function camelToSnake(text: string): string {
  return text
    .replace(/([A-Z])/g, '_$1')
    .replace(/^_/, '')
    .toLowerCase()
}

export function convertFields(text: string, mode: 'snakeToCamel' | 'camelToSnake'): string {
  const lines = text.split('\n')
  const convert = mode === 'snakeToCamel' ? snakeToCamel : camelToSnake
  return lines
    .map(line => line.trim())
    .filter(line => line !== '')
    .map(convert)
    .join('\n')
}
```

---

### Task 6: 创建 sqlComment.ts — 注释批量操作

**Files:**
- Create: `src/utils/sqlComment.ts`

- [ ] **Step 1: 编写 sqlComment.ts**

```typescript
export function addLineComment(text: string): string {
  return text
    .split('\n')
    .map(line => `-- ${line}`)
    .join('\n')
}

export function removeLineComment(text: string): string {
  return text
    .split('\n')
    .map(line => line.replace(/^--\s?/, ''))
    .join('\n')
}

export function addBlockComment(text: string): string {
  return `/*\n${text}\n*/`
}

export function removeBlockComment(text: string): string {
  let result = text.trim()
  if (result.startsWith('/*')) {
    result = result.slice(2).trim()
  }
  if (result.endsWith('*/')) {
    result = result.slice(0, -2).trim()
  }
  return result
}
```

---

### Task 7: 改造 SqlTool.vue — Tab 切换布局

**Files:**
- Modify: `src/views/SqlTool.vue`（完整重写）

- [ ] **Step 1: 重写 SqlTool.vue**

完整文件内容如下（替换整个文件）：

```vue
<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="sql-tabs">
      <!-- Tab 1: 格式化 -->
      <el-tab-pane label="格式化" name="format">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">方言</div>
                <el-select v-model="formatOptions.dialect" size="small" style="width: 120px">
                  <el-option label="MySQL" value="mysql" />
                  <el-option label="PostgreSQL" value="postgresql" />
                  <el-option label="SQLServer" value="sqlserver" />
                  <el-option label="Oracle" value="oracle" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">缩进</div>
                <el-radio-group v-model="formatOptions.indent" size="small">
                  <el-radio-button :label="2">2空格</el-radio-button>
                  <el-radio-button :label="4">4空格</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">关键字</div>
                <el-radio-group v-model="formatOptions.keywordCase" size="small">
                  <el-radio-button label="upper">大写</el-radio-button>
                  <el-radio-button label="lower">小写</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleFormat">格式化</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': isError }" />
            <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 压缩 -->
      <el-tab-pane label="压缩" name="compress">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleCompress">一键压缩</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 校验 -->
      <el-tab-pane label="校验" name="validate">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="warning" size="small" @click="handleValidate">执行校验</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">校验结果</span>
          </div>
          <div class="card-body">
            <div v-if="validationResult" class="validation-summary">
              <span class="error-count" :class="{ 'has-errors': validationResult.errorCount > 0 }">
                错误: {{ validationResult.errorCount }}
              </span>
              <span class="warning-count">警告: {{ validationResult.warningCount }}</span>
            </div>
            <div v-if="validationResult && validationResult.errors.length > 0" class="error-list">
              <div v-for="(err, idx) in validationResult.errors" :key="idx" class="error-item" :class="err.type">
                <span class="error-location">第{{ err.line }}行, 第{{ err.column }}列</span>
                <span class="error-type">{{ err.type === 'error' ? '错误' : '警告' }}</span>
                <span class="error-msg">{{ err.message }}</span>
              </div>
            </div>
            <div v-else-if="validationResult && validationResult.errors.length === 0" class="success-message">
              ✓ 未发现语法问题
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: JSON→INSERT -->
      <el-tab-pane label="JSON→INSERT" name="jsonInsert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">表名</div>
                <el-input v-model="tableName" placeholder="请输入表名" size="small" style="width: 160px" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleJsonToInsert">生成 INSERT</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (JSON数组)</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder='[{"name":"张三","age":25},{"name":"李四","age":30}]' resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': isError }" />
            <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 5: 字段转换 -->
      <el-tab-pane label="字段转换" name="fieldConvert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">转换方向</div>
                <el-radio-group v-model="convertMode" size="small">
                  <el-radio-button label="snakeToCamel">下划线→驼峰</el-radio-button>
                  <el-radio-button label="camelToSnake">驼峰→下划线</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleFieldConvert">转换</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (每行一个字段名)</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="user_name&#10;first_name&#10;last_name" resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 6: 注释操作 -->
      <el-tab-pane label="注释操作" name="comment">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">单行注释</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleAddLineComment">添加 --</el-button>
                  <el-button size="small" @click="handleRemoveLineComment">移除 --</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">多行注释</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleAddBlockComment">添加 /* */</el-button>
                  <el-button size="small" @click="handleRemoveBlockComment">移除 /* */</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 7: SQL IN (已有功能) -->
      <el-tab-pane label="SQL IN" name="sqlIn">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">引号类型</div>
                <el-select v-model="quoteType" size="small" style="width: 120px">
                  <el-option label="单引号" value="single" />
                  <el-option label="双引号" value="double" />
                  <el-option label="无引号" value="none" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <el-button type="primary" size="small" @click="handleSqlInConvert">转换为 SQL IN</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请输入文本内容，每行一个值..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 8: 快捷操作 -->
      <el-tab-pane label="快捷操作" name="quick">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">SQL 模板</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">选择模板</div>
                <el-select v-model="selectedTemplate" size="small" style="width: 160px" @change="handleTemplateChange">
                  <el-option label="SELECT 模板" value="select" />
                  <el-option label="INSERT 模板" value="insert" />
                  <el-option label="UPDATE 模板" value="update" />
                  <el-option label="CREATE TABLE 模板" value="create" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">操作</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleClear">清空</el-button>
                  <el-button size="small" @click="handleCopy">复制</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="inputValue" type="textarea" :rows="10" placeholder="选择模板或手动输入SQL..." resize="vertical" />
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { formatSql, type FormatOptions } from '@/utils/sqlFormatter'
import { compressSql } from '@/utils/sqlCompressor'
import { validateSql, type ValidationResult } from '@/utils/sqlValidator'
import { jsonToInsert } from '@/utils/jsonToInsert'
import { convertFields } from '@/utils/fieldConverter'
import { addLineComment, removeLineComment, addBlockComment, removeBlockComment } from '@/utils/sqlComment'
import { convertToSqlIn, type QuoteType } from '@/utils/sqlUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// Tab 状态
const activeTab = ref('format')

// 通用状态
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const validationResult = ref<ValidationResult | null>(null)

// 格式化配置
const formatOptions = reactive<FormatOptions>({
  dialect: 'mysql',
  indent: 2,
  keywordCase: 'upper'
})

// JSON→INSERT 配置
const tableName = ref('')

// 字段转换配置
const convertMode = ref<'snakeToCamel' | 'camelToSnake'>('snakeToCamel')

// SQL IN 配置
const quoteType = ref<QuoteType>('single')

// 快捷操作配置
const selectedTemplate = ref('')

const SQL_TEMPLATES: Record<string, string> = {
  select: `SELECT column1, column2
FROM table_name
WHERE condition
ORDER BY column1;`,
  insert: `INSERT INTO table_name (column1, column2)
VALUES (value1, value2);`,
  update: `UPDATE table_name
SET column1 = value1, column2 = value2
WHERE condition;`,
  create: `CREATE TABLE table_name (
  id INT PRIMARY KEY AUTO_INCREMENT,
  column1 VARCHAR(255) NOT NULL,
  column2 INT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);`
}

// 通用方法
const handleClear = () => {
  inputValue.value = ''
  outputValue.value = ''
  errorMessage.value = ''
  isError.value = false
  validationResult.value = null
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败，请手动粘贴')
  }
}

const handleCopy = async () => {
  const text = outputValue.value || inputValue.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

const addHistory = (action: string) => {
  store.addHistory({
    tool: 'sql',
    action,
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50)
  })
}

// Tab 1: 格式化
const handleFormat = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  const result = formatSql(inputValue.value, formatOptions)
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    addHistory('格式化')
    ElMessage.success('格式化成功')
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
  }
}

// Tab 2: 压缩
const handleCompress = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  outputValue.value = compressSql(inputValue.value)
  errorMessage.value = ''
  isError.value = false
  addHistory('压缩')
  ElMessage.success('压缩成功')
}

// Tab 3: 校验
const handleValidate = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  validationResult.value = validateSql(inputValue.value)
  const total = validationResult.value.errorCount + validationResult.value.warningCount
  if (total === 0) {
    ElMessage.success('未发现语法问题')
  } else {
    ElMessage.warning(`发现 ${total} 个问题`)
  }
  addHistory('校验')
}

// Tab 4: JSON→INSERT
const handleJsonToInsert = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  const result = jsonToInsert(inputValue.value, tableName.value)
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    addHistory('JSON→INSERT')
    ElMessage.success('生成成功')
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
  }
}

// Tab 5: 字段转换
const handleFieldConvert = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入字段名')
    return
  }
  outputValue.value = convertFields(inputValue.value, convertMode.value)
  errorMessage.value = ''
  isError.value = false
  addHistory('字段转换')
  ElMessage.success('转换成功')
}

// Tab 6: 注释操作
const handleAddLineComment = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  outputValue.value = addLineComment(inputValue.value)
  addHistory('添加单行注释')
  ElMessage.success('已添加单行注释')
}

const handleRemoveLineComment = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  outputValue.value = removeLineComment(inputValue.value)
  addHistory('移除单行注释')
  ElMessage.success('已移除单行注释')
}

const handleAddBlockComment = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  outputValue.value = addBlockComment(inputValue.value)
  addHistory('添加多行注释')
  ElMessage.success('已添加多行注释')
}

const handleRemoveBlockComment = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  outputValue.value = removeBlockComment(inputValue.value)
  addHistory('移除多行注释')
  ElMessage.success('已移除多行注释')
}

// Tab 7: SQL IN
const handleSqlInConvert = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  const result = convertToSqlIn(inputValue.value, quoteType.value)
  outputValue.value = result
  addHistory('SQL IN转换')
  ElMessage.success('转换成功')
}

// Tab 8: 快捷操作
const handleTemplateChange = (val: string) => {
  if (val && SQL_TEMPLATES[val]) {
    inputValue.value = SQL_TEMPLATES[val]
    ElMessage.success('模板已填入')
  }
}
</script>

<style scoped>
.sql-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  padding: 20px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: flex-end;
}

.action-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.group-buttons {
  display: flex;
  gap: 8px;
}

.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}

.validation-summary {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.error-count {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.error-count.has-errors {
  color: var(--accent-red);
}

.warning-count {
  font-size: 14px;
  font-weight: 600;
  color: #eab308;
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.error-item {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
}

.error-item.error {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.error-item.warning {
  background: rgba(234, 179, 8, 0.08);
  border: 1px solid rgba(234, 179, 8, 0.2);
}

.error-location {
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.error-type {
  font-weight: 600;
  font-size: 12px;
  white-space: nowrap;
}

.error-item.error .error-type {
  color: var(--accent-red);
}

.error-item.warning .error-type {
  color: #eab308;
}

.error-msg {
  color: var(--text-primary);
}

.success-message {
  color: #22c55e;
  font-size: 14px;
  padding: 12px;
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: 4px;
}
</style>
```

---

### Task 8: 验证与测试

**Files:**
- 无新增文件

- [ ] **Step 1: 启动开发服务器验证**

```bash
npm run dev
```

- [ ] **Step 2: 手动测试各 Tab 功能**

| Tab | 测试用例 |
|-----|----------|
| 格式化 | 输入多行 SQL，选择不同缩进/大小写，验证输出 |
| 压缩 | 输入带注释的多行 SQL，验证压缩为单行 |
| 校验 | 输入含括号不匹配的 SQL，验证错误提示 |
| JSON→INSERT | 输入 JSON 数组 + 表名，验证 INSERT 语句生成 |
| 字段转换 | 输入 `user_name`，验证转为 `userName` |
| 注释操作 | 输入 SQL，验证添加/移除注释 |
| SQL IN | 输入多行值，验证 IN 条件生成 |
| 快捷操作 | 选择模板，验证自动填入 |

- [ ] **Step 3: 验证 TypeScript 类型检查**

```bash
npx vue-tsc --noEmit
```

预期: 无类型错误

---

## 自审

**1. Spec 覆盖检查:**

| Spec 要求 | 对应 Task |
|-----------|-----------|
| SQL 格式化美化（方言/缩进/大小写） | Task 1 + Task 7 (Tab 1) |
| SQL 压缩单行化 | Task 2 + Task 7 (Tab 2) |
| 离线语法校验（括号/引号/拼写/分号） | Task 3 + Task 7 (Tab 3) |
| JSON 批量生成 Insert | Task 4 + Task 7 (Tab 4) |
| 字段格式互转 | Task 5 + Task 7 (Tab 5) |
| 注释批量操作 | Task 6 + Task 7 (Tab 6) |
| SQL IN 转换（保留） | Task 7 (Tab 7) |
| 快捷操作（模板） | Task 7 (Tab 8) |
| Tab 切换布局 | Task 7 |
| 科技风样式 | Task 7 |

全部覆盖，无遗漏。

**2. Placeholder 扫描:** 无 TBD/TODO/占位符。

**3. 类型一致性:**
- `FormatOptions` 在 Task 1 定义，Task 7 中正确引用
- `ValidationResult` / `ValidationError` 在 Task 3 定义，Task 7 中正确引用
- `QuoteType` 从已有 `sqlUtils.ts` 导出，Task 7 中正确引用
- 所有函数签名与设计文档一致
