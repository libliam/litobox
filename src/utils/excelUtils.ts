import * as XLSX from 'xlsx'
import { decodeTextSmart } from './textEncoding'

// re-export 保持调用方（ExcelTool.vue）导入不变
export { decodeTextSmart }

/** 单个 Sheet 的规范化数据（表头 + 行数据，单元格统一为字符串） */
export interface SheetData {
  name: string
  headers: string[]
  rows: string[][]
}

/** 解析 Excel 文件（xlsx/xls），返回所有 Sheet 的规范化数据 */
export async function readExcelFile(file: File): Promise<SheetData[]> {
  const buf = await file.arrayBuffer()
  const wb = XLSX.read(buf, { type: 'array', cellDates: false })
  const sheets: SheetData[] = []
  for (const sheetName of wb.SheetNames) {
    const ws = wb.Sheets[sheetName]
    // 原始二维数组（可能包含合并单元格空位、无表头的情况）
    const raw: any[][] = XLSX.utils.sheet_to_json(ws, { header: 1, defval: '' })
    const { headers, rows } = normalizeSheet(raw)
    sheets.push({ name: sheetName, headers, rows })
  }
  return sheets
}

/** 解析 CSV 文本，得到单个 Sheet 数据 */
export function parseCsvText(text: string, delimiter: string = ','): SheetData {
  const rows = parseDelimited(text, delimiter)
  const { headers, rows: dataRows } = normalizeSheet(rows)
  return { name: 'CSV', headers, rows: dataRows }
}

/** 将任意二维数组规范化为 headers + rows（自动去空行、补齐列数、转字符串） */
function normalizeSheet(raw: any[][]): { headers: string[]; rows: string[][] } {
  const meaningful = raw.filter(row => row && row.some(cell => cell !== '' && cell !== null && cell !== undefined))
  if (meaningful.length === 0) return { headers: [], rows: [] }
  const maxCols = Math.max(...meaningful.map(row => row.length))
  // 表头：取第一行，空表头补 "列N"
  const headers = Array.from({ length: maxCols }, (_, i) => {
    const h = meaningful[0][i]
    return h === '' || h === null || h === undefined ? `列${i + 1}` : String(h).trim()
  })
  const rows = meaningful.slice(1).map(row => {
    return Array.from({ length: maxCols }, (_, i) => {
      const v = row[i]
      return v === null || v === undefined ? '' : String(v)
    })
  })
  return { headers, rows }
}

/** 简易 CSV 解析：支持引号转义、分隔符内换行 */
function parseDelimited(text: string, delimiter: string): string[][] {
  const rows: string[][] = []
  let row: string[] = []
  let field = ''
  let inQuotes = false
  const t = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n')
  for (let i = 0; i < t.length; i++) {
    const ch = t[i]
    if (inQuotes) {
      if (ch === '"') {
        if (t[i + 1] === '"') {
          field += '"'
          i++
        } else {
          inQuotes = false
        }
      } else {
        field += ch
      }
    } else if (ch === '"') {
      inQuotes = true
    } else if (ch === delimiter) {
      row.push(field)
      field = ''
    } else if (ch === '\n') {
      row.push(field)
      rows.push(row)
      row = []
      field = ''
    } else {
      field += ch
    }
  }
  row.push(field)
  rows.push(row)
  // 去掉全空的尾部行
  while (rows.length > 0 && rows[rows.length - 1].every(c => c.trim() === '')) {
    rows.pop()
  }
  return rows
}

