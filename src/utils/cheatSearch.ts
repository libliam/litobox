// 全局搜索（首页搜索框 / 命令面板）的统一数据源：
// 除工具本身外，把多 tab 工具的每个子 tab 也作为可搜索子项，
// 命中后跳转到对应工具并切到指定 tab。子 tab 粒度 = 独立子功能（不搜具体内容）。
import type { ToolItem } from '@/store'
import { filterTools } from './commandPalette'

export interface SearchEntry {
  /** 列表唯一 key：工具为 tool.id；子 tab 为 tab:<toolId>:<tabId> */
  id: string
  /** 跳转目标工具 id */
  toolId: string
  name: string
  description: string
  icon?: string
  iconSvg?: string
  category: string
  /** 子 tab 专属：跳转后切到的工具与 tab */
  tab?: { toolId: string; tabId: string }
  score: number
}

/** 子 tab 对搜索词的评分：label 精确 > 前缀 > 包含；工具名/描述命中次之 */
export function scoreToolTab(rawQ: string, tool: ToolItem, tabLabel: string): number {
  const q = rawQ.toLowerCase()
  const label = tabLabel.toLowerCase()
  let score = 0
  if (label === q) score += 100
  else if (label.startsWith(q)) score += 50
  else if (label.includes(q)) score += 25
  if ((tool.name + ' ' + (tool.description || '')).toLowerCase().includes(q)) score += 10
  return score
}

/** 对所有工具的子 tab 做检索（纯函数，便于自检） */
export function matchToolTabEntries(
  rawQ: string,
  tools: ToolItem[],
): Array<Pick<SearchEntry, 'id' | 'name' | 'description' | 'tab' | 'score'>> {
  const q = rawQ.trim().toLowerCase()
  if (!q) return []
  const out: ReturnType<typeof matchToolTabEntries> = []
  for (const tool of tools) {
    if (!tool.tabs?.length) continue
    for (const tab of tool.tabs) {
      const score = scoreToolTab(q, tool, tab.label)
      if (score <= 0) continue
      out.push({
        id: `tab:${tool.id}:${tab.id}`,
        name: tab.label,
        description: tool.name,
        tab: { toolId: tool.id, tabId: tab.id },
        score,
      })
    }
  }
  out.sort((a, b) => b.score - a.score)
  return out
}

const toolToEntry = (tool: ToolItem, score: number): SearchEntry => ({
  id: tool.id,
  toolId: tool.id,
  name: tool.name,
  description: tool.description,
  icon: tool.icon,
  iconSvg: tool.iconSvg,
  category: tool.category || '其他',
  score,
})

/**
 * 全局搜索：工具结果 + 子 tab 结果合并排序。
 * 空词时仅返回全部工具；有词时子 tab 按名称/工具描述匹配，
 * 且每个有子 tab 命中的工具，若工具入口本身未命中，给一个兜底低分入口。
 */
export function searchEntries(query: string, tools: ToolItem[]): SearchEntry[] {
  const q = query.trim().toLowerCase()
  if (!q) return tools.map(tool => toolToEntry(tool, 0))

  const entries = filterTools(query, tools).map(r => toolToEntry(r.tool, r.score))
  const toolMap = new Map(tools.map(t => [t.id, t]))

  const tabRows = matchToolTabEntries(q, tools)
  // 按工具分组，给每个命中工具补兜底入口
  const hitTools = new Set<string>()
  for (const r of tabRows) hitTools.add(r.tab!.toolId)
  for (const toolId of hitTools) {
    if (!entries.some(e => e.id === toolId)) {
      const tool = toolMap.get(toolId)
      if (tool) entries.push(toolToEntry(tool, 12))
    }
  }
  for (const r of tabRows) {
    const tool = toolMap.get(r.tab!.toolId)
    entries.push({
      ...r,
      toolId: r.tab!.toolId,
      icon: tool?.icon,
      iconSvg: tool?.iconSvg,
      category: tool?.category || '其他',
    })
  }

  entries.sort((a, b) => b.score - a.score)
  return entries
}
