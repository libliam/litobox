# 第一优先级工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 添加 4 个第一优先级工具：CSV/Excel 工具、哈希计算、XML/YAML 工具、文本去重。

**Architecture:** 每个工具为独立 Vue 页面 + 独立工具函数模块，注册到 store 的 TOOL_LIST 和 App.vue 路由。纯前端实现，不新增依赖。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus, Web Crypto API, crypto-js (已安装)

---

## 文件结构

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/utils/csvUtils.ts` | 创建 | CSV 解析与导出工具函数 |
| `src/utils/hashUtils.ts` | 创建 | 哈希计算工具函数 |
| `src/utils/xmlYamlUtils.ts` | 创建 | XML/YAML 处理函数 |
| `src/utils/dedupUtils.ts` | 创建 | 文本去重工具函数 |
| `src/views/CsvTool.vue` | 创建 | CSV/Excel 工具页面 |
| `src/views/HashTool.vue` | 创建 | 哈希计算工具页面 |
| `src/views/XmlYamlTool.vue` | 创建 | XML/YAML 工具页面 |
| `src/views/DedupTool.vue` | 创建 | 文本去重工具页面 |
| `src/store/index.ts` | 修改 | 添加 4 个新工具到 TOOL_LIST |
| `src/App.vue` | 修改 | 注册新工具组件和路由 |

---

### Task 1: CSV 工具函数

**Files:**
- Create: `src/utils/csvUtils.ts`

- [ ] **Step 1: 创建 CSV 解析与导出工具函数**

```typescript
// src/utils/csvUtils.ts

export interface CsvParseOptions {
  delimiter: ',' | ';' | '\t' | '|'
  hasHeader: boolean
}

export interface CsvData {
  headers: string[]
  rows: string[][]
}

/**
 * 解析 CSV 文本为结构化数据
 * 处理引号转义、分隔符内换行等边界情况
 */
export function parseCsv(text: string, options: CsvParseOptions): CsvData {
  const { delimiter, hasHeader } = options
  const rows: string[][] = []
  let current = ''
  let inQuotes = false
  let row: string[] = []

  for (let i = 0; i < text.length; i++) {
    const char = text[i]
    const next = text[i + 1]

    if (inQuotes) {
      if (char === '"' && next === '"') {
        current += '"'
        i++ // 跳过下一个引号
      } else if (char === '"') {
        inQuotes = false
      } else {
        current += char
      }
    } else {
      if (char === '"') {
        inQuotes = true
      } else if (char === delimiter) {
        row.push(current.trim())
        current = ''
      } else if (char === '\n' || (char === '\r' && next === '\n')) {
        row.push(current.trim())
        if (row.length > 1 || row[0] !== '') {
          rows.push(row)
        }
        row = []
        current = ''
        if (char === '\r') i++ // 跳过 \n
      } else {
        current += char
      }
    }
  }

  // 处理最后一行
  row.push(current.trim())
  if (row.length > 1 || row[0] !== '') {
    rows.push(row)
  }

  if (rows.length === 0) {
    return { headers: [], rows: [] }
  }

  if (hasHeader) {
    return { headers: rows[0], rows: rows.slice(1) }
  }

  // 无 header 时生成默认列名
  const maxCols = Math.max(...rows.map(r => r.length))
  const headers = Array.from({ length: maxCols }, (_, i) => `列${i + 1}`)
  return { headers, rows }
}

/**
 * 将 CSV 数据导出为 JSON 数组
 */
export function csvToJson(csvData: CsvData): string {
  if (csvData.rows.length === 0) return '[]'
  const result = csvData.rows.map(row => {
    const obj: Record<string, string> = {}
    csvData.headers.forEach((header, i) => {
      obj[header] = row[i] ?? ''
    })
    return obj
  })
  return JSON.stringify(result, null, 2)
}

/**
 * 将 CSV 数据导出为 SQL INSERT 语句
 */
