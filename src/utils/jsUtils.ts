import JSON5 from 'json5'

export interface JsFormatResult {
  success: boolean
  data?: string
  error?: string
}

export interface JsObfuscateResult {
  success: boolean
  data?: string
  error?: string
}

export interface JsonExtractResult {
  success: boolean
  data?: string
  error?: string
}

// 简单的 JS 格式化（基础缩进和换行）
export function formatJsCode(code: string): JsFormatResult {
  try {
    if (!code.trim()) {
      return { success: false, error: '代码不能为空' }
    }

    // 使用简单的缩进策略
    let indent = 0
    const lines: string[] = []
    const indentStr = '  '

    // 先按分号和换行分割
    const tokens = code.replace(/\n/g, ' ').split(/(?<=[;{}])|(?=[{}])/).filter(t => t.trim())

    for (const token of tokens) {
      const trimmed = token.trim()
      if (!trimmed) continue

      if (trimmed === '}') {
        indent = Math.max(0, indent - 1)
      }

      lines.push(indentStr.repeat(indent) + trimmed)

      if (trimmed.endsWith('{')) {
        indent++
      }
    }

    return { success: true, data: lines.join('\n') }
  } catch (error) {
    const message = error instanceof Error ? error.message : '格式化失败'
    return { success: false, error: message }
  }
}

// 简单的 JS 压缩（移除空白和换行）
export function compressJsCode(code: string): JsFormatResult {
  try {
    if (!code.trim()) {
      return { success: false, error: '代码不能为空' }
    }

    // 移除注释
    let compressed = code
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .replace(/\/\/.*$/gm, '')

    // 移除多余空白（保留字符串内的空白）
    compressed = compressed
      .replace(/\s+/g, ' ')
      .replace(/\s*([{}();,=+\-*/<>!&|])\s*/g, '$1')
      .trim()

    return { success: true, data: compressed }
  } catch (error) {
    const message = error instanceof Error ? error.message : '压缩失败'
    return { success: false, error: message }
  }
}

// 从 JSON 对象生成提取代码
export function generateJsonExtractCode(jsonStr: string, targetPath: string): JsonExtractResult {
  try {
    if (!jsonStr.trim()) {
      return { success: false, error: 'JSON 不能为空' }
    }

    const parsed = JSON5.parse(jsonStr)
    const pathParts = targetPath.split('.').filter(p => p)

    // 验证路径
    let current: any = parsed
    for (const part of pathParts) {
      if (current === undefined || current === null) {
        return { success: false, error: `路径 "${targetPath}" 不存在` }
      }
      current = current[part]
    }

    // 生成提取代码
    const code = `// 从 JSON 对象中提取 "${targetPath}"
const json = ${JSON.stringify(parsed, null, 2)};
const result = ${pathParts.map(p => `['${p}']`).join('')};
console.log(result);`

    return { success: true, data: code }
  } catch (error) {
    const message = error instanceof Error ? error.message : '生成失败'
    return { success: false, error: message }
  }
}
