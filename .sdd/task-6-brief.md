# Task 6: 修改 store.addHistory 同时写入 details 表

**Files:**
- Modify: `src/store/index.ts`

## Current State

The store's `HistoryRecord` interface currently has:
```typescript
export interface HistoryRecord {
  id?: number
  tool: string
  action: string
  timestamp: string
  inputPreview: string
  outputPreview: string
}
```

The current `addHistory` function (around line 173-195) saves to the history table but NOT to the details table.

## Steps

### Step 1: 修改 `HistoryRecord` 接口

Add three optional fields:

```typescript
export interface HistoryRecord {
  id?: number
  tool: string
  action: string
  timestamp: string
  inputPreview: string
  outputPreview: string
  inputFull?: string
  outputFull?: string
  options?: Record<string, any>
}
```

### Step 2: 修改 `addHistory` 函数

Replace the existing `addHistory` function with:

```typescript
  const addHistory = async (record: Omit<HistoryRecord, 'timestamp'>) => {
    const newRecord = {
      ...record,
      timestamp: new Date().toISOString()
    }
    // 保存到 SQLite
    try {
      const id = await db.addHistory({
        tool: newRecord.tool,
        action: newRecord.action,
        input_preview: newRecord.inputPreview,
        output_preview: newRecord.outputPreview,
      })

      // 如果有完整数据，写入 details 表
      if (record.inputFull !== undefined || record.outputFull !== undefined || record.options) {
        await db.addHistoryDetail({
          history_id: id,
          input_full: record.inputFull ?? null,
          output_full: record.outputFull ?? null,
          options_json: JSON.stringify(record.options || {}),
        })
      }

      // 同步更新本地状态（带 id）
      newRecord.id = id
    } catch (error) {
      console.error('保存历史失败:', error)
    }
    history.value.unshift(newRecord)
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY)
    }
  }
```

Note: The local state update (`history.value.unshift(newRecord)`) stays AFTER the try/catch, matching the current pattern from Task 5.

### Step 3: Verify TypeScript

Run: `npx tsc --noEmit`
Expected: zero errors
