import { invoke } from '@tauri-apps/api/core'

// 前端接口字段名与 Rust 结构体保持一致（Tauri 不会自动转换结构体字段名）
export type DiffEntryStatus = 'added' | 'removed' | 'modified' | 'same'

export interface DiffEntry {
  path: string
  status: DiffEntryStatus
  left_size: number | null
  right_size: number | null
  left_modified: number | null
  right_modified: number | null
}

export interface DiffSummary {
  added: number
  removed: number
  modified: number
  same: number
  total: number
  skipped: number
  duration_ms: number
}

export interface DiffPage {
  items: DiffEntry[]
  total: number
}

export type DiffState =
  | { status: 'running' }
  | { status: 'completed' }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }

export async function dirDiffStart(
  left: string,
  right: string,
  ignore: string[],
  compareContent: boolean,
): Promise<string> {
  return invoke<string>('dir_diff_start', { left, right, ignore, compareContent })
}

export async function dirDiffCancel(scanId: string): Promise<void> {
  return invoke('dir_diff_cancel', { scanId })
}

export async function dirDiffStatus(scanId: string): Promise<DiffState> {
  return invoke<DiffState>('dir_diff_status', { scanId })
}

export async function dirDiffGetSummary(scanId: string): Promise<DiffSummary> {
  return invoke<DiffSummary>('dir_diff_get_summary', { scanId })
}

export async function dirDiffGetEntries(
  scanId: string,
  status?: string,
  keyword?: string,
  limit?: number,
  offset?: number,
): Promise<DiffPage> {
  return invoke<DiffPage>('dir_diff_get_entries', { scanId, status, keyword, limit, offset })
}

export async function dirDiffClear(scanId: string): Promise<void> {
  return invoke('dir_diff_clear', { scanId })
}
