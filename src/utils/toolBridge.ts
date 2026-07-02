/**
 * 工具间数据桥接 — 传递非序列化数据（如 Blob）
 * 用于 PdfTool → OcrTool 的图片传递
 */

// 传递中的图片 Blob 列表
const _pendingImageBlobs: Blob[] = []

export function setPendingImages(blobs: Blob[]): void {
  _pendingImageBlobs.length = 0
  _pendingImageBlobs.push(...blobs)
}

export function getPendingImages(): Blob[] {
  const result = [..._pendingImageBlobs]
  _pendingImageBlobs.length = 0
  return result
}

export function clearPendingImages(): void {
  _pendingImageBlobs.length = 0
}
