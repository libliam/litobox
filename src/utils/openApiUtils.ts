import { load as yamlLoad } from 'js-yaml'
import jsf from 'json-schema-faker'
import { schemaToTs } from './schemaUtils'

// ============ 类型定义 ============

export interface OpenApiParameter {
  name: string
  in: 'query' | 'path' | 'header' | 'cookie' | 'body' | 'formData' | 'string'
  required: boolean
  description?: string
  schema: any
}

export interface OpenApiContent {
  mediaType: string
  schema: any
}

export interface OpenApiRequestBody {
  required: boolean
  description?: string
  content: OpenApiContent[]
}

export interface OpenApiResponse {
  status: string
  description?: string
  content: OpenApiContent[]
}

export interface OpenApiOperation {
  method: string
  path: string
  operationId: string
  summary?: string
  description?: string
  tags: string[]
  deprecated: boolean
  parameters: OpenApiParameter[]
  requestBody?: OpenApiRequestBody
  responses: OpenApiResponse[]
}

export interface OpenApiDoc {
  version: '3.x' | '2.0'
  title: string
  description: string
  apiVersion: string
  baseUrl: string
  operations: OpenApiOperation[]
  tagOrder: string[]
  tagCounts: Record<string, number>
}

export interface MockOptions {
  alwaysFakeOptionals: boolean
  useDefaultValue: boolean
}

export interface RequestExampleOptions {
  baseUrl: string
  lang: 'curl' | 'fetch'
}

// ============ 文档解析 ============

/** 解析 OpenAPI 文本（JSON 或 YAML）为对象 */
export function parseOpenApiText(text: string): any {
  const t = text.trim()
  if (!t) throw new Error('内容为空')
  try {
    return JSON.parse(t)
  } catch {
    /* 不是标准 JSON，按 YAML 解析 */
  }
  try {
    return stringifyDates(yamlLoad(t))
  } catch (e: any) {
    throw new Error('YAML 解析失败: ' + (e?.message || e))
  }
}

/** 将 js-yaml 解析出的 Date 对象转为字符串（YAML 裸日期默认转 Date） */
function stringifyDates(v: any): any {
  if (v instanceof Date) return v.toISOString()
  if (Array.isArray(v)) return v.map(stringifyDates)
  if (v && typeof v === 'object') {
    const out: any = {}
    for (const [k, x] of Object.entries(v)) out[k] = stringifyDates(x)
    return out
  }
  return v
}

// ============ $ref 解析 ============

/** 将 $ref 指向的文档节点取出 */
function resolveRef(doc: any, ref: string, cache: Map<string, any>): any {
  if (!ref.startsWith('#/')) throw new Error(`暂不支持外部引用: ${ref}`)
  if (cache.has(ref)) return cache.get(ref)
  const parts = ref.slice(2).split('/')
  let cur: any = doc
  for (const p of parts) {
    if (cur == null) throw new Error(`引用无法解析: ${ref}`)
    cur = cur[decodeURIComponent(p)]
  }
  if (cur == null) throw new Error(`引用不存在: ${ref}`)
  cache.set(ref, cur)
  return cur
}

/**
 * 递归展开 schema 中的全部 $ref，得到无引用的纯 schema 对象
 * @param depth 递归深度上限，截断循环引用（如 A → B → A）
 */
export function resolveSchemaDeep(doc: any, schema: any): any {
  return resolveSchema(doc, schema, 0, new Map())
}

function resolveSchema(doc: any, schema: any, depth: number, cache: Map<string, any>): any {
  if (depth > 25) return { type: 'object', description: '// 循环引用已截断' }
  if (schema === null || typeof schema !== 'object') return schema
  if (Array.isArray(schema)) return schema.map(s => resolveSchema(doc, s, depth + 1, cache))
  if (schema.$ref) {
    const target = resolveRef(doc, schema.$ref, cache)
    const sibling = { ...schema }
    delete sibling.$ref
    // 兄弟字段（如 description）合并进引用目标
    return resolveSchema(doc, { ...target, ...sibling }, depth + 1, cache)
  }
  const out: any = {}
  for (const [k, v] of Object.entries(schema)) {
    if (k === 'properties' || k === 'patternProperties' || k === 'definitions') {
      const sub: any = {}
      for (const [name, subSchema] of Object.entries(v as any)) {
        sub[name] = resolveSchema(doc, subSchema, depth + 1, cache)
      }
      out[k] = sub
    } else if (k === 'items' || k === 'additionalProperties' || k === 'not' || k === 'contains' || k === 'if' || k === 'then' || k === 'else') {
      out[k] = resolveSchema(doc, v, depth + 1, cache)
    } else if (k === 'allOf' || k === 'anyOf' || k === 'oneOf' || k === 'prefixItems') {
      out[k] = (v as any[]).map(s => resolveSchema(doc, s, depth + 1, cache))
    } else {
      out[k] = v
    }
  }
  return out
}

