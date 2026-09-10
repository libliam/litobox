import type { CheatSheetData } from './index'

export const regexCheatsheet: CheatSheetData = {
  id: 'regex',
  name: '正则表达式速查',
  description: '正则元字符/量词/断言/分组/常见 pattern 模板，PCRE/JavaScript/Java/Python 通用',
  columns: [
    { key: 'pattern', label: '表达式', width: '160px', copyable: true },
    { key: 'category', label: '分类', width: '100px' },
    { key: 'desc', label: '说明', width: '260px' },
    { key: 'example', label: '示例（匹配 / 不匹配）' },
  ],
  rows: [
    // === 字符类 ===
    { pattern: '.', category: '元字符', desc: '除换行外任意单个字符', example: ['"." 匹配 "a"/"5"/"_"', '不匹配 "\\n"'] },
    { pattern: '\\d / \\D', category: '元字符', desc: '数字 / 非数字（0-9）', example: ['"\\d{3}" 匹配 "123"', '不匹配 "abc"'] },
    { pattern: '\\w / \\W', category: '元字符', desc: '单词字符（字母数字下划线）/ 非单词字符', example: ['"\\w+" 匹配 "hello_world"', '不匹配 "@#"'] },
    { pattern: '\\s / \\S', category: '元字符', desc: '空白（空格/Tab/换行）/ 非空白', example: ['"a\\sb" 匹配 "a b"/"a\\tb"', '不匹配 "ab"'] },
    { pattern: '[abc] / [^abc] / [a-z]', category: '元字符', desc: '字符集/排除/范围', example: ['"[aeiou]" 匹配元音', '"[^0-9]" 匹配非数字', '"[A-Z]" 匹配大写字母'] },
    { pattern: '\\b / \\B', category: '元字符', desc: '单词边界 / 非单词边界', example: ['"\\bcat\\b" 匹配独立 cat（不匹配 category）', '"\\Bcat\\B" 匹配中间的 cat'] },
    // === 量词 ===
    { pattern: '* / + / ?', category: '量词', desc: '0或多 / 1或多 / 0或1（都是贪婪）', example: ['"ab*c" 匹配 ac/abc/abbc', '"ab+c" 不匹配 ac', '"ab?c" 匹配 ac/abc'] },
    { pattern: '{n} / {n,} / {n,m}', category: '量词', desc: '精确N次 / 至少N次 / N到M次', example: ['"\\d{4}" 匹配 "2024"', '"\\d{3,}" 匹配 3位以上数字', '"\\d{2,4}" 匹配 2-4位'] },
    { pattern: '*? / +? / ?? / {n,m}?', category: '量词', desc: '非贪婪匹配（尽可能短）', example: ['"<.*?>" 在 "<a>b<c>" 匹配 "<a>" 而非 "<a>b<c>"'] },
    // === 位置 ===
    { pattern: '^ / $', category: '位置', desc: '行首 / 行尾（多行模式下每行的首尾）', example: ['"^https://" 匹配行首的 https://', '"/.png$" 匹配 .png 结尾', '多行模式下用 (?m)^/$'] },
    { pattern: '\\A / \\Z / \\z', category: '位置', desc: '绝对首/绝对尾（不受多行模式影响，PCRE/Java/Python）', example: ['"\\Ahttps://" 只匹配整个字符串开头', 'JS 不支持 \\A\\Z'] },
    // === 分组 ===
    { pattern: '(abc) / (?:abc)', category: '分组', desc: '捕获组 / 非捕获组（不占反向引用编号）', example: ['"(a)(b)(c)" 可以 $1=a $2=b $3=c', '"(?:ab)+" 只匹配，不捕获'] },
    { pattern: '\\1 / \\2', category: '分组', desc: '反向引用（匹配前面第N个捕获组的相同文本）', example: ['"(\\w)\\1" 匹配 "aa"/"bb"/"77"（连续相同字符）', '"(.)\\1\\1" 匹配 "aaa"/"bbb"'] },
    { pattern: '(?<name>abc)', category: '分组', desc: '命名捕获组（PCRE/Java/Python/JS ES2018+）', example: ['"(?<year>\\d{4})-(?<month>\\d{2})"', 'Java/Python 用 group("year") 访问'] },
    { pattern: 'a|b', category: '分组', desc: '或（a 或 b）', example: ['"https?://" 匹配 http:// 和 https://', '"cat|dog|fish" 三选一'] },
    // === 断言 ===
    { pattern: '(?=abc) / (?!abc)', category: '断言', desc: '正向前瞻 / 负向前瞻（后面是/不是 abc）', example: ['"\\d(?=元)" 匹配 "100元" 中的 100（只匹配数字）', '"\\w+@(?=com)" 匹配邮箱后缀前的用户名'] },
    { pattern: '(?<=abc) / (?<!abc)', category: '断言', desc: '正向后顾 / 负向后顾（前面是/不是 abc）', example: ['"(?<=￥)\\d+" 匹配 "￥100" 中的 100', '"(?<!a)b" 匹配前面不是 a 的 b'] },
    // === 常见 Pattern 模板 ===
    { pattern: '邮箱', category: '模板', desc: '标准邮箱匹配（实用版）', example: ['"[\\w.+-]+@[\\w-]+\\.[a-zA-Z]{2,6}(\\.[a-zA-Z]{2,4})?"', '匹配 user@domain.com / a.b+c@x.cn', '不匹配 @domain.com / user@ / user@com'] },
    { pattern: '手机号（中国大陆）', category: '模板', desc: '11 位手机号', example: ['"1[3-9]\\d{9}"', '匹配 13812345678 / 19900001111', '不匹配 12345678901 / 123456789'] },
    { pattern: 'URL 提取', category: '模板', desc: 'http/https 链接', example: ['"https?://[\\w.-]+(?:/[\\w./?=&%_-]*)?"', '匹配 https://example.com/path?k=v'] },
    { pattern: 'IP v4', category: '模板', desc: 'IPv4 地址（严格版，含 0-255 校验）', example: ['"((25[0-5]|2[0-4]\\d|[01]?\\d\\d?)\\.){3}(25[0-5]|2[0-4]\\d|[01]?\\d\\d?)"', '匹配 192.168.1.1 / 10.0.0.1', '不匹配 999.999.999.999 / 256.1.1.1'] },
    { pattern: '日期 YYYY-MM-DD', category: '模板', desc: 'ISO 日期格式', example: ['"\\d{4}-\\d{2}-\\d{2}"', '更严格版："\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\\d|3[01])"'] },
    { pattern: '时间 HH:MM(:SS)?', category: '模板', desc: '24 小时制时间', example: ['"(?:[01]\\d|2[0-3]):[0-5]\\d(?::[0-5]\\d)?"', '匹配 09:30 / 23:59:59', '不匹配 25:00'] },
    { pattern: '十六进制颜色', category: '模板', desc: '#RRGGBB 或 #RGB 或带 alpha', example: ['"#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?"', '匹配 #fff / #FFFFFF / #FF00FF80', '不匹配 #ggg'] },
    { pattern: 'HTML 标签', category: '模板', desc: '匹配某标签的内容', example: ['"<div[^>]*>([\\s\\S]*?)</div>"', '非贪婪匹配最内层，或用 (?s) 开启 dotall'] },
    { pattern: 'Markdown 链接', category: '模板', desc: '[text](url)', example: ['"\\[([^\\]]+)\\]\\(([^)]+)\\)"', '提取 text 和 url'] },
    { pattern: '密码强度（>=8位含大小写数字）', category: '模板', desc: 'Lookahead 组合', example: ['"^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)[A-Za-z\\d@#$%]{8,}$"', '最少8位，必须有大写+小写+数字'] },
    { pattern: '版本号 SemVer', category: '模板', desc: 'v1.2.3-beta 格式', example: ['"^v?\\d+\\.\\d+\\.\\d+(?:-[\\w.-]+)?(?:\\+[\\w.-]+)?$"', '匹配 v1.0.0 / 2.3.4-beta.1'] },
    // === 常见用法 ===
    { pattern: 'global / multiline / dotall', category: '模式', desc: 'JS 正则修饰符 g/m/s (dotall=.*也匹配换行)', example: ['"/pattern/g" 全局（找所有）', '"/pattern/m" 多行（^/$ 每行）', '"/pattern/s" dotall（. 也匹配换行）'] },
    { pattern: 'PCRE / Java 通用 flags', category: '模式', desc: 'i 忽略大小写; x 忽略空白(允许注释); u Unicode', example: ['(?i) 内联修饰符 (PCRE/Java)', '(?i)pattern 从这里开始忽略大小写'] },
    { pattern: 'String.replace() 回调', category: '用法', desc: 'JS/Python replace 函数可做高级替换', example: ['str.replace(/(\\w+)\\s+(\\d+)/g, "$2 $1")  交换顺序', 'str.replace(/"([^"]+)"/g, (m,p1) => `\'${p1}\'`)'] },
    { pattern: '贪婪/非贪婪选择', category: '用法', desc: '默认贪婪（匹配尽可能多），? 变非贪婪', example: ['"<.+>" 在 "<a><b>" 匹配 "<a><b>" (贪婪)', '"<.+?>" 在 "<a><b>" 匹配 "<a>" (非贪婪，通常才对)'] },
    // === 陷阱 ===
    { pattern: 'DOTALL / \\s\\S', category: '陷阱', desc: '. 默认不匹配换行，跨多行要用 [\\s\\S] 或 (?s)', example: ['"a.*b" 跨多行匹配失败', '"a[\\s\\S]*b" 或 "(?s)a.*b" 才能跨多行'] },
    { pattern: 'HTML 不要用正则解析', category: '陷阱', desc: '正则不适合解析嵌套结构，用 DOM 解析器', example: ['"<(div|span)>.*?</\\1>" 简单场景可以', '复杂嵌套（div里有div）容易挂，用 cheerio/soup'] },
    { pattern: '量词膨胀（Catastrophic Backtracking）', category: '陷阱', desc: '嵌套量词 + 最后失败会指数级回溯（卡死）', example: ['"(a+)+b" 对长 a 结尾没有 b 的字符串会死循环', '改成 "a+b" 或原子群 (?>a+)+b'] },
  ],
}
