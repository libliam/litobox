// 变量命名风格转换：任意命名风格/短语 → 7 种命名风格统一转换
// ponytail: 自研分词而非复用 stringUtils —— 现有实现按分隔符/大写简单切分，
// 无法处理缩写词边界（如 HTTPServer → HTTP + Server、userID → user + ID），
// 批量迁移存量代码命名时这是核心诉求；如需更多风格在此扩展 NAME_STYLE_META 即可。

export type NameStyle = 'camel' | 'pascal' | 'snake' | 'scream' | 'kebab' | 'dot' | 'title'

export interface NameStyleMeta {
  key: NameStyle
  label: string
  desc: string
}

/** 风格定义：顺序即页面列顺序（title 常用于展示文案，放最后） */
export const NAME_STYLE_META: NameStyleMeta[] = [
  { key: 'camel', label: 'camelCase', desc: '小驼峰' },
  { key: 'pascal', label: 'PascalCase', desc: '大驼峰' },
  { key: 'snake', label: 'snake_case', desc: '下划线' },
  { key: 'scream', label: 'SCREAMING_SNAKE', desc: '常量命名' },
  { key: 'kebab', label: 'kebab-case', desc: '连字符' },
  { key: 'dot', label: 'dot.case', desc: '点分隔' },
  { key: 'title', label: 'Title Case', desc: '首字母大写' },
]

export type NameCaseResult = Record<NameStyle, string>

const capFirst = (w: string): string => (w ? w.charAt(0).toUpperCase() + w.slice(1) : w)

/**
 * 把任意来源的标识符/短语拆成小写单词数组：
 * - 非字母数字（含中文按词保留）统一当分隔符
 * - 手工补大小写边界：小写/数字/中文 后紧跟大写 → 新词；连续大写后跟“大写+小写” → 缩写词结束
 *   例：userID → [user, id]；HTTPServer → [http, server]；XMLHttpRequest → [xml, http, request]
 */
export function splitWords(name: string): string[] {
  let s = name.replace(/[^a-zA-Z0-9\u4e00-\u9fff]+/g, ' ')
  s = s.replace(/([a-z0-9\u4e00-\u9fff])([A-Z])/g, '$1 $2')
  s = s.replace(/([A-Z])([\u4e00-\u9fff])/g, '$1 $2')
  s = s.replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
  return s.split(' ').map(w => w.toLowerCase()).filter(Boolean)
}

/** 把单个标识符/短语转换为全部 7 种风格 */
export function convertName(name: string): NameCaseResult {
  const words = splitWords(name)
  const pascal = words.map(capFirst)
  const snake = words.join('_')
  return {
    camel: words.length ? pascal[0].toLowerCase() + pascal.slice(1).join('') : '',
    pascal: pascal.join(''),
    snake,
    scream: snake.toUpperCase(),
    kebab: words.join('-'),
    dot: words.join('.'),
    title: pascal.join(' '),
  }
}

export interface NameCaseRow {
  /** 原始输入（trim 后） */
  source: string
  results: NameCaseResult
}

/** 批量转换：每行一个标识符/短语，跳过空行 */
export function convertNameText(text: string): NameCaseRow[] {
  const rows: NameCaseRow[] = []
  for (const line of text.split(/\r?\n/)) {
    const source = line.trim()
    if (!source) continue
    rows.push({ source, results: convertName(source) })
  }
  return rows
}
