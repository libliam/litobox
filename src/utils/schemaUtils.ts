import Ajv from 'ajv'
import type { ValidateFunction } from 'ajv'
import jsf from 'json-schema-faker'

/** 校验结果：单个问题 */
export interface ValidationIssue {
  path: string
  message: string
  keyword: string
}

/** 校验结果 */
export interface ValidationResult {
  ok: boolean
  issues: ValidationIssue[]
  schemaError?: string
}

/** 校验 JSON 数据是否符合 JSON Schema（draft-07） */
export function validateJsonData(schemaText: string, dataText: string): ValidationResult {
  let schema: any
  let data: any
  try {
    schema = JSON.parse(schemaText)
  } catch (e: any) {
    return { ok: false, issues: [{ path: '/', message: 'Schema 解析失败: ' + e.message, keyword: 'parse' }] }
  }
  try {
    data = JSON.parse(dataText)
  } catch (e: any) {
    return { ok: false, issues: [{ path: '/', message: '数据解析失败: ' + e.message, keyword: 'parse' }] }
  }
  const ajv = new Ajv({ allErrors: true, strict: false })
  let validate: ValidateFunction
  try {
    validate = ajv.compile(schema)
  } catch (e: any) {
    return { ok: false, issues: [], schemaError: 'Schema 无效: ' + e.message }
  }
  const ok = validate(data)
  const issues = (validate.errors || []).map(err => ({
    path: err.instancePath || '/',
    message: err.message || '',
    keyword: err.keyword,
  }))
  return { ok, issues }
}

/** Mock 生成选项 */
export interface MockOptions {
  alwaysFakeOptionals: boolean
  useDefaultValue: boolean
}

/** 依据 Schema 生成 Mock 数据，返回格式化 JSON 字符串 */
export function generateMock(schemaText: string, opts: MockOptions): string {
  const schema = JSON.parse(schemaText)
  jsf.option({ alwaysFakeOptionals: opts.alwaysFakeOptionals, useDefaultValue: opts.useDefaultValue })
  const mock = jsf.generate(schema)
  return JSON.stringify(mock, null, 2)
}

/** 将单个 Schema 子节点转为 TS 类型字符串（嵌套对象内联） */
function tsType(schema: any, indent: number): string {
  const pad = '  '.repeat(indent)
  if ('const' in schema) return JSON.stringify(schema.const)
  if (Array.isArray(schema.enum)) {
    return schema.enum.map((v: any) => JSON.stringify(v)).join(' | ')
  }
  if (schema.allOf) {
    const merged: any = { ...schema }
    const props: Record<string, any> = {}
    const required: string[] = []
    for (const sub of schema.allOf) {
      Object.assign(props, sub.properties || {})
      required.push(...(sub.required || []))
    }
    if (Object.keys(props).length) merged.properties = props
    if (required.length) merged.required = required
    delete merged.allOf
    return tsType(merged, indent)
  }
  if (schema.anyOf || schema.oneOf) {
    return (schema.anyOf || schema.oneOf).map((s: any) => tsType(s, indent)).join(' | ')
  }
  const type = schema.type || (schema.properties ? 'object' : undefined)
  if (Array.isArray(type)) {
    return type.map(t => tsType({ ...schema, type: t }, indent)).join(' | ')
  }
  switch (type) {
    case 'string': return 'string'
    case 'number':
    case 'integer': return 'number'
    case 'boolean': return 'boolean'
    case 'null': return 'null'
    case 'array': {
      const item = schema.items ? tsType(schema.items, indent) : 'any'
      return `(${item})[]`
    }
    case 'object': {
      const props = schema.properties || {}
      const required = schema.required || []
      const keys = Object.keys(props)
      if (keys.length === 0) return 'Record<string, any>'
      const lines = keys.map(k => {
        const opt = required.includes(k) ? '' : '?'
        const t = tsType(props[k], indent + 1)
        return `${pad}  ${/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(k) ? k : JSON.stringify(k)}${opt}: ${t};`
      })
      return `{\n${lines.join('\n')}\n${pad}}`
    }
    default: return 'any'
  }
}

