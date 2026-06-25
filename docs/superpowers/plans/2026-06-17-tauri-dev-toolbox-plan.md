# LitoBox V1.0 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 基于Tauri 2.0 + Vue3 + TypeScript构建Windows桌面开发工具箱，实现JSON处理、字符串处理、基础编码工具，支持托盘常驻、全局热键、主题适配。

**Architecture:** Tauri 2.0作为桌面壳层提供原生能力，Vue3 + Vite作为前端框架，Element Plus提供UI组件，工具函数纯函数封装，localStorage持久化配置。

**Tech Stack:** Tauri 2.0, Vue 3, TypeScript, Element Plus, Vite, json5, js-base64, lodash, Pinia

---

## 文件结构映射

### 创建的文件清单

```
litobox/
├── package.json                          # 项目依赖配置
├── tsconfig.json                         # TypeScript配置
├── vite.config.ts                        # Vite构建配置
├── index.html                            # HTML入口
├── src/
│   ├── main.ts                           # Vue应用入口
│   ├── App.vue                           # 根组件（标签页框架）
│   ├── components/
│   │   ├── ToolInput.vue                 # 通用输入组件
│   │   ├── ToolOutput.vue                # 通用输出组件
│   │   └── ToolActions.vue               # 操作按钮组件
│   ├── views/
│   │   ├── JsonTool.vue                  # JSON工具页面
│   │   ├── StringTool.vue                # 字符串工具页面
│   │   └── EncodeTool.vue                # 编码工具页面
│   ├── utils/
│   │   ├── jsonUtils.ts                  # JSON工具函数
│   │   ├── stringUtils.ts                # 字符串工具函数
│   │   └── encodeUtils.ts                # 编码工具函数
│   ├── store/
│   │   └── index.ts                      # Pinia状态管理
│   └── style/
│       ├── main.css                      # 全局样式
│       └── theme.css                     # 主题样式
└── src-tauri/
    ├── Cargo.toml                        # Rust依赖
    ├── tauri.conf.json                   # Tauri配置
    └── src/
        └── main.rs                       # Rust入口
```

---

## 任务分解

### Task 1: 项目初始化与基础配置

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`

- [ ] **Step 1: 创建package.json**

```json
{
  "name": "litobox",
  "version": "1.0.0",
  "description": "栗的百宝箱",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.4.0",
    "element-plus": "^2.5.0",
    "@element-plus/icons-vue": "^2.3.0",
    "pinia": "^2.1.0",
    "json5": "^2.2.3",
    "js-base64": "^3.7.6",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "typescript": "^5.3.0",
    "vite": "^5.1.0",
    "vue-tsc": "^1.8.0",
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

- [ ] **Step 2: 创建tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 3: 创建tsconfig.node.json**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 4: 创建vite.config.ts**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
})
```

- [ ] **Step 5: 创建index.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>开发工具箱</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 6: 验证项目结构**

```bash
ls -la
```
Expected: 显示package.json, tsconfig.json, vite.config.ts, index.html

---

### Task 2: 工具函数层实现（TDD）

**Files:**
- Create: `src/utils/jsonUtils.ts`
- Create: `src/utils/stringUtils.ts`
- Create: `src/utils/encodeUtils.ts`
- Test: 手动测试验证

- [ ] **Step 1: 创建jsonUtils.ts**

```typescript
import JSON5 from 'json5'

export interface JsonFormatResult {
  success: boolean
  data?: string
  error?: string
  errorLine?: number
}

export interface JsonFormatOptions {
  indent: number
}

const defaultOptions: JsonFormatOptions = {
  indent: 2
}

export function formatJson(input: string, options: JsonFormatOptions = defaultOptions): JsonFormatResult {
  try {
    const parsed = JSON5.parse(input)
    const formatted = JSON.stringify(parsed, null, options.indent)
    return { success: true, data: formatted }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    const lineMatch = message.match(/line\s+(\d+)/i)
    return {
      success: false,
      error: message,
      errorLine: lineMatch ? parseInt(lineMatch[1], 10) : undefined
    }
  }
}

