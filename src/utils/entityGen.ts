// 代码实体生成：JSON / SQL DDL → 六语言实体代码（Go/Java/Rust/Python/C#/TypeScript）
// ponytail: 类型推断是启发式的（样本值/正则匹配），输出是可直接使用的骨架代码，
// 不做完整类型系统推导；字段命名按各语言惯用风格转换，JSON 里嵌套对象递归生成子实体。

import JSON5 from 'json5'
import { convertName } from './nameCaseUtils'

export type EntityLang = 'go' | 'java' | 'rust' | 'python' | 'csharp' | 'typescript'

export interface EntityLangMeta {
  key: EntityLang
  label: string
  ext: string
  lang: string
}

export const ENTITY_LANG_META: EntityLangMeta[] = [
  { key: 'typescript', label: 'TypeScript', ext: 'ts', lang: 'typescript' },
  { key: 'java', label: 'Java', ext: 'java', lang: 'java' },
  { key: 'go', label: 'Go', ext: 'go', lang: 'go' },
  { key: 'rust', label: 'Rust', ext: 'rs', lang: 'rust' },
  { key: 'python', label: 'Python', ext: 'py', lang: 'python' },
  { key: 'csharp', label: 'C#', ext: 'cs', lang: 'csharp' },
]

export type CanonicalType =
  | 'string' | 'int' | 'int64' | 'float' | 'double' | 'bool'
  | 'datetime' | 'date' | 'time' | 'bytes' | 'any'

export interface EntityField {
  name: string
  type: CanonicalType
  array: boolean
  nested?: string
  nullable: boolean
  comment?: string
  primaryKey?: boolean
}

export interface EntityDef {
  name: string
  comment?: string
  fields: EntityField[]
}

export interface ParseResult {
  entities: EntityDef[]
  error?: string
}

export type InputFormat = 'json' | 'sql'

const isPlainObject = (v: unknown): v is Record<string, any> =>
  !!v && typeof v === 'object' && !Array.isArray(v)

const pascal = (s: string): string => convertName(s).pascal || 'Nested'
const camel = (s: string): string => convertName(s).camel || s
const snake = (s: string): string => convertName(s).snake || s

const DATE_RE = /^\d{4}-\d{2}-\d{2}$/
const DATETIME_RE = /^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}/
const TIME_RE = /^\d{2}:\d{2}(:\d{2})?$/

function inferPrimitive(v: unknown): CanonicalType {
  if (typeof v === 'boolean') return 'bool'
  if (typeof v === 'number') {
    if (!Number.isInteger(v)) return 'double'
    return Math.abs(v) > 2147483647 ? 'int64' : 'int'
  }
  if (typeof v === 'string') {
    if (DATE_RE.test(v)) return 'date'
    if (DATETIME_RE.test(v)) return 'datetime'
    if (TIME_RE.test(v)) return 'time'
    return 'string'
  }
  return 'any'
}

// ---------- JSON 解析 ----------

function ensureEntity(key: string, sample: Record<string, any>, out: Map<string, EntityDef>): string {
  const name = pascal(key)
  if (!out.has(name)) collect(name, sample, out)
  return name
}

function buildField(key: string, val: any, out: Map<string, EntityDef>): EntityField {
  if (Array.isArray(val)) {
    const item = val.find(v => v !== null && v !== undefined)
    if (isPlainObject(item)) {
      return { name: key, type: 'any', array: true, nested: ensureEntity(key, item, out), nullable: false }
    }
    return {
      name: key,
      type: item === undefined ? 'any' : inferPrimitive(item),
      array: true,
      nullable: false,
    }
  }
  if (isPlainObject(val)) {
    return { name: key, type: 'any', array: false, nested: ensureEntity(key, val, out), nullable: false }
  }
  return { name: key, type: inferPrimitive(val), array: false, nullable: val === null || val === undefined }
}

