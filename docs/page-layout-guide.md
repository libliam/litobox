# LitoBox 页面布局规范

> 所有新增工具页面必须遵循此规范，保持视觉一致性。

## 一、页面类型与间距

### 1. 独立工具页面（直接在 App.vue 路由中）
如：SQL工具、字符串工具、JS工具、JSON工具、正则工具等

```vue
<style scoped>
/* 不定义 .tool-container，使用 main.css 全局样式：padding: 20px */
</style>
```

### 2. 一级导航容器页面（开发工具/文件处理）
如：DevTools.vue、FileProcessing.vue

```vue
<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 16px 16px 16px 8px;  /* 左侧 8px 与顶部菜单对齐 */
}
</style>
```

### 3. 二级工具页面（嵌套在一级导航内）
如：CryptoTool.vue、TimeTool.vue、URLTool.vue、BatchTextTool.vue、FileEncodingTool.vue

```vue
<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 0;  /* 由父容器控制间距 */
}
</style>
```

---

## 二、Tab 样式规范

### 一级 Tab（导航级）
用于开发工具/文件处理等主导航切换

```vue
<el-tabs v-model="activeTab" class="dev-tabs">
  <el-tab-pane label="编码工具" name="encode" />
  <el-tab-pane label="加解密" name="crypto" />
</el-tabs>

<style scoped>
/* 一级 Tab */
.dev-tabs {
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  padding-left: 8px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .dev-tabs {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.dev-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
}

.dev-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.dev-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.dev-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.dev-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.dev-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}
</style>
```

**关键特征：**
- sticky 置顶，z-index: 20
- 底部 box-shadow 分隔内容区
- 字号 14px，字重 500
- 左侧间距：容器 `padding-left: 8px` + nav-wrap `padding-left: 4px`

### 二级 Tab（子功能切换）
用于工具内部的子功能切换，胶囊按钮风格

```vue
<el-tabs v-model="activeTab" class="crypto-tabs">
  <el-tab-pane label="MD5" name="md5" />
  <el-tab-pane label="SHA" name="sha" />
</el-tabs>

<style scoped>
/* 二级 Tab（子功能切换） */
.crypto-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.crypto-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.crypto-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.crypto-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.crypto-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 400;
  padding: 0 12px;
  height: 28px;
  line-height: 28px;
  border-radius: 4px;
  margin-right: 4px;
  transition: all 0.2s;
}

.crypto-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.crypto-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.crypto-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .crypto-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .crypto-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .crypto-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(8, 145, 178, 0.1);
}
</style>
```

**关键特征：**
- 胶囊容器风格（圆角边框包裹）
- 无下划线指示器，用背景色区分选中态
- 字号 12px，字重 400，高度 28px
- `margin-top: -4px` 收缩顶部间距
- 左侧间距 12px（容器 padding-left）

---

## 三、卡片布局规范

### 基础卡片

```vue
<div class="tool-card">
  <div class="card-header">
    <span class="card-title">标题</span>
    <div class="card-actions">
      <el-button size="small" @click="handleClear">清空</el-button>
      <el-button size="small" @click="handleCopy">复制</el-button>
    </div>
  </div>
  <div class="card-body">
    <!-- 内容 -->
  </div>
</div>
```

### Sticky 卡片（操作栏置顶）

```vue
<div class="tool-card sticky-card">
  <div class="card-header">
    <span class="card-title">操作</span>
  </div>
  <div class="card-body">
    <!-- 操作按钮 -->
  </div>
</div>
```

### 完整样式定义

```css
/* 工具卡片 */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

/* Sticky 卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
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

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
```

---

## 四、操作按钮布局规范

### 简单按钮组

```vue
<div class="card-actions">
  <el-button size="small" @click="handleClear">清空</el-button>
  <el-button size="small" @click="handlePaste">粘贴</el-button>
</div>
```

### 分组按钮（带标签）

