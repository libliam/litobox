<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">软件环境</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <template v-if="data">
      <el-tabs v-model="activeTab" class="env-tabs">
        <el-tab-pane :label="`已安装软件 (${data.installed_software.length})`" name="software">
          <div class="tool-card">
            <div class="card-header">
              <span class="card-title">已安装软件</span>
              <el-input v-model="softwareSearch" size="small" placeholder="搜索软件名..." style="width: 200px" clearable />
            </div>
            <div class="card-body">
              <el-table :data="filteredSoftware" border size="small" max-height="500" style="width: 100%">
                <el-table-column prop="name" label="名称" min-width="200" sortable />
                <el-table-column prop="version" label="版本" width="100" />
                <el-table-column prop="publisher" label="发布者" min-width="150" />
                <el-table-column prop="install_date" label="安装日期" width="120" />
              </el-table>
            </div>
          </div>
        </el-tab-pane>

        <el-tab-pane :label="`环境变量 (${data.environment_variables.length})`" name="env">
          <div class="tool-card">
            <div class="card-header">
              <span class="card-title">环境变量</span>
              <el-input v-model="envSearch" size="small" placeholder="搜索变量名..." style="width: 200px" clearable />
            </div>
            <div class="card-body">
              <el-table :data="filteredEnv" border size="small" max-height="500" style="width: 100%">
                <el-table-column prop="key" label="变量名" width="200" sortable />
                <el-table-column prop="value" label="值" min-width="300" show-overflow-tooltip />
              </el-table>
            </div>
          </div>
        </el-tab-pane>


      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getSoftwareEnv, formatTimestamp, type SoftwareEnv } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<SoftwareEnv | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')
const activeTab = ref('software')
const softwareSearch = ref('')
const envSearch = ref('')

const filteredSoftware = computed(() => {
  if (!data.value) return []
  const q = softwareSearch.value.toLowerCase().trim()
  if (!q) return data.value.installed_software
  return data.value.installed_software.filter(s => s.name.toLowerCase().includes(q))
})

const filteredEnv = computed(() => {
  if (!data.value) return []
  const q = envSearch.value.toLowerCase().trim()
  if (!q) return data.value.environment_variables
  return data.value.environment_variables.filter(e => e.key.toLowerCase().includes(q))
})

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getSoftwareEnv()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'softwareEnv',
      action: '查看软件环境',
      inputPreview: '',
      outputPreview: `${data.value.installed_software.length} 软件 | ${data.value.environment_variables.length} 环境变量`,
      inputFull: '',
      outputFull: JSON.stringify(data.value, null, 2),
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
    loadingInstance.close()
  }
}

onMounted(() => { loadData() })
</script>

<style scoped>
.tool-card { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; margin-bottom: 16px; overflow: hidden; transition: border-color 0.3s; }
.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }
.tool-card:last-child { margin-bottom: 0; }
.sticky-card { position: sticky; top: 0; z-index: 10; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
.card-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px; background: rgba(0, 0, 0, 0.2); border-bottom: 1px solid var(--border-color); }
.card-title { font-weight: 600; font-size: 14px; color: var(--accent-cyan); text-transform: uppercase; letter-spacing: 1px; }
.card-body { padding: 16px 20px; }
.card-actions { display: flex; align-items: center; gap: 12px; }
.refresh-time { font-size: 12px; color: var(--text-muted); }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
.env-tabs :deep(.el-tabs__header) { margin-bottom: 16px; padding-left: 8px; position: sticky; top: 0; z-index: 20; background: var(--bg-primary); box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3); }
.env-tabs :deep(.el-tabs__item) { color: var(--text-secondary); font-size: 14px; font-weight: 500; }
.env-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.env-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.env-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
