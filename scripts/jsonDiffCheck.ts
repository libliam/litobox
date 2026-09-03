// JSON 结构化对比核心逻辑自检（node scripts/jsonDiffCheck.ts，Node>=23.6 直接运行）
import { diffJson, formatValue } from '../src/utils/jsonDiffUtils.ts'

let failed = 0
function assert(cond: boolean, label: string) {
  if (cond) {
    console.log(`  ok  ${label}`)
  } else {
    failed++
    console.error(`FAIL  ${label}`)
  }
}

console.log('差异分类与路径:')
const left = JSON.parse('{"name":"旧","b":1,"a":{"x":[1,2,3]},"onlyLeft":true}')
const right = JSON.parse('{"name":"新","b":1,"a":{"x":[1,9,3,4]},"c":null}')
const res = diffJson(left, right)
const summary = res.rows.map(r => `${r.kind}@${r.path}`)
assert(
  summary.join('|') === 'changed@a.x[1]|added@a.x[3]|added@c|changed@name|removed@onlyLeft',
  `五类差异与路径排序: ${summary.join('|')}`,
)
assert(res.identical === false, '非一致')
assert(res.counts.added === 2 && res.counts.removed === 1 && res.counts.changed === 2, '计数 added2/removed1/changed2')

console.log('键顺序无关 + 完全一致:')
const same1 = diffJson(JSON.parse('{"a":1,"b":{"c":[1,2]}}'), JSON.parse('{"b":{"c":[1,2]},"a":1}'))
assert(same1.identical && same1.rows.length === 0, '键乱序视为一致')

console.log('类型不匹配与数组增减:')
const typeMismatch = diffJson(JSON.parse('{"a":{}}'), JSON.parse('{"a":[]}'))
assert(typeMismatch.rows.length === 1 && typeMismatch.rows[0].kind === 'changed' && typeMismatch.rows[0].path === 'a', '对象 vs 数组记为类型变更')
const numStr = diffJson(JSON.parse('{"n":1}'), JSON.parse('{"n":"1"}'))
assert(numStr.rows.length === 1 && numStr.rows[0].kind === 'changed', '1 vs "1" 视为不同')
const arrShrink = diffJson(JSON.parse('[1,2]'), JSON.parse('[1]'))
assert(arrShrink.rows.length === 1 && arrShrink.rows[0].kind === 'removed' && arrShrink.rows[0].path === '[1]', '数组变短记 removed')

console.log('null 处理:')
const nullDiff = diffJson(JSON.parse('{"x":null}'), JSON.parse('{"x":null}'))
assert(nullDiff.identical, 'null 相同不算差异')
const nullAdded = diffJson(JSON.parse('{}'), JSON.parse('{"x":null}'))
assert(nullAdded.rows.length === 1 && nullAdded.rows[0].kind === 'added' && formatValue(nullAdded.rows[0].right) === 'null', '新增 null 值可展示')

console.log('深层嵌套路径:')
const deep = diffJson(JSON.parse('{"a":{"b":[{"c":1}]}}'), JSON.parse('{"a":{"b":[{"c":2}]}}'))
assert(deep.rows.length === 1 && deep.rows[0].path === 'a.b[0].c', `深层路径: ${deep.rows[0].path}`)

if (failed) {
  console.error(`\n${failed} 个用例失败`)
  process.exit(1)
}
console.log('\n全部用例通过')
