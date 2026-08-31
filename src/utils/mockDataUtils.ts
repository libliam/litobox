// 姓氏库
const SURNAMES = [
  '王', '李', '张', '刘', '陈', '杨', '黄', '赵', '周', '吴',
  '徐', '孙', '马', '朱', '胡', '郭', '何', '高', '林', '罗',
  '郑', '梁', '谢', '宋', '唐', '许', '韩', '冯', '邓', '曹',
  '彭', '曾', '肖', '田', '董', '袁', '潘', '于', '蒋', '蔡',
  '余', '杜', '叶', '程', '魏', '苏', '吕', '丁', '任', '沈',
  '姚', '卢', '姜', '崔', '钟', '谭', '陆', '汪', '范', '金',
  '石', '廖', '贾', '夏', '韦', '付', '方', '白', '邹', '孟',
  '熊', '秦', '邱', '江', '尹', '薛', '闫', '段', '雷', '侯',
  '龙', '史', '陶', '黎', '贺', '顾', '毛', '郝', '龚', '邵',
  '万', '覃', '武', '戴', '莫', '孔', '向', '汤'
]

// 男性名字用字
const MALE_CHARS = [
  '伟', '强', '磊', '军', '勇', '杰', '明', '华', '亮', '龙',
  '飞', '涛', '鹏', '波', '浩', '然', '宇', '轩', '昊', '睿',
  '翔', '毅', '峰', '斌', '超', '鑫', '健', '凯', '辉', '阳'
]

// 女性名字用字
const FEMALE_CHARS = [
  '芳', '娜', '敏', '静', '丽', '婷', '雪', '燕', '玲', '蕾',
  '莹', '洁', '琳', '颖', '茜', '媛', '佳', '欣', '悦', '彤',
  '瑶', '瑾', '萱', '妍', '菲', '晴', '梦', '诗', '露', '蓉'
]

// 通用名字用字
const COMMON_CHARS = [
  '文', '博', '思', '心', '天', '云', '风', '雨', '晨', '光',
  '平', '安', '康', '乐', '宁', '欢', '庆', '祥', '瑞', '德'
]

/**
 * 生成随机整数 [min, max]
 */
function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

/**
 * 从数组中随机取一个元素
 */
function randomPick<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)]
}

/**
 * 生成姓名
 */
export function generateName(options: {
  count: number
  gender?: 'male' | 'female' | 'random'
}): string[] {
  const { count, gender = 'random' } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    const surname = randomPick(SURNAMES)
    let charPool: string[]

    if (gender === 'male') {
      charPool = MALE_CHARS
    } else if (gender === 'female') {
      charPool = FEMALE_CHARS
    } else {
      charPool = [...MALE_CHARS, ...FEMALE_CHARS, ...COMMON_CHARS]
    }

    // 名字长度 1-2 个字
    const nameLen = Math.random() > 0.5 ? 2 : 1
    const name = Array.from({ length: nameLen }, () => randomPick(charPool)).join('')
    results.push(surname + name)
  }

  return results
}

/**
 * 生成身份证号
 */
export function generateIdCard(options: {
  count: number
  gender?: 'male' | 'female' | 'random'
  ageRange?: 'adult' | 'elder' | 'random'
}): string[] {
  const { count, gender = 'random', ageRange = 'random' } = options
  const results: string[] = []

  // 省份代码
  const PROVINCE_CODES = [
    '11', '12', '13', '14', '15', '21', '22', '23', '31', '32',
    '33', '34', '35', '36', '37', '41', '42', '43', '44', '45',
    '46', '50', '51', '52', '53', '54', '61', '62', '63', '64', '65'
  ]

  // 计算校验位
  function calcCheckCode(id17: string): string {
    const weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
    const checkCodes = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2']
    let sum = 0
    for (let i = 0; i < 17; i++) {
      sum += parseInt(id17[i]) * weights[i]
    }
    return checkCodes[sum % 11]
  }

  for (let i = 0; i < count; i++) {
    const province = randomPick(PROVINCE_CODES)
    const city = String(randomInt(1, 20)).padStart(2, '0')
    const district = String(randomInt(1, 20)).padStart(2, '0')

    // 出生日期
    let year: number
    if (ageRange === 'adult') {
      year = randomInt(1970, 2005)
    } else if (ageRange === 'elder') {
      year = randomInt(1940, 1970)
    } else {
      year = randomInt(1950, 2005)
    }
    const month = String(randomInt(1, 12)).padStart(2, '0')
    const day = String(randomInt(1, 28)).padStart(2, '0')
    const birthday = `${year}${month}${day}`

    // 顺序码（第17位决定性别：奇数=男，偶数=女）
    const sequence = String(randomInt(1, 999)).padStart(3, '0')
    const genderDigit = gender === 'male'
      ? String(randomPick([1, 3, 5, 7, 9]))
      : gender === 'female'
        ? String(randomPick([0, 2, 4, 6, 8]))
        : String(randomInt(0, 9))

    const id17 = province + city + district + birthday + sequence.slice(0, 2) + genderDigit
    const checkCode = calcCheckCode(id17)

    results.push(id17 + checkCode)
  }

  return results
}

