import CryptoJS from 'crypto-js'

export type HashAlgorithm = 'md5' | 'sha1' | 'sha224' | 'sha256' | 'sha384' | 'sha512' | 'sha3' | 'ripemd160'

export interface HashResult {
  algorithm: string
  hash: string
}

/**
 * 计算文本的哈希值
 */
export function hashText(text: string, algorithm: HashAlgorithm): string {
  switch (algorithm) {
    case 'md5':
      return CryptoJS.MD5(text).toString()
    case 'sha1':
      return CryptoJS.SHA1(text).toString()
    case 'sha224':
      return CryptoJS.SHA224(text).toString()
    case 'sha256':
      return CryptoJS.SHA256(text).toString()
    case 'sha384':
      return CryptoJS.SHA384(text).toString()
    case 'sha512':
      return CryptoJS.SHA512(text).toString()
    case 'sha3':
      return CryptoJS.SHA3(text, { outputLength: 256 }).toString()
    case 'ripemd160':
      return CryptoJS.RIPEMD160(text).toString()
    default:
      throw new Error(`不支持的算法: ${algorithm}`)
  }
}

/**
 * 计算 HMAC 值
 */
export function hmacText(text: string, key: string, algorithm: HashAlgorithm): string {
  switch (algorithm) {
    case 'md5':
      return CryptoJS.HmacMD5(text, key).toString()
    case 'sha1':
      return CryptoJS.HmacSHA1(text, key).toString()
    case 'sha224':
      return CryptoJS.HmacSHA224(text, key).toString()
    case 'sha256':
      return CryptoJS.HmacSHA256(text, key).toString()
    case 'sha384':
      return CryptoJS.HmacSHA384(text, key).toString()
    case 'sha512':
      return CryptoJS.HmacSHA512(text, key).toString()
    case 'sha3':
      return CryptoJS.HmacSHA3(text, key, { outputLength: 256 }).toString()
    case 'ripemd160':
      return CryptoJS.HmacRIPEMD160(text, key).toString()
    default:
      throw new Error(`不支持的算法: ${algorithm}`)
  }
}

/**
 * 计算文件的哈希值（使用 Web Crypto API，支持大文件）
 * 仅支持 SHA-1/SHA-256/SHA-384/SHA-512（Web Crypto API 限制）
 */
export async function hashFile(file: File, algorithm: 'sha1' | 'sha256' | 'sha384' | 'sha512'): Promise<string> {
  const buffer = await file.arrayBuffer()
  const cryptoAlgorithm = `SHA-${algorithm.slice(3)}`

  const hashBuffer = await crypto.subtle.digest(cryptoAlgorithm, buffer)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
}
