import { parseJsonEntities, parseSqlEntities, generateEntity, ENTITY_LANG_META } from './entityGen'

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

console.log('Test: json object -> entities')
{
  const { entities, error } = parseJsonEntities('{ "id": 1, "name": "tom", "active": true }', 'User')
  assert(!error, 'no error')
  assert(entities.length === 1, 'one entity')
  assert(entities[0].name === 'User', 'root name honors rootName')
  assert(entities[0].fields.map(f => f.type).join(',') === 'int,string,bool', 'infer int/string/bool')
}

console.log('Test: json array merges keys + nullable')
{
  const { entities, error } = parseJsonEntities('[{"id":1,"name":"a"},{"id":2}]', 'Item')
  assert(!error, 'no error')
  const name = entities[0].fields.find(f => f.name === 'name')
  assert(!!name && name.nullable, 'missing key in some element -> nullable')
}

console.log('Test: nested object -> child entity')
{
  const { entities } = parseJsonEntities('{ "id": 1, "address": { "city": "sz", "zip": 518000 } }', 'User')
  assert(entities.length === 2, 'root + nested entity')
  assert(entities[0].fields.some(f => f.nested === 'Address'), 'root references nested entity')
  assert(entities[1].name === 'Address', 'nested entity named from key')
}

console.log('Test: json5 comments + date detection')
{
  const { entities, error } = parseJsonEntities('{ // note\n "created_at": "2024-01-02T03:04:05Z", "birth": "2024-01-02" }', 'Event')
  assert(!error, 'json5 parsed')
  assert(entities[0].fields[0].type === 'datetime', 'iso datetime detected')
  assert(entities[0].fields[1].type === 'date', 'iso date detected')
}

console.log('Test: sql ddl')
{
  const ddl = `CREATE TABLE IF NOT EXISTS \`t_user\` (
    \`id\` bigint(20) NOT NULL AUTO_INCREMENT COMMENT '主键',
    user_name varchar(64) NOT NULL,
    age int(11) DEFAULT NULL,
    created_at datetime NOT NULL,
    remark text,
    PRIMARY KEY (\`id\`),
    KEY idx_name (user_name)
  );`
  const { entities, error } = parseSqlEntities(ddl)
  assert(!error, 'no error')
  assert(entities.length === 1, 'one table')
  assert(entities[0].name === 'TUser', 'table name -> Pascal entity')
  assert(entities[0].fields.length === 5, '5 columns (constraints skipped)')
  const id = entities[0].fields[0]
  assert(id.type === 'int64' && id.primaryKey === true && id.nullable === false, 'bigint pk not null')
  assert(id.comment === '主键', 'column comment captured')
  assert(entities[0].fields[2].nullable === true, 'default null -> nullable')
  assert(entities[0].fields[3].type === 'datetime', 'datetime mapped')
}

console.log('Test: generate all languages without throwing')
for (const meta of ENTITY_LANG_META) {
  const { entities } = parseJsonEntities('{ "user_id": 1, "name": "x", "tags": ["a","b"], "meta": { "k": 1 } }', 'User')
  const code = generateEntity(entities, meta.key)
  assert(code.length > 0 && code.includes('User'), `${meta.label} output non-empty`)
}
{
  const { entities } = parseJsonEntities('{ "id": 1, "name": "x" }', 'User')
  const ts = generateEntity(entities, 'typescript')
  assert(ts.includes('export interface User'), 'ts interface')
  assert(ts.includes('id: number') && ts.includes('name: string'), 'ts field types')
  const go = generateEntity(entities, 'go')
  assert(go.includes('type User struct'), 'go struct')
  assert(go.includes('json:"id"'), 'go json tag')
  const rust = generateEntity(entities, 'rust')
  assert(rust.includes('pub struct User'), 'rust struct')
  const py = generateEntity(entities, 'python')
  assert(py.includes('class User:'), 'python dataclass')
}

console.log('Test: error paths')
{
  assert(!!parseJsonEntities('not json').error, 'invalid json -> error')
  assert(!!parseJsonEntities('123').error, 'primitive -> error')
  assert(!!parseSqlEntities('SELECT 1').error, 'no create table -> error')
}

console.log(`\nResults: ${passed} passed, ${failed} failed`)
if (failed > 0) process.exit(1)