function collect(name: string, sample: Record<string, any>, out: Map<string, EntityDef>, optional?: Set<string>): void {
  const entity: EntityDef = { name, fields: [] }
  out.set(name, entity)
  for (const key of Object.keys(sample)) {
    const field = buildField(key, sample[key], out)
    if (optional?.has(key)) field.nullable = true
    entity.fields.push(field)
  }
}

/** 合并对象数组的键：取每个键的首个非空值作为类型样本，缺失/为空的键标记为可空 */
function mergeObjects(objs: Record<string, any>[]): { sample: Record<string, any>; optional: Set<string> } {
  const sample: Record<string, any> = {}
  for (const o of objs) {
    for (const k of Object.keys(o)) {
      if ((!(k in sample) || sample[k] === null || sample[k] === undefined) && o[k] !== null && o[k] !== undefined) {
        sample[k] = o[k]
      }
    }
  }
  const optional = new Set<string>()
  for (const o of objs) {
    for (const k of Object.keys(sample)) {
      if (!(k in o) || o[k] === null || o[k] === undefined) optional.add(k)
    }
  }
  return { sample, optional }
}

export function parseJsonEntities(text: string, rootName = 'Root'): ParseResult {
  let data: any
  try {
    data = JSON5.parse(text)
  } catch (e) {
    return { entities: [], error: 'JSON 解析失败: ' + (e instanceof Error ? e.message : String(e)) }
  }
  const out = new Map<string, EntityDef>()
  const name = pascal(rootName)
  if (Array.isArray(data)) {
    const objs = data.filter(isPlainObject)
    if (!objs.length) return { entities: [], error: '数组中没有可用的对象元素' }
    const { sample, optional } = mergeObjects(objs)
    collect(name, sample, out, optional)
  } else if (isPlainObject(data)) {
    collect(name, data, out)
  } else {
    return { entities: [], error: '请输入 JSON 对象或对象数组' }
  }
  return { entities: Array.from(out.values()) }
}

// ---------- SQL DDL 解析 ----------

function sqlTypeToCanonical(rawType: string): CanonicalType {
  const t = rawType.toLowerCase()
  if (/^(tinyint|smallint|mediumint|int|integer|serial|bigint|bigserial|number)/.test(t)) {
    return /big/.test(t) ? 'int64' : 'int'
  }
  if (/^(decimal|numeric|money|float|double|real)/.test(t)) {
    return /^(float|real)/.test(t) ? 'float' : 'double'
  }
  if (/^(varchar|char|text|tinytext|mediumtext|longtext|nvarchar|nchar|clob|string|uuid|enum|set)/.test(t)) return 'string'
  if (/^(datetime|timestamp)/.test(t)) return 'datetime'
  if (/^date\b/.test(t)) return 'date'
  if (/^time\b/.test(t)) return 'time'
  if (/^(bool|boolean|bit)/.test(t)) return 'bool'
  if (/^(blob|binary|varbinary|bytea)/.test(t)) return 'bytes'
  return 'any'
}

/** 按顶层逗号切分（忽略括号内和引号内的逗号） */
function splitTopLevel(s: string): string[] {
  const parts: string[] = []
  let depth = 0
  let cur = ''
  let quote: string | null = null
  for (let i = 0; i < s.length; i++) {
    const ch = s[i]
    if (quote) {
      cur += ch
      if (ch === quote && s[i - 1] !== '\\') quote = null
      continue
    }
    if (ch === "'" || ch === '"' || ch === '`') { quote = ch; cur += ch; continue }
    if (ch === '(') depth++
    else if (ch === ')') depth--
    if (ch === ',' && depth === 0) { parts.push(cur); cur = ''; continue }
    cur += ch
  }
  if (cur.trim()) parts.push(cur)
  return parts
}

/** 从 openIdx 处的 '(' 开始，返回配对的括号内部文本 */
function extractParenBody(text: string, openIdx: number): string | null {
  let depth = 0
  let quote: string | null = null
  for (let i = openIdx; i < text.length; i++) {
    const ch = text[i]
    if (quote) {
      if (ch === quote && text[i - 1] !== '\\') quote = null
      continue
    }
    if (ch === "'" || ch === '"' || ch === '`') { quote = ch; continue }
    if (ch === '(') depth++
    else if (ch === ')') {
      depth--
      if (depth === 0) return text.slice(openIdx + 1, i)
    }
  }
  return null
}

