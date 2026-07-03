<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">系统信息</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="loading" @click="loadData">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ error }}</div>
      </div>
    </div>

    <template v-if="data">
      <div class="info-row">
        <div class="tool-card">
          <div class="card-header"><span class="card-title">操作系统</span></div>
          <div class="card-body">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">系统</span><span class="kv-value">{{ data.os_name }}</span></div>
              <div class="kv-item"><span class="kv-label">版本</span><span class="kv-value">{{ data.os_version }}</span></div>
              <div class="kv-item"><span class="kv-label">架构</span><span class="kv-value">{{ data.os_arch }}</span></div>
              <div class="kv-item"><span class="kv-label">主机名</span><span class="kv-value">{{ data.hostname }}</span></div>
              <div class="kv-item"><span class="kv-label">运行时长</span><span class="kv-value">{{ formatUptime(data.uptime_secs) }}</span></div>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header"><span class="card-title">CPU</span></div>
          <div class="card-body">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">型号</span><span class="kv-value">{{ data.cpu.brand }}</span></div>
              <div class="kv-item"><span class="kv-label">物理核心</span><span class="kv-value">{{ data.cpu.core_count }}</span></div>
              <div class="kv-item"><span class="kv-label">逻辑线程</span><span class="kv-value">{{ data.cpu.thread_count }}</span></div>
              <div class="kv-item"><span class="kv-label">频率</span><span class="kv-value">{{ data.cpu.frequency_mhz }} MHz</span></div>
            </div>
            <div class="progress-row">
              <span class="progress-label">使用率</span>
              <el-progress :percentage="Math.round(data.cpu.usage_percent)" :stroke-width="10" />
            </div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header"><span class="card-title">内存</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">总计</span><span class="kv-value">{{ formatBytes(data.memory.total_bytes) }}</span></div>
            <div class="kv-item"><span class="kv-label">已用</span><span class="kv-value">{{ formatBytes(data.memory.used_bytes) }}</span></div>
            <div class="kv-item"><span class="kv-label">可用</span><span class="kv-value">{{ formatBytes(data.memory.available_bytes) }}</span></div>
          </div>
          <div class="progress-row">
            <span class="progress-label">使用率</span>
            <el-progress :percentage="Math.round(data.memory.used_bytes / data.memory.total_bytes * 100)" :stroke-width="10" />
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header"><span class="card-title">磁盘</span></div>
        <div class="card-body">
          <el-table :data="data.disks" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" width="100" />
            <el-table-column label="总量">
              <template #default="{ row }">{{ formatBytes(row.total_bytes) }}</template>
            </el-table-column>
            <el-table-column label="已用">
              <template #default="{ row }">{{ formatBytes(row.used_bytes) }}</template>
            </el-table-column>
            <el-table-column label="使用率" width="180">
              <template #default="{ row }">
                <el-progress :percentage="Math.round(row.used_bytes / row.total_bytes * 100)" :stroke-width="8" />
              </template>
            </el-table-column>
            <el-table-column prop="file_system" label="文件系统" width="100" />
            <el-table-column label="类型" width="80">
              <template #default="{ row }">{{ row.is_removable ? '可移动' : '固定' }}</template>
            </el-table-column>
          </el-table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getSystemInfo, formatBytes, formatUptime, formatTimestamp, type SystemInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<SystemInfo | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getSystemInfo()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'systemInfo',
      action: '查看系统信息',
      inputPreview: '',
      outputPreview: `${data.value.cpu.brand} | ${formatBytes(data.value.memory.total_bytes)}`,
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

onMounted(() => {
  loadData()
})
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
.info-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }
.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.progress-row { display: flex; align-items: center; gap: 12px; margin-top: 16px; }
.progress-label { font-size: 13px; color: var(--text-secondary); white-space: nowrap; min-width: 50px; }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
:deep(.el-progress__text) { color: var(--text-primary); }
</style>