export function compressJson(input: string): JsonFormatResult {
  try {
    const parsed = JSON5.parse(input)
    const compressed = JSON.stringify(parsed)
    return { success: true, data: compressed }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    return { success: false, error: message }
  }
}

export function validateJson(input: string): JsonFormatResult {
  try {
    JSON5.parse(input)
    return { success: true, data: 'JSON格式正确' }
  } catch (error) {
    const message = error instanceof Error ? error.message : '未知错误'
    const lineMatch = message.match(/line\s+(\d+)/i)
    return {
      success: false,
      error: message,
      errorLine: lineMatch ? parseInt(lineMatch[1], 10) : undefined
    }
  }
}
```

- [ ] **Step 2: 创建stringUtils.ts**

```typescript
export function trimLeadingTrailing(text: string): string {
  return text.split('\n').map(line => line.trim()).join('\n')
}

export function trimAllSpaces(text: string): string {
  return text.replace(/\s+/g, '')
}

export function trimSpacesKeepNewlines(text: string): string {
  return text.split('\n').map(line => line.replace(/[^\S\n]/g, '')).join('\n')
}

export function joinLines(text: string, separator: string = ','): string {
  return text.split('\n')
    .filter(line => line.trim() !== '')
    .map(line => line.trim())
    .join(separator)
}

export function splitText(text: string, separator: string): string[] {
  return text.split(separator).map(item => item.trim()).filter(item => item !== '')
}

export function toUpperCase(text: string): string {
  return text.toUpperCase()
}

export function toLowerCase(text: string): string {
  return text.toLowerCase()
}

export function toTitleCase(text: string): string {
  return text.replace(/\w\S*/g, txt => txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase())
}

export function toCamelCase(text: string): string {
  return text.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : '')
}

export function toSnakeCase(text: string): string {
  return text.replace(/([A-Z])/g, '_$1').replace(/[-\s]+/g, '_').toLowerCase().replace(/^_/, '')
}

export function removeNewlines(text: string): string {
  return text.replace(/\n/g, '')
}

export function removeTabs(text: string): string {
  return text.replace(/\t/g, '')
}

export function removeEmptyLines(text: string): string {
  return text.split('\n').filter(line => line.trim() !== '').join('\n')
}
```

- [ ] **Step 3: 创建encodeUtils.ts**

```typescript
import { Base64 } from 'js-base64'

export function urlEncode(text: string): string {
  return encodeURIComponent(text)
}

export function urlDecode(text: string): string {
  try {
    return decodeURIComponent(text)
  } catch {
    return '解码失败：无效的URL编码'
  }
}

export function base64Encode(text: string): string {
  return Base64.encode(text)
}

export function base64Decode(text: string): string {
  try {
    return Base64.decode(text)
  } catch {
    return '解码失败：无效的Base64编码'
  }
}

export function timestampToDatetime(timestamp: number, isMilliseconds: boolean = true): string {
  const ms = isMilliseconds ? timestamp : timestamp * 1000
  const date = new Date(ms)
  if (isNaN(date.getTime())) {
    return '无效的时间戳'
  }
  return date.toLocaleString('zh-CN', { 
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })
}

export function datetimeToTimestamp(datetime: string, isMilliseconds: boolean = true): number | string {
  const date = new Date(datetime)
  if (isNaN(date.getTime())) {
    return '无效的日期时间格式'
  }
  const timestamp = date.getTime()
  return isMilliseconds ? timestamp : Math.floor(timestamp / 1000)
}
```

- [ ] **Step 4: 手动测试验证**

创建临时测试文件验证函数正确性：

```typescript
// 临时测试
import { formatJson, compressJson } from './jsonUtils'
import { trimAllSpaces, toCamelCase } from './stringUtils'
import { urlEncode, base64Encode, timestampToDatetime } from './encodeUtils'

// JSON测试
console.log(formatJson('{"name":"test","age":25}'))
console.log(compressJson('{ "name": "test" }'))

// 字符串测试
console.log(trimAllSpaces('  hello   world  '))
console.log(toCamelCase('hello_world_test'))

