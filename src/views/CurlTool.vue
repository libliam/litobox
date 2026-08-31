<template>
  <div class="tool-container">
    <!-- 顶部：方法 + URL -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Curl 构建器</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 根据参数实时生成 curl 命令（Linux bash 风格）</p>
                <p>• 支持请求头 / Basic / Bearer 认证 / JSON / Form / 原始请求体</p>
                <p>• 生成后一键复制到终端执行</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="request-bar">
          <el-select v-model="method" size="default" style="width: 120px">
            <el-option label="GET" value="GET" />
            <el-option label="POST" value="POST" />
          </el-select>
          <el-input
            v-model="url"
            placeholder="输入请求 URL，如: https://api.example.com/v1/users"
            size="default"
            style="flex: 1; margin-left: 8px"
          >
            <template #append>
              <VariablePicker @select="handleInsertVariable" />
            </template>
          </el-input>
        </div>
      </div>
    </div>

    <!-- 请求头 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">请求头</span>
        <el-button size="small" @click="addHeader">+ 添加</el-button>
      </div>
      <div class="card-body">
        <div v-for="(header, idx) in headers" :key="idx" class="kv-row">
          <el-input v-model="header.key" placeholder="Header 名称" size="small" style="width: 200px" />
          <el-input v-model="header.value" placeholder="值" size="small" style="flex: 1; margin-left: 8px" />
          <el-button size="small" type="danger" :icon="Delete" circle @click="removeHeader(idx)" style="margin-left: 8px" />
        </div>
        <div v-if="headers.length === 0" class="empty-hint">暂无请求头</div>
      </div>
    </div>

    <!-- 认证 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">认证</span>
      </div>
      <div class="card-body">
        <el-radio-group v-model="authType" size="small">
          <el-radio-button label="none">无</el-radio-button>
          <el-radio-button label="basic">Basic Auth</el-radio-button>
          <el-radio-button label="bearer">Bearer Token</el-radio-button>
        </el-radio-group>
        <template v-if="authType === 'basic'">
          <div class="kv-row" style="margin-top: 12px">
            <el-input v-model="basicUser" placeholder="用户名" size="small" style="width: 200px" />
            <el-input v-model="basicPass" placeholder="密码" type="password" show-password size="small" style="flex: 1; margin-left: 8px" />
          </div>
        </template>
        <template v-else-if="authType === 'bearer'">
          <el-input v-model="bearerToken" placeholder="输入 Token" size="small" style="margin-top: 12px" />
        </template>
      </div>
    </div>

    <!-- 请求体 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">请求体</span>
        <div class="card-actions">
          <el-radio-group v-model="bodyType" size="small">
            <el-radio-button label="none">无</el-radio-button>
            <el-radio-button label="json">JSON</el-radio-button>
            <el-radio-button label="form">Form</el-radio-button>
            <el-radio-button label="raw">原始文本</el-radio-button>
          </el-radio-group>
          <VariablePicker v-if="bodyType === 'json' || bodyType === 'raw'" @select="handleInsertBodyVariable" />
        </div>
      </div>
      <div class="card-body">
        <template v-if="bodyType === 'json'">
          <el-input v-model="jsonBody" type="textarea" :rows="6" placeholder='{"key": "value"}' resize="vertical" />
        </template>
        <template v-else-if="bodyType === 'form'">
          <div v-for="(field, idx) in formFields" :key="idx" class="kv-row">
            <el-input v-model="field.key" placeholder="字段名" size="small" style="width: 200px" />
            <el-input v-model="field.value" placeholder="值" size="small" style="flex: 1; margin-left: 8px" />
            <el-button size="small" type="danger" :icon="Delete" circle @click="formFields.splice(idx, 1)" style="margin-left: 8px" />
          </div>
          <el-button size="small" style="margin-top: 8px" @click="formFields.push({ key: '', value: '' })">+ 添加字段</el-button>
        </template>
        <template v-else-if="bodyType === 'raw'">
          <el-input v-model="rawBody" type="textarea" :rows="6" placeholder="输入原始请求体内容" resize="vertical" />
        </template>
      </div>
    </div>

    <!-- 其他选项 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">其他选项</span>
      </div>
      <div class="card-body">
        <div class="option-grid">
          <div class="option-item" style="flex: 1">
            <span class="option-label">Cookie</span>
            <el-input v-model="cookie" placeholder="如: session=abc; token=xyz" size="small" />
          </div>
          <div class="option-item">
            <span class="option-label">超时（秒）</span>
            <el-input v-model="timeout" placeholder="如: 30" size="small" style="width: 120px" />
          </div>
        </div>
      </div>
    </div>

    <!-- 批量变量 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">批量变量</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 在 URL / 请求头值 / Body / Cookie 中写入 <code v-text="'{{变量名}}'"></code> 占位符</p>
                <p>• 表头填写变量名，每行一组值，自动生成多条 curl 命令</p>
                <p>• 关闭开关即回到单条模式，不影响已有配置</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-switch v-model="batchEnabled" inline-prompt active-text="启用批量" inactive-text="单条模式" />
        </div>
      </div>
      <div class="card-body" v-if="batchEnabled">
        <div class="batch-toolbar">
          <el-button size="small" @click="addBatchColumn">+ 添加列</el-button>
          <el-button size="small" @click="addBatchRow">+ 添加行</el-button>
          <span class="batch-hint">变量名：<code v-for="n in batchCols" :key="'hint-'+n" class="var-chip">{{ batchHeader[n] || '(未命名)' }}</code></span>
        </div>
        <table class="batch-table" v-if="batchCols.length">
          <thead>
            <tr>
              <th class="th-idx">#</th>
              <th v-for="col in batchCols" :key="'h-'+col">
                <div class="th-wrapper">
                  <el-input v-model="batchHeader[col]" :placeholder="`变量${col+1}名`" size="small" />
                  <el-button size="small" type="danger" link @click="removeBatchColumn(col)">移除</el-button>
                </div>
              </th>
              <th class="th-actions"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(_row, ri) in batchRows" :key="'r-'+ri">
              <td class="td-idx">{{ ri + 1 }}</td>
              <td v-for="col in batchCols" :key="`c-${ri}-${col}`">
                <el-input v-model="batchRows[ri][col]" :placeholder="batchHeader[col] ? `{{${batchHeader[col]}}}` : `值`" size="small" />
              </td>
              <td class="td-actions">
                <el-button size="small" type="danger" link @click="removeBatchRow(ri)">删除行</el-button>
              </td>
            </tr>
          </tbody>
        </table>
        <div v-else class="empty-hint">先点「+ 添加列」创建至少一个变量，然后每行填一组值</div>
      </div>
    </div>

    <!-- 生成的命令 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">
          生成的命令
          <span v-if="batchEnabled && batchCommands.length" class="count-badge">{{ batchCommands.length }} 条</span>
        </span>
        <div class="card-actions">
          <el-button size="small" type="primary" :disabled="!hasAnyCommand" @click="handleCopy">
            {{ batchEnabled ? '复制全部命令' : '复制命令' }}
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <template v-if="batchEnabled">
          <div v-if="batchCommands.length">
            <div v-for="(cmd, i) in batchCommands" :key="i" class="batch-cmd-item">
              <div class="cmd-index-bar">
                <span class="cmd-index">#{{ i + 1 }}</span>
                <el-button size="small" link @click="copyOne(cmd)">复制</el-button>
              </div>
              <pre class="curl-output">{{ cmd }}</pre>
            </div>
          </div>
          <div v-else class="empty-tip">填写 URL 或启用批量后创建变量行</div>
        </template>
        <template v-else>
          <pre v-if="curlCommand" class="curl-output">{{ curlCommand }}</pre>
          <div v-else class="empty-tip">填写 URL 后自动生成 curl 命令</div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Delete } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import { buildCurlCommand, type CurlParams } from '@/utils/curlUtils'

