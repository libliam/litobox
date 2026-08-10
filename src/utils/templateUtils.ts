import Handlebars from 'handlebars'

/** 模板渲染结果 */
export interface TemplateRenderResult {
  ok: boolean
  output?: string
  templateError?: string // 模板编译/执行错误
  dataError?: string // 数据 JSON 解析错误
}

/**
 * 渲染 Handlebars 模板
 * - 数据允许为空字符串（视为空对象）
 * - 数据 JSON 解析失败时 dataError 提示，但模板仍可用空对象渲染
 */
export function renderTemplate(templateText: string, dataText: string): TemplateRenderResult {
  let data: any = {}
  if (dataText && dataText.trim()) {
    try {
      data = JSON.parse(dataText)
    } catch (e: any) {
      // 数据解析失败不阻断，用空对象渲染，同时提示数据错误
      return {
        ok: false,
        dataError: '数据 JSON 解析失败: ' + e.message,
      }
    }
  }
  try {
    const template = Handlebars.compile(templateText)
    const output = template(data)
    return { ok: true, output }
  } catch (e: any) {
    return { ok: false, templateError: e.message || String(e) }
  }
}

/** 内置示例模板 */
export const EXAMPLE_TEMPLATE = `<!DOCTYPE html>
<html>
<body>
  <h1>用户列表 - {{title}}</h1>
  {{#if showHeader}}
    <p>共 {{users.length}} 位用户</p>
  {{/if}}
  <ul>
    {{#each users}}
      <li>
        <strong>{{name}}</strong>（{{age}} 岁）
        {{#if isVip}}
          <span style="color:#e67e22">★ VIP</span>
        {{else}}
          <span>普通用户</span>
        {{/if}}
      </li>
    {{else}}
      <li>暂无用户</li>
    {{/each}}
  </ul>
  <footer>{{copyright}} © {{year}}</footer>
</body>
</html>`

/** 内置示例数据 */
export const EXAMPLE_DATA = `{
  "title": "团队成员",
  "showHeader": true,
  "year": 2026,
  "copyright": "LitoBox",
  "users": [
    { "name": "张三", "age": 25, "isVip": true },
    { "name": "李四", "age": 30, "isVip": false },
    { "name": "王五", "age": 28, "isVip": true }
  ]
}`

/** 快捷插入片段（name: 按钮标签, insert: 插入的模板代码） */
export const SNIPPETS: Array<{ name: string; insert: string }> = [
  { name: '变量', insert: '{{placeholder}}' },
  { name: '#each', insert: '{{#each items}}\n  {{this}}\n{{/each}}' },
  { name: '#if', insert: '{{#if condition}}\n  {{value}}\n{{/if}}' },
  { name: '#unless', insert: '{{#unless condition}}\n  {{value}}\n{{/unless}}' },
  { name: '#with', insert: '{{#with context}}\n  {{this}}\n{{/with}}' },
  { name: '{{else}}', insert: '{{else}}' },
  { name: '注释', insert: '{{!-- 注释内容 --}}' },
  { name: 'safe', insert: '{{{rawHtml}}}' },
]

/** 自检函数：验证模板渲染核心逻辑 */
export function selfCheck(): string[] {
  const errors: string[] = []
  try {
    // 基本变量渲染
    const r1 = renderTemplate('Hello {{name}}', '{"name": "张三"}')
    if (!r1.ok || r1.output !== 'Hello 张三') errors.push('变量渲染失败: ' + r1.output)
    // each 循环
    const r2 = renderTemplate('{{#each list}}[{{this}}]{{/each}}', '{"list": [1, 2, 3]}')
    if (!r2.ok || r2.output !== '[1][2][3]') errors.push('each 渲染失败: ' + r2.output)
    // if/else 分支
    const r3 = renderTemplate('{{#if ok}}Y{{else}}N{{/if}}', '{"ok": false}')
    if (!r3.ok || r3.output !== 'N') errors.push('if 渲染失败: ' + r3.output)
    // HTML 转义（默认 {{ }} 转义 < >）
    const r4 = renderTemplate('{{html}}', '{"html": "<b>x</b>"}')
    if (!r4.ok || r4.output !== '&lt;b&gt;x&lt;/b&gt;') errors.push('HTML 转义失败: ' + r4.output)
    // 模板语法错误应捕获
    const r5 = renderTemplate('{{#if x}}', '{}')
    if (r5.ok || !r5.templateError) errors.push('模板语法错误未捕获')
    // 数据解析错误应捕获
    const r6 = renderTemplate('x', '{bad json')
    if (r6.ok || !r6.dataError) errors.push('数据解析错误未捕获')
    // 空数据渲染
    const r7 = renderTemplate('ok-{{a}}', '')
    if (!r7.ok || r7.output !== 'ok-') errors.push('空数据渲染失败')
  } catch (e: any) {
    errors.push('自检异常: ' + e.message)
  }
  return errors
}
