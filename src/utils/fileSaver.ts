import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

/**
 * 将 Blob 转换为 base64 字符串
 */
function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onloadend = () => {
      const result = reader.result as string
      // 移除 data:xxx;base64, 前缀
      const base64 = result.split(',')[1] || result
      resolve(base64)
    }
    reader.onerror = () => reject(new Error('Blob 转 base64 失败'))
    reader.readAsDataURL(blob)
  })
}

/**
 * 使用 Tauri 原生保存对话框保存文件
 * @param blob 要保存的文件内容
 * @param filename 默认文件名
 * @param defaultExt 默认扩展名（不含点）
 * @returns 成功返回文件路径，取消返回 null
 */
export async function saveFileWithDialog(
  blob: Blob,
  filename: string,
  defaultExt: string
): Promise<string | null> {
  try {
    // 尝试使用 Tauri 原生对话框
    const base64 = await blobToBase64(blob)
    const result = await invoke<string>('save_file_with_dialog', {
      dataBase64: base64,
      filename,
      defaultExt,
    })

    if (result === 'cancelled') {
      ElMessage.info('已取消保存')
      return null
    }

    ElMessage.success(`文件已保存至: ${result}`)
    return result
  } catch (error) {
    // Tauri 不可用或调用失败，降级为浏览器下载
    console.warn('Tauri 保存对话框不可用，使用浏览器下载:', error)
    fallbackDownload(blob, filename)
    ElMessage.success('文件已开始下载')
    return null
  }
}

/**
 * 降级方案：使用浏览器默认下载行为
 */
function fallbackDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