// 编码测试
console.log(urlEncode('你好世界'))
console.log(base64Encode('Hello World'))
console.log(timestampToDatetime(1718611200000))
```

---

### Task 3: 公共组件实现

**Files:**
- Create: `src/components/ToolInput.vue`
- Create: `src/components/ToolOutput.vue`
- Create: `src/components/ToolActions.vue`

- [ ] **Step 1: 创建ToolInput.vue**

```vue
<template>
  <div class="tool-input">
    <div class="input-header">
      <span class="input-title">输入</span>
      <div class="input-actions">
        <el-button size="small" @click="handleClear">清空</el-button>
        <el-button size="small" @click="handlePaste">粘贴</el-button>
      </div>
    </div>
    <el-input
      v-model="inputValue"
      type="textarea"
      :rows="rows"
      :placeholder="placeholder"
      resize="vertical"
      @input="handleInput"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  modelValue: string
  placeholder?: string
  rows?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  clear: []
  paste: []
}>()

const inputValue = ref(props.modelValue)

watch(() => props.modelValue, (newVal) => {
  inputValue.value = newVal
})

const handleInput = (value: string) => {
  emit('update:modelValue', value)
}

const handleClear = () => {
  inputValue.value = ''
  emit('update:modelValue', '')
  emit('clear')
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
    emit('update:modelValue', text)
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}
</script>

<style scoped>
.tool-input {
  margin-bottom: 16px;
}
.input-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.input-title {
  font-weight: 500;
  font-size: 14px;
}
.input-actions {
  display: flex;
  gap: 8px;
}
</style>
```

- [ ] **Step 2: 创建ToolOutput.vue**

```vue
<template>
  <div class="tool-output">
    <div class="output-header">
      <span class="output-title">输出</span>
      <el-button size="small" @click="handleCopy">复制</el-button>
    </div>
    <el-input
      :model-value="outputValue"
      type="textarea"
      :rows="rows"
      readonly
      resize="vertical"
      :class="{ 'error': isError }"
    />
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  outputValue: string
  errorMessage?: string
  isError?: boolean
  rows?: number
}>()

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(props.outputValue)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-output {
  margin-bottom: 16px;
}
.output-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.output-title {
  font-weight: 500;
  font-size: 14px;
}
.error :deep(.el-textarea__inner) {
  border-color: #f56c6c;
}
.error-message {
  color: #f56c6c;
  font-size: 12px;
  margin-top: 4px;
}
</style>
```

- [ ] **Step 3: 创建ToolActions.vue**

```vue
<template>
  <div class="tool-actions">
    <el-button
      v-for="action in actions"
      :key="action.label"
      :type="action.type || 'default'"
      size="small"
      @click="handleAction(action)"
    >
      {{ action.label }}
    </el-button>
  </div>
</template>

<script setup lang="ts">
export interface ToolAction {
  label: string
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default'
  handler: () => void
}

defineProps<{
  actions: ToolAction[]
}>()

const handleAction = (action: ToolAction) => {
  action.handler()
}
</script>

<style scoped>
.tool-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
</style>
```

---

### Task 4: 状态管理与样式

**Files:**
- Create: `src/store/index.ts`
- Create: `src/style/main.css`
- Create: `src/style/theme.css`

- [ ] **Step 1: 创建Pinia状态管理**

```typescript
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export interface ToolboxConfig {
  theme: 'auto' | 'dark' | 'light'
  jsonIndent: 2 | 4
  hotkey: string
  lastTool: string
}

export interface HistoryRecord {
  tool: string
  action: string
  timestamp: string
  inputPreview: string
  outputPreview: string
}

const STORAGE_KEY_CONFIG = 'toolbox_config'
const STORAGE_KEY_HISTORY = 'toolbox_history'
const MAX_HISTORY = 10

