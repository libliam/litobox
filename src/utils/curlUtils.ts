/** curl 命令构建参数 */
export interface CurlParams {
  method: string
  url: string
  headers: { key: string; value: string }[]
  authType: 'none' | 'basic' | 'bearer'
  basicUser: string
  basicPass: string
  bearerToken: string
  bodyType: 'none' | 'json' | 'form' | 'raw'
  jsonBody: string
  formFields: { key: string; value: string }[]
  rawBody: string
  cookie: string
  timeout: string
}

/** 单引号包裹并转义内部单引号（bash 规则: ' → '\''） */
const sq = (s: string): string => `'${s.replace(/'/g, `'\\''`)}'`

/**
 * 根据参数生成 bash 风格 curl 命令（多行 + \ 续行）。
 * URL 为空时返回空字符串。
 */
export function buildCurlCommand(p: CurlParams): string {
  const u = p.url.trim()
  if (!u) return ''
  const lines: string[] = [`curl -X ${p.method}`]
  // URL 用双引号（JSON.stringify 同时正确处理内部引号/反斜杠转义）
  lines.push(`  ${JSON.stringify(u)}`)
  // 认证
  if (p.authType === 'basic' && p.basicUser) {
    lines.push(`  -u ${sq(`${p.basicUser}:${p.basicPass}`)}`)
  }
  if (p.authType === 'bearer' && p.bearerToken.trim()) {
    lines.push(`  -H ${sq(`Authorization: Bearer ${p.bearerToken.trim()}`)}`)
  }
  // 请求头
  for (const h of p.headers) {
    if (h.key.trim()) lines.push(`  -H ${sq(`${h.key.trim()}: ${h.value}`)}`)
  }
  // 请求体
  if (p.bodyType === 'json' && p.jsonBody.trim()) {
    let body = p.jsonBody
    try { body = JSON.stringify(JSON.parse(body)) } catch { /* 非合法 JSON 时保留原样 */ }
    lines.push(`  -d ${sq(body)}`)
    if (!p.headers.some(h => h.key.trim().toLowerCase() === 'content-type')) {
      lines.push(`  -H ${sq('Content-Type: application/json')}`)
    }
  } else if (p.bodyType === 'form') {
    for (const f of p.formFields) {
      if (f.key.trim()) lines.push(`  --data-urlencode ${sq(`${f.key.trim()}=${f.value}`)}`)
    }
  } else if (p.bodyType === 'raw' && p.rawBody) {
    lines.push(`  --data-raw ${sq(p.rawBody)}`)
  }
  // Cookie
  if (p.cookie.trim()) lines.push(`  -b ${sq(p.cookie.trim())}`)
  // 超时
  const t = parseInt(p.timeout, 10)
  if (!Number.isNaN(t) && t > 0) lines.push(`  --max-time ${t}`)
  return lines.join(' \\\n')
}
