// 图片对比核心工具：加载图片、统一尺寸、像素级差异对比与差异率统计

/** 从 base64 数据 URL 或 URL 加载图片 */
export function loadImageFromUrl(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败，请确认文件格式支持'))
    img.src = url
  })
}

/** 最大边尺寸（过大图片降采样，控制像素遍历开销） */
export const MAX_DIM = 1000

export interface SizeInfo {
  width: number
  height: number
}

/**
 * 将图片按最大边 MAX_DIM 等比缩放并绘制到离屏 canvas，返回 { canvas, imgData }
 */
export function drawScaledImage(img: HTMLImageElement): { canvas: HTMLCanvasElement; imgData: ImageData } {
  const scale = Math.min(1, MAX_DIM / Math.max(img.naturalWidth, img.naturalHeight))
  const width = Math.max(1, Math.round(img.naturalWidth * scale))
  const height = Math.max(1, Math.round(img.naturalHeight * scale))
  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  const ctx = canvas.getContext('2d')!
  ctx.drawImage(img, 0, 0, width, height)
  return { canvas, imgData: ctx.getImageData(0, 0, width, height) }
}

/** 合并两个尺寸（取每维较大值），供画布对齐 */
export function unionSize(w1: number, h1: number, w2: number, h2: number): SizeInfo {
  return { width: Math.max(w1, w2), height: Math.max(h1, h2) }
}

export interface CompareResult {
  diffRatio: number
  diffCount: number
  total: number
}

/**
 * 像素级对比两个已对齐的 ImageData
 * @param dataA 基准图
 * @param dataB 对比图
 * @param tolerance 每通道差异阈值（欧氏距离），0-255，默认 32
 * @returns diffRatio 差异像素占比；diffData 差异画布（透明底 + 红色差异点），与 dataA 同尺寸
 */
export function compareImages(
  dataA: ImageData,
  dataB: ImageData,
  tolerance = 32,
): { result: CompareResult; diffData: ImageData } {
  const w = Math.min(dataA.width, dataB.width)
  const h = Math.min(dataA.height, dataB.height)
  const total = w * h
  const a = dataA.data
  const b = dataB.data
  const diffData = new ImageData(w, h)
  const d = diffData.data

  let diffCount = 0
  // 同一行内逐像素遍历，利用行缓冲减小越界判断开销
  for (let y = 0; y < h; y++) {
    const rowStart = y * w * 4
    const rowEnd = rowStart + w * 4
    for (let i = rowStart; i < rowEnd; i += 4) {
      const dr = a[i] - b[i]
      const dg = a[i + 1] - b[i + 1]
      const db = a[i + 2] - b[i + 2]
      if (dr * dr + dg * dg + db * db > tolerance * tolerance * 3) {
        diffCount++
        // 差异像素标红，保留亮度信息便于定位
        const lum = Math.round((a[i] + a[i + 1] + a[i + 2]) / 3)
        d[i] = 255
        d[i + 1] = Math.max(0, lum - 160)
        d[i + 2] = Math.max(0, lum - 160)
        d[i + 3] = 235
      }
    }
  }

  return {
    result: { diffRatio: total ? diffCount / total : 0, diffCount, total },
    diffData,
  }
}

/** 自检：验证像素对比逻辑 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    const mk = (r: number, g: number, b: number) => {
      const d = new ImageData(2, 1)
      const p = d.data
      for (let i = 0; i < 8; i += 4) {
        p[i] = r; p[i + 1] = g; p[i + 2] = b; p[i + 3] = 255
      }
      return d
    }
    const red = mk(255, 0, 0)
    const mixed = mk(255, 0, 0) // 复制后再改一个像素
    mixed.data[4] = 0; mixed.data[5] = 0; mixed.data[6] = 255 // 第二像素变蓝
    const { result, diffData } = compareImages(red, mixed, 0)
    if (Math.abs(result.diffRatio - 0.5) > 1e-9) errors.push(`差异率应为 0.5，实际 ${result.diffRatio}`)
    if (diffData.width !== 2 || diffData.height !== 1) errors.push('差异画布尺寸不对')
    // 相同图片差异率为 0
    const same = compareImages(red, red, 0)
    if (same.result.diffRatio !== 0) errors.push('相同图片差异率应为 0')
    // 容差应生效：小差异 + 大容差 → 0
    const small = compareImages(red, mixed, 300)
    if (small.result.diffRatio !== 0) errors.push('大容差下差异率应为 0')
  } catch (e: any) {
    errors.push('selfCheck 异常: ' + (e.message || e))
  }
  return errors
}
