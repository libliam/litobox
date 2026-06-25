export interface ImageInfo {
  width: number
  height: number
  size: number
  type: string
  name: string
}

export function getImageInfo(file: File): Promise<ImageInfo> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      resolve({
        width: img.naturalWidth,
        height: img.naturalHeight,
        size: file.size,
        type: file.type,
        name: file.name
      })
      URL.revokeObjectURL(img.src)
    }
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = URL.createObjectURL(file)
  })
}

export function compressImage(file: File, quality: number): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      canvas.width = img.naturalWidth
      canvas.height = img.naturalHeight
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        reject(new Error('Canvas context not available'))
        return
      }
      ctx.drawImage(img, 0, 0)
      canvas.toBlob(
        (blob) => {
          if (blob) resolve(blob)
          else reject(new Error('压缩失败'))
        },
        'image/jpeg',
        quality / 100
      )
      URL.revokeObjectURL(img.src)
    }
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = URL.createObjectURL(file)
  })
}

export function resizeImage(file: File, width: number, height: number): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      canvas.width = width
      canvas.height = height
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        reject(new Error('Canvas context not available'))
        return
      }
      ctx.drawImage(img, 0, 0, width, height)
      canvas.toBlob(
        (blob) => {
          if (blob) resolve(blob)
          else reject(new Error('缩放失败'))
        },
        file.type === 'image/png' ? 'image/png' : 'image/jpeg',
        file.type === 'image/png' ? undefined : 0.92
      )
      URL.revokeObjectURL(img.src)
    }
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = URL.createObjectURL(file)
  })
}

export function imageToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(new Error('读取文件失败'))
    reader.readAsDataURL(file)
  })
}

export function calculateAspectRatio(
  origWidth: number,
  origHeight: number,
  targetWidth?: number,
  targetHeight?: number
): { width: number; height: number } {
  if (targetWidth && targetHeight) {
    return { width: targetWidth, height: targetHeight }
  }
  if (targetWidth) {
    return { width: targetWidth, height: Math.round(targetWidth * (origHeight / origWidth)) }
  }
  if (targetHeight) {
    return { width: Math.round(targetHeight * (origWidth / origHeight)), height: targetHeight }
  }
  return { width: origWidth, height: origHeight }
}

export { saveFileWithDialog } from './fileSaver'

export function copyBase64(base64: string): Promise<void> {
  return navigator.clipboard.writeText(base64)
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}
