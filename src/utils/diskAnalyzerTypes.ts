// 磁盘分析工具类型定义，与 Rust 端 disk_analyzer.rs 一一对应

export interface ScanOptions {
  includeHidden: boolean
  detectDuplicates: boolean
  maxFiles?: number | null
  followSymlinks: boolean
}

export type ScanStatus =
  | { status: 'running' }
  | { status: 'completed' }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }

export interface FolderInfo {
  path: string
  parent: string | null
  name: string
  depth: number
  fileCount: number
  sizeBytes: number
  percentOfRoot: number
}

export interface FileInfo {
  path: string
  name: string
  sizeBytes: number
  modifiedMs: number
  extension: string
}

export interface ExtensionStat {
  extension: string
  fileCount: number
  totalSize: number
  percent: number
}

export interface DuplicateGroup {
  groupId: number
  fileSize: number
  fileCount: number
  wastedBytes: number
  files: FileInfo[]
}

export interface ScanSummary {
  totalFiles: number
  totalDirs: number
  totalSize: number
  skippedCount: number
  durationMs: number
  duplicatesWastedBytes: number | null
}

export interface DeleteFailure {
  path: string
  error: string
}

export interface DeleteResult {
  succeeded: string[]
  failed: DeleteFailure[]
}

export interface FolderPage {
  items: FolderInfo[]
  total: number
}

export interface FilePage {
  items: FileInfo[]
  total: number
}

export interface ExtStatPage {
  items: ExtensionStat[]
  total: number
}

export interface DupPage {
  items: DuplicateGroup[]
  total: number
}

export interface ScanProgress {
  scanId: string
  filesScanned: number
  bytesScanned: number
  currentPath: string
}
