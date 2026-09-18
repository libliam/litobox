<template>
  <div class="tool-container fill-height">
    <!-- 操作栏 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">右键菜单管理</span>
        <el-tooltip placement="top" effect="dark">
          <template #content>
            <div class="tooltip-content">
              <p>读取文件/文件夹/桌面右键的静态菜单项、COM 处理器及通用位置</p>
              <p>第三方项可删除，删除前自动精确备份该项，可随时恢复</p>
            </div>
          </template>
          <el-icon class="hint-icon"><QuestionFilled /></el-icon>
        </el-tooltip>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">作用域</div>
            <div class="group-buttons">
              <el-button
                size="small"
                :type="activeScope === 'file' ? 'primary' : 'default'"
                @click="switchScope('file')"
              >文件右键</el-button>
              <el-button
                size="small"
                :type="activeScope === 'folder' ? 'primary' : 'default'"
                @click="switchScope('folder')"
              >文件夹右键</el-button>
              <el-button
                size="small"
                :type="activeScope === 'desktop' ? 'primary' : 'default'"
                @click="switchScope('desktop')"
              >桌面右键</el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">操作</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="showAddDialog = true">添加项</el-button>
              <el-button type="success" size="small" @click="handleBackup">备份当前</el-button>
              <el-button size="small" @click="refreshItems" :loading="loading">刷新</el-button>
              <el-button size="small" @click="openBackupDialog">备份管理 ({{ backups.length }})</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 菜单项列表 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">{{ scopeLabel }} ({{ filteredItems.length }}<template v-if="filtering"> / {{ items.length }}</template>)</span>
      </div>
      <div class="filter-bar">
        <el-input
          v-model="keyword"
          size="small"
          clearable
          class="filter-search"
          placeholder="搜索显示名 / 键名 / 命令 / 处理器"
          :prefix-icon="Search"
        />
        <div class="filter-item">
          <span class="filter-label">类型</span>
          <el-radio-group v-model="kindFilter" size="small">
            <el-radio-button value="">全部</el-radio-button>
            <el-radio-button value="shell">静态项</el-radio-button>
            <el-radio-button value="shellex">COM</el-radio-button>
          </el-radio-group>
        </div>
        <div class="filter-item">
          <span class="filter-label">归属</span>
          <el-radio-group v-model="ownerFilter" size="small">
            <el-radio-button value="">全部</el-radio-button>
            <el-radio-button value="custom">自定义</el-radio-button>
            <el-radio-button value="system">系统</el-radio-button>
            <el-radio-button value="blocked">已屏蔽</el-radio-button>
          </el-radio-group>
        </div>
        <el-button v-if="filtering" size="small" link @click="resetFilter">重置</el-button>
      </div>
      <div class="card-body">
        <div v-if="loading" style="text-align: center; padding: 40px">
          <el-icon class="is-loading" :size="32"><Loading /></el-icon>
          <p style="color: var(--text-secondary); margin-top: 8px">加载中...</p>
        </div>
        <el-empty v-else-if="!filteredItems.length" :description="items.length ? '没有匹配的菜单项' : '暂无菜单项'" />
        <div v-else class="item-list">
          <div
            v-for="item in filteredItems"
            :key="item.item_path"
            class="item-card"
            :class="{ system: !item.is_custom }"
          >
            <div class="item-header">
              <span class="item-name">{{ item.display_name }}</span>
              <el-tag size="small" effect="plain">{{ item.source_label }}</el-tag>
              <el-tag v-if="item.kind === 'shellex'" size="small" type="warning">COM</el-tag>
              <el-tag v-if="!item.is_enabled" size="small" type="danger">已屏蔽</el-tag>
              <el-tag v-if="!item.is_custom" type="info" size="small">系统</el-tag>
              <el-tag v-else type="success" size="small">自定义</el-tag>
              <el-button
                v-if="item.is_custom"
                type="danger"
                size="small"
                link
                class="item-delete"
                @click="handleDelete(item)"
              >删除</el-button>
            </div>
            <div class="item-meta">
              <span class="meta-label">键名:</span>
              <span class="meta-value mono-text">{{ item.key_name.trim() }}</span>
            </div>
            <div v-if="item.command" class="item-meta">
              <span class="meta-label">命令:</span>
              <span class="meta-value mono-text" :title="item.command">{{ item.command }}</span>
            </div>
            <div v-if="item.dll_path" class="item-meta">
              <span class="meta-label">处理器:</span>
              <span class="meta-value mono-text" :title="item.dll_path">{{ item.dll_path }}</span>
            </div>
            <div v-if="item.icon" class="item-meta">
              <span class="meta-label">图标:</span>
              <span class="meta-value mono-text">{{ item.icon }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加项对话框 -->
    <el-dialog v-model="showAddDialog" title="添加右键菜单项" width="520px" :close-on-click-modal="false" append-to-body>
      <el-form :model="addForm" label-width="90px" size="default">
        <el-form-item label="作用域">
          <el-select v-model="addForm.scope" style="width: 100%">
            <el-option label="文件右键" value="file" />
            <el-option label="文件夹右键" value="folder" />
            <el-option label="桌面右键" value="desktop" />
          </el-select>
        </el-form-item>
        <el-form-item label="键名">
          <el-input v-model="addForm.key_name" placeholder="注册表键名，如 MyTool" />
        </el-form-item>
        <el-form-item label="显示名称">
          <el-input v-model="addForm.display_name" placeholder="右键菜单显示的文字" />
        </el-form-item>
        <el-form-item label="命令">
          <el-input
            v-model="addForm.command"
            placeholder="执行命令，如 notepad.exe %1"
            type="textarea"
            :rows="2"
          />
          <div class="form-hint">%1 代表选中的文件路径</div>
        </el-form-item>
        <el-form-item label="图标">
          <el-input v-model="addForm.icon" placeholder="可选，如 C:\path\to\icon.ico,0" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="handleAdd" :loading="adding">添加</el-button>
      </template>
    </el-dialog>

    <!-- 备份管理对话框 -->
    <el-dialog v-model="showBackupDialog" title="备份管理" width="640px" append-to-body>
      <div class="backup-list">
        <div v-if="backups.length === 0" style="text-align: center; padding: 30px">
          <el-empty description="暂无备份" />
        </div>
        <div v-for="b in backups" :key="b.filename" class="backup-item">
          <div class="backup-info">
            <span class="backup-scope">{{ scopeName(b.scope) }}</span>
            <span class="backup-time">{{ formatTime(b.timestamp_ms) }}</span>
            <span class="backup-size">{{ formatSize(b.size) }}</span>
          </div>
          <div class="backup-actions">
            <el-button size="small" type="primary" link @click="handleRestore(b)">恢复</el-button>
            <el-button size="small" type="danger" link @click="handleDeleteBackup(b)">删除</el-button>
          </div>
        </div>
      </div>
      <template #footer>
        <el-button @click="showBackupDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Loading, Search } from '@element-plus/icons-vue'
import {
  cmListItems,
  cmAddItem,
  cmDeleteItem,
  cmBackup,
  cmRestore,
  cmListBackups,
  cmDeleteBackup,
  type ContextMenuItem,
  type BackupInfo,
} from '@/utils/contextMenuClient'

// ============ 状态 ============
const activeScope = ref('file')
const items = ref<ContextMenuItem[]>([])
const loading = ref(false)
const backups = ref<BackupInfo[]>([])

const showAddDialog = ref(false)
const showBackupDialog = ref(false)
const adding = ref(false)

const addForm = reactive({
  scope: 'file',
  key_name: '',
  display_name: '',
  command: '',
  icon: '',
})

// ============ 筛选 ============
const keyword = ref('')
const kindFilter = ref('')
const ownerFilter = ref('')
const filtering = computed(
  () => keyword.value.trim() !== '' || kindFilter.value !== '' || ownerFilter.value !== ''
)

const resetFilter = () => {
  keyword.value = ''
  kindFilter.value = ''
  ownerFilter.value = ''
}

// ============ 计算属性 ============
const scopeLabel = computed(() => {
  const labels: Record<string, string> = {
    file: '文件右键菜单',
    folder: '文件夹右键菜单',
    desktop: '桌面右键菜单',
  }
  return labels[activeScope.value] || ''
})

const filteredItems = computed(() => {
  const kw = keyword.value.trim().toLowerCase()
  return items.value.filter((it) => {
    if (kw) {
      const hay = [it.display_name, it.key_name, it.command, it.dll_path]
        .join('\n')
        .toLowerCase()
      if (!hay.includes(kw)) return false
    }
    if (kindFilter.value && it.kind !== kindFilter.value) return false
    if (ownerFilter.value === 'custom' && !it.is_custom) return false
    if (ownerFilter.value === 'system' && it.is_custom) return false
    if (ownerFilter.value === 'blocked' && it.is_enabled) return false
    return true
  })
})

// ============ 工具函数 ============
const scopeName = (scope: string) => {
  const map: Record<string, string> = {
    file: '文件',
    folder: '文件夹',
    desktop: '桌面',
  }
  return map[scope] || scope
}

const formatTime = (ms: number) => {
  const d = new Date(ms)
  return d.toLocaleString('zh-CN')
}

const formatSize = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}