/**
 * 生成手机号
 */
export function generatePhone(options: {
  count: number
  carrier?: 'mobile' | 'unicom' | 'telecom' | 'random'
}): string[] {
  const { count, carrier = 'random' } = options
  const results: string[] = []

  const prefixes: Record<string, string[]> = {
    mobile: ['134', '135', '136', '137', '138', '139', '150', '151', '152', '157', '158', '159', '182', '183', '184', '187', '188'],
    unicom: ['130', '131', '132', '155', '156', '185', '186'],
    telecom: ['133', '153', '180', '181', '189']
  }

  for (let i = 0; i < count; i++) {
    let prefix: string
    if (carrier === 'random') {
      const carrierType = randomPick(['mobile', 'unicom', 'telecom'] as const)
      prefix = randomPick(prefixes[carrierType])
    } else {
      prefix = randomPick(prefixes[carrier])
    }
    const suffix = String(randomInt(0, 99999999)).padStart(8, '0')
    results.push(prefix + suffix)
  }

  return results
}

/**
 * 生成邮箱
 */
export function generateEmail(options: {
  count: number
  domain?: 'qq' | '163' | '126' | 'gmail' | 'outlook' | 'sina' | 'random'
}): string[] {
  const { count, domain = 'random' } = options
  const results: string[] = []

  const domains: Record<string, string> = {
    qq: 'qq.com',
    '163': '163.com',
    '126': '126.com',
    gmail: 'gmail.com',
    outlook: 'outlook.com',
    sina: 'sina.com'
  }

  const domainKeys = Object.keys(domains) as Array<keyof typeof domains>

  for (let i = 0; i < count; i++) {
    // 生成用户名：字母+数字组合
    const usernameLen = randomInt(6, 12)
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
    const username = Array.from({ length: usernameLen }, () =>
      chars[Math.floor(Math.random() * chars.length)]
    ).join('')

    let domainValue: string
    if (domain === 'random') {
      domainValue = domains[randomPick(domainKeys)]
    } else {
      domainValue = domains[domain]
    }

    results.push(`${username}@${domainValue}`)
  }

  return results
}

/**
 * 生成 IP 地址
 */
export function generateIP(options: {
  count: number
  type?: 'ipv4' | 'ipv6'
}): string[] {
  const { count, type = 'ipv4' } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    if (type === 'ipv6') {
      // 生成简化的 IPv6
      const segments = Array.from({ length: 8 }, () =>
        Math.floor(Math.random() * 65536).toString(16).padStart(4, '0')
      )
      results.push(segments.join(':'))
    } else {
      // 生成 IPv4
      const octets = Array.from({ length: 4 }, () => randomInt(1, 254))
      results.push(octets.join('.'))
    }
  }

  return results
}

/**
 * 生成网址
 */
export function generateURL(options: {
  count: number
  protocol?: 'http' | 'https' | 'random'
}): string[] {
  const { count, protocol = 'random' } = options
  const results: string[] = []

  const domains = [
    'example.com', 'test.com', 'demo.org', 'sample.net',
    'mysite.com', 'webapp.io', 'project.dev', 'app.co'
  ]

  const paths = [
    '/api/v1/users', '/products/list', '/dashboard', '/login',
    '/docs/getting-started', '/search?q=test', '/profile/settings',
    '/admin/panel', '/blog/post/123', '/about'
  ]

  for (let i = 0; i < count; i++) {
    const proto = protocol === 'random'
      ? (Math.random() > 0.5 ? 'https' : 'http')
      : protocol

    const domain = randomPick(domains)
    const path = randomPick(paths)

    results.push(`${proto}://${domain}${path}`)
  }

  return results
}

/**
 * 生成国内地址
 */
