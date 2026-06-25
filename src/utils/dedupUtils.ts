export interface DedupOptions {
  mode: 'first' | 'last'
  ignoreCase: boolean
  ignoreWhitespace: boolean
}

export interface DedupResult {
  output: string
  originalLines: number
  uniqueLines: number
  duplicateLines: number
  duplicates: Map<string, number>
}

/**
 * 按行去重
 */
export function dedupLines(text: string, options: DedupOptions): DedupResult {
  const lines = text.split('\n')
  const { mode, ignoreCase, ignoreWhitespace } = options

  const normalize = (line: string): string => {
    let result = line
    if (ignoreWhitespace) result = result.trim()
    if (ignoreCase) result = result.toLowerCase()
    return result
  }

  const countMap = new Map<string, number>()
  for (const line of lines) {
    const key = normalize(line)
    countMap.set(key, (countMap.get(key) || 0) + 1)
  }

  const seen = new Map<string, boolean>()
  const outputLines: string[] = []
  const duplicates = new Map<string, number>()

  if (mode === 'first') {
    for (const line of lines) {
      const key = normalize(line)
      if (!seen.has(key)) {
        seen.set(key, true)
        outputLines.push(line)
      } else {
        duplicates.set(line, countMap.get(key) || 1)
      }
    }
  } else {
    const reversed = [...lines].reverse()
    for (const line of reversed) {
      const key = normalize(line)
      if (!seen.has(key)) {
        seen.set(key, true)
        outputLines.unshift(line)
      } else {
        duplicates.set(line, countMap.get(key) || 1)
      }
    }
  }

  return {
    output: outputLines.join('\n'),
    originalLines: lines.length,
    uniqueLines: outputLines.length,
    duplicateLines: lines.length - outputLines.length,
    duplicates,
  }
}
