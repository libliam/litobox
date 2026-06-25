<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="password-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 密码生成 -->
      <el-tab-pane label="密码生成" name="generate">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>配置密码长度和字符类型，生成随机密码</p>
                    <p>使用 crypto.getRandomValues 生成真随机数</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleGenerate">生成</el-button>
              <el-button size="small" @click="handleCopy('generate')">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">长度</div>
                <el-input-number v-model="passwordLength" :min="4" :max="128" size="small" style="width: 120px" />
              </div>
              <div class="action-group">
                <div class="group-label">字符类型</div>
                <el-checkbox-group v-model="passwordOptions" size="small">
                  <el-checkbox label="uppercase">大写</el-checkbox>
                  <el-checkbox label="lowercase">小写</el-checkbox>
                  <el-checkbox label="numbers">数字</el-checkbox>
                  <el-checkbox label="symbols">符号</el-checkbox>
                </el-checkbox-group>
              </div>
              <div class="action-group">
                <div class="group-label">数量</div>
                <el-input-number v-model="passwordCount" :min="1" :max="20" size="small" style="width: 100px" />
              </div>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">生成的密码</span>
            <el-button size="small" @click="handleCopy('generate')">复制全部</el-button>
          </div>
          <div class="card-body">
            <div v-if="generatedPasswords.length > 0" class="password-list">
              <div v-for="(item, idx) in generatedPasswords" :key="idx" class="password-item">
                <div class="password-content">
                  <code class="password-value">{{ item.password }}</code>
                  <div class="password-strength" :class="item.strength.class">
                    <div class="strength-bar">
                      <div class="strength-fill" :style="{ width: item.strength.percent + '%' }"></div>
                    </div>
                    <span class="strength-label">{{ item.strength.label }}</span>
                  </div>
                </div>
                <el-button size="small" @click="copyValue(item.password)">复制</el-button>
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"生成"按钮生成密码
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: API Key 生成 -->
      <el-tab-pane label="API Key" name="apikey">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
            <div class="card-actions">
              <el-button size="small" @click="handleGenerateApiKey">生成</el-button>
              <el-button size="small" @click="handleCopy('apikey')">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">格式</div>
                <el-select v-model="apiKeyFormat" size="small" style="width: 140px">
                  <el-option label="UUID v4" value="uuid" />
                  <el-option label="随机字符串" value="random" />
                  <el-option label="Bearer Token" value="bearer" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">数量</div>
                <el-input-number v-model="apiKeyCount" :min="1" :max="20" size="small" style="width: 100px" />
              </div>
              <div class="action-group" v-if="apiKeyFormat === 'random'">
                <div class="group-label">长度</div>
                <el-input-number v-model="apiKeyLength" :min="16" :max="128" size="small" style="width: 100px" />
              </div>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">生成的 Key</span>
            <el-button size="small" @click="handleCopy('apikey')">复制全部</el-button>
          </div>
          <div class="card-body">
            <div v-if="generatedApiKeys.length > 0" class="password-list">
              <div v-for="(key, idx) in generatedApiKeys" :key="idx" class="password-item">
                <code class="password-value apikey-value">{{ key }}</code>
                <el-button size="small" @click="copyValue(key)">复制</el-button>
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"生成"按钮生成 API Key
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 密码强度检测 -->
      <el-tab-pane label="强度检测" name="strength">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入密码，实时检测密码强度</p>
                    <p>根据长度、字符类型多样性综合评分</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('strength')">清空</el-button>
              <el-button size="small" @click="handlePaste('strength')">粘贴</el-button>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入密码</span>
          </div>
          <div class="card-body">
            <el-input
              v-model="strengthInput"
              type="password"
              placeholder="输入要检测的密码..."
              show-password
              @input="handleStrengthCheck"
            />
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">强度结果</span>
          </div>
          <div class="card-body">
            <div v-if="strengthResult" class="strength-result">
              <div class="strength-meter">
                <div class="strength-bar-bg">
                  <div class="strength-bar-fill" :class="strengthResult.class" :style="{ width: strengthResult.percent + '%' }"></div>
                </div>
                <div class="strength-label-large" :class="strengthResult.class">{{ strengthResult.label }}</div>
              </div>
              <div class="strength-details">
                <div class="detail-item">
                  <span class="detail-label">长度</span>
                  <span class="detail-value">{{ strengthResult.length }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">字符类型</span>
                  <span class="detail-value">{{ strengthResult.charTypes.join('、') || '无' }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">熵值</span>
                  <span class="detail-value">{{ strengthResult.entropy.toFixed(1) }} bits</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">建议</span>
                  <span class="detail-value">{{ strengthResult.suggestion }}</span>
                </div>
              </div>
            </div>
            <div v-else class="stats-empty">
              输入密码后自动检测强度
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('generate')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  generate: { input: '', output: '', error: '', isError: false },
  apikey: { input: '', output: '', error: '', isError: false },
  strength: { input: '', output: '', error: '', isError: false }
})

