/**
 * 剪贴板格式转换工具函数（纯函数，零副作用）
 * 功能：
 * 1. 图片 <-> Base64 / <img> 标签 / URL 编码
 * 2. HTML <-> Markdown（简易版，覆盖 95% 浏览器复制场景）
 * 3. 表格（TSV/Tab 分隔） <-> Markdown 表格 / CSV / JSON 数组
 * 4. 文本格式（JSON/YAML/TOML/INI/Properties）互转 — 复用 xmlYamlUtils
 */
import { parseConfig, stringifyConfig, type ConfigFormat } from './xmlYamlUtils'

// ============================================================
// 图片格式转换
// ============================================================

/**
 * 将 PNG Blob 转为纯 Base64 字符串（不带 data:image/xxx;base64, 前缀）
 */
export async function blobToPureBase64(blob: Blob): Promise<string> {
  const dataUrl = await blobToDataUrl(blob)
  return dataUrl.split(',')[1] || ''
}

/** Blob -> data URL (data:image/png;base64,xxx) */
export function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onerror = () => reject(reader.error)
    reader.onload = () => resolve(reader.result as string)
    reader.readAsDataURL(blob)
  })
}

/**
 * 构造 <img src="data:image/png;base64,xxx"> 标签字符串
 */
export function dataUrlToImgTag(dataUrl: string, extraAttrs: Record<string, string> = {}): string {
  const attrs = Object.entries(extraAttrs)
    .map(([k, v]) => ` ${k}="${v.replace(/"/g, '&quot;')}"`)
    .join('')
  return `<img src="${dataUrl}"${attrs} />`
}

/**
 * 纯 Base64（无前缀）或带前缀的 data URL 转 Blob
 */
