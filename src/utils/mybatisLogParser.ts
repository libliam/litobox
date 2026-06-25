// 参数类型枚举
export type ParamType = 'String' | 'Integer' | 'Long' | 'Double' | 'Float' | 'Boolean' | 'Date' | 'null' | 'Unknown'

// 解析结果接口
export interface ParseResult {
  success: boolean
  sql?: string
  error?: string
}

// 类型关键词映射
const STRING_TYPES = new Set(['String', 'VARCHAR', 'TEXT', 'CHAR', 'NVARCHAR', 'CLOB'])
const NUMBER_TYPES = new Set(['Integer', 'Long', 'Short', 'Byte', 'Double', 'Float', 'BigDecimal', 'BigInteger', 'int', 'long', 'short', 'byte', 'double', 'float'])
const BOOLEAN_TYPES = new Set(['Boolean', 'boolean'])
const DATE_TYPES = new Set(['Date', 'Timestamp', 'LocalDateTime', 'LocalDate', 'Time', 'java.util.Date', 'java.sql.Timestamp', 'java.sql.Date'])

// 根据类型格式化参数值
function formatParamValue(value: string, type: string): string {
  // null 类型
  if (type === 'null' || value.toLowerCase() === 'null') {
    return 'NULL'
  }

  // 布尔类型
  if (BOOLEAN_TYPES.has(type)) {
    return value.toLowerCase() === 'true' ? 'true' : 'false'
  }

  // 数字类型
  if (NUMBER_TYPES.has(type)) {
    return value
  }

  // 日期类型
  if (DATE_TYPES.has(type)) {
    return `'${value}'`
  }

  // 字符串类型
  if (STRING_TYPES.has(type)) {
    return `'${value}'`
  }

  // 未知类型默认加引号
  return `'${value}'`
}

// 解析 Parameters 行，提取参数列表
// 支持两种格式：
// 1. "Parameters: alertEnable(String), 0(Integer)"
// 2. "@ts~... @msg~[]==> Parameters: alertEnable(String), 0(Integer)"
function parseParameters(paramsLine: string): Array<{ value: string; type: string }> {
  const params: Array<{ value: string; type: string }> = []
  // 提取 Parameters: 之后的内容
  const match = paramsLine.match(/Parameters:\s*(.*)/)
  if (!match) return params
  const content = match[1].trim()
  if (!content) return params

  // 按逗号分割，但要注意括号内的逗号不能分割
  const regex = /([^,]+?)\(([^)]+)\)|\s*([^,]+?)\s*(?=,|$)/g
  let m
  while ((m = regex.exec(content)) !== null) {
    if (m[1] && m[2]) {
      // 匹配到 value(Type) 格式
      params.push({ value: m[1].trim(), type: m[2].trim() })
    } else if (m[3]) {
      // 匹配到单独的 null 等
      const val = m[3].trim()
      if (val) {
        params.push({ value: val, type: 'null' })
      }
    }
  }
  return params
}

// 从 MyBatis 日志中解析 SQL 和参数，生成完整 SQL
export function parseMybatisLog(logText: string): ParseResult {
  const lines = logText.split('\n').map(line => line.trim()).filter(line => line.length > 0)

  // 查找 Preparing 行
  const preparingLine = lines.find(line => line.startsWith('Preparing:'))
  if (!preparingLine) {
    return { success: false, error: '未找到 Preparing 语句，请确保日志包含 MyBatis SQL 日志' }
  }

  // 提取 SQL 模板（去掉 "Preparing: " 前缀）
  let sqlTemplate = preparingLine.replace(/^Preparing:\s*/, '').trim()

  // 查找 Parameters 行（可能包含噪声前缀，如 "@ts~... ==> Parameters:"）
  const paramsLine = lines.find(line => line.includes('Parameters:'))
  if (!paramsLine) {
    return { success: false, error: '未找到 Parameters 参数行' }
  }

  // 解析参数
  const params = parseParameters(paramsLine)
  if (params.length === 0) {
    return { success: false, error: '未解析到任何参数' }
  }

  // 替换占位符
  let paramIndex = 0
  let resultSql = sqlTemplate.replace(/\?/g, () => {
    if (paramIndex >= params.length) {
      return '?' // 参数不足，保留原样
    }
    const param = params[paramIndex++]
    return formatParamValue(param.value, param.type)
  })

  // 确保 SQL 以分号结尾，方便直接执行
  if (!resultSql.trim().endsWith(';')) {
    resultSql = resultSql.trim() + ';'
  }

  return { success: true, sql: resultSql }
}
