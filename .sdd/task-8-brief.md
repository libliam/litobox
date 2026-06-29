# Task 8: JsonTool restoreFromHistory

**Files:**
- Modify: `src/views/JsonTool.vue`

## Steps

### Step 1: Import HistoryRestoreState and add onMounted check

Add import for `HistoryRestoreState`:
```typescript
import { useToolboxStore, type HistoryRestoreState } from '@/store'
```

Add `onMounted` hook (after existing imports, the store is already imported):

```typescript
import { onMounted } from 'vue'

onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'json') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})
```

### Step 2: Implement restoreFromHistory function

Add after `handleCopy` function:

```typescript
const restoreFromHistory = (data: HistoryRestoreState) => {
  // 填充输入框
  inputValue.value = data.input
  // 填充输出框（不重新执行）
  outputValue.value = data.output
  // 还原配置
  if (data.options?.indentSize !== undefined) {
    indentSize.value = data.options.indentSize
  }
  // 显示提示
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}
```

Note: JsonTool doesn't have tabs, so no activeTab restoration needed. The only config is `indentSize`.

### Step 3: Verify TypeScript

Run: `npx tsc --noEmit`
Expected: zero errors
