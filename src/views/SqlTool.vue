<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="sql-tabs">
      <!-- Tab 1: 格式化 -->
      <el-tab-pane label="格式化" name="format">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">方言</div>
                <el-select v-model="formatOptions.dialect" size="small" style="width: 120px">
                  <el-option label="MySQL" value="mysql" />
                  <el-option label="PostgreSQL" value="postgresql" />
                  <el-option label="SQLServer" value="sqlserver" />
                  <el-option label="Oracle" value="oracle" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">缩进</div>
                <el-radio-group v-model="formatOptions.indent" size="small">
                  <el-radio-button :label="2">2空格</el-radio-button>
                  <el-radio-button :label="4">4空格</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">关键字</div>
                <el-radio-group v-model="formatOptions.keywordCase" size="small">
                  <el-radio-button label="upper">大写</el-radio-button>
                  <el-radio-button label="lower">小写</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleFormat">格式化</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': currentIsError }" />
            <div v-if="currentError" class="error-message">{{ currentError }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 压缩 -->
      <el-tab-pane label="压缩" name="compress">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleCompress">一键压缩</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 校验 -->
      <el-tab-pane label="校验" name="validate">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="warning" size="small" @click="handleValidate">执行校验</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">校验结果</span>
          </div>
          <div class="card-body">
            <div v-if="validationResult" class="validation-summary">
              <span class="error-count" :class="{ 'has-errors': validationResult.errorCount > 0 }">
                错误: {{ validationResult.errorCount }}
              </span>
              <span class="warning-count">警告: {{ validationResult.warningCount }}</span>
            </div>
            <div v-if="validationResult && validationResult.errors.length > 0" class="error-list">
              <div v-for="(err, idx) in validationResult.errors" :key="idx" class="error-item" :class="err.type">
                <span class="error-location">第{{ err.line }}行, 第{{ err.column }}列</span>
                <span class="error-type">{{ err.type === 'error' ? '错误' : '警告' }}</span>
                <span class="error-msg">{{ err.message }}</span>
              </div>
            </div>
            <div v-else-if="validationResult && validationResult.errors.length === 0" class="success-message">
              ✓ 未发现语法问题
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: JSON→INSERT -->
      <el-tab-pane label="JSON→INSERT" name="jsonInsert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">表名</div>
                <el-input v-model="tableName" placeholder="请输入表名" size="small" style="width: 160px" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleJsonToInsert">生成 INSERT</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (JSON数组)</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder='[{"name":"张三","age":25},{"name":"李四","age":30}]' resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': currentIsError }" />
            <div v-if="currentError" class="error-message">{{ currentError }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 5: 字段转换 -->
      <el-tab-pane label="字段转换" name="fieldConvert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">转换方向</div>
                <el-radio-group v-model="convertMode" size="small">
                  <el-radio-button label="snakeToCamel">下划线→驼峰</el-radio-button>
                  <el-radio-button label="camelToSnake">驼峰→下划线</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleFieldConvert">转换</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (每行一个字段名)</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="user_name&#10;first_name&#10;last_name" resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 6: 注释操作 -->
      <el-tab-pane label="注释操作" name="comment">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">单行注释</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleAddLineComment">添加 --</el-button>
                  <el-button size="small" @click="handleRemoveLineComment">移除 --</el-button>
                </div>
              </div>
              <div class="action-group">
                <div class="group-label">多行注释</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleAddBlockComment">添加 /* */</el-button>
                  <el-button size="small" @click="handleRemoveBlockComment">移除 /* */</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请输入SQL语句..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 7: SQL IN (已有功能) -->
      <el-tab-pane label="SQL IN" name="sqlIn">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">引号类型</div>
                <el-select v-model="quoteType" size="small" style="width: 120px">
                  <el-option label="单引号" value="single" />
                  <el-option label="双引号" value="double" />
                  <el-option label="无引号" value="none" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">转换</div>
                <el-button type="primary" size="small" @click="handleSqlInConvert">转换为 SQL IN</el-button>
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
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请输入文本内容，每行一个值..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 8: 快捷操作 -->
      <el-tab-pane label="快捷操作" name="quick">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">SQL 模板</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">选择模板</div>
                <el-select v-model="selectedTemplate" size="small" style="width: 160px" @change="handleTemplateChange">
                  <el-option label="SELECT 模板" value="select" />
                  <el-option label="INSERT 模板" value="insert" />
                  <el-option label="UPDATE 模板" value="update" />
                  <el-option label="CREATE TABLE 模板" value="create" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">操作</div>
                <div class="group-buttons">
                  <el-button size="small" @click="handleClear">清空</el-button>
                  <el-button size="small" @click="handleCopy">复制</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <VariablePicker @select="handleInsertVariable" />
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="10" placeholder="选择模板或手动输入SQL..." resize="vertical" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 9: MyBatis解析 -->
      <el-tab-pane label="MyBatis解析" name="logParse">
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
            <el-input v-model="currentInput" type="textarea" :rows="8" placeholder="请粘贴 MyBatis 日志内容，要以Preparing开始..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': currentIsError }" />
            <div v-if="currentError" class="error-message">{{ currentError }}</div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 10: Java提取SQL -->
      <el-tab-pane label="Java提取SQL" name="javaExtract">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <el-button type="primary" size="small" @click="handleJavaExtract">提取SQL</el-button>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (Java代码)</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="currentInput" type="textarea" :rows="10" placeholder="请粘贴包含SQL定义的Java代码..." resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出 (完整SQL)</span>
            <el-button size="small" @click="handleCopy">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="currentOutput" type="textarea" :rows="8" readonly resize="vertical" :class="{ 'error': currentIsError }" />
            <div v-if="currentError" class="error-message">{{ currentError }}</div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { formatSql, type FormatOptions } from '@/utils/sqlFormatter'
import { compressSql } from '@/utils/sqlCompressor'
import { validateSql, type ValidationResult } from '@/utils/sqlValidator'
import { jsonToInsert } from '@/utils/jsonToInsert'
import { convertFields } from '@/utils/fieldConverter'
import { addLineComment, removeLineComment, addBlockComment, removeBlockComment } from '@/utils/sqlComment'
import { parseMybatisLog } from '@/utils/mybatisLogParser'
import { convertToSqlIn, type QuoteType } from '@/utils/sqlUtils'
import { extractSqlFromJava } from '@/utils/javaSqlExtractor'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()

// Tab 状态
const activeTab = ref('format')

// 每个 Tab 独立的输入/输出状态
const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  format: { input: '', output: '', error: '', isError: false },
  compress: { input: '', output: '', error: '', isError: false },
  validate: { input: '', output: '', error: '', isError: false },
  jsonInsert: { input: '', output: '', error: '', isError: false },
  fieldConvert: { input: '', output: '', error: '', isError: false },
  comment: { input: '', output: '', error: '', isError: false },
  sqlIn: { input: '', output: '', error: '', isError: false },
  quick: { input: '', output: '', error: '', isError: false },
  logParse: { input: '', output: '', error: '', isError: false },
  javaExtract: { input: '', output: '', error: '', isError: false }
})

