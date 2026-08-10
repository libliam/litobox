<template>
  <div class="tool-container">
    <div v-if="isUnlocked" class="lock-screen">
      <div class="lock-card">
        <div class="lock-icon">🔐</div>
        <h2>{{ hasMasterPassword ? '验证主密码' : '设置主密码' }}</h2>
        <p class="lock-hint">{{ hasMasterPassword ? '请输入主密码解锁密码保管箱' : '请设置主密码以保护您的凭据' }}</p>
        <el-input
          v-model="masterPassword"
          type="password"
          placeholder="输入密码"
          show-password
          @keyup.enter="handleUnlock"
          style="margin-bottom: 16px"
        />
        <el-input
          v-if="!hasMasterPassword"
          v-model="confirmPassword"
          type="password"
          placeholder="确认密码"
          show-password
          @keyup.enter="handleUnlock"
          style="margin-bottom: 16px"
        />
        <el-button type="primary" @click="handleUnlock" :loading="isLoading">
          {{ hasMasterPassword ? '解锁' : '设置' }}
        </el-button>
        <el-button v-if="hasMasterPassword" text @click="handleReset" style="margin-top: 12px">
          忘记密码？重置主密码
        </el-button>
        <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
      </div>
    </div>

    <div v-else class="vault-content">
      <div class="tool-card sticky-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">密码保管箱</span>
            <el-tooltip placement="top" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>本地加密存储您的网站凭据</p>
                  <p>所有数据仅存储在本地，不会上传</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
            <span v-if="autoLockEnabled" class="autolock-badge" :title="`${autoLockMinutes} 分钟无操作自动锁定`">
              <el-icon><Clock /></el-icon>
              {{ remainingMinutesDisplay }}
            </span>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="settingsVisible = true">⚙️ 设置</el-button>
            <el-button size="small" @click="handleExport">导出</el-button>
            <div class="import-wrapper">
              <el-button size="small" @click="handleImport">批量导入</el-button>
              <el-tooltip placement="bottom" effect="dark">
                <template #content>
                  <div class="import-tip">
                    <p><strong>支持格式</strong></p>
                    <p>Edge / Chrome 浏览器导出的 CSV 文件</p>
                    <p style="margin-top: 6px;"><strong>CSV 表头</strong></p>
                    <p>name, url, username, password, note</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <el-button size="small" @click="handleAdd">添加凭据</el-button>
            <el-button size="small" @click="changePwdVisible = true">修改密码</el-button>
            <el-button size="small" type="danger" @click="handleLock">🔒 锁定</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="search-bar">
            <el-input
              v-model="searchQuery"
              placeholder="搜索网站、用户名..."
              clearable
              @input="handleSearch"
              style="width: 300px"
            />
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">凭据列表</span>
          <div class="header-right">
            <span class="count-badge">{{ credentials.length }} 条</span>
            <el-button v-if="selectedIds.length > 0" size="small" type="danger" @click="handleBatchDelete">
              批量删除 ({{ selectedIds.length }})
            </el-button>
          </div>
        </div>
        <div class="card-body">
          <DataTable ref="tableRef" :data="credentials" max-height="600" @selection-change="onSelectionChange">
            <el-table-column type="selection" width="40" />
            <el-table-column prop="name" label="网站" min-width="150" />
            <el-table-column prop="url" label="网址" min-width="200" />
            <el-table-column prop="username" label="用户名" min-width="120" />
            <el-table-column prop="password" label="密码" min-width="150">
              <template #default="scope">
                <span>{{ '*'.repeat(scope.row.password.length) }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="notes" label="备注" min-width="150" />
            <el-table-column label="操作" width="200" fixed="right">
              <template #default="scope">
                <div class="action-buttons">
                  <el-button size="small" @click="handleCopy(scope.row.password)">复制</el-button>
                  <el-button size="small" @click="handleEdit(scope.row)">编辑</el-button>
                  <el-button size="small" type="danger" @click="handleDelete(scope.row.id)">删除</el-button>
                </div>
              </template>
            </el-table-column>
          </DataTable>
        </div>
      </div>

      <el-dialog
        v-model="dialogVisible"
        :title="isEditing ? '编辑凭据' : '添加凭据'"
        width="500px"
      >
        <el-form :model="formData" label-width="80px">
          <el-form-item label="网站名称" required>
            <el-input v-model="formData.name" placeholder="如：GitHub" />
          </el-form-item>
          <el-form-item label="网址">
            <el-input v-model="formData.url" placeholder="如：https://github.com" />
          </el-form-item>
          <el-form-item label="用户名" required>
            <el-input v-model="formData.username" placeholder="输入用户名" />
          </el-form-item>
          <el-form-item label="密码" required>
            <el-input v-model="formData.password" type="password" show-password placeholder="输入密码" />
          </el-form-item>
          <el-form-item label="备注">
            <el-input v-model="formData.notes" type="textarea" :rows="3" placeholder="备注信息" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSave">保存</el-button>
        </template>
      </el-dialog>

      <el-dialog v-model="changePwdVisible" title="修改主密码" width="420px">
        <el-form label-width="100px">
          <el-form-item label="当前密码" required>
            <el-input v-model="changePwdForm.oldPassword" type="password" show-password placeholder="输入当前主密码" />
          </el-form-item>
          <el-form-item label="新密码" required>
            <el-input v-model="changePwdForm.newPassword" type="password" show-password placeholder="输入新密码（至少4位）" />
          </el-form-item>
          <el-form-item label="确认新密码" required>
            <el-input v-model="changePwdForm.confirmPassword" type="password" show-password placeholder="再次输入新密码" />
          </el-form-item>
        </el-form>
        <p v-if="changePwdError" class="change-pwd-error">{{ changePwdError }}</p>
        <template #footer>
          <el-button @click="changePwdVisible = false">取消</el-button>
          <el-button type="primary" @click="handleChangePassword" :loading="isChangingPwd">确认修改</el-button>
        </template>
      </el-dialog>

      <input
        ref="fileInputRef"
        type="file"
        accept=".csv"
        style="display: none"
        @change="handleFileSelect"
      />
    </div>
  </div>

  <el-dialog v-model="settingsVisible" title="密码保管箱设置" width="420px">
    <el-form label-width="120px">
      <el-form-item label="自动锁定">
        <el-switch v-model="autoLockEnabled" active-text="开启  无操作自动锁定" />
      </el-form-item>
      <el-form-item label="锁定时长">
        <el-select v-model="autoLockMinutes" :disabled="!autoLockEnabled" style="width: 160px">
          <el-option :value="1" label="1 分钟" />
          <el-option :value="3" label="3 分钟" />
          <el-option :value="5" label="5 分钟" />
          <el-option :value="10" label="10 分钟" />
          <el-option :value="15" label="15 分钟" />
          <el-option :value="30" label="30 分钟" />
        </el-select>
        <span class="form-hint">空闲 {{ autoLockMinutes }} 分钟后自动锁定</span>
      </el-form-item>
    </el-form>
    <p v-if="settingsError" class="change-pwd-error">{{ settingsError }}</p>
    <template #footer>
      <el-button @click="settingsVisible = false">取消</el-button>
      <el-button type="primary" @click="handleSaveSettings" :loading="isSavingSettings">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="resetConfirmVisible" title="⚠️ 重置主密码" width="480px" :close-on-click-modal="false" @open="generateResetChallenge">
    <div class="reset-warning">
      <p class="reset-warning-title">此操作将删除所有已保存的凭据！</p>
      <p class="reset-warning-desc">所有凭据数据将被永久删除，此操作不可撤销。</p>
    </div>

    <div class="reset-challenge-section">
      <p class="reset-challenge-label">请在下方输入框中手动输入以下验证字符串：</p>
      <div class="reset-challenge-code">
        <code>{{ resetChallenge }}</code>
        <el-tooltip content="刷新验证码" placement="top">
          <el-button size="small" circle @click="generateResetChallenge" class="refresh-btn">
            <el-icon><Refresh /></el-icon>
          </el-button>
        </el-tooltip>
      </div>
      <el-input
        v-model="resetChallengeInput"
        placeholder="请手动输入上方的验证字符串"
        @paste="onResetInputPaste"
        @input="watchResetInput"
        :disabled="resetConfirmLoading"
      />
      <p v-if="resetInputError" class="reset-input-error">{{ resetInputError }}</p>
    </div>

    <template #footer>
      <el-button @click="resetConfirmVisible = false" :disabled="resetConfirmLoading">取消</el-button>
      <el-button
        type="danger"
        @click="handleConfirmReset"
        :loading="resetConfirmLoading"
        :disabled="resetChallengeInput.length === 0"
      >确认重置</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Refresh, Clock } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import DataTable from '@/components/DataTable.vue'