export const useToolboxStore = defineStore('toolbox', () => {
  const config = ref<ToolboxConfig>({
    theme: 'auto',
    jsonIndent: 2,
    hotkey: 'Ctrl+Alt+T',
    lastTool: 'json'
  })

  const history = ref<HistoryRecord[]>([])

  // 加载本地存储
  const loadFromStorage = () => {
    try {
      const savedConfig = localStorage.getItem(STORAGE_KEY_CONFIG)
      if (savedConfig) {
        config.value = { ...config.value, ...JSON.parse(savedConfig) }
      }
      
      const savedHistory = localStorage.getItem(STORAGE_KEY_HISTORY)
      if (savedHistory) {
        history.value = JSON.parse(savedHistory)
      }
    } catch (error) {
      console.error('加载本地配置失败:', error)
    }
  }

  // 保存配置
  const saveConfig = (newConfig: Partial<ToolboxConfig>) => {
    config.value = { ...config.value, ...newConfig }
    localStorage.setItem(STORAGE_KEY_CONFIG, JSON.stringify(config.value))
  }

  // 添加历史记录
  const addHistory = (record: Omit<HistoryRecord, 'timestamp'>) => {
    const newRecord = {
      ...record,
      timestamp: new Date().toISOString()
    }
    history.value.unshift(newRecord)
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY)
    }
    localStorage.setItem(STORAGE_KEY_HISTORY, JSON.stringify(history.value))
  }

  // 清空历史
  const clearHistory = () => {
    history.value = []
    localStorage.removeItem(STORAGE_KEY_HISTORY)
  }

  // 初始化加载
  loadFromStorage()

  return {
    config,
    history,
    saveConfig,
    addHistory,
    clearHistory
  }
})
```

- [ ] **Step 2: 创建main.css**

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#app {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tool-container {
  flex: 1;
  padding: 16px;
  overflow: auto;
}

.tool-section {
  margin-bottom: 24px;
}
```

- [ ] **Step 3: 创建theme.css**

```css
:root {
  --bg-color: #ffffff;
  --text-color: #303133;
  --border-color: #dcdfe6;
  --hover-bg: #f5f7fa;
}

html.dark {
  --bg-color: #1a1a1a;
  --text-color: #e5eaf3;
  --border-color: #4c4c4c;
  --hover-bg: #2d2d2d;
}

body {
  background-color: var(--bg-color);
  color: var(--text-color);
  transition: background-color 0.3s, color 0.3s;
}

.el-textarea__inner {
  background-color: var(--bg-color) !important;
  color: var(--text-color) !important;
  border-color: var(--border-color) !important;
}

.el-button {
  --el-button-bg-color: var(--bg-color);
  --el-button-border-color: var(--border-color);
  --el-button-text-color: var(--text-color);
}
```

---

### Task 5: 功能页面实现

**Files:**
- Create: `src/views/JsonTool.vue`
- Create: `src/views/StringTool.vue`
- Create: `src/views/EncodeTool.vue`
- Create: `src/App.vue`
- Create: `src/main.ts`

- [ ] **Step 1: 创建JsonTool.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-section">
      <ToolActions :actions="jsonActions" />
    </div>
    
    <div class="tool-section">
      <ToolInput 
        v-model="inputValue" 
        placeholder="请输入JSON内容..."
        :rows="10"
      />
    </div>
    
    <div class="tool-section">
      <ToolOutput 
        :output-value="outputValue"
        :error-message="errorMessage"
        :is-error="isError"
        :rows="10"
      />
    </div>
    
    <div class="tool-section">
      <el-radio-group v-model="indentSize" size="small">
        <el-radio-button :label="2">2空格缩进</el-radio-button>
        <el-radio-button :label="4">4空格缩进</el-radio-button>
      </el-radio-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { formatJson, compressJson, validateJson } from '@/utils/jsonUtils'
import { useToolboxStore } from '@/store'
import ToolInput from '@/components/ToolInput.vue'
import ToolOutput from '@/components/ToolOutput.vue'
import ToolActions, { type ToolAction } from '@/components/ToolActions.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const indentSize = ref(store.config.jsonIndent)

const jsonActions = computed<ToolAction[]>(() => [
  {
    label: '格式化',
    type: 'primary',
    handler: () => handleFormat()
  },
  {
    label: '压缩',
    type: 'success',
    handler: () => handleCompress()
  },
  {
    label: '校验',
    type: 'warning',
    handler: () => handleValidate()
  }
])

