# 表格识别导出CSV 设计文档

**日期:** 2026-06-25
**状态:** 已批准
**关联计划:** [2026-06-25-ocr-extensions.md](../plans/2026-06-25-ocr-extensions.md) Phase 2

---

## 目标

基于现有OCR的坐标信息（`box`字段），通过算法推断表格的行列结构，将图片中的表格识别为CSV格式，支持导出和表格预览。

---

## 需求总结

| 需求 | 决策 |
|------|------|
| 实现方案 | 方案A：基于坐标的表格推断 |
| 包体积影响 | 0（纯算法，无新增依赖） |
| UI形式 | 在OcrTool.vue中新增Tab |
| 输出格式 | CSV文本 + 表格预览（el-table） |

---

## 核心算法

### 流程

```
OCR结果（文字+坐标） 
  → 按Y坐标分组（行检测，容差5px）
  → 每行内按X坐标排序（从左到右）
  → 分析列对齐（X坐标范围聚类）
  → 生成二维数组
  → 输出CSV
```

### 行检测

- 按Y坐标排序所有文字块
- 容差范围内（默认5px）的Y坐标归为同一行
- 容差可配置（适应不同字体大小）

### 列检测

- 对每行内的文字块，按X坐标排序
- 分析所有行的X坐标分布，聚类出列边界
- 容差范围内（默认10px）的X坐标归为同一列

### 空单元格处理

- 如果某行在某列位置没有文字块，填充空字符串
- 确保每行列数一致

---

## UI结构

### Tab结构

在OcrTool.vue中使用el-tabs：
- **Tab 1: 文字识别**（现有功能，保持不变）
- **Tab 2: 表格识别**（新增）

### 表格识别Tab内容

**操作卡片：**
- 上传文件按钮
- 粘贴剪贴板按钮
- 识别按钮

**预览区：**
- 图片预览（复用现有预览组件）

**结果区：**
- CSV文本输出框（可复制/导出）
- 表格预览（el-table展示，直观查看）
- 导出CSV按钮

---

## 数据结构

```typescript
export interface OcrResult {
  text: string
  confidence: number
  box: number[][]  // [[x1,y1], [x2,y2], [x3,y3], [x4,y4]]
}
```

---

## 核心函数

### detectTable 函数

```typescript
// src/utils/ocrUtils.ts
export function detectTable(
  ocrResults: OcrResult[],
  rowTolerance: number = 5,
  colTolerance: number = 10
): string[][]
```

**参数：**
- `ocrResults`: OCR识别结果数组
- `rowTolerance`: 行检测容差（px），默认5
- `colTolerance`: 列检测容差（px），默认10

**返回：** 二维字符串数组，表示表格内容

### toCsv 函数

```typescript
export function toCsv(table: string[][]): string
```

**参数：**
- `table`: 二维字符串数组

**返回：** CSV格式字符串（处理逗号、引号转义）

### recognizeTable 函数

```typescript
export async function recognizeTable(image: Blob | File): Promise<string[][]>
```

**流程：**
1. 调用 `initOcr()` 获取引擎
2. 调用 `engine.predict(image)` 获取带坐标的OCR结果
3. 调用 `detectTable(ocrResults)` 推断表格结构
4. 返回二维数组

---

## 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `src/utils/ocrUtils.ts` | 添加 `detectTable()`、`toCsv()`、`recognizeTable()` 函数 |
| `src/views/OcrTool.vue` | 添加el-tabs包裹现有内容和新增表格识别Tab |

---

## 错误处理

- 未识别到文字：提示"未识别到文字，请检查图片是否清晰"
- 未检测到表格结构：提示"未检测到表格结构，请确保图片包含表格"
- CSV导出失败：提示"导出失败，请重试"

---

## 性能考虑

- 表格识别算法为O(n log n)（排序为主），处理1000个文字块约1-2ms
- 表格预览使用el-table虚拟滚动（如果行数>100）
- CSV生成使用字符串拼接，避免中间数组

---

## 算法细节

### 行检测实现

```typescript
function groupByY(results: OcrResult[], tolerance: number): OcrResult[][] {
  const sorted = [...results].sort((a, b) => {
    const yA = Math.min(...a.box.map(p => p[1]))
    const yB = Math.min(...b.box.map(p => p[1]))
    return yA - yB
  })
  
  const rows: OcrResult[][] = []
  for (const item of sorted) {
    const y = Math.min(...item.box.map(p => p[1]))
    const existingRow = rows.find(row => {
      const rowY = Math.min(...row[0].box.map(p => p[1]))
      return Math.abs(y - rowY) <= tolerance
    })
    if (existingRow) {
      existingRow.push(item)
    } else {
      rows.push([item])
    }
  }
  return rows
}
```

### 列检测实现

```typescript
function detectColumns(rows: OcrResult[][], tolerance: number): number[] {
  // 收集所有X坐标
  const xCoords: number[] = []
  for (const row of rows) {
    for (const item of row) {
      const x = Math.min(...item.box.map(p => p[0]))
      xCoords.push(x)
    }
  }
  
  // 聚类X坐标
  const columns: number[] = []
  const sorted = [...xCoords].sort((a, b) => a - b)
  for (const x of sorted) {
    const existingCol = columns.find(col => Math.abs(x - col) <= tolerance)
    if (!existingCol) {
      columns.push(x)
    }
  }
  return columns
}
```

### 表格生成

```typescript
function buildTable(
  rows: OcrResult[][],
  columns: number[],
  colTolerance: number
): string[][] {
  return rows.map(row => {
    const cells: string[] = new Array(columns.length).fill('')
    for (const item of row) {
      const x = Math.min(...item.box.map(p => p[0]))
      const colIndex = columns.findIndex(col => Math.abs(x - col) <= colTolerance)
      if (colIndex >= 0) {
        cells[colIndex] = item.text
      }
    }
    return cells
  })
}
```
