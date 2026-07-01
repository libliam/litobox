<template>
  <div ref="editorRef" class="codemirror-wrapper"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, shallowRef } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { searchKeymap, highlightSelectionMatches, openSearchPanel } from '@codemirror/search'
import { bracketMatching, foldGutter, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { xml } from '@codemirror/lang-xml'
import { sql } from '@codemirror/lang-sql'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { useToolboxStore } from '@/store'

const props = defineProps<{
  modelValue: string
  language?: string
  readOnly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': [value: string]
}>()

const store = useToolboxStore()
const editorRef = ref<HTMLElement>()
const view = shallowRef<EditorView>()

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
}

// 根据主题获取扩展
const getThemeExtension = () => {
  return store.config.theme === 'light' ? [] : [oneDark]
}

// 获取语言扩展
const getLangExtension = (lang: string) => {
  const ext = langExtensions[lang]
  return ext ? [ext()] : []
}

// 创建编辑器
const createEditor = () => {
  if (!editorRef.value) return

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      bracketMatching(),
      foldGutter(),
      syntaxHighlighting(defaultHighlightStyle),
      history(),
      highlightSelectionMatches(),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
        ...foldKeymap,
      ]),
      ...getThemeExtension(),
      ...getLangExtension(props.language || 'plaintext'),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const newValue = update.state.doc.toString()
          emit('update:modelValue', newValue)
          emit('change', newValue)
        }
      }),
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: '14px',
        },
        '.cm-scroller': {
          overflow: 'auto',
        },
      }),
    ],
  })

  view.value = new EditorView({
    state,
    parent: editorRef.value,
  })
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
const updateLanguage = (lang: string) => {
  if (!view.value) return
  const currentLang = props.language || 'plaintext'
  if (lang !== currentLang) {
    const content = view.value.state.doc.toString()
    view.value.destroy()
    createEditor()
  }
}

// 查找
const openFind = () => {
  if (view.value) {
    openSearchPanel(view.value)
  }
}

// 替换
const openReplace = () => {
  if (view.value) {
    openSearchPanel(view.value)
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
    } catch {
      // JSON 格式错误，不处理
    }
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
})

onMounted(() => {
  createEditor()
})

onUnmounted(() => {
  view.value?.destroy()
})

watch(() => props.modelValue, (newVal) => {
  updateContent(newVal)
})

watch(() => props.language, (newLang) => {
  if (newLang) updateLanguage(newLang)
})

watch(() => store.config.theme, () => {
  if (view.value) {
    view.value.destroy()
    createEditor()
  }
})
</script>

<style scoped>
.codemirror-wrapper {
  height: 100%;
  min-height: 400px;
}
</style>
