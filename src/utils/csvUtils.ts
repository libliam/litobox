export interface CsvParseOptions {
  delimiter: ',' | ';' | '\t' | '|'
  hasHeader: boolean
}

export interface CsvData {
  headers: string[]
  rows: string[][]
}

/**
 * 解析 CSV 文本为结构化数据
 * 处理引号转义、分隔符内换行等边界情况
 */
export function parseCsv(text: string, options: CsvParseOptions): CsvData {
  const { delimiter, hasHeader } = options
  const rows: string[][] = []
  let current = ''
  let inQuotes = false
  let row: string[] = []

  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    const next = text[i + 1]

    if (inQuotes) {
      if (char === '"' && next === '"') {
        current += '"'
        i++ // 跳过下一个引号
      } else if (char === '"') {
        inQuotes = false
      } else {
        current += char
      }
    } else {
      if (char === '"') {
        inQuotes = true
      } else if (char === delimiter) {
        row.push(current.trim())
        current = ''
      } else if (char === '\n' || (char === '\r' && next === '\n')) {
        row.push(current.trim())
        if (row.length > 1 || row[0] !== '') {
          rows.push(row)
        }
        row = []
        current = ''
        if (char === '\r') i++ // 跳过 \n
      } else {
        current += char
      }
    }
  }

  // 处理最后一行
  row.push(current.trim())
  if (row.length > 1 || row[0] !== '') {
    rows.push(row)
  }

  if (rows.length === 0) {
    return { headers: [], rows: [] }
  }

  if (hasHeader) {
    return { headers: rows[0], rows: rows.slice(1) }
  }

  // 无 header 时生成默认列名
  const maxCols = Math.max(...rows.map(r => r.length))
  const headers = Array.from({ length: maxCols }, (_, i) => `列${i + 1}`)
  return { headers, rows }
}

/**
 * 将 CSV 数据导出为 JSON 数组
 */
export function csvToJson(csvData: CsvData): string {
  if (csvData.rows.length === 0) return '[]'
  const result = csvData.rows.map(row => {
    const obj: Record<string, string> = {}
    csvData.headers.forEach((header, i) => {
      obj[header] = row[i] ?? ''
    })
    return obj
  })
  return JSON.stringify(result, null, 2)
}

/**
 * 将 CSV 数据导出为 SQL INSERT 语句
 */
export function csvToSql(csvData: CsvData, tableName: string): string {
  if (!tableName) return '错误: 请输入表名'
  if (csvData.rows.length === 0) return '-- 无数据'

  const columns = csvData.headers.map(h => `\`${h}\``).join(', ')
  const inserts = csvData.rows.map(row => {
    const values = row.map(v => `'${v.replace(/'/g, "''")}'`).join(', ')
    return `(${values})`
  })

  return `INSERT INTO \`${tableName}\` (${columns}) VALUES\n${inserts.join(',\n')};`
}