const SKIP_LINE_RE = /^(PRIMARY\s+KEY|FOREIGN\s+KEY|UNIQUE|KEY|INDEX|CONSTRAINT|CHECK|FULLTEXT|SPATIAL|PARTITION)\b/i
const COLUMN_RE = /^[`"[\s]*([A-Za-z0-9_$]+)[`"\]]*\s+([A-Za-z]+(?:\s*\([^)]*\))?)\s*([\s\S]*)$/

function parseColumn(line: string): EntityField | null {
  const m = line.match(COLUMN_RE)
  if (!m) return null
  const [, name, rawType, rest] = m
  const primaryKey = /PRIMARY\s+KEY/i.test(rest) || /\bAUTO_INCREMENT\b/i.test(rest)
  const notNull = /NOT\s+NULL/i.test(rest) || primaryKey
  const commentMatch = rest.match(/COMMENT\s+'((?:[^']|'')*)'/i)
  return {
    name,
    type: sqlTypeToCanonical(rawType),
    array: false,
    nullable: !notNull,
    primaryKey,
    comment: commentMatch ? commentMatch[1] : undefined,
  }
}

export function parseSqlEntities(text: string): ParseResult {
  const entities: EntityDef[] = []
  const tableRe = /CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?[`"[]?([A-Za-z0-9_.]+)[`"\]]?\s*\(/gi
  let m: RegExpExecArray | null
  while ((m = tableRe.exec(text))) {
    const rawName = m[1].split('.').pop() || m[1]
    const body = extractParenBody(text, m.index + m[0].length - 1)
    if (!body) continue
    const entity: EntityDef = { name: pascal(rawName), fields: [] }
    for (const raw of splitTopLevel(body)) {
      const line = raw.trim()
      if (!line || SKIP_LINE_RE.test(line)) continue
      const col = parseColumn(line)
      if (col) entity.fields.push(col)
    }
    if (entity.fields.length) entities.push(entity)
  }
  if (!entities.length) return { entities: [], error: '未找到可解析的 CREATE TABLE 语句' }
  return { entities }
}

// ---------- 代码生成 ----------

const TYPE_MAP: Record<EntityLang, Record<CanonicalType, string>> = {
  go: {
    string: 'string', int: 'int', int64: 'int64', float: 'float32', double: 'float64', bool: 'bool',
    datetime: 'time.Time', date: 'time.Time', time: 'time.Time', bytes: '[]byte', any: 'any',
  },
  java: {
    string: 'String', int: 'Integer', int64: 'Long', float: 'Float', double: 'Double', bool: 'Boolean',
    datetime: 'LocalDateTime', date: 'LocalDate', time: 'LocalTime', bytes: 'byte[]', any: 'Object',
  },
  rust: {
    string: 'String', int: 'i32', int64: 'i64', float: 'f32', double: 'f64', bool: 'bool',
    datetime: 'String', date: 'String', time: 'String', bytes: 'Vec<u8>', any: 'serde_json::Value',
  },
  python: {
    string: 'str', int: 'int', int64: 'int', float: 'float', double: 'float', bool: 'bool',
    datetime: 'datetime', date: 'date', time: 'time', bytes: 'bytes', any: 'Any',
  },
  csharp: {
    string: 'string', int: 'int', int64: 'long', float: 'float', double: 'double', bool: 'bool',
    datetime: 'DateTime', date: 'DateOnly', time: 'TimeOnly', bytes: 'byte[]', any: 'object',
  },
  typescript: {
    string: 'string', int: 'number', int64: 'number', float: 'number', double: 'number', bool: 'boolean',
    datetime: 'string', date: 'string', time: 'string', bytes: 'string', any: 'any',
  },
}

function goType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.go[f.type]
  if (f.array) return `[]${base}`
  const pointer = f.nested ? true : f.type !== 'any' && f.type !== 'bytes'
  return f.nullable && pointer ? `*${base}` : base
}

