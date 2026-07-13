// ponytail: 简单 assert 自检，不引入测试框架。开发时运行 npx tsx src/store/tabs.selfcheck.ts 验证
import { setActivePinia, createPinia } from 'pinia'
import { useToolboxStore } from './index'

function assert(cond: boolean, msg: string) {
  if (!cond) { console.error('❌ FAIL:', msg); process.exit(1) }
  else console.log('✅ PASS:', msg)
}

setActivePinia(createPinia())
const store = useToolboxStore()

assert(store.tabs.length === 1, '初始只有 home tab')
assert(store.tabs[0].toolId === 'home', '初始 tab 是 home')
assert(store.activeTabId === 'home', '初始 activeTabId 是 home')

store.openTab('json')
assert(store.tabs.length === 2, 'openTab 新工具追加')
assert(store.activeTabId === 'json', 'openTab 激活新 tab')

store.openTab('time')
store.openTab('json')
assert(store.tabs.length === 3, 'openTab 已存在工具不重复添加')
assert(store.activeTabId === 'json', 'openTab 已存在工具仅激活')

store.closeTab('time')
assert(store.activeTabId === 'json', 'closeTab 非当前不影响 active')
assert(store.tabs.length === 2, 'closeTab 减少数量')

store.closeTab('json')
assert(store.activeTabId === 'home', 'closeTab 当前 tab 激活相邻')

store.closeTab('home')
assert(store.tabs.find(t => t.toolId === 'home') !== undefined, 'home 不可关闭')

// LRU
const s2 = useToolboxStore()
;(s2 as any).tabs = [{ toolId: 'home' }]
;(s2 as any).activeTabId = 'home'
for (let i = 1; i <= 8; i++) s2.openTab(`t${i}` as any)
assert(s2.tabs.length === 8, 'LRU 限制 8 个')
assert(s2.tabs.find((t: any) => t.toolId === 't1') === undefined, 'LRU 淘汰最早非 home')
assert(s2.tabs[0].toolId === 'home', 'LRU 保留 home')

console.log('\n全部通过')
