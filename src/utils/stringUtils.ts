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