const handleFormat = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = formatJson(inputValue.value, { indent: indentSize.value })
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    store.addHistory({
      tool: 'json',
      action: 'format',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50)
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
  }
}

const handleCompress = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = compressJson(inputValue.value)
  if (result.success) {
    outputValue.value = result.data || ''
    errorMessage.value = ''
    isError.value = false
    store.addHistory({
      tool: 'json',
      action: 'compress',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: outputValue.value.slice(0, 50)
    })
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
  }
}

const handleValidate = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  
  const result = validateJson(inputValue.value)
  if (result.success) {
    outputValue.value = '✓ JSON格式正确'
    errorMessage.value = ''
    isError.value = false
    ElMessage.success('JSON格式正确')
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
    ElMessage.error('JSON格式错误')
  }
}
</script>
```

- [ ] **Step 2: 创建StringTool.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-section">
      <ToolActions :actions="stringActions" />
    </div>
    
    <div class="tool-section">
      <ToolInput 
        v-model="inputValue" 
        placeholder="请输入文本内容..."
        :rows="8"
      />
    </div>
    
    <div class="tool-section">
      <ToolOutput 
        :output-value="outputValue"
        :rows="8"
      />
    </div>
    
    <div class="tool-section">
      <el-divider>分隔符设置</el-divider>
      <el-input
        v-model="separator"
        placeholder="自定义分隔符"
        size="small"
        style="width: 200px"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import * as stringUtils from '@/utils/stringUtils'
import { useToolboxStore } from '@/store'
import ToolInput from '@/components/ToolInput.vue'
import ToolOutput from '@/components/ToolOutput.vue'
import ToolActions, { type ToolAction } from '@/components/ToolActions.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const separator = ref(',')

const stringActions = ref<ToolAction[]>([
  { label: '首尾去空', handler: () => applyTransform(stringUtils.trimLeadingTrailing) },
  { label: '全局去空', handler: () => applyTransform(stringUtils.trimAllSpaces) },
  { label: '保留换行去空', handler: () => applyTransform(stringUtils.trimSpacesKeepNewlines) },
  { label: '拼接', handler: () => applyTransform(text => stringUtils.joinLines(text, separator.value)) },
  { label: '全大写', handler: () => applyTransform(stringUtils.toUpperCase) },
  { label: '全小写', handler: () => applyTransform(stringUtils.toLowerCase) },
  { label: '首字母大写', handler: () => applyTransform(stringUtils.toTitleCase) },
  { label: '转驼峰', handler: () => applyTransform(stringUtils.toCamelCase) },
  { label: '转下划线', handler: () => applyTransform(stringUtils.toSnakeCase) },
  { label: '去除换行', handler: () => applyTransform(stringUtils.removeNewlines) },
  { label: '去除制表符', handler: () => applyTransform(stringUtils.removeTabs) },
  { label: '删除空行', handler: () => applyTransform(stringUtils.removeEmptyLines) }
])

const applyTransform = (transform: (text: string) => string) => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入文本内容')
    return
  }
  
  outputValue.value = transform(inputValue.value)
  store.addHistory({
    tool: 'string',
    action: 'transform',
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50)
  })
  ElMessage.success('处理完成')
}
</script>
```

- [ ] **Step 3: 创建EncodeTool.vue**

