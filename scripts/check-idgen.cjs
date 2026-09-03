// ID 生成逻辑自检（node scripts/check-idgen.cjs）
// 通过 esbuild 转译真实源码执行断言，不复制生产逻辑
const fs = require('fs')
const path = require('path')
const assert = require('assert')
const esbuild = require('esbuild')

const src = fs.readFileSync(path.join(__dirname, '..', 'src', 'utils', 'uuidUtils.ts'), 'utf8')
const { code } = esbuild.transformSync(src, { format: 'cjs', loader: 'ts' })
const mod = { exports: {} }
new Function('module', 'exports', code)(mod, mod.exports)
const { generateIds } = mod.exports

async function main() {
  // 雪花：63 位以内十进制数字，互不相同
  const snowflakes = await generateIds({ type: 'snowflake', count: 100 })
  assert.ok(snowflakes.every((id) => /^\d{1,19}$/.test(id) && BigInt(id) < 1n << 63n), '雪花ID必须为63位内十进制')
  assert.strictEqual(new Set(snowflakes).size, 100, '雪花ID必须互不相同')

  // ObjectId：24 位小写十六进制，互不相同
  const objectIds = await generateIds({ type: 'objectid', count: 100 })
  assert.ok(objectIds.every((id) => /^[0-9a-f]{24}$/.test(id)), 'ObjectId必须为24位小写十六进制')
  assert.strictEqual(new Set(objectIds).size, 100, 'ObjectId必须互不相同')

  // 自增序列：前缀 + 补零 + 步长
  const seq = await generateIds({ type: 'sequence', count: 3, prefix: 'ORD-', start: 1, step: 2, padLength: 6 })
  assert.deepStrictEqual(seq, ['ORD-000001', 'ORD-000003', 'ORD-000005'], '自增序列格式错误')

  // UUID 兼容：无横线 + 大写
  const uuids = await generateIds({ type: 'uuid', count: 5, uppercase: true, removeDashes: true })
  assert.ok(uuids.every((id) => /^[0-9A-F]{32}$/.test(id)), 'UUID无横线大写格式错误')

  // NanoID：21 字符 URL 安全，互不相同
  const nanoIds = await generateIds({ type: 'nanoid', count: 100 })
  assert.ok(nanoIds.every((id) => /^[A-Za-z0-9_-]{21}$/.test(id)), 'NanoID必须为21字符URL安全字符')
  assert.strictEqual(new Set(nanoIds).size, 100, 'NanoID必须互不相同')

  // ULID：26 字符 Crockford Base32（无 I/L/O/U），互不相同
  const ulids = await generateIds({ type: 'ulid', count: 100 })
  assert.ok(ulids.every((id) => /^[0-9A-HJKMNP-TV-Z]{26}$/.test(id)), 'ULID必须为26字符Crockford Base32')
  assert.strictEqual(new Set(ulids).size, 100, 'ULID必须互不相同')

  // UUID v7：版本位为 7，variant 为 8-b，互不相同
  const uuidV7s = await generateIds({ type: 'uuidv7', count: 100 })
  assert.ok(uuidV7s.every((id) => /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(id)), 'UUID v7格式错误')
  assert.strictEqual(new Set(uuidV7s).size, 100, 'UUID v7必须互不相同')

  // UUID v1：版本位为 1，variant 为 8-b，互不相同
  const uuidV1s = await generateIds({ type: 'uuidv1', count: 100 })
  assert.ok(uuidV1s.every((id) => /^[0-9a-f]{8}-[0-9a-f]{4}-1[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(id)), 'UUID v1格式错误')
  assert.strictEqual(new Set(uuidV1s).size, 100, 'UUID v1必须互不相同')

  // UUID v5：RFC 4122 标准测试向量 + 确定性 + 空命名空间回退 DNS
  const v5 = await generateIds({ type: 'uuidv5', count: 1, name: 'www.example.com', namespace: '6ba7b810-9dad-11d1-80b4-00c04fd430c8' })
  assert.strictEqual(v5[0], '2ed6657d-e927-568b-95e1-2665a8aea6a2', 'UUID v5测试向量不符')
  const v5Again = await generateIds({ type: 'uuidv5', count: 1, name: 'www.example.com', namespace: '6ba7b810-9dad-11d1-80b4-00c04fd430c8' })
  assert.strictEqual(v5Again[0], v5[0], 'UUID v5必须确定性')
  const v5Default = await generateIds({ type: 'uuidv5', count: 1, name: 'www.example.com', namespace: '' })
  assert.strictEqual(v5Default[0], v5[0], 'UUID v5空命名空间应回退DNS')

  // KSUID：27 字符 Base62，互不相同
  const ksuids = await generateIds({ type: 'ksuid', count: 100 })
  assert.ok(ksuids.every((id) => /^[0-9A-Za-z]{27}$/.test(id)), 'KSUID必须为27字符Base62')
  assert.strictEqual(new Set(ksuids).size, 100, 'KSUID必须互不相同')

  // CUID2：前缀 + 24 字符 base36，互不相同
  const cuid2s = await generateIds({ type: 'cuid2', count: 100, prefix: 'user_' })
  assert.ok(cuid2s.every((id) => /^user_[0-9a-z]{24}$/.test(id)), 'CUID2必须为前缀+24字符base36')
  assert.strictEqual(new Set(cuid2s).size, 100, 'CUID2必须互不相同')

  // XID：20 字符 Base32hex，互不相同
  const xids = await generateIds({ type: 'xid', count: 100 })
  assert.ok(xids.every((id) => /^[0-9a-v]{20}$/.test(id)), 'XID必须为20字符Base32hex')
  assert.strictEqual(new Set(xids).size, 100, 'XID必须互不相同')

  console.log('idgen self-check OK')
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
