import { PaddleOCR } from '@paddleocr/paddleocr-js'

export interface OcrResult {
  text: string
  confidence: number
  box: number[][]
}

export interface OcrEngine {
  predict: (image: Blob | File) => Promise<OcrResult[]>
  destroy: () => void
}

let ocrInstance: any = null
let initPromise: Promise<OcrEngine> | null = null

/**
 * 初始化OCR引擎（懒加载，单例）
 */
export async function initOcr(): Promise<OcrEngine> {
  if (initPromise) return initPromise

  // ponytail: 使用本地模型文件，避免打包后网络请求失败
  const modelBasePath = import.meta.env.BASE_URL + 'ocr-models'

  initPromise = (async () => {
    ocrInstance = await PaddleOCR.create({
      textDetectionModelName: 'PP-OCRv6_tiny_det',
      textRecognitionModelName: 'PP-OCRv6_tiny_rec',
      textDetectionModelAsset: {
        url: `${modelBasePath}/PP-OCRv6_tiny_det.tar`
      },
      textRecognitionModelAsset: {
        url: `${modelBasePath}/PP-OCRv6_tiny_rec.tar`
      },
      // 使用 CPU 后端，避免 WASM 文件依赖问题
      ortOptions: {
        backend: 'cpu'
      }
    })

    return {
      predict: async (image: Blob | File) => {
        if (!ocrInstance) throw new Error('OCR引擎未初始化')
        const results = await ocrInstance.predict(image)
        return results[0].items.map((item: any) => ({
          text: item.text,
          confidence: item.score,
          box: item.poly
        }))
      },
      destroy: () => {
        ocrInstance = null
        initPromise = null
      }
    }
  })()

  return initPromise
}

/**
 * 执行OCR识别
 */
export async function recognizeImage(image: Blob | File): Promise<string> {
  const engine = await initOcr()
  const results = await engine.predict(image)

  if (results.length === 0) {
    throw new Error('未识别到文字，请检查图片是否清晰')
  }

  const avgHeight = results.reduce((sum, r) => sum + getBlockHeight(r), 0) / results.length
  const rowTolerance = avgHeight * 0.5
  const rows = groupByY(results, rowTolerance)

  const lines: string[] = []
  for (const row of rows) {
    const sortedByX = [...row].sort((a, b) => {
      const xA = Math.min(...a.box.map(p => p[0]))
      const xB = Math.min(...b.box.map(p => p[0]))
      return xA - xB
    })
    lines.push(sortedByX.map(r => r.text).join(''))
  }

  return lines.join('\n')
}

/**
 * 清理OCR引擎（释放内存）
 */
export function destroyOcr(): void {
  if (ocrInstance) {
    ocrInstance = null
    initPromise = null
  }
}

/**
 * 检查引擎是否已初始化
 */
export function isOcrReady(): boolean {
  return ocrInstance !== null
}

/**
 * 清理文本：移除多余空行和首尾空格
 */
export function cleanText(text: string): string {
  return text
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0)
    .join('\n')
}

import { saveFileWithDialog } from './fileSaver'

/**
 * 导出文本为txt文件
 */
export async function exportAsTxt(text: string, filename: string = 'ocr-result.txt'): Promise<void> {
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' })
  await saveFileWithDialog(blob, filename, 'txt')
}

export interface BatchImage {
  id: string
  file: File | Blob
  thumbnail: string
  originalUrl: string
  name: string
  status: 'pending' | 'recognizing' | 'success' | 'error'
  result?: string
  error?: string
}

/**
 * 批量OCR识别（并行处理）
 */
export async function batchRecognize(
  images: BatchImage[],
  onProgress?: (completed: number, total: number) => void
): Promise<void> {
  // 串行处理，避免 PaddleOCR 引擎并发冲突
  for (let i = 0; i < images.length; i++) {
    const image = images[i]
    image.status = 'recognizing'
    try {
      const text = await recognizeImage(image.file)
      image.result = text
      image.status = 'success'
    } catch (e: any) {
      image.error = e.message || '识别失败'
      image.status = 'error'
    }
    onProgress?.(i + 1, images.length)
  }
}

/**
 * 获取合并的OCR结果
 */
export function getMergedResult(images: BatchImage[]): string {
  return images
    .filter(i => i.status === 'success' && i.result)
    .map(i => `--- ${i.name} ---\n${i.result}`)
    .join('\n\n')
}

/**
 * 按Y坐标分组文字块（行检测）
 */
function groupByY(results: OcrResult[], tolerance: number): OcrResult[][] {
  const sorted = [...results].sort((a, b) => {
    const yA = Math.min(...a.box.map(p => p[1]))
    const yB = Math.min(...b.box.map(p => p[1]))
    return yA - yB
  })

  const rows: OcrResult[][] = []
  for (const item of sorted) {
    const y = Math.min(...item.box.map(p => p[1]))
    const existingRow = rows.find(row => {
      const rowY = Math.min(...row[0].box.map(p => p[1]))
      return Math.abs(y - rowY) <= tolerance
    })
    if (existingRow) {
      existingRow.push(item)
    } else {
      rows.push([item])
    }
  }
  return rows
}

/**
 * 检测列边界（X坐标聚类）
 */
