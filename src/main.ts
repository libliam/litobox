import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import App from './App.vue'
import { computeTabIndent } from './utils/tabIndent'
import './style/main.css'
import './style/theme.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(ElementPlus)
app.mount('#app')

// 全局接管多行文本域的 Tab 键：插入缩进而非切换焦点跳出文本区。
// 单行 input 保持原生 Tab 焦点导航；CodeMirror 编辑器由其 smartTab keymap 自行处理。
// ponytail: 缩进固定 2 空格，无用户配置；需要可配置时把 TAB_INDENT 挪到设置页。
document.addEventListener('keydown', (e: KeyboardEvent) => {
  if (e.key !== 'Tab' || e.isComposing) return
  const el = e.target
  if (!(el instanceof HTMLTextAreaElement) || el.readOnly || el.disabled) return
  e.preventDefault()

  const edit = computeTabIndent(el.value, el.selectionStart ?? 0, el.selectionEnd ?? 0, e.shiftKey)

  // execCommand 走原生输入栈：可撤销（Ctrl+Z 单步回退）且自动触发 input 事件，Vue v-model 可感知
  el.setSelectionRange(edit.start, edit.end)
  if (!document.execCommand('insertText', false, edit.text)) {
    el.setRangeText(edit.text, edit.start, edit.end, 'select')
    el.dispatchEvent(new Event('input', { bubbles: true }))
  }
  el.setSelectionRange(edit.selStart, edit.selEnd)
})

// 启动期间主窗口保持隐藏（tauri.conf visible:false），界面就绪后再显示，消除启动白屏。
// 非 Tauri 环境（纯浏览器访问 dev url）invoke 失败，忽略即可。
invoke('app_ready').catch(() => {})