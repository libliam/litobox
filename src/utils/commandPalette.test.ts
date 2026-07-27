import { filterTools } from './commandPalette'

let passed = 0
let failed = 0

function assert(condition: boolean, message: string) {
  if (condition) {
    passed++
    console.log(`  \u2713 ${message}`)
  } else {
    failed++
    console.error(`  \u2717 ${message}`)
  }
}

// 测试数据（符合 ToolItem 形状，不依赖 @/store 的运行时 import）
const testTools = [
  { id: 'json', name: 'JSON工具', icon: '', iconSvg: '', description: 'JSON格式化压缩校验', keywords: ['json', '格式化', '压缩'] },
  { id: 'hash', name: '哈希计算', icon: '', iconSvg: '', description: 'MD5/SHA 哈希', keywords: ['hash', 'md5', 'sha', '哈希'] },
  { id: 'note', name: '文本编辑器', icon: '', iconSvg: '', description: '草稿本便签', keywords: ['笔记', 'notepad', '草稿'] },
  { id: 'home', name: '首页', icon: '', iconSvg: '', description: '搜索工具最近使用', keywords: ['首页', '搜索', '主页'] },
]

// Test 1: 空查询返回全部
console.log('Test: empty query returns all')
{
  const result = filterTools('', testTools)
  assert(result.length === testTools.length, '空查询返回全部工具')
}

// Test 2: json 命中且 JSON工具排首位
console.log('Test: json query ranks JSON tool first')
{
  const result = filterTools('json', testTools)
  assert(result.length > 0, 'json 有匹配结果')
  assert(result[0].tool.id === 'json', 'JSON工具排首位')
}

// Test 3: 无匹配返回空数组
console.log('Test: no match returns empty')
{
  const result = filterTools('zzz', testTools)
  assert(result.length === 0, 'zzz 无匹配返回空数组')
}

// Test 4: keywords 命中（md5 命中哈希计算）
console.log('Test: keywords match')
{
  const result = filterTools('md5', testTools)
  const hashIdx = result.findIndex(r => r.tool.id === 'hash')
  assert(hashIdx !== -1, 'md5 命中哈希计算工具')
}

// Test 5: name 前缀匹配排序高于 keywords 匹配
console.log('Test: name prefix outranks keywords match')
{
  // '哈希' 命中 hash 工具的 name 前缀（score 50）+ keywords（30）= 80
  const result = filterTools('哈希', testTools)
  assert(result.length > 0, '哈希 有匹配')
  assert(result[0].tool.id === 'hash', '哈希计算排首位')
}

// Test 6: 大小写不敏感
console.log('Test: case insensitive')
{
  const result = filterTools('JSON', testTools)
  assert(result[0].tool.id === 'json', '大写 JSON 也能命中')
}

console.log(`\n${passed} passed, ${failed} failed`)
process.exit(failed > 0 ? 1 : 0)