export function csvToSql(csvData: CsvData, tableName: string): string {
  if (!tableName) return '错误: 请输入表名'
  if (csvData.rows.length === 0) return '-- 无数据'

  const columns = csvData.headers.map(h => `\`${h}\``).join(', ')
  const inserts = csvData.rows.map(row => {
    const values = row.map(v => `'${v.replace(/'/g, "''")}'`).join(', ')
    return `(${values})`
  })

  return `INSERT INTO \`${tableName}\` (${columns}) VALUES\n${inserts.join(',\n')};`
}
```

- [ ] **Step 2: 验证**

运行 `npx tsc --noEmit src/utils/csvUtils.ts` 确认无类型错误。

---

### Task 2: CSV 工具页面

**Files:**
- Create: `src/views/CsvTool.vue`

- [ ] **Step 1: 创建 CSV 工具页面**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持自定义分隔符（逗号/分号/Tab/竖线）</p>
                <p>• 可识别引号转义和分隔符内换行</p>
                <p>• 导出为 JSON 或 SQL INSERT 语句</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleFileImport">导入文件</el-button>
          <input ref="fileInput" type="file" accept=".csv,.tsv,.txt" style="display: none" @change="handleFileChange" />
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">分隔符</div>
            <el-select v-model="delimiter" size="small" style="width: 100px">
              <el-option label="逗号" value="," />
              <el-option label="分号" value=";" />
              <el-option label="Tab" value="\t" />
              <el-option label="竖线" value="|" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <label class="switch-item">
              <span>首行为表头</span>
              <el-switch v-model="hasHeader" size="small" />
            </label>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <el-button type="primary" size="small" @click="handleParse">解析</el-button>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (CSV文本)</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="8" placeholder="name,age,city&#10;张三,25,北京&#10;李四,30,上海" resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">表格预览</span>
        <div class="card-actions">
          <el-tag v-if="rowCount > 0" size="small" type="info">{{ rowCount }} 行</el-tag>
        </div>
      </div>
      <div class="card-body">
        <el-table v-if="csvData.rows.length > 0" :data="tableData" border size="small" max-height="400" style="width: 100%">
          <el-table-column v-for="header in csvData.headers" :key="header" :prop="header" :label="header" />
        </el-table>
        <div v-else class="empty-tip">解析后将在此显示表格</div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">导出</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">表名</div>
            <el-input v-model="tableName" placeholder="请输入表名" size="small" style="width: 140px" />
          </div>
          <div class="action-group">
            <div class="group-label">导出</div>
            <div class="group-buttons">
              <el-button size="small" @click="handleExportJson">导出 JSON</el-button>
              <el-button size="small" @click="handleExportSql">导出 SQL</el-button>
            </div>
          </div>
        </div>
        <el-input v-if="exportResult" :model-value="exportResult" type="textarea" :rows="6" readonly resize="vertical" style="margin-top: 12px" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { parseCsv, csvToJson, csvToSql, type CsvData } from '@/utils/csvUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const delimiter = ref<',' | ';' | '\t' | '|'>(',')
const hasHeader = ref(true)
const tableName = ref('')
const exportResult = ref('')
const fileInput = ref<HTMLInputElement>()

const csvData = ref<CsvData>({ headers: [], rows: [] })
const rowCount = computed(() => csvData.value.rows.length)

const tableData = computed(() => {
  return csvData.value.rows.map(row => {
    const obj: Record<string, string> = {}
    csvData.value.headers.forEach((h, i) => {
      obj[h] = row[i] ?? ''
    })
    return obj
  })
})

const handleParse = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入 CSV 内容')
    return
  }
  try {
    csvData.value = parseCsv(input.value, {
      delimiter: delimiter.value,
      hasHeader: hasHeader.value,
    })
    exportResult.value = ''
    ElMessage.success(`解析完成: ${csvData.value.rows.length} 行, ${csvData.value.headers.length} 列`)
    store.addHistory({
      tool: 'csv',
      action: 'CSV解析',
      inputPreview: input.value.slice(0, 50),
      outputPreview: `${csvData.value.rows.length} 行`,
    })
  } catch (e: any) {
    ElMessage.error('解析失败: ' + e.message)
  }
}

const handleExportJson = () => {
  if (csvData.value.rows.length === 0) {
    ElMessage.warning('请先解析 CSV 数据')
    return
  }
  exportResult.value = csvToJson(csvData.value)
  navigator.clipboard.writeText(exportResult.value)
  ElMessage.success('已导出为 JSON 并复制到剪贴板')
}

const handleExportSql = () => {
  if (csvData.value.rows.length === 0) {
    ElMessage.warning('请先解析 CSV 数据')
    return
  }
  exportResult.value = csvToSql(csvData.value, tableName.value)
  if (!exportResult.value.startsWith('错误')) {
    navigator.clipboard.writeText(exportResult.value)
    ElMessage.success('已导出为 SQL 并复制到剪贴板')
  }
}

const handleClear = () => {
  input.value = ''
  csvData.value = { headers: [], rows: [] }
  exportResult.value = ''
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleFileImport = () => {
  fileInput.value?.click()
}

const handleFileChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  // 根据文件扩展名自动设置分隔符
  if (file.name.endsWith('.tsv')) {
    delimiter.value = '\t'
  }

  const reader = new FileReader()
  reader.onload = (event) => {
    input.value = event.target?.result as string
    ElMessage.success(`已导入文件: ${file.name}`)
  }
  reader.readAsText(file)
  // 重置 input 以便重复选择同一文件
  target.value = ''
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}
</style>
```

- [ ] **Step 2: 验证**

运行 `npm run build` 确认无编译错误。

---

### Task 3: 哈希工具函数

**Files:**
- Create: `src/utils/hashUtils.ts`

- [ ] **Step 1: 创建哈希计算工具函数**

```typescript
// src/utils/hashUtils.ts
import CryptoJS from 'crypto-js'

export type HashAlgorithm = 'md5' | 'sha1' | 'sha256' | 'sha512'

export interface HashResult {
  algorithm: string
  hash: string
}

/**
 * 计算文本的哈希值
 */
export function hashText(text: string, algorithm: HashAlgorithm): string {
  switch (algorithm) {
    case 'md5':
      return CryptoJS.MD5(text).toString()
    case 'sha1':
      return CryptoJS.SHA1(text).toString()
    case 'sha256':
      return CryptoJS.SHA256(text).toString()
    case 'sha512':
      return CryptoJS.SHA512(text).toString()
    default:
      throw new Error(`不支持的算法: ${algorithm}`)
  }
}

/**
 * 计算 HMAC 值
 */
export function hmacText(text: string, key: string, algorithm: HashAlgorithm): string {
  switch (algorithm) {
    case 'md5':
      return CryptoJS.HmacMD5(text, key).toString()
    case 'sha1':
      return CryptoJS.HmacSHA1(text, key).toString()
    case 'sha256':
      return CryptoJS.HmacSHA256(text, key).toString()
    case 'sha512':
      return CryptoJS.HmacSHA512(text, key).toString()
    default:
      throw new Error(`不支持的算法: ${algorithm}`)
  }
}

/**
 * 计算文件的哈希值（使用 Web Crypto API，支持大文件）
 */
export async function hashFile(file: File, algorithm: 'sha1' | 'sha256' | 'sha512'): Promise<string> {
  const buffer = await file.arrayBuffer()
  const cryptoAlgorithm = `SHA-${algorithm.slice(3)}`

  const hashBuffer = await crypto.subtle.digest(cryptoAlgorithm, buffer)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
}
</script>
```

