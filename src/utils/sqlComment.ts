export function addLineComment(text: string): string {
  return text
    .split('\n')
    .map(line => `-- ${line}`)
    .join('\n')
}

export function removeLineComment(text: string): string {
  return text
    .split('\n')
    .map(line => line.replace(/^--\s?/, ''))
    .join('\n')
}

export function addBlockComment(text: string): string {
  return `/*\n${text}\n*/`
}

export function removeBlockComment(text: string): string {
  let result = text.trim()
  if (result.startsWith('/*')) {
    result = result.slice(2).trim()
  }
  if (result.endsWith('*/')) {
    result = result.slice(0, -2).trim()
  }
  return result
}
