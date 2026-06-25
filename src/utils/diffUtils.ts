import { diffLines, diffChars } from 'diff'

export interface DiffLine {
  type: 'equal' | 'add' | 'remove' | 'modify'
  leftLine?: string
  rightLine?: string
  leftLineNum?: number
  rightLineNum?: number
  charDiffs?: CharDiff[]
}

export interface CharDiff {
  type: 'equal' | 'add' | 'remove'
  value: string
}

export interface DiffOptions {
  ignoreWhitespace: boolean
  ignoreCase: boolean
}

/**
 * 行级对比
 */
export function computeLineDiff(
  left: string,
  right: string,
  options: DiffOptions = { ignoreWhitespace: false, ignoreCase: false }
): DiffLine[] {
  let leftText = left
  let rightText = right

  if (options.ignoreCase) {
    leftText = leftText.toLowerCase()
    rightText = rightText.toLowerCase()
  }

  // ponytail: ensure trailing newline so diffLines treats lines consistently
  if (leftText && !leftText.endsWith('\n')) leftText += '\n'
  if (rightText && !rightText.endsWith('\n')) rightText += '\n'

  const changes = diffLines(leftText, rightText, {
    ignoreWhitespace: options.ignoreWhitespace,
    newlineIsToken: true,
  })

  const result: DiffLine[] = []
  let leftNum = 0
  let rightNum = 0

  for (const change of changes) {
    // skip pure newline tokens
    if (change.value.trim() === '' && (change.added || change.removed)) continue

    const lines = change.value.split('\n').filter(l => l.length > 0)

    if (change.added) {
      for (const line of lines) {
        rightNum++
        result.push({ type: 'add', rightLine: line, rightLineNum: rightNum })
      }
    } else if (change.removed) {
      for (const line of lines) {
        leftNum++
        result.push({ type: 'remove', leftLine: line, leftLineNum: leftNum })
      }
    } else {
      for (const line of lines) {
        leftNum++
        rightNum++
        result.push({ type: 'equal', leftLine: line, rightLine: line, leftLineNum: leftNum, rightLineNum: rightNum })
      }
    }
  }

  return markModifications(result)
}

/**
 * 标记修改行（相邻的 remove 和 add 配对为 modify）
 */
function markModifications(lines: DiffLine[]): DiffLine[] {
  const result: DiffLine[] = []
  let i = 0

  while (i < lines.length) {
    if (lines[i].type === 'remove' && i + 1 < lines.length && lines[i + 1].type === 'add') {
      result.push({
        type: 'modify',
        leftLine: lines[i].leftLine,
        rightLine: lines[i + 1].rightLine,
        leftLineNum: lines[i].leftLineNum,
        rightLineNum: lines[i + 1].rightLineNum,
      })
      i += 2
    } else {
      result.push(lines[i])
      i++
    }
  }

  return result
}

/**
 * 字符级对比（用于 modify 行）
 */
export function computeCharDiff(leftLine: string, rightLine: string): CharDiff[] {
  const changes = diffChars(leftLine, rightLine)
  return changes.map(c => ({
    type: c.added ? 'add' : c.removed ? 'remove' : 'equal',
    value: c.value,
  }))
}