import { useConfirmDialog } from '@/composables/useConfirmDialog'
import * as db from '@/utils/dbClient'

const { confirm } = useConfirmDialog()

interface Credential {
  id: number
  name: string
  url: string
  username: string
  password: string
  notes: string
  created_at: string
  updated_at: string
}

const isUnlocked = ref(true)
const hasMasterPassword = ref(false)
const masterPassword = ref('')
const confirmPassword = ref('')
const isLoading = ref(false)
const errorMessage = ref('')

const credentials = ref<Credential[]>([])
const searchQuery = ref('')
const fileInputRef = ref<HTMLInputElement | null>(null)
const isImporting = ref(false)
const tableRef = ref<any>(null)
const selectedIds = ref<number[]>([])

const onSelectionChange = (rows: Credential[]) => {
  selectedIds.value = rows.map(r => r.id)
}

const changePwdVisible = ref(false)
const isChangingPwd = ref(false)
const changePwdError = ref('')
const changePwdForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
})

// ============ 自动锁定配置 ============
const AUTOLOCK_CONFIG_KEY_ENABLED = 'password_vault_autolock_enabled'
const AUTOLOCK_CONFIG_KEY_MINUTES = 'password_vault_autolock_minutes'
const AUTOLOCK_DEFAULT_ENABLED = false
const AUTOLOCK_DEFAULT_MINUTES = 5

