# SQL 工具实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增独立的 SQL 工具页面，提供将换行分隔的字符串列表转换为 SQL IN 查询条件的功能。

**Architecture:** 创建独立的 SqlTool.vue 页面组件和 sqlUtils.ts 工具函数，注册到导航栏和 App.vue 路由中。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus

---

### Task 1: 创建 sqlUtils.ts 工具函数

**Files:**
- Create: `src/utils/sqlUtils.ts`

- [ ] **Step 1: 编写工具函数**

```typescript
export type QuoteType = 'single' | 'double' | 'none'

export function convertToSqlIn(text: string, quoteType: QuoteType = 'single'): string {
  const lines = text.split('\n')
    .map(line => line.trim())
    .filter(line => line !== '')

  if (lines.length === 0) {
    return ''
  }

  let formatted: string
  if (quoteType === 'single') {
    formatted = lines.map(line => `'${line}'`).join(',')
  } else if (quoteType === 'double') {
    formatted = lines.map(line => `"${line}"`).join(',')
  } else {
    formatted = lines.join(',')
  }

  return `(${formatted})`
}
```

- [ ] **Step 2: 验证函数逻辑**

手动测试用例：
- 输入 `"111\n222"`, quoteType=`'single'` → 输出 `('111','222')`
- 输入 `"111\n222"`, quoteType=`'double'` → 输出 `("111","222")`
- 输入 `"111\n222"`, quoteType=`'none'` → 输出 `(111,222)`
- 输入 `"  111  \n\n222  "`, quoteType=`'single'` → 输出 `('111','222')`（自动去空行和trim）
- 输入 `""` → 输出 `''`

---

### Task 2: 创建 SqlTool.vue 页面组件

**Files:**
- Create: `src/views/SqlTool.vue`

- [ ] **Step 1: 创建页面组件**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">操作</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">引号类型</div>
            <el-select v-model="quoteType" size="small" style="width: 120px">
              <el-option label="单引号" value="single" />
              <el-option label="双引号" value="double" />
              <el-option label="无引号" value="none" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">转换</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleConvert">转换为 SQL IN</el-button>
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
        <el-input
          v-model="inputValue"
          type="textarea"
          :rows="8"
          placeholder="请输入文本内容，每行一个值..."
          resize="vertical"
        />
      </div>
    </div>
    
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <el-button size="small" @click="handleCopy">复制</el-button>
      </div>
      <div class="card-body">
        <el-input
          :model-value="outputValue"
          type="textarea"
          :rows="8"
          readonly
          resize="vertical"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { convertToSqlIn, type QuoteType } from '@/utils/sqlUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const inputValue = ref('')
const outputValue = ref('')
const quoteType = ref<QuoteType>('single')

const handleConvert = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  
  try {
    const result = convertToSqlIn(inputValue.value, quoteType.value)
    outputValue.value = result
    store.addHistory({
      tool: 'sql',
      action: '转换为 SQL IN',
      inputPreview: inputValue.value.slice(0, 50),
      outputPreview: result.slice(0, 50)
    })
    ElMessage.success('转换成功')
  } catch (error) {
    ElMessage.error('转换失败')
  }
}

const handleClear = () => {
  inputValue.value = ''
  outputValue.value = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    inputValue.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败，请手动粘贴')
  }
}

const handleCopy = async () => {
  if (!outputValue.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  
  try {
    await navigator.clipboard.writeText(outputValue.value)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>
```

---

### Task 3: 注册 SQL 工具到导航和路由

**Files:**
- Modify: `src/store/index.ts` - 添加 SQL 工具到 TOOL_LIST
- Modify: `src/App.vue` - 添加 SqlTool 组件路由

- [ ] **Step 1: 修改 store/index.ts**

在 `TOOL_LIST` 数组中，`uuid` 和 `history` 之间添加：

```typescript
{ id: 'sql', name: 'SQL工具', icon: 'SQL', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16v16H4z"/><path d="M4 9h16"/><path d="M9 4v16"/></svg>`, description: '字符串列表转SQL IN查询条件', keywords: ['sql', 'in', '查询', '转换'] },
```

- [ ] **Step 2: 修改 App.vue**

在 `<script setup>` 中添加导入：

```typescript
import SqlTool from '@/views/SqlTool.vue'
```

在模板的条件渲染中，`UUIDTool` 和 `HistoryView` 之间添加：

```vue
<SqlTool v-else-if="activeTool === 'sql'" />
```

---

### Task 4: 验证与测试

- [ ] **Step 1: 启动开发服务器**

```bash
npm run dev
```

- [ ] **Step 2: 手动测试**

1. 打开应用，导航到"SQL工具"
2. 输入测试数据：
   ```
   111
   222
   333
   ```
3. 选择"单引号"，点击"转换为 SQL IN"
4. 验证输出为 `('111','222','333')`
5. 切换"双引号"，再次转换，验证输出为 `("111","222","333")`
6. 切换"无引号"，再次转换，验证输出为 `(111,222,333)`
7. 测试空输入，验证提示"请输入内容"
8. 测试复制/粘贴/清空功能

- [ ] **Step 3: 检查 TypeScript 编译**

确保无类型错误：

```bash
npx tsc --noEmit
```

---

## 自审

1. **Spec 覆盖**: 所有需求都有对应任务实现
   - sqlUtils.ts 函数 ✓
   - SqlTool.vue 页面 ✓
   - 导航注册 ✓
   - 引号类型选择 ✓
   - 错误处理 ✓

2. **占位符扫描**: 无 TBD/TODO/不完整内容

3. **类型一致性**: QuoteType 在 sqlUtils.ts 定义，SqlTool.vue 正确导入使用

4. **命令完整性**: 所有步骤包含完整代码和命令
