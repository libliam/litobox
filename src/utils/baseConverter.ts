export interface BaseConversionResult {
  success: boolean
  binary?: string
  octal?: string
  decimal?: string
  hexadecimal?: string
  error?: string
}

export function convertBase(
  input: string,
  fromBase: 2 | 8 | 10 | 16
): BaseConversionResult {
  if (!input.trim()) {
    return { success: false, error: '请输入数值' }
  }

  try {
    // 处理负数
    const isNegative = input.trim().startsWith('-')
    const cleanInput = isNegative ? input.trim().slice(1) : input.trim()

    // 转换为十进制
    const decimal = parseInt(cleanInput, fromBase)
    if (isNaN(decimal)) {
      return { success: false, error: `无效的${fromBase}进制数值` }
    }

    const finalDecimal = isNegative ? -decimal : decimal

    return {
      success: true,
      binary: finalDecimal.toString(2),
      octal: finalDecimal.toString(8),
      decimal: finalDecimal.toString(10),
      hexadecimal: finalDecimal.toString(16).toUpperCase()
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : '转换失败'
    return { success: false, error: message }
  }
}