export function generateAddress(options: {
  count: number
  province?: string
}): string[] {
  const { count, province } = options
  const results: string[] = []

  const provinces = [
    '北京市', '上海市', '广东省', '浙江省', '江苏省', '四川省',
    '湖北省', '湖南省', '河南省', '山东省', '河北省', '福建省',
    '安徽省', '辽宁省', '陕西省', '重庆市', '天津市'
  ]

  const cities: Record<string, string[]> = {
    '北京市': ['东城区', '西城区', '朝阳区', '海淀区', '丰台区'],
    '上海市': ['黄浦区', '浦东新区', '静安区', '徐汇区', '长宁区'],
    '广东省': ['广州市', '深圳市', '珠海市', '佛山市', '东莞市'],
    '浙江省': ['杭州市', '宁波市', '温州市', '嘉兴市', '绍兴市'],
    '江苏省': ['南京市', '苏州市', '无锡市', '常州市', '南通市'],
    '四川省': ['成都市', '绵阳市', '德阳市', '乐山市', '宜宾市'],
    '湖北省': ['武汉市', '宜昌市', '襄阳市', '荆州市', '黄石市'],
    '湖南省': ['长沙市', '株洲市', '湘潭市', '衡阳市', '岳阳市'],
    '河南省': ['郑州市', '洛阳市', '开封市', '安阳市', '南阳市'],
    '山东省': ['济南市', '青岛市', '烟台市', '潍坊市', '威海市'],
    '河北省': ['石家庄市', '唐山市', '保定市', '邯郸市', '廊坊市'],
    '福建省': ['福州市', '厦门市', '泉州市', '漳州市', '莆田市'],
    '安徽省': ['合肥市', '芜湖市', '蚌埠市', '马鞍山市', '安庆市'],
    '辽宁省': ['沈阳市', '大连市', '鞍山市', '抚顺市', '本溪市'],
    '陕西省': ['西安市', '宝鸡市', '咸阳市', '铜川市', '渭南市'],
    '重庆市': ['渝中区', '江北区', '南岸区', '沙坪坝区', '九龙坡区'],
    '天津市': ['和平区', '河东区', '河西区', '南开区', '河北区']
  }

  const streets = ['中山路', '人民路', '建设路', '解放路', '和平路', '胜利路', '光明路', '文化路', '幸福路', '团结路']
  const roadTypes = ['段', '号', '巷', '街']

  for (let i = 0; i < count; i++) {
    const prov = province || randomPick(provinces)
    const cityList = cities[prov] || ['市区']
    const city = randomPick(cityList)
    const street = randomPick(streets)
    const roadType = randomPick(roadTypes)
    const number = randomInt(1, 999)
    const detail = `${randomInt(1, 30)}栋${randomInt(1, 3000)}室`

    results.push(`${prov}${city}${street}${roadType}${number}号${detail}`)
  }

  return results
}

/**
 * 生成银行卡号（Luhn 算法）
 */
export function generateBankCard(options: {
  count: number
  type?: 'debit' | 'credit'
}): string[] {
  const { count, type = 'debit' } = options
  const results: string[] = []

  // BIN 号段
  const bins: Record<string, string[]> = {
    debit: ['621700', '622202', '622848', '623051', '621226', '622588', '622609'],
    credit: ['436745', '518710', '524374', '552599', '622575', '622579']
  }

  // Luhn 校验位计算
  function calcLuhnCheckDigit(number: string): number {
    let sum = 0
    let isDouble = true
    for (let i = number.length - 1; i >= 0; i--) {
      let digit = parseInt(number[i])
      if (isDouble) {
        digit *= 2
        if (digit > 9) digit -= 9
      }
      sum += digit
      isDouble = !isDouble
    }
    return (10 - (sum % 10)) % 10
  }

  for (let i = 0; i < count; i++) {
    const bin = randomPick(bins[type])
    // 卡号总长度 16-19 位
    const totalLen = randomInt(16, 19)
    const middleLen = totalLen - bin.length - 1
    const middle = Array.from({ length: middleLen }, () => randomInt(0, 9)).join('')
    const partial = bin + middle
    const checkDigit = calcLuhnCheckDigit(partial)

    results.push(partial + checkDigit)
  }

  return results
}

/**
 * 生成统一社会信用代码
 */
