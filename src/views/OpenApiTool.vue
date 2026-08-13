<template>
  <div class="tool-container">
    <!-- 输入卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">OpenAPI 文档</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>粘贴 OpenAPI 3.x 或 Swagger 2.0 文档（JSON / YAML）</p>
                <p>• 解析全部接口，按标签筛选、展开查看参数/请求体/响应</p>
                <p>• 每个接口可生成 curl / fetch 请求示例、Mock 数据、TS 类型</p>
                <p>• 支持一键导出全量接口清单与全部 Schema 的 TS 类型</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="v => (docText = v)" />
          <el-button size="small" @click="loadExample">示例文档</el-button>
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" type="primary" @click="handleParse">解析</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="docText"
          type="textarea"
          :rows="8"
          placeholder="粘贴 OpenAPI 文档（JSON 或 YAML）..."
          resize="vertical"
          class="code-input"
        />
        <div v-if="parseError" class="openapi-error">{{ parseError }}</div>
      </div>
    </div>

    <!-- 接口清单卡片 -->
    <div v-if="info" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">接口清单</span>
          <span class="api-meta">{{ info.title }}{{ info.apiVersion ? ' · v' + info.apiVersion : '' }} · {{ info.operations.length }} 个接口</span>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleAllTs">全部 Schema TS</el-button>
          <el-button size="small" @click="handleExportOps">导出接口清单</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="filter-row">
          <div class="tag-filters">
            <span
              class="tag-chip"
              :class="{ active: activeTag === '__all__' }"
              @click="activeTag = '__all__'"
            >全部 {{ info.operations.length }}</span>
            <span
              v-for="t in info.tagOrder"
              :key="t"
              class="tag-chip"
              :class="{ active: activeTag === t }"
              @click="activeTag = t"
            >{{ t }} {{ info.tagCounts[t] }}</span>
          </div>
          <div class="base-url-row">
            <span class="group-label">Base URL</span>
            <el-input v-model="baseUrl" size="small" placeholder="https://api.example.com" class="base-url-input" clearable />
          </div>
        </div>

        <el-collapse v-model="expandedKeys" class="openapi-collapse">
          <el-collapse-item v-for="op in filteredOps" :key="opKey(op)" :name="opKey(op)" class="op-item">
            <template #title>
              <span class="method-badge" :class="'m-' + op.method.toLowerCase()">{{ op.method }}</span>
              <span class="op-path">{{ op.path }}</span>
              <span v-if="op.summary" class="op-summary">{{ op.summary }}</span>
              <span v-if="op.deprecated" class="op-deprecated">DEPRECATED</span>
            </template>

            <div class="op-detail">
              <div v-if="op.description" class="op-desc">{{ op.description }}</div>

              <template v-if="op.parameters.length">
                <div class="op-section-title">Parameters（{{ op.parameters.length }}）</div>
                <el-table :data="op.parameters" size="small" border class="param-table">
                  <el-table-column prop="name" label="名称" width="150" show-overflow-tooltip />
                  <el-table-column prop="in" label="位置" width="80" />
                  <el-table-column label="类型" width="160">
                    <template #default="{ row }">{{ schemaTypeLabel(row.schema) }}</template>
                  </el-table-column>
                  <el-table-column label="必填" width="60" align="center">
                    <template #default="{ row }">{{ row.required ? '是' : '否' }}</template>
                  </el-table-column>
                  <el-table-column prop="description" label="描述" min-width="180" show-overflow-tooltip />
                </el-table>
              </template>

              <div v-if="op.requestBody" class="op-schema-block">
                <div class="op-section-title">Request Body{{ op.requestBody.required ? '（必填）' : '' }}</div>
                <div v-for="c in op.requestBody.content" :key="c.mediaType" class="schema-box">
                  <span class="media-type">{{ c.mediaType }}</span>
                  <pre class="schema-pre">{{ jsonOf(c.schema) }}</pre>
                </div>
              </div>

              <div v-if="op.responses.length" class="op-schema-block">
                <div class="op-section-title">Responses</div>
                <div v-for="r in op.responses" :key="r.status" class="resp-item">
                  <div class="resp-head">
                    <span class="status-code" :class="statusClass(r.status)">{{ r.status }}</span>
                    <span v-if="r.description" class="resp-desc">{{ r.description }}</span>
                  </div>
                  <template v-if="r.content.length">
                    <div v-for="c in r.content" :key="c.mediaType" class="schema-box">
                      <span class="media-type">{{ c.mediaType }}</span>
                      <pre class="schema-pre">{{ jsonOf(c.schema) }}</pre>
                    </div>
                  </template>
                  <div v-else class="resp-empty">（无响应体）</div>
                </div>
              </div>

              <div class="action-grid op-actions">
                <div class="action-group">
                  <div class="group-label">请求示例</div>
                  <el-radio-group v-model="exampleLang" size="small">
                    <el-radio-button label="curl">curl</el-radio-button>
                    <el-radio-button label="fetch">fetch</el-radio-button>
                  </el-radio-group>
                  <el-button size="small" type="primary" @click="genRequestExample(op)">生成</el-button>
                </div>
                <div class="action-group">
                  <div class="group-label">Mock</div>
                  <el-button size="small" @click="genMock(op)">响应 Mock</el-button>
                </div>
                <div class="action-group">
                  <div class="group-label">TS</div>
                  <el-button size="small" @click="genTs(op)">响应类型</el-button>
                </div>
              </div>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
    </div>

    <!-- 输出卡片 -->
    <div v-if="output" class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ outputTitle }}</span>
        <div class="card-actions">
          <el-button size="small" @click="handleDownload">下载</el-button>
          <el-button size="small" type="primary" @click="handleCopy">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input :model-value="output" type="textarea" :rows="12" readonly resize="vertical" class="code-input" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import VariablePicker from '@/components/VariablePicker.vue'
