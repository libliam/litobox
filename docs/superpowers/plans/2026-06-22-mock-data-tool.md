# 随机假数据工具实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增随机假数据生成工具，支持 11 种数据类型（姓名、身份证、手机号、邮箱、IP、网址、地址、银行卡号、统一社会信用代码、车架号、车牌号），卡片网格布局，纯前端实现。

**Architecture:** 在 `src/utils/mockDataUtils.ts` 中创建纯函数生成各类数据，在 `src/views/MockDataTool.vue` 中创建卡片网格页面，修改 `store/index.ts` 和 `App.vue` 注册路由。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus

---

## 文件结构

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/utils/mockDataUtils.ts` | 创建 | 数据生成工具函数 |
| `src/views/MockDataTool.vue` | 创建 | 页面组件 |
| `src/store/index.ts` | 修改 | 添加工具列表项 |
| `src/App.vue` | 修改 | 添加路由 |

---

### Task 1: 创建数据生成工具函数

**Files:**
- Create: `src/utils/mockDataUtils.ts`

- [ ] **Step 1: 创建 mockDataUtils.ts 文件**

```typescript
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

    // 顺序码（第17位决定性别）
    const sequence = String(randomInt(1, 999)).padStart(3, '0')
    const genderDigit = gender === 'male'
      ? String(randomPick([0, 2, 4, 6, 8]))
      : gender === 'female'
        ? String(randomPick([1, 3, 5, 7, 9]))
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
```

- [ ] **Step 2: 验证文件创建成功**

确认 `src/utils/mockDataUtils.ts` 文件已创建，无语法错误。

---

### Task 2: 创建 MockDataTool.vue 页面

**Files:**
- Create: `src/views/MockDataTool.vue`

- [ ] **Step 1: 创建页面组件**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">随机假数据</span>
          <el-tooltip placement="bottom" effect="light">
            <template #content>
              <div class="tooltip-content">
                <p>生成各类模拟测试数据</p>
                <p>• 所有数据均为随机生成，仅供测试使用</p>
                <p>• 每个卡片可独立配置生成数量</p>
                <p>• 部分类型支持额外选项配置</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <el-button size="small" type="primary" @click="handleGenerateAll">全部生成</el-button>
      </div>
    </div>

    <div class="data-grid">
      <!-- 姓名 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">姓名</span>
          <el-button size="small" @click="handleGenerate('name')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.name.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.name.gender" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="男" value="male" />
              <el-option label="女" value="female" />
            </el-select>
          </div>
          <div v-if="results.name.length" class="result-list">
            <div v-for="(item, idx) in results.name" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 身份证 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">身份证</span>
          <el-button size="small" @click="handleGenerate('idCard')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.idCard.count" :min="1" :max="20" size="small" style="width: 80px" />
            <el-select v-model="options.idCard.gender" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="男" value="male" />
              <el-option label="女" value="female" />
            </el-select>
          </div>
          <div v-if="results.idCard.length" class="result-list">
            <div v-for="(item, idx) in results.idCard" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 手机号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">手机号</span>
          <el-button size="small" @click="handleGenerate('phone')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.phone.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.phone.carrier" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="移动" value="mobile" />
              <el-option label="联通" value="unicom" />
              <el-option label="电信" value="telecom" />
            </el-select>
          </div>
          <div v-if="results.phone.length" class="result-list">
            <div v-for="(item, idx) in results.phone" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 邮箱 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">邮箱</span>
          <el-button size="small" @click="handleGenerate('email')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.email.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.email.domain" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="QQ" value="qq" />
              <el-option label="163" value="163" />
              <el-option label="Gmail" value="gmail" />
            </el-select>
          </div>
          <div v-if="results.email.length" class="result-list">
            <div v-for="(item, idx) in results.email" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- IP地址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">IP地址</span>
          <el-button size="small" @click="handleGenerate('ip')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.ip.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.ip.type" size="small" style="width: 80px">
              <el-option label="IPv4" value="ipv4" />
              <el-option label="IPv6" value="ipv6" />
            </el-select>
          </div>
          <div v-if="results.ip.length" class="result-list">
            <div v-for="(item, idx) in results.ip" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 网址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">网址</span>
          <el-button size="small" @click="handleGenerate('url')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.url.count" :min="1" :max="50" size="small" style="width: 80px" />
            <el-select v-model="options.url.protocol" size="small" style="width: 80px">
              <el-option label="随机" value="random" />
              <el-option label="HTTP" value="http" />
              <el-option label="HTTPS" value="https" />
            </el-select>
          </div>
          <div v-if="results.url.length" class="result-list">
            <div v-for="(item, idx) in results.url" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 国内地址 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">国内地址</span>
          <el-button size="small" @click="handleGenerate('address')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.address.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.address.length" class="result-list">
            <div v-for="(item, idx) in results.address" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 银行卡号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">银行卡号</span>
          <el-button size="small" @click="handleGenerate('bankCard')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.bankCard.count" :min="1" :max="20" size="small" style="width: 80px" />
            <el-select v-model="options.bankCard.type" size="small" style="width: 80px">
              <el-option label="储蓄卡" value="debit" />
              <el-option label="信用卡" value="credit" />
            </el-select>
          </div>
          <div v-if="results.bankCard.length" class="result-list">
            <div v-for="(item, idx) in results.bankCard" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 统一社会信用代码 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">统一社会信用代码</span>
          <el-button size="small" @click="handleGenerate('creditCode')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.creditCode.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.creditCode.length" class="result-list">
            <div v-for="(item, idx) in results.creditCode" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 车架号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">车架号</span>
          <el-button size="small" @click="handleGenerate('vin')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.vin.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.vin.length" class="result-list">
            <div v-for="(item, idx) in results.vin" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 车牌号 -->
      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">车牌号</span>
          <el-button size="small" @click="handleGenerate('plate')">生成</el-button>
        </div>
        <div class="card-body">
          <div class="options-row">
            <span class="option-label">数量</span>
            <el-input-number v-model="options.plate.count" :min="1" :max="20" size="small" style="width: 80px" />
          </div>
          <div v-if="results.plate.length" class="result-list">
            <div v-for="(item, idx) in results.plate" :key="idx" class="data-item">
              <span class="item-index">#{{ idx + 1 }}</span>
              <code class="item-text">{{ item }}</code>
              <el-button size="small" text @click="handleCopy(item)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  generateName,
  generateIdCard,
  generatePhone,
  generateEmail,
  generateIP,
  generateURL,
  generateAddress,
  generateBankCard,
  generateCreditCode,
  generateVIN,
  generatePlate
} from '@/utils/mockDataUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// 配置选项
const options = reactive({
  name: { count: 5, gender: 'random' as 'male' | 'female' | 'random' },
  idCard: { count: 5, gender: 'random' as 'male' | 'female' | 'random', ageRange: 'random' as 'adult' | 'elder' | 'random' },
  phone: { count: 5, carrier: 'random' as 'mobile' | 'unicom' | 'telecom' | 'random' },
  email: { count: 5, domain: 'random' as 'qq' | '163' | '126' | 'gmail' | 'outlook' | 'sina' | 'random' },
  ip: { count: 5, type: 'ipv4' as 'ipv4' | 'ipv6' },
  url: { count: 5, protocol: 'random' as 'http' | 'https' | 'random' },
  address: { count: 5 },
  bankCard: { count: 5, type: 'debit' as 'debit' | 'credit' },
  creditCode: { count: 5 },
  vin: { count: 5 },
  plate: { count: 5 }
})

// 结果存储
const results = reactive<Record<string, string[]>>({
  name: [],
  idCard: [],
  phone: [],
  email: [],
  ip: [],
  url: [],
  address: [],
  bankCard: [],
  creditCode: [],
  vin: [],
  plate: []
})

// 生成单个类型
const handleGenerate = (type: string) => {
  switch (type) {
    case 'name':
      results.name = generateName(options.name)
      break
    case 'idCard':
      results.idCard = generateIdCard(options.idCard)
      break
    case 'phone':
      results.phone = generatePhone(options.phone)
      break
    case 'email':
      results.email = generateEmail(options.email)
      break
    case 'ip':
      results.ip = generateIP(options.ip)
      break
    case 'url':
      results.url = generateURL(options.url)
      break
    case 'address':
      results.address = generateAddress(options.address)
      break
    case 'bankCard':
      results.bankCard = generateBankCard(options.bankCard)
      break
    case 'creditCode':
      results.creditCode = generateCreditCode(options.creditCode)
      break
    case 'vin':
      results.vin = generateVIN(options.vin)
      break
    case 'plate':
      results.plate = generatePlate(options.plate)
      break
  }

  store.addHistory({
    tool: 'mockData',
    action: `generate_${type}`,
    inputPreview: `count=${(options as any)[type].count}`,
    outputPreview: results[type][0] || ''
  })

  ElMessage.success(`已生成 ${(options as any)[type].count} 条数据`)
}

// 全部生成
const handleGenerateAll = () => {
  Object.keys(options).forEach(type => {
    handleGenerate(type)
  })
}

// 复制
const handleCopy = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

/* 工具卡片 */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

/* 置顶卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  margin-bottom: 16px;
}

/* 标题栏 */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

/* 卡片内容 */
.card-body {
  padding: 16px 20px;
}

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover {
  color: var(--accent-cyan);
}
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}
.tooltip-content p {
  margin: 2px 0;
}

/* 网格布局 */
.data-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
@media (max-width: 1200px) {
  .data-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
@media (max-width: 768px) {
  .data-grid {
    grid-template-columns: 1fr;
  }
}

/* 选项行 */
.options-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}
.option-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

/* 结果列表 */
.result-list {
  max-height: 200px;
  overflow-y: auto;
}
.data-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  margin-bottom: 8px;
  transition: border-color 0.3s;
}
.data-item:hover {
  border-color: var(--accent-cyan);
}
.item-index {
  font-size: 11px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  min-width: 30px;
  text-align: center;
}
.item-text {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

/* 滚动条 */
.result-list::-webkit-scrollbar {
  width: 4px;
}
.result-list::-webkit-scrollbar-track {
  background: transparent;
}
.result-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}
</style>
```

