/**
 * 格式化 XML
 */
export function formatXml(xml: string, indent: number = 2): string {
  let formatted = ''
  let pad = 0
  const lines = xml.replace(/>\s*</g, '><').split('><')

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i]
    if (!line.trim()) continue

    if (line.match(/^\/\w/)) {
      pad = Math.max(0, pad - 1)
    }

    formatted += ' '.repeat(pad * indent) + line + '\n'

    if (line.match(/^<\w/) && !line.match(/\/>$/) && !line.match(/<\/\w+>$/)) {
      pad++
    }
  }

  return formatted.trim()
}

/**
 * 校验 XML
 */
export function validateXml(xml: string): { valid: boolean; error?: string } {
  try {
    const parser = new DOMParser()
    const doc = parser.parseFromString(xml, 'text/xml')
    const errorNode = doc.querySelector('parsererror')
    if (errorNode) {
      return { valid: false, error: errorNode.textContent?.slice(0, 200) }
    }
    return { valid: true }
  } catch (e: any) {
    return { valid: false, error: e.message }
  }
}

/**
 * XML 转 JSON
 */
export function xmlToJson(xml: string): string {
  const parser = new DOMParser()
  const doc = parser.parseFromString(xml, 'text/xml')
  const errorNode = doc.querySelector('parsererror')
  if (errorNode) {
    throw new Error('XML 格式错误: ' + errorNode.textContent?.slice(0, 100))
  }

  function nodeToJson(node: Node): any {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent?.trim()
      return text ? text : undefined
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return undefined

    const children = Array.from(node.childNodes)
    const result: Record<string, any> = {}

    for (const child of children) {
      if (child.nodeType === Node.TEXT_NODE) {
        const text = child.textContent?.trim()
        if (text) {
          return text
        }
        continue
      }
      if (child.nodeType !== Node.ELEMENT_NODE) continue

      const childName = child.nodeName
      const childValue = nodeToJson(child)
      if (childValue === undefined) continue

      if (result[childName] !== undefined) {
        if (!Array.isArray(result[childName])) {
          result[childName] = [result[childName]]
        }
        result[childName].push(childValue)
      } else {
        result[childName] = childValue
      }
    }

    if (node.nodeType === Node.ELEMENT_NODE) {
      const attrs = (node as Element).attributes
      if (attrs.length > 0) {
        result['$attrs'] = {}
        for (let i = 0; i < attrs.length; i++) {
          result['$attrs'][attrs[i].name] = attrs[i].value
        }
      }
    }

    return result
  }

  const json = nodeToJson(doc.documentElement)
  return JSON.stringify(json, null, 2)
}

/**
 * JSON 转 XML
 */
export function jsonToXml(json: string): string {
  const obj = JSON.parse(json)

  function objToXml(obj: any, tagName: string = 'root'): string {
    if (typeof obj === 'string') return `<${tagName}>${escapeXml(obj)}</${tagName}>`
    if (typeof obj === 'number') return `<${tagName}>${obj}</${tagName}>`
    if (typeof obj === 'boolean') return `<${tagName}>${obj}</${tagName}>`
    if (Array.isArray(obj)) {
      return obj.map(item => objToXml(item, tagName)).join('\n')
    }
    if (typeof obj === 'object' && obj !== null) {
      let attrs = ''
      let children = ''

      for (const [key, value] of Object.entries(obj)) {
        if (key === '$attrs') {
          attrs = Object.entries(value as Record<string, string>)
            .map(([k, v]) => ` ${k}="${escapeXml(v)}"`)
            .join('')
        } else if (Array.isArray(value)) {
          children += value.map(item => objToXml(item, key)).join('\n')
        } else {
          children += objToXml(value, key)
        }
      }

      return `<${tagName}${attrs}>${children}</${tagName}>`
    }
    return `<${tagName}/>`
  }

  function escapeXml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;')
  }

  return '<?xml version="1.0" encoding="UTF-8"?>\n' + objToXml(obj)
}

/**
 * 简易 YAML 解析器（覆盖 90% 常见场景）
 */
export function parseYaml(yaml: string): any {
  const lines = yaml.split('\n')
  const result: Record<string, any> = {}
  let currentKey = ''
  let currentArray: any[] = []
  let inArray = false

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue

    if (trimmed.startsWith('- ')) {
      const value = trimmed.slice(2).trim()
      if (inArray) {
        currentArray.push(parseYamlValue(value))
      } else {
        inArray = true
        currentArray = [parseYamlValue(value)]
      }
      continue
    }

    if (inArray && currentKey) {
      result[currentKey] = currentArray
      inArray = false
      currentArray = []
    }

    const colonIdx = trimmed.indexOf(':')
    if (colonIdx > -1) {
      const key = trimmed.slice(0, colonIdx).trim()
      const value = trimmed.slice(colonIdx + 1).trim()

      if (value === '' || value === '|' || value === '>') {
        currentKey = key
        inArray = false
      } else {
        result[key] = parseYamlValue(value)
        currentKey = ''
      }
    }
  }

  if (inArray && currentKey) {
    result[currentKey] = currentArray
  }

  return result
}

function parseYamlValue(value: string): any {
  if (value === 'true') return true
  if (value === 'false') return false
  if (value === 'null' || value === '~') return null
  if (!isNaN(Number(value)) && value !== '') return Number(value)
  if ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))) {
    return value.slice(1, -1)
  }
  return value
}

/**
 * JSON 转 YAML（简易实现）
 */
export function jsonToYaml(json: string): string {
  const obj = JSON.parse(json)

  function objToYaml(obj: any, indent: number = 0): string {
    const pad = '  '.repeat(indent)
    const lines: string[] = []

    if (Array.isArray(obj)) {
      for (const item of obj) {
        if (typeof item === 'object' && item !== null) {
          let first = true
          for (const [key, value] of Object.entries(item)) {
            const prefix = first ? `${pad}- ` : `${pad}  `
            if (typeof value === 'object' && value !== null) {
              lines.push(`${prefix}${key}:`)
              lines.push(objToYaml(value, indent + 2))
            } else {
              lines.push(`${prefix}${key}: ${formatYamlValue(value)}`)
            }
            first = false
          }
        } else {
          lines.push(`${pad}- ${formatYamlValue(item)}`)
        }
      }
    } else if (typeof obj === 'object' && obj !== null) {
      for (const [key, value] of Object.entries(obj)) {
        if (typeof value === 'object' && value !== null) {
          lines.push(`${pad}${key}:`)
          lines.push(objToYaml(value, indent + 1))
        } else {
          lines.push(`${pad}${key}: ${formatYamlValue(value)}`)
        }
      }
    }

    return lines.join('\n')
  }

  function formatYamlValue(value: any): string {
    if (value === null) return 'null'
    if (typeof value === 'boolean') return value.toString()
    if (typeof value === 'string') {
      if (value.includes(':') || value.includes('#') || value.includes(',') || value === '') {
        return `"${value}"`
      }
      return value
    }
    return String(value)
  }

  return objToYaml(obj)
}
