<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 输入 JSON / JSON5（对象或对象数组）自动推断实体</p>
                <p>• 或粘贴 SQL 建表语句（CREATE TABLE）解析为实体</p>
                <p>• 勾选目标语言，实时生成对应实体代码</p>
                <p>• 嵌套对象会递归拆分为独立实体，字段自动转命名风格</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">输入格式</span>
            <el-radio-group v-model="format" size="small">
              <el-radio-button label="json">JSON</el-radio-button>
              <el-radio-button label="sql">SQL DDL</el-radio-button>
            </el-radio-group>
          </div>
          <div v-if="format === 'json'" class="action-group">
            <span class="group-label">根实体名</span>
            <el-input v-model="rootName" size="small" class="root-input" placeholder="Root" />
          </div>
          <div class="action-group">
            <span class="group-label">目标语言</span>
            <el-checkbox
              :model-value="isAllSelected"
              :indeterminate="isIndeterminate"
              size="small"
              @change="handleToggleAll"
            >全选</el-checkbox>
            <el-checkbox-group v-model="enabledLangs" size="small" class="lang-checks">
              <el-checkbox v-for="m in ENTITY_LANG_META" :key="m.key" :label="m.key" size="small">
                {{ m.label }}
              </el-checkbox>
            </el-checkbox-group>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入卡 -->
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
        <el-input
          v-model="input"
          type="textarea"
          :rows="6"
          :placeholder="placeholder"
          resize="vertical"
        />
        <div v-if="error" class="error-tip">{{ error }}</div>
        <div v-else-if="entities.length" class="meta-tip">
          解析出 {{ entities.length }} 个实体：{{ entities.map(e => e.name).join('、') }}
        </div>
      </div>
    </div>

    <!-- 输出卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">输出</span>
        <div class="card-actions">
          <el-tag v-if="hasOutput" size="small" type="success">{{ visibleLangs.length }} 种语言</el-tag>
        </div>
      </div>
      <div class="card-body">
        <div v-if="!hasOutput" class="empty-tip">输入内容后自动生成，结果将展示在此处</div>
        <div v-for="m in visibleLangs" :key="m.key" class="lang-block">
          <div class="lang-header">
            <span class="lang-name">{{ m.label }}</span>
            <div class="lang-actions">
              <el-button size="small" text @click="copyCode(m)">复制</el-button>
              <el-button size="small" text @click="saveCode(m)">保存</el-button>
            </div>
          </div>
          <pre class="code-block">{{ code[m.key] }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import { saveFileWithDialog } from '@/utils/fileSaver'
import {
  ENTITY_LANG_META, generateFromInput,
  type EntityLang, type InputFormat, type EntityDef,
} from '@/utils/entityGen'

const store = useToolboxStore()

const input = ref('')
const format = ref<InputFormat>('json')
const rootName = ref('Root')
const entities = ref<EntityDef[]>([])
const code = ref<Record<string, string>>({})
const error = ref('')
const enabledLangs = ref<EntityLang[]>(ENTITY_LANG_META.map(m => m.key))

const ALL_LANG_KEYS = ENTITY_LANG_META.map(m => m.key) as EntityLang[]
const visibleLangs = computed(() => ENTITY_LANG_META.filter(m => enabledLangs.value.includes(m.key)))
const hasOutput = computed(() => Object.values(code.value).some(Boolean))

const isAllSelected = computed(() => enabledLangs.value.length === ENTITY_LANG_META.length)
const isIndeterminate = computed(() => enabledLangs.value.length > 0 && !isAllSelected.value)

const placeholder = computed(() =>
  format.value === 'sql'
    ? 'CREATE TABLE t_user (\n  id bigint NOT NULL AUTO_INCREMENT,\n  user_name varchar(64) NOT NULL,\n  age int DEFAULT NULL\n);'
    : '{\n  "id": 1,\n  "user_name": "tom",\n  "active": true\n}',
)

const handleToggleAll = (checked: boolean | string | number) => {
  enabledLangs.value = checked ? [...ALL_LANG_KEYS] : []
}

// 至少保留一种语言，避免空输出
watch(enabledLangs, (val) => {
  if (val.length === 0) enabledLangs.value = [ALL_LANG_KEYS[0]]
})

// ============ 生成（输入/格式/语言变化 500ms 防抖自动执行） ============
let execTimer: ReturnType<typeof setTimeout> | null = null
let lastSavedKey = ''

const runGenerate = () => {
  const text = input.value
  if (!text.trim()) {
    entities.value = []
    code.value = {}
    error.value = ''
    return
  }
  const res = generateFromInput(text, { format: format.value, rootName: rootName.value || 'Root' }, enabledLangs.value)
  entities.value = res.entities
  code.value = res.code
  error.value = res.error ?? ''
  if (res.error || !res.entities.length) return
  const key = `${format.value}|${rootName.value}|${enabledLangs.value.join(',')}|${text}`
  if (key === lastSavedKey) return
  lastSavedKey = key
  store.addHistory({
    tool: 'entityGen',
    action: '代码实体生成',
    inputPreview: text.slice(0, 50),
    outputPreview: enabledLangs.value.join(' / ') + ' · ' + res.entities.map(e => e.name).join('、'),
    inputFull: text,
    outputFull: enabledLangs.value.map(l => `===== ${l} =====\n${res.code[l] ?? ''}`).join('\n\n'),
    options: { format: format.value, rootName: rootName.value, langs: [...enabledLangs.value] },
  })
}

watch([input, format, rootName, enabledLangs], () => {
  if (execTimer) clearTimeout(execTimer)
  execTimer = setTimeout(runGenerate, 500)
})

onBeforeUnmount(() => {
  if (execTimer) clearTimeout(execTimer)
})

// ============ 历史双击还原 ============
watch(
  () => store.pendingHistoryRestore,
  (data) => {
    if (!data || data.tool !== 'entityGen') return
    if (data.input) {
      input.value = data.input
      if (data.options?.format) format.value = data.options.format
      if (data.options?.rootName) rootName.value = data.options.rootName
      if (data.options?.langs?.length) enabledLangs.value = [...data.options.langs]
      store.clearHistoryRestore()
      runGenerate()
    }
  },
)

// ============ 复制 / 保存 ============
const copyCode = async (m: { key: EntityLang; label: string }) => {
  const text = code.value[m.key]
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(`已复制: ${m.label}`)
  } catch {
    ElMessage.error('复制失败')
  }
}

