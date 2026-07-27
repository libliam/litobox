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
          </div>
          <div class="card-actions">
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
          <span class="count-badge">{{ credentials.length }} 条</span>
        </div>
        <div class="card-body">
          <DataTable :data="credentials" max-height="600">
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
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import DataTable from '@/components/DataTable.vue'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

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

const changePwdVisible = ref(false)
const isChangingPwd = ref(false)
const changePwdError = ref('')
const changePwdForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
})

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
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    isLoading.value = false
  }
}

const handleLock = async () => {
  const ok = await confirm.ask('锁定确认', '确定要锁定密码保管箱吗？', { type: 'warning' })
  if (!ok) return
  
  isUnlocked.value = true
  masterPassword.value = ''
  confirmPassword.value = ''
  credentials.value = []
  ElMessage.success('已锁定')
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

const handleReset = async () => {
  const ok = await confirm.ask(
    '重置主密码',
    '⚠️ 警告：此操作将删除所有已保存的凭据！\n\n' +
    '如果您只是忘记了主密码，可以尝试在数据库文件中找回：\n' +
    '📁 %APPDATA%\\com.dev.toolbox\\litobox.db\n\n' +
    '使用 SQLite 查看工具打开后执行：\n' +
    'SELECT value FROM config WHERE key = "password_vault_master_plain"\n\n' +
    '❌ 确定要重置吗？所有凭据数据将被永久删除！',
    { type: 'danger', confirmText: '确认重置', cancelText: '取消' }
  )
  if (!ok) return

  try {
    await invoke('pv_reset_master_password')
    hasMasterPassword.value = false
    masterPassword.value = ''
    confirmPassword.value = ''
    errorMessage.value = ''
    ElMessage.success('已重置，请设置新的主密码')
  } catch (e) {
    ElMessage.error('重置失败: ' + String(e))
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

const parseCSV = (text: string): Array<{ name: string; url: string; username: string; password: string; notes: string }> => {
  const lines = text.trim().split('\n')
  if (lines.length < 2) return []

  const headerLine = lines[0].trim()
  const headers = headerLine.split(',').map(h => h.trim().toLowerCase())

  const nameIdx = headers.findIndex(h => h === 'name' || h === '网站名称' || h === '显示名称')
  const urlIdx = headers.findIndex(h => h === 'url' || h === '网址' || h === '登录URL')
  const userIdx = headers.findIndex(h => h === 'username' || h === '用户名' || h === '用户名.1')
  const pwdIdx = headers.findIndex(h => h === 'password' || h === '密码')
  const noteIdx = headers.findIndex(h => h === 'note' || h === 'notes' || h === '备注')

  const result: Array<{ name: string; url: string; username: string; password: string; notes: string }> = []

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].trim()
    if (!line) continue

    const fields = line.split(',')
    const name = nameIdx >= 0 ? (fields[nameIdx] || '').trim() : ''
    const url = urlIdx >= 0 ? (fields[urlIdx] || '').trim() : ''
    const username = userIdx >= 0 ? (fields[userIdx] || '').trim() : ''
    const password = pwdIdx >= 0 ? (fields[pwdIdx] || '').trim() : ''
    const notes = noteIdx >= 0 ? (fields[noteIdx] || '').trim() : ''

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

    const ok = await confirm.ask(
      '批量导入',
      `检测到 ${parsed.length} 条凭据，确定要导入吗？`,
      { type: 'info', confirmText: '确认导入', cancelText: '取消' }
    )
    if (!ok) return

    const count = await invoke<number>('pv_import_credentials', {
      masterPassword: masterPassword.value,
      credentials: parsed
    })

    ElMessage.success(`成功导入 ${count} 条凭据`)
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

onMounted(async () => {
  try {
    hasMasterPassword.value = await invoke<boolean>('pv_has_master_password')
  } catch (e) {
    ElMessage.error('初始化失败: ' + String(e))
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
</style>
