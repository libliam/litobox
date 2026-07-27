import type { ToolItem } from '@/store'

export interface RankedTool {
  tool: ToolItem
  score: number
}

/**
 * 模糊搜索工具：按 name/keywords/description/id 子串匹配，加权排序。
 * @param query 搜索词（空则返回全部，score=0）
 * @param tools 工具列表（由调用方传入，保持函数纯净无外部依赖）
 */
export function filterTools(query: string, tools: ToolItem[]): RankedTool[] {
  const q = query.trim().toLowerCase()
  if (!q) {
    return tools.map(tool => ({ tool, score: 0 }))
  }

  const matched: RankedTool[] = []
  for (const tool of tools) {
    const name = tool.name.toLowerCase()
    const id = tool.id.toLowerCase()
    const desc = tool.description.toLowerCase()
    const keywords = (tool.keywords || []).map(k => k.toLowerCase())

    let score = 0
    if (name === q) score += 100
    else if (name.startsWith(q)) score += 50
    else if (name.includes(q)) score += 5

    if (keywords.some(k => k.includes(q))) score += 30
    if (desc.includes(q)) score += 20
    if (id.includes(q)) score += 10

    if (score > 0) matched.push({ tool, score })
  }

  matched.sort((a, b) => b.score - a.score)
  return matched
}
