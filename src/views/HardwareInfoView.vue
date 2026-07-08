<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">硬件外设</span>
        <div class="card-actions">
          <span v-if="lastRefresh" class="refresh-time">采集于 {{ lastRefresh }}</span>
          <el-button type="primary" size="small" :loading="collecting" @click="collect">刷新</el-button>
        </div>
      </div>
    </div>

    <div v-if="error" class="tool-card">
      <div class="card-body"><div class="error-message">{{ error }}</div></div>
    </div>

    <div v-if="!data" class="tool-card">
      <div class="card-body">
        <el-empty description="暂无数据，点击「刷新」采集硬件外设" />
      </div>
    </div>

    <template v-if="data">
      <!-- CPU -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">CPU</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">名称</span><span class="kv-value">{{ data.cpu.name || '—' }}</span></div>
            <div class="kv-item"><span class="kv-label">核心 / 线程</span><span class="kv-value">{{ data.cpu.cores }} 核 / {{ data.cpu.threads }} 线程</span></div>
            <div class="kv-item"><span class="kv-label">最大频率</span><span class="kv-value">{{ data.cpu.frequency_mhz > 0 ? data.cpu.frequency_mhz + ' MHz' : '—' }}</span></div>
          </div>
        </div>
      </div>

      <!-- 内存 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">内存</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">总容量</span><span class="kv-value">{{ fmt(data.memory.total_gb) }} GB</span></div>
            <div class="kv-item"><span class="kv-label">已使用</span><span class="kv-value">{{ fmt(data.memory.used_gb) }} GB</span></div>
            <div class="kv-item"><span class="kv-label">可用</span><span class="kv-value">{{ fmt(data.memory.available_gb) }} GB</span></div>
          </div>
        </div>
      </div>

      <!-- 磁盘 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">磁盘</span></div>
        <div class="card-body">
          <el-table :data="data.disks" border size="small" style="width: 100%">
            <el-table-column prop="name" label="盘符" width="80" />
            <el-table-column prop="model" label="型号" min-width="180" />
            <el-table-column label="容量" width="120">
              <template #default="{ row }">{{ fmt(row.size_gb) }} GB</template>
            </el-table-column>
            <el-table-column prop="fs_type" label="类型" width="100" />
          </el-table>
          <div v-if="data.disks.length === 0" class="empty-tip">未检测到磁盘</div>
        </div>
      </div>

      <!-- GPU -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">GPU</span></div>
        <div class="card-body">
          <div v-for="(gpu, i) in data.gpus" :key="i" class="hw-section">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">名称</span><span class="kv-value">{{ gpu.name || '—' }}</span></div>
              <div class="kv-item"><span class="kv-label">驱动版本</span><span class="kv-value">{{ gpu.driver_version || '—' }}</span></div>
              <div class="kv-item"><span class="kv-label">显存</span><span class="kv-value">{{ gpu.vram_mb > 0 ? gpu.vram_mb + ' MB' : '未知' }}</span></div>
            </div>
          </div>
          <div v-if="data.gpus.length === 0" class="empty-tip">未检测到 GPU</div>
        </div>
      </div>

      <!-- 显示器 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">显示器</span></div>
        <div class="card-body">
          <div v-for="(display, i) in data.displays" :key="i" class="hw-section">
            <div class="kv-grid">
              <div class="kv-item"><span class="kv-label">名称</span><span class="kv-value">{{ display.name }}</span></div>
              <div class="kv-item"><span class="kv-label">分辨率</span><span class="kv-value">{{ display.resolution || '—' }}</span></div>
            </div>
          </div>
          <div v-if="data.displays.length === 0" class="empty-tip">未检测到显示器</div>
        </div>
      </div>

      <!-- 音频设备 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">音频设备</span></div>
        <div class="card-body">
          <el-table :data="data.audio_devices" border size="small" style="width: 100%">
            <el-table-column prop="name" label="名称" min-width="200" />
            <el-table-column prop="status" label="状态" width="100" />
          </el-table>
          <div v-if="data.audio_devices.length === 0" class="empty-tip">未检测到音频设备</div>
        </div>
      </div>

      <!-- 主板 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">主板</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">制造商</span><span class="kv-value">{{ data.motherboard.manufacturer || '—' }}</span></div>
            <div class="kv-item"><span class="kv-label">型号</span><span class="kv-value">{{ data.motherboard.product || '—' }}</span></div>
            <div class="kv-item"><span class="kv-label">序列号</span><span class="kv-value">{{ data.motherboard.serial || '—' }}</span></div>
          </div>
        </div>
      </div>

      <!-- 电池 -->
      <div v-if="data.battery" class="tool-card">
        <div class="card-header"><span class="card-title">电池</span></div>
        <div class="card-body">
          <div class="kv-grid">
            <div class="kv-item"><span class="kv-label">状态</span><span class="kv-value">{{ data.battery.status }}</span></div>
            <div class="kv-item"><span class="kv-label">电量</span><span class="kv-value">{{ data.battery.charge_percent }}%</span></div>
            <div class="kv-item"><span class="kv-label">预计续航</span><span class="kv-value">{{ data.battery.estimated_time }}</span></div>
          </div>
        </div>
      </div>

      <!-- USB 设备 -->
      <div class="tool-card">
        <div class="card-header"><span class="card-title">USB 设备 ({{ data.usb_devices.length }})</span></div>
        <div class="card-body">
          <el-table :data="data.usb_devices" border size="small" max-height="300" style="width: 100%">
            <el-table-column prop="name" label="名称" min-width="200" />
            <el-table-column prop="device_id" label="设备 ID" min-width="200" />
          </el-table>
          <div v-if="data.usb_devices.length === 0" class="empty-tip">未检测到 USB 设备</div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { formatTimestamp, type HardwareInfo } from '@/utils/systemInfoClient'