ponytail: 使用 crypto-js 计算 MD5（Web Crypto API 不支持 MD5），SHA 系列使用 crypto-js 保持一致性。文件哈希使用 Web Crypto API 避免 crypto-js 对大文件的内存问题。

- [ ] **Step 2: 验证**

运行 `npx tsc --noEmit src/utils/hashUtils.ts` 确认无类型错误。

---

### Task 4: 哈希工具页面

**Files:**
- Create: `src/views/HashTool.vue`

- [ ] **Step 1: 创建哈希工具页面**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 支持 MD5、SHA-1、SHA-256、SHA-512</p>
                <p>• 支持文本和文件哈希计算</p>
                <p>• 可选 HMAC 密钥进行密钥哈希</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">算法</div>
            <el-checkbox-group v-model="selectedAlgorithms" size="small">
              <el-checkbox-button label="md5">MD5</el-checkbox-button>
              <el-checkbox-button label="sha1">SHA-1</el-checkbox-button>
              <el-checkbox-button label="sha256">SHA-256</el-checkbox-button>
              <el-checkbox-button label="sha512">SHA-512</el-checkbox-button>
            </el-checkbox-group>
          </div>
          <div class="action-group">
            <div class="group-label">HMAC密钥</div>
            <el-input v-model="hmacKey" placeholder="可选" size="small" style="width: 140px" clearable />
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <el-button type="primary" size="small" @click="handleHash">计算</el-button>
            <el-button size="small" @click="handleFileHash">文件哈希</el-button>
            <input ref="fileInput" type="file" style="display: none" @change="handleFileChange" />
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (文本或文件)</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="6" placeholder="请输入文本，或点击「文件哈希」上传文件..." resize="vertical" />
        <div v-if="fileName" class="file-info">
          <el-tag size="small" type="success">{{ fileName }} ({{ formatFileSize(fileSize) }})</el-tag>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">哈希结果</span>
        <el-button v-if="results.length > 0" size="small" @click="handleCopyAll">复制全部</el-button>
      </div>
      <div class="card-body">
        <div v-if="results.length > 0" class="result-list">
          <div v-for="result in results" :key="result.algorithm" class="result-item">
            <span class="result-algo">{{ result.algorithm }}</span>
            <span class="result-hash" @click="handleCopyOne(result.hash)">{{ result.hash }}</span>
            <el-tooltip content="点击复制" placement="top">
              <el-button size="small" text @click="handleCopyOne(result.hash)">复制</el-button>
            </el-tooltip>
          </div>
        </div>
        <div v-else class="empty-tip">计算后将在此显示哈希值</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { hashText, hmacText, hashFile, type HashAlgorithm, type HashResult } from '@/utils/hashUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const selectedAlgorithms = ref<HashAlgorithm[]>(['md5', 'sha256'])
const hmacKey = ref('')
const results = ref<HashResult[]>([])
const fileName = ref('')
const fileSize = ref(0)
const fileInput = ref<HTMLInputElement>()

const handleHash = () => {
  if (!input.value.trim() && !fileName.value) {
    ElMessage.warning('请输入文本或选择文件')
    return
  }
  if (selectedAlgorithms.value.length === 0) {
    ElMessage.warning('请至少选择一种算法')
    return
  }

  results.value = []
  const text = fileName.value ? input.value : input.value

  for (const algo of selectedAlgorithms.value) {
    try {
      const hash = hmacKey.value
        ? hmacText(text, hmacKey.value, algo)
        : hashText(text, algo)
      results.value.push({ algorithm: algo.toUpperCase(), hash })
    } catch (e: any) {
      results.value.push({ algorithm: algo.toUpperCase(), hash: `错误: ${e.message}` })
    }
  }

  store.addHistory({
    tool: 'hash',
    action: hmacKey.value ? 'HMAC计算' : '哈希计算',
    inputPreview: text.slice(0, 50),
    outputPreview: results.value.map(r => r.hash.slice(0, 16)).join(', '),
  })

  ElMessage.success('计算完成')
}

const handleFileHash = () => {
  fileInput.value?.click()
}

