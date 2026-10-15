// textarea 内 Tab 缩进的纯计算逻辑（与 DOM 无关，便于 node 自检）
export const TAB_INDENT = '  '

export interface TabEdit {
  start: number      // 替换起点
  end: number        // 替换终点
  text: string       // 替换文本
  selStart: number   // 替换后选区起点
  selEnd: number     // 替换后选区终点
}

/**
 * 计算 textarea 在给定光标/选区下按 Tab（shift=false）或 Shift+Tab 的文本替换：
 * - 无选区 Tab：光标处插入一层缩进
 * - 跨行选区 Tab/Shift+Tab：选区涉及的完整行整体缩进/反缩进，替换后整块保持选中
 */
export function computeTabIndent(doc: string, from: number, to: number, shift: boolean): TabEdit {
  // 无选区 Tab：光标处插入一层缩进
  if (!shift && from === to) {
    return {
      start: from,
      end: to,
      text: TAB_INDENT,
      selStart: from + TAB_INDENT.length,
      selEnd: from + TAB_INDENT.length,
    }
  }

  // 块操作：扩展到选区涉及的完整行
  const blockStart = doc.lastIndexOf('\n', from - 1) + 1
  let blockEnd = doc.indexOf('\n', to)
  if (blockEnd === -1) blockEnd = doc.length
  // 选区结尾恰好在行首时不纳入最后的空行（无选区 Shift+Tab 除外，它本就作用于当前行）
  if (from !== to && to > blockStart && doc[to - 1] === '\n') {
    blockEnd = to - 1
  }

  const text = doc
    .slice(blockStart, blockEnd)
    .split('\n')
    .map((line) => {
      if (!shift) return TAB_INDENT + line
      if (line.startsWith(TAB_INDENT)) return line.slice(TAB_INDENT.length)
      if (line[0] === ' ' || line[0] === '\t') return line.slice(1)
      return line
    })
    .join('\n')

  return { start: blockStart, end: blockEnd, text, selStart: blockStart, selEnd: blockStart + text.length }
}
