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

  let text = ''
  for (let i = 1; i <= totalPages; i++) {
    const page = await pdfDocument.getPage(i)
    const content = await page.getTextContent()
    const pageText = content.items
      .map((item: any) => item.str)
      .join('')
    text += pageText + '\n'
  }

  return text.trim()
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
