let renderSeq = 0

/** mermaid 内置主题选项（auto = 跟随应用主题） */
export const MERMAID_THEME_OPTIONS: { label: string; value: string }[] = [
  { label: '跟随应用', value: 'auto' },
  { label: '经典', value: 'default' },
  { label: '森林', value: 'forest' },
  { label: '中性', value: 'neutral' },
  { label: '新潮', value: 'neo' },
  { label: '新潮暗色', value: 'neo-dark' },
  { label: '暗色', value: 'dark' },
  { label: '基础', value: 'base' },
  { label: 'Redux', value: 'redux' },
  { label: 'Redux暗色', value: 'redux-dark' },
  { label: 'Redux彩色', value: 'redux-color' },
  { label: 'Redux彩色暗色', value: 'redux-dark-color' },
]

/** mermaid 合法主题集合（含所有内置主题） */
export type MermaidThemeName =
  | 'default'
  | 'forest'
  | 'neutral'
  | 'neo'
  | 'neo-dark'
  | 'dark'
  | 'base'
  | 'redux'
  | 'redux-dark'
  | 'redux-color'
  | 'redux-dark-color'

/**
 * 导出用字体栈：mermaid 布局测量与图片栅格化共用同一字体。
 * 若用 inherit，SVG 独立加载（img/查看器）时继承 UA 默认字体，
 * 与布局测量时的页面字体宽度不同，导致文字溢出节点框（导出 PNG 文字错位）。
 */
export const MERMAID_EXPORT_FONT = '"Microsoft YaHei", "PingFang SC", "Noto Sans SC", sans-serif'

/** 解析主题：auto 时按应用当前主题取 dark / default */
export function resolveMermaidTheme(theme: string): MermaidThemeName {
  const valid: MermaidThemeName[] = [
    'default',
    'forest',
    'neutral',
    'neo',
    'neo-dark',
    'dark',
    'base',
    'redux',
    'redux-dark',
    'redux-color',
    'redux-dark-color',
  ]
  if ((valid as string[]).includes(theme)) return theme as MermaidThemeName
  return document.documentElement.classList.contains('light') ? 'default' : 'dark'
}

/**
 * 渲染 mermaid 代码为 SVG 字符串
 * @param code mermaid 源码
 * @param opts.theme 主题（auto/default/forest/neutral/dark），默认 auto 跟随应用
 * @param opts.fontSize 正文字号（px），默认 16
 * @param opts.htmlLabels 标签渲染方式，默认 true（HTML 标签，样式更丰富）；
 *   false 时用纯 SVG 文本（无 foreignObject）。mermaid 是单例，必须始终显式设置，
 *   避免一次渲染后配置残留影响后续渲染
 * @param opts.fontFamily SVG 字体，默认 'inherit'；导出用 MERMAID_EXPORT_FONT 保证栅格化与布局字体一致
 * @returns SVG 字符串
 * @throws 渲染失败时抛出包含解析错误的异常
 */
export async function renderMermaid(code: string, opts: { theme?: string; fontSize?: number; htmlLabels?: boolean; fontFamily?: string } = {}): Promise<string> {
  // 懒加载 mermaid，避免进入主包（首屏按需加载，与 ocr/prettier 模式一致）
  const mermaid = (await import('mermaid')).default
  mermaid.initialize({
    startOnLoad: false,
    theme: resolveMermaidTheme(opts.theme || 'auto'),
    fontFamily: opts.fontFamily || 'inherit',
    themeVariables: { fontSize: opts.fontSize || 16 },
    htmlLabels: opts.htmlLabels ?? true,
    // 保持默认 strict，避免 HTML 标签注入（本地工具输入可信，仍以安全为先）
  })
  const id = `mmd-${Date.now()}-${++renderSeq}`
  const { svg } = await mermaid.render(id, code)
  return svg
}

/**
 * 修复 mermaid 导出 SVG 的两个已知缺陷（htmlLabels: false 渲染路径）：
 * 1. 根 <svg> 是 width="100%" 且无 height，作为 <img> 栅格化时固有尺寸退化为 85×150，
 *    导致 PNG 内容裁剪错位；改为 viewBox 的实际像素尺寸。
 * 2. mindmap 根节点的文本框以形状中心为锚点，缺 text-anchor 时从中心向右展开造成
 *    文字水平偏右（子节点/其余图表类型的框 mermaid 已按 start 布局对称包裹文字，不受影响）。
 *    给 mindmap 根节点（section-root）的 text 补 text-anchor="middle" 使其居中。
 */
export function fixExportSvg(svg: string): string {
  const doc = new DOMParser().parseFromString(svg, 'image/svg+xml')
  const root = doc.documentElement
  // 1. 固有尺寸：以 viewBox 实际像素尺寸替换 width="100%" / 缺失 height
  const vb = root.getAttribute('viewBox')
  if (vb) {
    const parts = vb.split(/[\s,]+/).map(Number)
    if (parts.length === 4 && parts[2] > 0 && parts[3] > 0) {
      root.setAttribute('width', String(parts[2]))
      root.setAttribute('height', String(parts[3]))
    }
  }
  // 2. mindmap 根节点文字居中
  root.querySelectorAll('g.label text').forEach((t) => {
    const nodeG = t.closest('g.label')?.parentElement
    if (!nodeG) return
    const gcls = nodeG.getAttribute('class') || ''
    // 仅 mindmap 根节点（section-root）：无论 circle/rect 形状，文本框以形状中心为锚点；
    // 子节点 mermaid 已按 start 布局对称包裹文字，不能加 middle（会偏左半文字宽）
    if (!gcls.includes('mindmap-node') || !gcls.includes('section-root')) return
    // 已有 text-anchor（属性或 style 内联）的不动
    if (t.getAttribute('text-anchor')) return
    if ((t.getAttribute('style') || '').includes('text-anchor')) return
    t.setAttribute('text-anchor', 'middle')
  })
  return new XMLSerializer().serializeToString(root)
}

