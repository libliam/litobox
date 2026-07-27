import { Base64 } from 'js-base64'

export function urlEncode(text: string): string {
  return encodeURIComponent(text)
}

export function urlDecode(text: string): string {
  try {
    return decodeURIComponent(text)
  } catch {
    return '解码失败：无效的URL编码'
  }
}

export function urlDoubleDecode(text: string): string {
  try {
    const first = decodeURIComponent(text)
    try {
      return decodeURIComponent(first)
    } catch {
      return first + '\n\n（注：输入仅编码了一层，第二次解码无变化）'
    }
  } catch {
    return '解码失败：无效的URL编码'
  }
}

export function base64Encode(text: string): string {
  const bytes = new TextEncoder().encode(text)
  return Base64.fromUint8Array(bytes)
}

export function base64Decode(text: string): string {
  try {
    const bytes = Base64.toUint8Array(text)
    return new TextDecoder().decode(bytes)
  } catch {
    return '解码失败：无效的Base64编码'
  }
}

export function timestampToDatetime(timestamp: number, isMilliseconds: boolean = true): string {
  const ms = isMilliseconds ? timestamp : timestamp * 1000
  const date = new Date(ms)
  if (isNaN(date.getTime())) {
    return '无效的时间戳'
  }
  return date.toLocaleString('zh-CN', { 
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })
}

export function datetimeToTimestamp(datetime: string, isMilliseconds: boolean = true): number | string {
  const date = new Date(datetime)
  if (isNaN(date.getTime())) {
    return '无效的日期时间格式'
  }
  const timestamp = date.getTime()
  return isMilliseconds ? timestamp : Math.floor(timestamp / 1000)
}