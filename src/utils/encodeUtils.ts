import { Base64 } from 'js-base64'
import CryptoJS from 'crypto-js'

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

// HTML实体编码
const htmlEscapeMap: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#39;'
}

export function htmlEncode(text: string): string {
  return text.replace(/[&<>"']/g, char => htmlEscapeMap[char])
}

export function htmlDecode(text: string): string {
  try {
    const doc = new DOMParser().parseFromString(text, 'text/html')
    return doc.documentElement.textContent || ''
  } catch {
    return text
  }
}

// Unicode编解码
export function unicodeEncode(text: string): string {
  return text.replace(/[\s\S]/g, char => {
    const code = char.charCodeAt(0)
    if (code > 127) {
      return '\\u' + code.toString(16).padStart(4, '0')
    }
    return char
  })
}

export function unicodeDecode(text: string): string {
  return text.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => {
    return String.fromCharCode(parseInt(hex, 16))
  })
}

// AES加密/解密
export function aesEncode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  return CryptoJS.AES.encrypt(text, key).toString()
}

export function aesDecode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  try {
    const bytes = CryptoJS.AES.decrypt(text, key)
    const result = bytes.toString(CryptoJS.enc.Utf8)
    if (!result) return '解密失败：密钥错误或密文无效'
    return result
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}

// DES加密/解密
export function desEncode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  return CryptoJS.DES.encrypt(text, key).toString()
}

export function desDecode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  try {
    const bytes = CryptoJS.DES.decrypt(text, key)
    const result = bytes.toString(CryptoJS.enc.Utf8)
    if (!result) return '解密失败：密钥错误或密文无效'
    return result
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}

// 3DES加密/解密
export function tripleDesEncode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  return CryptoJS.TripleDES.encrypt(text, key).toString()
}

export function tripleDesDecode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  try {
    const bytes = CryptoJS.TripleDES.decrypt(text, key)
    const result = bytes.toString(CryptoJS.enc.Utf8)
    if (!result) return '解密失败：密钥错误或密文无效'
    return result
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}

// RC4加密/解密
export function rc4Encode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  return CryptoJS.RC4.encrypt(text, key).toString()
}

export function rc4Decode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  try {
    const bytes = CryptoJS.RC4.decrypt(text, key)
    const result = bytes.toString(CryptoJS.enc.Utf8)
    if (!result) return '解密失败：密钥错误或密文无效'
    return result
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}

// Rabbit加密/解密
export function rabbitEncode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  return CryptoJS.Rabbit.encrypt(text, key).toString()
}

export function rabbitDecode(text: string, key: string): string {
  if (!key) return '密钥不能为空'
  try {
    const bytes = CryptoJS.Rabbit.decrypt(text, key)
    const result = bytes.toString(CryptoJS.enc.Utf8)
    if (!result) return '解密失败：密钥错误或密文无效'
    return result
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}

// 时间戳转换
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