// ============ 接口提取 ============

const HTTP_METHODS = ['get', 'post', 'put', 'delete', 'patch', 'options', 'head', 'trace'] as const

/** 解析 OpenAPI 文档，提取接口清单与元信息 */
export function analyzeOpenApi(doc: any): OpenApiDoc {
  const version: string = doc?.openapi?.startsWith('3.')
    ? '3.x'
    : doc?.swagger === '2.0'
      ? '2.0'
      : ''
  if (!version) throw new Error('不是有效的 OpenAPI 3.x 或 Swagger 2.0 文档')

  const operations: OpenApiOperation[] = []
  const tagCounts: Record<string, number> = {}
  const tagOrder: string[] = []
  const addTag = (tag: string) => {
    if (!tagCounts[tag]) { tagCounts[tag] = 0; tagOrder.push(tag) }
    tagCounts[tag]++
  }

  const paths = doc.paths || {}
  for (const [path, pathItem] of Object.entries<any>(paths)) {
    if (!pathItem || typeof pathItem !== 'object') continue
    const sharedParams: any[] = pathItem.parameters || []
    for (const method of HTTP_METHODS) {
      const op = pathItem[method]
      if (!op || typeof op !== 'object') continue
      const tags: string[] = Array.isArray(op.tags) ? op.tags : []
      if (!tags.length) addTag('未分组')
      else tags.forEach(addTag)
      const parameters = buildParameters(doc, [...sharedParams, ...(op.parameters || [])], version)
      operations.push({
        method: method.toUpperCase(),
        path,
        operationId: op.operationId || `${method}_${path.replace(/[^a-zA-Z0-9]/g, '_')}`,
        summary: op.summary,
        description: op.description,
        tags,
        deprecated: !!op.deprecated,
        parameters,
        requestBody: buildRequestBody(doc, op, version),
        responses: buildResponses(doc, op.responses, version),
      })
    }
  }

  const servers = Array.isArray(doc.servers) ? doc.servers : []
  const baseUrl = servers[0]?.url
    ? String(servers[0].url)
    : doc.schemes?.length && doc.host
      ? `${doc.schemes[0]}://${doc.host}${doc.basePath || ''}`
      : ''

  return {
    version: version as '3.x' | '2.0',
    title: doc.info?.title || '未命名 API',
    description: doc.info?.description || '',
    apiVersion: doc.info?.version || '',
    baseUrl,
    operations,
    tagOrder,
    tagCounts,
  }
}

function buildParameters(_doc: any, params: any[], version: string): OpenApiParameter[] {
  return (params || [])
    .filter((p: any) => p && p.name)
    .map((p: any) => {
      // Swagger 2.0 非 body 参数直接用 type 字段而非 schema
      const schema = version === '2.0'
        ? (p.schema || { type: p.type, format: p.format, enum: p.enum, default: p.default })
        : (p.schema || { type: p.type })
      return {
        name: p.name,
        in: p.in,
        required: !!p.required,
        description: p.description,
        schema,
      } as OpenApiParameter
    })
}

function buildRequestBody(doc: any, op: any, version: string): OpenApiRequestBody | undefined {
  if (version === '3.x') {
    const rb = op.requestBody
    if (!rb) return undefined
    return {
      required: !!rb.required,
      description: rb.description,
      content: buildContent(doc, rb.content),
    }
  }
  // Swagger 2.0：in: body 参数承载请求体
  const bodyParam = (op.parameters || []).find((p: any) => p.in === 'body')
  if (!bodyParam) return undefined
  return {
    required: !!bodyParam.required,
    description: bodyParam.description,
    content: bodyParam.schema ? [{ mediaType: 'application/json', schema: bodyParam.schema }] : [],
  }
}

function buildResponses(doc: any, responses: any, version: string): OpenApiResponse[] {
  if (!responses || typeof responses !== 'object') return []
  return Object.entries(responses).map(([status, resp]: [string, any]) => ({
    status,
    description: resp?.description,
    content: version === '3.x'
      ? buildContent(doc, resp?.content)
      : (resp?.schema ? [{ mediaType: 'application/json', schema: resp.schema }] : []),
  }))
}