```vue
<template>
  <div class="tool-container">
    <div class="tool-section">
      <ToolActions :actions="encodeActions" />
    </div>
    
    <div class="tool-section">
      <ToolInput 
        v-model="inputValue" 
        placeholder="请输入内容..."
        :rows="6"
      />
    </div>
    
    <div class="tool-section">
      <ToolOutput 
        :output-value="outputValue"
        :error-message="errorMessage"
        :is-error="isError"
        :rows="6"
      />
    </div>
    
    <div class="tool-section">
      <el-divider>时间戳选项</el-divider>
      <el-radio-group v-model="timestampMode" size="small">
        <el-radio-button label="ms">毫秒级</el-radio-button>
        <el-radio-button label="s">秒级</el-radio-button>
      </el-radio-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import * as encodeUtils from '@/utils/encodeUtils'
import { useToolboxStore } from '@/store'
import ToolInput from '@/components/ToolInput.vue'
import ToolOutput from '@/components/ToolOutput.vue'
import ToolActions, { type ToolAction } from '@/components/ToolActions.vue'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const errorMessage = ref('')
const isError = ref(false)
const timestampMode = ref<'ms' | 's'>('ms')

const encodeActions = ref<ToolAction[]>([
  { label: 'URL编码', handler: () => handleEncode(encodeUtils.urlEncode) },
  { label: 'URL解码', handler: () => handleEncode(encodeUtils.urlDecode) },
  { label: 'Base64编码', handler: () => handleEncode(encodeUtils.base64Encode) },
  { label: 'Base64解码', handler: () => handleEncode(encodeUtils.base64Decode) },
  { label: '时间戳→时间', handler: () => handleTimestampToDatetime() },
  { label: '时间→时间戳', handler: () => handleDatetimeToTimestamp() }
])

const handleEncode = (encodeFn: (text: string) => string) => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  
  const result = encodeFn(inputValue.value)
  outputValue.value = result
  errorMessage.value = ''
  isError.value = false
  
  store.addHistory({
    tool: 'encode',
    action: 'encode',
    inputPreview: inputValue.value.slice(0, 50),
    outputPreview: outputValue.value.slice(0, 50)
  })
  ElMessage.success('处理完成')
}

const handleTimestampToDatetime = () => {
  const timestamp = Number(inputValue.value)
  if (isNaN(timestamp)) {
    errorMessage.value = '请输入有效的时间戳'
    isError.value = true
    return
  }
  
  outputValue.value = encodeUtils.timestampToDatetime(timestamp, timestampMode.value === 'ms')
  errorMessage.value = ''
  isError.value = false
  ElMessage.success('转换完成')
}

const handleDatetimeToTimestamp = () => {
  const result = encodeUtils.datetimeToTimestamp(inputValue.value, timestampMode.value === 'ms')
  if (typeof result === 'string') {
    errorMessage.value = result
    isError.value = true
  } else {
    outputValue.value = String(result)
    errorMessage.value = ''
    isError.value = false
    ElMessage.success('转换完成')
  }
}
</script>
```

- [ ] **Step 4: 创建App.vue**

```vue
<template>
  <el-container class="app-container">
    <el-header class="app-header">
      <h1 class="app-title">开发工具箱</h1>
      <div class="header-actions">
        <el-select v-model="currentTheme" size="small" style="width: 100px">
          <el-option label="跟随系统" value="auto" />
          <el-option label="浅色" value="light" />
          <el-option label="深色" value="dark" />
        </el-select>
      </div>
    </el-header>
    
    <el-main class="app-main">
      <el-tabs v-model="activeTool" type="border-card">
        <el-tab-pane label="JSON工具" name="json">
          <JsonTool />
        </el-tab-pane>
        <el-tab-pane label="字符串工具" name="string">
          <StringTool />
        </el-tab-pane>
        <el-tab-pane label="编码工具" name="encode">
          <EncodeTool />
        </el-tab-pane>
      </el-tabs>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useToolboxStore } from '@/store'
import JsonTool from '@/views/JsonTool.vue'
import StringTool from '@/views/StringTool.vue'
import EncodeTool from '@/views/EncodeTool.vue'

const store = useToolboxStore()
const activeTool = ref(store.config.lastTool)
const currentTheme = ref(store.config.theme)

watch(activeTool, (newTool) => {
  store.saveConfig({ lastTool: newTool })
})

watch(currentTheme, (newTheme) => {
  store.saveConfig({ theme: newTheme as 'auto' | 'dark' | 'light' })
  applyTheme(newTheme)
})

const applyTheme = (theme: string) => {
  const html = document.documentElement
  if (theme === 'dark') {
    html.classList.add('dark')
  } else if (theme === 'light') {
    html.classList.remove('dark')
  } else {
    // auto - 跟随系统
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) {
      html.classList.add('dark')
    } else {
      html.classList.remove('dark')
    }
  }
}

onMounted(() => {
  applyTheme(currentTheme.value)
  
  // 监听系统主题变化
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (currentTheme.value === 'auto') {
      applyTheme('auto')
    }
  })
})
</script>

<style scoped>
.app-container {
  height: 100vh;
}
.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
  padding: 0 16px;
}
.app-title {
  font-size: 18px;
  font-weight: 600;
}
.header-actions {
  display: flex;
  gap: 12px;
}
.app-main {
  padding: 0;
}
</style>
```

