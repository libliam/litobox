import TOML from '@iarna/toml'

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

// ============================================================
// 配置格式互转：JSON / YAML / TOML / INI / Properties
// 统一模式：先 parse 成 JS object，再 stringify 成目标格式
// ============================================================

export type ConfigFormat = 'json' | 'yaml' | 'toml' | 'ini' | 'properties'

/** 把任意配置格式文本解析为 JS 对象 */
export function parseConfig(text: string, format: ConfigFormat): any {
  const trimmed = text.trim()
  if (!trimmed) return {}
  switch (format) {
    case 'json': return JSON.parse(trimmed)
    case 'yaml': return parseYaml(trimmed)
    case 'toml': return parseToml(trimmed)
    case 'ini': return parseIni(trimmed)
    case 'properties': return parseProperties(trimmed)
  }
}

/** 把 JS 对象序列化为指定配置格式文本 */
export function stringifyConfig(obj: any, format: ConfigFormat): string {
  switch (format) {
    case 'json': return JSON.stringify(obj, null, 2)
    case 'yaml': return jsonToYaml(JSON.stringify(obj))
    case 'toml': return tomlStringify(obj)
    case 'ini': return iniStringify(obj)
    case 'properties': return propertiesStringify(obj)
  }
}

// ---------- TOML ----------

export function parseToml(text: string): any {
  return TOML.parse(text)
}

export function tomlStringify(obj: any): string {
  return TOML.stringify(obj as any)
}

// ---------- INI ----------
/**
 * 简易 INI 解析器（支持 section、注释、引号字符串）
 * [section]
 * key = value  ; comment
 * key = "quoted value"
 */