function buildContent(_doc: any, content: any): OpenApiContent[] {
  if (!content || typeof content !== 'object') return []
  return Object.entries(content).map(([mediaType, v]: [string, any]) => ({
    mediaType,
    schema: v?.schema || {},
  }))
}

// ============ 展示辅助 ============

/** 格式化 JSON 用于展示 */
export function jsonOf(obj: any): string {
  try {
    return JSON.stringify(obj, null, 2)
  } catch {
    return String(obj)
  }
}

/** Schema 的类型摘要（用于参数表展示） */
export function schemaTypeLabel(schema: any): string {
  const s = schema || {}
  if (s.$ref) return (s.$ref.split('/').pop() || 'ref') as string
  if (Array.isArray(s.type)) return s.type.join(' | ')
  const t = s.type || (s.properties ? 'object' : s.items ? 'array' : 'any')
  if (t === 'array') return `array<${schemaTypeLabel(s.items)}>`
  if (t === 'object' && s.properties) return `object{${Object.keys(s.properties).length}}`
  if (t === 'string' && s.format) return `string(${s.format})`
  if (s.enum) return `${t}(enum)`
  return String(t)
}

/** 从 content 中挑出 JSON 响应/请求体 schema（优先 application/json） */
export function pickJsonContent(content?: OpenApiContent[]): any | undefined {
  if (!content || !content.length) return undefined
  return content.find(c => c.mediaType.includes('json'))?.schema
    ?? content.find(c => c.mediaType === '*/*')?.schema
    ?? content[0].schema
}

/** 从操作响应中挑出首选 JSON schema（优先 2xx） */
export function pickResponseSchema(op: OpenApiOperation): any | undefined {
  const resp = op.responses.find(r => /^2\d\d$/.test(r.status)) || op.responses.find(r => r.status !== 'default') || op.responses[0]
  return resp ? pickJsonContent(resp.content) : undefined
}

/** 未加引号的示例值（用于 URL / header） */
function exampleUrlValue(schema?: any): string {
  const s = schema || {}
  if (s.example !== undefined) return String(s.example)
  if (Array.isArray(s.enum) && s.enum.length) return String(s.enum[0])
  if (s.default !== undefined && typeof s.default !== 'object') return String(s.default)
  switch (s.type) {
    case 'integer':
    case 'number': return s.minimum !== undefined ? String(s.minimum) : '1'
    case 'boolean': return 'true'
    case 'string':
      if (s.format === 'date-time') return '2024-01-01T00:00:00Z'
      if (s.format === 'date') return '2024-01-01'
      if (s.format === 'email') return 'user@example.com'
      if (s.format === 'uri' || s.format === 'url') return 'https://example.com'
      if (s.format === 'uuid') return '00000000-0000-4000-8000-000000000000'
      return 'string'
    default: return 'value'
  }
}

// ============ Mock 生成 ============

/** 依据（已解析的）Schema 生成 Mock 数据 */
export function buildMockFromSchema(doc: any, schema: any, opts: MockOptions): string {
  const resolved = resolveSchemaDeep(doc, schema)
  jsf.option({ alwaysFakeOptionals: opts.alwaysFakeOptionals, useDefaultValue: opts.useDefaultValue })
  const mock = jsf.generate(resolved)
  return JSON.stringify(mock, null, 2)
}

/** 依据接口响应 Schema 生成 Mock 数据 */
export function buildOperationMock(doc: any, op: OpenApiOperation, opts: MockOptions): string {
  const schema = pickResponseSchema(op)
  if (!schema) return '// 该接口没有 JSON 响应 Schema，无法生成 Mock'
  return buildMockFromSchema(doc, schema, opts)
}

// ============ 请求示例 ============

