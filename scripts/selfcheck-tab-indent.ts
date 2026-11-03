// tabIndent 纯函数自检：node scripts/selfcheck-tab-indent.ts（Node 22+ 原生跑 TS，零依赖）
import assert from 'node:assert'
import { computeTabIndent } from '../src/utils/tabIndent.ts'

// 应用 edit 到 doc，便于断言结果文本与光标
function apply(doc: string, from: number, to: number, shift: boolean) {
  const e = computeTabIndent(doc, from, to, shift)
  const next = doc.slice(0, e.start) + e.text + doc.slice(e.end)
  return { next, sel: [e.selStart, e.selEnd] as const }
}

// 1. 无选区 Tab：光标处插入 2 空格，光标落到缩进之后
assert.deepEqual(apply('abc', 1, 1, false), { next: 'a  bc', sel: [3, 3] })

// 2. 跨行选区 Tab：涉及的完整行全部缩进，整块保持选中
{
  const r = apply('aaa\nbbb\nccc', 2, 6, false)
  assert.equal(r.next, '  aaa\n  bbb\nccc')
  assert.deepEqual(r.sel, [0, '  aaa\n  bbb'.length])
}

// 3. 选区结尾恰好在行首：不缩进最后的空行
{
  const r = apply('aaa\nbbb', 0, 4, false) // 选中 "aaa\n"，to 指向 b 行首
  assert.equal(r.next, '  aaa\nbbb')
  assert.deepEqual(r.sel, [0, 5])
}

// 4. 行内选区 Tab：缩进光标所在整行
assert.equal(apply('abc', 1, 2, false).next, '  abc')

// 5. 无选区 Shift+Tab：当前行回退一层缩进
assert.equal(apply('  aaa', 5, 5, true).next, 'aaa')

// 6. Shift+Tab 作用于无缩进行：文本不变
assert.equal(apply('aaa', 3, 3, true).next, 'aaa')

// 7. Shift+Tab 多行反缩进：2空格/1 tab/无缩进 各行的处理
{
  const r = apply('  aaa\n\tbbb\nccc', 0, '  aaa\n\tbbb\ncc'.length, true)
  assert.equal(r.next, 'aaa\nbbb\nccc')
}

// 8. 最后一行（无结尾换行）Shift+Tab 也能正确计算块范围
assert.equal(apply('aaa\n  bbb', 9, 9, true).next, 'aaa\nbbb')

console.log('tabIndent self-check OK (8 cases)')