const settingsVisible = ref(false)
const isSavingSettings = ref(false)
const settingsError = ref('')
const autoLockEnabled = ref(AUTOLOCK_DEFAULT_ENABLED)
const autoLockMinutes = ref(AUTOLOCK_DEFAULT_MINUTES)

let lastActivityAt = 0                  // 上次用户活动时间戳（ms）
let idleCheckTimer: ReturnType<typeof setInterval> | null = null
let idleDisplayTick = ref(0)            // 每秒自增，驱动剩余时间显示重算

const remainingMinutesDisplay = computed(() => {
  // 依赖 idleDisplayTick，确保每秒重算
  void idleDisplayTick.value
  if (!autoLockEnabled.value) return ''
  const elapsedMs = Date.now() - lastActivityAt
  const remainingSec = Math.max(0, autoLockMinutes.value * 60 - Math.floor(elapsedMs / 1000))
  const m = Math.floor(remainingSec / 60)
  const s = remainingSec % 60
  return `${m}:${s.toString().padStart(2, '0')}`
})

const resetIdleTimer = () => {
  // ponytail: 只在已解锁状态更新 lastActivity，避免锁定后用户操作还在计时
  if (!isUnlocked.value) {
    lastActivityAt = Date.now()
  }
}

const doLock = (silent: boolean = false) => {
  isUnlocked.value = true
  masterPassword.value = ''
  confirmPassword.value = ''
  credentials.value = []
  if (idleCheckTimer) {
    clearInterval(idleCheckTimer)
    idleCheckTimer = null
  }
  if (!silent) ElMessage.success('已锁定')
}