// ============ 操作 ============
const refreshItems = async () => {
  loading.value = true
  try {
    items.value = await cmListItems(activeScope.value)
  } catch (e) {
    ElMessage.error('加载菜单项失败: ' + String(e))
  } finally {
    loading.value = false
  }
}

const switchScope = (scope: string) => {
  activeScope.value = scope
  refreshItems()
}

const handleAdd = async () => {
  if (!addForm.key_name.trim()) {
    ElMessage.warning('请输入键名')
    return
  }
  if (!addForm.display_name.trim()) {
    ElMessage.warning('请输入显示名称')
    return
  }
  if (!addForm.command.trim()) {
    ElMessage.warning('请输入命令')
    return
  }

  adding.value = true
  try {
    const result = await cmAddItem(
      addForm.scope,
      addForm.key_name.trim(),
      addForm.display_name.trim(),
      addForm.command.trim(),
      addForm.icon.trim()
    )
    if (result.success) {
      ElMessage.success(result.message)
      showAddDialog.value = false
      // 重置表单
      addForm.key_name = ''
      addForm.display_name = ''
      addForm.command = ''
      addForm.icon = ''
      // 切换到对应 scope 并刷新
      activeScope.value = addForm.scope
      await refreshItems()
      await refreshBackups()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error('添加失败: ' + String(e))
  } finally {
    adding.value = false
  }
}

const handleDelete = async (item: ContextMenuItem) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除右键菜单项「${item.display_name}」吗？\n删除前已自动备份，可在备份管理中恢复。`,
      '确认删除',
      { type: 'warning' }
    )
  } catch {
    return
  }

  try {
    const result = await cmDeleteItem(item.scope, item.item_path, item.display_name)
    if (result.success) {
      ElMessage.success(result.message)
      await refreshItems()
      await refreshBackups()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error('删除失败: ' + String(e))
  }
}

const handleBackup = async () => {
  try {
    const list = await cmBackup(activeScope.value)
    ElMessage.success(
      `已备份 ${list.length} 个注册表项（${scopeName(activeScope.value)} 右键菜单）`
    )
    await refreshBackups()
  } catch (e) {
    ElMessage.error('备份失败: ' + String(e))
  }
}

const refreshBackups = async () => {
  try {
    backups.value = await cmListBackups()
  } catch (e) {
    ElMessage.error('加载备份列表失败: ' + String(e))
  }
}

const openBackupDialog = () => {
  refreshBackups()
  showBackupDialog.value = true
}

const handleRestore = async (b: BackupInfo) => {
  try {
    await ElMessageBox.confirm(
      `确定要恢复备份「${b.filename}」吗？\n这将覆盖当前 ${scopeName(b.scope)} 右键菜单的注册表项。`,
      '确认恢复',
      { type: 'warning' }
    )
  } catch {
    return
  }

  try {
    const result = await cmRestore(b.path)
    if (result.success) {
      ElMessage.success(result.message)
      await refreshItems()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error('恢复失败: ' + String(e))
  }
}

const handleDeleteBackup = async (b: BackupInfo) => {
  try {
    await ElMessageBox.confirm(`确定要删除备份「${b.filename}」吗？`, '确认删除', { type: 'warning' })
  } catch {
    return
  }

  try {
    const result = await cmDeleteBackup(b.filename)
    if (result.success) {
      ElMessage.success(result.message)
      await refreshBackups()
    } else {
      ElMessage.error(result.message)
    }
  } catch (e) {
    ElMessage.error('删除备份失败: ' + String(e))
  }
}

// ============ 初始化 ============
onMounted(() => {
  refreshItems()
  refreshBackups()
})
</script>

<style scoped>
.filter-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.filter-search {
  width: 260px;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.item-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item-card {
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border-left: 3px solid var(--accent-cyan);
}

.item-card.system {
  border-left-color: var(--text-secondary, #888);
  opacity: 0.85;
}

.item-header {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.item-delete {
  margin-left: auto;
}

.item-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  margin-top: 4px;
}

.meta-label {
  color: var(--text-secondary);
  min-width: 48px;
}

.meta-value {
  color: var(--text-primary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mono-text {
  font-family: 'Consolas', 'Monaco', monospace;
}

.form-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

/* 备份列表 */
.backup-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.backup-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.backup-info {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 13px;
}

.backup-scope {
  font-weight: 600;
  color: var(--accent-cyan);
}

.backup-time {
  color: var(--text-secondary);
}

.backup-size {
  color: var(--text-secondary);
  font-family: 'Consolas', monospace;
}

.backup-actions {
  display: flex;
  gap: 8px;
}

.tooltip-content p {
  margin: 0;
  font-size: 12px;
}
</style>