/** 合并多个 Sheet：byRows=纵向追加行（表头取第一个），byColumns=横向拼接列 */
export function mergeSheets(
  sources: SheetData[],
  options: { mode: 'rows' | 'columns' }
): SheetData {
  const nonEmpty = sources.filter(s => s.headers.length > 0)
  if (nonEmpty.length === 0) return { name: '合并结果', headers: [], rows: [] }

  if (options.mode === 'columns') {
    const headers: string[] = []
    const rowCount = Math.max(...nonEmpty.map(s => s.rows.length))
    const columns: string[][] = [] // 每个来源的列
    for (const s of nonEmpty) {
      const h = s.headers.map(h => (sources.length > 1 ? `${s.name}[${h}]` : h))
      headers.push(...h)
      columns.push(...s.headers.map((_, colIdx) => s.rows.map(r => r[colIdx] ?? '')))
    }
    const rows = Array.from({ length: rowCount }, (_, ri) =>
      columns.map(col => col[ri] ?? '')
    )
    return { name: '合并结果', headers, rows }
  }

  // 纵向：保留第一个文件的表头，追加其余行
  const base = nonEmpty[0]
  const rows: string[][] = [...base.rows]
  for (const s of nonEmpty.slice(1)) {
    rows.push(...s.rows)
  }
  return { name: '合并结果', headers: base.headers, rows }
}

/** 数据清洗选项 */
export interface CleanOptions {
  removeEmptyRows: boolean // 去掉整行为空的记录
  trimCells: boolean // 单元格去首尾空格
  deduplicate: boolean // 按整行去重
  fillEmpty: boolean // 空单元格填充为 -
}

export function cleanData(sheet: SheetData, options: CleanOptions): SheetData {
  let rows = sheet.rows.map(r => [...r])
  if (options.trimCells) {
    rows = rows.map(r => r.map(c => (typeof c === 'string' ? c.trim() : c)))
  }
  if (options.fillEmpty) {
    rows = rows.map(r => r.map(c => (c === '' ? '-' : c)))
  }
  if (options.removeEmptyRows) {
    rows = rows.filter(r => r.some(c => c !== ''))
  }
  if (options.deduplicate) {
    const seen = new Set<string>()
    rows = rows.filter(r => {
      const key = r.join('\u0001')
      if (seen.has(key)) return false
      seen.add(key)
      return true
    })
  }
  return { name: sheet.name, headers: [...sheet.headers], rows }
}

