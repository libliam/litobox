<template>
  <div ref="wrapperRef" class="codemirror-wrapper">
    <div ref="editorRef" class="codemirror-internal"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, shallowRef, nextTick } from 'vue'
import { EditorState, Compartment } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { search, searchKeymap, highlightSelectionMatches, openSearchPanel } from '@codemirror/search'
import { bracketMatching, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
import { dracula } from '@uiw/codemirror-theme-dracula'
import { monokai } from '@uiw/codemirror-theme-monokai'
import { vscodeDark, vscodeLight } from '@uiw/codemirror-theme-vscode'
import { githubDark, githubLight } from '@uiw/codemirror-theme-github'
import { sublime } from '@uiw/codemirror-theme-sublime'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { xml } from '@codemirror/lang-xml'
import { sql } from '@codemirror/lang-sql'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { yaml } from '@codemirror/lang-yaml'
import { shell } from '@codincod/codemirror-lang-shell'
import { useToolboxStore } from '@/store'

const props = defineProps<{
  modelValue: string
  language?: string
  readOnly?: boolean
  initialWrap?: boolean  // 初始自动换行状态（多 tab 切换时保留各自状态）
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': [value: string]
  // ponytail: 底部状态栏需要的状态信息
  'status': [info: EditorStatus]
  'goto-line': []  // Ctrl+G 请求跳转行
  'save': []  // Ctrl+S 请求保存
}>()

export interface EditorStatus {
  line: number
  column: number
  selectedCount: number  // 选中字符数（0 表示未选中）
  selectedLines: number  // 选中行数（含部分行）
  totalLines: number
  totalChars: number
  lineEnding: 'CRLF' | 'LF' | 'CR'
}

// 根据文档推断换行符类型
const detectLineEnding = (doc: string): EditorStatus['lineEnding'] => {
  if (doc.includes('\r\n')) return 'CRLF'
  if (doc.includes('\r')) return 'CR'
  return 'LF'
}

const emitStatus = (state: any) => {
  const doc = state.doc
  const sel = state.selection.main
  const pos = sel.head
  const line = doc.lineAt(pos)
  const selectedCount = sel.empty ? 0 : sel.to - sel.from
  let selectedLines = 0
  if (!sel.empty) {
    selectedLines = doc.lineAt(sel.to).number - doc.lineAt(sel.from).number + 1
  }
  emit('status', {
    line: line.number,
    column: pos - line.from + 1,
    selectedCount,
    selectedLines,
    totalLines: doc.lines,
    totalChars: doc.length,
    lineEnding: detectLineEnding(doc.toString()),
  })
}

const store = useToolboxStore()
const wrapperRef = ref<HTMLElement>()
const editorRef = ref<HTMLElement>()
const view = shallowRef<EditorView>()
let resizeObserver: ResizeObserver | null = null

// 自动换行热切换
const wrapCompartment = new Compartment()
const wordWrap = ref(!!props.initialWrap)

// 语言扩展映射
const langExtensions: Record<string, () => any> = {
  javascript: () => javascript(),
  typescript: () => javascript({ typescript: true }),
  json: () => json(),
  html: () => html(),
  css: () => css(),
  markdown: () => markdown(),
  xml: () => xml(),
  sql: () => sql(),
  python: () => python(),
  rust: () => rust(),
  yaml: () => yaml(),
  shell: () => shell(),
  bash: () => shell(),
}

// 编辑器主题名 → Extension 映射（深色 / 浅色都在表里）
const editorThemes: Record<string, () => any> = {
  oneDark: () => oneDark,
  dracula: () => dracula,
  monokai: () => monokai,
  vscodeDark: () => vscodeDark,
  vscodeLight: () => vscodeLight,
  githubDark: () => githubDark,
  githubLight: () => githubLight,
  sublime: () => sublime,
  none: () => [],
}

// 根据编辑器主题获取扩展
const getThemeExtension = () => {
  const name = store.config.editorTheme || 'oneDark'
  const fn = editorThemes[name] || editorThemes.oneDark
  return fn()
}

// 获取语言扩展
const getLangExtension = (lang: string) => {
  const ext = langExtensions[lang]
  return ext ? [ext()] : []
}

// ponytail: 统一字体和行高，确保 .cm-gutters 与 .cm-scroller 像素级对齐
// 之前依赖父容器 height:100% 但 flex 链上没有明确高度，异步组件初始化时
// 容器 auto 高度 → gutter 按内容独立算高度 → 行号和文字彻底脱节
const baseFontStyle = {
  fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
  fontSize: '14px',
  lineHeight: '1.6',
}

// 创建编辑器
const createEditor = () => {
  if (!editorRef.value || !wrapperRef.value) return

  view.value?.destroy()
  // 清理上次遗留的 DOM（destroy 不自动移除子节点）
  editorRef.value.innerHTML = ''

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      EditorView.editable.of(!props.readOnly),
      search(),
      lineNumbers(),
      highlightActiveLine(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle),
      history(),
      highlightSelectionMatches(),
      keymap.of([
        // 自定义快捷键优先于 searchKeymap（拦截 Ctrl+G 的"查找下一个"）
        { key: 'Mod-g', run: () => { emit('goto-line'); return true } },
        { key: 'Mod-s', run: () => { emit('save'); return true } },
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
      ]),
      ...getThemeExtension(),
      ...getLangExtension(props.language || 'plaintext'),
      wrapCompartment.of(wordWrap.value ? EditorView.lineWrapping : []),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const newValue = update.state.doc.toString()
          emit('update:modelValue', newValue)
          emit('change', newValue)
        }
        // 文档变化、光标移动、选中变化都要更新状态栏
        if (update.docChanged || update.selectionSet) {
          emitStatus(update.state)
        }
      }),
      EditorView.theme({
        '&': {
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          ...baseFontStyle,
        },
        '.cm-scroller': {
          flex: '1 1 auto',
          overflow: 'auto',
          ...baseFontStyle,
        },
        '.cm-content': {
          padding: '8px 12px 8px 6px',
          ...baseFontStyle,
        },
        '.cm-gutters': {
          ...baseFontStyle,
          borderRight: 'none',
        },
        '.cm-lineNumbers': {
          ...baseFontStyle,
        },
        '.cm-gutters .cm-gutterElement': {
          padding: '0 6px 0 4px',
          lineHeight: baseFontStyle.lineHeight,
          fontSize: baseFontStyle.fontSize,
          fontFamily: baseFontStyle.fontFamily,
        },
        '.cm-lineNumbers .cm-gutterElement': {
          textAlign: 'right',
          paddingRight: '6px',
        },
      }),
    ],
  })

  view.value = new EditorView({
    state,
    parent: editorRef.value,
  })
  emitStatus(view.value.state) // 初始状态
}