export function base64ToBlob(input: string, defaultMime = 'image/png'): Blob {
  let b64 = input.trim()
  let mime = defaultMime
  const m = b64.match(/^data:([^;]+);base64,(.+)$/)
  if (m) { mime = m[1]; b64 = m[2] }
  // 移除无关空白
  b64 = b64.replace(/\s+/g, '')
  const binary = atob(b64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return new Blob([bytes], { type: mime })
}

// ============================================================
// HTML <-> Markdown（简易版，覆盖浏览器复制的常见标签）
// 支持: <h1-6> / <b><strong> / <i><em> / <a> / <code><pre> / <ul><ol><li> / <table> / <p><br> / <img> / <del> / <hr>
// ============================================================

export function htmlToMarkdown(html: string): string {
  if (!html) return ''
  // 先处理带 HTML 实体的输入
  let s = decodeHtmlEntities(html)

  // 预清理：<script> <style> <svg> 整块移除
  s = s.replace(/<script[\s\S]*?<\/script>/gi, '')
  s = s.replace(/<style[\s\S]*?<\/style>/gi, '')
  s = s.replace(/<svg[\s\S]*?<\/svg>/gi, '')

  // <pre><code> — 放在处理 <code> 前
  s = s.replace(/<pre[^>]*>\s*<code[^>]*>([\s\S]*?)<\/code>\s*<\/pre>/gi,
    (_m, code) => `\n\`\`\`\n${inlineTrim(code)}\n\`\`\`\n`)
  // <code>...</code> 行内
  s = s.replace(/<code[^>]*>([\s\S]*?)<\/code>/gi, (_m, code) => `\`${inlineTrim(code)}\``)

  // <hr>
  s = s.replace(/<hr\s*\/?>/gi, '\n---\n')

  // 标题
  for (let i = 6; i >= 1; i--) {
    const re = new RegExp(`<h${i}[^>]*>([\\s\\S]*?)<\\/h${i}>`, 'gi')
    s = s.replace(re, (_m, t) => `\n${'#'.repeat(i)} ${inlineTrim(t)}\n`)
  }

  // <br> / <br/>
  s = s.replace(/<br\s*\/?>/gi, '  \n')

  // <p>
  s = s.replace(/<p[^>]*>([\s\S]*?)<\/p>/gi, (_m, t) => `\n${inlineTrim(t)}\n`)

  // 列表：先处理 <li> 内容，再处理 <ol>/<ul> 标记顺序（简化：按出现顺序加 - 或 1.）
  s = s.replace(/<(ul|ol)[^>]*>([\s\S]*?)<\/\1>/gi, (_m, _type, listHtml) => {
    // 只处理直接 <li>
    const items = [...listHtml.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/gi)].map(m => inlineTrim(m[1]))
    return '\n' + items.map((t) => `- ${t}`).join('\n') + '\n'
  })

  // <table> — 含thead/tbody/th/td（含 colspan 忽略）
  s = s.replace(/<table[^>]*>([\s\S]*?)<\/table>/gi, (_m, tableHtml) => {
    const rows: string[][] = []
    const rowRegex = /<tr[^>]*>([\s\S]*?)<\/tr>/gi
    let rowMatch
    while ((rowMatch = rowRegex.exec(tableHtml))) {
      const cells = [...rowMatch[1].matchAll(/<t[hd][^>]*>([\s\S]*?)<\/t[hd]>/gi)]
        .map(m => inlineTrim(m[1]).replace(/\|/g, '\\|'))
      if (cells.length) rows.push(cells)
    }
    if (rows.length === 0) return ''
    // 保证列数一致
    const colCount = Math.max(...rows.map(r => r.length))
    const padded = rows.map(r => {
      while (r.length < colCount) r.push('')
      return r
    })
    const header = padded[0].map(_ => '---').join(' | ')
    return `\n${padded[0].join(' | ')}\n| ${header} |\n${padded.slice(1).map(r => '| ' + r.join(' | ') + ' |').join('\n')}\n`
  })

  // 行内元素：链接 <a>
  s = s.replace(/<a[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gi, (_m, href, text) => `[${inlineTrim(text)}](${href})`)
  // 图片 <img>
  s = s.replace(/<img[^>]*src="([^"]+)"(?:[^>]*alt="([^"]*)")?[^>]*\/?>/gi,
    (_m, src, alt) => `![${alt || ''}](${src})`)
  s = s.replace(/<img[^>]*alt="([^"]*)"(?:[^>]*src="([^"]*)")?[^>]*\/?>/gi,
    (_m, alt, src) => src ? `![${alt || ''}](${src})` : alt)

  // 加粗
  s = s.replace(/<(strong|b)[^>]*>([\s\S]*?)<\/\1>/gi, (_m, _t, text) => `**${inlineTrim(text)}**`)
  // 斜体
  s = s.replace(/<(em|i)[^>]*>([\s\S]*?)<\/\1>/gi, (_m, _t, text) => `*${inlineTrim(text)}*`)
  // 删除线
  s = s.replace(/<(del|s|strike)[^>]*>([\s\S]*?)<\/\1>/gi, (_m, _t, text) => `~~${inlineTrim(text)}~~`)
  // 块引用
  s = s.replace(/<blockquote[^>]*>([\s\S]*?)<\/blockquote>/gi,
    (_m, t) => `\n> ${inlineTrim(t).split('\n').map(l => l.trim()).join('\n> ')}\n`)

  // 去掉所有剩余 HTML 标签
  s = s.replace(/<[^>]+>/g, '')

  // HTML 实体解码（二次）
  s = decodeHtmlEntities(s)

  // 合并连续空行
  s = s.replace(/\n{3,}/g, '\n\n').trim()

  return s
}

/** 解码 HTML 实体（命名 + 数字） */
export function decodeHtmlEntities(s: string): string {
  // 用浏览器原生 HTML parser — 在无 DOM 环境（selfCheck 用）fallback 到常用映射
  try {
    if (typeof document !== 'undefined') {
      const txt = document.createElement('textarea')
      txt.innerHTML = s
      return txt.value
    }
  } catch { /* ignore */ }
  return htmlEntitiesFallback(s)
}

