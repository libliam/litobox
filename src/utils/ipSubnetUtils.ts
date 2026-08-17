/**
 * IPv4 子网计算工具（纯函数，零依赖）
 *
 * ponytail: JS 位运算按 32 位有符号处理，>2^31 时会失真。
 * 这里主机数、IP 范围合并等涉及大数的场景一律用普通算术 + Number（安全整数到 2^53），
 * 避免位运算陷阱。仅 <2^31 的掩码计算用位运算（掩码最大 0xFFFFFFFF，用 >>> 0 转无符号）。
 */

// ============ IP ↔ 整数 ============

/** IPv4 点分十进制 → 无符号 32 位整数（0 ~ 4294967295） */
export function ipToInt(ip: string): number {
  const parts = ip.trim().split('.')
  if (parts.length !== 4) throw new Error('IPv4 地址必须为 4 段点分十进制')
  const nums = parts.map(p => Number(p))
  if (nums.some(n => !Number.isInteger(n) || n < 0 || n > 255)) {
    throw new Error('IPv4 各段必须在 0~255 之间')
  }
  // 用乘法而非位运算，避免 >2^31 的有符号溢出
  return nums[0] * 0x1000000 + nums[1] * 0x10000 + nums[2] * 0x100 + nums[3]
}

/** 无符号 32 位整数 → IPv4 点分十进制 */
export function intToIp(int: number): string {
  if (!Number.isInteger(int) || int < 0 || int > 0xFFFFFFFF) {
    throw new Error('整数必须在 0~4294967295 之间')
  }
  return [
    Math.floor(int / 0x1000000) & 0xFF,
    Math.floor(int / 0x10000) & 0xFF,
    Math.floor(int / 0x100) & 0xFF,
    int & 0xFF,
  ].join('.')
}

/** 整数 → 32 位二进制字符串 */
export function intToBinary(int: number): string {
  if (!Number.isInteger(int) || int < 0 || int > 0xFFFFFFFF) {
    throw new Error('整数必须在 0~4294967295 之间')
  }
  let s = ''
  for (let i = 31; i >= 0; i--) {
    s += (Math.floor(int / Math.pow(2, i)) & 1).toString()
  }
  return s
}

export function ipToBinary(ip: string): string {
  return intToBinary(ipToInt(ip))
}

// ============ 掩码 ↔ CIDR ============

/** CIDR 前缀长度 → 子网掩码整数 */
export function cidrToMaskInt(cidr: number): number {
  if (!Number.isInteger(cidr) || cidr < 0 || cidr > 32) {
    throw new Error('CIDR 必须在 0~32 之间')
  }
  if (cidr === 0) return 0
  // cidr=32 时 0xFFFFFFFF，用 Math.pow 避免 << 在 32 位时的边界问题
  return (Math.pow(2, cidr) - 1) * Math.pow(2, 32 - cidr)
}

export function cidrToMask(cidr: number): string {
  return intToIp(cidrToMaskInt(cidr))
}

/** 子网掩码（点分十进制）→ CIDR 前缀长度 */
export function maskToCidr(mask: string): number {
  const int = ipToInt(mask)
  // 掩码必须是连续高位 1 + 低位 0
  // 合法掩码：~int + 1 是 2 的幂（即 int 是 ...111000... 形式）
  const inv = (~int >>> 0) + 1
  // 检查 inv 是否为 2 的幂（inv & (inv-1) === 0），但 inv 可能 > 2^31
  if (int !== 0 && (inv & (inv - 1)) !== 0) {
    throw new Error('子网掩码必须为连续高位 1（如 255.255.255.0）')
  }
  // 数连续高位的 1
  let count = 0
  let n = int
  // 从最高位开始数
  for (let i = 31; i >= 0; i--) {
    if (Math.floor(n / Math.pow(2, i)) & 1) count++
    else break
  }
  return count
}

/** 反掩码（wildcard mask） */
export function wildcardFromCidr(cidr: number): string {
  return intToIp((~cidrToMaskInt(cidr)) >>> 0)
}

// ============ 输入解析 ============