const store = useToolboxStore()

// ============ 表单状态（默认一行请求头、bodyType=none；POST 时自动切 JSON） ============
const method = ref('GET')
const url = ref('')
const headers = ref<Array<{ key: string; value: string }>>([{ key: '', value: '' }])
const authType = ref<'none' | 'basic' | 'bearer'>('none')
const basicUser = ref('')
const basicPass = ref('')
const bearerToken = ref('')
const bodyType = ref<'none' | 'json' | 'form' | 'raw'>('none')
const jsonBody = ref('')
const formFields = ref<Array<{ key: string; value: string }>>([])
const rawBody = ref('')
const cookie = ref('')
const timeout = ref('')

// ============ 交互联动默认值 ============
// 方法切到 POST → 若未选 bodyType，默认切 JSON 类型
watch(method, (val, old) => {
  if (val === 'POST' && old !== 'POST' && bodyType.value === 'none') {
    bodyType.value = 'json'
  }
})
// bodyType 切到 form → 至少有一行空字段
watch(bodyType, (val, old) => {
  if (val === 'form' && old !== 'form' && formFields.value.length === 0) {
    formFields.value = [{ key: '', value: '' }]
  }
})

// ============ 请求头操作 ============
const addHeader = () => { headers.value.push({ key: '', value: '' }) }
const removeHeader = (idx: number) => { headers.value.splice(idx, 1) }

