# 第二阶段工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 CSS工具、JWT解析、字数统计 3 个独立工具页面，并集成到应用中，版本号升级至 2.2.0。

**Architecture:** 每个工具创建独立 Vue 页面文件，基于现有模板结构（操作/输入/输出卡片），在 store 的 TOOL_LIST 中注册，在 App.vue 中添加路由。所有工具纯前端实现，不引入新依赖。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus, CryptoJS (已安装), js-base64 (已安装)

---

## 文件结构

| 操作 | 文件 | 说明 |
|------|------|------|
| Create | `src/views/CssTool.vue` | CSS工具页面 |
| Create | `src/views/JwtTool.vue` | JWT解析页面 |
| Create | `src/views/WordCountTool.vue` | 字数统计页面 |
| Create | `src/utils/cssUtils.ts` | CSS工具函数 |
| Modify | `src/store/index.ts:29-47` | TOOL_LIST 添加3个新工具 |
| Modify | `src/App.vue:1-60` | 导入组件+路由 |
| Modify | `src/components/SidebarNav.vue:7` | 版本号 v2.0 → v2.2 |
| Modify | `package.json:3` | 版本号 2.1.0 → 2.2.0 |

---

### Task 1: CSS工具函数 (cssUtils.ts)

**Files:**
- Create: `src/utils/cssUtils.ts`

- [ ] **Step 1: 创建 cssUtils.ts 工具函数**

```typescript
// src/utils/cssUtils.ts

// ============ 颜色转换 ============

export interface RGB { r: number; g: number; b: number }
export interface HSL { h: number; s: number; l: number }

export function hexToRgb(hex: string): RGB | null {
  const match = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim())
  if (!match) return null
  return {
    r: parseInt(match[1], 16),
    g: parseInt(match[2], 16),
    b: parseInt(match[3], 16)
  }
}

export function rgbToHex(r: number, g: number, b: number): string {
  return '#' + [r, g, b].map(x => {
    const hex = Math.round(Math.max(0, Math.min(255, x))).toString(16)
    return hex.length === 1 ? '0' + hex : hex
  }).join('')
}

export function rgbToHsl(r: number, g: number, b: number): HSL {
  r /= 255; g /= 255; b /= 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b)
  let h = 0, s = 0
  const l = (max + min) / 2

  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break
      case g: h = ((b - r) / d + 2) / 6; break
      case b: h = ((r - g) / d + 4) / 6; break
    }
  }

  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) }
}

export function hslToRgb(h: number, s: number, l: number): RGB {
  h /= 360; s /= 100; l /= 100
  let r: number, g: number, b: number

  if (s === 0) {
    r = g = b = l
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1
      if (t > 1) t -= 1
      if (t < 1/6) return p + (q - p) * 6 * t
      if (t < 1/2) return q
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6
      return p
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    r = hue2rgb(p, q, h + 1/3)
    g = hue2rgb(p, q, h)
    b = hue2rgb(p, q, h - 1/3)
  }

  return { r: Math.round(r * 255), g: Math.round(g * 255), b: Math.round(b * 255) }
}

export function parseColor(input: string): { hex: string; rgb: string; hsl: string } | null {
  input = input.trim()

  // Try hex
  if (/^#?[a-f\d]{6}$/i.test(input)) {
    const hex = input.startsWith('#') ? input : '#' + input
    const rgb = hexToRgb(hex)
    if (rgb) {
      const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b)
      return {
        hex,
        rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
        hsl: `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`
      }
    }
  }

  // Try rgb
  const rgbMatch = input.match(/rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/)
  if (rgbMatch) {
    const r = parseInt(rgbMatch[1]), g = parseInt(rgbMatch[2]), b = parseInt(rgbMatch[3])
    const hex = rgbToHex(r, g, b)
    const hsl = rgbToHsl(r, g, b)
    return {
      hex,
      rgb: input,
      hsl: `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`
    }
  }

  // Try hsl
  const hslMatch = input.match(/hsl\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*\)/)
  if (hslMatch) {
    const h = parseInt(hslMatch[1]), s = parseInt(hslMatch[2]), l = parseInt(hslMatch[3])
    const rgb = hslToRgb(h, s, l)
    const hex = rgbToHex(rgb.r, rgb.g, rgb.b)
    return {
      hex,
      rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
      hsl: input
    }
  }

  return null
}

// ============ 单位换算 ============

export interface UnitConversion {
  from: string
  to: string
  value: number
  result: number
}

export function convertUnit(value: number, from: string, to: string, baseFontSize: number = 16): number | null {
  // 先转换为 px
  let px: number
  switch (from.toLowerCase()) {
    case 'px': px = value; break
    case 'rem': px = value * baseFontSize; break
    case 'em': px = value * baseFontSize; break
    case 'vw': px = value * 19.2; break  // 假设 1920px 视口
    case 'vh': px = value * 10.8; break  // 假设 1080px 视口
    default: return null
  }

  // 从 px 转换为目标单位
  switch (to.toLowerCase()) {
    case 'px': return px
    case 'rem': return px / baseFontSize
    case 'em': return px / baseFontSize
    case 'vw': return px / 19.2
    case 'vh': return px / 10.8
    default: return null
  }
}

// ============ CSS 压缩 ============

export function compressCss(css: string): string {
  return css
    .replace(/\/\*[\s\S]*?\*\//g, '')  // 移除注释
    .replace(/\s+/g, ' ')               // 合并空白
    .replace(/\s*([{}:;,])\s*/g, '$1') // 移除符号周围空格
    .replace(/;}/g, '}')               // 移除最后一个分号
    .trim()
}

export function formatCss(css: string): string {
  let formatted = ''
  let indent = 0
  const tab = '  '

  for (const char of css) {
    if (char === '{') {
      formatted += ' {\n'
      indent++
      formatted += tab.repeat(indent)
    } else if (char === '}') {
      indent--
      formatted += '\n' + tab.repeat(indent) + '}\n\n'
    } else if (char === ';') {
      formatted += ';\n' + tab.repeat(indent)
    } else if (char === '\n' || char === '\r') {
      // 跳过原始换行
    } else if (char === ' ' && formatted.endsWith(tab.repeat(indent))) {
      // 跳过多余空格
    } else {
      formatted += char
    }
  }

  return formatted.trim()
}
```