/** 生成接口请求示例（curl / fetch） */
export function buildRequestExample(doc: any, op: OpenApiOperation, opts: RequestExampleOptions): string {
  const base = (opts.baseUrl || '').replace(/\/+$/, '')
  const pathParams = op.parameters.filter(p => p.in === 'path')
  let path = op.path
  for (const p of pathParams) path = path.replace(`{${p.name}}`, exampleUrlValue(p.schema))
  path = path.replace(/\{[^}]+\}/g, '1') // 未定义参数兜底
  const queryParams = op.parameters.filter(p => p.in === 'query')
  const queryStr = queryParams
    .filter(p => !(p.schema && p.schema.type === 'object'))
    .map(p => `${p.name}=${exampleUrlValue(p.schema)}`)
    .join('&')
  const url = base + path + (queryStr ? `?${queryStr}` : '')

  const headerParams = op.parameters.filter(p => p.in === 'header')
  const bodySchema = pickJsonContent(op.requestBody?.content)
  const hasJsonBody = !!bodySchema
  const hasJsonResp = op.responses.some(r => !!pickJsonContent(r.content))

  const headers: [string, string][] = headerParams.map(p => [p.name, exampleUrlValue(p.schema)])
  if (hasJsonBody) headers.push(['Content-Type', 'application/json'])
  if (hasJsonResp) headers.push(['Accept', 'application/json'])

  let body = ''
  if (bodySchema) {
    try {
      body = buildMockFromSchema(doc, bodySchema, { alwaysFakeOptionals: true, useDefaultValue: true })
    } catch {
      body = '{}'
    }
  }

  const comment = op.summary ? `// ${op.summary}\n` : ''

  if (opts.lang === 'curl') {
    const lines = [`${comment}curl -X ${op.method} "${url}"`]
    for (const [name, val] of headers) lines.push(`  -H "${name}: ${val}"`)
    if (body) lines.push(`  -d '${body}'`)
    return lines.join(' \\\n')
  }

  // fetch
  const headerObj = headers.length
    ? `  headers: { ${headers.map(([n, v]) => `"${n}": "${v}"`).join(', ')} },`
    : ''
  const lines: string[] = [`${comment}const res = await fetch("${url}", {`, `  method: "${op.method}",`]
  if (headerObj) lines.push(headerObj)
  if (body) lines.push(`  body: JSON.stringify(${body})`)
  lines.push('});')
  lines.push('const data = await res.json();')
  lines.push('console.log(data);')
  return lines.join('\n')
}

// ============ TS 类型生成 ============

/** Schema 名转 TS 类型名（PascalCase） */
function typeNameOf(name: string): string {
  return String(name).replace(/[^a-zA-Z0-9_$]/g, ' ').split(/\s+/).filter(Boolean)
    .map(s => s[0].toUpperCase() + s.slice(1)).join('') || 'GeneratedType'
}

/** 依据接口响应 Schema 生成 TS 类型 */
export function buildOperationTs(doc: any, op: OpenApiOperation, typeName: string): string {
  const schema = pickResponseSchema(op)
  if (!schema) return '// 该接口没有 JSON 响应 Schema，无法生成 TS 类型'
  const resolved = resolveSchemaDeep(doc, schema)
  return schemaToTs(JSON.stringify(resolved), typeName || `${typeNameOf(op.operationId)}Response`)
}

/** 为文档中所有命名 Schema 生成 TS 类型 */
export function buildAllSchemasTs(doc: any): string {
  const schemas = doc.components?.schemas || doc.definitions || {}
  const names = Object.keys(schemas)
  if (!names.length) return '// 文档中没有命名 Schema（components/schemas 或 definitions）'
  const parts: string[] = []
  for (const name of names) {
    try {
      const resolved = resolveSchemaDeep(doc, schemas[name])
      parts.push(schemaToTs(JSON.stringify(resolved), typeNameOf(name)))
    } catch (e: any) {
      parts.push(`// ${name} 生成失败: ${e?.message || e}`)
    }
  }
  return parts.join('\n\n')
}

// ============ 全量清单导出 ============

/** 导出接口清单摘要（JSON） */
export function exportOperationsJson(doc: any): string {
  const info = analyzeOpenApi(doc)
  return jsonOf({
    title: info.title,
    apiVersion: info.apiVersion,
    version: info.version,
    baseUrl: info.baseUrl,
    operationCount: info.operations.length,
    operations: info.operations.map(op => ({
      method: op.method,
      path: op.path,
      operationId: op.operationId,
      summary: op.summary,
      tags: op.tags,
      deprecated: op.deprecated,
      parameters: op.parameters.map(p => ({ name: p.name, in: p.in, required: p.required, type: schemaTypeLabel(p.schema) })),
    })),
  })
}

// ============ 内置示例 ============