// 更新内容（外部修改时）
const updateContent = (newContent: string) => {
  if (view.value && newContent !== view.value.state.doc.toString()) {
    view.value.dispatch({
      changes: { from: 0, to: view.value.state.doc.length, insert: newContent },
    })
  }
}

// 更新语言
const updateLanguage = (_lang: string) => {
  if (!view.value) return
  const currentLang = props.language || 'plaintext'
  if (_lang !== currentLang) {
    createEditor()
  }
}

// ResizeObserver 重建 — 修复异步组件在 layout 前初始化导致的尺寸错乱
const setupResizeObserver = () => {
  if (!wrapperRef.value) return
  resizeObserver?.disconnect()
  resizeObserver = new ResizeObserver(() => {
    createEditor()
  })
  resizeObserver.observe(wrapperRef.value)
}

// 查找
const openFind = () => {
  if (view.value) {
    openSearchPanel(view.value)
    const inputs = view.value.dom.querySelectorAll<HTMLInputElement>('.cm-panel input.cm-textfield')
    if (inputs.length > 0) {
      inputs[0].focus()
      inputs[0].select()
    }
  }
}

// 替换
const openReplace = () => {
  if (!view.value) return
  openSearchPanel(view.value)
  const panel = view.value.dom.querySelector('.cm-panel')
  if (panel) {
    panel.querySelectorAll<HTMLElement>('.cm-replace-section').forEach((el) => {
      el.style.display = ''
    })
    const inputs = panel.querySelectorAll<HTMLInputElement>('input.cm-textfield')
    if (inputs.length >= 2) {
      inputs[1].focus()
      inputs[1].select()
    }
  }
}

