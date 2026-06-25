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