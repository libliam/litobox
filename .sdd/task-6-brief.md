# Task 6: 修改 store.addHistory 同时写入 details 表

**Files:**
- Modify: `src/store/index.ts`

## Steps

### Step 1: 修改 `HistoryRecord` 接口（前端类型）

Change the existing `HistoryRecord` interface (around line 14-20) to:

```typescript
export interface HistoryRecord {
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
    // 同步更新本地状态
    history.value.unshift(newRecord)
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY)
    }
    // 保存到 SQLite
    try {
      const historyId = await db.addHistory({
        tool: newRecord.tool,
        action: newRecord.action,
        input_preview: newRecord.inputPreview,
        output_preview: newRecord.outputPreview,
      })

      // 如果有完整数据，写入 details 表
      if (record.inputFull !== undefined || record.outputFull !== undefined || record.options) {
        await db.addHistoryDetail({
          history_id: historyId,
          input_full: record.inputFull ?? null,
          output_full: record.outputFull ?? null,
          options_json: JSON.stringify(record.options || {}),
        })
      }
    } catch (error) {
      console.error('保存历史失败:', error)
    }
  }
```

Note: Existing tool pages call `addHistory` without `inputFull`/`outputFull`/`options`, which is fine since they're optional fields.