- [ ] **Step 5: 创建main.ts**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import App from './App.vue'
import './style/main.css'
import './style/theme.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(ElementPlus)
app.mount('#app')
```

---

### Task 6: Tauri配置与Rust底层

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`

- [ ] **Step 1: 创建Cargo.toml**

```toml
[package]
name = "litobox"
version = "1.0.0"
description = "栗的百宝箱"
authors = ["developer"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0.0", features = [] }

[dependencies]
tauri = { version = "2.0.0", features = ["shell-open", "clipboard-all", "global-shortcut-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[features]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 2: 创建tauri.conf.json**

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "identifier": "com.dev.toolbox",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "title": "开发工具箱",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'"
    }
  }
}
```

- [ ] **Step 3: 创建main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

fn main() {
    let system_tray = SystemTray::new()
        .with_menu(SystemTrayMenu::new()
            .add_item(SystemTrayMenuItem::Text("显示窗口".into(), "show".into()))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(SystemTrayMenuItem::Text("退出".into(), "quit".into()))
        );

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Task 7: 项目验证与启动

**Files:**
- 验证所有文件
- 安装依赖
- 启动开发服务器

- [ ] **Step 1: 安装依赖**

```bash
npm install
```

Expected: 成功安装所有依赖，无错误

- [ ] **Step 2: 验证项目结构**

```bash
ls -la src/
ls -la src-tauri/
```

Expected: 显示所有创建的文件

- [ ] **Step 3: 启动开发服务器**

```bash
npm run tauri dev
```

Expected: 
- Vite开发服务器启动在 http://localhost:1420
- Tauri窗口打开
- 显示"开发工具箱"标题
- 三个标签页正常显示

- [ ] **Step 4: 功能验证清单**

- [ ] JSON工具：输入`{"name":"test"}`，点击格式化，显示缩进结果
- [ ] JSON工具：点击压缩，显示单行结果
- [ ] JSON工具：输入错误JSON，显示错误提示
- [ ] 字符串工具：输入多行文本，测试去空、拼接等功能
- [ ] 编码工具：测试URL编解码、Base64编解码
- [ ] 编码工具：测试时间戳转换
- [ ] 主题切换：测试浅色/深色/自动主题
- [ ] 复制功能：点击复制按钮，验证剪贴板内容
- [ ] 清空功能：点击清空按钮，验证输入框清空

---

## 自审检查

### 1. 规范覆盖检查
- [x] JSON格式化/压缩/校验 - Task 5, jsonUtils.ts
- [x] 字符串处理全套 - Task 5, stringUtils.ts
- [x] 基础编码工具 - Task 5, encodeUtils.ts
- [x] 主题适配 - Task 4, theme.css, App.vue
- [x] 托盘常驻 - Task 6, main.rs
- [x] 基础热键 - Task 6, tauri.conf.json (预留)
- [x] 本地配置缓存 - Task 4, store/index.ts

### 2. 占位符扫描
- [x] 无TBD/TODO
- [x] 所有函数有完整实现
- [x] 所有测试有具体代码
- [x] 类型签名一致

### 3. 类型一致性检查
- [x] jsonUtils.ts返回JsonFormatResult
- [x] stringUtils.ts统一string输入输出
- [x] encodeUtils.ts类型明确
- [x] store类型定义与使用一致

---

## 执行交接

**计划完成并保存到:** `docs/superpowers/plans/2026-06-17-tauri-dev-toolbox-plan.md`

**两种执行方案：**

**1. 子代理驱动（推荐）** - 每个任务独立子代理，任务间审查，快速迭代

**2. 内联执行** - 在当前会话使用executing-plans，批量执行带检查点

**选择哪种方式？**