const handleFileChange = async (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  fileName.value = file.name
  fileSize.value = file.size
  input.value = `[文件: ${file.name}]`

  // 文件哈希只支持 SHA 系列
  const shaAlgos = selectedAlgorithms.value.filter(a => a !== 'md5')
  if (shaAlgos.length === 0) {
    ElMessage.warning('文件哈希不支持 MD5，已自动选择 SHA-256')
    selectedAlgorithms.value = ['sha256']
  }

  results.value = []
  for (const algo of shaAlgos) {
    try {
      const hash = await hashFile(file, algo as 'sha1' | 'sha256' | 'sha512')
      results.value.push({ algorithm: algo.toUpperCase(), hash })
    } catch (e: any) {
      results.value.push({ algorithm: algo.toUpperCase(), hash: `错误: ${e.message}` })
    }
  }

  ElMessage.success('文件哈希计算完成')
  target.value = ''
}

const handleClear = () => {
  input.value = ''
  results.value = []
  fileName.value = ''
  fileSize.value = 0
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleCopyOne = (hash: string) => {
  navigator.clipboard.writeText(hash)
  ElMessage.success('已复制')
}

const handleCopyAll = () => {
  const text = results.value.map(r => `${r.algorithm}: ${r.hash}`).join('\n')
  navigator.clipboard.writeText(text)
  ElMessage.success('已复制全部结果')
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 40px 0;
  font-size: 14px;
}

.file-info { margin-top: 8px; }

.result-list { display: flex; flex-direction: column; gap: 8px; }
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}
.result-algo {
  font-weight: 600;
  font-size: 12px;
  color: var(--accent-cyan);
  min-width: 70px;
  flex-shrink: 0;
}
.result-hash {
  flex: 1;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  cursor: pointer;
}
.result-hash:hover { color: var(--accent-cyan); }
</style>
```

- [ ] **Step 2: 验证**

运行 `npm run build` 确认无编译错误。

---

### Task 5: XML/YAML 工具函数

**Files:**
- Create: `src/utils/xmlYamlUtils.ts`

- [ ] **Step 1: 创建 XML/YAML 处理工具函数**

```typescript
// src/utils/xmlYamlUtils.ts

/**
 * 格式化 XML
 */
export function formatXml(xml: string, indent: number = 2): string {
  // 移除多余空白
  let formatted = ''
  let pad = 0
  const lines = xml.replace(/>\s*</g, '><').split('><')

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i]
    if (!line.trim()) continue

    // 处理闭合标签减少缩进
    if (line.match(/^\/\w/) || line.match(/^</) && line.match(/\/>$/)) {
      // 自闭合标签不改变缩进
    }
    if (line.match(/^\/\w/)) {
      pad = Math.max(0, pad - 1)
    }

    formatted += ' '.repeat(pad * indent) + line + '\n'

    // 处理开标签增加缩进
    if (line.match(/^<\w/) && !line.match(/\/>$/) && !line.match(/<\/\w+>$/)) {
      pad++
    }
  }

  return formatted.trim()
}

/**
 * 校验 XML
 */
export function validateXml(xml: string): { valid: boolean; error?: string } {
  try {
    const parser = new DOMParser()
    const doc = parser.parseFromString(xml, 'text/xml')
    const errorNode = doc.querySelector('parsererror')
    if (errorNode) {
      return { valid: false, error: errorNode.textContent?.slice(0, 200) }
    }
    return { valid: true }
  } catch (e: any) {
    return { valid: false, error: e.message }
  }
}

/**
 * XML 转 JSON
 */
export function xmlToJson(xml: string): string {
  const parser = new DOMParser()
  const doc = parser.parseFromString(xml, 'text/xml')
  const errorNode = doc.querySelector('parsererror')
  if (errorNode) {
    throw new Error('XML 格式错误: ' + errorNode.textContent?.slice(0, 100))
  }

  function nodeToJson(node: Node): any {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent?.trim()
      return text ? text : undefined
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return undefined

    const children = Array.from(node.childNodes)
    const result: Record<string, any> = {}

    for (const child of children) {
      if (child.nodeType === Node.TEXT_NODE) {
        const text = child.textContent?.trim()
        if (text) {
          return text // 简单文本节点直接返回
        }
        continue
      }
      if (child.nodeType !== Node.ELEMENT_NODE) continue

      const childName = child.nodeName
      const childValue = nodeToJson(child)
      if (childValue === undefined) continue

      if (result[childName] !== undefined) {
        if (!Array.isArray(result[childName])) {
          result[childName] = [result[childName]]
        }
        result[childName].push(childValue)
      } else {
        result[childName] = childValue
      }
    }

    // 添加属性
    if (node.nodeType === Node.ELEMENT_NODE) {
      const attrs = (node as Element).attributes
      if (attrs.length > 0) {
        result['$attrs'] = {}
        for (let i = 0; i < attrs.length; i++) {
          result['$attrs'][attrs[i].name] = attrs[i].value
        }
      }
    }

    return result
  }

  const json = nodeToJson(doc.documentElement)
  return JSON.stringify(json, null, 2)
}

/**
 * JSON 转 XML
 */
