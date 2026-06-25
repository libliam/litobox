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

    const firstRow = data[0]
    if (typeof firstRow !== 'object' || firstRow === null || Array.isArray(firstRow)) {
      return { success: false, error: 'JSON数组元素必须为对象' }
    }

    const columns = Object.keys(firstRow)

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
