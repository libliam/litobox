export type QuoteType = 'single' | 'double' | 'none'

export function convertToSqlIn(text: string, quoteType: QuoteType = 'single'): string {
  const lines = text.split('\n')
    .map(line => line.trim())
    .filter(line => line !== '')

  if (lines.length === 0) {
    return ''
  }

  let formatted: string
  if (quoteType === 'single') {
    formatted = lines.map(line => `'${line}'`).join(',')
  } else if (quoteType === 'double') {
    formatted = lines.map(line => `"${line}"`).join(',')
  } else {
    formatted = lines.join(',')
  }

  return `(${formatted})`
}
