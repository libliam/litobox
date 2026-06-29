# Task 5: HistoryView 双击跳转逻辑

**Files:**
- Modify: `src/views/HistoryView.vue`

## Steps

### Step 1: 模板改造 — 添加双击事件和 tooltip

Change `<div class="history-item">` to:

```vue
<div
  v-for="(record, index) in filteredHistory"
  :key="index"
  class="history-item"
  @dblclick="handleJumpToTool(record)"
  :title="'双击跳转到对应工具'"
>
```

### Step 2: 新增 `handleJumpToTool` 函数

In `<script setup>`, after `handleClear` function, add:

```typescript
import * as db from '@/utils/dbClient'

const handleJumpToTool = async (record: any) => {
  // 检查工具是否在导航列表中
  const toolIds = store.tools.map(t => t.id)
  if (!toolIds.includes(record.tool)) {
    ElMessage.warning('该工具当前不可用')
    return
  }

  const loading = ElLoading.service({
    lock: true,
    text: '正在加载历史数据...',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  try {
    // 获取完整数据
    const detail = await db.getHistoryDetail(record.id)

    store.triggerHistoryRestore({
      tool: record.tool,
      input: detail?.input_full || record.inputPreview || '',
      output: detail?.output_full || record.outputPreview || '',
      options: detail ? JSON.parse(detail.options_json || '{}') : {},
      timestamp: record.timestamp,
    })

    // 切换页面
    store.activeTool = record.tool
    ElMessage.success('已加载历史记录，输入和输出已填充')
  } catch (e: any) {
    ElMessage.error('加载失败: ' + (e.message || e))
  } finally {
    loading.close()
  }
}
```

Note: `store.tools` refers to `TOOL_LIST` exported from the store. The store doesn't currently expose `tools` — you need to import `TOOL_LIST` from `@/store` instead. Use:

```typescript
import { useToolboxStore, TOOL_LIST } from '@/store'
```

And check: `if (!TOOL_LIST.find(t => t.id === record.tool))`

### Step 3: 样式改造 — 添加 cursor 和 hover 效果

In `<style scoped>`, add `cursor: pointer;` to `.history-item`:

```css
.history-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.3s;
  cursor: pointer;
}
```

### Step 4: 大文本提示

Add `formatPreview` function and use it in the template:

```typescript
const LARGE_TEXT_THRESHOLD = 10240

const formatPreview = (text: string): string => {
  if (!text) return ''
  if (text.length > LARGE_TEXT_THRESHOLD) {
    return '[大文本 · 双击查看]'
  }
  return text
}
```

In the template, change the preview rows to use `formatPreview`:

```vue
<div class="preview-row">
  <span class="preview-label">输入</span>
  <code class="preview-text">{{ formatPreview(record.inputPreview) }}</code>
</div>
<div class="preview-row">
  <span class="preview-label">输出</span>
  <code class="preview-text">{{ formatPreview(record.outputPreview) }}</code>
</div>
```