export function generateCreditCode(options: {
  count: number
}): string[] {
  const { count } = options
  const results: string[] = []

  // 登记管理部门代码
  const deptCodes = ['1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'N', 'Y']
  // 机构类别代码
  const typeCodes = ['1', '2', '3', '4', '5', '6', '7', '8', '9']
  // 登记管理机关行政区划代码
  const regionCodes = [
    '110000', '120000', '130000', '310000', '320000', '330000',
    '440000', '440100', '440300', '500000', '510000'
  ]

  // 字符集
  const chars = '0123456789ABCDEFGHJKLMNPQRTUWXY'
  const weights = [1, 3, 9, 27, 19, 26, 16, 17, 20, 29, 25, 13, 8, 24, 10, 30, 28]

  function calcCheckCode(base: string): string {
    let sum = 0
    for (let i = 0; i < base.length; i++) {
      sum += chars.indexOf(base[i]) * weights[i]
    }
    const checkIndex = 31 - (sum % 31)
    return chars[checkIndex === 31 ? 0 : checkIndex]
  }

  for (let i = 0; i < count; i++) {
    const dept = randomPick(deptCodes)
    const type = randomPick(typeCodes)
    const region = randomPick(regionCodes)
    const orgCode = Array.from({ length: 9 }, () => randomPick(chars.split(''))).join('')

    const base17 = dept + type + region + orgCode
    const checkCode = calcCheckCode(base17)

    results.push(base17 + checkCode)
  }

  return results
}

/**
 * 生成车架号（VIN）
 */
export function generateVIN(options: {
  count: number
}): string[] {
  const { count } = options
  const results: string[] = []

  // VIN 字符集（不含 I、O、Q）
  const vinChars = 'ABCDEFGHJKLMNPRSTUVWXYZ0123456789'

  // 校验位权重
  const weights = [8, 7, 6, 5, 4, 3, 2, 10, 0, 9, 8, 7, 6, 5, 4, 3, 2]
  // 字符对应数值
  const charValues: Record<string, number> = {
    'A': 1, 'B': 2, 'C': 3, 'D': 4, 'E': 5, 'F': 6, 'G': 7, 'H': 8,
    'J': 1, 'K': 2, 'L': 3, 'M': 4, 'N': 5, 'P': 7, 'R': 9,
    'S': 2, 'T': 3, 'U': 4, 'V': 5, 'W': 6, 'X': 7, 'Y': 8, 'Z': 9,
    '0': 0, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9
  }

  function calcVINCheckChar(vin16: string): string {
    let sum = 0
    for (let i = 0; i < 16; i++) {
      sum += (charValues[vin16[i]] || 0) * weights[i]
    }
    const remainder = sum % 11
    return remainder === 10 ? 'X' : String(remainder)
  }

  for (let i = 0; i < count; i++) {
    // WMI（世界制造厂识别代号）3 位
    const wmi = Array.from({ length: 3 }, () => randomPick(vinChars.split(''))).join('')
    // VDS（车辆说明部分）5 位
    const vds = Array.from({ length: 5 }, () => randomPick(vinChars.split(''))).join('')
    // 年份代码
    const yearChars = 'ABCDEFGHJKLMNPRSTVWXY123456789'
    const year = randomPick(yearChars.split(''))
    // 装配厂代码
    const plant = randomPick(vinChars.split(''))
    // 生产序号 6 位
    const serial = Array.from({ length: 6 }, () => randomPick(vinChars.split(''))).join('')

    const vin16 = wmi + vds + year + plant + serial
    const checkChar = calcVINCheckChar(vin16)

    // 第 9 位是校验位
    results.push(vin16.slice(0, 8) + checkChar + vin16.slice(8))
  }

  return results
}

/**
 * 生成车牌号
 */
export function generatePlate(options: {
  count: number
  province?: string
}): string[] {
  const { count, province } = options
  const results: string[] = []

  const provinces: Record<string, string[]> = {
    '京': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'Y'],
    '沪': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'R', 'S'],
    '粤': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '苏': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '浙': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '川': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '鄂': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '湘': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '豫': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'],
    '鲁': ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z']
  }

  const plateChars = 'ABCDEFGHJKLMNPQRSTUVWXYZ0123456789'

  for (let i = 0; i < count; i++) {
    let prov: string
    let letter: string

    if (province) {
      prov = province
      const letters = provinces[province] || ['A']
      letter = randomPick(letters)
    } else {
      const provKeys = Object.keys(provinces)
      prov = randomPick(provKeys)
      letter = randomPick(provinces[prov])
    }

    // 车牌后 5 位
    const suffix = Array.from({ length: 5 }, () => randomPick(plateChars.split(''))).join('')

    results.push(`${prov}${letter}${suffix}`)
  }

  return results
}

/**
 * 生成 MAC 地址
 */
export function generateMAC(options: {
  count: number
  format?: 'colon' | 'dash' | 'dot'
}): string[] {
  const { count, format = 'colon' } = options
  const results: string[] = []

  const separators: Record<string, string> = {
    colon: ':',
    dash: '-',
    dot: '.'
  }

  for (let i = 0; i < count; i++) {
    const segments = Array.from({ length: 6 }, () =>
      Math.floor(Math.random() * 256).toString(16).padStart(2, '0').toUpperCase()
    )

    if (format === 'dot') {
      // AA:BB:CC.DD:EE:FF 格式（Cisco 风格）
      results.push(`${segments.slice(0, 3).join(':')}.${segments.slice(3).join(':')}`)
    } else {
      const sep = separators[format]
      results.push(segments.join(sep))
    }
  }

  return results
}

/**
 * 中文填充文本（Lorem Ipsum）
 */
