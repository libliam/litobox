<template>
  <div class="tool-container">
    <!-- admin-banner -->
    <div v-if="!isAdmin" class="admin-banner">
      <span class="admin-icon">🛡️</span>
      编辑 hosts 文件需要<strong>管理员权限</strong>。请以管理员身份运行栗的百宝箱后再操作。
    </div>

    <!-- Tab 栏（sticky） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="hosts-tabs">
        <el-tab-pane label="Hosts 编辑" name="editor" />
        <el-tab-pane label="Profile 管理" name="profiles" />
        <el-tab-pane label="备份恢复" name="backups" />
      </el-tabs>
    </div>

    <!-- Tab 1: 编辑 -->
    <div v-if="activeTab === 'editor'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Hosts 条目 ({{ entries.length }} 条，启用 {{ enabledCount }} 条)</span>
        <div class="card-actions">
          <el-input v-model="searchQuery" size="small" placeholder="搜索 IP/域名/备注..." style="width: 200px" clearable />
          <el-button size="small" @click="loadHosts" :loading="loading">刷新</el-button>
          <el-button type="primary" size="small" @click="addEntry">添加条目</el-button>
          <el-button type="success" size="small" :disabled="!isAdmin" :loading="saving" @click="saveHosts">保存</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="filteredEntries" border size="small" max-height="600" style="width: 100%" v-loading="loading">
          <el-table-column label="启用" width="70">
            <template #default="{ row }">
              <el-checkbox v-model="row.enabled" />
            </template>
          </el-table-column>
          <el-table-column label="IP 地址" width="180">
            <template #default="{ row }">
              <el-input v-model="row.ip" size="small" placeholder="127.0.0.1" />
            </template>
          </el-table-column>
          <el-table-column label="域名" min-width="280">
            <template #default="{ row }">
              <el-input
                v-model="row.domainsText"
                size="small"
                type="textarea"
                :autosize="{ minRows: 1, maxRows: 3 }"
                placeholder="example.com api.example.com"
              />
            </template>
          </el-table-column>
          <el-table-column label="备注" width="200">
            <template #default="{ row }">
              <el-input v-model="row.comment" size="small" placeholder="备注（可选）" />
            </template>
          </el-table-column>
          <el-table-column label="操作" width="80" fixed="right">
            <template #default="{ $index }">
              <el-button type="danger" size="small" link @click="removeEntry($index)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Tab 2: Profile 管理 -->
    <div v-if="activeTab === 'profiles'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Profile 列表 ({{ profiles.length }})</span>
        <div class="card-actions">
          <el-button size="small" @click="loadProfiles" :loading="profilesLoading">刷新</el-button>
          <el-button size="small" @click="showNewProfileDialog">新建 Profile</el-button>
          <el-button type="primary" size="small" @click="saveCurrentAsProfile">从当前 Hosts 保存为 Profile</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="profiles" border size="small" style="width: 100%" v-loading="profilesLoading">
          <el-table-column prop="name" label="名称" width="160" />
          <el-table-column prop="entry_count" label="条目数" width="100" />
          <el-table-column prop="updated_at" label="更新时间" min-width="180" />
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button type="primary" size="small" link @click="applyProfile(row.name)">切换</el-button>
              <el-button v-if="!row.is_default" type="danger" size="small" link @click="deleteProfile(row.name)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- Tab 3: 备份恢复 -->
    <div v-if="activeTab === 'backups'" class="tool-card">
      <div class="card-header">
        <span class="card-title">备份列表 ({{ backups.length }})</span>
        <div class="card-actions">
          <el-button size="small" @click="loadBackups" :loading="backupsLoading">刷新</el-button>
          <el-button type="primary" size="small" @click="createBackupNow">立即备份</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="backups" border size="small" style="width: 100%" v-loading="backupsLoading">
          <el-table-column prop="timestamp" label="时间" min-width="180" />
          <el-table-column label="大小" width="100">
            <template #default="{ row }">{{ formatSize(row.size) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="200" fixed="right">
            <template #default="{ row }">
              <el-button size="small" link @click="previewBackup(row.filename)">预览</el-button>
              <el-button type="warning" size="small" link @click="restoreBackup(row.filename)">恢复</el-button>
              <el-button type="danger" size="small" link @click="deleteBackup(row.filename)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