const startIdleChecker = () => {
  if (idleCheckTimer) clearInterval(idleCheckTimer)
  lastActivityAt = Date.now()
  idleDisplayTick.value = 0
  idleCheckTimer = setInterval(() => {
    idleDisplayTick.value++
    if (!autoLockEnabled.value) return
    if (isUnlocked.value) return               // 已锁定状态跳过
    const elapsed = Date.now() - lastActivityAt
    if (elapsed >= autoLockMinutes.value * 60 * 1000) {
      // 自动锁定
      const mins = autoLockMinutes.value
      doLock(true)
      ElMessage.info(`空闲 ${mins} 分钟，密码保管箱已自动锁定`)
    }
  }, 1000)
}

const loadAutoLockSettings = async () => {
  try {
    const enabledStr = await db.getConfig(AUTOLOCK_CONFIG_KEY_ENABLED).catch(() => '')
    const minutesStr = await db.getConfig(AUTOLOCK_CONFIG_KEY_MINUTES).catch(() => '')
    autoLockEnabled.value = enabledStr === '' ? AUTOLOCK_DEFAULT_ENABLED : enabledStr === 'true'
    const m = parseInt(minutesStr, 10)
    autoLockMinutes.value = Number.isFinite(m) && [1, 3, 5, 10, 15, 30].includes(m) ? m : AUTOLOCK_DEFAULT_MINUTES
  } catch (e) {
    console.warn('加载自动锁定配置失败，使用默认值:', e)
    autoLockEnabled.value = AUTOLOCK_DEFAULT_ENABLED
    autoLockMinutes.value = AUTOLOCK_DEFAULT_MINUTES
  }
}

const handleSaveSettings = async () => {
  settingsError.value = ''
  isSavingSettings.value = true
  try {
    await db.setConfig(AUTOLOCK_CONFIG_KEY_ENABLED, String(autoLockEnabled.value))
    await db.setConfig(AUTOLOCK_CONFIG_KEY_MINUTES, String(autoLockMinutes.value))
    // 如果已经解锁且启用了自动锁定，重置活动时间并启动/重启定时器
    if (!isUnlocked.value && autoLockEnabled.value) {
      startIdleChecker()
    }
    settingsVisible.value = false
    ElMessage.success('设置已保存')
  } catch (e) {
    settingsError.value = String(e)
  } finally {
    isSavingSettings.value = false
  }
}


// 重置密码二次确认
const resetConfirmVisible = ref(false)
const resetChallenge = ref('')
const resetChallengeInput = ref('')
const resetInputError = ref('')
const resetConfirmLoading = ref(false)

const generateResetChallenge = () => {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+'
  let result = ''
  for (let i = 0; i < 15; i++) {
    result += chars[Math.floor(Math.random() * chars.length)]
  }
  resetChallenge.value = result
  resetChallengeInput.value = ''
  resetInputError.value = ''
}

const onResetInputPaste = (e: ClipboardEvent) => {
  e.preventDefault()
}

const watchResetInput = () => {
  resetInputError.value = ''
}

const dialogVisible = ref(false)
const isEditing = ref(false)
const formData = reactive({
  id: 0,
  name: '',
  url: '',
  username: '',
  password: '',
  notes: ''
})

const loadCredentials = async () => {
  try {
    credentials.value = await invoke<Credential[]>('pv_list_credentials', {
      masterPassword: masterPassword.value
    })
  } catch (e) {
    ElMessage.error('加载凭据失败: ' + String(e))
  }
}