// ============ 密码生成 Tab ============
const passwordLength = ref(16)
const passwordOptions = ref(['uppercase', 'lowercase', 'numbers', 'symbols'])
const passwordCount = ref(1)

interface GeneratedPassword {
  password: string
  strength: { label: string; class: string; percent: number }
}

const generatedPasswords = ref<GeneratedPassword[]>([])

// 真随机数生成
const getRandomValues = (array: Uint8Array): Uint8Array => {
  return crypto.getRandomValues(array)
}

const generateRandomPassword = (length: number, options: string[]): string => {
  const charSets: Record<string, string> = {
    uppercase: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
    lowercase: 'abcdefghijklmnopqrstuvwxyz',
    numbers: '0123456789',
    symbols: '!@#$%^&*()_+-=[]{}|;:,.<>?'
  }

  const chars = options.map(opt => charSets[opt]).join('')
  if (!chars) return ''

  const result = new Array(length)
  const randomBytes = getRandomValues(new Uint8Array(length))
  for (let i = 0; i < length; i++) {
    result[i] = chars[randomBytes[i] % chars.length]
  }
  return result.join('')
}

const evaluatePasswordStrength = (password: string): { label: string; class: string; percent: number } => {
  const len = password.length
  let score = 0

  if (len >= 8) score += 1
  if (len >= 12) score += 1
  if (len >= 16) score += 1
  if (/[a-z]/.test(password)) score += 1
  if (/[A-Z]/.test(password)) score += 1
  if (/[0-9]/.test(password)) score += 1
  if (/[^a-zA-Z0-9]/.test(password)) score += 1

  if (score <= 2) return { label: '弱', class: 'weak', percent: 25 }
  if (score <= 4) return { label: '中', class: 'medium', percent: 50 }
  if (score <= 5) return { label: '强', class: 'strong', percent: 75 }
  return { label: '极强', class: 'very-strong', percent: 100 }
}

const handleGenerate = () => {
  if (passwordOptions.value.length === 0) {
    ElMessage.warning('请至少选择一种字符类型')
    return
  }

  generatedPasswords.value = []
  for (let i = 0; i < passwordCount.value; i++) {
    const pwd = generateRandomPassword(passwordLength.value, passwordOptions.value)
    generatedPasswords.value.push({
      password: pwd,
      strength: evaluatePasswordStrength(pwd)
    })
  }

  tabState.generate.output = generatedPasswords.value.map(p => p.password).join('\n')
  store.addHistory({ tool: 'password', action: 'generate', inputPreview: `${passwordLength.value}位`, outputPreview: generatedPasswords.value[0].password.slice(0, 10) + '...' })
  ElMessage.success(`已生成 ${passwordCount.value} 个密码`)
}

// ============ API Key 生成 Tab ============
const apiKeyFormat = ref('uuid')
const apiKeyCount = ref(1)
const apiKeyLength = ref(32)
const generatedApiKeys = ref<string[]>([])

