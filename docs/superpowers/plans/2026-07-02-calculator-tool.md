# 计算器工具 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 添加计算器工具，覆盖表达式计算、单位换算、日期计算、时间戳转换。

**架构:** 纯前端实现，使用 `mathjs` 库进行表达式计算，单位换算和日期计算使用原生 JS。无需后端改动。

**Tech Stack:** Vue 3 (Composition API), TypeScript, mathjs

---

### Task 1: 工作流集成 — 添加 calculator 支持

**Files:**
- Modify: `src/views/WorkflowView.vue`

**Analysis:** 当前 `WorkflowView.vue` 的 `TOOL_ACTIONS` 和 `executeStep()` 中缺少 `calculator` 分支。计算器的工作流场景是：输入表达式 → 返回计算结果。需要添加 `TOOL_ACTIONS` 入口和 `executeStep` 分支。

- [ ] **Step 1: 在脚本顶部添加 mathjs import**

在 WorkflowView.vue 的 `import` 块中添加：

```ts
import { create, all } from 'mathjs'
const workflowMath = create(all, {})
```

- [ ] **Step 2: 在 TOOL_ACTIONS 中添加 calculator 入口**

```ts
const TOOL_ACTIONS: Record<string, string[]> = {
  // ... 现有工具
  calculator: ['表达式计算'],
}
```

- [ ] **Step 3: 在 executeStep 中添加 calculator 分支**

在 `executeStep` 函数的 `switch (tool)` 中添加：

```ts
case 'calculator':
  return executeCalculatorAction(action, input)
```

- [ ] **Step 4: 添加 executeCalculatorAction 函数**

在 `executeSqlAction` 函数后面添加：

```ts
// 计算器执行 — 复用 mathjs
function executeCalculatorAction(_action: string, input: string): string {
  if (!input.trim()) return ''
  try {
    const result = workflowMath.evaluate(input.trim())
    return String(result)
  } catch (e: any) {
    return `计算错误: ${e.message}`
  }
}
```

- [ ] **Step 5: 在步骤编辑器的工具下拉列表中添加计算器选项**

在 `editingSteps` 的工具选择 `el-select` 中添加：

```vue
<el-option label="计算器" value="calculator" />
```

- [ ] **Step 5: 验证**

运行 `npm run build` 确认无编译错误。手动测试：创建新工作流，添加计算器步骤，输入表达式，确认输出正确结果。

---

### Task 2: 完善 CalculatorTool.vue — VariablePicker 集成

**Files:**
- Modify: `src/views/CalculatorTool.vue`

**Analysis:** 当前表达式输入区没有集成 `VariablePicker` 组件（工作流变量选择器），需要添加。

- [ ] **Step 1: 在表达式卡片 header 的 card-actions 中添加 VariablePicker**

```vue
<!-- 表达式卡片的 card-actions -->
<div class="card-actions">
  <VariablePicker @select="handleVarSelect" />
  <el-button size="small" @click="calcInput = ''; calcResult = null; calcError = ''">清空</el-button>
  <el-button size="small" @click="handlePaste">粘贴</el-button>
</div>
```

- [ ] **Step 2: 添加 handleVarSelect 和 handlePaste 方法**

```ts
const handleVarSelect = (varName: string) => {
  calcInput.value += varName
}

const handlePaste = async () => {
  try {
    calcInput.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}
```

- [ ] **Step 3: 引入 VariablePicker 组件**

```ts
import VariablePicker from '@/components/VariablePicker.vue'
```

---

### Task 3: 最终验证

- [ ] **Step 1: 编译检查**

Run: `npm run build`
Expected: 编译成功，无错误

- [ ] **Step 2: 功能验证清单**
  - 表达式计算: 输入 `3 * (4 + 5) / 2` → 输出 `13.5`
  - 三角函数: `sin(pi/2)` → `1`
  - 单位换算: 1 m → cm → `100`
  - 温度换算: 100 °C → °F → `212`
  - 日期差: 2026-01-01 ~ 2026-12-31 → 相差 364 天
  - 日期加减: 2026-07-02 + 30 天 → `2026-08-01`
  - 时间戳转换: 时间戳 → 可读日期
  - 工作流: 添加计算器步骤，输入 `2 + 3` → 输出 `5`