export function parseIni(text: string): any {
  const result: Record<string, any> = {}
  let currentSection: Record<string, any> = result
  const sectionRegex = /^\[([^\]]+)\]\s*(?:[;#].*)?$/
  const keyValueRegex = /^([^=;#]+?)\s*=\s*(.*?)\s*$/

  for (const rawLine of text.split(/\r?\n/)) {
    let line = rawLine.trim()
    if (!line) continue
    // 整行注释
    if (/^[;#]/.test(line)) continue
    // 行尾注释（若不在引号内则移除）
    let inQuote = false
    let quoteChar = ''
    let cleanLine = ''
    for (let i = 0; i < line.length; i++) {
      const ch = line[i]
      if (!inQuote && (ch === ';' || ch === '#')) break
      if ((ch === '"' || ch === "'") && (i === 0 || line[i - 1] !== '\\')) {
        if (!inQuote) { inQuote = true; quoteChar = ch }
        else if (ch === quoteChar) { inQuote = false }
      }
      cleanLine += ch
    }
    line = cleanLine.trim()
    if (!line) continue

    const sectionMatch = line.match(sectionRegex)
    if (sectionMatch) {
      const sectionName = sectionMatch[1].trim()
      if (!result[sectionName]) result[sectionName] = {}
      currentSection = result[sectionName]
      continue
    }

    const kvMatch = line.match(keyValueRegex)
    if (kvMatch) {
      const key = kvMatch[1].trim()
      let value = kvMatch[2].trim()
      currentSection[key] = parseIniValue(value)
    }
  }
  return result
}

function parseIniValue(value: string): any {
  if ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))) {
    return value.slice(1, -1)
  }
  if (value === 'true') return true
  if (value === 'false') return false
  if (value === 'null' || value === '') return null
  if (!isNaN(Number(value)) && value !== '') return Number(value)
  return value
}

/**
 * JS 对象 → INI 字符串
 * 顶层标量写在 [DEFAULT] 前（无 section），嵌套对象写为 [section]
 */
export function iniStringify(obj: any): string {
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    throw new Error('INI 只能从对象（含嵌套对象）转换，不支持数组/标量顶层值')
  }
  const lines: string[] = []
  const topLevel: Record<string, any> = {}
  const sections: Array<[string, Record<string, any>]> = []

  for (const [k, v] of Object.entries(obj)) {
    if (typeof v === 'object' && v !== null && !Array.isArray(v)) {
      sections.push([k, v as Record<string, any>])
    } else {
      topLevel[k] = v
    }
  }

  const formatValue = (v: any): string => {
    if (v === null) return ''
    if (typeof v === 'boolean') return v ? 'true' : 'false'
    if (typeof v === 'string') {
      if (v.includes(';') || v.includes('#') || /^\s|\s$/.test(v)) {
        return `"${v.replace(/"/g, '\\"')}"`
      }
      return v
    }
    if (Array.isArray(v)) return v.map(formatValue).join(', ')
    return String(v)
  }

  const writeSection = (o: Record<string, any>) => {
    for (const [k, v] of Object.entries(o)) {
      if (typeof v === 'object' && v !== null && !Array.isArray(v)) {
        // INI 不支持二级嵌套 section，用 key.subkey 扁平写
        for (const [sk, sv] of Object.entries(v)) {
          lines.push(`${k}.${sk} = ${formatValue(sv)}`)
        }
      } else {
        lines.push(`${k} = ${formatValue(v)}`)
      }
    }
  }

  writeSection(topLevel)
  for (const [name, data] of sections) {
    if (lines.length && lines[lines.length - 1] !== '') lines.push('')
    lines.push(`[${name}]`)
    writeSection(data)
  }
  return lines.join('\n')
}

// ---------- Properties ----------
/**
 * 简易 Java Properties 解析器
 * 支持：a.b.c=value 点号键 → 嵌套对象、注释(#!)、\n \t 转义
 */
export function parseProperties(text: string): any {
  const result: Record<string, any> = {}
  for (const rawLine of text.split(/\r?\n/)) {
    let line = rawLine.trim()
    if (!line) continue
    if (/^[#!]/.test(line)) continue
    // 行尾 \ 续行（简化：不处理，按单行解析）
    let sepIdx = -1
    // 找第一个 = 或 : 作为分隔符，不以 \ 转义
    for (let i = 0; i < line.length; i++) {
      const ch = line[i]
      if ((ch === '=' || ch === ':' ) && (i === 0 || line[i - 1] !== '\\')) {
        sepIdx = i; break
      }
    }
    if (sepIdx === -1) continue
    const rawKey = line.slice(0, sepIdx).trim()
    let rawValue = line.slice(sepIdx + 1).trim()
    // 去掉值外层可选引号
    if ((rawValue.startsWith('"') && rawValue.endsWith('"'))) rawValue = rawValue.slice(1, -1)
    // 转义 \n \t \\
    rawValue = rawValue.replace(/\\n/g, '\n').replace(/\\t/g, '\t').replace(/\\\\/g, '\\')
    const keys = rawKey.split('.')
    let node: any = result
    for (let i = 0; i < keys.length; i++) {
      const k = keys[i]
      if (i === keys.length - 1) {
        node[k] = parseIniValue(rawValue)
      } else {
        if (node[k] == null || typeof node[k] !== 'object') {
          node[k] = {}
        }
        node = node[k]
      }
    }
  }
  return result
}

/**
 * JS 对象 → Properties 字符串（点号扁平）
 * 嵌套对象用 a.b.c 展开，数组用 list.0 list.1 展开（Java Properties 惯例）
 */
export function propertiesStringify(obj: any): string {
  if (typeof obj !== 'object' || obj === null) {
    throw new Error('Properties 只能从对象转换')
  }
  const lines: string[] = []
  const escValue = (v: string): string => {
    return v.replace(/\\/g, '\\\\').replace(/\n/g, '\\n').replace(/\t/g, '\\t')
  }
  const walk = (o: any, prefix: string) => {
    if (Array.isArray(o)) {
      o.forEach((item, i) => walk(item, prefix ? `${prefix}.${i}` : `${i}`))
    } else if (typeof o === 'object' && o !== null) {
      for (const [k, v] of Object.entries(o)) {
        walk(v, prefix ? `${prefix}.${k}` : k)
      }
    } else {
      let val: string
      if (o === null || o === undefined) val = ''
      else if (typeof o === 'boolean') val = o ? 'true' : 'false'
      else val = String(o)
      lines.push(`${prefix}=${escValue(val)}`)
    }
  }
  walk(obj, '')
  return lines.join('\n')
}

// ============================================================
// 简易 YAML 解析器（覆盖 90% 常见场景）
// ============================================================

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
