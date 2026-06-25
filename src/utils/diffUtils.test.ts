import { computeLineDiff, computeCharDiff } from './diffUtils'

let passed = 0
let failed = 0

function assert(condition: boolean, message: string) {
  if (condition) {
    passed++
    console.log(`  ✓ ${message}`)
  } else {
    failed++
    console.error(`  ✗ ${message}`)
  }
}

// 测试行级对比 - 相同文本
console.log('Test: equal lines')
{
  const result = computeLineDiff('hello\nworld', 'hello\nworld')
  assert(result.length === 2, '应有2行')
  assert(result[0].type === 'equal', '第1行应相同')
  assert(result[1].type === 'equal', '第2行应相同')
}

// 测试行级对比 - 新增行
console.log('Test: add line')
{
  const result = computeLineDiff('hello', 'hello\nworld')
  assert(result.length === 2, '应有2行')
  assert(result[0].type === 'equal', '第1行应相同')
  assert(result[1].type === 'add', '第2行应新增')
}

// 测试行级对比 - 删除行
console.log('Test: remove line')
{
  const result = computeLineDiff('hello\nworld', 'hello')
  assert(result.length === 2, '应有2行')
  assert(result[0].type === 'equal', '第1行应相同')
  assert(result[1].type === 'remove', '第2行应删除')
}

// 测试行级对比 - 修改行
console.log('Test: modify line')
{
  const result = computeLineDiff('hello\nfoo', 'hello\nbar')
  assert(result.length === 2, '应有2行')
  assert(result[0].type === 'equal', '第1行应相同')
  assert(result[1].type === 'modify', '第2行应修改')
}

// 测试字符级对比
console.log('Test: char diff')
{
  const result = computeCharDiff('hello', 'hallo')
  const types = result.map(c => c.type)
  assert(types.includes('remove'), '应有删除字符')
  assert(types.includes('add'), '应有新增字符')
}

// 测试忽略大小写
console.log('Test: ignore case')
{
  const result = computeLineDiff('Hello', 'hello', { ignoreWhitespace: false, ignoreCase: true })
  assert(result[0].type === 'equal', '忽略大小写时应相同')
}

console.log(`\nResults: ${passed} passed, ${failed} failed`)
if (failed > 0) process.exit(1)
