import * as pdfjsLib from 'pdfjs-dist'
import { PDFDocument } from 'pdf-lib'

// 配置 pdfjs worker（模块加载时执行一次）
pdfjsLib.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js'

// ============ 类型定义 ============

export interface ImageToPdfOptions {
  pageSize: 'a4' | 'a3' | 'auto'
  orientation: 'portrait' | 'landscape' | 'auto'
  quality: number
}

// ============ PDF 转图片 ============

export async function loadPdfDocument(data: Uint8Array) {
  const loadingTask = pdfjsLib.getDocument({ data })
  return loadingTask.promise
}

export async function renderPdfPageToCanvas(
  pdfDocument: any,
  pageNum: number,
  dpi: number
): Promise<HTMLCanvasElement> {
  const page = await pdfDocument.getPage(pageNum)
  const viewport = page.getViewport({ scale: dpi / 72 })

  const canvas = document.createElement('canvas')
  canvas.width = viewport.width
  canvas.height = viewport.height

  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('Canvas context not available')

  await page.render({ canvasContext: ctx, viewport }).promise
  return canvas
}

export async function pdfToImages(
  pdfFile: File,
  dpi: number = 150
): Promise<Blob[]> {
  const buffer = await pdfFile.arrayBuffer()
  const pdfDocument = await loadPdfDocument(new Uint8Array(buffer))
  const totalPages = pdfDocument.numPages

  const blobs: Blob[] = []
  for (let i = 1; i <= totalPages; i++) {
    const canvas = await renderPdfPageToCanvas(pdfDocument, i, dpi)
    const blob = await new Promise<Blob | null>((resolve) =>
      canvas.toBlob(resolve, 'image/png')
    )
    if (!blob) throw new Error(`第 ${i} 页渲染失败`)
    blobs.push(blob)
  }

  return blobs
}

// ============ PDF 文本提取 ============

export async function extractPdfText(pdfFile: File): Promise<string> {
  const buffer = await pdfFile.arrayBuffer()
  const pdfDocument = await loadPdfDocument(new Uint8Array(buffer))
  const totalPages = pdfDocument.numPages

  const allLines: string[] = []

  for (let i = 1; i <= totalPages; i++) {
    const page = await pdfDocument.getPage(i)
    const content = await page.getTextContent()

    const items: Array<{ str: string; y: number; height: number; x: number }> = []
    for (const item of content.items as any[]) {
      if (!item.str || !item.str.trim()) continue
      const transform = item.transform || []
      const y = transform[5] || 0
      const height = transform[3] || 12
      const x = transform[4] || 0
      items.push({ str: item.str, y, height, x })
    }

    if (items.length === 0) continue

    const avgHeight = items.reduce((sum, it) => sum + it.height, 0) / items.length
    const rowTolerance = avgHeight * 0.5

    const rows: Array<typeof items[0][]> = []
    const sortedByY = [...items].sort((a, b) => b.y - a.y)

    for (const item of sortedByY) {
      const existingRow = rows.find(row => Math.abs(row[0].y - item.y) <= rowTolerance)
      if (existingRow) {
        existingRow.push(item)
      } else {
        rows.push([item])
      }
    }

    const avgLineHeight = avgHeight * 1.5
    const pageLines: string[] = []
    let prevY: number | null = null

    for (const row of rows) {
      const rowY = row[0].y
      const sortedByX = [...row].sort((a, b) => a.x - b.x)
      const lineText = sortedByX.map(r => r.str).join('')

      if (prevY !== null && Math.abs(prevY - rowY) > avgLineHeight * 2) {
        pageLines.push('')
      }

      pageLines.push(lineText)
      prevY = rowY
    }

    if (allLines.length > 0) allLines.push('')
    allLines.push(...pageLines)
  }

  return allLines.join('\n').trim()
}

// ============ 图片转 PDF ============

