// 变量命名转换核心逻辑自检（node scripts/nameCaseCheck.ts，Node>=23.6 直接运行）
// 覆盖缩写词边界、中文混合、全风格一致性等易错场景；逻辑变更后必须同步用例。
import { convertName, convertNameText, splitWords } from '../src/utils/nameCaseUtils.ts'

let failed = 0
function eq(actual: string, expected: string, label: string) {
  if (actual === expected) {
    console.log(`  ok  ${label}: ${expected}`)
  } else {
    failed++
    console.error(`FAIL  ${label}: expected "${expected}", got "${actual}"`)
  }
}

console.log('splitWords 缩写/边界拆分:')
eq(splitWords('userID').join(','), 'user,id', 'userID')
eq(splitWords('HTTPServer').join(','), 'http,server', 'HTTPServer')
eq(splitWords('XMLHttpRequest').join(','), 'xml,http,request', 'XMLHttpRequest')
eq(splitWords('HTTP_Server_URL').join(','), 'http,server,url', 'HTTP_Server_URL')
eq(splitWords('用户ID').join(','), '用户,id', '用户ID')
eq(splitWords('user2Name').join(','), 'user2,name', 'user2Name')
eq(splitWords('page1').join(','), 'page1', 'page1')

console.log('convertName 全风格一致性:')
const r1 = convertName('userID')
eq(r1.camel, 'userId', 'camel userID')
eq(r1.pascal, 'UserId', 'pascal userID')
eq(r1.snake, 'user_id', 'snake userID')
eq(r1.scream, 'USER_ID', 'scream userID')
eq(r1.kebab, 'user-id', 'kebab userID')
eq(r1.dot, 'user.id', 'dot userID')
eq(r1.title, 'User Id', 'title userID')

const r2 = convertName('HTTP_Server_URL')
eq(r2.camel, 'httpServerUrl', 'camel HTTP_Server_URL')
eq(r2.pascal, 'HttpServerUrl', 'pascal HTTP_Server_URL')
eq(r2.scream, 'HTTP_SERVER_URL', 'scream HTTP_Server_URL')

const r3 = convertName('foo.bar-baz_qux')
eq(r3.camel, 'fooBarBazQux', '混合分隔符 camel')
eq(r3.kebab, 'foo-bar-baz-qux', '混合分隔符 kebab')
eq(r3.title, 'Foo Bar Baz Qux', '混合分隔符 title')

const r4 = convertName('user name')
eq(r4.camel, 'userName', '空格短语 camel')
eq(r4.title, 'User Name', '空格短语 title')

eq(convertName('用户ID').snake, '用户_id', '中文混合 snake')

console.log('convertNameText 批量:')
const rows = convertNameText('userID\n\nHTTP Server\nuser name')
eq(rows.length.toString(), '3', '跳空行行数')
eq(rows[1].results.camel, 'httpServer', '第二行 camel')
eq(convertNameText('').length.toString(), '0', '空输入')

if (failed) {
  console.error(`\n${failed} 个用例失败`)
  process.exit(1)
}
console.log('\n全部用例通过')