export function generateText(options: {
  count: number
  wordCount?: number
}): string[] {
  const { count, wordCount = 100 } = options
  const results: string[] = []

  const sentences = [
    '春眠不觉晓，处处闻啼鸟。',
    '床前明月光，疑是地上霜。',
    '白日依山尽，黄河入海流。',
    '千山鸟飞绝，万径人踪灭。',
    '大漠孤烟直，长河落日圆。',
    '海内存知己，天涯若比邻。',
    '会当凌绝顶，一览众山小。',
    '采菊东篱下，悠然见南山。',
    '明月松间照，清泉石上流。',
    '空山新雨后，天气晚来秋。',
    '野火烧不尽，春风吹又生。',
    '随风潜入夜，润物细无声。',
    '两个黄鹂鸣翠柳，一行白鹭上青天。',
    '接天莲叶无穷碧，映日荷花别样红。',
    '山重水复疑无路，柳暗花明又一村。',
    '落霞与孤鹜齐飞，秋水共长天一色。',
    '天生我材必有用，千金散尽还复来。',
    '长风破浪会有时，直挂云帆济沧海。',
    '路漫漫其修远兮，吾将上下而求索。',
    '不畏浮云遮望眼，自缘身在最高层。'
  ]

  for (let i = 0; i < count; i++) {
    const textParts: string[] = []
    let currentLen = 0
    while (currentLen < wordCount) {
      const sentence = sentences[Math.floor(Math.random() * sentences.length)]
      textParts.push(sentence)
      currentLen += sentence.length
    }
    results.push(textParts.join(''))
  }

  return results
}

/**
 * 生成随机日期时间
 */
export function generateDateTime(options: {
  count: number
  range?: 'recent7' | 'recent30' | 'recent365' | 'custom'
  startDate?: string
  endDate?: string
  format?: 'date' | 'datetime' | 'timestamp'
}): string[] {
  const { count, range = 'recent30', startDate, endDate, format = 'datetime' } = options
  const results: string[] = []

  const rangeDays: Record<string, number> = {
    recent7: 7,
    recent30: 30,
    recent365: 365
  }

  const now = Date.now()
  let startMs: number
  let endMs: number

  if (range === 'custom' && startDate && endDate) {
    startMs = new Date(startDate).getTime()
    endMs = new Date(endDate).getTime()
  } else {
    const days = rangeDays[range] || 30
    startMs = now - days * 24 * 60 * 60 * 1000
    endMs = now
  }

  for (let i = 0; i < count; i++) {
    const randomMs = startMs + Math.random() * (endMs - startMs)
    const date = new Date(randomMs)

    if (format === 'timestamp') {
      results.push(String(Math.floor(randomMs / 1000)))
    } else if (format === 'date') {
      const y = date.getFullYear()
      const m = String(date.getMonth() + 1).padStart(2, '0')
      const d = String(date.getDate()).padStart(2, '0')
      results.push(`${y}-${m}-${d}`)
    } else {
      const y = date.getFullYear()
      const m = String(date.getMonth() + 1).padStart(2, '0')
      const d = String(date.getDate()).padStart(2, '0')
      const h = String(date.getHours()).padStart(2, '0')
      const min = String(date.getMinutes()).padStart(2, '0')
      const s = String(date.getSeconds()).padStart(2, '0')
      results.push(`${y}-${m}-${d} ${h}:${min}:${s}`)
    }
  }

  return results
}

/**
 * 生成邮政编码
 */
export function generateZipCode(options: {
  count: number
  province?: string
}): string[] {
  const { count, province } = options
  const results: string[] = []

  // 省份邮编前两位映射
  const provincePrefixes: Record<string, string[]> = {
    '北京': ['10'],
    '上海': ['20'],
    '天津': ['30'],
    '重庆': ['40'],
    '广东': ['51', '52'],
    '浙江': ['31', '32'],
    '江苏': ['21', '22'],
    '四川': ['61', '62', '63', '64'],
    '湖北': ['43', '44'],
    '湖南': ['41', '42'],
    '河南': ['45', '46', '47'],
    '山东': ['25', '26', '27'],
    '河北': ['05', '06', '07'],
    '福建': ['35', '36'],
    '安徽': ['23', '24'],
    '辽宁': ['11', '12'],
    '陕西': ['71', '72'],
  }

  for (let i = 0; i < count; i++) {
    let prefix: string
    if (province && provincePrefixes[province]) {
      prefix = randomPick(provincePrefixes[province])
    } else {
      const allPrefixes = Object.values(provincePrefixes).flat()
      prefix = randomPick(allPrefixes)
    }
    const suffix = String(randomInt(0, 9999)).padStart(4, '0')
    results.push(prefix + suffix)
  }

  return results
}

/**
 * 生成 UUID v4
 */
export function generateUUID(options: {
  count: number
  format?: 'standard' | 'no-dash' | 'upper'
}): string[] {
  const { count, format = 'standard' } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    let uuid = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0
      const v = c === 'x' ? r : (r & 0x3 | 0x8)
      return v.toString(16)
    })

    if (format === 'no-dash') {
      uuid = uuid.replace(/-/g, '')
    } else if (format === 'upper') {
      uuid = uuid.toUpperCase()
    }

    results.push(uuid)
  }

  return results
}

/**
 * 生成颜色值
 */