const generateUUID = (): string => {
  const bytes = getRandomValues(new Uint8Array(16))
  bytes[6] = (bytes[6] & 0x0f) | 0x40 // version 4
  bytes[8] = (bytes[8] & 0x3f) | 0x80 // variant 1
  const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

const generateRandomString = (length: number): string => {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  const bytes = getRandomValues(new Uint8Array(length))
  return Array.from(bytes, b => chars[b % chars.length]).join('')
}

const handleGenerateApiKey = () => {
  generatedApiKeys.value = []
  for (let i = 0; i < apiKeyCount.value; i++) {
    let key: string
    switch (apiKeyFormat.value) {
      case 'uuid':
        key = generateUUID()
        break
      case 'random':
        key = generateRandomString(apiKeyLength.value)
        break
      case 'bearer':
        key = 'Bearer ' + generateRandomString(64)
        break
      default:
        key = ''
    }
    generatedApiKeys.value.push(key)
  }

  tabState.apikey.output = generatedApiKeys.value.join('\n')
  store.addHistory({ tool: 'password', action: 'apikey', inputPreview: apiKeyFormat.value, outputPreview: generatedApiKeys.value[0].slice(0, 20) + '...' })
  ElMessage.success(`已生成 ${apiKeyCount.value} 个 API Key`)
}

// ============ 强度检测 Tab ============
const strengthInput = ref('')
const strengthResult = ref<{
  length: number
  charTypes: string[]
  entropy: number
  label: string
  class: string
  percent: number
  suggestion: string
} | null>(null)

const handleStrengthCheck = () => {
  const pwd = strengthInput.value
  if (!pwd) {
    strengthResult.value = null
    return
  }

  const len = pwd.length
  const charTypes: string[] = []
  let charsetSize = 0

  if (/[a-z]/.test(pwd)) { charTypes.push('小写'); charsetSize += 26 }
  if (/[A-Z]/.test(pwd)) { charTypes.push('大写'); charsetSize += 26 }
  if (/[0-9]/.test(pwd)) { charTypes.push('数字'); charsetSize += 10 }
  if (/[^a-zA-Z0-9]/.test(pwd)) { charTypes.push('符号'); charsetSize += 32 }

  // 计算熵值: entropy = length * log2(charsetSize)
  const entropy = charsetSize > 0 ? len * Math.log2(charsetSize) : 0

  // 评分
  let score = 0
  if (len >= 8) score += 1
  if (len >= 12) score += 1
  if (len >= 16) score += 1
  if (charTypes.length >= 2) score += 1
  if (charTypes.length >= 3) score += 1
  if (charTypes.length >= 4) score += 1
  if (entropy >= 60) score += 1

  let label: string, cls: string, percent: number, suggestion: string

  if (score <= 2) {
    label = '弱'
    cls = 'weak'
    percent = 25
    suggestion = len < 8 ? '增加密码长度（至少8位）' : '增加更多字符类型（大写、数字、符号）'
  } else if (score <= 4) {
    label = '中'
    cls = 'medium'
    percent = 50
    suggestion = '继续增加长度或字符类型多样性'
  } else if (score <= 5) {
    label = '强'
    cls = 'strong'
    percent = 75
    suggestion = '已经很安全了！'
  } else {
    label = '极强'
    cls = 'very-strong'
    percent = 100
    suggestion = '非常安全，可以放心使用'
  }

  strengthResult.value = {
    length: len,
    charTypes,
    entropy,
    label,
    class: cls,
    percent,
    suggestion
  }
}

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'strength') {
    strengthInput.value = ''
    strengthResult.value = null
  }
}

const handlePaste = async (tab: string) => {
  try {
    const text = await navigator.clipboard.readText()
    tabState[tab].input = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (tab: string) => {
  const text = tabState[tab].output
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const copyValue = async (value: string) => {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.password-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .password-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.password-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.password-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.password-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.password-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.password-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 工具卡片 ===== */
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
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

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 操作按钮 */
.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

/* 提示图标 */
.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}

/* ===== 密码列表 ===== */
.password-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.password-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.password-content {
  flex: 1;
  min-width: 0;
}

.password-value {
  display: block;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 15px;
  color: var(--text-primary);
  word-break: break-all;
  margin-bottom: 8px;
}

.apikey-value {
  font-size: 13px;
}

/* 强度指示 */
.password-strength {
  display: flex;
  align-items: center;
  gap: 8px;
}

.strength-bar {
  flex: 1;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.strength-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s, background 0.3s;
}

.strength-bar .strength-fill.weak { background: var(--accent-red); }
.strength-bar .strength-fill.medium { background: #f59e0b; }
.strength-bar .strength-fill.strong { background: #22c55e; }
.strength-bar .strength-fill.very-strong { background: var(--accent-cyan); }

.strength-label {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.strength-label.weak { color: var(--accent-red); }
.strength-label.medium { color: #f59e0b; }
.strength-label.strong { color: #22c55e; }
.strength-label.very-strong { color: var(--accent-cyan); }

/* ===== 强度检测结果 ===== */
.strength-result {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.strength-meter {
  display: flex;
  align-items: center;
  gap: 12px;
}

.strength-bar-bg {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.strength-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s, background 0.3s;
}

.strength-bar-fill.weak { background: var(--accent-red); }
.strength-bar-fill.medium { background: #f59e0b; }
.strength-bar-fill.strong { background: #22c55e; }
.strength-bar-fill.very-strong { background: var(--accent-cyan); }

.strength-label-large {
  font-size: 16px;
  font-weight: 700;
  white-space: nowrap;
  min-width: 50px;
  text-align: center;
}

.strength-label-large.weak { color: var(--accent-red); }
.strength-label-large.medium { color: #f59e0b; }
.strength-label-large.strong { color: #22c55e; }
.strength-label-large.very-strong { color: var(--accent-cyan); }

.strength-details {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.detail-item {
  display: flex;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.detail-label {
  color: var(--text-secondary);
  min-width: 60px;
}

.detail-value {
  color: var(--text-primary);
}

/* ===== 空状态 ===== */
.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}
</style>