const saveCode = async (m: { key: EntityLang; label: string; ext: string }) => {
  const text = code.value[m.key]
  if (!text) return
  const name = (rootName.value || 'entity').replace(/[^\w.-]/g, '') || 'entity'
  await saveFileWithDialog(new Blob([text], { type: 'text/plain;charset=utf-8' }), `${name}.${m.ext}`, m.ext)
}

// ============ 输入区操作 ============
const handleClear = () => {
  input.value = ''
  entities.value = []
  code.value = {}
  error.value = ''
  lastSavedKey = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      ElMessage.warning('剪贴板为空')
      return
    }
    input.value = text
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  input.value = value
}
</script>

<style scoped>
/* 页面特有样式；布局类（.tool-card/.card-header 等）使用 theme.css 全局定义 */
.root-input {
  width: 160px;
}

.lang-checks {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 14px;
}

.error-tip {
  margin-top: 8px;
  color: var(--color-danger, #f56c6c);
  font-size: 13px;
}

.meta-tip {
  margin-top: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

.empty-tip {
  padding: 24px 0;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.lang-block {
  margin-bottom: 14px;
}

.lang-block:last-child {
  margin-bottom: 0;
}

.lang-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 2px;
}

.lang-name {
  color: var(--accent-cyan);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.lang-actions {
  display: flex;
  gap: 4px;
}

.code-block {
  margin: 0;
  padding: 12px 14px;
  max-height: 360px;
  overflow: auto;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre;
  tab-size: 2;
}
</style>
