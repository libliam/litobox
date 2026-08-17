// src/utils/codeFormatterUtils.ts
// 基于 prettier standalone 的多语言代码格式化，全部插件懒加载（首次使用时才下载对应 chunk）

export interface FormatCodeOptions {
  tabWidth?: 2 | 4
  semi?: boolean
  singleQuote?: boolean
  printWidth?: number
}

export interface FormatCodeResult {
  success: boolean
  data?: string
  error?: string
}

export interface LanguageConfig {
  id: string
  label: string
  parser: string
  plugins: string[]
  sample: string
}

// 支持的语言（parser 与插件映射见 prettier standalone 文档）
export const LANGUAGE_LIST: LanguageConfig[] = [
  {
    id: 'javascript',
    label: 'JavaScript / JSX',
    parser: 'babel',
    plugins: ['babel', 'estree'],
    sample: `const users=[{name:"Alice",age:30},{name:"Bob",age:25}];\nfunction greet(u){return\`Hello \${u.name}!\`;}\nusers.map(u=>console.log(greet(u)));`,
  },
  {
    id: 'typescript',
    label: 'TypeScript / TSX',
    parser: 'typescript',
    plugins: ['babel', 'estree'],
    sample: `interface User{name:string;age:number;}\nconst users:User[]=[{name:"Alice",age:30}];\nfunction greet(u:User):string{return\`Hello \${u.name}!\`;}`,
  },
  {
    id: 'json',
    label: 'JSON',
    parser: 'json',
    plugins: ['babel', 'estree'],
    sample: `{"name":"litobox","version":"7.13.0","tools":["json","sql","encode"]}`,
  },
  {
    id: 'json5',
    label: 'JSON5 / JSONC（支持注释/尾逗号）',
    parser: 'json5',
    plugins: ['babel', 'estree'],
    sample: `{// 开发配置\nname:"litobox",tools:["json","sql",],/* 注释保留 */}`,
  },
  {
    id: 'css',
    label: 'CSS',
    parser: 'css',
    plugins: ['postcss'],
    sample: `.card{border:1px solid #ddd;border-radius:8px;padding:16px;}\n.card:hover{border-color:#00d4ff;}`,
  },
  {
    id: 'scss',
    label: 'SCSS / Less',
    parser: 'scss',
    plugins: ['postcss'],
    sample: `$primary:#00d4ff;\n.card{color:$primary;&:hover{color:darken($primary,10%);}}`,
  },
  {
    id: 'html',
    label: 'HTML',
    parser: 'html',
    plugins: ['html'],
    sample: `<div class="card"><h2>标题</h2><p>内容</p><button>按钮</button></div>`,
  },
  {
    id: 'vue',
    label: 'Vue SFC',
    parser: 'vue',
    plugins: ['html'],
    sample: `<template><div class="app"><h1>{{title}}</h1></div></template>\n<script setup>\nconst title="Hello";\n</script>\n<style scoped>\n.app{color:#00d4ff;}\n</style>`,
  },
  {
    id: 'markdown',
    label: 'Markdown',
    parser: 'markdown',
    plugins: ['markdown'],
    sample: `# 标题\n\n## 二级\n- 列表项1\n- 列表项2\n\n> 引用\n`,
  },
  {
    id: 'yaml',
    label: 'YAML',
    parser: 'yaml',
    plugins: ['yaml'],
    sample: `name: litobox\nversion: "7.13.0"\ntools:\n  - json\n  - sql\nconfig:\n  theme: auto\n  hotkey: Ctrl+Alt+T\n`,
  },
]

// 插件懒加载映射（chunk 按需加载，避免拖慢启动）
const PLUGIN_LOADERS: Record<string, () => Promise<any>> = {
  babel: () => import('prettier/plugins/babel'),
  estree: () => import('prettier/plugins/estree'),
  postcss: () => import('prettier/plugins/postcss'),
  html: () => import('prettier/plugins/html'),
  markdown: () => import('prettier/plugins/markdown'),
  yaml: () => import('prettier/plugins/yaml'),
}

let standalonePromise: Promise<any> | null = null

function loadStandalone(): Promise<any> {
  if (!standalonePromise) {
    standalonePromise = import('prettier/standalone')
  }
  return standalonePromise
}

/** 按语言 id 获取配置 */
export function getLanguageConfig(languageId: string): LanguageConfig | undefined {
  return LANGUAGE_LIST.find((l) => l.id === languageId)
}

/**
 * 格式化代码
 * @param code 源码
 * @param languageId 语言 id（LANGUAGE_LIST）
 * @param options 格式化选项（未传则用 prettier 默认）
 */
export async function formatCode(
  code: string,
  languageId: string,
  options: FormatCodeOptions = {}
): Promise<FormatCodeResult> {
  try {
    const conf = getLanguageConfig(languageId)
    if (!conf) return { success: false, error: `不支持的语言: ${languageId}` }
    if (!code.trim()) return { success: false, error: '代码不能为空' }

    const [standalone, ...pluginModules] = await Promise.all([
      loadStandalone(),
      ...conf.plugins.map((p) => PLUGIN_LOADERS[p]()),
    ])

    const formatted: string = await standalone.format(code, {
      parser: conf.parser,
      plugins: pluginModules,
      ...(options.tabWidth !== undefined ? { tabWidth: options.tabWidth } : {}),
      ...(options.semi !== undefined ? { semi: options.semi } : {}),
      ...(options.singleQuote !== undefined ? { singleQuote: options.singleQuote } : {}),
      ...(options.printWidth !== undefined ? { printWidth: options.printWidth } : {}),
    })
    return { success: true, data: formatted }
  } catch (error: any) {
    // prettier 解析错误信息较长，提取核心部分
    let message = error instanceof Error ? error.message : String(error)
    if (message.length > 500) message = message.slice(0, 500) + '...'
    return { success: false, error: message }
  }
}

/**
 * 自检：在 dev console 运行 codeFormatterUtils.selfCheck() 验证格式化管线可用
 */
export async function selfCheck(): Promise<void> {
  const cases: Array<[string, string, (s: string) => boolean]> = [
    ['json', `{"a":1,"b":[1,2]}`, (s) => s.includes('\n') && s.includes('"a"')],
    ['javascript', `const a=1;function f(){return a;}`, (s) => s.includes('\n') && s.includes('const a = 1')],
    ['css', `.a{color:red;margin:0;}`, (s) => s.includes('\n') && s.includes('color: red')],
  ]
  for (const [lang, code, check] of cases) {
    const r = await formatCode(code, lang, { tabWidth: 2 })
    if (!r.success) throw new Error(`[${lang}] 格式化失败: ${r.error}`)
    if (!r.data || !check(r.data)) throw new Error(`[${lang}] 格式化结果异常: ${r.data}`)
    console.log(`[codeFormatter] ${lang} ✓\n${r.data}`)
  }
  console.log('[codeFormatter] selfCheck passed ✓')
}
