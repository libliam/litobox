# 表格识别导出CSV 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在OcrTool中新增表格识别Tab，基于OCR坐标信息推断表格结构，输出CSV并支持表格预览。

**Architecture:** 在ocrUtils.ts中添加detectTable()、toCsv()、recognizeTable()三个纯函数，在OcrTool.vue中使用el-tabs包裹现有内容和新增表格识别Tab，表格识别Tab包含图片上传、CSV输出、el-table预览。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus el-tabs/el-table, @paddleocr/paddleocr-js

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/utils/ocrUtils.ts` | 修改 | 新增 `detectTable()`、`toCsv()`、`recognizeTable()` |
| `src/views/OcrTool.vue` | 修改 | 添加el-tabs包裹现有内容和新增表格识别Tab |

---

### Task 1: 新增表格识别工具函数

**Files:**
- Modify: `src/utils/ocrUtils.ts`

- [ ] **Step 1: 添加 detectTable 函数**

在 `src/utils/ocrUtils.ts` 末尾（getMergedResult之后）添加：

```typescript
/**
 * 按Y坐标分组文字块（行检测）
 */
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

/**
 * 检测列边界（X坐标聚类）
 */
function detectColumns(rows: OcrResult[][], tolerance: number): number[] {
  const xCoords: number[] = []
  for (const row of rows) {
    for (const item of row) {
      const x = Math.min(...item.box.map(p => p[0]))
      xCoords.push(x)
    }
  }

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

/**
 * 从OCR结果推断表格结构
 * @param ocrResults OCR识别结果数组
 * @param rowTolerance 行检测容差（px），默认5
 * @param colTolerance 列检测容差（px），默认10
 * @returns 二维字符串数组，表示表格内容
 */
export function detectTable(
  ocrResults: OcrResult[],
  rowTolerance: number = 5,
  colTolerance: number = 10
): string[][] {
  if (ocrResults.length === 0) return []

  // 1. 按Y坐标分组（行检测）
  const rows = groupByY(ocrResults, rowTolerance)

  // 2. 检测列边界
  const columns = detectColumns(rows, colTolerance)

  // 3. 构建表格
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

- [ ] **Step 2: 添加 toCsv 函数**

```typescript
/**
 * 将二维数组转为CSV字符串
 * @param table 二维字符串数组
 * @returns CSV格式字符串
 */
export function toCsv(table: string[][]): string {
  return table
    .map(row =>
      row
        .map(cell => {
          // 处理包含逗号、引号、换行符的单元格
          if (cell.includes(',') || cell.includes('"') || cell.includes('\n')) {
            return `"${cell.replace(/"/g, '""')}"`
          }
          return cell
        })
        .join(',')
    )
    .join('\n')
}
```

- [ ] **Step 3: 添加 recognizeTable 函数**

```typescript
/**
 * 识别图片中的表格
 * @param image 图片文件/Blob
 * @returns 二维字符串数组（表格内容）
 */
export async function recognizeTable(image: Blob | File): Promise<string[][]> {
  const engine = await initOcr()
  const results = await engine.predict(image)

  if (results.length === 0) {
    throw new Error('未识别到文字，请检查图片是否清晰')
  }

  return detectTable(results)
}
```

- [ ] **Step 4: 验证代码无语法错误**

运行 `npx tsc --noEmit` 确认类型检查通过。

---

### Task 2: 修改OcrTool.vue - 添加Tab结构

**Files:**
- Modify: `src/views/OcrTool.vue`

- [ ] **Step 1: 在模板中添加el-tabs包裹**

找到 `<div class="tool-container">` 下的第一个 `.tool-card`，在其前面添加：

```vue
<el-tabs v-model="activeTab" class="ocr-tabs">
  <el-tab-pane label="文字识别" name="text">
    <!-- 现有所有内容（操作卡片、图片预览、识别结果、历史记录） -->
  </el-tab-pane>
  <el-tab-pane label="表格识别" name="table">
    <!-- 表格识别Tab内容 -->
  </el-tab-pane>