/** 解析用户输入，支持两种格式：192.168.1.0/24 或 192.168.1.0 255.255.255.0 */
export function parseInput(input: string): { ip: string; cidr: number } {
  const trimmed = input.trim()
  if (!trimmed) throw new Error('请输入 IP 地址')

  // 格式1：CIDR
  if (trimmed.includes('/')) {
    const [ip, cidrStr] = trimmed.split('/')
    const cidr = Number(cidrStr)
    if (!Number.isInteger(cidr) || cidr < 0 || cidr > 32) {
      throw new Error('CIDR 前缀长度必须在 0~32 之间')
    }
    return { ip: ip.trim(), cidr }
  }

  // 格式2：IP + 掩码（空格分隔）
  const parts = trimmed.split(/\s+/)
  if (parts.length === 2) {
    return { ip: parts[0], cidr: maskToCidr(parts[1]) }
  }

  // 格式3：仅 IP，默认 /32
  return { ip: trimmed, cidr: 32 }
}

// ============ 子网信息 ============

export interface SubnetInfo {
  ip: string
  cidr: number
  networkAddress: string
  broadcastAddress: string
  mask: string
  wildcard: string
  firstHost: string
  lastHost: string
  hostCount: number
  ipClass: string
  isPrivate: boolean
  isLoopback: boolean
  binaryIp: string
  binaryMask: string
}

/** 判断 IP 类别（A/B/C/D/E/环回） */
export function getIpClass(ip: string): string {
  const first = ipToInt(ip) / 0x1000000 & 0xFF
  if (first === 127) return '环回 (Loopback)'
  if (first >= 1 && first <= 126) return 'A'
  if (first >= 128 && first <= 191) return 'B'
  if (first >= 192 && first <= 223) return 'C'
  if (first >= 224 && first <= 239) return 'D（组播）'
  return 'E（保留）'
}

/** 是否私网地址 */
export function isPrivateIp(ip: string): boolean {
  const int = ipToInt(ip)
  // 10.0.0.0/8
  if (Math.floor(int / 0x1000000) === 10) return true
  // 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
  if (Math.floor(int / 0x10000) >= 0xAC10 && Math.floor(int / 0x10000) <= 0xAC1F) return true
  // 192.168.0.0/16
  if (Math.floor(int / 0x10000) === 0xC0A8) return true
  return false
}

/** 是否环回地址 127.0.0.0/8 */
export function isLoopbackIp(ip: string): boolean {
  return (ipToInt(ip) / 0x1000000 & 0xFF) === 127
}

/** 计算子网完整信息 */
export function calcSubnet(input: string): SubnetInfo {
  const { ip, cidr } = parseInput(input)
  const ipInt = ipToInt(ip)
  const maskInt = cidrToMaskInt(cidr)
  const networkInt = Math.floor(ipInt / Math.pow(2, 32 - cidr)) * Math.pow(2, 32 - cidr)
  const broadcastInt = cidr === 0 ? 0xFFFFFFFF : networkInt + Math.pow(2, 32 - cidr) - 1

  // 可用主机数：/31 和 /32 特殊处理
  let hostCount: number
  let firstHost: string
  let lastHost: string
  if (cidr >= 31) {
    // /31 点对点链路（RFC 3021）两个地址都可用；/32 单主机
    hostCount = cidr === 31 ? 2 : 1
    firstHost = intToIp(networkInt)
    lastHost = intToIp(broadcastInt)
  } else {
    hostCount = Math.pow(2, 32 - cidr) - 2
    firstHost = intToIp(networkInt + 1)
    lastHost = intToIp(broadcastInt - 1)
  }

  return {
    ip,
    cidr,
    networkAddress: intToIp(networkInt),
    broadcastAddress: intToIp(broadcastInt),
    mask: intToIp(maskInt),
    wildcard: intToIp((~maskInt) >>> 0),
    firstHost,
    lastHost,
    hostCount,
    ipClass: getIpClass(ip),
    isPrivate: isPrivateIp(ip),
    isLoopback: isLoopbackIp(ip),
    binaryIp: intToBinary(ipInt),
    binaryMask: intToBinary(maskInt),
  }
}

// ============ 子网划分 ============