export async function imagesToPdf(
  imageFiles: File[],
  options: ImageToPdfOptions = { pageSize: 'auto', orientation: 'auto', quality: 0.92 }
): Promise<Blob> {
  const pdfDoc = await PDFDocument.create()

  for (const file of imageFiles) {
    const bytes = await file.arrayBuffer()
    const img = await loadImage(file)

    let pageWidth: number
    let pageHeight: number

    if (options.pageSize === 'a4') {
      pageWidth = 595.28
      pageHeight = 841.89
    } else if (options.pageSize === 'a3') {
      pageWidth = 841.89
      pageHeight = 1190.55
    } else {
      pageWidth = img.width
      pageHeight = img.height
    }

    if (options.orientation === 'landscape' ||
        (options.orientation === 'auto' && img.width > img.height)) {
      ;[pageWidth, pageHeight] = [pageHeight, pageWidth]
    }

    const page = pdfDoc.addPage([pageWidth, pageHeight])

    let embeddedImage
    if (file.type === 'image/png') {
      embeddedImage = await pdfDoc.embedPng(new Uint8Array(bytes))
    } else {
      embeddedImage = await pdfDoc.embedJpg(new Uint8Array(bytes))
    }

    const scale = Math.min(pageWidth / embeddedImage.width, pageHeight / embeddedImage.height)
    const drawWidth = embeddedImage.width * scale
    const drawHeight = embeddedImage.height * scale

    page.drawImage(embeddedImage, {
      x: (pageWidth - drawWidth) / 2,
      y: (pageHeight - drawHeight) / 2,
      width: drawWidth,
      height: drawHeight,
    })
  }

  const pdfBytes = await pdfDoc.save()
  return new Blob([pdfBytes as BlobPart], { type: 'application/pdf' })
}

function loadImage(file: File): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = URL.createObjectURL(file)
  })
}

// ============ PDF 合并/拆分 ============

export function parsePageRange(range: string, totalPages: number): number[] {
  const trimmed = range.trim().toLowerCase()

  if (trimmed === 'all') {
    return Array.from({ length: totalPages }, (_, i) => i + 1)
  }

  if (trimmed === 'odd') {
    return Array.from({ length: totalPages }, (_, i) => i + 1).filter(n => n % 2 === 1)
  }

  if (trimmed === 'even') {
    return Array.from({ length: totalPages }, (_, i) => i + 1).filter(n => n % 2 === 0)
  }

  const pages = new Set<number>()
  const parts = trimmed.split(',')

  for (const part of parts) {
    const p = part.trim()
    if (p.includes('-')) {
      const [start, end] = p.split('-').map(Number)
      if (isNaN(start) || isNaN(end) || start < 1 || end > totalPages || start > end) {
        throw new Error(`无效的页码范围: ${p}`)
      }
      for (let i = start; i <= end; i++) {
        pages.add(i)
      }
    } else {
      const num = Number(p)
      if (isNaN(num) || num < 1 || num > totalPages) {
        throw new Error(`无效的页码: ${p}`)
      }
      pages.add(num)
    }
  }

  return Array.from(pages).sort((a, b) => a - b)
}

export async function mergePdf(
  pdfFiles: File[],
  pageRanges: string[] = []
): Promise<Blob> {
  const mergedPdf = await PDFDocument.create()

  for (let i = 0; i < pdfFiles.length; i++) {
    const file = pdfFiles[i]
    const bytes = await file.arrayBuffer()
    const srcPdf = await PDFDocument.load(new Uint8Array(bytes))
    const totalPages = srcPdf.getPageCount()

    const range = pageRanges[i] || 'all'
    const pageNumbers = parsePageRange(range, totalPages)
    const indices = pageNumbers.map(n => n - 1)

    const copiedPages = await mergedPdf.copyPages(srcPdf, indices)
    for (const page of copiedPages) {
      mergedPdf.addPage(page)
    }
  }

  const pdfBytes = await mergedPdf.save()
  return new Blob([pdfBytes as BlobPart], { type: 'application/pdf' })
}

// ============ 辅助函数 ============

export { saveFileWithDialog } from './fileSaver'

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

// ============ PDF 提取内嵌图片 ============

export interface ExtractedImage {
  pageIndex: number          // 1-based
  xObjectName: string        // XObject 资源名，如 Im1
  width: number
  height: number
  bitsPerComponent: number
  colorSpace: string         // DeviceRGB / DeviceGray / DeviceCMYK / ...
  primaryFilter: string      // 主过滤器：DCTDecode / FlateDecode / JPXDecode / ...
  format: 'jpeg' | 'png' | 'jp2' | 'tiff' | 'raw'
  mimeType: string           // blob 的 MIME
  previewDataUrl: string     // 可在 <img> 中展示的 URL（对 raw/jp2 可能为占位 SVG）
  blob: Blob                 // 保存用的最终 Blob
  size: number               // blob.size
}