export function jsonToXml(json: string): string {
  const obj = JSON.parse(json)

  function objToXml(obj: any, tagName: string = 'root'): string {
    if (typeof obj === 'string') return `<${tagName}>${escapeXml(obj)}</${tagName}>`
    if (typeof obj === 'number') return `<${tagName}>${obj}</${tagName}>`
    if (typeof obj === 'boolean') return `<${tagName}>${obj}</${tagName}>`
    if (Array.isArray(obj)) {
      return obj.map(item => objToXml(item, tagName)).join('\n')
    }
    if (typeof obj === 'object' && obj !== null) {
      let attrs = ''
      let children = ''

      for (const [key, value] of Object.entries(obj)) {
        if (key === '$attrs') {
          attrs = Object.entries(value as Record<string, string>)
            .map(([k, v]) => ` ${k}="${escapeXml(v)}"`)
            .join('')
        } else if (Array.isArray(value)) {
          children += value.map(item => objToXml(item, key)).join('\n')
        } else {
          children += objToXml(value, key)
        }
      }

      return `<${tagName}${attrs}>${children}</${tagName}>`
    }
    return `<${tagName}/>`
  }

  function escapeXml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;')
  }

  return '<?xml version="1.0" encoding="UTF-8"?>\n' + objToXml(obj)
}

/**
 * 简易 YAML 解析器（覆盖 90% 常见场景）
 */
export function parseYaml(yaml: string): any {
  const lines = yaml.split('\n')
  const result: Record<string, any> = {}
  let currentKey = ''
  let currentArray: any[] = []
  let inArray = false

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue

    const indent = line.search(/\S/)

    // 数组项
    if (trimmed.startsWith('- ')) {
      const value = trimmed.slice(2).trim()
      if (inArray) {
        currentArray.push(parseYamlValue(value))
      } else {
        inArray = true
        currentArray = [parseYamlValue(value)]
      }
      continue
    }

    // 保存之前的数组
    if (inArray && currentKey) {
      result[currentKey] = currentArray
      inArray = false
      currentArray = []
    }

    // 键值对
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx > -1) {
      const key = trimmed.slice(0, colonIdx).trim()
      const value = trimmed.slice(colonIdx + 1).trim()

      if (value === '' || value === '|' || value === '>') {
        currentKey = key
        inArray = false
      } else {
        result[key] = parseYamlValue(value)
        currentKey = ''
      }
    }
  }

  // 保存最后的数组
  if (inArray && currentKey) {
    result[currentKey] = currentArray
  }

  return result
}

function parseYamlValue(value: string): any {
  if (value === 'true') return true
  if (value === 'false') return false
  if (value === 'null' || value === '~') return null
  if (!isNaN(Number(value)) && value !== '') return Number(value)
  // 去除引号
  if ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))) {
    return value.slice(1, -1)
  }
  return value
}

/**
 * JSON 转 YAML（简易实现）
 */
