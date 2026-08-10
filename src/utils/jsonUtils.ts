import JSON5 from 'json5'

export interface JsonFormatResult {
  success: boolean
  data?: string
  error?: string
  errorLine?: number
}

export interface JsonFormatOptions {
  indent: number
}

export interface JsonStats {
  type: 'array' | 'object' | 'string' | 'number' | 'boolean' | 'null'
  arrayLength?: number
  objectKeys?: string[]
  totalArrayElements?: number
  totalObjectKeys?: number
  stringLength?: number
}

const defaultOptions: JsonFormatOptions = {
  indent: 2
}

export function formatJson(input: string, options: JsonFormatOptions = defaultOptions): JsonFormatResult {
  try {
    const parsed = JSON5.parse(input)
    const formatted = JSON.stringify(parsed, null, options.indent)
    return { success: true, data: formatted }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    const lineMatch = message.match(/line\s+(\d+)/i)
    return {
      success: false,
      error: message,
      errorLine: lineMatch ? parseInt(lineMatch[1], 10) : undefined
    }
  }
}

export function compressJson(input: string): JsonFormatResult {
  try {
    const parsed = JSON5.parse(input)
    const compressed = JSON.stringify(parsed)
    return { success: true, data: compressed }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    return { success: false, error: message }
  }
}

export function validateJson(input: string): JsonFormatResult {
  try {
    JSON5.parse(input)
    return { success: true, data: 'JSON格式正确' }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    const lineMatch = message.match(/line\s+(\d+)/i)
    return {
      success: false,
      error: message,
      errorLine: lineMatch ? parseInt(lineMatch[1], 10) : undefined
    }
  }
}

export function getJsonStats(input: string): JsonStats | null {
  try {
    const parsed = JSON5.parse(input)
    return getStatsFromValue(parsed)
  } catch {
    return null
  }
}

function getStatsFromValue(value: unknown): JsonStats {
  if (Array.isArray(value)) {
    const totalElements = countArrayElements(value)
    return {
      type: 'array',
      arrayLength: value.length,
      totalArrayElements: totalElements
    }
  }
  if (value !== null && typeof value === 'object') {
    const obj = value as Record<string, unknown>
    const keys = Object.keys(obj)
    const totalKeys = countObjectKeys(obj)
    return {
      type: 'object',
      objectKeys: keys,
      totalObjectKeys: totalKeys
    }
  }
  if (typeof value === 'string') {
    return { type: 'string', stringLength: value.length }
  }
  if (typeof value === 'number') {
    return { type: 'number' }
  }
  if (typeof value === 'boolean') {
    return { type: 'boolean' }
  }
  return { type: 'null' }
}

function countArrayElements(arr: unknown[]): number {
  let count = arr.length
  for (const item of arr) {
    if (Array.isArray(item)) {
      count += countArrayElements(item)
    } else if (item !== null && typeof item === 'object') {
      count += countObjectKeys(item as Record<string, unknown>)
    }
  }
  return count
}

function countObjectKeys(obj: Record<string, unknown>): number {
  let count = Object.keys(obj).length
  for (const value of Object.values(obj)) {
    if (Array.isArray(value)) {
      count += countArrayElements(value)
    } else if (value !== null && typeof value === 'object') {
      count += countObjectKeys(value as Record<string, unknown>)
    }
  }
  return count
}

export function filterJsonByPath(input: string, path: string): JsonFormatResult {
  try {
    const parsed = JSON5.parse(input)
    // 简单 key（不含 . 或 [）：深度搜索收集所有匹配项
    if (!path.includes('.') && !path.includes('[')) {
      const matches = collectByKey(parsed, path)
      if (matches.length === 0) {
        return { success: false, error: `未找到 key "${path}"` }
      }
      const result = matches.length === 1 ? matches[0] : matches
      const filtered = JSON.stringify(result, null, 2)
      return { success: true, data: filtered }
    }
    // 路径模式：按路径精确取值
    const result = getValueByPath(parsed, path)
    if (result === undefined) {
      return { success: false, error: `路径 "${path}" 未找到` }
    }
    const filtered = JSON.stringify(result, null, 2)
    return { success: true, data: filtered }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    return { success: false, error: message }
  }
}

function collectByKey(obj: unknown, key: string): unknown[] {
  const results: unknown[] = []
  const walk = (val: unknown) => {
    if (Array.isArray(val)) {
      for (const item of val) walk(item)
    } else if (val !== null && typeof val === 'object') {
      const o = val as Record<string, unknown>
      if (Object.prototype.hasOwnProperty.call(o, key)) {
        results.push(o[key])
      }
      for (const v of Object.values(o)) walk(v)
    }
  }
  walk(obj)
  return results
}

function getValueByPath(obj: unknown, path: string): unknown {
  const keys = path.split(/[.\[\]]/).filter(Boolean)
  let current = obj
  for (const key of keys) {
    if (current === null || current === undefined) {
      return undefined
    }
    if (Array.isArray(current)) {
      const index = parseInt(key, 10)
      if (isNaN(index)) {
        const found = current.find(item => {
          if (item !== null && typeof item === 'object' && !Array.isArray(item)) {
            return Object.prototype.hasOwnProperty.call(item, key)
          }
          return false
        })
        if (found !== undefined) {
          const foundObj = found as Record<string, unknown>
          current = foundObj[key]
        } else {
          return undefined
        }
      } else {
        current = current[index]
      }
    } else if (typeof current === 'object' && current !== null) {
      const obj = current as Record<string, unknown>
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        current = obj[key]
      } else {
        const arrayMatch = key.match(/^(.+)\[(\d+)\]$/)
        if (arrayMatch) {
          const arrName = arrayMatch[1]
          const arrIndex = parseInt(arrayMatch[2], 10)
          if (Object.prototype.hasOwnProperty.call(obj, arrName)) {
            const arr = obj[arrName]
            if (Array.isArray(arr) && arr[arrIndex] !== undefined) {
              current = arr[arrIndex]
            } else {
              return undefined
            }
          } else {
            return undefined
          }
        } else {
          return undefined
        }
      }
    } else {
      return undefined
    }
  }
  return current
}