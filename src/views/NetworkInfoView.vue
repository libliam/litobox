<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">网络信息</span>
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
      <div class="tool-card">
        <div class="card-header"><span class="card-title">概览</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">主机名</span><span class="kv-value">{{ data.hostname }}</span></div>
            <div class="kv-item"><span class="kv-label">WiFi</span><span class="kv-value">{{ data.wifi_name || '未连接' }}</span></div>
            <div class="kv-item"><span class="kv-label">默认网关</span><span class="kv-value">{{ data.default_gateway || '—' }}</span></div>
            <div class="kv-item"><span class="kv-label">DNS</span><span class="kv-value">{{ data.dns_servers.join(', ') || '—' }}</span></div>
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header"><span class="card-title">网络接口</span></div>
        <div class="card-body">
          <el-table :data="data.interfaces" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" min-width="120" />
            <el-table-column prop="mac" label="MAC 地址" width="160" />
            <el-table-column label="IPv4" min-width="140">
              <template #default="{ row }">{{ row.ipv4.join(', ') || '—' }}</template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="80" />
          </el-table>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header"><span class="card-title">活动连接 ({{ data.active_connections.length }})</span></div>
        <div class="card-body">
          <el-table :data="data.active_connections" border size="small" max-height="400" style="width: 100%">
            <el-table-column prop="protocol" label="协议" width="60" />
            <el-table-column prop="local_addr" label="本地地址" min-width="160" />
            <el-table-column prop="remote_addr" label="远程地址" min-width="160" />
            <el-table-column prop="state" label="状态" width="100" />
            <el-table-column prop="pid" label="PID" width="70" />
          </el-table>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header"><span class="card-title">监听端口 ({{ data.listening_ports.length }})</span></div>
        <div class="card-body">
          <el-table :data="data.listening_ports" border size="small" max-height="400" style="width: 100%">
            <el-table-column prop="protocol" label="协议" width="60" />
            <el-table-column prop="local_addr" label="地址" min-width="160" />
            <el-table-column prop="pid" label="PID" width="70" />
            <el-table-column prop="process_name" label="进程" min-width="120" />
          </el-table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElLoading } from 'element-plus'
import { getNetworkInfo, formatTimestamp, type NetworkInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()
const data = ref<NetworkInfo | null>(null)
const loading = ref(false)
const error = ref('')
const lastRefresh = ref('')

const loadData = async () => {
  loading.value = true
  error.value = ''
  const loadingInstance = ElLoading.service({ text: '采集中...' })
  try {
    data.value = await getNetworkInfo()
    lastRefresh.value = formatTimestamp()
    store.addHistory({
      tool: 'networkInfo',
      action: '查看网络信息',
      inputPreview: '',
      outputPreview: `${data.value.interfaces.length} 个接口 | ${data.value.listening_ports.length} 个监听端口`,
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
.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
