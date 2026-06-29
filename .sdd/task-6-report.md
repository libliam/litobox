# Task 6 Report: 修改 store.addHistory 同时写入 details 表

## What was implemented

1. **Modified `HistoryRecord` interface** (`src/store/index.ts`): Added three optional fields:
   - `inputFull?: string`
   - `outputFull?: string`
   - `options?: Record<string, any>`

2. **Modified `addHistory` function** (`src/store/index.ts`): After saving to the history table, conditionally writes to the `history_details` table when any of `inputFull`, `outputFull`, or `options` is provided. Uses `db.addHistoryDetail` from dbClient.ts.

## What was tested and test results

- `npx tsc --noEmit` passed with zero errors.
- Verified `db.addHistoryDetail` signature in `src/utils/dbClient.ts` matches the call: `history_id: number`, `input_full: string | null`, `output_full: string | null`, `options_json: string`.

## Files changed

- `src/store/index.ts` (+14 lines)

## Self-review findings

- No issues found. The conditional guard (`record.inputFull !== undefined || record.outputFull !== undefined || record.options`) ensures existing callers that don't pass full data are unaffected.
- `options_json` uses `JSON.stringify(record.options || {})` which safely handles `undefined`/`null` options by writing `{}`.
- The local state update (`history.value.unshift`) remains outside the try/catch, preserving the existing pattern.

## Any issues or concerns

- None.
