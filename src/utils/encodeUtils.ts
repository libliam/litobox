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

export function base64Encode(text: string): string {
  return Base64.encode(text)
}

export function base64Decode(text: string): string {
  try {
    return Base64.decode(text)
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