// 当前 Tab 的状态引用
const currentInput = computed({
  get: () => tabState[activeTab.value].input,
  set: (val) => { tabState[activeTab.value].input = val }
})
const currentOutput = computed({
  get: () => tabState[activeTab.value].output,
  set: (val) => { tabState[activeTab.value].output = val }
})
const currentError = computed({
  get: () => tabState[activeTab.value].error,
  set: (val) => { tabState[activeTab.value].error = val }
})
const currentIsError = computed({
  get: () => tabState[activeTab.value].isError,
  set: (val) => { tabState[activeTab.value].isError = val }
})

// 格式化配置
const formatOptions = reactive<FormatOptions>({
  dialect: 'mysql',
  indent: 2,
  keywordCase: 'upper'
})

// JSON→INSERT 配置
const tableName = ref('')

// 字段转换配置
const convertMode = ref<'snakeToCamel' | 'camelToSnake'>('snakeToCamel')

// SQL IN 配置
const quoteType = ref<QuoteType>('single')

// 快捷操作配置
const selectedTemplate = ref('')

const SQL_TEMPLATES: Record<string, string> = {
  select: `SELECT column1, column2
FROM table_name
WHERE condition
ORDER BY column1;`,
  insert: `INSERT INTO table_name (column1, column2)
VALUES (value1, value2);`,
  update: `UPDATE table_name
SET column1 = value1, column2 = value2
WHERE condition;`,
  create: `CREATE TABLE table_name (
  id INT PRIMARY KEY AUTO_INCREMENT,
  column1 VARCHAR(255) NOT NULL,
  column2 INT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);`
}