/**
 * 自检：验证 fixExportSvg 的核心规则
 * - 根 svg 缺失 width/height 时按 viewBox 补齐
 * - mindmap 根节点补 text-anchor="middle"
 * - mindmap 子节点保持无 text-anchor（mermaid 已按 start 布局，加 middle 会偏左）
 * dev console 运行 mermaidUtils.selfCheck() 即可验证
 */
export function selfCheck(): string[] {
  const errors: string[] = []
  const svg = (body: string, rootAttr = 'width="100%"') =>
    `<svg ${rootAttr} viewBox="0 0 200 100">${body}</svg>`
  const rootNode = '<g class="label"><g><text>根节点</text></g></g>'
  const childNode = '<g class="label"><g><text>子节点</text></g></g>'
  // 1. 尺寸补齐
  const fixed = fixExportSvg(svg(`<g class="node mindmap-node section-root section--1">${rootNode}</g><g class="node mindmap-node section-0">${childNode}</g>`))
  if (!/<svg[^>]*width="200"[^>]*height="100"/.test(fixed)) errors.push('根 svg 未按 viewBox 补齐 width/height')
  // 2. mindmap 根节点居中
  const rootText = fixed.match(/<text[^>]*>根节点<\/text>/)
  if (rootText && !rootText[0].includes('text-anchor="middle"')) errors.push('mindmap 根节点未补 text-anchor="middle"')
  // 3. 子节点保持 start（不误伤）
  const childText = fixed.match(/<text[^>]*>子节点<\/text>/)
  if (childText && childText[0].includes('text-anchor')) errors.push('mindmap 子节点被误加 text-anchor')
  return errors
}

/** 常用图表类型模板（classDef 美化配色，深浅主题下均清晰） */
export const MERMAID_TEMPLATES: { label: string; value: string; code: string }[] = [
  {
    label: '流程图',
    value: 'flowchart',
    code: `flowchart TD
    classDef start fill:#10b981,stroke:#059669,stroke-width:2px,color:#fff;
    classDef decide fill:#f59e0b,stroke:#d97706,color:#fff;
    classDef action fill:#00d4ff,stroke:#0891b2,color:#04283a;
    classDef endp fill:#64748b,stroke:#475569,color:#fff;
    A(开始):::start --> B{判断条件}:::decide
    B -- 是 --> C[执行操作]:::action
    B -- 否 --> D[结束]:::endp
    C --> D`,
  },
  {
    label: '时序图',
    value: 'sequenceDiagram',
    code: `sequenceDiagram
    participant 用户
    participant 前端
    participant 后端
    用户->>+前端: 发起请求
    前端->>+后端: 转发请求
    后端-->>-前端: 返回响应
    前端-->>-用户: 展示结果`,
  },
  {
    label: '类图',
    value: 'classDiagram',
    code: `classDiagram
    class Animal {
        +String name
        +int age
        +makeSound() String
    }
    class Dog {
        +fetch() void
    }
    class Cat {
        +purr() void
    }
    Animal <|-- Dog
    Animal <|-- Cat`,
  },
  {
    label: '状态图',
    value: 'stateDiagram',
    code: `stateDiagram-v2
    classDef done fill:#10b981,color:#fff;
    classDef active fill:#f59e0b,color:#fff;
    [*] --> 待机
    待机 --> 运行 : 启动
    运行 --> 暂停 : 暂停
    暂停 --> 运行 : 恢复
    运行 --> 停止 : 停止
    停止:::done --> [*]`,
  },
  {
    label: 'ER 图',
    value: 'erDiagram',
    code: `erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ ORDER_ITEM : contains
    PRODUCT ||--o{ ORDER_ITEM : "ordered in"`,
  },
  {
    label: '甘特图',
    value: 'gantt',
    code: `gantt
    title 项目开发计划
    dateFormat YYYY-MM-DD
    section 需求
    需求分析     :done, a1, 2026-08-01, 3d
    需求评审     :done, a2, after a1, 2d
    section 开发
    前端开发     :active, b1, after a2, 5d
    后端开发     :b2, after a2, 5d
    section 测试
    功能测试     :c1, after b1, 3d
    发布上线     :milestone, after c1, 0d`,
  },
  {
    label: '饼图',
    value: 'pie',
    code: `pie title 预算分配
    "开发" : 50
    "测试" : 30
    "运维" : 20`,
  },
  {
    label: '思维导图',
    value: 'mindmap',
    code: `mindmap
  root((栗的百宝箱))
    开发工具
      JSON 工具
      正则测试
      编码转换
    系统工具
      进程管理
      网络信息
    实用工具
      压缩解压
      Mermaid 图表`,
  },
  {
    label: '用户旅程',
    value: 'journey',
    code: `journey
    title 用户操作旅程
    section 使用工具
      打开应用: 5: 用户
      输入内容: 4: 用户
      查看结果: 5: 用户`,
  },
  {
    label: 'Git 图',
    value: 'gitGraph',
    code: `gitGraph
    commit id: "初始"
    branch feature
    checkout feature
    commit id: "开发功能"
    checkout main
    merge feature`,
  },
]