export function generateColor(options: {
  count: number
  format?: 'hex' | 'rgb' | 'hsl'
}): string[] {
  const { count, format = 'hex' } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    const r = randomInt(0, 255)
    const g = randomInt(0, 255)
    const b = randomInt(0, 255)

    if (format === 'hex') {
      results.push(`#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`)
    } else if (format === 'rgb') {
      results.push(`rgb(${r}, ${g}, ${b})`)
    } else {
      // HSL
      const h = randomInt(0, 360)
      const s = randomInt(20, 100)
      const l = randomInt(20, 80)
      results.push(`hsl(${h}, ${s}%, ${l}%)`)
    }
  }

  return results
}

/**
 * 生成随机 JSON 对象
 */
export function generateJSON(options: {
  count: number
  depth?: number
}): string[] {
  const { count, depth = 2 } = options
  const results: string[] = []

  const sampleKeys = ['id', 'name', 'email', 'age', 'city', 'status', 'score', 'price', 'title', 'description', 'url', 'phone', 'address', 'active', 'count']
  const sampleValues = ['hello', 'world', 'test', 'demo', 'sample', 'example', 'data', 'info', 'item', 'value']

  function randomValue(currentDepth: number): any {
    if (currentDepth <= 0) {
      const type = randomPick(['string', 'number', 'boolean', 'null'] as const)
      switch (type) {
        case 'string': return randomPick(sampleValues)
        case 'number': return randomInt(0, 10000)
        case 'boolean': return Math.random() > 0.5
        case 'null': return null
      }
    }

    const type = randomPick(['object', 'array', 'primitive'] as const)
    switch (type) {
      case 'object': {
        const obj: Record<string, any> = {}
        const keyCount = randomInt(2, 5)
        const usedKeys = new Set<string>()
        for (let i = 0; i < keyCount; i++) {
          let key: string
          do {
            key = randomPick(sampleKeys)
          } while (usedKeys.has(key))
          usedKeys.add(key)
          obj[key] = randomValue(currentDepth - 1)
        }
        return obj
      }
      case 'array': {
        const arr: any[] = []
        const len = randomInt(2, 4)
        for (let i = 0; i < len; i++) {
          arr.push(randomValue(currentDepth - 1))
        }
        return arr
      }
      case 'primitive':
        return randomValue(0)
    }
  }

  for (let i = 0; i < count; i++) {
    const obj = randomValue(depth)
    results.push(JSON.stringify(obj, null, 2))
  }

  return results
}

// ========== 英文姓名 ==========
const FIRST_NAMES_MALE = ['James', 'John', 'Robert', 'Michael', 'William', 'David', 'Richard', 'Joseph', 'Thomas', 'Charles', 'Daniel', 'Matthew', 'Anthony', 'Mark', 'Christopher']
const FIRST_NAMES_FEMALE = ['Mary', 'Patricia', 'Jennifer', 'Linda', 'Elizabeth', 'Barbara', 'Susan', 'Jessica', 'Sarah', 'Karen', 'Lisa', 'Nancy', 'Betty', 'Sandra', 'Margaret']
const LAST_NAMES = ['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez', 'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson', 'Taylor', 'Moore', 'Jackson', 'Martin', 'Lee', 'Perez', 'Thompson', 'White']

/**
 * 生成英文姓名（First + Last）
 */
export function generateEnglishName(options: {
  count: number
  gender?: 'male' | 'female' | 'random'
}): string[] {
  const { count, gender = 'random' } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    const g = gender === 'random' ? randomPick(['male', 'female'] as const) : gender
    const first = g === 'male' ? randomPick(FIRST_NAMES_MALE) : randomPick(FIRST_NAMES_FEMALE)
    results.push(`${first} ${randomPick(LAST_NAMES)}`)
  }

  return results
}

// ========== 经纬度坐标 ==========
/**
 * 生成经纬度坐标（"纬度, 经度"）
 */
export function generateCoordinate(options: {
  count: number
  range?: 'china' | 'global'
  decimals?: number
}): string[] {
  const { count, range = 'china', decimals = 6 } = options
  const results: string[] = []

  const latRange = range === 'china' ? [3, 54] : [-90, 90]
  const lngRange = range === 'china' ? [73, 135] : [-180, 180]

  for (let i = 0; i < count; i++) {
    const lat = (latRange[0] + Math.random() * (latRange[1] - latRange[0])).toFixed(decimals)
    const lng = (lngRange[0] + Math.random() * (lngRange[1] - lngRange[0])).toFixed(decimals)
    results.push(`${lat}, ${lng}`)
  }

  return results
}

// ========== 公司名称 ==========
const COMPANY_REGIONS = ['北京', '上海', '广州', '深圳', '杭州', '成都', '武汉', '南京', '苏州', '西安', '重庆', '天津']
const COMPANY_CHARS = ['瑞', '源', '恒', '达', '宏', '远', '盛', '嘉', '联', '创', '科', '讯', '博', '捷', '信', '通', '云', '星', '鹏', '华', '中', '天', '金']
const COMPANY_INDUSTRIES = ['科技', '网络', '信息技术', '电子商务', '电子', '传媒', '商贸', '实业', '建设工程', '文化', '医药', '食品', '物流', '环保', '智能制造']
const COMPANY_TYPES = ['有限公司', '股份有限公司', '集团有限公司', '合伙企业', '工作室']

