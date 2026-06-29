# Task 9: StringTool restoreFromHistory

**Files:**
- Modify: `src/views/StringTool.vue`

## Steps

### Step 1: Import HistoryRestoreState and add onMounted check

Add import for `HistoryRestoreState` and `onMounted`:
```typescript
import { ref, onMounted } from 'vue'
import { useToolboxStore, type HistoryRestoreState } from '@/store'
```

Add `onMounted` hook:

```typescript
onMounted(() => {
  if (store.pendingHistoryRestore?.tool === 'string') {
    restoreFromHistory(store.pendingHistoryRestore)
    store.clearHistoryRestore()
  }
})
```

### Step 2: Implement restoreFromHistory function

Add after `handleCopyBatchAll` function:

```typescript
const restoreFromHistory = (data: HistoryRestoreState) => {
  // 填充输入框
  inputValue.value = data.input
  // 填充输出框（不重新执行）
  outputValue.value = data.output
  // 还原配置
  if (data.options?.activeTab) {
    activeTab.value = data.options.activeTab
  }
  if (data.options?.separator !== undefined) {
    separator.value = data.options.separator
  }
  // 显示提示
  ElMessage({
    message: `已加载历史记录（${new Date(data.timestamp).toLocaleString('zh-CN')} 的操作）`,
    type: 'info',
    duration: 3000,
  })
}
```

### Step 3: Verify TypeScript

Run: `npx tsc --noEmit`
Expected: zero errors