---

### Task 2: CSS工具页面 (CssTool.vue)

**Files:**
- Create: `src/views/CssTool.vue`

- [ ] **Step 1: 创建 CssTool.vue 页面**

```vue
<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="css-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 颜色转换 -->
      <el-tab-pane label="颜色转换" name="color">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 支持 Hex、RGB、HSL 格式</p>
                    <p>• 自动识别输入格式并转换</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleColorConvert">转换</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('color')">清空</el-button>
              <el-button size="small" @click="handlePaste('color')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="tabState.color.input" type="textarea" :rows="4" placeholder="输入颜色值，如 #ff0000、rgb(255, 0, 0)、hsl(0, 100%, 50%)" resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('color')">复制</el-button>
          </div>
          <div class="card-body">
            <div v-if="colorResult" class="color-results">
              <div class="color-preview" :style="{ backgroundColor: colorResult.hex }"></div>
              <div class="color-item"><span class="color-label">HEX:</span><code>{{ colorResult.hex }}</code></div>
              <div class="color-item"><span class="color-label">RGB:</span><code>{{ colorResult.rgb }}</code></div>
              <div class="color-item"><span class="color-label">HSL:</span><code>{{ colorResult.hsl }}</code></div>
            </div>
            <div v-if="tabState.color.error" class="error-message">{{ tabState.color.error }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 单位换算 -->
      <el-tab-pane label="单位换算" name="unit">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">数值</div>
                <el-input-number v-model="unitValue" :precision="4" size="small" style="width: 120px" />
              </div>
              <div class="action-group">
                <div class="group-label">从</div>
                <el-select v-model="unitFrom" size="small" style="width: 80px">
                  <el-option label="px" value="px" />
                  <el-option label="rem" value="rem" />
                  <el-option label="em" value="em" />
                  <el-option label="vw" value="vw" />
                  <el-option label="vh" value="vh" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">到</div>
                <el-select v-model="unitTo" size="small" style="width: 80px">
                  <el-option label="px" value="px" />
                  <el-option label="rem" value="rem" />
                  <el-option label="em" value="em" />
                  <el-option label="vw" value="vw" />
                  <el-option label="vh" value="vh" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">基准字号</div>
                <el-input-number v-model="baseFontSize" :min="12" :max="32" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleUnitConvert">转换</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('unit')">复制</el-button>
          </div>
          <div class="card-body">
            <div v-if="unitResult" class="unit-result">
              <span class="unit-value">{{ unitValue }}{{ unitFrom }}</span>
              <span class="unit-arrow">=</span>
              <span class="unit-value">{{ unitResult }}{{ unitTo }}</span>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: CSS压缩 -->
      <el-tab-pane label="CSS压缩" name="compress">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleCssCompress">压缩</el-button>
                  <el-button size="small" @click="handleCssFormat">格式化</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('compress')">清空</el-button>
              <el-button size="small" @click="handlePaste('compress')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="tabState.compress.input" type="textarea" :rows="8" placeholder="请输入 CSS 代码..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('compress')">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="tabState.compress.output" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': tabState.compress.isError }" />
            <div v-if="tabState.compress.error" class="error-message">{{ tabState.compress.error }}</div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import { parseColor, convertUnit, compressCss, formatCss } from '@/utils/cssUtils'

const store = useToolboxStore()

const activeTab = ref('color')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  color: { input: '', output: '', error: '', isError: false },
  unit: { input: '', output: '', error: '', isError: false },
  compress: { input: '', output: '', error: '', isError: false }
})

const colorResult = ref<{ hex: string; rgb: string; hsl: string } | null>(null)

const unitValue = ref(16)
const unitFrom = ref('px')
const unitTo = ref('rem')
const baseFontSize = ref(16)
const unitResult = ref<number | null>(null)

const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'color') colorResult.value = null
  if (tab === 'unit') unitResult.value = null
}

const handlePaste = async (tab: string) => {
  try {
    const text = await navigator.clipboard.readText()
    tabState[tab].input = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (tab: string) => {
  const text = tab === 'color'
    ? (colorResult.value ? `HEX: ${colorResult.value.hex}\nRGB: ${colorResult.value.rgb}\nHSL: ${colorResult.value.hsl}` : '')
    : tab === 'unit'
    ? (unitResult.value !== null ? `${unitValue.value}${unitFrom.value} = ${unitResult.value}${unitTo.value}` : '')
    : tabState[tab].output

  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleColorConvert = () => {
  const input = tabState.color.input.trim()
  if (!input) {
    ElMessage.warning('请输入颜色值')
    return
  }

  const result = parseColor(input)
  if (result) {
    colorResult.value = result
    tabState.color.error = ''
    tabState.color.isError = false
    store.addHistory({ tool: 'css', action: '颜色转换', inputPreview: input, outputPreview: result.hex })
    ElMessage.success('转换成功')
  } else {
    colorResult.value = null
    tabState.color.error = '无法识别的颜色格式'
    tabState.color.isError = true
    ElMessage.error('无法识别的颜色格式')
  }
}

const handleUnitConvert = () => {
  const result = convertUnit(unitValue.value, unitFrom.value, unitTo.value, baseFontSize.value)
  if (result !== null) {
    unitResult.value = parseFloat(result.toFixed(4))
    ElMessage.success('转换成功')
  } else {
    unitResult.value = null
    ElMessage.error('不支持的单位')
  }
}

const handleCssCompress = () => {
  const input = tabState.compress.input.trim()
  if (!input) {
    ElMessage.warning('请输入 CSS 代码')
    return
  }
  tabState.compress.output = compressCss(input)
  tabState.compress.error = ''
  tabState.compress.isError = false
  store.addHistory({ tool: 'css', action: 'CSS压缩', inputPreview: input.slice(0, 50), outputPreview: tabState.compress.output.slice(0, 50) })
  ElMessage.success('压缩完成')
}

const handleCssFormat = () => {
  const input = tabState.compress.input.trim()
  if (!input) {
    ElMessage.warning('请输入 CSS 代码')
    return
  }
  tabState.compress.output = formatCss(input)
  tabState.compress.error = ''
  tabState.compress.isError = false
  store.addHistory({ tool: 'css', action: 'CSS格式化', inputPreview: input.slice(0, 50), outputPreview: tabState.compress.output.slice(0, 50) })
  ElMessage.success('格式化完成')
}

// 自动执行（颜色转换）
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(() => tabState[activeTab.value].input, (val) => {
  if (activeTab.value !== 'color' || !val.trim()) return
  if (autoExecTimer) clearTimeout(autoExecTimer)
  autoExecTimer = setTimeout(() => {
    handleColorConvert()
  }, 300)
})
</script>

<style scoped>
.css-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .css-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.css-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.css-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.css-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.css-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.css-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}

.color-results { display: flex; flex-direction: column; gap: 12px; }
.color-preview { height: 60px; border-radius: 8px; border: 1px solid var(--border-color); }
.color-item { display: flex; align-items: center; gap: 8px; }
.color-label { font-weight: 600; color: var(--accent-cyan); min-width: 50px; }
.color-item code {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-input);
  padding: 4px 8px;
  border-radius: 4px;
}

.unit-result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 20px 0;
  font-size: 24px;
}
.unit-value { font-weight: 600; color: var(--text-primary); }
.unit-arrow { color: var(--accent-cyan); }
</style>
```