/**
 * 生成中文公司名称（地域 + 字号 + 行业 + 组织形式）
 */
export function generateCompanyName(options: {
  count: number
}): string[] {
  const { count } = options
  const results: string[] = []

  for (let i = 0; i < count; i++) {
    const region = randomPick(COMPANY_REGIONS)
    const brandLen = Math.random() > 0.5 ? 2 : 3
    const brand = Array.from({ length: brandLen }, () => randomPick(COMPANY_CHARS)).join('')
    const industry = randomPick(COMPANY_INDUSTRIES)
    const type = randomPick(COMPANY_TYPES)
    results.push(`${region}${brand}${industry}${type}`)
  }

  return results
}

// ========== 个人档案 ==========
/**
 * 生成一套完整个人信息（JSON），内部字段一致：
 * 性别 → 姓名/身份证；出生日期/年龄 ← 身份证解析
 */
export function generatePersonalProfile(options: {
  count: number
}): string[] {
  const { count } = options
  const results: string[] = []

  const genderText = { male: '男', female: '女' } as const

  for (let i = 0; i < count; i++) {
    const gender = randomPick(['male', 'female'] as const)
    const name = generateName({ count: 1, gender })[0]
    const idCard = generateIdCard({ count: 1, gender })[0]
    const birthDate = `${idCard.slice(6, 10)}-${idCard.slice(10, 12)}-${idCard.slice(12, 14)}`
    const age = new Date().getFullYear() - parseInt(idCard.slice(6, 10), 10)

    const profile = {
      name,
      gender: genderText[gender],
      idCard,
      birthDate,
      age,
      phone: generatePhone({ count: 1 })[0],
      email: generateEmail({ count: 1 })[0],
      address: generateAddress({ count: 1 })[0],
      bankCard: generateBankCard({ count: 1 })[0],
    }

    results.push(JSON.stringify(profile, null, 2))
  }

  return results
}

// ========== 用户名 ==========
const USERNAME_ADJ = ['cool', 'happy', 'lucky', 'super', 'big', 'tiny', 'fast', 'smart', 'crazy', 'brave', 'gentle', 'sweet', 'wild', 'clever', 'swift', 'silent', 'bright', 'golden', 'silver', 'mega']
const USERNAME_NOUN = ['panda', 'tiger', 'eagle', 'wolf', 'fox', 'lion', 'bear', 'cat', 'dog', 'bird', 'fish', 'dragon', 'phoenix', 'unicorn', 'koala', 'penguin', 'rabbit', 'turtle', 'shark', 'whale']
const USERNAME_SEP: Record<string, string> = { snake: '_', dot: '.', dash: '-', none: '' }

/**
 * 生成用户名（形容词 + 分隔符 + 名词 + 随机数字后缀）
 */
export function generateUsername(options: {
  count: number
  style?: 'random' | 'snake' | 'dot' | 'dash' | 'none'
}): string[] {
  const { count, style = 'random' } = options
  const results: string[] = []

  const styleKeys = Object.keys(USERNAME_SEP) as Array<keyof typeof USERNAME_SEP>

  for (let i = 0; i < count; i++) {
    const sep = style === 'random' ? USERNAME_SEP[randomPick(styleKeys)] : USERNAME_SEP[style]
    const number = Math.random() > 0.7 ? '' : String(randomInt(10, 9999))
    results.push(`${randomPick(USERNAME_ADJ)}${sep}${randomPick(USERNAME_NOUN)}${number}`)
  }

  return results
}

// ========== 快递单号 ==========
/**
 * 生成快递单号（按快递公司常见格式）
 */
export function generateCourierNumber(options: {
  count: number
  carrier?: 'sf' | 'yt' | 'zt' | 'yd' | 'sto' | 'ems' | 'jd' | 'random'
}): string[] {
  const { count, carrier = 'random' } = options
  const results: string[] = []

  // 前缀字母 + 数字位数
  const carriers: Record<string, { prefix: string; digits: number }> = {
    sf: { prefix: 'SF', digits: 13 },
    yt: { prefix: 'YT', digits: 10 },
    zt: { prefix: 'ZT', digits: 12 },
    yd: { prefix: 'YD', digits: 12 },
    sto: { prefix: 'STO', digits: 12 },
    jd: { prefix: 'JD', digits: 12 },
  }

  for (let i = 0; i < count; i++) {
    const c = carrier === 'random' ? randomPick(Object.keys(carriers)) : carrier

    if (c === 'ems') {
      // EMS：2 字母 + 9 位数字 + CN
      const code = `E${randomPick('ABCDEFGHJKLMNPQRSTUVWXYZ'.split(''))}`
      results.push(`${code}${String(randomInt(0, 999999999)).padStart(9, '0')}CN`)
    } else {
      const { prefix, digits } = carriers[c]
      const num = String(randomInt(0, Math.pow(10, digits) - 1)).padStart(digits, '0')
      results.push(prefix + num)
    }
  }

  return results
}