```vue
<div class="action-grid">
  <div class="action-group">
    <div class="group-label">转换方向</div>
    <el-radio-group v-model="mode" size="small">
      <el-radio-button label="a">选项A</el-radio-button>
      <el-radio-button label="b">选项B</el-radio-button>
    </el-radio-group>
  </div>
  <div class="action-group">
    <div class="group-label">执行</div>
    <div class="group-buttons">
      <el-button type="primary" size="small" @click="handleExecute">执行</el-button>
    </div>
  </div>
</div>
```

### 完整样式

```css
/* 操作按钮 */
.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}
```

---

## 五、标题提示图标规范

```vue
<div class="card-header">
  <div class="header-left">
    <span class="card-title">标题</span>
    <el-tooltip placement="top" effect="dark">
      <template #content>
        <div class="tooltip-content">
          <p>提示说明文字</p>
        </div>
      </template>
      <el-icon class="hint-icon"><QuestionFilled /></el-icon>
    </el-tooltip>
  </div>
</div>
```

```css
/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}
```

---

## 六、错误提示规范

```vue
<el-input :model-value="output" type="textarea" :rows="8" readonly :class="{ 'error': isError }" />
<div v-if="error" class="error-message">{{ error }}</div>
```

```css
/* 错误提示 */
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
```

---

## 七、多 Tab 状态管理规范

```typescript
import { ref, reactive, computed, watch } from 'vue'

const activeTab = ref('tab1')

// 每个 Tab 独立状态
const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  tab1: { input: '', output: '', error: '', isError: false },
  tab2: { input: '', output: '', error: '', isError: false }
})

// 当前 Tab 状态引用
const currentInput = computed({
  get: () => tabState[activeTab.value].input,
  set: (val) => { tabState[activeTab.value].input = val }
})

// 监听当前 Tab 的 input 变化（300ms 防抖自动执行）
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(() => tabState[activeTab.value].input, (val) => {
  if (!val.trim()) return
  if (autoExecTimer) clearTimeout(autoExecTimer)
  autoExecTimer = setTimeout(() => {
    autoExecute()
  }, 300)
})
```

**要点：**
- 使用 `watch(() => tabState[activeTab.value].input, ...)` 直接监听当前 Tab 的 input
- 不要用 `watchEffect`，会追踪所有响应式依赖导致混乱
- 不要用 `deep watch` 整个对象，会在 output/error 变化时循环触发

---

## 八、主题适配

所有颜色必须使用 CSS 变量，禁止硬编码色值：

| 变量 | 用途 |
|------|------|
| `var(--bg-primary)` | 页面主背景 |
| `var(--bg-card)` | 卡片背景 |
| `var(--bg-input)` | 输入框背景 |
| `var(--text-primary)` | 主文本 |
| `var(--text-secondary)` | 次文本 |
| `var(--accent-cyan)` | 青色强调 |
| `var(--border-color)` | 边框颜色 |
| `var(--accent-red)` | 错误色 |

亮色模式特殊样式使用 `html.light` 选择器：

```css
html.light .dev-tabs {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
```

---

## 九、间距速查表

| 元素 | 间距值 |
|------|--------|
| 独立工具页面容器 | `padding: 20px`（四周统一） |
| 一级导航容器 | `padding: 16px 16px 16px 8px` |
| 二级工具容器 | `padding: 0` |
| 一级 Tab 底部间距 | `margin-bottom: 16px` |
| 一级 Tab 左侧 | `padding-left: 8px` + nav-wrap `4px` |
| 二级 Tab 底部间距 | `margin-bottom: 8px` |
| 二级 Tab 顶部收缩 | `margin-top: -4px` |
| 二级 Tab 左侧 | 容器 `padding-left: 12px` |
| 卡片间距 | `margin-bottom: 16px` |
| 卡片内边距 | `padding: 16px 20px` |
| 标题栏内边距 | `padding: 16px 20px` |
| 按钮间距 | `gap: 6px` |
| 按钮组间距 | `gap: 16px` |