// ============ Tab 状态 ============
const activeTab = ref('editor')

// ============ 数据 ============
interface HostsEntry {
  enabled: boolean
  ip: string
  domains: string[]
  comment: string
  // 前端编辑用：域名文本（空格分隔）
  domainsText: string
}

const entries = ref<HostsEntry[]>([])
const isAdmin = ref(false)
const loading = ref(false)
const saving = ref(false)
const searchQuery = ref('')

// ============ Profile 数据 ============
interface ProfileMeta {
  name: string
  entry_count: number
  updated_at: string
  is_default: boolean
}

interface BackupInfo {
  filename: string
  timestamp: string
  size: number
  path: string
}

const profiles = ref<ProfileMeta[]>([])
const profilesLoading = ref(false)
const backups = ref<BackupInfo[]>([])
const backupsLoading = ref(false)

// ============ Profile 方法 ============
async function loadProfiles() {
  profilesLoading.value = true
  try {
    profiles.value = await invoke<ProfileMeta[]>('hosts_profile_list')
  } catch (e) {
    ElMessage.error(`加载 profile 列表失败: ${e}`)
  } finally {
    profilesLoading.value = false
  }
}

async function applyProfile(name: string) {
  try {
    await ElMessageBox.confirm(
      `切换到 profile "${name}"？当前 hosts 将被覆盖（会自动备份）。`,
      '确认切换',
      { type: 'warning' }
    )
    await invoke('hosts_profile_apply', { name })
    ElMessage.success(`已切换到 profile: ${name}`)
    await loadHosts()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`切换失败: ${e}`)
  }
}

async function deleteProfile(name: string) {
  try {
    await ElMessageBox.confirm(`删除 profile "${name}"？此操作不可恢复。`, '确认删除', { type: 'warning' })
    await invoke('hosts_profile_delete', { name })
    ElMessage.success('已删除')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`删除失败: ${e}`)
  }
}

async function showNewProfileDialog() {
  try {
    const { value } = await ElMessageBox.prompt('输入 profile 名称', '新建 Profile', {
      inputPattern: /^[^<>:"/\\|?*]+$/,
      inputErrorMessage: '名称包含非法字符'
    })
    await invoke('hosts_profile_save', { name: value, entries: [] })
    ElMessage.success('已创建空 profile')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`创建失败: ${e}`)
  }
}

async function saveCurrentAsProfile() {
  try {
    const { value } = await ElMessageBox.prompt('输入 profile 名称（已存在则覆盖）', '从当前 Hosts 保存', {
      inputPattern: /^[^<>:"/\\|?*]+$/,
      inputErrorMessage: '名称包含非法字符'
    })
    const payload = entries.value.map(e => ({
      enabled: e.enabled,
      ip: e.ip,
      domains: e.domainsText.split(/\s+/).filter(s => s.length > 0),
      comment: e.comment
    }))
    await invoke('hosts_profile_save', { name: value, entries: payload })
    ElMessage.success('已保存为 profile')
    await loadProfiles()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`保存失败: ${e}`)
  }
}

// ============ 备份方法 ============
async function loadBackups() {
  backupsLoading.value = true
  try {
    backups.value = await invoke<BackupInfo[]>('hosts_list_backups')
  } catch (e) {
    ElMessage.error(`加载备份列表失败: ${e}`)
  } finally {
    backupsLoading.value = false
  }
}

async function createBackupNow() {
  try {
    await invoke('hosts_create_backup')
    ElMessage.success('已创建备份')
    await loadBackups()
  } catch (e) {
    ElMessage.error(`备份失败: ${e}`)
  }
}

async function previewBackup(filename: string) {
  try {
    const content = await invoke<string>('hosts_preview_backup', { filename })
    const escaped = content.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    await ElMessageBox.alert(`<pre style="white-space: pre-wrap; word-break: break-all; max-height: 60vh; overflow-y: auto; font-size: 12px;">${escaped}</pre>`, `预览: ${filename}`, {
      dangerouslyUseHTMLString: true,
      confirmButtonText: '关闭'
    })
  } catch (e) {
    ElMessage.error(`预览失败: ${e}`)
  }
}