// ============ 变量池插入 ============
const handleInsertVariable = (value: string) => { url.value = value }
const handleInsertBodyVariable = (value: string) => {
  if (bodyType.value === 'json') jsonBody.value = value
  else if (bodyType.value === 'raw') rawBody.value = value
}

// ============ curl 命令生成（bash 风格） ============
const formSnapshot = (): CurlParams => ({
  method: method.value,
  url: url.value,
  headers: headers.value.map(h => ({ key: h.key, value: h.value })),
  authType: authType.value,
  basicUser: basicUser.value,
  basicPass: basicPass.value,
  bearerToken: bearerToken.value,
  bodyType: bodyType.value,
  jsonBody: jsonBody.value,
  formFields: formFields.value.map(f => ({ key: f.key, value: f.value })),
  rawBody: rawBody.value,
  cookie: cookie.value,
  timeout: timeout.value,
})

const curlCommand = computed(() => buildCurlCommand(formSnapshot()))

// ============ 批量变量 ============
type BatchRow = { [colIdx: number]: string }

const batchEnabled = ref(false)
const batchHeader = ref<Record<number, string>>({}) // colIdx -> 变量名
const batchCols = ref<number[]>([])             // 存在的列索引（按顺序 push 新索引）
const batchRows = ref<BatchRow[]>([])           // 每行按列索引填值
let nextColIdx = 0

const addBatchColumn = () => {
  const idx = nextColIdx++
  batchCols.value.push(idx)
  batchHeader.value[idx] = ''
  // 已存在的行补齐该列空位
  for (const row of batchRows.value) row[idx] = ''
}
const removeBatchColumn = (col: number) => {
  batchCols.value = batchCols.value.filter(c => c !== col)
  delete batchHeader.value[col]
  for (const row of batchRows.value) delete row[col]
}
const addBatchRow = () => {
  if (batchCols.value.length === 0) {
    ElMessage.warning('请先添加变量列')
    return
  }
  const row: BatchRow = {}
  for (const c of batchCols.value) row[c] = ''
  batchRows.value.push(row)
}
const removeBatchRow = (ri: number) => { batchRows.value.splice(ri, 1) }