// 通用方法
const handleClear = () => {
  currentInput.value = ''
  currentOutput.value = ''
  currentError.value = ''
  currentIsError.value = false
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    currentInput.value = text
    ElMessage.success('粘贴成功')
    // 粘贴后自动执行
    autoExecute()
  } catch {
    ElMessage.error('粘贴失败，请手动粘贴')
  }
}

const handleInsertVariable = (value: string) => {
  currentInput.value = value
  autoExecute()
}

const handleCopy = async () => {
  const text = currentOutput.value || currentInput.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

const addHistory = (action: string) => {
  store.addHistory({
    tool: 'sql',
    action,
    inputPreview: currentInput.value.slice(0, 50),
    outputPreview: currentOutput.value.slice(0, 50),
    inputFull: currentInput.value,
    outputFull: currentOutput.value,
  })
}

// 自动执行：根据当前 Tab 触发对应操作
const autoExecute = () => {
  const tab = activeTab.value
  switch (tab) {
    case 'format':
      handleFormat()
      break
    case 'compress':
      handleCompress()
      break
    case 'validate':
      handleValidate()
      break
    case 'jsonInsert':
      handleJsonToInsert()
      break
    case 'fieldConvert':
      handleFieldConvert()
      break
    case 'sqlIn':
      handleSqlInConvert()
      break
    case 'logParse':
      handleParseLog()
      break
    case 'javaExtract':
      handleJavaExtract()
      break
  }
}

// Tab 1: 格式化
const handleFormat = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  const result = formatSql(currentInput.value, formatOptions)
  if (result.success) {
    currentOutput.value = result.data || ''
    currentError.value = ''
    currentIsError.value = false
    addHistory('格式化')
    ElMessage.success('格式化成功')
  } else {
    currentOutput.value = ''
    currentError.value = `错误: ${result.error}`
    currentIsError.value = true
  }
}

// Tab 2: 压缩
const handleCompress = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  currentOutput.value = compressSql(currentInput.value)
  currentError.value = ''
  currentIsError.value = false
  addHistory('压缩')
  ElMessage.success('压缩成功')
}

// Tab 3: 校验
const validationResult = ref<ValidationResult | null>(null)
const handleValidate = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  validationResult.value = validateSql(currentInput.value)
  const total = validationResult.value.errorCount + validationResult.value.warningCount
  if (total === 0) {
    ElMessage.success('未发现语法问题')
  } else {
    ElMessage.warning(`发现 ${total} 个问题`)
  }
  addHistory('校验')
}

// Tab 4: JSON→INSERT
const handleJsonToInsert = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入JSON内容')
    return
  }
  const result = jsonToInsert(currentInput.value, tableName.value)
  if (result.success) {
    currentOutput.value = result.data || ''
    currentError.value = ''
    currentIsError.value = false
    addHistory('JSON→INSERT')
    ElMessage.success('生成成功')
  } else {
    currentOutput.value = ''
    currentError.value = `错误: ${result.error}`
    currentIsError.value = true
  }
}

// Tab 5: 字段转换
const handleFieldConvert = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入字段名')
    return
  }
  currentOutput.value = convertFields(currentInput.value, convertMode.value)
  currentError.value = ''
  currentIsError.value = false
  addHistory('字段转换')
  ElMessage.success('转换成功')
}