---

### Task 3: JWT解析页面 (JwtTool.vue)

**Files:**
- Create: `src/views/JwtTool.vue`

- [ ] **Step 1: 创建 JwtTool.vue 页面**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 解析 JWT token 的 Header 和 Payload</p>
                <p>• 自动检测过期时间并高亮</p>
                <p>• 支持标准 Base64 和 Base64URL</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleParse">解析</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="4" placeholder="请输入 JWT token..." resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">Header</span>
        <el-button v-if="header" size="small" @click="handleCopy(JSON.stringify(header, null, 2))">复制</el-button>
      </div>
      <div class="card-body">
        <pre v-if="header" class="json-output">{{ JSON.stringify(header, null, 2) }}</pre>
        <div v-else class="empty-tip">解析后将在此显示 Header 信息</div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">Payload</span>
        <el-button v-if="payload" size="small" @click="handleCopy(JSON.stringify(payload, null, 2))">复制</el-button>
      </div>
      <div class="card-body">
        <div v-if="payload" class="payload-section">
          <pre class="json-output">{{ JSON.stringify(payload, null, 2) }}</pre>
          <div v-if="expInfo" class="exp-info" :class="{ 'expired': expInfo.isExpired }">
            <span class="exp-label">过期时间:</span>
            <span>{{ expInfo.datetime }}</span>
            <span class="exp-status">{{ expInfo.isExpired ? '已过期' : '有效' }}</span>
          </div>
        </div>
        <div v-else class="empty-tip">解析后将在此显示 Payload 信息</div>
      </div>
    </div>

    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { Base64 } from 'js-base64'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const header = ref<Record<string, any> | null>(null)