/** 按一组变量值 (colIdx => value) 替换所有 string 字段中的 {{name}} 占位符，返回新的 CurlParams */
const applyVars = (base: CurlParams, row: BatchRow): CurlParams => {
  // colIdx -> 变量名 -> 值
  const nameToVal: Record<string, string> = {}
  for (const c of batchCols.value) {
    const name = (batchHeader.value[c] || '').trim()
    if (name) nameToVal[name] = row[c] ?? ''
  }
  // ponytail: 简单的一次性占位符替换，不支持嵌套/转义；够用
  const re = new RegExp(
    `\\{\\{(${Object.keys(nameToVal).map(k => k.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')})\\}\\}`,
    'g'
  )
  const t = (s: string): string =>
    Object.keys(nameToVal).length ? s.replace(re, (_, k) => nameToVal[k] ?? '') : s

  return {
    ...base,
    url: t(base.url),
    headers: base.headers.map(h => ({ key: h.key, value: t(h.value) })),
    basicUser: t(base.basicUser),
    basicPass: t(base.basicPass),
    bearerToken: t(base.bearerToken),
    jsonBody: t(base.jsonBody),
    formFields: base.formFields.map(f => ({ key: f.key, value: t(f.value) })),
    rawBody: t(base.rawBody),
    cookie: t(base.cookie),
  }
}

const batchCommands = computed<string[]>(() => {
  if (!batchEnabled.value) return []
  const base = formSnapshot()
  // 只处理有 URL 的行（避免整行空的无效行）
  const rows = batchRows.value.filter(r => Object.values(r).some(v => v))
  if (rows.length === 0) {
    // 如果还没任何行，兜底输出一条按当前配置（不做替换）的命令，让输出区不为空
    return buildCurlCommand(base) ? [buildCurlCommand(base)] : []
  }
  const out: string[] = []
  for (const r of rows) {
    const cmd = buildCurlCommand(applyVars(base, r))
    if (cmd) out.push(cmd)
  }
  return out
})

const hasAnyCommand = computed(() =>
  batchEnabled.value ? batchCommands.value.length > 0 : !!curlCommand.value
)

const copyOne = async (cmd: string) => {
  try {
    await navigator.clipboard.writeText(cmd)
    ElMessage.success('已复制该条命令')
  } catch { ElMessage.error('复制失败') }
}

// ============ 复制 + 历史 ============
const restoreFromSnapshot = (data: any) => {
  if (!data) return
  method.value = data.method || 'GET'
  url.value = data.url || ''
  headers.value = (data.headers || []).map((h: any) => ({ key: h.key || '', value: h.value || '' }))
  authType.value = data.authType || 'none'
  basicUser.value = data.basicUser || ''
  basicPass.value = data.basicPass || ''
  bearerToken.value = data.bearerToken || ''
  bodyType.value = data.bodyType || 'none'
  jsonBody.value = data.jsonBody || ''
  formFields.value = (data.formFields || []).map((f: any) => ({ key: f.key || '', value: f.value || '' }))
  rawBody.value = data.rawBody || ''
  cookie.value = data.cookie || ''
  timeout.value = data.timeout || ''
  batchEnabled.value = !!data.batchEnabled
  batchHeader.value = { ...(data.batchHeader || {}) }
  batchCols.value = Array.isArray(data.batchCols) ? [...data.batchCols] : []
  nextColIdx = batchCols.value.length ? Math.max(...batchCols.value) + 1 : 0
  batchRows.value = Array.isArray(data.batchRows) ? data.batchRows.map((r: any) => ({ ...r })) : []
}