function detectColumns(rows: OcrResult[][], tolerance: number): number[] {
  const xCoords: number[] = []
  for (const row of rows) {
    for (const item of row) {
      const x = Math.min(...item.box.map(p => p[0]))
      xCoords.push(x)
    }
  }

  const columns: number[] = []
  const sorted = [...xCoords].sort((a, b) => a - b)
  for (const x of sorted) {
    const existingCol = columns.find(col => Math.abs(x - col) <= tolerance)
    if (!existingCol) {
      columns.push(x)
    }
  }
  return columns
}

/**
 * 从OCR结果推断表格结构
 * @param ocrResults OCR识别结果数组
 * @param rowTolerance 行检测容差（px），默认5
 * @param colTolerance 列检测容差（px），默认10
 * @returns 二维字符串数组，表示表格内容
 */
export function detectTable(
  ocrResults: OcrResult[],
  rowTolerance: number = 5,
  colTolerance: number = 10
): string[][] {
  if (ocrResults.length === 0) return []

  // 1. 按Y坐标分组（行检测）
  const rows = groupByY(ocrResults, rowTolerance)

  // 2. 检测列边界
  const columns = detectColumns(rows, colTolerance)

  // 3. 构建表格
  return rows.map(row => {
    const cells: string[] = new Array(columns.length).fill('')
    for (const item of row) {
      const x = Math.min(...item.box.map(p => p[0]))
      const colIndex = columns.findIndex(col => Math.abs(x - col) <= colTolerance)
      if (colIndex >= 0) {
        cells[colIndex] = item.text
      }
    }
    return cells
  })
}

/**
 * 将二维数组转为CSV字符串
 * @param table 二维字符串数组
 * @returns CSV格式字符串
 */
export function toCsv(table: string[][]): string {
  return table
    .map(row =>
      row
        .map(cell => {
          // 处理包含逗号、引号、换行符的单元格
          if (cell.includes(',') || cell.includes('"') || cell.includes('\n')) {
            return `"${cell.replace(/"/g, '""')}"`
          }
          return cell
        })
        .join(',')
    )
    .join('\n')
}

/**
 * 识别图片中的表格
 * @param image 图片文件/Blob
 * @returns 二维字符串数组（表格内容）
 */
export async function recognizeTable(image: Blob | File): Promise<string[][]> {
  const engine = await initOcr()
  const results = await engine.predict(image)

  if (results.length === 0) {
    throw new Error('未识别到文字，请检查图片是否清晰')
  }

  return detectTable(results)
}

/**
 * 计算文字块的高度（基于box的Y坐标差）
 */
function getBlockHeight(result: OcrResult): number {
  const yCoords = result.box.map(p => p[1])
  return Math.max(...yCoords) - Math.min(...yCoords)
}

/**
 * 根据高度推断Markdown标题前缀
 * ponytail: 使用简单的平均高度倍数阈值，如需更精确可引入聚类算法
 */
function getHeadingPrefix(height: number, avgHeight: number): string {
  if (height > avgHeight * 1.5) return '# '
  if (height > avgHeight * 1.2) return '## '
  if (height > avgHeight * 1.0) return '### '
  return ''
}

/**
 * 将OCR结果转换为Markdown格式
 * @param ocrResults OCR识别结果数组
 * @returns Markdown格式文本
 */
export function convertToMarkdown(ocrResults: OcrResult[]): string {
  if (ocrResults.length === 0) return ''

  const avgHeight = ocrResults.reduce((sum, r) => sum + getBlockHeight(r), 0) / ocrResults.length

  const rowTolerance = avgHeight * 0.5
  const rows = groupByY(ocrResults, rowTolerance)

  const avgLineHeight = avgHeight * 1.5

  const lines: string[] = []
  let prevY: number | null = null

  for (const row of rows) {
    const rowY = Math.min(...row.map(r => Math.min(...r.box.map(p => p[1]))))
    const rowHeight = row.reduce((sum, r) => sum + getBlockHeight(r), 0) / row.length

    const sortedByX = [...row].sort((a, b) => {
      const xA = Math.min(...a.box.map(p => p[0]))
      const xB = Math.min(...b.box.map(p => p[0]))
      return xA - xB
    })

    const lineText = sortedByX.map(r => r.text).join('')

    if (prevY !== null && (rowY - prevY) > avgLineHeight * 2) {
      lines.push('')
    }

    const prefix = getHeadingPrefix(rowHeight, avgHeight)
    lines.push(`${prefix}${lineText}`)

    prevY = rowY
  }

  return lines.join('\n')
}

/**
 * 识别图片并返回Markdown格式文本
 * @param image 图片文件/Blob
 * @returns Markdown格式文本
 */
export async function recognizeMarkdown(image: Blob | File): Promise<string> {
  const engine = await initOcr()
  const results = await engine.predict(image)

  if (results.length === 0) {
    throw new Error('未识别到文字，请检查图片是否清晰')
  }

  return convertToMarkdown(results)
}

/**
 * 导出文本为md文件
 */
export async function exportAsMd(text: string, filename: string = 'markdown-result.md'): Promise<void> {
  const blob = new Blob([text], { type: 'text/markdown;charset=utf-8' })
  await saveFileWithDialog(blob, filename, 'md')
}
