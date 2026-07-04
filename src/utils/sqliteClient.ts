import { invoke } from '@tauri-apps/api/core'

// ============ 类型定义 ============

export interface TableInfo {
  name: string
  row_count: number
}

export interface ColumnInfo {
  name: string
  data_type: string
  not_null: boolean
  is_primary_key: boolean
  default_value: string | null
}

export interface QueryResult {
  columns: string[]
  rows: unknown[][]
  affected_rows: number
  execution_ms: number
}

// ============ invoke 封装 ============

export function sqliteListTables(dbPath: string): Promise<TableInfo[]> {
  return invoke<TableInfo[]>('sqlite_list_tables', { dbPath })
}

export function sqliteGetSchema(dbPath: string, tableName: string): Promise<ColumnInfo[]> {
  return invoke<ColumnInfo[]>('sqlite_get_schema', { dbPath, tableName })
}

export function sqliteQuery(dbPath: string, sql: string, limit?: number): Promise<QueryResult> {
  return invoke<QueryResult>('sqlite_query', { dbPath, sql, limit: limit ?? null })
}

export function sqliteTablePreview(dbPath: string, tableName: string): Promise<QueryResult> {
  return invoke<QueryResult>('sqlite_table_preview', { dbPath, tableName })
}

export function sqliteExportCsv(dbPath: string, sql: string, savePath: string): Promise<number> {
  return invoke<number>('sqlite_export_csv', { dbPath, sql, savePath })
}

export function sqliteGetAppDbPath(): Promise<string> {
  return invoke<string>('sqlite_get_app_db_path')
}