/** 将 JSON Schema 转换为 TypeScript 接口定义 */
export function schemaToTs(schemaText: string, typeName: string): string {
  const schema = JSON.parse(schemaText)
  const name = (typeName || 'GeneratedType').replace(/[^A-Za-z0-9_$]/g, '')
  const isArray = schema.type === 'array'
  const isObject = schema.type === 'object' || schema.properties
  if (isArray) {
    const item = schema.items ? tsType(schema.items, 0) : 'any'
    return `type ${name} = ${item}[]`
  }
  if (!isObject) {
    return `type ${name} = ${tsType(schema, 0)}`
  }
  const props = schema.properties || {}
  const required = schema.required || []
  const lines = Object.keys(props).map(k => {
    const opt = required.includes(k) ? '' : '?'
    const t = tsType(props[k], 1)
    return `  ${/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(k) ? k : JSON.stringify(k)}${opt}: ${t};`
  })
  return `export interface ${name} {\n${lines.join('\n')}\n}`
}

/** 内置示例 Schema */
export const EXAMPLE_SCHEMA = `{
  "type": "object",
  "required": ["id", "name", "email", "tags"],
  "properties": {
    "id": { "type": "integer", "minimum": 1 },
    "name": { "type": "string", "minLength": 2, "maxLength": 20 },
    "email": { "type": "string", "format": "email" },
    "age": { "type": "integer", "minimum": 18, "maximum": 99 },
    "active": { "type": "boolean", "default": true },
    "tags": { "type": "array", "items": { "type": "string" }, "minItems": 1, "maxItems": 5 },
    "address": {
      "type": "object",
      "properties": {
        "city": { "type": "string" },
        "zip": { "type": "string", "pattern": "^[0-9]{6}$" }
      }
    }
  }
}`

/** 与示例 Schema 匹配的示例数据 */
export const EXAMPLE_DATA = `{
  "id": 1,
  "name": "张三",
  "email": "zhangsan@example.com",
  "age": 25,
  "active": true,
  "tags": ["admin", "dev"],
  "address": {
    "city": "北京",
    "zip": "100000"
  }
}`

/** 自检函数：验证校验、Mock、TS 导出的核心逻辑 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    // 正确数据应通过校验
    const ok = validateJsonData(EXAMPLE_SCHEMA, EXAMPLE_DATA)
    if (!ok.ok) errors.push('正确数据校验失败: ' + JSON.stringify(ok.issues.slice(0, 2)))
    // 错误数据应报错
    const bad = validateJsonData(EXAMPLE_SCHEMA, '{"id": "abc", "name": 1}')
    if (bad.ok) errors.push('错误数据应校验失败')
    // Schema 语法错误应返回 schemaError
    const badSchema = validateJsonData('{invalid', '{}')
    if (!badSchema.issues.some(i => i.keyword === 'parse')) errors.push('Schema 解析错误未识别')
  } catch (e: any) { errors.push('校验逻辑异常: ' + e.message) }
  try {
    // Mock 生成
    const mock = generateMock(EXAMPLE_SCHEMA, { alwaysFakeOptionals: true, useDefaultValue: true })
    const parsed = JSON.parse(mock)
    if (typeof parsed.id !== 'number' || typeof parsed.email !== 'string' || !Array.isArray(parsed.tags)) {
      errors.push('Mock 数据结构异常')
    }
  } catch (e: any) { errors.push('Mock 生成异常: ' + e.message) }
  try {
    // TS 导出
    const ts = schemaToTs(EXAMPLE_SCHEMA, 'User')
    if (!ts.includes('export interface User') || !ts.includes('id: number') || !ts.includes('email?: string')) {
      errors.push('TS 导出结构异常: ' + ts.slice(0, 100))
    }
  } catch (e: any) { errors.push('TS 导出异常: ' + e.message) }
  return errors
}
