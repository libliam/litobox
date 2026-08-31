// ========== UUID v4 ==========
export function generateUUID(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0
    const v = c === 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

export interface UUIDOptions {
  count: number
  uppercase: boolean
  removeDashes: boolean
}

export function generateUUIDs(options: UUIDOptions = { count: 1, uppercase: false, removeDashes: false }): string[] {
  const uuids: string[] = []
  for (let i = 0; i < options.count; i++) {
    let uuid = generateUUID()
    if (options.uppercase) {
      uuid = uuid.toUpperCase()
    }
    if (options.removeDashes) {
      uuid = uuid.replace(/-/g, '')
    }
    uuids.push(uuid)
  }
  return uuids
}

// ========== 雪花算法 ID（标准 64 位：41位时间戳 + 10位机器ID + 12位序列号） ==========
// 自定义起始时间戳（epoch），默认 2020-01-01 00:00:00 UTC
const SNOWFLAKE_EPOCH = Date.UTC(2020, 0, 1)
let snowflakeLastTs = 0
let snowflakeSeq = 0

export function generateSnowflakeId(machineId = 0): string {
  let now = Date.now()
  if (now === snowflakeLastTs) {
    // 同一毫秒内递增序列号
    snowflakeSeq = (snowflakeSeq + 1) & 0xfff
    if (snowflakeSeq === 0) {
      // 序列号用尽（4096个/毫秒），等待下一毫秒
      while (now <= snowflakeLastTs) now = Date.now()
      snowflakeLastTs = now
    }
  } else {
    snowflakeLastTs = now
    snowflakeSeq = 0
  }
  const timestamp = BigInt(now - SNOWFLAKE_EPOCH) << 22n
  const machine = BigInt(machineId & 0x3ff) << 12n
  return (timestamp | machine | BigInt(snowflakeSeq)).toString()
}

// ========== MongoDB ObjectId（12 字节：4字节时间戳 + 5字节随机 + 3字节自增计数，24 位十六进制） ==========
let objectIdCounter = Math.floor(Math.random() * 0xffffff)

export function generateObjectId(): string {
  const timestamp = Math.floor(Date.now() / 1000).toString(16).padStart(8, '0')
  const random = Array.from(crypto.getRandomValues(new Uint8Array(5)))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
  objectIdCounter = (objectIdCounter + 1) & 0xffffff
  return timestamp + random + objectIdCounter.toString(16).padStart(6, '0')
}

// ========== 数字自增序列（自定义前缀 + 起始值 + 步长 + 位数补零） ==========
export interface SequentialIdOptions {
  count: number
  prefix: string
  start: number
  step: number
  padLength: number // 0 表示不补零
}

export function generateSequentialIds(options: SequentialIdOptions): string[] {
  const ids: string[] = []
  let cur = options.start
  for (let i = 0; i < options.count; i++) {
    let num = String(cur)
    if (options.padLength > 0) {
      num = num.padStart(options.padLength, '0')
    }
    ids.push(options.prefix + num)
    cur += options.step
  }
  return ids
}

// ========== NanoID（URL 安全短 ID，默认 21 字符） ==========
// 64 字符字母表（256 % 64 = 0，取模无偏差）
const NANOID_ALPHABET = 'useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict'

export function generateNanoId(size = 21): string {
  const bytes = crypto.getRandomValues(new Uint8Array(size))
  let id = ''
  for (let i = 0; i < size; i++) {
    id += NANOID_ALPHABET[bytes[i] % NANOID_ALPHABET.length]
  }
  return id
}

// ========== ULID（26 字符：48bit 时间戳 + 80bit 随机，时间有序） ==========
const CROCKFORD = '0123456789ABCDEFGHJKMNPQRSTVWXYZ' // Crockford Base32（无 I/L/O/U）

export function generateUlid(): string {
  const timePart: string[] = []
  let ts = BigInt(Date.now())
  for (let i = 0; i < 10; i++) {
    timePart.unshift(CROCKFORD[Number(ts & 31n)])
    ts >>= 5n
  }
  const randBytes = crypto.getRandomValues(new Uint8Array(10))
  let rand = 0n
  for (const b of randBytes) rand = (rand << 8n) | BigInt(b)
  const randPart: string[] = []
  for (let i = 0; i < 16; i++) {
    randPart.unshift(CROCKFORD[Number(rand & 31n)])
    rand >>= 5n
  }
  return timePart.join('') + randPart.join('')
}

// ========== UUID v7（RFC 9562：48bit 时间戳 + version 7 + 随机，时间有序） ==========
export function generateUuidV7(): string {
  const bytes = new Uint8Array(16)
  let ts = BigInt(Date.now())
  for (let i = 5; i >= 0; i--) {
    bytes[i] = Number(ts & 0xffn)
    ts >>= 8n
  }
  bytes.set(crypto.getRandomValues(new Uint8Array(10)), 6)
  bytes[6] = (bytes[6] & 0x0f) | 0x70 // version 7
  bytes[8] = (bytes[8] & 0x3f) | 0x80 // variant 10
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  return hex.replace(/(\w{8})(\w{4})(\w{4})(\w{4})(\w{12})/, '$1-$2-$3-$4-$5')
}

// ========== UUID v1（时间戳 + 时钟序列 + 节点，可反解生成时间） ==========
const GREGORIAN_OFFSET = 122192928000000000n // 1582-10-15 至 1970-01-01 的 100ns 数

export function generateUuidV1(): string {
  const ts100ns = BigInt(Date.now()) * 10000n + GREGORIAN_OFFSET
  const tsHex = ts100ns.toString(16).padStart(15, '0') // 60bit 时间戳 = 15 hex
  const timeLow = tsHex.slice(-8)
  const timeMid = tsHex.slice(-12, -8)
  const timeHi = tsHex.slice(-15, -12)
  const clockSeq = crypto.getRandomValues(new Uint8Array(2))
  const seq = (((clockSeq[0] << 8) | clockSeq[1]) & 0x3fff) | 0x8000 // variant 10
  const node = Array.from(crypto.getRandomValues(new Uint8Array(6)), (b) => b.toString(16).padStart(2, '0')).join('')
  return `${timeLow}-${timeMid}-1${timeHi}-${seq.toString(16).padStart(4, '0')}-${node}`
}

// ========== UUID v5（RFC 4122：命名空间 + 名字 SHA-1 哈希，确定性） ==========
// 同一名字 + 同一命名空间永远得到同一个 ID
export const DNS_NAMESPACE = '6ba7b810-9dad-11d1-80b4-00c04fd430c8'

function parseHexUuid(uuid: string): Uint8Array | null {
  const hex = uuid.replace(/-/g, '')
  if (!/^[0-9a-f]{32}$/i.test(hex)) return null
  const bytes = new Uint8Array(16)
  for (let i = 0; i < 16; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
  }
  return bytes
}

export async function generateUuidV5(name: string, namespace = DNS_NAMESPACE): Promise<string> {
  const nsBytes = parseHexUuid(namespace) ?? parseHexUuid(DNS_NAMESPACE)!
  const nameBytes = new TextEncoder().encode(name)
  const data = new Uint8Array(nsBytes.length + nameBytes.length)
  data.set(nsBytes, 0)
  data.set(nameBytes, nsBytes.length)
  const hash = new Uint8Array(await crypto.subtle.digest('SHA-1', data))
  const bytes = hash.slice(0, 16)
  bytes[6] = (bytes[6] & 0x0f) | 0x50 // version 5
  bytes[8] = (bytes[8] & 0x3f) | 0x80 // variant 10
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  return hex.replace(/(\w{8})(\w{4})(\w{4})(\w{4})(\w{12})/, '$1-$2-$3-$4-$5')
}

// ========== KSUID（27 字符 Base62：4 字节秒级时间戳 + 16 字节随机，时间有序） ==========
const BASE62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'

export function generateKsuid(): string {
  const bytes = new Uint8Array(20)
  new DataView(bytes.buffer).setUint32(0, Math.floor(Date.now() / 1000))
  bytes.set(crypto.getRandomValues(new Uint8Array(16)), 4)
  let n = 0n
  for (const b of bytes) n = (n << 8n) | BigInt(b)
  let out = ''
  for (let i = 0; i < 27; i++) {
    out = BASE62[Number(n % 62n)] + out
    n /= 62n
  }
  return out
}

// ========== CUID2（24 字符 base36 短 ID：时间 + 计数器 + 随机，支持自定义前缀） ==========
const BASE36 = '0123456789abcdefghijklmnopqrstuvwxyz'
let cuid2Counter = Math.floor(Math.random() * 0x100000000)

export function generateCuid2(prefix = '', size = 24): string {
  const time = Date.now().toString(36).slice(-6)
  const count = cuid2Counter.toString(36)
  cuid2Counter = (cuid2Counter + 1) % 0x100000000
  const randBytes = crypto.getRandomValues(new Uint8Array(size))
  let randomPart = ''
  for (const b of randBytes) randomPart += BASE36[b % 36]
  return prefix + (time + count + randomPart).slice(0, size)
}

// ========== XID（20 字符 Base32hex：4 字节秒级时间戳 + 机器ID + 进程ID + 3 字节计数） ==========
const BASE32HEX = '0123456789abcdefghijklmnopqrstuv'
let xidCounter = Math.floor(Math.random() * 0xffffff)

export function generateXid(): string {
  const bytes = new Uint8Array(12)
  new DataView(bytes.buffer).setUint32(0, Math.floor(Date.now() / 1000))
  // 机器ID(3字节) + 进程ID(2字节)：浏览器无 MAC/进程号，用密码学随机替代
  bytes.set(crypto.getRandomValues(new Uint8Array(5)), 4)
  xidCounter = (xidCounter + 1) & 0xffffff
  bytes[9] = (xidCounter >> 16) & 0xff
  bytes[10] = (xidCounter >> 8) & 0xff
  bytes[11] = xidCounter & 0xff
  let n = 0n
  for (const b of bytes) n = (n << 8n) | BigInt(b)
  let out = ''
  for (let i = 0; i < 20; i++) {
    out = BASE32HEX[Number(n & 31n)] + out
    n >>= 5n
  }
  return out
}

// ========== 统一入口 ==========
export type IdType = 'uuid' | 'uuidv1' | 'uuidv5' | 'uuidv7' | 'snowflake' | 'objectid' | 'sequence' | 'nanoid' | 'ulid' | 'ksuid' | 'cuid2' | 'xid'

export interface IdGenerateOptions {
  type: IdType
  count: number
  uppercase?: boolean
  removeDashes?: boolean
  machineId?: number
  prefix?: string
  start?: number
  step?: number
  padLength?: number
  nanoIdSize?: number
  name?: string
  namespace?: string
}

export async function generateIds(options: IdGenerateOptions): Promise<string[]> {
  switch (options.type) {
    case 'uuid':
      return generateUUIDs({
        count: options.count,
        uppercase: !!options.uppercase,
        removeDashes: !!options.removeDashes,
      })
    case 'snowflake':
      return Array.from({ length: options.count }, () => generateSnowflakeId(options.machineId ?? 0))
    case 'objectid':
      return Array.from({ length: options.count }, () => generateObjectId())
    case 'sequence':
      return generateSequentialIds({
        count: options.count,
        prefix: options.prefix ?? '',
        start: options.start ?? 1,
        step: options.step ?? 1,
        padLength: options.padLength ?? 0,
      })
    case 'nanoid':
      return Array.from({ length: options.count }, () => generateNanoId(options.nanoIdSize ?? 21))
    case 'ulid':
      return Array.from({ length: options.count }, () => generateUlid())
    case 'uuidv7':
      return Array.from({ length: options.count }, () => generateUuidV7())
    case 'uuidv1':
      return Array.from({ length: options.count }, () => generateUuidV1())
    case 'uuidv5':
      // 确定性 ID：同一名字 + 命名空间恒为同一结果，仅生成 1 个
      return [await generateUuidV5(options.name ?? '', options.namespace)]
    case 'ksuid':
      return Array.from({ length: options.count }, () => generateKsuid())
    case 'cuid2':
      return Array.from({ length: options.count }, () => generateCuid2(options.prefix ?? ''))
    case 'xid':
      return Array.from({ length: options.count }, () => generateXid())
    default:
      return []
  }
}

export const ID_TYPE_LABELS: Record<IdType, string> = {
  uuid: 'UUID v4',
  uuidv1: 'UUID v1',
  uuidv5: 'UUID v5',
  uuidv7: 'UUID v7',
  snowflake: '雪花算法ID',
  objectid: 'MongoDB ObjectId',
  sequence: '自增序列',
  nanoid: 'NanoID',
  ulid: 'ULID',
  ksuid: 'KSUID',
  cuid2: 'CUID2',
  xid: 'XID',
}