import { useToolboxStore } from '@/store'
import { saveFileWithDialog } from '@/utils/fileSaver'
import {
  parseOpenApiText, analyzeOpenApi,
  schemaTypeLabel, jsonOf, pickResponseSchema,
  buildMockFromSchema, buildRequestExample,
  buildOperationTs, buildAllSchemasTs, exportOperationsJson,
  EXAMPLE_OPENAPI, selfCheck,
  type OpenApiDoc, type OpenApiOperation, type MockOptions,
} from '@/utils/openApiUtils'

const store = useToolboxStore()

// ============ 状态 ============
const docText = ref('')
const parseError = ref('')
const doc = ref<any>(null)       // 解析出的原始文档对象
const info = ref<OpenApiDoc | null>(null)
const activeTag = ref('__all__')
const baseUrl = ref('')
const expandedKeys = ref<string[]>([])
const exampleLang = ref<'curl' | 'fetch'>('curl')
const mockOptions = ref<MockOptions>({ alwaysFakeOptionals: true, useDefaultValue: true })
const output = ref('')
const outputTitle = ref('')

// ============ 过滤接口 ============
const filteredOps = computed(() => {
  if (!info.value) return []
  if (activeTag.value === '__all__') return info.value.operations
  if (activeTag.value === '未分组') return info.value.operations.filter(o => o.tags.length === 0)
  return info.value.operations.filter(o => o.tags.includes(activeTag.value))
})

const opKey = (op: OpenApiOperation) => `${op.method}-${op.path}`

const statusClass = (status: string) => {
  if (/^2\d\d$/.test(status)) return 'status-2xx'
  if (/^3\d\d$/.test(status)) return 'status-3xx'
  if (/^4\d\d$/.test(status)) return 'status-4xx'
  if (/^5\d\d$/.test(status)) return 'status-5xx'
  return 'status-default'
}

// ============ 解析 ============
const doParse = (manual: boolean) => {
  const text = docText.value.trim()
  if (!text) return
  try {
    const d = parseOpenApiText(text)
    const i = analyzeOpenApi(d)
    doc.value = d
    info.value = i
    parseError.value = ''
    if (!baseUrl.value && i.baseUrl) baseUrl.value = i.baseUrl
    if (manual) {
      store.addHistory({
        tool: 'openApi',
        action: '解析 OpenAPI 文档',
        inputPreview: text.slice(0, 60),
        outputPreview: `${i.operations.length} 个接口`,
        inputFull: text,
        outputFull: exportOperationsJson(d),
      })
      ElMessage.success(`解析成功：${i.operations.length} 个接口`)
    }
  } catch (e: any) {
    parseError.value = e?.message || String(e)
    if (manual) ElMessage.error('解析失败: ' + parseError.value)
  }
}

