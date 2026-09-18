export function trimLeadingTrailing(text: string): string {
  return text.split('\n').map(line => line.trim()).join('\n')
}

export function trimAllSpaces(text: string): string {
  return text.replace(/\s+/g, '')
}

export function trimSpacesKeepNewlines(text: string): string {
  return text.split('\n').map(line => line.replace(/[^\S\n]/g, '')).join('\n')
}

export function joinLines(text: string, separator: string = ','): string {
  return text.split('\n')
    .filter(line => line.trim() !== '')
    .map(line => line.trim())
    .join(separator)
}

export function splitText(text: string, separator: string): string[] {
  return text.split(separator).map(item => item.trim()).filter(item => item !== '')
}

export function toUpperCase(text: string): string {
  return text.toUpperCase()
}

export function toLowerCase(text: string): string {
  return text.toLowerCase()
}

export function toTitleCase(text: string): string {
  return text.replace(/\w\S*/g, txt => txt.charAt(0).toUpperCase() + txt.slice(1).toLowerCase())
}

export function toCamelCase(text: string): string {
  return text.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : '')
}

export function toSnakeCase(text: string): string {
  return text.replace(/([A-Z])/g, (_, char) => '_' + char)
    .replace(/[-\s]+/g, '_')
    .toLowerCase()
    .replace(/^_/, '')
    .replace(/_+/g, '_')
}

export function removeNewlines(text: string): string {
  return text.replace(/\n/g, '')
}

export function removeTabs(text: string): string {
  return text.replace(/\t/g, '')
}

export function removeEmptyLines(text: string): string {
  return text.split('\n').filter(line => line.trim() !== '').join('\n')
}

export function capitalize(text: string): string {
  if (!text) return text
  return text.charAt(0).toUpperCase() + text.slice(1).toLowerCase()
}

export function removeAllSpaces(text: string): string {
  return text.replace(/\s/g, '')
}

export function normalizeSpaces(text: string): string {
  return text.replace(/\s+/g, ' ').trim()
}

export function removeDuplicates(text: string): string {
  const lines = text.split('\n')
  const seen = new Set<string>()
  return lines.filter(line => {
    const trimmed = line.trim()
    if (seen.has(trimmed)) return false
    seen.add(trimmed)
    return true
  }).join('\n')
}

export function reverseLines(text: string): string {
  return text.split('\n').reverse().join('\n')
}

export function sortLines(text: string): string {
  return text.split('\n').sort().join('\n')
}

export function sortLinesDescending(text: string): string {
  return text.split('\n').sort((a, b) => b.localeCompare(a)).join('\n')
}

export function sortLinesIgnoreCase(text: string): string {
  return text.split('\n').sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase())).join('\n')
}

export function sortLinesNumeric(text: string): string {
  return text.split('\n').sort((a, b) => {
    const numA = parseFloat(a) || 0
    const numB = parseFloat(b) || 0
    return numA - numB
  }).join('\n')
}

export function sortLinesNatural(text: string): string {
  return text.split('\n').sort((a, b) =>
    a.localeCompare(b, undefined, { numeric: true, sensitivity: 'base' })
  ).join('\n')
}

export function sortLinesRandom(text: string): string {
  const lines = text.split('\n')
  for (let i = lines.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[lines[i], lines[j]] = [lines[j], lines[i]]
  }
  return lines.join('\n')
}

export function sortLinesUnique(text: string): string {
  const seen = new Set<string>()
  return text.split('\n').filter(line => {
    const trimmed = line.trim()
    if (seen.has(trimmed)) return false
    seen.add(trimmed)
    return true
  }).sort().join('\n')
}

export function sortLinesByColumn(text: string, delimiter: string, columnIndex: number, descending = false): string {
  return text.split('\n').sort((a, b) => {
    const valA = a.split(delimiter)[columnIndex] || ''
    const valB = b.split(delimiter)[columnIndex] || ''
    const result = valA.localeCompare(valB)
    return descending ? -result : result
  }).join('\n')
}

export function toKebabCase(text: string): string {
  return text.replace(/([A-Z])/g, (_, char) => '-' + char)
    .replace(/[_\s]+/g, '-')
    .toLowerCase()
    .replace(/^-/, '')
    .replace(/-+/g, '-')
}

export function toPascalCase(text: string): string {
  return text.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : '')
    .replace(/^[a-z]/, char => char.toUpperCase())
}

export function reverseString(text: string): string {
  return text.split('').reverse().join('')
}

export function toFullWidth(text: string): string {
  return text.replace(/[\u0020-\u007e]/g, char =>
    String.fromCharCode(char.charCodeAt(0) + 0xfee0)
  ).replace(/\u0020/g, '\u3000')
}

export function toHalfWidth(text: string): string {
  return text.replace(/[\uff01-\uff5e]/g, char =>
    String.fromCharCode(char.charCodeAt(0) - 0xfee0)
  ).replace(/\u3000/g, '\u0020')
}