</el-tabs>
```

将现有的所有 `.tool-card` 移入第一个 `<el-tab-pane>` 中。

- [ ] **Step 2: 添加activeTab状态**

在 `<script setup>` 中添加：

```typescript
const activeTab = ref('text')
```

- [ ] **Step 3: 添加Tab样式**

在 `<style scoped>` 末尾添加：

```css
/* Tab样式 */
.ocr-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.ocr-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}
```

---

### Task 3: 新增表格识别Tab内容

**Files:**
- Modify: `src/views/OcrTool.vue`

- [ ] **Step 1: 在第二个el-tab-pane中添加表格识别UI**

```vue
<el-tab-pane label="表格识别" name="table">
  <div class="tool-container">
    <!-- 操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>识别图片中的表格，输出CSV格式</p>
                <p>支持 PNG/JPG/WebP/BMP 格式图片</p>
                <p>• 点击「上传文件」选择本地图片</p>
                <p>• 使用 Ctrl+V 粘贴剪贴板中的图片</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-actions">
          <el-tag v-if="isTableRecognizing" size="small" type="info">识别中...</el-tag>
          <el-button size="small" @click="handleClearTable">清除结果</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">图片输入</span>
            <div class="group-buttons">
              <el-button size="small" type="primary" @click="triggerTableFileInput">
                上传文件
              </el-button>
              <el-button size="small" @click="handleTablePaste">
                粘贴剪贴板
              </el-button>
            </div>
          </div>
        </div>
        <input
          ref="tableFileInputRef"
          type="file"
          accept="image/png,image/jpeg,image/webp,image/bmp"
          style="display: none"
          @change="handleTableFileSelect"
        />
      </div>
    </div>

    <!-- 图片预览卡片 -->
    <div v-if="tableImagePreview" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片预览</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClearTableImage">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="image-preview-container">
          <img :src="tableImagePreview" alt="预览图片" class="preview-image" />
        </div>
      </div>
    </div>

    <!-- 识别结果卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">识别结果</span>
        </div>
        <div class="card-actions">
          <el-button size="small" :disabled="!tableCsvText" @click="handleCopyTableCsv">复制CSV</el-button>
          <el-button size="small" :disabled="!tableCsvText" @click="handleExportTableCsv">导出CSV</el-button>
        </div>
      </div>
      <div class="card-body" v-loading="isTableRecognizing" element-loading-text="正在识别表格...">
        <el-input
          v-model="tableCsvText"
          type="textarea"
          :rows="8"
          placeholder="上传图片后，自动识别表格并输出CSV..."
          readonly
          class="result-textarea"
        />
        <div v-if="tableError" class="error-message">{{ tableError }}</div>
      </div>
    </div>

    <!-- 表格预览卡片 -->
    <div v-if="tableData.length > 0" class="tool-card">
      <div class="card-header">
        <span class="card-title">表格预览</span>
      </div>
      <div class="card-body">
        <el-table :data="tableDisplayData" border stripe size="small" class="table-preview">
          <el-table-column
            v-for="(header, idx) in tableHeaders"
            :key="idx"
            :prop="'col' + idx"
            :label="header"
            min-width="100"
          />
        </el-table>
      </div>
    </div>
  </div>
</el-tab-pane>
```

- [ ] **Step 2: 添加表格识别相关状态**

在 `<script setup>` 中添加：

```typescript
// 表格识别状态
const tableFileInputRef = ref<HTMLInputElement | null>(null)
const tableImagePreview = ref<string>('')
const tableCsvText = ref('')
const tableError = ref('')
const isTableRecognizing = ref(false)
const tableData = ref<string[][]>([])

// 表格显示数据
const tableHeaders = computed(() => {
  if (tableData.value.length === 0) return []
  const colCount = tableData.value[0].length
  return Array.from({ length: colCount }, (_, i) => `列${i + 1}`)
})

const tableDisplayData = computed(() => {
  return tableData.value.map((row, idx) => {
    const obj: Record<string, string> = { id: idx.toString() }
    row.forEach((cell, colIdx) => {
      obj['col' + colIdx] = cell
    })
    return obj
  })
})
```

- [ ] **Step 3: 添加表格识别相关函数**

```typescript
// 表格识别相关函数
const triggerTableFileInput = () => {
  tableFileInputRef.value?.click()
}

const handleTableFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await processTableImage(file)
  input.value = ''
}

const handleTablePaste = async () => {
  try {
    const items = await navigator.clipboard.read()
    for (const item of items) {
      for (const type of item.types) {
        if (type.startsWith('image/')) {
          const blob = await item.getType(type)
          await processTableImage(blob)
          return
        }
      }
    }
    ElMessage.warning('剪贴板中没有图片')
  } catch {
    ElMessage.warning('无法读取剪贴板，请确保已复制图片')
  }
}

const processTableImage = async (blob: Blob | File) => {
  tableError.value = ''
  tableImagePreview.value = URL.createObjectURL(blob)
  isTableRecognizing.value = true

  try {
    const table = await recognizeTable(blob)
    tableData.value = table
    tableCsvText.value = toCsv(table)

    // 添加到全局历史
    store.addHistory({
      tool: 'ocr',
      action: '表格识别',
      inputPreview: '[表格图片]',
      outputPreview: tableCsvText.value.substring(0, 100)
    })

    ElMessage.success(`表格识别完成，${table.length}行${table[0]?.length || 0}列`)
  } catch (e: any) {
    tableError.value = e.message || '表格识别失败'
  } finally {
    isTableRecognizing.value = false
  }
}

const handleClearTable = () => {
  tableCsvText.value = ''
  tableError.value = ''
  tableData.value = []
}

const handleClearTableImage = () => {
  tableImagePreview.value = ''
  if (tableFileInputRef.value) tableFileInputRef.value.value = ''
}

const handleCopyTableCsv = async () => {
  try {
    await navigator.clipboard.writeText(tableCsvText.value)
    ElMessage.success('已复制CSV')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportTableCsv = async () => {
  const blob = new Blob([tableCsvText.value], { type: 'text/csv;charset=utf-8' })
  await saveFileWithDialog(blob, 'table-result.csv', 'csv')
}
```

- [ ] **Step 4: 添加fileSaver导入**

在import区域添加：

```typescript
import { saveFileWithDialog } from '@/utils/fileSaver'
```

- [ ] **Step 5: 添加表格预览样式**

在 `<style scoped>` 末尾添加：

```css
/* 表格预览 */
.table-preview {
  font-size: 13px;
}
:deep(.el-table th) {
  background: var(--bg-input);
  color: var(--accent-cyan);
  font-weight: 600;
}
:deep(.el-table td) {
  color: var(--text-primary);
}
:deep(.el-table--border) {
  border-color: var(--border-color);
}
```

---

### Task 4: 验证和测试

**Files:**
- No file changes

- [ ] **Step 1: 类型检查**

运行：`npx tsc --noEmit`
预期：无错误

- [ ] **Step 2: 启动开发服务器**

运行：`npm run dev`
预期：无编译错误

- [ ] **Step 3: 功能测试清单**

| 测试场景 | 预期结果 |
|----------|----------|
| 切换到表格识别Tab | 显示操作区，无报错 |
| 上传表格图片 | 显示预览，识别后输出CSV |
| CSV输出 | 格式正确，逗号/引号转义正确 |
| 表格预览 | el-table正确显示行列结构 |
| 复制CSV | 复制到剪贴板成功 |
| 导出CSV | 下载CSV文件成功 |
| 空表格图片 | 显示错误提示"未识别到文字" |
| 非表格图片 | 输出单行/单列结果（算法降级） |
| 切换Tab | 文字识别Tab功能不受影响 |

---

## 自审检查

- [x] 所有需求都有对应Task实现
- [x] 无TBD/TODO占位符
- [x] 类型定义在各Task中保持一致（OcrResult接口复用）
- [x] 每个Task可独立编译验证
- [x] 遵循现有代码风格（Composition API + script setup）
- [x] 文字识别Tab功能完全兼容
- [x] 错误处理覆盖主要场景