export interface SubnetDivision {
  index: number
  network: string
  cidr: string
  firstHost: string
  lastHost: string
  broadcast: string
  hostCount: number
}

/**
 * 子网划分
 * @param input 原网络（CIDR 或 IP+掩码）
 * @param mode 'count' 按子网数量划分 | 'hosts' 按每子网主机数划分
 * @param value 子网数量 或 每子网可用主机数
 */
export function divideSubnets(input: string, mode: 'count' | 'hosts', value: number): SubnetDivision[] {
  const { ip, cidr } = parseInput(input)
  const baseNetworkInt = Math.floor(ipToInt(ip) / Math.pow(2, 32 - cidr)) * Math.pow(2, 32 - cidr)

  let newCidr: number
  if (mode === 'count') {
    if (!Number.isInteger(value) || value < 1) throw new Error('子网数量必须为正整数')
    // 找最小的 2^k >= value，新前缀 = 原前缀 + k
    let k = 0
    while (Math.pow(2, k) < value) k++
    newCidr = cidr + k
  } else {
    if (!Number.isInteger(value) || value < 2) throw new Error('每子网主机数至少为 2')
    // 找最小的 2^k >= value+2（网络+广播），新前缀 = 32 - k
    let k = 0
    while (Math.pow(2, k) < value + 2) k++
    newCidr = 32 - k
  }

  if (newCidr > 32) throw new Error('划分粒度过细，超出 32 位限制')
  if (newCidr <= cidr) throw new Error('划分后子网前缀必须大于原前缀，请检查输入')

  const blockSize = Math.pow(2, 32 - newCidr)
  const totalSubnets = Math.pow(2, newCidr - cidr)
  const result: SubnetDivision[] = []

  for (let i = 0; i < totalSubnets; i++) {
    const networkInt = baseNetworkInt + i * blockSize
    const broadcastInt = networkInt + blockSize - 1
    const hostCount = newCidr >= 31
      ? (newCidr === 31 ? 2 : 1)
      : blockSize - 2
    result.push({
      index: i + 1,
      network: intToIp(networkInt),
      cidr: `/${newCidr}`,
      firstHost: intToIp(newCidr >= 31 ? networkInt : networkInt + 1),
      lastHost: intToIp(newCidr >= 31 ? broadcastInt : broadcastInt - 1),
      broadcast: intToIp(broadcastInt),
      hostCount,
    })
  }
  return result
}

// ============ IP 范围 → 最小 CIDR 列表 ============

export interface CidrRange {
  cidr: string
  network: string
  broadcast: string
  hostCount: number
}

/**
 * 将起止 IP 范围转换为最少的 CIDR 块列表（标准范围合并算法）
 * ponytail: 用普通算术求最低 set bit，规避 JS 位运算 >2^31 的有符号溢出。
 * 复杂度 O(log(end-start))，IPv4 范围内最多输出 32 个块。
 */
export function ipRangeToCidrs(startIp: string, endIp: string): CidrRange[] {
  let start = ipToInt(startIp)
  const end = ipToInt(endIp)
  if (start > end) throw new Error('起始 IP 不能大于结束 IP')

  const result: CidrRange[] = []
  let safety = 0
  while (start <= end && safety++ < 64) {
    // start 的最低 set bit（对齐约束）；start=0 时块大小可达 2^32
    const maxByAlign = lowestSetBit(start)
    const maxByEnd = end - start + 1
    const maxSize = Math.min(maxByAlign, maxByEnd)
    // 找最大的 2 的幂 <= maxSize
    const size = Math.pow(2, Math.floor(Math.log2(maxSize)))
    const cidr = 32 - Math.floor(Math.log2(size))
    const networkInt = start
    const broadcastInt = start + size - 1
    const hostCount = cidr >= 31 ? (cidr === 31 ? 2 : 1) : size - 2
    result.push({
      cidr: `${intToIp(networkInt)}/${cidr}`,
      network: intToIp(networkInt),
      broadcast: intToIp(broadcastInt),
      hostCount,
    })
    start += size
  }
  return result
}