function javaType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.java[f.type]
  return f.array ? `List<${base}>` : base
}

function rustType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.rust[f.type]
  const full = f.array ? `Vec<${base}>` : base
  return f.nullable ? `Option<${full}>` : full
}

function pyType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.python[f.type]
  const full = f.array ? `List[${base}]` : base
  return f.nullable ? `Optional[${full}]` : full
}

function csType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.csharp[f.type]
  const full = f.array ? `List<${base}>` : base
  return f.nullable ? `${full}?` : full
}

function tsType(f: EntityField): string {
  const base = f.nested ?? TYPE_MAP.typescript[f.type]
  const full = f.array ? `${base}[]` : base
  return f.nullable ? `${full} | null` : full
}

/** 生成合法标识符：非法字符按词分割后拼接，非法开头补前缀 */
function safeIdent(name: string, style: 'camel' | 'pascal' | 'snake' | 'raw'): string {
  let id = style === 'camel' ? camel(name) : style === 'pascal' ? pascal(name) : style === 'snake' ? snake(name) : name
  if (!id) id = 'field'
  if (/^[0-9]/.test(id)) id = '_' + id
  return id
}

const docLines = (comment: string | undefined, prefix: string): string[] =>
  comment ? [prefix + comment.replace(/\r?\n/g, ' ')] : []

function genGo(entities: EntityDef[]): string {
  const needsTime = entities.some(e => e.fields.some(f => !f.nested && ['datetime', 'date', 'time'].includes(f.type)))
  const lines: string[] = ['package model', '']
  if (needsTime) lines.push('import "time"', '')
  for (const e of entities) {
    lines.push(...docLines(e.comment, '// ' + e.name + ' '))
    lines.push(`type ${e.name} struct {`)
    const rows = e.fields.map(f => ({ name: safeIdent(f.name, 'pascal'), type: goType(f), tag: `json:"${f.name}"`, comment: f.comment }))
    const nameW = Math.max(0, ...rows.map(r => r.name.length))
    const typeW = Math.max(0, ...rows.map(r => r.type.length))
    for (const r of rows) {
      const line = `\t${r.name.padEnd(nameW)} ${r.type.padEnd(typeW)} \`${r.tag}\``
      lines.push(r.comment ? `${line} // ${r.comment}` : line)
    }
    lines.push('}', '')
  }
  return lines.join('\n').trimEnd() + '\n'
}

function genJava(entities: EntityDef[]): string {
  const types = new Set(entities.flatMap(e => e.fields.map(javaType)))
  const imports = ['java.util.List']
  for (const t of ['LocalDateTime', 'LocalDate', 'LocalTime']) {
    if (types.has(t)) imports.push(`java.time.${t}`)
  }
  const lines: string[] = [...imports.sort().map(i => `import ${i};`), '']
  for (const e of entities) {
    lines.push(...docLines(e.comment, '// '))
    lines.push(`public class ${e.name} {`)
    for (const f of e.fields) {
      lines.push(...docLines(f.comment, '    // '))
      lines.push(`    public ${javaType(f)} ${safeIdent(f.name, 'camel')};`)
    }
    lines.push('}', '')
  }
  return lines.join('\n').trimEnd() + '\n'
}

function genRust(entities: EntityDef[]): string {
  const lines: string[] = ['use serde::{Deserialize, Serialize};', '']
  for (const e of entities) {
    lines.push(...docLines(e.comment, '/// '))
    lines.push('#[derive(Debug, Clone, Serialize, Deserialize)]')
    lines.push(`pub struct ${e.name} {`)
    for (const f of e.fields) {
      const name = safeIdent(f.name, 'snake')
      lines.push(...docLines(f.comment, '    /// '))
      if (name !== f.name) lines.push(`    #[serde(rename = "${f.name}")]`)
      lines.push(`    pub ${name}: ${rustType(f)},`)
    }
    lines.push('}', '')
  }
  return lines.join('\n').trimEnd() + '\n'
}