/** Sheet → CSV 文本 */
export function toCSV(sheet: SheetData, delimiter: string = ','): string {
  const esc = (cell: string) => {
    const s = String(cell ?? '')
    if (s.includes(delimiter) || s.includes('"') || s.includes('\n') || s.includes('\r')) {
      return '"' + s.replace(/"/g, '""') + '"'
    }
    return s
  }
  const lines = [sheet.headers.map(esc).join(delimiter)]
  for (const row of sheet.rows) {
    lines.push(row.map(esc).join(delimiter))
  }
  return lines.join('\n')
}

/** Sheet → JSON 文本（对象数组） */
export function toJSON(sheet: SheetData, pretty: boolean = true): string {
  const arr = sheet.rows.map(row => {
    const obj: Record<string, string> = {}
    sheet.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
  return pretty ? JSON.stringify(arr, null, 2) : JSON.stringify(arr)
}

/** Sheet → Markdown 表格文本 */
export function toMarkdown(sheet: SheetData): string {
  const colCount = sheet.headers.length
  const lines: string[] = []
  lines.push('| ' + sheet.headers.join(' | ') + ' |')
  lines.push('|' + Array.from({ length: colCount }, () => ' --- ').join('|') + '|')
  for (const row of sheet.rows) {
    lines.push('| ' + row.map(c => String(c ?? '').replace(/\|/g, '\\|')).join(' | ') + ' |')
  }
  return lines.join('\n')
}

/** Sheet → SQL INSERT 语句 */
export function toSQL(sheet: SheetData, tableName: string): string {
  const table = (tableName || 'table').replace(/[^A-Za-z0-9_]/g, '_')
  const colList = sheet.headers.map(h => '`' + h.replace(/`/g, '') + '`').join(', ')
  const esc = (v: string) => {
    if (v === '' || v === null || v === undefined) return 'NULL'
    if (/^-?\d+(\.\d+)?$/.test(v)) return v
    return "'" + String(v).replace(/'/g, "''") + "'"
  }
  const lines: string[] = []
  for (const row of sheet.rows) {
    lines.push(`INSERT INTO \`${table}\` (${colList}) VALUES (${row.map(esc).join(', ')});`)
  }
  return lines.join('\n')
}

/** 由 SheetData 生成 xlsx Blob，用于下载/保存 */
export function sheetToXlsxBlob(sheet: SheetData): Blob {
  const ws = XLSX.utils.aoa_to_sheet([sheet.headers, ...sheet.rows])
  const wb = XLSX.utils.book_new()
  XLSX.utils.book_append_sheet(wb, ws, (sheet.name || 'Sheet1').slice(0, 31))
  const out = XLSX.write(wb, { bookType: 'xlsx', type: 'array' })
  return new Blob([out], {
    type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  })
}

/** 导出单个 Sheet 数据（供合并/清洗后的结果使用） */
export function exportSheet(sheet: SheetData, format: 'csv' | 'json' | 'md' | 'sql', tableName: string): string {
  switch (format) {
    case 'csv': return toCSV(sheet, ',')
    case 'json': return toJSON(sheet, true)
    case 'md': return toMarkdown(sheet)
    case 'sql': return toSQL(sheet, tableName)
  }
}

/** 自检函数：验证核心解析逻辑正确性 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    // CSV 解析（含引号转义）
    const csv = parseCsvText('name,age\n"张三",25\n"李,四",30\n')
    if (csv.headers.length !== 2 || csv.rows.length !== 2 || csv.rows[1][0] !== '李,四') {
      errors.push('CSV 解析失败')
    }
  } catch (e: any) { errors.push('CSV 解析异常: ' + e.message) }
  try {
    // 去重
    const base: SheetData = { name: 't', headers: ['a'], rows: [['1'], ['1'], ['2']] }
    const dedup = cleanData(base, { removeEmptyRows: true, trimCells: false, deduplicate: true, fillEmpty: false })
    if (dedup.rows.length !== 2) errors.push('去重失败')
  } catch (e: any) { errors.push('去重异常: ' + e.message) }
  try {
    // Markdown 导出
    const base: SheetData = { name: 't', headers: ['a', 'b'], rows: [['1', '2']] }
    const md = toMarkdown(base)
    if (!md.includes('| a | b |') || !md.includes('| 1 | 2 |')) errors.push('Markdown 导出失败')
  } catch (e: any) { errors.push('Markdown 导出异常: ' + e.message) }
  try {
    // SQL 导出
    const base: SheetData = { name: 't', headers: ['id', 'name'], rows: [['1', "O'Brien"]] }
    const sql = toSQL(base, 'users')
    if (!sql.includes('INSERT INTO `users`') || !sql.includes("O''Brien")) errors.push('SQL 导出失败')
  } catch (e: any) { errors.push('SQL 导出异常: ' + e.message) }
  try {
    // xlsx 生成
    const base: SheetData = { name: 'Sheet1', headers: ['a'], rows: [['1']] }
    const blob = sheetToXlsxBlob(base)
    if (blob.size === 0) errors.push('xlsx 生成失败')
  } catch (e: any) { errors.push('xlsx 生成异常: ' + e.message) }
  try {
    // GBK 编码智能解码（"中文" 的 GBK 字节序列，应回退到 GBK 而非按 UTF-8 乱码解码）
    const gbkBytes = new Uint8Array([0xd6, 0xd0, 0xce, 0xc4])
    const dec = decodeTextSmart(gbkBytes)
    if (dec !== '中文') errors.push(`GBK 解码失败: ${dec}`)
    // UTF-8 带 BOM 检测
    const utf8Bom = new Uint8Array([0xef, 0xbb, 0xbf, ...new TextEncoder().encode('编码')])
    if (decodeTextSmart(utf8Bom) !== '编码') errors.push('UTF-8 BOM 解码失败')
  } catch (e: any) { errors.push('GBK 解码异常: ' + e.message) }
  return errors
}