const payload = ref<Record<string, any> | null>(null)
const error = ref('')

const expInfo = computed(() => {
  if (!payload.value || !payload.value.exp) return null
  const expTimestamp = payload.value.exp * 1000
  const expDate = new Date(expTimestamp)
  const isExpired = Date.now() > expTimestamp
  return {
    datetime: expDate.toLocaleString('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
    }),
    isExpired
  }
})

const base64UrlDecode = (str: string): string => {
  str = str.replace(/-/g, '+').replace(/_/g, '/')
  while (str.length % 4) str += '='
  return Base64.decode(str)
}

const handleParse = () => {
  const token = input.value.trim()
  if (!token) {
    ElMessage.warning('请输入 JWT token')
    return
  }

  const parts = token.split('.')
  if (parts.length !== 3) {
    error.value = '无效的 JWT token，应包含 3 个部分（用 . 分隔）'
    header.value = null
    payload.value = null
    ElMessage.error('无效的 JWT token')
    return
  }

  try {
    header.value = JSON.parse(base64UrlDecode(parts[0]))
  } catch {
    error.value = '无法解析 Header 部分'
    header.value = null
    payload.value = null
    ElMessage.error('无法解析 Header')
    return
  }

  try {
    payload.value = JSON.parse(base64UrlDecode(parts[1]))
  } catch {
    error.value = '无法解析 Payload 部分'
    payload.value = null
    ElMessage.error('无法解析 Payload')
    return
  }

  error.value = ''
  store.addHistory({
    tool: 'jwt',
    action: 'JWT解析',
    inputPreview: token.slice(0, 30) + '...',
    outputPreview: JSON.stringify(payload.value).slice(0, 50)
  })
  ElMessage.success('解析成功')
}

