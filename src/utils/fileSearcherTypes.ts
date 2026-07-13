export interface SearchOptions {
  mode: 'filename' | 'content'
  query: string
  caseSensitive: boolean
  extensions: string[]
  excludeExtensions: string[]
  includeHidden: boolean
  maxContentFileBytes: number
}

export interface MatchedLine {
  lineNumber: number
  lineText: string
  matchRanges: [number, number][]
}

export interface SearchResultItem {
  path: string
  name: string
  extension: string
  sizeBytes: number
  modifiedMs: number
  matchCount: number
  matchedLines: MatchedLine[]
}

export type SearchStatus =
  | { status: 'running' }
  | { status: 'completed' }
  | { status: 'failed'; error: string }
  | { status: 'cancelled' }

export interface SearchProgress {
  searchId: string
  filesScanned: number
  bytesScanned: number
  matchesFound: number
  currentPath: string
}

export interface SearchSummary {
  totalFiles: number
  totalDirs: number
  bytesScanned: number
  matchesFound: number
  durationMs: number
  truncated: boolean
  skippedCount: number
}

export interface SearchResultsPage {
  items: SearchResultItem[]
  total: number
}