const handleUnlock = async () => {
  if (!masterPassword.value.trim()) {
    errorMessage.value = '请输入密码'
    return
  }

  if (!hasMasterPassword.value && masterPassword.value !== confirmPassword.value) {
    errorMessage.value = '两次输入的密码不一致'
    return
  }

  isLoading.value = true
  errorMessage.value = ''

  try {
    if (!hasMasterPassword.value) {
      await invoke('pv_set_master_password', { password: masterPassword.value })
      ElMessage.success('主密码设置成功')
    } else {
      const verified = await invoke<boolean>('pv_verify_master_password', {
        password: masterPassword.value
      })
      if (!verified) {
        errorMessage.value = '密码错误'
        isLoading.value = false
        return
      }
    }

    isUnlocked.value = false
    await loadCredentials()
    // 解锁成功后启动空闲检测（如启用自动锁定）
    if (autoLockEnabled.value) {
      startIdleChecker()
    }
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    isLoading.value = false
  }
}

const handleLock = async () => {
  const ok = await confirm.ask('锁定确认', '确定要锁定密码保管箱吗？', { type: 'warning' })
  if (!ok) return
  doLock(false)
}

const handleChangePassword = async () => {
  changePwdError.value = ''
  
  if (!changePwdForm.oldPassword.trim()) {
    changePwdError.value = '请输入当前密码'
    return
  }
  if (changePwdForm.newPassword.length < 4) {
    changePwdError.value = '新密码长度至少4位'
    return
  }
  if (changePwdForm.newPassword !== changePwdForm.confirmPassword) {
    changePwdError.value = '两次输入的新密码不一致'
    return
  }

  isChangingPwd.value = true
  try {
    await invoke('pv_change_master_password', {
      oldPassword: changePwdForm.oldPassword,
      newPassword: changePwdForm.newPassword
    })
    masterPassword.value = changePwdForm.newPassword
    changePwdVisible.value = false
    changePwdForm.oldPassword = ''
    changePwdForm.newPassword = ''
    changePwdForm.confirmPassword = ''
    ElMessage.success('主密码修改成功')
  } catch (e) {
    changePwdError.value = String(e)
  } finally {
    isChangingPwd.value = false
  }
}

const handleReset = () => {
  generateResetChallenge()
  resetConfirmVisible.value = true
}

const handleConfirmReset = async () => {
  resetInputError.value = ''

  if (resetChallengeInput.value !== resetChallenge.value) {
    resetInputError.value = '输入的验证字符串不匹配，请仔细核对'
    return
  }

  resetConfirmLoading.value = true
  try {
    await invoke('pv_reset_master_password')
    hasMasterPassword.value = false
    masterPassword.value = ''
    confirmPassword.value = ''
    errorMessage.value = ''
    resetConfirmVisible.value = false
    // 重置主密码会清掉 SQL 里 password_vault_% 配置，内存也同步为默认值
    autoLockEnabled.value = AUTOLOCK_DEFAULT_ENABLED
    autoLockMinutes.value = AUTOLOCK_DEFAULT_MINUTES
    if (idleCheckTimer) {
      clearInterval(idleCheckTimer)
      idleCheckTimer = null
    }
    ElMessage.success('已重置，请设置新的主密码')
  } catch (e) {
    ElMessage.error('重置失败: ' + String(e))
  } finally {
    resetConfirmLoading.value = false
  }
}

const handleSearch = async () => {
  if (!searchQuery.value.trim()) {
    await loadCredentials()
    return
  }

  try {
    credentials.value = await invoke<Credential[]>('pv_search_credentials', {
      masterPassword: masterPassword.value,
      query: searchQuery.value
    })
  } catch (e) {
    ElMessage.error('搜索失败: ' + String(e))
  }
}

