// 全局搜索「工具子 tab」逻辑自检：node scripts/selfcheck-cheat-search.ts
// 子项粒度 = 多 tab 工具的每个 tab（按 tab 名称匹配，不搜具体内容）。
import { register } from 'node:module'
import assert from 'node:assert/strict'

const hookCode = `
export function resolve(specifier, context, nextResolve) {
  if (/^\\.\\.?\\/[^.]*$/.test(specifier)) return nextResolve(specifier + '.ts', context)
  return nextResolve(specifier, context)
}
`
register('data:text/javascript,' + encodeURIComponent(hookCode), import.meta.url)

const { matchToolTabEntries, scoreToolTab, searchEntries } =
  await import('../src/utils/cheatSearch.ts')

const pdfTool: any = {
  id: 'pdf', name: 'PDF工具', description: 'PDF 处理', icon: '', iconSvg: '',
  keywords: ['pdf'], category: 'document',
  tabs: [{ id: 'pdfToImages', label: 'PDF转图片' }, { id: 'compress', label: 'PDF压缩' }],
}
const cheatTool: any = {
  id: 'cheatSheet', name: '速查表', description: '常用命令速查', icon: '', iconSvg: '',
  keywords: ['速查表'], category: 'dev',
  tabs: [{ id: 'git', label: 'Git 命令' }],
}
const jsonTool: any = {
  id: 'json', name: 'JSON工具', description: '格式化', icon: '', iconSvg: '',
  keywords: ['json'], category: 'dev',
}
const tools = [pdfTool, cheatTool, jsonTool]

let passed = 0
const ok = (name: string, fn: () => void) => { fn(); passed++; console.log('PASS:', name) }

ok('按 tab 名称命中 git', () => {
  const r = matchToolTabEntries('git', tools)
  assert.equal(r.length, 1)
  assert.equal(r[0].id, 'tab:cheatSheet:git')
  assert.equal(r[0].name, 'Git 命令')
  assert.deepEqual(r[0].tab, { toolId: 'cheatSheet', tabId: 'git' })
})

ok('按 tab 名称包含命中（压缩）', () => {
  const r = matchToolTabEntries('压缩', tools)
  assert.equal(r.length, 1)
  assert.equal(r[0].id, 'tab:pdf:compress')
})

ok('前缀命中多个 tab（pdf）', () => {
  const r = matchToolTabEntries('pdf', tools)
  assert.equal(r.length, 2)
  assert.ok(r.every(x => x.tab!.toolId === 'pdf'))
})

ok('大小写不敏感（GIT）', () => {
  assert.equal(matchToolTabEntries('GIT', tools)[0].id, 'tab:cheatSheet:git')
})

ok('空词不返回子项', () => {
  assert.deepEqual(matchToolTabEntries('', tools), [])
})

ok('无关词无命中', () => {
  assert.deepEqual(matchToolTabEntries('zzzz', tools), [])
})

ok('评分：名称精确 > 前缀 > 包含', () => {
  const sExact = scoreToolTab('PDF压缩', pdfTool, 'PDF压缩')
  const sStart = scoreToolTab('PDF', pdfTool, 'PDF压缩')
  const sIncl = scoreToolTab('压缩', pdfTool, 'PDF压缩')
  assert.ok(sExact >= 100)
  assert.ok(sStart >= 50 && sStart < sExact)
  assert.ok(sIncl >= 25 && sIncl < sStart)
})

ok('searchEntries：tab 命中时带工具入口兜底', () => {
  const r = searchEntries('git', tools)
  const ids = r.map(e => e.id)
  assert.ok(ids.includes('cheatSheet'))
  assert.ok(ids.includes('tab:cheatSheet:git'))
})

ok('searchEntries：命中项携带跳转 tab 信息', () => {
  const r = searchEntries('git', tools)
  const hit = r.find(e => e.id === 'tab:cheatSheet:git')!
  assert.deepEqual(hit.tab, { toolId: 'cheatSheet', tabId: 'git' })
  assert.equal(hit.toolId, 'cheatSheet')
})

ok('searchEntries：空词只返回工具，无子项', () => {
  const r = searchEntries('  ', tools)
  assert.equal(r.length, tools.length)
  assert.ok(r.every(e => !e.tab))
})

ok('searchEntries：工具自身命中（json）不依赖子项', () => {
  const r = searchEntries('json', tools)
  assert.equal(r[0].id, 'json')
})

console.log(`\n全部通过：${passed} 个用例`)
