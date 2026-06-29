# Task 4: Pinia Store 新增状态传递

**Files:**
- Modify: `src/store/index.ts`

## Steps

### Step 1: 新增 `HistoryRestoreState` 接口和状态

在 `HistoryRecord` 接口之后添加：

```typescript
export interface HistoryRestoreState {
  tool: string
  input: string
  output: string
  options: Record<string, any>
  timestamp: string
}
```

In the store function, after `const history = ref<HistoryRecord[]>([])` add:

```typescript
  const pendingHistoryRestore = ref<HistoryRestoreState | null>(null)
  let restoreTimeout: ReturnType<typeof setTimeout> | null = null

  const triggerHistoryRestore = (data: HistoryRestoreState) => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = data
    // 30 秒未消费自动清除
    restoreTimeout = setTimeout(() => {
      pendingHistoryRestore.value = null
    }, 30000)
  }

  const clearHistoryRestore = () => {
    if (restoreTimeout) clearTimeout(restoreTimeout)
    pendingHistoryRestore.value = null
  }
```

### Step 2: 导出新方法

In the return statement of the store, add:

```typescript
    pendingHistoryRestore,
    triggerHistoryRestore,
    clearHistoryRestore,
```
