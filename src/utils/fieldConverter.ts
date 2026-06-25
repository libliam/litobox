export function snakeToCamel(text: string): string {
  return text
    .split('_')
    .map((part, index) => {
      if (index === 0) return part.toLowerCase()
      return part.charAt(0).toUpperCase() + part.slice(1).toLowerCase()
    })
    .join('')
}

export function camelToSnake(text: string): string {
  return text
    .replace(/([A-Z])/g, '_$1')
    .replace(/^_/, '')
    .toLowerCase()
}

export function convertFields(text: string, mode: 'snakeToCamel' | 'camelToSnake'): string {
  const lines = text.split('\n')
  const convert = mode === 'snakeToCamel' ? snakeToCamel : camelToSnake
  return lines
    .map(line => line.trim())
    .filter(line => line !== '')
    .map(convert)
    .join('\n')
}
