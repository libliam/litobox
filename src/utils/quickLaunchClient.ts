import { invoke } from '@tauri-apps/api/core'

export interface QuickLaunchResult {
  id: number
  name: string
  path: string
  extension: string
  sizeBytes: number
  modifiedAt: number
  drive: string
}

export interface DriveIndexInfo {
  drive: string
  lastScanned: number
  fileCount: number
  status: 'pending' | 'indexing' | 'ready' | 'failed'
}

export interface IndexStatus {
  drives: DriveIndexInfo[]
  isBuilding: boolean
}

export interface QLIndexProgress {
  searchId: string
  filesScanned: number
  totalFiles: number
  currentDrive: string
  currentPath: string
  status: 'indexing' | 'completed' | 'cancelled' | 'failed'
  error?: string
}

export async function qlSearch(query: string): Promise<QuickLaunchResult[]> {
  return invoke<QuickLaunchResult[]>('ql_search', { query })
}

export async function qlIndexStatus(): Promise<IndexStatus> {
  return invoke<IndexStatus>('ql_index_status')
}

export async function qlBuildIndex(): Promise<string> {
  return invoke<string>('ql_build_index')
}

export async function qlRebuildIndex(): Promise<string> {
  return invoke<string>('ql_rebuild_index')
}

export async function qlCancelIndex(): Promise<void> {
  return invoke<void>('ql_cancel_index')
}

export async function qlOpenFile(path: string): Promise<void> {
  return invoke<void>('ql_open_file', { path })
}