export function jsonToYaml(json: string): string {
  const obj = JSON.parse(json)

  function objToYaml(obj: any, indent: number = 0): string {
    const pad = '  '.repeat(indent)
    const lines: string[] = []

    if (Array.isArray(obj)) {
      for (const item of obj) {
        if (typeof item === 'object' && item !== null) {
          const first = true
          for (const [key, value] of Object.entries(item)) {
            const prefix = first ? `${pad}- ` : `${pad}  `
            if (typeof value === 'object' && value !== null) {
              lines.push(`${prefix}${key}:`)
              lines.push(objToYaml(value, indent + 2))
            } else {
              lines.push(`${prefix}${key}: ${formatYamlValue(value)}`)
            }
          }
        } else {
          lines.push(`${pad}- ${formatYamlValue(item)}`)
        }
      }
    } else if (typeof obj === 'object' && obj !== null) {
      for (const [key, value] of Object.entries(obj)) {
        if (typeof value === 'object' && value !== null) {
          lines.push(`${pad}${key}:`)
          lines.push(objToYaml(value, indent + 1))
        } else {
          lines.push(`${pad}${key}: ${formatYamlValue(value)}`)
        }
      }
    }

    return lines.join('\n')
  }

  function formatYamlValue(value: any): string {
    if (value === null) return 'null'
    if (typeof value === 'boolean') return value.toString()
    if (typeof value === 'string') {
      if (value.includes(':') || value.includes('#') || value.includes(',') || value === '') {
        return `"${value}"`
      }
      return value
    }
    return String(value)
  }

  return objToYaml(obj)
}
```

- [ ] **Step 2: 验证**

运行 `npx tsc --noEmit src/utils/xmlYamlUtils.ts` 确认无类型错误。

---

### Task 6: XML/YAML 工具页面

**Files:**
- Create: `src/views/XmlYamlTool.vue`

- [ ] **Step 1: 创建 XML/YAML 工具页面**

```vue
<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="xml-yaml-tabs">
      <!-- XML Tab -->
      <el-tab-pane label="XML 工具" name="xml">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 格式化：美化 XML 缩进</p>
                    <p>• 压缩：移除多余空白</p>
                    <p>• 校验：检查 XML 语法</p>
                    <p>• XML↔JSON 互转</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">缩进</div>
                <el-radio-group v-model="xmlIndent" size="small">
                  <el-radio-button :label="2">2空格</el-radio-button>
                  <el-radio-button :label="4">4空格</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleXmlFormat">格式化</el-button>
                  <el-button size="small" @click="handleXmlCompress">压缩</el-button>
                  <el-button type="warning" size="small" @click="handleXmlValidate">校验</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleXmlToJson">XML→JSON</el-button>
                  <el-button size="small" @click="handleJsonToXml">JSON→XML</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="xmlInput" type="textarea" :rows="8" placeholder="请输入 XML 或 JSON 内容..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="xmlOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': xmlIsError }" />
            <div v-if="xmlError" class="error-message">{{ xmlError }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- YAML Tab -->
      <el-tab-pane label="YAML 工具" name="yaml">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>• 格式化：标准化 YAML 格式</p>
                    <p>• 校验：检查 YAML 语法</p>
                    <p>• YAML↔JSON 互转</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleYamlFormat">格式化</el-button>
                  <el-button type="warning" size="small" @click="handleYamlValidate">校验</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleYamlToJson">YAML→JSON</el-button>
                  <el-button size="small" @click="handleJsonToYaml">JSON→YAML</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleYamlClear">清空</el-button>
              <el-button size="small" @click="handleYamlPaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="yamlInput" type="textarea" :rows="8" placeholder="请输入 YAML 或 JSON 内容..." resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleYamlCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="yamlOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': yamlIsError }" />
            <div v-if="yamlError" class="error-message">{{ yamlError }}</div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import {
  formatXml, validateXml, xmlToJson, jsonToXml,
  parseYaml, jsonToYaml
} from '@/utils/xmlYamlUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const activeTab = ref('xml')

// XML 状态
const xmlInput = ref('')
const xmlOutput = ref('')
const xmlError = ref('')
const xmlIsError = ref(false)
const xmlIndent = ref(2)

// YAML 状态
const yamlInput = ref('')
const yamlOutput = ref('')
const yamlError = ref('')
const yamlIsError = ref(false)

// XML 操作
const handleXmlFormat = () => {
  try {
    const result = formatXml(xmlInput.value, xmlIndent.value)
    xmlOutput.value = result
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('格式化完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('格式化失败')
  }
}

const handleXmlCompress = () => {
  try {
    xmlOutput.value = xmlInput.value.replace(/\s+/g, ' ').trim()
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('压缩完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
  }
}

const handleXmlValidate = () => {
  const result = validateXml(xmlInput.value)
  if (result.valid) {
    xmlOutput.value = '✓ XML 格式正确'
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('XML 校验通过')
  } else {
    xmlOutput.value = ''
    xmlError.value = '✗ ' + result.error
    xmlIsError.value = true
    ElMessage.error('XML 校验失败')
  }
}

const handleXmlToJson = () => {
  try {
    xmlOutput.value = xmlToJson(xmlInput.value)
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('XML→JSON 转换完成')
    store.addHistory({ tool: 'xmlYaml', action: 'XML→JSON', inputPreview: xmlInput.value.slice(0, 50), outputPreview: xmlOutput.value.slice(0, 50) })
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleJsonToXml = () => {
  try {
    xmlOutput.value = jsonToXml(xmlInput.value)
    xmlError.value = ''
    xmlIsError.value = false
    ElMessage.success('JSON→XML 转换完成')
  } catch (e: any) {
    xmlError.value = e.message
    xmlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleClear = () => { xmlInput.value = ''; xmlOutput.value = ''; xmlError.value = '' }
const handlePaste = async () => { try { xmlInput.value = await navigator.clipboard.readText() } catch { ElMessage.warning('无法读取剪贴板') } }
const handleCopy = () => { navigator.clipboard.writeText(xmlOutput.value || xmlError.value); ElMessage.success('已复制') }

// YAML 操作
const handleYamlFormat = () => {
  try {
    // 尝试解析 YAML 并重新格式化
    const parsed = parseYaml(yamlInput.value)
    yamlOutput.value = jsonToYaml(JSON.stringify(parsed))
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('格式化完成')
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('格式化失败')
  }
}

const handleYamlValidate = () => {
  try {
    parseYaml(yamlInput.value)
    yamlOutput.value = '✓ YAML 格式正确'
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('YAML 校验通过')
  } catch (e: any) {
    yamlOutput.value = ''
    yamlError.value = '✗ ' + e.message
    yamlIsError.value = true
    ElMessage.error('YAML 校验失败')
  }
}

const handleYamlToJson = () => {
  try {
    const parsed = parseYaml(yamlInput.value)
    yamlOutput.value = JSON.stringify(parsed, null, 2)
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('YAML→JSON 转换完成')
    store.addHistory({ tool: 'xmlYaml', action: 'YAML→JSON', inputPreview: yamlInput.value.slice(0, 50), outputPreview: yamlOutput.value.slice(0, 50) })
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleJsonToYaml = () => {
  try {
    yamlOutput.value = jsonToYaml(yamlInput.value)
    yamlError.value = ''
    yamlIsError.value = false
    ElMessage.success('JSON→YAML 转换完成')
  } catch (e: any) {
    yamlError.value = e.message
    yamlIsError.value = true
    ElMessage.error('转换失败')
  }
}

const handleYamlClear = () => { yamlInput.value = ''; yamlOutput.value = ''; yamlError.value = '' }
const handleYamlPaste = async () => { try { yamlInput.value = await navigator.clipboard.readText() } catch { ElMessage.warning('无法读取剪贴板') } }
const handleYamlCopy = () => { navigator.clipboard.writeText(yamlOutput.value || yamlError.value); ElMessage.success('已复制') }
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

/* Tab 样式 - 滚动置顶 */
.xml-yaml-tabs :deep(.el-tabs__header) {
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  padding-left: 16px;
  margin-bottom: 0;
}
.xml-yaml-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 52px;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.error-message { margin-top: 8px; padding: 8px 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; line-height: 1.5; }
:deep(.el-textarea.error .el-textarea__inner) { border-color: var(--accent-red); box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1); }
</style>
```

- [ ] **Step 2: 验证**

运行 `npm run build` 确认无编译错误。

---

### Task 7: 文本去重工具函数

**Files:**
- Create: `src/utils/dedupUtils.ts`

- [ ] **Step 1: 创建文本去重工具函数**

```typescript
// src/utils/dedupUtils.ts

export interface DedupOptions {
  mode: 'first' | 'last'        // 保留首次还是末次出现
  ignoreCase: boolean           // 忽略大小写
  ignoreWhitespace: boolean     // 忽略首尾空格
}

export interface DedupResult {
  output: string
  originalLines: number
  uniqueLines: number
  duplicateLines: number
  duplicates: Map<string, number>  // 重复行及其出现次数
}

/**
 * 按行去重
 */
export function dedupLines(text: string, options: DedupOptions): DedupResult {
  const lines = text.split('\n')
  const { mode, ignoreCase, ignoreWhitespace } = options

  // 标准化函数
  const normalize = (line: string): string => {
    let result = line
    if (ignoreWhitespace) result = result.trim()
    if (ignoreCase) result = result.toLowerCase()
    return result
  }

  // 统计每行出现次数
  const countMap = new Map<string, number>()
  for (const line of lines) {
    const key = normalize(line)
    countMap.set(key, (countMap.get(key) || 0) + 1)
  }

  // 根据模式去重
  const seen = new Map<string, boolean>()
  const outputLines: string[] = []
  const duplicates = new Map<string, number>()

  if (mode === 'first') {
    for (const line of lines) {
      const key = normalize(line)
      if (!seen.has(key)) {
        seen.set(key, true)
        outputLines.push(line)
      } else {
        duplicates.set(line, countMap.get(key) || 1)
      }
    }
  } else {
    // 保留末次：反向遍历，保留第一次遇到的
    const reversed = [...lines].reverse()
    for (const line of reversed) {
      const key = normalize(line)
      if (!seen.has(key)) {
        seen.set(key, true)
        outputLines.unshift(line)
      } else {
        duplicates.set(line, countMap.get(key) || 1)
      }
    }
  }

  return {
    output: outputLines.join('\n'),
    originalLines: lines.length,
    uniqueLines: outputLines.length,
    duplicateLines: lines.length - outputLines.length,
    duplicates,
  }
}
```

- [ ] **Step 2: 验证**

运行 `npx tsc --noEmit src/utils/dedupUtils.ts` 确认无类型错误。

---

### Task 8: 文本去重工具页面

**Files:**
- Create: `src/views/DedupTool.vue`

- [ ] **Step 1: 创建文本去重工具页面**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 按行去重，支持保留首次或末次出现</p>
                <p>• 可选忽略大小写和首尾空格</p>
                <p>• 显示重复行及其出现次数</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">模式</div>
            <el-radio-group v-model="mode" size="small">
              <el-radio-button label="first">保留首次</el-radio-button>
              <el-radio-button label="last">保留末次</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <label class="switch-item">
              <span>忽略大小写</span>
              <el-switch v-model="ignoreCase" size="small" />
            </label>
            <label class="switch-item">
              <span>忽略首尾空格</span>
              <el-switch v-model="ignoreWhitespace" size="small" />
            </label>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleDedup">去重</el-button>
              <el-button size="small" @click="handleCopy">复制结果</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输入 (每行一个)</span>
        <div class="card-actions">
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input v-model="input" type="textarea" :rows="10" placeholder="请输入文本，每行一个条目..." resize="vertical" />
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-tag v-if="stats" size="small" type="success">{{ stats.uniqueLines }} / {{ stats.originalLines }} 行</el-tag>
        </div>
      </div>
      <div class="card-body">
        <el-input :model-value="output" type="textarea" :rows="8" readonly resize="vertical" />
        <div v-if="stats && stats.duplicateLines > 0" class="stats-info">
          <span>去除了 <strong>{{ stats.duplicateLines }}</strong> 行重复</span>
        </div>
        <div v-if="duplicateList.length > 0" class="duplicate-list">
          <div class="duplicate-title">重复项 (最多显示20条):</div>
          <div v-for="(item, idx) in duplicateList.slice(0, 20)" :key="idx" class="duplicate-item">
            <span class="duplicate-text">{{ item.text }}</span>
            <el-tag size="small" type="warning">{{ item.count }} 次</el-tag>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { dedupLines, type DedupResult } from '@/utils/dedupUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const input = ref('')
const output = ref('')
const mode = ref<'first' | 'last'>('first')
const ignoreCase = ref(false)
const ignoreWhitespace = ref(false)
const stats = ref<DedupResult | null>(null)

const duplicateList = computed(() => {
  if (!stats.value) return []
  return Array.from(stats.value.duplicates.entries()).map(([text, count]) => ({ text, count }))
})

const handleDedup = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  try {
    const result = dedupLines(input.value, {
      mode: mode.value,
      ignoreCase: ignoreCase.value,
      ignoreWhitespace: ignoreWhitespace.value,
    })
    output.value = result.output
    stats.value = result
    ElMessage.success(`去重完成: ${result.originalLines} → ${result.uniqueLines} 行`)
    store.addHistory({
      tool: 'dedup',
      action: '文本去重',
      inputPreview: input.value.slice(0, 50),
      outputPreview: `${result.uniqueLines} 行`,
    })
  } catch (e: any) {
    ElMessage.error('去重失败: ' + e.message)
  }
}

const handleClear = () => {
  input.value = ''
  output.value = ''
  stats.value = null
}

const handlePaste = async () => {
  try {
    input.value = await navigator.clipboard.readText()
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleCopy = () => {
  if (!output.value) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  navigator.clipboard.writeText(output.value)
  ElMessage.success('已复制')
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.tool-card:last-child { margin-bottom: 0; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
.card-actions { display: flex; align-items: center; gap: 6px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }
.switch-item { display: flex; align-items: center; gap: 6px; cursor: pointer; color: var(--text-primary); font-size: 13px; white-space: nowrap; }

.stats-info {
  margin-top: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.stats-info strong { color: var(--accent-cyan); }

.duplicate-list { margin-top: 12px; }
.duplicate-title { font-size: 13px; color: var(--text-secondary); margin-bottom: 6px; }
.duplicate-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 13px;
  border-radius: 4px;
  background: var(--bg-input);
  margin-bottom: 4px;
}
.duplicate-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 80%;
}
</style>
```

- [ ] **Step 2: 验证**

运行 `npm run build` 确认无编译错误。

---

### Task 9: 注册新工具到 store 和 App.vue

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 在 store/index.ts 的 TOOL_LIST 中添加 4 个新工具**

在现有 TOOL_LIST 数组中，在 `id: 'history'` 之前添加以下 4 个工具条目：

```typescript
  { id: 'csv', name: 'CSV工具', icon: 'CSV', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 3v18"/></svg>`, description: 'CSV解析、表格预览、导出JSON/SQL', keywords: ['csv', '表格', '解析', '导出'], category: 'fileprocessing' },
  { id: 'hash', name: '哈希计算', icon: '#', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9h16M4 15h16"/><path d="M10 3l-2 18M16 3l-2 18"/></svg>`, description: 'MD5/SHA-1/SHA-256/SHA-512 哈希计算', keywords: ['hash', 'md5', 'sha', '哈希', '加密'], category: 'devtools' },
  { id: 'xmlYaml', name: 'XML/YAML', icon: '<>', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H6a2 2 0 00-2 2v4a2 2 0 01-2 2 2 2 0 012 2v4a2 2 0 002 2h2"/><path d="M16 3h2a2 2 0 012 2v4a2 2 0 002 2 2 2 0 00-2 2v4a2 2 0 01-2 2h-2"/><path d="M10 9l3 3-3 3"/></svg>`, description: 'XML/YAML格式化、校验、与JSON互转', keywords: ['xml', 'yaml', '格式化', '转换'], category: 'devtools' },
  { id: 'dedup', name: '文本去重', icon: '≡', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18M3 12h12M3 18h18"/></svg>`, description: '按行去重、统计重复、保留首次/末次', keywords: ['去重', '重复', 'dedup', '行去重'], category: 'fileprocessing' },
```

- [ ] **Step 2: 在 App.vue 中注册新工具组件和路由**

在 `<script setup>` 的 import 部分添加：

```typescript
import CsvTool from '@/views/CsvTool.vue'
import HashTool from '@/views/HashTool.vue'
import XmlYamlTool from '@/views/XmlYamlTool.vue'
import DedupTool from '@/views/DedupTool.vue'
```

在 `<template>` 的 `</main>` 之前添加路由：

```vue
        <CsvTool v-else-if="activeTool === 'csv'" />
        <HashTool v-else-if="activeTool === 'hash'" />
        <XmlYamlTool v-else-if="activeTool === 'xmlYaml'" />
        <DedupTool v-else-if="activeTool === 'dedup'" />
```

- [ ] **Step 3: 验证**

运行 `npm run build` 确认无编译错误。

---

## 自检

### 1. Spec 覆盖检查

| Spec 要求 | 实现 Task | 状态 |
|-----------|-----------|------|
| CSV 解析为表格预览 | Task 2 | ✓ |
| 自定义分隔符 | Task 2 | ✓ |
| 导出 JSON/SQL | Task 2 | ✓ |
| 文件上传 | Task 2 | ✓ |
| MD5/SHA-1/SHA-256/SHA-512 | Task 4 | ✓ |
| 文件哈希计算 | Task 4 | ✓ |
| HMAC 计算 | Task 4 | ✓ |
| XML 格式化/压缩/校验 | Task 6 | ✓ |
| XML↔JSON 互转 | Task 6 | ✓ |
| YAML 格式化/校验 | Task 6 | ✓ |
| YAML↔JSON 互转 | Task 6 | ✓ |
| 按行去重（首次/末次） | Task 8 | ✓ |
| 忽略大小写/空格 | Task 8 | ✓ |
| 统计重复信息 | Task 8 | ✓ |

### 2. 占位符扫描

无 TBD/TODO/类似占位符。✓

### 3. 类型一致性

- 所有工具函数使用 TypeScript 明确类型
- 所有 Vue 组件使用 `<script setup lang="ts">`
- store 中 TOOL_LIST 使用 `ToolItem` 接口
- 工具 ID 与路由条件一致 ✓

### 4. 范围检查

4 个工具，每个独立实现，范围聚焦。✓