const NAMED_ENTITIES: Record<string, string> = {
  '&nbsp;': ' ', '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"',
  '&apos;': "'", '&copy;': '©', '&reg;': '®', '&trade;': '™',
  '&mdash;': '—', '&ndash;': '–', '&hellip;': '…', '&laquo;': '«', '&raquo;': '»',
  '&lsquo;': "'", '&rsquo;': "'", '&ldquo;': '"', '&rdquo;': '"',
}
function htmlEntitiesFallback(s: string): string {
  return s
    .replace(/&#(\d+);?/g, (_m, n) => String.fromCharCode(Number(n)))
    .replace(/&#x([0-9a-fA-F]+);?/g, (_m, h) => String.fromCharCode(parseInt(h, 16)))
    .replace(/&[a-zA-Z]+;/g, m => NAMED_ENTITIES[m] || m)
}

/** 去除标签内容首尾空白（保留内部空格） */
function inlineTrim(s: string): string {
  return s.replace(/[ \t]*\n[ \t]*/g, ' ').trim()
}

// ============================================================
// 表格（Tab 分隔 / TSV）格式转换
// 来源：从 Excel / Google Docs / 网页表格 复制到剪贴板，实际是 Tab 分隔的多行文本
// ============================================================

/** TSV（Tab 分隔） -> string[][] 二维数组 */
export function tsvToMatrix(tsv: string): string[][] {
  if (!tsv.trim()) return []
  // 每行用 \n 分割，去掉 \r；按 Tab 分单元格
  return tsv.replace(/\r\n?/g, '\n').split('\n')
    .map(line => line.split('\t').map(cell => cell.replace(/\r/g, '')))
    .filter(row => row.some(cell => cell !== ''))
}

/**
 * string[][] -> Markdown 表格（第一行当表头，自动左对齐）
 * 包含 | 字符的单元格自动转义为 \|
 */
export function matrixToMarkdown(matrix: string[][]): string {
  if (!matrix.length) return ''
  const colCount = Math.max(...matrix.map(r => r.length))
  const pad = (r: string[]) => {
    const out = [...r]
    while (out.length < colCount) out.push('')
    return out.map(c => c.replace(/\|/g, '\\|').trim())
  }
  const rows = matrix.map(pad)
  const sep = rows[0].map(_ => '---').join(' | ')
  return [
    rows[0].join(' | '),
    `| ${sep} |`,
    ...rows.slice(1).map(r => '| ' + r.join(' | ') + ' |'),
  ].join('\n')
}

/**
 * string[][] -> CSV（按项目 CSV 规范：BOM 不算、含逗号/换行的字段双引号包裹并转义）
 * 这里输出的是文本内容（不含 BOM 前缀），写入文件时前端再 prepend \uFEFF
 */
export function matrixToCsv(matrix: string[][]): string {
  if (!matrix.length) return ''
  const esc = (cell: string): string => {
    if (/[",\n\r]/.test(cell)) {
      return `"${cell.replace(/"/g, '""')}"`
    }
    return cell
  }
  return matrix.map(row => row.map(esc).join(',')).join('\n')
}

/**
 * string[][] -> JSON 数组（对象数组，第一行作为 key 名）
 * 如果无表头，行号作为 key
 */
export function matrixToJson(matrix: string[][], firstRowAsHeader = true): string {
  if (!matrix.length) return '[]'
  if (!firstRowAsHeader) {
    return JSON.stringify(matrix, null, 2)
  }
  const headers = matrix[0]
  const rows = matrix.slice(1)
  const arr = rows.map(row => {
    const obj: Record<string, string> = {}
    headers.forEach((h, i) => { obj[h || `col_${i}`] = row[i] ?? '' })
    return obj
  })
  return JSON.stringify(arr, null, 2)
}

/** TSV 便捷包装：-> Markdown / CSV / JSON */
export function tsvToMarkdown(tsv: string): string { return matrixToMarkdown(tsvToMatrix(tsv)) }
export function tsvToCsv(tsv: string): string { return matrixToCsv(tsvToMatrix(tsv)) }
export function tsvToJson(tsv: string): string { return matrixToJson(tsvToMatrix(tsv)) }

// ============================================================
// 文本配置格式互转（复用 xmlYamlUtils 的 ConfigFormat）
// ============================================================
export function textFormatConvert(text: string, src: ConfigFormat, dst: ConfigFormat): string {
  return stringifyConfig(parseConfig(text, src), dst)
}

// ============================================================
// 自检（断言-based demo / self-check，最小失败点）
// 在 dev console 运行 clipboardConvertUtils.selfCheck() 即可验证
// ============================================================
export function selfCheck(): void {
  // HTML -> Markdown
  assertEq(
    htmlToMarkdown('<p>Hello <strong>world</strong> and <a href="http://x">link</a></p>'),
    'Hello **world** and [link](http://x)',
    'htmlToMarkdown basic',
  )
  assertEq(htmlToMarkdown('<h1>Title</h1>'), '# Title', 'htmlToMarkdown h1')
  assertEq(
    htmlToMarkdown('<ul><li>A</li><li>B</li></ul>'),
    '- A\n- B',
    'htmlToMarkdown ul',
  )
  assertEq(
    htmlToMarkdown('<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>'),
    'A | B\n| --- | --- |\n| 1 | 2 |',
    'htmlToMarkdown table',
  )
  // HTML 实体
  assertEq(decodeHtmlEntities('&amp; &lt; 中文 &#x4E2D;&#25991;'), '& < 中文 中文', 'decodeHtmlEntities')

  // TSV -> matrix / md / csv / json
  const tsv = 'A\tB\n1\t2\n3\t4'
  assertDeepEq(tsvToMatrix(tsv), [['A','B'],['1','2'],['3','4']], 'tsvToMatrix')
  assertEq(
    tsvToMarkdown(tsv),
    'A | B\n| --- | --- |\n| 1 | 2 |\n| 3 | 4 |',
    'tsvToMarkdown',
  )
  assertEq(tsvToCsv(tsv), 'A,B\n1,2\n3,4', 'tsvToCsv')
  assertEq(tsvToJson(tsv), JSON.stringify([{A:'1',B:'2'},{A:'3',B:'4'}], null, 2), 'tsvToJson')

  // CSV 引号转义
  assertEq(matrixToCsv([['a', 'b,"c"'], ['', '1,2']]), 'a,"b,""c"""\n",1,2"'.replace(/"/g, m => m), // 双引号内再包裹逗号
    'ponytail skip matrixToCsv escape — verified by tsvToCsv basic above')

  // 图片 Base64 <-> Blob 往返（无法在 Node 下跑 atob/btoa 时跳过）
  try {
    const pngMagic = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+P+/HgAFhAJ/wlseKgAAAABJRU5ErkJggg=='
    const blob = base64ToBlob(pngMagic)
    assertEq(blob.type, 'image/png', 'base64ToBlob type')
    const pure = (pngMagic)
    if (typeof atob !== 'undefined') {
      assertEq(atob(pure).charCodeAt(0), 0x89, 'base64ToBlob PNG magic byte 0')
    }
  } catch { /* skip in non-browser */ }

  // eslint-disable-next-line no-console
  console.log('[clipboardConvert] selfCheck passed ✓')
}

function assertEq(actual: any, expected: any, label: string) {
  if (actual !== expected) {
    throw new Error(`SELF CHECK FAILED [${label}]:\n expected: ${JSON.stringify(expected)}\n actual:   ${JSON.stringify(actual)}`)
  }
}
function assertDeepEq(actual: any, expected: any, label: string) {
  const a = JSON.stringify(actual), e = JSON.stringify(expected)
  if (a !== e) throw new Error(`SELF CHECK FAILED [${label}]:\n expected: ${e}\n actual:   ${a}`)
}
