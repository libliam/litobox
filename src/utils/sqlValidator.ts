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
  let bestDist = 3

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

      if (ch === "'" && !inDoubleQuote) {
        if (inSingleQuote && line[colIdx + 1] === "'") {
          colIdx++
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

      if (!inSingleQuote && !inDoubleQuote && ch === '-' && line[colIdx + 1] === '-') break

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

  for (const item of parenStack) {
    errors.push({
      line: item.line,
      column: item.col,
      message: '未闭合的左括号 "("',
      type: 'error'
    })
  }

  // 2. 关键字拼写检查
  const wordRegex = /\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g
  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const line = lines[lineIdx]
    let match
    while ((match = wordRegex.exec(line)) !== null) {
      const word = match[1]
      if (word.length < 3) continue

      const closest = findClosestKeyword(word)
      if (closest) {
        errors.push({
          line: lineIdx + 1,
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