const handleCopy = async (password: string) => {
  try {
    await navigator.clipboard.writeText(password)
    ElMessage.success('密码已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleImport = () => {
  fileInputRef.value?.click()
}

const escapeCSV = (value: string): string => {
  if (!value) return ''
  if (value.includes(',') || value.includes('"') || value.includes('\n') || value.includes('\r')) {
    return '"' + value.replace(/"/g, '""') + '"'
  }
  return value
}

const handleExport = async () => {
  if (credentials.value.length === 0) {
    ElMessage.warning('没有凭据可导出')
    return
  }

  const header = 'name,url,username,password,notes'
  const rows = credentials.value.map(c =>
    [c.name, c.url, c.username, c.password, c.notes].map(escapeCSV).join(',')
  )
  const csv = '\uFEFF' + header + '\n' + rows.join('\n')
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
  const filename = `password_vault_${timestamp}.csv`

  try {
    const savedPath = await invoke<string>('save_text_with_dialog', { content: csv, filename })
    if (savedPath !== 'cancelled') {
      ElMessage.success(`已导出到: ${savedPath}`)
    }
  } catch (e) {
    ElMessage.error('导出失败: ' + String(e))
  }
}

const splitCSVLine = (line: string): string[] => {
  const fields: string[] = []
  let current = ''
  let inQuotes = false

  for (let i = 0; i < line.length; i++) {
    const ch = line[i]
    if (ch === '"') {
      // 处理转义的双引号 ""
      if (inQuotes && i + 1 < line.length && line[i + 1] === '"') {
        current += '"'
        i++ // 跳过下一个双引号
      } else {
        inQuotes = !inQuotes
      }
    } else if (ch === ',' && !inQuotes) {
      fields.push(current)
      current = ''
    } else {
      current += ch
    }
  }
  fields.push(current)
  return fields
}

const parseCSV = (text: string): Array<{ name: string; url: string; username: string; password: string; notes: string }> => {
  const lines = text.trim().split('\n')
  if (lines.length < 2) return []

  const headerLine = lines[0].trim()
  const headers = splitCSVLine(headerLine).map(h => h.trim().toLowerCase().replace(/^"/, '').replace(/"$/, ''))

  const nameIdx = headers.findIndex(h => h === 'name' || h === '网站名称' || h === '显示名称')
  const urlIdx = headers.findIndex(h => h === 'url' || h === '网址' || h === '登录URL')
  const userIdx = headers.findIndex(h => h === 'username' || h === '用户名' || h === '用户名.1')
  const pwdIdx = headers.findIndex(h => h === 'password' || h === '密码')
  const noteIdx = headers.findIndex(h => h === 'note' || h === 'notes' || h === '备注')

  const result: Array<{ name: string; url: string; username: string; password: string; notes: string }> = []

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].trim()
    if (!line) continue

    const fields = splitCSVLine(line)
    const name = nameIdx >= 0 ? (fields[nameIdx] || '').trim().replace(/^"/, '').replace(/"$/, '') : ''
    const url = urlIdx >= 0 ? (fields[urlIdx] || '').trim().replace(/^"/, '').replace(/"$/, '') : ''
    const username = userIdx >= 0 ? (fields[userIdx] || '').trim().replace(/^"/, '').replace(/"$/, '') : ''
    const password = pwdIdx >= 0 ? (fields[pwdIdx] || '').trim().replace(/^"/, '').replace(/"$/, '') : ''
    const notes = noteIdx >= 0 ? (fields[noteIdx] || '').trim().replace(/^"/, '').replace(/"$/, '') : ''

    if (name && password) {
      result.push({ name, url, username, password, notes })
    }
  }

  return result
}

const handleFileSelect = async (e: Event) => {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  try {
    isImporting.value = true
    const text = await file.text()
    const parsed = parseCSV(text)

    if (parsed.length === 0) {
      ElMessage.warning('未解析到有效数据')
      return
    }

    // 检查重复
    const duplicates = await invoke<Array<{ index: number; name: string; url: string; username: string }>>('pv_check_duplicates', {
      credentials: parsed
    })

    let toImport = parsed
    if (duplicates.length > 0) {
      const maxShow = 20
      const dupNames = duplicates.slice(0, maxShow).map(d => `  ${d.name} - ${d.username}`).join('\n')
      const suffix = duplicates.length > maxShow ? `\n  ... 及其他 ${duplicates.length - maxShow} 条` : ''
      const ok = await confirm.ask(
        '发现重复数据',
        `检测到 ${duplicates.length} 条重复凭据（基于网站名称+用户名判断）：\n\n${dupNames}${suffix}\n\n跳过重复项，只导入新数据？`,
        { type: 'warning', confirmText: '跳过重复导入', cancelText: '取消' }
      )
      if (!ok) return
      const dupIndices = new Set(duplicates.map(d => d.index))
      toImport = parsed.filter((_, i) => !dupIndices.has(i))
    }

    if (toImport.length === 0) {
      ElMessage.warning('无新数据可导入')
      return
    }

    if (duplicates.length === 0) {
      const ok = await confirm.ask(
        '批量导入',
        `检测到 ${parsed.length} 条凭据，确定要导入吗？`,
        { type: 'info', confirmText: '确认导入', cancelText: '取消' }
      )
      if (!ok) return
    }

    const count = await invoke<number>('pv_import_credentials', {
      masterPassword: masterPassword.value,
      credentials: toImport
    })

    ElMessage.success(`成功导入 ${count} 条凭据` + (duplicates.length > 0 ? `，已跳过 ${duplicates.length} 条重复` : ''))
    await loadCredentials()
  } catch (e) {
    ElMessage.error('导入失败: ' + String(e))
  } finally {
    isImporting.value = false
    if (fileInputRef.value) {
      fileInputRef.value.value = ''
    }
  }
}

const handleAdd = () => {
  isEditing.value = false
  formData.id = 0
  formData.name = ''
  formData.url = ''
  formData.username = ''
  formData.password = ''
  formData.notes = ''
  dialogVisible.value = true
}

const handleEdit = (credential: Credential) => {
  isEditing.value = true
  formData.id = credential.id
  formData.name = credential.name
  formData.url = credential.url
  formData.username = credential.username
  formData.password = credential.password
  formData.notes = credential.notes
  dialogVisible.value = true
}

const handleDelete = async (id: number) => {
  const ok = await confirm.ask('删除确认', '确定要删除这条凭据吗？', { type: 'danger' })
  if (!ok) return

  try {
    await invoke('pv_delete_credential', { id })
    await loadCredentials()
    ElMessage.success('删除成功')
  } catch (e) {
    ElMessage.error('删除失败: ' + String(e))
  }
}

const handleBatchDelete = async () => {
  const ok = await confirm.ask(
    '批量删除',
    `确定要删除选中的 ${selectedIds.value.length} 条凭据吗？此操作不可恢复。`,
    { type: 'danger', confirmText: '确认删除', cancelText: '取消' }
  )
  if (!ok) return

  try {
    const count = await invoke<number>('pv_batch_delete', { ids: selectedIds.value })
    selectedIds.value = []
    await loadCredentials()
    ElMessage.success(`已删除 ${count} 条凭据`)
  } catch (e) {
    ElMessage.error('批量删除失败: ' + String(e))
  }
}

const handleSave = async () => {
  if (!formData.name.trim()) {
    ElMessage.warning('请输入网站名称')
    return
  }
  if (!formData.username.trim()) {
    ElMessage.warning('请输入用户名')
    return
  }
  if (!formData.password.trim()) {
    ElMessage.warning('请输入密码')
    return
  }

  try {
    if (isEditing.value) {
      await invoke('pv_update_credential', {
        masterPassword: masterPassword.value,
        credential: {
          id: formData.id,
          name: formData.name,
          url: formData.url,
          username: formData.username,
          password: formData.password,
          notes: formData.notes
        }
      })
      ElMessage.success('更新成功')
    } else {
      await invoke('pv_add_credential', {
        masterPassword: masterPassword.value,
        credential: {
          name: formData.name,
          url: formData.url,
          username: formData.username,
          password: formData.password,
          notes: formData.notes
        }
      })
      ElMessage.success('添加成功')
    }
    dialogVisible.value = false
    await loadCredentials()
  } catch (e) {
    ElMessage.error('保存失败: ' + String(e))
  }
}

const IDLE_EVENTS = ['mousemove', 'mousedown', 'keydown', 'click', 'scroll', 'wheel'] as const

onMounted(async () => {
  try {
    hasMasterPassword.value = await invoke<boolean>('pv_has_master_password')
  } catch (e) {
    ElMessage.error('初始化失败: ' + String(e))
  }
  await loadAutoLockSettings()
  // 注册全局活动事件监听（整个 window 级别，用户任何操作都重置计时）
  for (const ev of IDLE_EVENTS) {
    window.addEventListener(ev, resetIdleTimer, { passive: true })
  }
})

onUnmounted(() => {
  for (const ev of IDLE_EVENTS) {
    window.removeEventListener(ev, resetIdleTimer)
  }
  if (idleCheckTimer) {
    clearInterval(idleCheckTimer)
    idleCheckTimer = null
  }
})
</script>

<style scoped>
.lock-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.lock-card {
  width: 400px;
  padding: 40px;
  background: var(--bg-card);
  border-radius: 12px;
  text-align: center;
  border: 1px solid var(--border-color);
}

.lock-icon {
  font-size: 64px;
  margin-bottom: 20px;
}

.lock-card h2 {
  color: var(--accent-cyan);
  font-size: 24px;
  margin-bottom: 8px;
}

.lock-hint {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 24px;
}

.error-message {
  color: var(--accent-red);
  font-size: 13px;
  margin-top: 12px;
}

.search-bar {
  display: flex;
  gap: 12px;
}

.search-bar :deep(.el-input__wrapper) {
  background: var(--bg-input);
}

.search-bar :deep(.el-input__inner) {
  background: transparent;
}

.search-bar :deep(.el-input__clear) {
  color: var(--text-secondary);
}

.import-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.import-tip {
  font-size: 12px;
  line-height: 1.6;
  text-align: left;
}

.import-tip p {
  margin: 0;
}

.count-badge {
  font-size: 13px;
  color: var(--text-secondary);
  background: var(--bg-input);
  padding: 4px 12px;
  border-radius: 12px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.change-pwd-error {
  color: var(--accent-red);
  font-size: 13px;
  margin-top: -8px;
  margin-bottom: 0;
  padding: 0 20px;
}



.action-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 100%;
  padding: 8px 0;
}

:deep(.el-table__cell) {
  vertical-align: middle !important;
}

:deep(.el-table__fixed-right .el-table__cell) {
  vertical-align: middle !important;
}

.tooltip-content {
  max-width: 300px;
}

.reset-warning {
  background: var(--bg-active);
  border: 1px solid var(--accent-red);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.reset-warning-title {
  color: var(--accent-red);
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 6px 0;
}

.reset-warning-desc {
  color: var(--text-secondary);
  font-size: 13px;
  margin: 0;
}

.reset-challenge-section {
  margin-bottom: 8px;
}

.reset-challenge-label {
  color: var(--text-primary);
  font-size: 14px;
  margin: 0 0 12px 0;
}

.reset-challenge-code {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.reset-challenge-code code {
  flex: 1;
  display: block;
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 18px;
  letter-spacing: 2px;
  text-align: center;
  color: var(--accent-cyan);
  user-select: all;
}

.refresh-btn {
  flex-shrink: 0;
}

.reset-input-error {
  color: var(--accent-red);
  font-size: 13px;
  margin-top: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.autolock-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--accent-cyan);
  background: var(--bg-active);
  border: 1px solid var(--accent-cyan);
  border-radius: 12px;
  padding: 2px 10px;
  font-family: 'Consolas', 'Courier New', monospace;
  letter-spacing: 0.5px;
  animation: autolock-pulse 2s ease-in-out infinite;
}

@keyframes autolock-pulse {
  0%, 100% { opacity: 0.85; }
  50% { opacity: 1; }
}

.form-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 8px;
}
</style>