function genPython(entities: EntityDef[]): string {
  const types = new Set(entities.flatMap(e => e.fields.map(pyType)))
  const typing: string[] = []
  if ([...types].some(t => t.includes('Any'))) typing.push('Any')
  if ([...types].some(t => t.includes('List['))) typing.push('List')
  if ([...types].some(t => t.includes('Optional['))) typing.push('Optional')
  const lines: string[] = ['from dataclasses import dataclass']
  if (typing.length) lines.push(`from typing import ${typing.join(', ')}`)
  const dt = ['datetime', 'date', 'time'].filter(t => types.has(t))
  if (dt.length) lines.push(`from datetime import ${dt.join(', ')}`)
  lines.push('')
  for (const e of entities) {
    lines.push('@dataclass')
    lines.push(`class ${e.name}:`)
    if (!e.fields.length) {
      lines.push('    pass')
    } else {
      if (e.comment) lines.push(`    """${e.comment}"""`)
      for (const f of e.fields) {
        lines.push(...docLines(f.comment, '    # '))
        lines.push(`    ${safeIdent(f.name, 'snake')}: ${pyType(f)}`)
      }
    }
    lines.push('')
  }
  return lines.join('\n').trimEnd() + '\n'
}

function genCsharp(entities: EntityDef[]): string {
  const types = new Set(entities.flatMap(e => e.fields.map(csType)))
  const lines: string[] = ['using System;']
  if ([...types].some(t => t.startsWith('List<'))) lines.push('using System.Collections.Generic;')
  lines.push('')
  for (const e of entities) {
    lines.push(...docLines(e.comment, '// '))
    lines.push(`public class ${e.name}`)
    lines.push('{')
    for (const f of e.fields) {
      lines.push(...docLines(f.comment, '    // '))
      lines.push(`    public ${csType(f)} ${safeIdent(f.name, 'pascal')} { get; set; }`)
    }
    lines.push('}', '')
  }
  return lines.join('\n').trimEnd() + '\n'
}

function genTypescript(entities: EntityDef[]): string {
  const lines: string[] = []
  for (const e of entities) {
    lines.push(...docLines(e.comment, '/** ').map(l => (l.endsWith(' */') ? l : l + ' */')))
    lines.push(`export interface ${e.name} {`)
    for (const f of e.fields) {
      const name = /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(f.name) ? f.name : JSON.stringify(f.name)
      lines.push(...docLines(f.comment, '  /** ').map(l => (l.endsWith(' */') ? l : l + ' */')))
      lines.push(`  ${name}${f.nullable ? '?' : ''}: ${tsType(f)}`)
    }
    lines.push('}', '')
  }
  return lines.join('\n').trimEnd() + '\n'
}

const GENERATORS: Record<EntityLang, (entities: EntityDef[]) => string> = {
  go: genGo,
  java: genJava,
  rust: genRust,
  python: genPython,
  csharp: genCsharp,
  typescript: genTypescript,
}

export function generateEntity(entities: EntityDef[], lang: EntityLang): string {
  if (!entities.length) return ''
  return GENERATORS[lang](entities)
}

export interface GenOptions {
  format: InputFormat
  rootName: string
}

/** 一站式：解析输入 → 按所选语言生成代码，返回每语言代码 + 错误 */
export function generateFromInput(text: string, opts: GenOptions, langs: EntityLang[]): { code: Record<string, string>; entities: EntityDef[]; error?: string } {
  const parsed = opts.format === 'sql' ? parseSqlEntities(text) : parseJsonEntities(text, opts.rootName)
  if (parsed.error) return { code: {}, entities: [], error: parsed.error }
  const code: Record<string, string> = {}
  for (const lang of langs) code[lang] = generateEntity(parsed.entities, lang)
  return { code, entities: parsed.entities }
}
