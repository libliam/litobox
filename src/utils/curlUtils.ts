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
  // 常用开关型参数
  followRedirects?: boolean   // -L 跟随重定向
  insecure?: boolean          // -k 跳过 SSL 证书校验
  includeHeaders?: boolean    // -i 输出包含响应头
  headOnly?: boolean          // -I 仅取响应头
  silent?: boolean            // -s 静默模式（不显示进度）
  showError?: boolean         // -S 静默模式下仍显示错误
  verbose?: boolean           // -v 详细输出
  failOnError?: boolean       // -f HTTP 错误码时返回失败而非输出错误页
  compressed?: boolean        // --compressed 请求压缩响应
  // 带值参数
  outputFile?: string         // -o 输出到文件
  connectTimeout?: string     // --connect-timeout 连接超时（秒）
  proxy?: string              // -x 代理地址
  userAgent?: string          // -A User-Agent
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
  // 开关型参数
  if (p.followRedirects) lines.push('  -L')
  if (p.insecure) lines.push('  -k')
  if (p.includeHeaders) lines.push('  -i')
  if (p.headOnly) lines.push('  -I')
  if (p.silent) lines.push('  -s')
  if (p.showError) lines.push('  -S')
  if (p.verbose) lines.push('  -v')
  if (p.failOnError) lines.push('  -f')
  if (p.compressed) lines.push('  --compressed')
  // 带值参数
  if (p.outputFile?.trim()) lines.push(`  -o ${sq(p.outputFile.trim())}`)
  const ct = parseInt(p.connectTimeout ?? '', 10)
  if (!Number.isNaN(ct) && ct > 0) lines.push(`  --connect-timeout ${ct}`)
  if (p.proxy?.trim()) lines.push(`  -x ${sq(p.proxy.trim())}`)
  if (p.userAgent?.trim()) lines.push(`  -A ${sq(p.userAgent.trim())}`)
  return lines.join(' \\\n')
}