async function restoreBackup(filename: string) {
  try {
    await ElMessageBox.confirm(
      `恢复备份 "${filename}"？当前 hosts 将被覆盖（会自动备份当前）。`,
      '确认恢复',
      { type: 'warning' }
    )
    await invoke('hosts_restore_backup', { filename })
    ElMessage.success('已恢复')
    await loadHosts()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`恢复失败: ${e}`)
  }
}

async function deleteBackup(filename: string) {
  try {
    await ElMessageBox.confirm(`删除备份 "${filename}"？此操作不可恢复。`, '确认删除', { type: 'warning' })
    await invoke('hosts_delete_backup', { filename })
    ElMessage.success('已删除')
    await loadBackups()
  } catch (e) {
    if (e !== 'cancel' && e !== 'close') ElMessage.error(`删除失败: ${e}`)
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

// ============ 计算属性 ============
const enabledCount = computed(() => entries.value.filter(e => e.enabled).length)

const filteredEntries = computed(() => {
  if (!searchQuery.value) return entries.value
  const kw = searchQuery.value.toLowerCase()
  return entries.value.filter(e =>
    e.ip.toLowerCase().includes(kw) ||
    e.domainsText.toLowerCase().includes(kw) ||
    e.comment.toLowerCase().includes(kw)
  )
})

// ============ 方法 ============
async function checkAdmin() {
  try {
    isAdmin.value = await invoke<boolean>('hosts_check_admin')
  } catch (e) {
    console.error('检测管理员权限失败:', e)
    isAdmin.value = false
  }
}

async function loadHosts() {
  loading.value = true
  try {
    const file = await invoke<{ entries: HostsEntry[], raw_lines: string[], path: string }>('hosts_read')
    // 转换：domains 数组 → domainsText 字符串
    entries.value = file.entries.map(e => ({
      enabled: e.enabled,
      ip: e.ip,
      domains: e.domains || [],
      comment: e.comment || '',
      domainsText: (e.domains || []).join(' ')
    }))
  } catch (e) {
    ElMessage.error(`读取 hosts 失败: ${e}`)
  } finally {
    loading.value = false
  }
}

function addEntry() {
  entries.value.push({
    enabled: true,
    ip: '',
    domains: [],
    comment: '',
    domainsText: ''
  })
}

function removeEntry(index: number) {
  entries.value.splice(index, 1)
}

async function saveHosts() {
  if (!isAdmin.value) {
    ElMessage.warning('需要管理员权限才能保存')
    return
  }

  // 转换：domainsText → domains 数组
  const payload = entries.value.map(e => ({
    enabled: e.enabled,
    ip: e.ip,
    domains: e.domainsText.split(/\s+/).filter(s => s.length > 0),
    comment: e.comment
  }))

  saving.value = true
  try {
    await invoke('hosts_save', { entries: payload })
    ElMessage.success('保存成功')
    // 保存后重新加载（确保 raw_lines 一致）
    await loadHosts()
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

// ============ 生命周期 ============
onMounted(async () => {
  await checkAdmin()
  await loadHosts()
})

onActivated(async () => {
  await checkAdmin()
  if (entries.value.length === 0) {
    await loadHosts()
  }
  if (activeTab.value === 'profiles' && profiles.value.length === 0) {
    await loadProfiles()
  }
  if (activeTab.value === 'backups' && backups.value.length === 0) {
    await loadBackups()
  }
})

watch(activeTab, async (newTab) => {
  if (newTab === 'profiles' && profiles.value.length === 0) {
    await loadProfiles()
  } else if (newTab === 'backups' && backups.value.length === 0) {
    await loadBackups()
  }
})
</script>

<style scoped>
.admin-banner {
  padding: 10px 16px;
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  color: #f59e0b;
  margin-bottom: 16px;
}
.admin-icon { font-size: 16px; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
