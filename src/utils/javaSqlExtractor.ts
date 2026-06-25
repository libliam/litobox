/**
 * 从 Java 代码中提取完整 SQL 语句
 * 功能：提取字符串拼接的 SQL，并将 :paramName 占位符替换为实际参数名
 */

interface ExtractResult {
  success: boolean
  sql?: string
  error?: string
}

/**
 * 从 Java 代码片段中提取 SQL 字符串并替换占位符
 */
export function extractSqlFromJava(javaCode: string): ExtractResult {
  try {
    const trimmed = javaCode.trim()
    if (!trimmed) {
      return { success: false, error: '请输入 Java 代码' }
    }

    // 1. 提取 SQL 字符串拼接部分
    const sqlString = extractSqlString(trimmed)
    if (!sqlString) {
      return { success: false, error: '未找到 SQL 字符串定义' }
    }

    // 2. 提取参数映射
    const paramMap = extractParamMap(trimmed)

    // 3. 替换占位符并添加分号
    const finalSql = replacePlaceholders(sqlString, paramMap) + ';'

    return { success: true, sql: finalSql }
  } catch (err: any) {
    return { success: false, error: `解析失败: ${err.message}` }
  }
}

/**
 * 提取 SQL 字符串（支持多行拼接）
 * Java 字符串拼接格式：
 *   String sql = "select ... \n" +
 *                "where ... \n" +
 *                "...";
 */
function extractSqlString(code: string): string {
  const lines = code.split('\n')
  const sqlParts: string[] = []
  let inSqlString = false

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const trimmedLine = line.trim()

    if (!inSqlString) {
      // 查找 SQL 字符串的开始：匹配双引号包裹的 SQL 关键字
      // 模式: "select ... 或 = "select ...
      const startMatch = trimmedLine.match(/=\s*"((?:select|insert|update|delete|with)\b[\s\S]*)/i)
      if (startMatch) {
        inSqlString = true
        // 提取双引号内的内容（从 SQL 关键字开始到引号结束）
        const quoteContent = extractDoubleQuotedContent(trimmedLine)
        if (quoteContent !== null) {
          sqlParts.push(quoteContent)
        }
      }
    } else {
      // 已经在 SQL 字符串内部
      // 提取双引号内容
      const quoteContent = extractDoubleQuotedContent(trimmedLine)
      if (quoteContent !== null) {
        sqlParts.push(quoteContent)
      }

      // 判断是否是最后一行（以 "; 结尾）
      if (/"\s*;?\s*$/.test(trimmedLine) && !trimmedLine.includes(' +')) {
        break
      }
    }
  }

  if (sqlParts.length === 0) {
    return ''
  }

  // 合并 SQL 片段
  let sql = sqlParts.join('')

  // 清理 \n 转义字符和多余空白
  sql = sql.replace(/\\n/g, ' ').replace(/\s+/g, ' ').trim()

  return sql
}

/**
 * 从行中提取双引号内的完整内容
 * 优先匹配最外层的双引号，忽略内部的单引号
 */
function extractDoubleQuotedContent(line: string): string | null {
  // 找到第一个双引号和最后一个双引号之间的内容
  const firstQuote = line.indexOf('"')
  const lastQuote = line.lastIndexOf('"')

  if (firstQuote === -1 || firstQuote === lastQuote) {
    return null
  }

  return line.substring(firstQuote + 1, lastQuote)
}

/**
 * 提取参数映射
 * 支持格式：
 * - MapKit.mapOf("key1", value1, "key2", value2)
 * - Map.of("key1", value1, "key2", value2)
 * - new HashMap() {{ put("key", value); }}
 */
function extractParamMap(code: string): Record<string, string> {
  const paramMap: Record<string, string> = {}

  // 匹配 MapKit.mapOf 或 Map.of 格式
  const mapOfMatch = code.match(/(?:MapKit|Map)\.mapOf\s*\(([\s\S]*?)\)\s*[);]/)
  if (mapOfMatch) {
    const args = mapOfMatch[1]
    // 提取 "key", value 对
    const pairs = args.match(/"([^"]+)"\s*,\s*(\w+)/g)
    if (pairs) {
      for (const pair of pairs) {
        const match = pair.match(/"([^"]+)"\s*,\s*(\w+)/)
        if (match) {
          paramMap[match[1]] = match[2]
        }
      }
    }
    return paramMap
  }

  // 匹配 new HashMap() {{ put("key", value); }} 格式
  const putMatches = code.match(/put\s*\(\s*"([^"]+)"\s*,\s*(\w+)\s*\)/g)
  if (putMatches) {
    for (const put of putMatches) {
      const match = put.match(/put\s*\(\s*"([^"]+)"\s*,\s*(\w+)\s*\)/)
      if (match) {
        paramMap[match[1]] = match[2]
      }
    }
    return paramMap
  }

  return paramMap
}

/**
 * 替换 SQL 中的占位符
 * :paramName → 实际的参数值
 */
function replacePlaceholders(sql: string, paramMap: Record<string, string>): string {
  let result = sql

  // 替换 :paramName 格式的占位符
  result = result.replace(/:(\w+)/g, (match, paramName) => {
    // 优先使用参数映射中的值，用单引号包裹当作字符串
    if (paramMap[paramName]) {
      return `'${paramMap[paramName]}'`
    }
    // 如果没有映射，保留原占位符
    return match
  })

  return result
}