const handleParse = () => {
  if (!docText.value.trim()) { ElMessage.warning('请先粘贴 OpenAPI 文档'); return }
  doParse(true)
}

// 粘贴/输入后 300ms 防抖自动解析（不写历史，避免刷屏）
let autoTimer: ReturnType<typeof setTimeout> | null = null
watch(docText, (val) => {
  if (!val.trim()) return
  if (autoTimer) clearTimeout(autoTimer)
  autoTimer = setTimeout(() => doParse(false), 300)
})

// ============ 示例 / 清空 ============
const loadExample = () => {
  docText.value = EXAMPLE_OPENAPI
  ElMessage.success('已载入示例文档')
}

const handleClear = () => {
  docText.value = ''
  parseError.value = ''
  doc.value = null
  info.value = null
  output.value = ''
  outputTitle.value = ''
  expandedKeys.value = []
  ElMessage.success('已清空')
}

// ============ 生成操作 ============
const recordHistory = (action: string) => {
  store.addHistory({
    tool: 'openApi',
    action,
    inputPreview: docText.value.slice(0, 60),
    outputPreview: output.value.slice(0, 60),
    inputFull: docText.value,
    outputFull: output.value,
  })
}

const genRequestExample = (op: OpenApiOperation) => {
  if (!doc.value) return
  try {
    output.value = buildRequestExample(doc.value, op, { baseUrl: baseUrl.value, lang: exampleLang.value })
    outputTitle.value = `${op.method} ${op.path} · 请求示例（${exampleLang.value}）`
    recordHistory(`请求示例 ${op.method} ${op.path}`)
    ElMessage.success('请求示例已生成')
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message || e))
  }
}

const genMock = (op: OpenApiOperation) => {
  if (!doc.value) return
  try {
    output.value = buildMockFromSchema(doc.value, pickResponseSchema(op) || {}, { ...mockOptions.value })
    outputTitle.value = `${op.method} ${op.path} · 响应 Mock`
    recordHistory(`Mock ${op.method} ${op.path}`)
    ElMessage.success('Mock 数据已生成')
  } catch (e: any) {
    ElMessage.error('Mock 生成失败: ' + (e?.message || e))
  }
}

const genTs = (op: OpenApiOperation) => {
  if (!doc.value) return
  try {
    output.value = buildOperationTs(doc.value, op, '')
    outputTitle.value = `${op.method} ${op.path} · 响应 TS 类型`
    recordHistory(`TS 类型 ${op.method} ${op.path}`)
    ElMessage.success('TS 类型已生成')
  } catch (e: any) {
    ElMessage.error('TS 生成失败: ' + (e?.message || e))
  }
}

const handleAllTs = () => {
  if (!doc.value) return
  try {
    output.value = buildAllSchemasTs(doc.value)
    outputTitle.value = '全部 Schema TS 类型'
    recordHistory('全部 Schema TS')
    ElMessage.success('已生成全部 Schema 的 TS 类型')
  } catch (e: any) {
    ElMessage.error('生成失败: ' + (e?.message || e))
  }
}

const handleExportOps = () => {
  if (!doc.value) return
  try {
    output.value = exportOperationsJson(doc.value)
    outputTitle.value = '接口清单'
    recordHistory('导出接口清单')
    ElMessage.success('接口清单已导出')
  } catch (e: any) {
    ElMessage.error('导出失败: ' + (e?.message || e))
  }
}

