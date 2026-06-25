export interface RegexMatch {
  text: string
  index: number
  groups?: Record<string, string>
  captures?: string[]
}

export interface RegexResult {
  success: boolean
  matches: RegexMatch[]
  error?: string
  replacedText?: string
}

export function testRegex(
  pattern: string,
  text: string,
  flags: string,
  replacePattern?: string
): RegexResult {
  if (!pattern) {
    return { success: false, matches: [], error: '请输入正则表达式' }
  }
  if (!text) {
    return { success: false, matches: [], error: '请输入测试文本' }
  }

  try {
    const regex = new RegExp(pattern, flags)
    const matches: RegexMatch[] = []
    let match

    // 使用 matchAll 获取所有匹配
    const globalRegex = new RegExp(pattern, flags.includes('g') ? flags : flags + 'g')
    for (match of text.matchAll(globalRegex)) {
      const matchResult: RegexMatch = {
        text: match[0],
        index: match.index || 0
      }
      // 提取位置捕获组 (match[1], match[2], ...)
      const captures = match.slice(1)
      if (captures.length > 0) {
        matchResult.captures = captures.map(c => c ?? '')
      }
      if (match.groups) {
        matchResult.groups = match.groups
      }
      matches.push(matchResult)
    }

    let replacedText: string | undefined
    if (replacePattern !== undefined) {
      replacedText = text.replace(regex, replacePattern)
    }

    return { success: true, matches, replacedText }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    return { success: false, matches: [], error: message }
  }
}