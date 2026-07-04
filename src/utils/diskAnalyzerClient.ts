import { invoke } from '@tauri-apps/api/core'
import type {
  ScanOptions,
  ScanStatus,
  ScanSummary,
  FolderPage,
  FilePage,
  ExtStatPage,
  DupPage,
  DeleteResult,
} from './diskAnalyzerTypes'

export async function diskScanStart(path: string, opts: ScanOptions): Promise<string> {
  return invoke<string>('disk_scan_start', { path, opts })
}

export async function diskScanCancel(scanId: string): Promise<void> {
  return invoke('disk_scan_cancel', { scanId })
}

export async function diskScanStatus(scanId: string): Promise<ScanStatus> {
  return invoke<ScanStatus>('disk_scan_status', { scanId })
}

export async function diskGetSummary(scanId: string): Promise<ScanSummary> {
  return invoke<ScanSummary>('disk_get_summary', { scanId })
}

export async function diskGetFolders(
  scanId: string,
  parent: string | null,
  limit?: number,
  offset?: number
): Promise<FolderPage> {
  return invoke<FolderPage>('disk_get_folders', { scanId, parent, limit, offset })
}

export async function diskGetTopFiles(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<FilePage> {
  return invoke<FilePage>('disk_get_top_files', { scanId, limit, offset })
}

export async function diskGetExtensionStats(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<ExtStatPage> {
  return invoke<ExtStatPage>('disk_get_extension_stats', { scanId, limit, offset })
}

export async function diskGetDuplicates(
  scanId: string,
  limit?: number,
  offset?: number
): Promise<DupPage> {
  return invoke<DupPage>('disk_get_duplicates', { scanId, limit, offset })
}

export async function diskDeleteFiles(paths: string[]): Promise<DeleteResult> {
  return invoke<DeleteResult>('disk_delete_files', { paths })
}

export async function diskClearScan(scanId: string): Promise<void> {
  return invoke('disk_clear_scan', { scanId })
}

export async function diskLocateInExplorer(path: string): Promise<void> {
  return invoke('disk_locate_in_explorer', { path })
}