/** 求一个非负整数的最低 set bit 的值（2 的幂）；0 返回 2^32 */
function lowestSetBit(n: number): number {
  if (n === 0) return Math.pow(2, 32)
  // 用除法求最低 set bit，避免位运算的 32 位符号陷阱
  let bit = 1
  while (n % 2 === 0) {
    n = Math.floor(n / 2)
    bit *= 2
  }
  return bit
}

// ============ 自检（AGENTS.md 规范：非平凡逻辑留可运行检查） ============

/** 运行时自检，逻辑错误时抛出异常 */
export function selfCheck(): void {
  const assert = (cond: boolean, msg: string) => { if (!cond) throw new Error(`ipSubnetUtils 自检失败: ${msg}`) }

  // IP ↔ 整数
  assert(ipToInt('0.0.0.0') === 0, '0.0.0.0 → 0')
  assert(ipToInt('127.0.0.1') === 0x7F000001, '127.0.0.1')
  assert(ipToInt('255.255.255.255') === 0xFFFFFFFF, '255.255.255.255')
  assert(intToIp(0) === '0.0.0.0', '0 → 0.0.0.0')
  assert(intToIp(0xFFFFFFFF) === '255.255.255.255', 'max → 255.255.255.255')
  assert(intToIp(0x7F000001) === '127.0.0.1', '127.0.0.1 round-trip')

  // 掩码 ↔ CIDR
  assert(cidrToMask(24) === '255.255.255.0', '/24 mask')
  assert(cidrToMask(16) === '255.255.0.0', '/16 mask')
  assert(cidrToMask(0) === '0.0.0.0', '/0 mask')
  assert(cidrToMask(32) === '255.255.255.255', '/32 mask')
  assert(maskToCidr('255.255.255.0') === 24, 'mask 24')
  assert(maskToCidr('255.255.0.0') === 16, 'mask 16')
  assert(maskToCidr('0.0.0.0') === 0, 'mask 0')

  // 子网计算
  const s = calcSubnet('192.168.1.100/24')
  assert(s.networkAddress === '192.168.1.0', '/24 network')
  assert(s.broadcastAddress === '192.168.1.255', '/24 broadcast')
  assert(s.firstHost === '192.168.1.1', '/24 first host')
  assert(s.lastHost === '192.168.1.254', '/24 last host')
  assert(s.hostCount === 254, '/24 host count')
  assert(s.ipClass === 'C', 'class C')
  assert(s.isPrivate === true, '192.168 is private')

  const s2 = calcSubnet('10.0.0.1/8')
  assert(s2.hostCount === 16777214, '/8 host count')

  // /31 点对点
  const s3 = calcSubnet('192.168.1.0/31')
  assert(s3.hostCount === 2, '/31 host count')

  // 子网划分
  const d = divideSubnets('192.168.1.0/24', 'count', 4)
  assert(d.length === 4, 'divide into 4')
  assert(d[0].network === '192.168.1.0', 'first subnet network')
  assert(d[1].network === '192.168.1.64', 'second subnet network')
  assert(d[3].network === '192.168.1.192', 'fourth subnet network')

  // IP 范围合并
  const r = ipRangeToCidrs('192.168.1.0', '192.168.1.255')
  assert(r.length === 1 && r[0].cidr === '192.168.1.0/24', 'range merge /24')

  // 192.168.0.0 ~ 192.168.1.255 = 512 地址，且起始对齐 /23 边界 → 合并为 1 个 /23
  const r2 = ipRangeToCidrs('192.168.0.0', '192.168.1.255')
  assert(r2.length === 1 && r2[0].cidr === '192.168.0.0/23', 'range merge to /23')

  // 不对齐的范围需要多个块：192.168.0.1 ~ 192.168.1.254
  const r3 = ipRangeToCidrs('192.168.0.1', '192.168.1.254')
  assert(r3.length > 1, 'unaligned range needs multiple blocks')

  // 边界：大数（>2^31）IP 范围
  const r4 = ipRangeToCidrs('128.0.0.0', '128.0.0.255')
  assert(r4.length === 1 && r4[0].cidr === '128.0.0.0/24', 'large IP range >2^31')

  console.log('[ipSubnetUtils] 自检通过 ✓')
}
