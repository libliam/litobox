/**
 * 从剪贴板读取图片的公共 composable
 * 提供按钮点击读取 + Ctrl+V 粘贴监听两种方式
 */
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

export interface ClipboardImage {
  blob: Blob
  dataUrl: string
  base64: string      // 纯 base64，不带前缀
  width: number
  height: number
}

/**
 * 从 Clipboard API 读取一张图片
 * 返回 null 表示剪贴板没有图片（不报错）
 */
async function readClipboardImage(): Promise<ClipboardImage | null> {
  if (!navigator.clipboard?.read) return null

  const items = await navigator.clipboard.read()
  for (const item of items) {
    const imageType = item.types.find(t => t.startsWith('image/'))
    if (imageType) {
      const blob = await item.getType(imageType)
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onload = () => resolve(reader.result as string)
        reader.onerror = () => reject(reader.error)
        reader.readAsDataURL(blob)
      })
      const base64 = dataUrl.split(',')[1] || ''

      // 获取尺寸
      const { width, height } = await new Promise<{ width: number; height: number }>(resolve => {
        const img = new Image()
        img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight })
        img.onerror = () => resolve({ width: 0, height: 0 })
        img.src = dataUrl
      })

      return { blob, dataUrl, base64, width, height }
    }
  }
  return null
}

/**
 * 将剪贴板图片保存为临时文件，返回文件路径
 * 用于需要 filePath 的后端命令
 */
export async function saveClipboardImageAsTemp(img: ClipboardImage, ext = 'png'): Promise<string> {
  const filename = `clip_${Date.now()}.${ext}`
  return await invoke<string>('save_temp_file', { data: img.base64, filename })
}

export function useClipboardImage() {
  /**
   * 从剪贴板读取图片，读取失败或无图片时返回 null
   */
  async function readImage(): Promise<ClipboardImage | null> {
    try {
      return await readClipboardImage()
    } catch {
      return null
    }
  }

  /**
   * 从剪贴板读取图片，带用户提示
   */
  async function readImageWithToast(): Promise<ClipboardImage | null> {
    const img = await readImage()
    if (!img) {
      ElMessage.warning('剪贴板中没有图片')
    }
    return img
  }

  /**
   * 监听 Ctrl+V 粘贴图片
   * @param callback 收到图片时的回调
   * @returns 取消监听函数
   */
  function onPasteImage(callback: (img: ClipboardImage) => void): () => void {
    const handler = async (e: ClipboardEvent) => {
      // 优先检查剪贴板事件中的图片
      const items = e.clipboardData?.items
      if (items) {
        for (const item of items) {
          if (item.type.startsWith('image/')) {
            const blob = item.getAsFile()
            if (!blob) continue
            const dataUrl = await new Promise<string>((resolve, reject) => {
              const reader = new FileReader()
              reader.onload = () => resolve(reader.result as string)
              reader.onerror = () => reject(reader.error)
              reader.readAsDataURL(blob)
            })
            const base64 = dataUrl.split(',')[1] || ''
            const { width, height } = await new Promise<{ width: number; height: number }>(resolve => {
              const imgEl = new Image()
              imgEl.onload = () => resolve({ width: imgEl.naturalWidth, height: imgEl.naturalHeight })
              imgEl.onerror = () => resolve({ width: 0, height: 0 })
              imgEl.src = dataUrl
            })
            e.preventDefault()
            callback({ blob, dataUrl, base64, width, height })
            return
          }
        }
      }
      // ClipboardEvent 没有图片，回退到 Clipboard API
      const clipImg = await readImage()
      if (clipImg) {
        e.preventDefault()
        callback(clipImg)
      }
    }
    document.addEventListener('paste', handler)
    return () => document.removeEventListener('paste', handler)
  }

  return { readImage, readImageWithToast, onPasteImage, saveClipboardImageAsTemp }
}