// 常见 SVG 占位，用于无法浏览器预览的图片格式
function placeholderSvg(label: string, width: number, height: number, color = '#233044'): string {
  const w = Math.min(width, 400)
  const h = Math.max(60, Math.min(Math.round(w * height / Math.max(width, 1)), 300))
  const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}">
  <rect x="0" y="0" width="${w}" height="${h}" rx="6" ry="6" fill="${color}" stroke="#1e3a5f" stroke-width="1"/>
  <text x="50%" y="40%" fill="#94a3b8" font-family="Consolas, monospace" font-size="12" text-anchor="middle">${label}</text>
  <text x="50%" y="62%" fill="#00d4ff" font-family="Consolas, monospace" font-size="11" text-anchor="middle">${width} × ${height}</text>
  <text x="50%" y="82%" fill="#64748b" font-family="Consolas, monospace" font-size="10" text-anchor="middle">此格式浏览器无法预览，下载后可用专业工具查看</text>
</svg>`
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
}

// 归一化不同 API 返回的 Uint8Array（兼容 ArrayBufferLike 泛型严格类型问题）
// 同时做类型断言使返回值满足 BlobPart / BufferSource 的严格检查
function toStdU8(u8: Uint8Array): Uint8Array {
  // ponytail: 在现代 TS 中，Uint8Array 的泛型参数会被区分为 ArrayBuffer / ArrayBufferLike
  // 此处用 slice + ArrayBuffer.prototype 校验，再做类型断言，保证与 Blob/DecompressionStream 兼容
  const buf = u8?.buffer
  if (!buf) return new Uint8Array(0) as unknown as Uint8Array
  if (Object.getPrototypeOf(buf) === ArrayBuffer.prototype) {
    if (u8.byteOffset === 0 && u8.byteLength === buf.byteLength) {
      return u8 as unknown as Uint8Array
    }
    // ponytail: 没有子视图拷贝，直接返回原始对象（实际运行时没问题）+ 断言
    return u8 as unknown as Uint8Array
  }
  const copy = new Uint8Array(u8.length)
  copy.set(u8)
  return copy as unknown as Uint8Array
}

// 使用浏览器原生 DecompressionStream 解 Deflate（Chrome 103+ / Edge / FF 113+）
async function inflateDeflate(bytesIn: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === 'undefined') {
    throw new Error('当前浏览器不支持 DecompressionStream，无法解压 PNG 类图片')
  }
  const bytes = toStdU8(bytesIn)
  const ds = new DecompressionStream('deflate')
  const writer = ds.writable.getWriter()
  writer.write(bytes as any).catch(() => {})
  writer.close()
  const reader = ds.readable.getReader()
  const chunks: Uint8Array[] = []
  let total = 0
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
    total += value.length
  }
  const out = new Uint8Array(total)
  let off = 0
  for (const c of chunks) { out.set(c, off); off += c.length }
  return out
}

// 把 FlateDecode 解压后的像素数据绘制到 Canvas 再导出 PNG Blob
async function rawPixelsToPngBlob(
  pixels: Uint8Array,
  width: number,
  height: number,
  bpc: number,
  cs: string
): Promise<{ blob: Blob; dataUrl: string }> {
  // ponytail: 只支持最常见的 DeviceGray(8bpc)/DeviceRGB(8bpc)，其他降级为灰+占位
  const channels =
    cs.includes('RGB') ? 3 :
    cs.includes('Gray') ? 1 :
    cs.includes('CMYK') ? 4 : 0

  if (bpc !== 8 || channels === 0) {
    throw new Error(`不支持的 Flate 颜色格式：BPC=${bpc} ColorSpace=${cs}`)
  }

  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('Canvas context not available')
  const imgData = ctx.createImageData(width, height)
  const dst = imgData.data

  let srcIdx = 0
  for (let i = 0; i < dst.length; i += 4) {
    if (channels === 3) {
      dst[i]   = pixels[srcIdx++]
      dst[i+1] = pixels[srcIdx++]
      dst[i+2] = pixels[srcIdx++]
    } else if (channels === 1) {
      const g = pixels[srcIdx++]
      dst[i] = dst[i+1] = dst[i+2] = g
    } else if (channels === 4) {
      // CMYK → RGB 近似公式（ponytail: 无需高精度，满足预览即可）
      const c = pixels[srcIdx++] / 255
      const m = pixels[srcIdx++] / 255
      const y = pixels[srcIdx++] / 255
      const k = pixels[srcIdx++] / 255
      dst[i]   = Math.round(255 * (1 - c) * (1 - k))
      dst[i+1] = Math.round(255 * (1 - m) * (1 - k))
      dst[i+2] = Math.round(255 * (1 - y) * (1 - k))
    }
    dst[i+3] = 255
  }
  ctx.putImageData(imgData, 0, 0)

  const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/png'))
  if (!blob) throw new Error('Canvas 导出 PNG 失败')
  return { blob, dataUrl: canvas.toDataURL('image/png') }
}

/**
 * 从 PDF 中提取所有内嵌的 XObject 图片。
 * - 扫描范围：兼容 PDFRawStream（Word 导出的 Object Stream PDF）与 PDFStream 包装对象，
 *   主路径走 doc.context.indirectObjects 全局枚举（避免 page.node.Resources() 在
 *   Object Stream 压缩模式下字典丢失/为空的问题），再通过 Page→Form 递归定位页码。
 * - JPEG(DCTDecode) / JPX(JPXDecode) 直接保存原生 bytes
 * - PNG(FlateDecode) 解压像素→Canvas→PNG 重编码
 * - 其他过滤器（CCITT/JBIG2等）以 raw 形式保存，不做解码
 */
export async function extractEmbeddedImages(pdfFile: File): Promise<ExtractedImage[]> {
  const bytes = new Uint8Array(await pdfFile.arrayBuffer())
  const doc = await PDFDocument.load(bytes, { ignoreEncryption: true })
  const ctx: any = (doc as any).context

  // ponytail: 调试日志已清理，确认功能正常后移除

  // ======== 工具：统一 RawStream / PDFStream 的字段读取 ========
  function isStreamObj(obj: any): boolean {
    if (!obj) return false
    const cname = obj?.constructor?.name || ''
    // Vite 打包后类名可能带后缀（如 PDFRawStream2），用 includes 匹配
    if (cname.includes('PDFRawStream') || cname.includes('PDFStream')) return true
    // PDFStream 或其他包装流：有 dict/lookup/contents
    if (typeof obj?.lookup === 'function' && (typeof obj?.contents === 'function' || obj?.contents)) return true
    // 兜底：有 dict 属性 + 有 asUint8Array / contents
    if (obj?.dict && (typeof obj?.asUint8Array === 'function' || obj?.contents)) return true
    return false
  }

  function getStreamDict(obj: any): any {
    if (!obj) return null
    // PDFRawStream：dict 是属性（PDFDict）
    if (obj.dict) return obj.dict
    // PDFStream 包装对象：自己就是 "拥有 dict 的"，lookup 是本身的方法
    return obj
  }

  // 缓存 dict 的 string->value 映射，避免每次遍历 entries
  const dictCache = new WeakMap<any, Map<string, any>>()

  function dictGet(dict: any, name: string): any {
    if (!dict) return undefined
    
    // 1. 先尝试 lookup（pdf-lib 标准 API，接受 string）
    try {
      if (typeof dict.lookup === 'function') {
        const v = dict.lookup(name)
        if (v !== null && v !== undefined) return v
      }
    } catch {}
    
    // 2. 尝试 get（接受 PDFName 对象或 string）
    try {
      if (typeof dict.get === 'function') {
        const v = dict.get(name)
        if (v !== null && v !== undefined) return v
      }
    } catch {}
    
    // 3. 兜底：遍历 entries，用 resolvePdfName 匹配 key
    let cache = dictCache.get(dict)
    if (!cache) {
      cache = new Map()
      try {
        if (typeof dict.entries === 'function') {
          for (const [k, v] of dict.entries()) {
            const kname = resolvePdfName(k)
            if (kname) cache.set(kname, v)
          }
        }
      } catch {}
      dictCache.set(dict, cache)
    }
    return cache.get(name)
  }

  function resolvePdfName(v: any): string {
    if (v == null) return ''
    try {
      // 兼容 1: v.name 是函数（pdf-lib CJS/Node 模式）
      if (typeof v.name === 'function') return v.name()
      // 兼容 2: v.name 是属性/字符串（浏览器打包后）
      if (typeof v.name === 'string') return v.name
      // 兼容 3: v.asName() 函数（某些构建版本）
      if (typeof v.asName === 'function') return v.asName()
      // 兼容 4: 自定义 toString 返回 '/Name' 格式
      const str = String(v)
      if (str.startsWith('/')) return str.slice(1)
      // 兼容 5: 某些版本用 getName() / key()
      if (typeof v.getName === 'function') return v.getName()
      if (typeof v.key === 'function') return v.key()
    } catch {}
    return ''
  }
  function resolvePdfNumber(v: any): number {
    try {
      if (v == null) return 0
      // 函数形式 valueOf()
      if (typeof v.valueOf === 'function') {
        const r = v.valueOf()
        if (typeof r === 'number' && !isNaN(r)) return r
      }
      // 数字字面量属性（打包后的形式）
      if (typeof v === 'number') return v
      // 某些版本用 .number / .value 属性
      if (typeof v.number === 'number' && !isNaN(v.number)) return v.number
      if (typeof v.value === 'number' && !isNaN(v.value)) return v.value
      // 强制转换
      const r = Number(v)
      return isNaN(r) ? 0 : r
    } catch {}
    return 0
  }

  function getStreamContents(obj: any): Uint8Array {
    if (!obj) return new Uint8Array(0)
    try {
      // PDFRawStream：contents 为属性 (Uint8Array/PDFUint8Array)
      if (typeof obj.contents !== 'function' && obj.contents?.length != null) {
        return toStdU8(obj.contents as Uint8Array)
      }
    } catch {}
    try {
      if (typeof obj.contents === 'function') return toStdU8(obj.contents() as Uint8Array)
    } catch {}
    try {
      if (typeof obj.asUint8Array === 'function') return toStdU8(obj.asUint8Array() as Uint8Array)
    } catch {}
    return new Uint8Array(0)
  }

  // 包装 getFilterName / getColorSpaceName：dict 可能是 PDFStream 自己，也可能是 PDFRawStream 的 dict 属性（PDFDict）
  // 为了兼容，重写内部读取，不依赖外层的 doc.context.lookup
  function getFilterNameLocal(dict: any, filterObjIn: any): string {
    const f = filterObjIn ?? dictGet(dict, 'Filter')
    if (!f) return 'None'
    const nm = resolvePdfName(f)
    if (nm) return nm
    try {
      if (typeof f.size === 'function' && typeof f.lookup === 'function') {
        const inner = (ctx as any).lookup(f.lookup(0))
        const inm = resolvePdfName(inner)
        if (inm) return inm
      }
    } catch {}
    return String(f).slice(0, 20) || 'Unknown'
  }
  function getColorSpaceNameLocal(dict: any, csIn: any): string {
    const cs = csIn ?? dictGet(dict, 'ColorSpace')
    if (!cs) return 'Unknown'
    const nm = resolvePdfName(cs)
    if (nm) return nm
    try {
      if (typeof cs.lookup === 'function') {
        const first = (ctx as any).lookup(cs.lookup(0))
        const rest = (ctx as any).lookup(cs.lookup(1))
        return [resolvePdfName(first), resolvePdfName(rest)].filter(Boolean).join('/') || 'Unknown'
      }
    } catch {}
    return String(cs)
  }

  // ======== Step 1: 全局扫描所有 Stream 对象，收集 Image XObject ========
  // 结构：{ obj, dict, refKey, rawObjKey }，rawObjKey 用于去重 & 反向定位
  type Candidate = {
    obj: any
    dict: any
    refKey: string   // JSON.stringify([objectNumber, gen]) 或自定义
    width: number
    height: number
    bpc: number
    primaryFilter: string
    colorSpace: string
    contents: Uint8Array
    xObjectName: string
  }
  const candidates: Candidate[] = []
  const seenCandKeys = new Set<string>()
  const refMap = new Map<any, Candidate>()  // indirect object reference -> candidate

  try {
    const indirect: any = ctx?.indirectObjects
    if (indirect && typeof indirect.entries === 'function') {
      let totalObjScanned = 0
      let streamObjCount = 0
      let rawStreamCount = 0
      let otherStreamCount = 0
      let subtypeEmptyCount = 0
      let subtypeNotImageCount = 0
      let whMissingCount = 0
      for (const [refKeyArr, obj] of indirect.entries()) {
        totalObjScanned++
        if (!isStreamObj(obj)) continue
        streamObjCount++
        const cname = obj?.constructor?.name
        if (cname === 'PDFRawStream') rawStreamCount++
        else otherStreamCount++
        const dict = getStreamDict(obj)
        const subtype = resolvePdfName(dictGet(dict, 'Subtype'))
        if (!subtype) {
          subtypeEmptyCount++
          continue
        }
        if (subtype !== 'Image') { subtypeNotImageCount++; continue }
        const width = resolvePdfNumber(dictGet(dict, 'Width'))
        const height = resolvePdfNumber(dictGet(dict, 'Height'))
        if (!width || !height) { whMissingCount++; continue }
        const refKey = JSON.stringify(refKeyArr)
        if (seenCandKeys.has(refKey)) continue
        seenCandKeys.add(refKey)
        const bpc = resolvePdfNumber(dictGet(dict, 'BitsPerComponent')) || 8
        const primaryFilter = getFilterNameLocal(dict, undefined)
        const colorSpace = getColorSpaceNameLocal(dict, undefined)
        const contents = getStreamContents(obj)
        candidates.push({
          obj, dict, refKey,
          width, height, bpc, primaryFilter, colorSpace,
          contents,
          xObjectName: `Img${candidates.length + 1}`
        })
        refMap.set(obj, candidates[candidates.length - 1])
      }
    }
  } catch (e) {
    // 兜底：旧 PDF / 特殊 PDF 可能没有 indirectObjects.entries；退回到原 Page→Resources 自上而下模式
  }

  // ======== Step 2（Fallback）：仍保留自上而下的 Page→Resources→XObject 遍历（兼容老 PDF & Form 嵌套） ========
  const pageCount = doc.getPageCount()
  const fallbackXObjects: Array<{obj: any, pageIndex: number, xObjectName: string}> = []
  function walkResourcesForFallback(resources: any, pageIndex: number, visitedForms: Set<string>) {
    if (!resources) return
    let xobjDict: any = null
    try { xobjDict = resources.XObject?.() } catch {}
    try {
      if (!xobjDict) {
        const xref = dictGet(resources, 'XObject')
        if (xref) xobjDict = (ctx as any).lookup(xref)
      }
    } catch {}
    if (!xobjDict || typeof xobjDict.entries !== 'function') return
    for (const [nameObj, refObj] of xobjDict.entries()) {
      let target: any = null
      try { target = (ctx as any).lookup(refObj) } catch {}
      if (!target || !isStreamObj(target)) continue
      const dict = getStreamDict(target)
      const subtype = resolvePdfName(dictGet(dict, 'Subtype'))
      const xobjName = typeof nameObj?.decodeText === 'function' ? nameObj.decodeText() : (nameObj?.toString() || '')
      if (subtype === 'Image') {
        // 去重：检查是否已在 candidates 中（用对象引用对比）
        let already = false
        for (const c of candidates) { if (c.obj === target) { already = true; break } }
        if (!already) {
          const refKey = (refObj?.objectNumber ? JSON.stringify([refObj.objectNumber(), refObj.generationNumber?.() || 0]) : `fb_${fallbackXObjects.length}`)
          if (seenCandKeys.has(refKey)) continue
          seenCandKeys.add(refKey)
          const width = resolvePdfNumber(dictGet(dict, 'Width'))
          const height = resolvePdfNumber(dictGet(dict, 'Height'))
          if (width && height) {
            const bpc = resolvePdfNumber(dictGet(dict, 'BitsPerComponent')) || 8
            const primaryFilter = getFilterNameLocal(dict, undefined)
            const colorSpace = getColorSpaceNameLocal(dict, undefined)
            const contents = getStreamContents(target)
            candidates.push({
              obj: target, dict, refKey, width, height, bpc, primaryFilter, colorSpace,
              contents, xObjectName: xobjName || `Img${candidates.length + 1}`
            })
          }
        }
        fallbackXObjects.push({ obj: target, pageIndex, xObjectName: xobjName })
      } else if (subtype === 'Form') {
        // 递归进入 Form XObject 内部 Resources
        let innerRes: any = null
        try { innerRes = target?.Resources?.() } catch {}
        try {
          if (!innerRes) {
            const rref = dictGet(dict, 'Resources')
            if (rref) innerRes = (ctx as any).lookup(rref)
          }
        } catch {}
        // 通过 objectNumber 去重 Form
        let formKey = `form_${fallbackXObjects.length}`
        try {
          if (typeof refObj.objectNumber === 'function') {
            formKey = `form_${refObj.objectNumber()}_${refObj.generationNumber?.() || 0}`
          }
        } catch {}
        if (!visitedForms.has(formKey)) {
          visitedForms.add(formKey)
          walkResourcesForFallback(innerRes, pageIndex, visitedForms)
        }
      }
    }
  }
  for (let pageNo = 1; pageNo <= pageCount; pageNo++) {
    const page = doc.getPage(pageNo - 1)
    const node: any = (page as any).node
    let res: any = null
    try { res = node?.Resources?.() } catch {}
    try {
      if (!res) {
        const rref = dictGet(node, 'Resources')
        if (rref) res = (ctx as any).lookup(rref)
      }
    } catch {}
    // 向上遍历 Parent 继承链（PDF 规范允许 Resources 在 Pages 节点上）
    let cur = node
    const collected = [res]
    for (let lvl = 0; lvl < 5; lvl++) {
      let parent: any = null
      try { parent = cur?.Parent?.() } catch {}
      if (!parent) break
      let pres: any = null
      try { pres = parent?.Resources?.() } catch {}
      try {
        if (!pres) {
          const prref = dictGet(parent, 'Resources')
          if (prref) pres = (ctx as any).lookup(prref)
        }
      } catch {}
      if (pres) collected.push(pres)
      cur = parent
    }
    for (const r of collected) {
      walkResourcesForFallback(r, pageNo, new Set())
    }
  }

  // ======== Step 3: 反向追踪每张图的页码（全局扫描的候选没有页码，需要补上） ========
  // 通过 Page→Resources→XObject→Form→... 深度优先，记录引用到的每个 obj 的 pageIndex
  const objPageMap = new Map<any, {pageIndex: number, xObjectName: string}>()
  function traceResourcesForPaging(resources: any, pageIndex: number, visitedForms: Set<string>) {
    if (!resources) return
    let xobjDict: any = null
    try { xobjDict = resources.XObject?.() } catch {}
    try {
      if (!xobjDict) {
        const xref = dictGet(resources, 'XObject')
        if (xref) xobjDict = (ctx as any).lookup(xref)
      }
    } catch {}
    if (!xobjDict || typeof xobjDict.entries !== 'function') return
    for (const [nameObj, refObj] of xobjDict.entries()) {
      let target: any = null
      try { target = (ctx as any).lookup(refObj) } catch {}
      if (!target) continue
      const dict = getStreamDict(target)
      const subtype = resolvePdfName(dictGet(dict, 'Subtype'))
      if (subtype === 'Image') {
        if (!objPageMap.has(target)) {
          const xobjName = typeof nameObj?.decodeText === 'function' ? nameObj.decodeText() : (nameObj?.toString() || '')
          objPageMap.set(target, { pageIndex, xObjectName: xobjName })
        }
      } else if (subtype === 'Form') {
        let formKey = `pform_${Math.random()}`
        try {
          if (typeof refObj.objectNumber === 'function') {
            formKey = `pform_${refObj.objectNumber()}_${refObj.generationNumber?.() || 0}`
          }
        } catch {}
        if (visitedForms.has(formKey)) continue
        visitedForms.add(formKey)
        let innerRes: any = null
        try { innerRes = target?.Resources?.() } catch {}
        try {
          if (!innerRes) {
            const rref = dictGet(dict, 'Resources')
            if (rref) innerRes = (ctx as any).lookup(rref)
          }
        } catch {}
        traceResourcesForPaging(innerRes, pageIndex, visitedForms)
      }
    }
  }
  for (let pageNo = 1; pageNo <= pageCount; pageNo++) {
    const page = doc.getPage(pageNo - 1)
    const node: any = (page as any).node
    const allRes = []
    try { const r = node?.Resources?.(); if (r) allRes.push(r) } catch {}
    try {
      const rref = dictGet(node, 'Resources')
      if (rref) { const r = (ctx as any).lookup(rref); if (r) allRes.push(r) }
    } catch {}
    let cur = node
    for (let lvl = 0; lvl < 5; lvl++) {
      let parent: any = null
      try { parent = cur?.Parent?.() } catch {}
      if (!parent) break
      try { const r = parent?.Resources?.(); if (r) allRes.push(r) } catch {}
      try {
        const prref = dictGet(parent, 'Resources')
        if (prref) { const r = (ctx as any).lookup(prref); if (r) allRes.push(r) }
      } catch {}
      cur = parent
    }
    for (const r of allRes) traceResourcesForPaging(r, pageNo, new Set())
  }
  // fallbackXObjects 也补进去（它们已经有 pageIndex）
  for (const f of fallbackXObjects) {
    if (!objPageMap.has(f.obj)) {
      objPageMap.set(f.obj, { pageIndex: f.pageIndex, xObjectName: f.xObjectName })
    }
  }

  // ======== Step 4: 生成最终结果 ========
  const results: ExtractedImage[] = []
  // 去重（按 refKey 全局扫描已去重，但 fallback 可能重复）
  const doneObj = new WeakSet()

  for (const cand of candidates) {
    if (doneObj.has(cand.obj)) continue
    doneObj.add(cand.obj)

    let { width, height, bpc, primaryFilter, colorSpace, contents } = cand
    let pageIndex = 1
    let xObjectName = cand.xObjectName
    const pageInfo = objPageMap.get(cand.obj)
    if (pageInfo) {
      pageIndex = pageInfo.pageIndex
      if (pageInfo.xObjectName) xObjectName = pageInfo.xObjectName
    }

    let format: ExtractedImage['format'] = 'raw'
    let mimeType = 'application/octet-stream'
    let blob = new Blob([contents as BlobPart], { type: mimeType })
    let previewDataUrl = ''

    try {
      if (primaryFilter === 'DCTDecode' || primaryFilter === 'DCT') {
        format = 'jpeg'
        mimeType = 'image/jpeg'
        blob = new Blob([contents as BlobPart], { type: mimeType })
        previewDataUrl = `data:${mimeType};base64,${arrayBufferToBase64Local(contents)}`
      } else if (primaryFilter === 'JPXDecode') {
        format = 'jp2'
        mimeType = 'image/jp2'
        blob = new Blob([contents as BlobPart], { type: mimeType })
        previewDataUrl = placeholderSvg('JPEG 2000 (JPX)', width, height)
      } else if (primaryFilter === 'FlateDecode' || primaryFilter === 'Flate' || primaryFilter === 'None') {
        let pixels: Uint8Array
        if (primaryFilter === 'None') pixels = contents
        else pixels = await inflateDeflate(contents)
        const { blob: pngBlob, dataUrl } = await rawPixelsToPngBlob(pixels, width, height, bpc, colorSpace)
        format = 'png'
        mimeType = 'image/png'
        blob = pngBlob
        previewDataUrl = dataUrl
      } else if (primaryFilter.includes('CCITT')) {
        format = 'tiff'
        mimeType = 'image/tiff'
        blob = new Blob([contents as BlobPart], { type: mimeType })
        previewDataUrl = placeholderSvg('CCITT G3/G4 Fax (TIFF)', width, height, '#2b1f3a')
      } else {
        format = 'raw'
        mimeType = 'application/octet-stream'
        blob = new Blob([contents as BlobPart], { type: mimeType })
        previewDataUrl = placeholderSvg(
          `过滤器：${primaryFilter || 'Unknown'}`,
          width, height, '#1f2e2e'
        )
      }
    } catch (convErr: any) {
      format = 'raw'
      mimeType = 'application/octet-stream'
      blob = new Blob([contents as BlobPart], { type: mimeType })
      previewDataUrl = placeholderSvg(
        `${primaryFilter} · ${(convErr?.message || 'decode fail').slice(0, 28)}`,
        width, height, '#3a1f1f'
      )
    }

    results.push({
      pageIndex,
      xObjectName,
      width, height,
      bitsPerComponent: bpc,
      colorSpace,
      primaryFilter,
      format, mimeType,
      previewDataUrl,
      blob,
      size: blob.size
    })
  }

  // 按页码升序排（未定位到页码的排末尾）
  results.sort((a, b) => a.pageIndex - b.pageIndex)
  return results
}

// 工具函数（避免依赖外部 base64 库，和 pdfUtils 顶部已有的 arrayBufferToBase64 逻辑一致）
function arrayBufferToBase64Local(buffer: Uint8Array): string {
  const bytes = buffer
  const chunkSize = 0x8000
  const chunks: string[] = []
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize)
    chunks.push(String.fromCharCode(...chunk))
  }
  return btoa(chunks.join(''))
}