// ========== 金额 ==========
const CURRENCY_SYMBOL: Record<string, string> = { cny: '¥', usd: '$', eur: '€', none: '' }

const UPPER_DIGITS = ['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖']
const UPPER_INNER_UNITS = ['', '拾', '佰', '仟']
const UPPER_GROUP_UNITS = ['', '万', '亿', '万亿']

/**
 * 4 位一组转大写（如 1005 → 壹仟零伍），连续零合并、末尾零省略
 */
function groupToUpper(g: number): string {
  const str = String(g)
  let result = ''

  for (let i = 0; i < str.length; i++) {
    const digit = parseInt(str[i], 10)
    const pos = str.length - 1 - i

    if (digit !== 0) {
      result += UPPER_DIGITS[digit] + UPPER_INNER_UNITS[pos]
    } else if (i < str.length - 1 && /[1-9]/.test(str.slice(i + 1)) && !result.endsWith('零')) {
      result += '零'
    }
  }

  return result
}

/**
 * 整数部分转中文大写（按 4 位分组：元/万/亿/万亿）
 */
function integerToUpper(num: number): string {
  if (num === 0) return '零'

  const groups: number[] = []
  let n = num
  while (n > 0) {
    groups.push(n % 10000)
    n = Math.floor(n / 10000)
  }

  let result = ''
  for (let i = groups.length - 1; i >= 0; i--) {
    const g = groups[i]
    if (g === 0) continue

    if (result) {
      // 组间补零：本组不足千、或更高组末位为零、或中间隔了全零组
      let needZero = g < 1000
      for (let j = i + 1; j < groups.length; j++) {
        if (groups[j] === 0) {
          needZero = true
          continue
        }
        if (groups[j] % 10 === 0) needZero = true
        break
      }
      if (needZero) result += '零'
    }

    result += groupToUpper(g) + (i > 0 ? UPPER_GROUP_UNITS[i] : '')
  }

  return result
}

/**
 * 金额转人民币大写（最高支持万亿级，两位小数）
 */
export function amountToUpper(num: number): string {
  if (!isFinite(num)) return ''
  const negative = num < 0
  const fixed = Math.abs(num).toFixed(2)
  const [intStr, decStr] = fixed.split('.')

  const intPart = parseInt(intStr, 10)
  const jiao = parseInt(decStr[0] || '0', 10)
  const fen = parseInt(decStr[1] || '0', 10)

  let result = (intPart === 0 ? '零元' : integerToUpper(intPart) + '元')

  if (jiao === 0 && fen === 0) {
    result += '整'
  } else if (jiao === 0) {
    result += '零' + UPPER_DIGITS[fen] + '分'
  } else if (fen === 0) {
    result += UPPER_DIGITS[jiao] + '角整'
  } else {
    result += UPPER_DIGITS[jiao] + '角' + UPPER_DIGITS[fen] + '分'
  }

  return (negative ? '负' : '') + result
}

/**
 * 金额原始值格式化（upper 时输出人民币大写，否则符号 + 千分位）
 */
export function formatAmountValue(value: number, options: {
  decimals?: 0 | 1 | 2
  currency?: 'cny' | 'usd' | 'eur' | 'none'
  upper?: boolean
} = {}): string {
  const { decimals = 2, currency = 'cny', upper = false } = options

  if (upper) return amountToUpper(value)

  const symbol = CURRENCY_SYMBOL[currency]
  const fixed = value.toFixed(decimals)
  const [intPart, decPart] = fixed.split('.')
  return symbol + Number(intPart).toLocaleString('en-US') + (decPart ? '.' + decPart : '')
}

/**
 * 生成金额原始数值（仅随机值，便于切换展示格式时复用）
 */
export function generateAmountValues(options: {
  count: number
  min?: number
  max?: number
}): number[] {
  const { count, min = 1, max = 10000 } = options
  const values: number[] = []

  for (let i = 0; i < count; i++) {
    values.push(min + Math.random() * (max - min))
  }

  return values
}

/**
 * 生成金额（符号 + 千分位，小数位可配；upper 时直接输出人民币大写，忽略货币符号）
 */
export function generateAmount(options: {
  count: number
  min?: number
  max?: number
  decimals?: 0 | 1 | 2
  currency?: 'cny' | 'usd' | 'eur' | 'none'
  upper?: boolean
}): string[] {
  const { count, min = 1, max = 10000, decimals = 2, currency = 'cny', upper = false } = options
  return generateAmountValues({ count, min, max })
    .map(value => formatAmountValue(value, { decimals, currency, upper }))
}