// ============ 输出操作 ============
const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(output.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const DOWNLOAD_MAP: Array<[RegExp, string, string]> = [
  [/TS|类型/i, 'openapi-types', 'ts'],
  [/Mock/i, 'openapi-mock', 'json'],
  [/清单/i, 'openapi-operations', 'json'],
]

const handleDownload = async () => {
  const hit = DOWNLOAD_MAP.find(([re]) => re.test(outputTitle.value))
  const name = hit ? hit[1] : 'openapi-output'
  const ext = hit ? hit[2] : 'txt'
  const ts = Date.now()
  await saveFileWithDialog(
    new Blob([output.value], { type: 'text/plain;charset=utf-8' }),
    `${name}-${ts}.${ext}`,
    ext,
  )
}

// ============ 自检 ============
const errors = selfCheck()
if (errors.length > 0) {
  console.warn('openApiUtils 自检失败:', errors)
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

.header-left { display: flex; align-items: center; gap: 8px; }
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 340px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.code-input :deep(.el-textarea__inner) {
  font-family: 'JetBrains Mono', Consolas, 'Courier New', monospace;
  font-size: 13px;
  background: var(--bg-input);
  color: var(--text-primary);
}

.openapi-error {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  color: var(--danger-color, #ef4444);
  font-size: 13px;
  word-break: break-all;
}

.api-meta {
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 360px;
}

.filter-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.tag-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex: 1;
}

.tag-chip {
  padding: 2px 10px;
  font-size: 12px;
  border-radius: 12px;
  cursor: pointer;
  color: var(--text-secondary);
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  white-space: nowrap;
}
.tag-chip:hover { border-color: var(--accent-cyan); color: var(--accent-cyan); }
.tag-chip.active {
  color: #fff;
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
}

.base-url-row { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.base-url-input { width: 260px; }

/* ===== 接口列表 ===== */
.openapi-collapse :deep(.el-collapse-item__header) {
  background: transparent;
  border-bottom-color: var(--border-color);
  padding-left: 12px;
  gap: 10px;
  height: auto;
  min-height: 44px;
  line-height: 1.4;
  display: flex;
  align-items: center;
  color: var(--text-primary);
}
.openapi-collapse :deep(.el-collapse-item__wrap) {
  background: transparent;
  border-bottom-color: var(--border-color);
}
.openapi-collapse :deep(.el-collapse-item__content) {
  padding: 12px 16px 16px;
}

.method-badge {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  padding: 2px 8px;
  border-radius: 4px;
  color: #fff;
}
.m-get { background: #10b981; }
.m-post { background: #3b82f6; }
.m-put { background: #f59e0b; }
.m-delete { background: #ef4444; }
.m-patch { background: #8b5cf6; }
.m-options, .m-head, .m-trace { background: #64748b; }

.op-path {
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 13px;
  color: var(--accent-cyan);
  white-space: nowrap;
}
.op-summary {
  color: var(--text-secondary);
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
.op-deprecated {
  flex-shrink: 0;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.5);
}

.op-detail { font-size: 13px; }
.op-desc {
  color: var(--text-secondary);
  margin-bottom: 12px;
  white-space: pre-wrap;
  word-break: break-word;
}
.op-section-title {
  font-weight: 600;
  font-size: 12px;
  letter-spacing: 0.5px;
  color: var(--text-primary);
  margin: 12px 0 8px;
  text-transform: uppercase;
}
.param-table { margin-bottom: 8px; }
.param-table :deep(th) { color: var(--text-secondary); }

.op-schema-block { margin-top: 4px; }
.schema-box { margin-bottom: 8px; }
.media-type {
  display: inline-block;
  font-size: 11px;
  font-family: 'JetBrains Mono', Consolas, monospace;
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 212, 255, 0.4);
  border-radius: 3px;
  padding: 1px 6px;
  margin-bottom: 6px;
}
.schema-pre {
  margin: 0;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-primary);
  max-height: 240px;
  overflow: auto;
  white-space: pre;
}
.resp-item { margin-bottom: 8px; }
.resp-head { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
.status-code {
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
  padding: 1px 8px;
  border-radius: 4px;
  color: #fff;
}
.status-2xx { background: #10b981; }
.status-3xx { background: #3b82f6; }
.status-4xx { background: #f59e0b; }
.status-5xx { background: #ef4444; }
.status-default { background: #64748b; }
.resp-desc { color: var(--text-secondary); font-size: 12px; }
.resp-empty { color: var(--text-secondary); font-size: 12px; padding: 4px 0; }

.op-actions { margin-top: 14px; padding-top: 12px; border-top: 1px dashed var(--border-color); }
</style>