export const EXAMPLE_OPENAPI = `openapi: 3.0.3
info:
  title: PetStore 精简示例
  description: 用于演示 OpenAPI 解析的宠物商店 API
  version: 1.0.0
servers:
  - url: https://api.example.com
tags:
  - name: pets
    description: 宠物相关接口
paths:
  /pets:
    get:
      tags: [pets]
      summary: 查询宠物列表
      operationId: listPets
      parameters:
        - name: limit
          in: query
          description: 返回数量上限
          required: false
          schema:
            type: integer
            format: int32
            maximum: 100
            default: 20
        - name: status
          in: query
          description: 按状态过滤
          required: false
          schema:
            type: string
            enum: [available, pending, sold]
      responses:
        '200':
          description: 宠物列表
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Pet'
        default:
          description: 错误
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
    post:
      tags: [pets]
      summary: 创建宠物
      operationId: createPet
      requestBody:
        required: true
        description: 宠物信息
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/NewPet'
      responses:
        '201':
          description: 创建成功
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Pet'
  /pets/{petId}:
    get:
      tags: [pets]
      summary: 查询宠物详情
      operationId: getPet
      parameters:
        - name: petId
          in: path
          required: true
          description: 宠物 ID
          schema:
            type: integer
            format: int64
      responses:
        '200':
          description: 宠物详情
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Pet'
        '404':
          description: 宠物不存在
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'
    delete:
      tags: [pets]
      summary: 删除宠物
      operationId: deletePet
      parameters:
        - name: petId
          in: path
          required: true
          description: 宠物 ID
          schema:
            type: integer
            format: int64
      responses:
        '204':
          description: 删除成功
components:
  schemas:
    Category:
      type: object
      properties:
        id:
          type: integer
          format: int64
        name:
          type: string
    Pet:
      type: object
      required: [id, name]
      properties:
        id:
          type: integer
          format: int64
        name:
          type: string
        tag:
          type: string
        status:
          type: string
          description: 宠物在商店中的状态
          enum: [available, pending, sold]
        category:
          $ref: '#/components/schemas/Category'
    NewPet:
      type: object
      required: [name]
      properties:
        name:
          type: string
        tag:
          type: string
        category:
          $ref: '#/components/schemas/Category'
    Error:
      type: object
      required: [code, message]
      properties:
        code:
          type: integer
          format: int32
        message:
          type: string
`

// ============ 自检 ============

/** 自检函数：验证解析、接口提取、示例、Mock、TS 生成的核心逻辑 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    const doc = parseOpenApiText(EXAMPLE_OPENAPI)
    const info = analyzeOpenApi(doc)
    if (info.operations.length !== 4) errors.push(`示例接口数异常: ${info.operations.length}`)
    if (!info.baseUrl.includes('api.example.com')) errors.push('baseUrl 提取失败')
    const post = info.operations.find(o => o.method === 'POST')
    if (!post?.requestBody) errors.push('POST 请求体提取失败')
    if (!post || !post.responses.some(r => r.status === '201')) errors.push('响应提取失败')
    // $ref 展开
    const op0 = info.operations.find(o => o.method === 'GET' && !o.path.includes('{'))
    if (op0) {
      const schema = pickResponseSchema(op0)
      const resolved = resolveSchemaDeep(doc, schema)
      const json = jsonOf(resolved)
      if (!json.includes('category')) errors.push('$ref 嵌套展开失败')
    }
    // 请求示例
    const curl = buildRequestExample(doc, info.operations[0], { baseUrl: '', lang: 'curl' })
    if (!curl.includes('curl -X GET')) errors.push('curl 示例生成异常')
    const fetch = buildRequestExample(doc, info.operations[0], { baseUrl: '', lang: 'fetch' })
    if (!fetch.includes('fetch(')) errors.push('fetch 示例生成异常')
    // Mock
    try {
      const mock = buildOperationMock(doc, info.operations[0], { alwaysFakeOptionals: true, useDefaultValue: true })
      const parsed = JSON.parse(mock)
      if (!Array.isArray(parsed)) errors.push('Mock 响应应为数组')
    } catch (e: any) { errors.push('Mock 生成异常: ' + e.message) }
    // TS
    const ts = buildOperationTs(doc, info.operations[0], 'PetList')
    if (!ts.includes('export interface PetList') || !ts.includes('category')) errors.push('TS 生成异常: ' + ts.slice(0, 80))
    const allTs = buildAllSchemasTs(doc)
    if (!allTs.includes('export interface Pet') || !allTs.includes('Category')) errors.push('全量 TS 生成异常')
    // 非法输入
    try {
      parseOpenApiText('{bad json')
      errors.push('非法输入未抛错')
    } catch { /* 预期抛错 */ }
  } catch (e: any) {
    errors.push('自检异常: ' + (e?.message || e))
  }
  return errors
}