// 行操作
const sortLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  lines.sort()
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: lines.join('\n') },
  })
}

const dedupLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  const unique = [...new Set(lines)]
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: unique.join('\n') },
  })
}

const reverseLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  lines.reverse()
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: lines.join('\n') },
  })
}

const toUpperCase = () => {
  if (!view.value) return
  const selection = view.value.state.selection.main
  if (selection.empty) return
  const text = view.value.state.doc.sliceString(selection.from, selection.to)
  view.value.dispatch({
    changes: { from: selection.from, to: selection.to, insert: text.toUpperCase() },
  })
}

const toLowerCase = () => {
  if (!view.value) return
  const selection = view.value.state.selection.main
  if (selection.empty) return
  const text = view.value.state.doc.sliceString(selection.from, selection.to)
  view.value.dispatch({
    changes: { from: selection.from, to: selection.to, insert: text.toLowerCase() },
  })
}

// 格式化代码
const formatCode = () => {
  if (!view.value) return
  const lang = props.language || 'plaintext'
  const content = view.value.state.doc.toString()

  if (lang === 'json') {
    try {
      const formatted = JSON.stringify(JSON.parse(content), null, 2)
      view.value.dispatch({
        changes: { from: 0, to: view.value.state.doc.length, insert: formatted },
      })
    } catch { /* noop */ }
  } else if (lang === 'javascript' || lang === 'typescript') {
    const lines = content.split('\n')
    let indent = 0
    const formatted = lines.map(line => {
      const trimmed = line.trim()
      if (trimmed.endsWith('{') || trimmed.endsWith('[')) {
        const result = '  '.repeat(indent) + trimmed
        indent++
        return result
      } else if (trimmed.startsWith('}') || trimmed.startsWith(']')) {
        indent = Math.max(0, indent - 1)
        return '  '.repeat(indent) + trimmed
      }
      return '  '.repeat(indent) + trimmed
    }).join('\n')
    view.value.dispatch({
      changes: { from: 0, to: view.value.state.doc.length, insert: formatted },
    })
  }
}

// 切换自动换行
const toggleWordWrap = () => {
  wordWrap.value = !wordWrap.value
  view.value?.dispatch({
    effects: wrapCompartment.reconfigure(wordWrap.value ? EditorView.lineWrapping : [])
  })
}

// 跳转到指定行号
const gotoLine = (lineNum: number): number => {
  if (!view.value) return 0
  const doc = view.value.state.doc
  const clamped = Math.max(1, Math.min(lineNum, doc.lines))
  const pos = doc.line(clamped).from
  view.value.dispatch({
    selection: { anchor: pos },
    scrollIntoView: true,
  })
  view.value.focus()
  return clamped
}

// 暴露方法给父组件
defineExpose({
  openFind,
  openReplace,
  sortLines,
  dedupLines,
  reverseLines,
  toUpperCase,
  toLowerCase,
  formatCode,
  updateLanguage,
  toggleWordWrap,
  isWordWrap: () => wordWrap.value,
  gotoLine,
  getLineCount: () => view.value?.state.doc.lines ?? 0,
  getCurrentLine: () => {
    if (!view.value) return 1
    return view.value.state.doc.lineAt(view.value.state.selection.main.head).number
  },
})

onMounted(async () => {
  // 等父容器 layout 完成再初始化，避免 auto 高度
  await nextTick()
  await new Promise(res => requestAnimationFrame(res))
  createEditor()
  setupResizeObserver()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  view.value?.destroy()
})

watch(() => props.modelValue, (newVal) => {
  updateContent(newVal)
})

watch(() => props.language, () => {
  createEditor()
})

watch(() => store.config.editorTheme, () => {
  createEditor()
})
</script>

<style scoped>
.codemirror-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
}

.codemirror-internal {
  position: absolute;
  inset: 0;
}
</style>
