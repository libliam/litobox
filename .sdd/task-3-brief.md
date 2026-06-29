# Task 3: 前端 dbClient 封装

**Files:**
- Modify: `src/utils/dbClient.ts`

## Steps

### Step 1: 新增 `HistoryDetail` 接口和调用函数

在 `searchHistory` 函数之后添加：

```typescript
export interface HistoryDetail {
  id?: number;
  history_id: number;
  input_full: string | null;
  output_full: string | null;
  options_json: string;
  created_at?: string;
}

export async function addHistoryDetail(detail: HistoryDetail): Promise<number> {
  return invoke('cmd_db_add_history_detail', { detail });
}

export async function getHistoryDetail(historyId: number): Promise<HistoryDetail | null> {
  return invoke('cmd_db_get_history_detail', { historyId });
}

export async function deleteHistoryDetailsForHistory(historyId: number): Promise<void> {
  return invoke('cmd_db_delete_history_details_for_history', { historyId });
}
```