// Tab 6: 注释操作
const handleAddLineComment = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  currentOutput.value = addLineComment(currentInput.value)
  addHistory('添加单行注释')
  ElMessage.success('已添加单行注释')
}

const handleRemoveLineComment = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  currentOutput.value = removeLineComment(currentInput.value)
  addHistory('移除单行注释')
  ElMessage.success('已移除单行注释')
}

const handleAddBlockComment = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  currentOutput.value = addBlockComment(currentInput.value)
  addHistory('添加多行注释')
  ElMessage.success('已添加多行注释')
}

const handleRemoveBlockComment = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入SQL内容')
    return
  }
  currentOutput.value = removeBlockComment(currentInput.value)
  addHistory('移除多行注释')
  ElMessage.success('已移除多行注释')
}

// Tab 7: SQL IN
const handleSqlInConvert = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请输入内容')
    return
  }
  const result = convertToSqlIn(currentInput.value, quoteType.value)
  currentOutput.value = result
  addHistory('SQL IN转换')
  ElMessage.success('转换成功')
}

// Tab 8: 快捷操作
const handleTemplateChange = (val: string) => {
  if (val && SQL_TEMPLATES[val]) {
    currentInput.value = SQL_TEMPLATES[val]
    ElMessage.success('模板已填入')
  }
}

// Tab 9: MyBatis解析
const handleParseLog = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请粘贴 MyBatis 日志内容')
    return
  }
  const result = parseMybatisLog(currentInput.value)
  if (result.success) {
    currentOutput.value = result.sql || ''
    currentError.value = ''
    currentIsError.value = false
    addHistory('MyBatis解析')
    ElMessage.success('解析成功')
  } else {
    currentOutput.value = ''
    currentError.value = `错误: ${result.error}`
    currentIsError.value = true
  }
}

// Tab 10: Java提取SQL
const handleJavaExtract = () => {
  if (!currentInput.value.trim()) {
    ElMessage.warning('请粘贴 Java 代码')
    return
  }
  const result = extractSqlFromJava(currentInput.value)
  if (result.success) {
    currentOutput.value = result.sql || ''
    currentError.value = ''
    currentIsError.value = false
    addHistory('Java提取SQL')
    ElMessage.success('提取成功')
  } else {
    currentOutput.value = ''
    currentError.value = `错误: ${result.error}`
    currentIsError.value = true
  }
}

// 监听输入变化，粘贴/输入后自动执行（带防抖）
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(
  () => tabState[activeTab.value].input,
  (value) => {
    if (!value.trim()) {
      currentOutput.value = ''
      currentError.value = ''
      currentIsError.value = false
      return
    }
    if (autoExecTimer) clearTimeout(autoExecTimer)
    autoExecTimer = setTimeout(() => {
      autoExecute()
    }, 300)
  }
)
</script>

<style scoped>
.sql-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.sql-tabs :deep(.el-tabs__nav-wrap) {
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

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  padding: 20px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: flex-end;
}

.action-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.group-buttons {
  display: flex;
  gap: 8px;
}

.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}

.validation-summary {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
}

.error-count {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.error-count.has-errors {
  color: var(--accent-red);
}

.warning-count {
  font-size: 14px;
  font-weight: 600;
  color: #eab308;
}

.error-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.error-item {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
}

.error-item.error {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.error-item.warning {
  background: rgba(234, 179, 8, 0.08);
  border: 1px solid rgba(234, 179, 8, 0.2);
}

.error-location {
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.error-type {
  font-weight: 600;
  font-size: 12px;
  white-space: nowrap;
}

.error-item.error .error-type {
  color: var(--accent-red);
}

.error-item.warning .error-type {
  color: #eab308;
}

.error-msg {
  color: var(--text-primary);
}

.success-message {
  color: #22c55e;
  font-size: 14px;
  padding: 12px;
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: 4px;
}
</style>
