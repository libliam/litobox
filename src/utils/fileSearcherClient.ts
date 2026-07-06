import { invoke } from '@tauri-apps/api/core'
import type {
  SearchOptions,
  SearchStatus,
  SearchSummary,
  SearchResultsPage,
} from './fileSearcherTypes'

export async function fileSearchStart(path: string, opts: SearchOptions): Promise<string> {
  return invoke<string>('file_search_start', { path, opts })
}

export async function fileSearchCancel(searchId: string): Promise<void> {
  return invoke('file_search_cancel', { searchId })
}

export async function fileSearchStatus(searchId: string): Promise<SearchStatus> {
  return invoke<SearchStatus>('file_search_status', { searchId })
}

export async function fileSearchGetSummary(searchId: string): Promise<SearchSummary> {
  return invoke<SearchSummary>('file_search_get_summary', { searchId })
}

export async function fileSearchGetResults(
  searchId: string,
  limit?: number,
  offset?: number
): Promise<SearchResultsPage> {
  return invoke<SearchResultsPage>('file_search_get_results', { searchId, limit, offset })
}

export async function fileSearchClear(searchId: string): Promise<void> {
  return invoke('file_search_clear', { searchId })
}