import { useToolboxStore } from '@/store'
import { useBackgroundCollect } from '@/composables/useBackgroundCollect'

const store = useToolboxStore()
const data = ref<HardwareInfo | null>(null)
const error = ref('')
const lastRefresh = ref('')

const fmt = (n: number) => n.toFixed(1)

const { collect, collecting } = useBackgroundCollect('hardware')

watch(() => store.collectResults['hardware'], (val) => {
  if (!val) return
  data.value = val as HardwareInfo
  lastRefresh.value = formatTimestamp()
  store.addHistory({
    tool: 'hardwareInfo',
    action: '查看硬件外设',
    inputPreview: '',
    outputPreview: `GPU ${(val as HardwareInfo).gpus.length} 个`,
    inputFull: '',
    outputFull: JSON.stringify(val),
  })
}, { immediate: true })
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
.hw-section { padding: 8px 0; border-bottom: 1px solid var(--border-color); }
.hw-section:last-child { border-bottom: none; }
.kv-grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px 24px; }
.kv-item { display: flex; flex-direction: column; gap: 2px; }
.kv-label { font-size: 11px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.kv-value { font-size: 14px; color: var(--text-primary); word-break: break-all; }
.empty-tip { color: var(--text-muted); font-size: 13px; padding: 8px 0; }
.error-message { padding: 12px; background: rgba(239, 68, 68, 0.1); border: 1px solid var(--accent-red); border-radius: 4px; color: var(--accent-red); font-size: 13px; }
:deep(.el-table) { background: var(--bg-card); color: var(--text-primary); }
:deep(.el-table th) { background: var(--bg-input) !important; color: var(--accent-cyan) !important; font-weight: 600; }
:deep(.el-table td) { background: var(--bg-card) !important; color: var(--text-primary) !important; }
:deep(.el-table--border) { border-color: var(--border-color) !important; }
:deep(.el-table tr) { background: var(--bg-card) !important; }
:deep(.el-table__body tr:hover > td) { background: rgba(0, 212, 255, 0.15) !important; }
:deep(.el-table__inner-wrapper::before) { background-color: var(--border-color) !important; }
</style>