- [ ] **Step 2: 验证页面创建成功**

确认 `src/views/MockDataTool.vue` 文件已创建，无语法错误。

---

### Task 3: 注册工具到系统

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 修改 store/index.ts**

在 `TOOL_LIST` 数组末尾添加工具项：

```typescript
{ id: 'mockData', name: '随机数据', icon: '🎲', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 8v8M8 12h8"/><circle cx="8" cy="8" r="1" fill="currentColor"/><circle cx="16" cy="8" r="1" fill="currentColor"/><circle cx="8" cy="16" r="1" fill="currentColor"/><circle cx="16" cy="16" r="1" fill="currentColor"/></svg>`, description: '姓名、身份证、手机号等随机生成', keywords: ['随机', '假数据', 'mock', '测试数据'] }
```

- [ ] **Step 2: 修改 App.vue**

在 import 区域添加：
```typescript
import MockDataTool from '@/views/MockDataTool.vue'
```

在模板路由区域添加（在 `<HistoryView v-else-if="activeTool === 'history'" />` 之前）：
```vue
<MockDataTool v-else-if="activeTool === 'mockData'" />
```

- [ ] **Step 3: 验证修改**

确认两个文件修改正确，无语法错误。

---

### Task 4: 测试验证

- [ ] **Step 1: 启动开发服务器**

```bash
npm run dev
```

- [ ] **Step 2: 功能验证**

1. 打开应用，确认侧边栏出现"随机数据"工具
2. 点击进入随机数据工具页面
3. 验证卡片网格布局正确显示（每行 3 个卡片）
4. 逐个点击每个卡片的"生成"按钮，验证数据生成正确
5. 点击"全部生成"按钮，验证所有类型同时生成
6. 验证复制功能正常工作
7. 验证数量选项和额外选项（性别、运营商等）正常工作
8. 验证响应式布局（调整窗口大小）

- [ ] **Step 3: 提交代码**

```bash
git add src/utils/mockDataUtils.ts src/views/MockDataTool.vue src/store/index.ts src/App.vue
git commit -m "feat: 新增随机假数据生成工具"
```

---

## 自审检查

1. **规范覆盖**：所有 11 种数据类型都有对应的生成函数和 UI 卡片 ✓
2. **占位符扫描**：无 TBD/TODO，所有代码完整 ✓
3. **类型一致性**：工具函数参数类型与页面调用匹配 ✓
4. **样式一致性**：遵循项目科技风 UI 规范，使用 CSS 变量 ✓
