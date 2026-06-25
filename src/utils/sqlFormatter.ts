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
          j += 2
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
      // 点号特殊处理：与前后标识符紧贴，不加空格
      if (sql[i] === '.') {
        // 检查前一个 token 是否是标识符/数字
        const prevToken = tokens.length > 0 ? tokens[tokens.length - 1] : ''
        const isPrevIdent = /[a-zA-Z0-9_'"]$/.test(prevToken)
        // 检查后一个字符是否是标识符
        const nextChar = sql[i + 1] || ''
        const isNextIdent = /[a-zA-Z0-9_]/.test(nextChar)

        if (isPrevIdent && isNextIdent) {
          // 标识符.标识符 模式，紧贴处理
          tokens[tokens.length - 1] = prevToken + '.'
          // 预读并吞并后面的标识符
          let j = i + 1
          while (j < sql.length && /[a-zA-Z0-9_]/.test(sql[j])) {
            j++
          }
          tokens[tokens.length - 1] = tokens[tokens.length - 1] + sql.slice(i + 1, j)
          i = j
          continue
        }
      }
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
    // 按行清理多余空格，但保留换行
    result = result.split('\n').map(line => line.replace(/\s+/g, ' ').trim()).join('\n')

    return { success: true, data: result }
  } catch (error) {
    const message = error instanceof Error ? error.message : '格式化失败'
    return { success: false, error: message }
  }
}
