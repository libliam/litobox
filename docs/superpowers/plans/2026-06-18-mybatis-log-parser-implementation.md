# MyBatis 日志解析功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增第 9 个 Tab「日志解析」，从 MyBatis 日志中提取 SQL 和参数，替换 `?` 占位符生成可执行的完整 SQL。

**Architecture:** 新建 `mybatisLogParser.ts` 工具函数，按行解析日志文本，提取 `Preparing:` 和 `Parameters:` 行，按类型规则格式化参数后替换到 SQL 模板中。在 `SqlTool.vue` 中新增第 9 个 Tab 调用该工具函数。

**Tech Stack:** TypeScript, Vue 3 Composition API, Element Plus

---

## 文件结构

- **新建:** `src/utils/mybatisLogParser.ts` — 日志解析核心逻辑（纯函数）
- **修改:** `src/views/SqlTool.vue` — 新增第 9 个 Tab + 引入工具函数 + 添加处理方法

---

### Task 1: 创建 mybatisLogParser.ts

**Files:**
- Create: `src/utils/mybatisLogParser.ts`

- [ ] **Step 1: 定义类型和常量**

```typescript
// 参数类型枚举
export type ParamType = 'String' | 'Integer' | 'Long' | 'Double' | 'Float' | 'Boolean' | 'Date' | 'null' | 'Unknown'

// 解析结果接口
export interface ParseResult {
  success: boolean
  sql?: string
  error?: string
}

// 类型关键词映射
const STRING_TYPES = new Set(['String', 'VARCHAR', 'TEXT', 'CHAR', 'NVARCHAR', 'CLOB'])
const NUMBER_TYPES = new Set(['Integer', 'Long', 'Short', 'Byte', 'Double', 'Float', 'BigDecimal', 'BigInteger', 'int', 'long', 'short', 'byte', 'double', 'float'])
const BOOLEAN_TYPES = new Set(['Boolean', 'boolean'])
const DATE_TYPES = new Set(['Date', 'Timestamp', 'LocalDateTime', 'LocalDate', 'Time', 'java.util.Date', 'java.sql.Timestamp', 'java.sql.Date'])
```

- [ ] **Step 2: 实现参数格式化函数**

```typescript
// 根据类型格式化参数值
function formatParamValue(value: string, type: string): string {
  // null 类型
  if (type === 'null' || value.toLowerCase() === 'null') {
    return 'NULL'
  }

  // 布尔类型
  if (BOOLEAN_TYPES.has(type)) {
    return value.toLowerCase() === 'true' ? 'true' : 'false'
  }

  // 数字类型
  if (NUMBER_TYPES.has(type)) {
    return value
  }

  // 日期类型
  if (DATE_TYPES.has(type)) {
    return `'${value}'`
  }

  // 字符串类型
  if (STRING_TYPES.has(type)) {
    return `'${value}'`
  }

  // 未知类型默认加引号
  return `'${value}'`
}
```

- [ ] **Step 3: 实现参数行解析函数**

```typescript
// 解析 Parameters 行，提取参数列表
// 输入示例: "Parameters: alertEnable(String), 0(Integer), null"
// 输出: [{ value: 'alertEnable', type: 'String' }, { value: '0', type: 'Integer' }, { value: 'null', type: 'null' }]
function parseParameters(paramsLine: string): Array<{ value: string; type: string }> {
  const params: Array<{ value: string; type: string }> = []
  // 去掉 "Parameters:" 前缀
  const content = paramsLine.replace(/^Parameters:\s*/, '').trim()
  if (!content) return params

  // 按逗号分割，但要注意括号内的逗号不能分割
  const regex = /([^,]+?)\(([^)]+)\)|\s*([^,]+?)\s*(?=,|$)/g
  let match
  while ((match = regex.exec(content)) !== null) {
    if (match[1] && match[2]) {
      // 匹配到 value(Type) 格式
      params.push({ value: match[1].trim(), type: match[2].trim() })
    } else if (match[3]) {
      // 匹配到单独的 null 等
      const val = match[3].trim()
      if (val) {
        params.push({ value: val, type: 'null' })
      }
    }
  }
  return params
}
```

- [ ] **Step 4: 实现日志解析主函数**

