/**
 * 文本编码智能解码：解决中文 Windows 下 GBK 编码文本文件（如 Excel 导出的 CSV）
 * 被按 UTF-8 解码导致乱码的问题。
 *
 * 解码策略：
 * 1. 检测 UTF-8 / UTF-16LE / UTF-16BE 的 BOM
 * 2. 无 BOM 时用严格模式校验 UTF-8（fatal），合法则按 UTF-8 解码
 * 3. 校验失败回退 GBK 解码
 */
export function decodeTextSmart(bytes: Uint8Array): string {
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    return new TextDecoder('utf-8').decode(bytes.subarray(3))
  }
  if (bytes.length >= 2 && bytes[0] === 0xff && bytes[1] === 0xfe) {
    return new TextDecoder('utf-16le').decode(bytes.subarray(2))
  }
  if (bytes.length >= 2 && bytes[0] === 0xfe && bytes[1] === 0xff) {
    return new TextDecoder('utf-16be').decode(bytes.subarray(2))
  }
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes)
  } catch {
    return new TextDecoder('gbk').decode(bytes)
  }
}
