// TOTP (RFC 6238) / HOTP (RFC 4226) 生成：纯前端实现，基于 crypto-js 的 HmacSHA*
// ponytail: 只做验证码计算与 otpauth:// URI 解析，密钥存储由后端 SQLite 负责（明文）

import CryptoJS from 'crypto-js'

export type TotpAlgorithm = 'SHA1' | 'SHA256' | 'SHA512'

export interface TotpOptions {
  algorithm: TotpAlgorithm
  digits: number
  period: number
  /** 毫秒时间戳，默认 Date.now() */
  timestamp?: number
}

export interface TotpResult {
  code: string
  /** 当前周期剩余秒数 */
  remaining: number
  /** 当前周期已过比例 0~1 */
  progress: number
}

export interface OtpauthParams {
  type: 'totp' | 'hotp'
  secret: string
  issuer: string
  account: string
  algorithm: TotpAlgorithm
  digits: number
  period: number
  counter?: number
}

const BASE32_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'
const ALGO_MAP = {
  SHA1: CryptoJS.HmacSHA1,
  SHA256: CryptoJS.HmacSHA256,
  SHA512: CryptoJS.HmacSHA512,
}

/** Base32 解码（RFC4648，忽略空格/连字符/填充） */
export function base32Decode(secret: string): number[] {
  const clean = secret.replace(/[\s-]/g, '').replace(/=+$/, '').toUpperCase()
  if (!clean) throw new Error('密钥为空')
  let bits = 0
  let value = 0
  const out: number[] = []
  for (const ch of clean) {
    const idx = BASE32_ALPHABET.indexOf(ch)
    if (idx === -1) throw new Error(`无效的 Base32 字符: ${ch}`)
    value = (value << 5) | idx
    bits += 5
    if (bits >= 8) {
      out.push((value >>> (bits - 8)) & 0xff)
      bits -= 8
    }
  }
  return out
}

/** 8 字节大端计数器 */
function counterBytes(counter: number): number[] {
  const bytes = new Array(8).fill(0)
  let n = counter
  for (let i = 7; i >= 0; i--) {
    bytes[i] = n & 0xff
    n = Math.floor(n / 256)
  }
  return bytes
}

function wordArrayToBytes(wa: CryptoJS.lib.WordArray): number[] {
  const out: number[] = []
  for (let i = 0; i < wa.sigBytes; i++) {
    out.push((wa.words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff)
  }
  return out
}

/** HOTP：基于计数器的动态口令 */
export function hotp(secret: string, counter: number, algorithm: TotpAlgorithm = 'SHA1', digits = 6): string {
  const key = CryptoJS.lib.WordArray.create(base32Decode(secret))
  const msg = CryptoJS.lib.WordArray.create(counterBytes(counter))
  const digest = wordArrayToBytes(ALGO_MAP[algorithm](msg, key))
  const offset = digest[digest.length - 1] & 0x0f
  const bin =
    ((digest[offset] & 0x7f) << 24) |
    ((digest[offset + 1] & 0xff) << 16) |
    ((digest[offset + 2] & 0xff) << 8) |
    (digest[offset + 3] & 0xff)
  return (bin % 10 ** digits).toString().padStart(digits, '0')
}

/** TOTP：基于时间的动态口令，返回验证码 + 剩余秒数 + 进度 */
export function totp(secret: string, opts: TotpOptions = { algorithm: 'SHA1', digits: 6, period: 30 }): TotpResult {
  const { algorithm, digits, period } = opts
  const seconds = Math.floor((opts.timestamp ?? Date.now()) / 1000)
  const counter = Math.floor(seconds / period)
  const remaining = period - (seconds % period)
  return {
    code: hotp(secret, counter, algorithm, digits),
    remaining,
    progress: (period - remaining) / period,
  }
}

/** 解析 otpauth:// URI（扫码/链接导入） */
export function parseOtpauthUri(uri: string): OtpauthParams {
  const trimmed = uri.trim()
  const m = trimmed.match(/^otpauth:\/\/(totp|hotp)\/(.+?)(?:\?(.*))?$/i)
  if (!m) throw new Error('不是有效的 otpauth:// 链接')
  const type = m[1].toLowerCase() as 'totp' | 'hotp'
  const label = decodeURIComponent(m[2])
  const query = new URLSearchParams(m[3] || '')
  const secret = query.get('secret')
  if (!secret) throw new Error('链接缺少 secret 参数')

  // label 形如 "Issuer:account"，issuer 参数优先
  let issuer = query.get('issuer') || ''
  let account = label
  const colon = label.indexOf(':')
  if (colon !== -1) {
    const labelIssuer = label.slice(0, colon).trim()
    account = label.slice(colon + 1).trim()
    if (!issuer) issuer = labelIssuer
  } else {
    account = label.trim()
  }

  const algoRaw = (query.get('algorithm') || 'SHA1').toUpperCase()
  const algorithm: TotpAlgorithm = algoRaw === 'SHA256' || algoRaw === 'SHA512' ? algoRaw : 'SHA1'
  const digits = Math.min(10, Math.max(4, parseInt(query.get('digits') || '6', 10) || 6))
  const period = Math.max(1, parseInt(query.get('period') || '30', 10) || 30)
  const counter = type === 'hotp' ? parseInt(query.get('counter') || '0', 10) || 0 : undefined

  return { type, secret: secret.replace(/\s/g, ''), issuer, account, algorithm, digits, period, counter }
}

/** 格式化为 6 位分组显示：123 456 */
export function formatCode(code: string): string {
  if (code.length === 6) return `${code.slice(0, 3)} ${code.slice(3)}`
  if (code.length === 8) return `${code.slice(0, 4)} ${code.slice(4)}`
  return code
}

/** 手动输入密钥的合法性校验 */
export function validateSecret(secret: string): string {
  try {
    const bytes = base32Decode(secret)
    if (!bytes.length) return '密钥无效'
    return ''
  } catch (e) {
    return e instanceof Error ? e.message : '密钥无效'
  }
}