```typescript
// 从 MyBatis 日志中解析 SQL 和参数，生成完整 SQL
export function parseMybatisLog(logText: string): ParseResult {
  const lines = logText.split('\n').map(line => line.trim()).filter(line => line.length > 0)

  // 查找 Preparing 行
  const preparingLine = lines.find(line => line.startsWith('Preparing:'))
  if (!preparingLine) {
    return { success: false, error: '未找到 Preparing 语句，请确保日志包含 MyBatis SQL 日志' }
  }

  // 提取 SQL 模板（去掉 "Preparing: " 前缀）
  let sqlTemplate = preparingLine.replace(/^Preparing:\s*/, '').trim()

  // 查找 Parameters 行
  const paramsLine = lines.find(line => line.startsWith('Parameters:'))
  if (!paramsLine) {
    return { success: false, error: '未找到 Parameters 参数行' }
  }

  // 解析参数
  const params = parseParameters(paramsLine)
  if (params.length === 0) {
    return { success: false, error: '未解析到任何参数' }
  }

  // 替换占位符
  let paramIndex = 0
  const resultSql = sqlTemplate.replace(/\?/g, () => {
    if (paramIndex >= params.length) {
      return '?' // 参数不足，保留原样
    }
    const param = params[paramIndex++]
    return formatParamValue(param.value, param.type)
  })

  return { success: true, sql: resultSql }
}
```

- [ ] **Step 5: 提交**

```bash
git add src/utils/mybatisLogParser.ts
git commit -m "feat: 添加 MyBatis 日志解析工具函数"
```

---

### Task 2: 改造 SqlTool.vue — 新增日志解析 Tab

**Files:**
- Modify: `src/views/SqlTool.vue`

- [ ] **Step 1: 在 script 中导入工具函数**

在现有的 import 语句后添加：

```typescript
import { parseMybatisLog } from '@/utils/mybatisLogParser'
```

- [ ] **Step 2: 在 template 中添加第 9 个 Tab**

在 `</el-tabs>` 闭合标签前、Tab 8（快捷操作）之后添加：

```vue
      <!-- Tab 9: 日志解析 -->
      <el-tab-pane label="日志解析" name="logParse">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleParseLog">解析日志</el-button>
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
            <el-input v-model="inputValue" type="textarea" :rows="8" placeholder="请粘贴 MyBatis 日志内容..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="outputValue" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': isError }" />
            <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
          </div>
        </div>
      </el-tab-pane>
```

- [ ] **Step 3: 添加日志解析处理方法**

在 script 部分，快捷操作相关方法之后添加：

```typescript
// Tab 9: 日志解析
const handleParseLog = () => {
  if (!inputValue.value.trim()) {
    ElMessage.warning('请粘贴 MyBatis 日志内容')
    return
  }
  const result = parseMybatisLog(inputValue.value)
  if (result.success) {
    outputValue.value = result.sql || ''
    errorMessage.value = ''
    isError.value = false
    addHistory('日志解析')
    ElMessage.success('解析成功')
  } else {
    outputValue.value = ''
    errorMessage.value = `错误: ${result.error}`
    isError.value = true
  }
}
```

- [ ] **Step 4: 提交**

```bash
git add src/views/SqlTool.vue
git commit -m "feat: SQL工具箱新增日志解析Tab"
```

---

### Task 3: 更新 README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 更新功能特性列表**

将 SQL 工具箱描述从：
```
- **SQL工具箱** - 格式化美化/压缩单行化/语法校验/JSON转Insert/字段格式互转/注释批量操作/SQL IN转换/快捷模板
```
改为：
```
- **SQL工具箱** - 格式化美化/压缩单行化/语法校验/JSON转Insert/字段格式互转/注释批量操作/SQL IN转换/快捷模板/MyBatis日志解析
```

- [ ] **Step 2: 更新版本规划**

将 V1.2 中的 SQL 工具箱描述从：
```
- SQL工具箱（格式化/压缩/校验/JSON转Insert/字段转换/注释操作/SQL IN/快捷模板）
```
改为：
```
- SQL工具箱（格式化/压缩/校验/JSON转Insert/字段转换/注释操作/SQL IN/快捷模板/日志解析）
```

- [ ] **Step 3: 提交**

```bash
git add README.md
git commit -m "docs: 更新README，添加MyBatis日志解析功能说明"
```

---

## 自审

### 1. 规范覆盖检查

| 规范项 | 对应 Task |
|--------|----------|
| 新建 mybatisLogParser.ts | Task 1 |
| 支持 SELECT/INSERT/UPDATE/DELETE | Task 1 Step 4（不区分 SQL 类型，统一处理） |
| 参数类型自动识别加引号 | Task 1 Step 2 |
| 过滤噪声行 | Task 1 Step 4（只匹配 Preparing/Parameters 行） |
| 新增第 9 个 Tab | Task 2 |
| 更新 README | Task 3 |

### 2. 占位符扫描
无 TBD/TODO，所有代码已完整写出。

### 3. 类型一致性
- `ParseResult` 接口在 Task 1 定义，Task 2 中直接使用
- `parseMybatisLog` 函数签名一致
- 所有类型定义与使用匹配