const handleClear = () => {
  input.value = ''
  header.value = null
  payload.value = null
  error.value = ''
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = (text: string) => {
  navigator.clipboard.writeText(text)
  ElMessage.success('已复制到剪贴板')
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.json-output {
  background: var(--bg-input);
  padding: 12px 16px;
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.payload-section { display: flex; flex-direction: column; gap: 12px; }

.exp-info {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(0, 212, 255, 0.05);
  border: 1px solid var(--border-color);
  font-size: 13px;
}
.exp-info.expired {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--accent-red);
}
.exp-label { color: var(--text-secondary); font-weight: 500; }
.exp-status { margin-left: auto; font-weight: 600; }
.exp-info:not(.expired) .exp-status { color: #22c55e; }
.exp-info.expired .exp-status { color: var(--accent-red); }

.error-message {
  margin-top: 16px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}
</style>
```

---

### Task 4: 字数统计页面 (WordCountTool.vue)

**Files:**
- Create: `src/views/WordCountTool.vue`

- [ ] **Step 1: 创建 WordCountTool.vue 页面**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 实时统计字符数、单词数、行数</p>
                <p>• 区分中英文字数统计</p>
                <p>• 估算阅读时间</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">阅读速度</div>
            <el-select v-model="readSpeed" size="small" style="width: 120px">
              <el-option label="中文 (300字/分)" :value="300" />
              <el-option label="英文 (200词/分)" :value="200" />
              <el-option label="快速 (500字/分)" :value="500" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleCount">统计</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="8" placeholder="请输入文本..." resize="vertical" @input="handleAutoCount" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">统计结果</span>
      </div>
      <div class="card-body">
        <div v-if="stats" class="stats-grid">
          <div class="stat-item">
            <div class="stat-value">{{ stats.charCount }}</div>
            <div class="stat-label">字符数（含空格）</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.charCountNoSpace }}</div>
            <div class="stat-label">字符数（不含空格）</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.chineseCount }}</div>
            <div class="stat-label">中文字数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.englishWords }}</div>
            <div class="stat-label">英文单词数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.lineCount }}</div>
            <div class="stat-label">行数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.paragraphCount }}</div>
            <div class="stat-label">段落数</div>
          </div>
          <div class="stat-item">
            <div class="stat-value">{{ stats.readTime }}</div>
            <div class="stat-label">阅读时间</div>
          </div>
        </div>
        <div v-else class="empty-tip">输入文本后将在此显示统计结果</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const readSpeed = ref(300)

interface Stats {
  charCount: number
  charCountNoSpace: number
  chineseCount: number
  englishWords: number
  lineCount: number
  paragraphCount: number
  readTime: string
}

const stats = ref<Stats | null>(null)

const countText = (text: string): Stats => {
  const charCount = text.length
  const charCountNoSpace = text.replace(/\s/g, '').length
  const chineseCount = (text.match(/[\u4e00-\u9fa5]/g) || []).length
  const englishWords = text.trim() ? (text.match(/[a-zA-Z]+/g) || []).length : 0
  const lineCount = text ? text.split('\n').length : 0
  const paragraphCount = text ? text.split(/\n\s*\n/).filter(p => p.trim()).length : 0

  const totalWords = chineseCount + englishWords
  const minutes = Math.ceil(totalWords / readSpeed.value)
  const readTime = minutes < 1 ? '< 1分钟' : `${minutes}分钟`

  return { charCount, charCountNoSpace, chineseCount, englishWords, lineCount, paragraphCount, readTime }
}

const handleCount = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入文本')
    return
  }
  stats.value = countText(input.value)
  store.addHistory({
    tool: 'wordCount',
    action: '字数统计',
    inputPreview: input.value.slice(0, 50),
    outputPreview: `字符数: ${stats.value.charCount}`
  })
  ElMessage.success('统计完成')
}

const handleAutoCount = () => {
  if (input.value.trim()) {
    stats.value = countText(input.value)
  } else {
    stats.value = null
  }
}

const handleClear = () => {
  input.value = ''
  stats.value = null
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.stat-item {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
  transition: border-color 0.2s;
}
.stat-item:hover { border-color: rgba(0, 212, 255, 0.3); }
.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent-cyan);
  line-height: 1.2;
}
.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}
</style>
```

---

### Task 5: 集成到应用

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`
- Modify: `src/components/SidebarNav.vue`
- Modify: `package.json`

- [ ] **Step 1: 修改 store/index.ts - 添加新工具到 TOOL_LIST**

在 TOOL_LIST 数组中，`id: 'dedup'` 之后、`id: 'history'` 之前，添加以下3个工具项：

```typescript
  { id: 'css', name: 'CSS工具', icon: 'CSS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3h12l-3 9H9z"/><path d="M9 12l-2 9h10l-2-9"/></svg>`, description: '颜色转换、单位换算、CSS压缩/格式化', keywords: ['css', '颜色', '单位', '压缩'], category: 'devtools' },
  { id: 'jwt', name: 'JWT解析', icon: 'JWT', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg>`, description: '解析 JWT token，查看 Header/Payload', keywords: ['jwt', 'token', '解析', '认证'], category: 'devtools' },
  { id: 'wordCount', name: '字数统计', icon: 'ABC', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/><path d="M16 16l-2-4-2 4"/><path d="M14 13h-4"/></svg>`, description: '字符数、单词数、行数、阅读时间估算', keywords: ['字数', '统计', '字符', '单词', '行数'], category: 'devtools' },
```

- [ ] **Step 2: 修改 App.vue - 导入组件并添加路由**

在 import 区域，`DedupTool` 之后添加：

```typescript
import CssTool from '@/views/CssTool.vue'
import JwtTool from '@/views/JwtTool.vue'
import WordCountTool from '@/views/WordCountTool.vue'
```

在 template 区域，`<DedupTool v-else-if="activeTool === 'dedup'" />` 之后、`<HistoryView v-else-if="activeTool === 'history'" />` 之前添加：

```vue
        <CssTool v-else-if="activeTool === 'css'" />
        <JwtTool v-else-if="activeTool === 'jwt'" />
        <WordCountTool v-else-if="activeTool === 'wordCount'" />
```

- [ ] **Step 3: 修改 SidebarNav.vue - 更新版本号**

将第7行的 `v2.0` 改为 `v2.2`：

```vue
          <span class="app-version">v2.2</span>
```

- [ ] **Step 4: 修改 package.json - 更新版本号**

将第3行的 `"version": "2.1.0"` 改为 `"version": "2.2.0"`：

```json
  "version": "2.2.0",
```

---

### Task 6: 验证与提交

- [ ] **Step 1: 运行 TypeScript 检查**

```bash
npx vue-tsc --noEmit
```

Expected: 无错误

- [ ] **Step 2: 运行开发服务器验证**

```bash
npm run dev
```

Expected: 启动成功，可在浏览器访问

- [ ] **Step 3: 提交代码**

```bash
git add src/utils/cssUtils.ts src/views/CssTool.vue src/views/JwtTool.vue src/views/WordCountTool.vue src/store/index.ts src/App.vue src/components/SidebarNav.vue package.json
git commit -m "feat: 添加CSS工具、JWT解析、字数统计 3个新工具 (v2.2.0)"
```