const handleCopy = async () => {
  if (!hasAnyCommand.value) { ElMessage.warning('请先填写 URL'); return }
  try {
    if (batchEnabled.value) {
      const allText = batchCommands.value.map((c, i) => `# ---- #${i + 1} ----\n${c}`).join('\n\n')
      await navigator.clipboard.writeText(allText)
      ElMessage.success(`已复制全部 ${batchCommands.value.length} 条命令`)
      const snapshot = { ...formSnapshot(), batchEnabled: true, batchHeader: batchHeader.value, batchCols: batchCols.value, batchRows: batchRows.value }
      store.addHistory({
        tool: 'curl',
        action: `批量 ${method.value} ${batchCommands.value.length}条`,
        inputPreview: `${method.value} ${batchCommands.value.length}条`,
        outputPreview: batchCommands.value[0]?.slice(0, 50) || '',
        inputFull: JSON.stringify(snapshot),
        outputFull: allText,
        options: snapshot as any,
      })
    } else {
      await navigator.clipboard.writeText(curlCommand.value)
      ElMessage.success('已复制到剪贴板')
      store.addHistory({
        tool: 'curl',
        action: `${method.value} ${url.value.slice(0, 40)}`,
        inputPreview: `${method.value} ${url.value.slice(0, 30)}`,
        outputPreview: curlCommand.value.slice(0, 50),
        inputFull: JSON.stringify(formSnapshot()),
        outputFull: curlCommand.value,
        options: formSnapshot(),
      })
    }
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 历史还原（KeepAlive 缓存组件用 watch） ============
watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== 'curl') return
  let snapshot: any = restore.options
  if (!snapshot) {
    try { snapshot = JSON.parse(restore.input || '{}') } catch { snapshot = null }
  }
  restoreFromSnapshot(snapshot)
  store.clearHistoryRestore()
}, { immediate: true })
</script>

<style scoped>
/* ===== 请求栏 ===== */
.request-bar {
  display: flex;
  align-items: center;
}

/* ===== 键值行（请求头/表单字段） ===== */
.kv-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.kv-row:last-child {
  margin-bottom: 0;
}

/* ===== 其他选项 ===== */
.option-grid {
  display: flex;
  gap: 16px;
  align-items: center;
}

.option-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.option-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

/* ===== 生成的命令 ===== */
.curl-output {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 12px 16px;
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* ===== 空/提示状态 ===== */
.empty-hint {
  text-align: center;
  padding: 12px 0;
  color: var(--text-muted);
  font-size: 13px;
}

.empty-tip {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-size: 14px;
}

/* ===== 批量变量 ===== */
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.hint-icon {
  color: var(--text-muted);
  cursor: help;
}
.batch-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
.batch-hint {
  margin-left: auto;
  color: var(--text-secondary);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 4px;
}
.var-chip {
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  color: var(--accent-cyan);
  font-size: 12px;
  margin-left: 4px;
}
.batch-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-input);
}
.batch-table th,
.batch-table td {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-color);
  text-align: left;
  vertical-align: middle;
  font-size: 13px;
}
.batch-table thead th {
  background: rgba(0, 229, 255, 0.05);
  color: var(--text-primary);
  font-weight: 500;
}
.batch-table tbody tr:last-child td { border-bottom: none; }
.th-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
}
.th-idx, .td-idx {
  width: 48px;
  color: var(--text-secondary);
  text-align: center;
}
.th-actions { width: 72px; }
.td-actions { text-align: right; }

/* ===== 批量命令列表 ===== */
.count-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 1px 8px;
  border-radius: 999px;
  background: rgba(0, 229, 255, 0.12);
  color: var(--accent-cyan);
  font-size: 12px;
  font-weight: 500;
}
.batch-cmd-item {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 14px;
  overflow: hidden;
  background: var(--bg-card);
}
.batch-cmd-item:last-child { margin-bottom: 0; }
.cmd-index-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: rgba(0, 229, 255, 0.08);
  border-bottom: 1px solid var(--border-color);
}
.cmd-index {
  font-weight: 600;
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  letter-spacing: 0.5px;
}
.batch-cmd-item .curl-output {
  border: none;
  border-radius: 0;
}
.tooltip-content {
  line-height: 1.7;
}
.tooltip-content p { margin: 2px 0; }
.tooltip-content code {
  padding: 0 4px;
  border-radius: 3px;
  background: rgba(0, 229, 255, 0.15);
  color: var(--accent-cyan);
}
</style>
