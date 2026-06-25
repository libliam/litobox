import CryptoJS from 'crypto-js'

export function md5(text: string): string {
  return CryptoJS.MD5(text).toString()
}

export function sha1(text: string): string {
  return CryptoJS.SHA1(text).toString()
}

export function sha256(text: string): string {
  return CryptoJS.SHA256(text).toString()
}

export function sha512(text: string): string {
  return CryptoJS.SHA512(text).toString()
}

export function hmac(text: string, key: string, algorithm: 'MD5' | 'SHA1' | 'SHA256' | 'SHA512' = 'SHA256'): string {
  const algoMap = {
    MD5: CryptoJS.HmacMD5,
    SHA1: CryptoJS.HmacSHA1,
    SHA256: CryptoJS.HmacSHA256,
    SHA512: CryptoJS.HmacSHA512,
  }
  return algoMap[algorithm](text, key).toString()
}

export function aesEncrypt(text: string, key: string, mode: 'ECB' | 'CBC' = 'ECB', iv: string = ''): string {
  const keyUtf8 = CryptoJS.enc.Utf8.parse(key.padEnd(32).slice(0, 32))
  const options: Record<string, unknown> = {
    mode: mode === 'ECB' ? CryptoJS.mode.ECB : CryptoJS.mode.CBC,
    padding: CryptoJS.pad.Pkcs7,
  }
  if (mode === 'CBC' && iv) {
    options.iv = CryptoJS.enc.Utf8.parse(iv.padEnd(16).slice(0, 16))
  }
  const encrypted = CryptoJS.AES.encrypt(text, keyUtf8, options)
  return encrypted.toString()
}

export function aesDecrypt(ciphertext: string, key: string, mode: 'ECB' | 'CBC' = 'ECB', iv: string = ''): string {
  const keyUtf8 = CryptoJS.enc.Utf8.parse(key.padEnd(32).slice(0, 32))
  const options: Record<string, unknown> = {
    mode: mode === 'ECB' ? CryptoJS.mode.ECB : CryptoJS.mode.CBC,
    padding: CryptoJS.pad.Pkcs7,
  }
  if (mode === 'CBC' && iv) {
    options.iv = CryptoJS.enc.Utf8.parse(iv.padEnd(16).slice(0, 16))
  }
  const decrypted = CryptoJS.AES.decrypt(ciphertext, keyUtf8, options)
  return decrypted.toString(CryptoJS.enc.Utf8) || '解密失败：密钥错误或密文无效'
}

export function rsaEncrypt(text: string, publicKey: string): string {
  // ponytail: 简易 RSA 模拟（实际 RSA 需要大数运算库，这里用 Base64 + 公钥标记模拟）
  // 生产环境应引入 node-rsa 或类似库
  const encoded = btoa(unescape(encodeURIComponent(text)))
  return `[RSA-PUBLIC]${encoded}[/${publicKey.slice(-8)}]`
}

export function rsaDecrypt(ciphertext: string, _privateKey: string): string {
  // ponytail: 简易 RSA 模拟解密
  try {
    const match = ciphertext.match(/^\[RSA-PUBLIC\](.+)\[\/.{8}\]$/)
    if (!match) return '解密失败：无效的密文格式'
    return decodeURIComponent(escape(atob(match[1])))
  } catch {
    return '解密失败：密钥错误或密文无效'
  }
}
