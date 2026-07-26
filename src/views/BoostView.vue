<template>
  <div class="tool-container">
    <!-- 统计卡片 -->
    <div v-if="scanResult" class="stats-row">
      <div class="stat-card memory" @click="showMemoryDetail = !showMemoryDetail">
        <span class="stat-icon">💾</span>
        <span class="stat-number">{{ formatBytes(scanResult.memory_total) }}</span>
        <span class="stat-label">可释放内存</span>
        <span class="stat-arrow">{{ showMemoryDetail ? '▼' : '▶' }}</span>
      </div>
      <div class="stat-card temp">
        <span class="stat-icon">📁</span>
        <span class="stat-number">{{ formatBytes(scanResult.temp_size) }}</span>
        <span class="stat-label">临时文件 ({{ scanResult.temp_file_count }} 个)</span>
      </div>
      <div class="stat-card recycle">
        <span class="stat-icon">🗑️</span>
        <span class="stat-number">{{ formatBytes(scanResult.recycle_size) }}</span>
        <span class="stat-label">回收站</span>
      </div>
    </div>

    <!-- 内存详情表格 -->
    <div v-if="showMemoryDetail && scanResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">进程内存详情</span>
        <span class="card-sub">{{ scanResult.processes.length }} 个进程</span>
      </div>
      <div class="card-body">
        <el-table :data="scanResult.processes" max-height="400" size="small" border stripe>
          <el-table-column prop="name" label="进程名称" width="200" />
          <el-table-column prop="pid" label="PID" width="80" />
          <el-table-column prop="working_set" label="工作集" width="140">
            <template #default="{ row }">{{ formatBytes(row.working_set) }}</template>
          </el-table-column>
          <el-table-column label="占比" width="100">
            <template #default="{ row }">{{ ((row.working_set / scanResult.memory_total) * 100).toFixed(1) }}%</template>
          </el-table-column>
        </el-table>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">一键加速</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button type="primary" @click="handleScan" :loading="scanning" :disabled="executing">
              🔍 {{ scanning ? '扫描中...' : '扫描' }}
            </el-button>
            <el-button type="success" @click="handleExecute" :loading="executing" :disabled="!scanResult">
              ⚡ {{ executing ? '加速中...' : '一键加速' }}
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 执行结果 -->
    <div v-if="executeResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">执行结果</span>
      </div>
      <div class="card-body">
        <div class="result-list">
          <div
            v-for="item in executeResult.items"
            :key="item.name"
            class="result-item"
            :class="{ failed: !item.success }"
          >
            <span class="result-icon">{{ item.success ? '✅' : '❌' }}</span>
            <span class="result-name">{{ item.name }}</span>
            <span class="result-freed">{{ formatBytes(item.freed) }}</span>
            <span class="result-duration">({{ item.duration_ms / 1000 }}s)</span>
            <span class="result-msg">{{ item.message }}</span>
          </div>
        </div>
        <div class="result-total">
          📊 总计释放: <strong>{{ formatBytes(executeResult.total_freed) }}</strong>
          · 耗时: {{ (executeResult.total_duration_ms / 1000).toFixed(1) }}s
        </div>
      </div>
    </div>

    <!-- 初始状态提示 -->
    <div v-if="!scanResult && !error" class="tool-card">
      <div class="card-body">
        <div class="empty-state">
          <span class="empty-icon">⚡</span>
          <p>点击「扫描」按钮，查看可释放的系统资源</p>
          <p class="empty-sub">内存释放 · 临时文件清理 · 回收站清空</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { boostScan, boostExecute } from '../utils/systemInfoClient'
import type { BoostScanResult, BoostExecuteResult } from '../utils/systemInfoClient'

const scanning = ref(false)
const executing = ref(false)
const scanResult = ref<BoostScanResult | null>(null)
const executeResult = ref<BoostExecuteResult | null>(null)
const error = ref('')
const showMemoryDetail = ref(false)

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + ' ' + units[i]
}

async function handleScan() {
  scanning.value = true
  error.value = ''
  executeResult.value = null
  try {
    scanResult.value = await boostScan()
  } catch (e) {
    error.value = String(e)
    ElMessage.error('扫描失败: ' + String(e))
    scanResult.value = null
  } finally {
    scanning.value = false
  }
}

async function handleExecute() {
  if (!scanResult.value) return
  executing.value = true
  error.value = ''
  try {
    executeResult.value = await boostExecute()
    ElMessage.success(`加速完成，总计释放 ${formatBytes(executeResult.value.total_freed)}`)
    await handleScan()
  } catch (e) {
    error.value = String(e)
    ElMessage.error('加速失败: ' + String(e))
  } finally {
    executing.value = false
  }
}
</script>

<style scoped>
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 12px;
  border-radius: 8px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
}

.stat-card.memory {
  cursor: pointer;
}

.stat-card.memory:hover {
  border-color: var(--accent-color);
}

.stat-icon {
  font-size: 24px;
  margin-bottom: 8px;
}

.stat-number {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  font-family: 'Consolas', 'Courier New', monospace;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.stat-arrow {
  font-size: 12px;
  color: var(--accent-color);
  margin-top: 4px;
}

.result-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  background: var(--bg-secondary);
  font-size: 13px;
}

.result-item.failed {
  background: rgba(255, 77, 79, 0.08);
}

.result-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.result-name {
  font-weight: 600;
  min-width: 100px;
  color: var(--text-primary);
}

.result-freed {
  font-family: 'Consolas', 'Courier New', monospace;
  color: var(--accent-color);
  font-weight: 600;
}

.result-duration {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 50px;
}

.result-msg {
  font-size: 12px;
  color: var(--text-secondary);
  flex: 1;
  text-align: right;
}

.result-total {
  text-align: center;
  padding: 12px;
  border-radius: 6px;
  background: var(--bg-secondary);
  border: 1px solid var(--accent-color);
  color: var(--text-primary);
  font-size: 14px;
}

.result-total strong {
  color: var(--accent-color);
  font-family: 'Consolas', 'Courier New', monospace;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.empty-state p {
  margin: 4px 0;
  font-size: 14px;
}

.empty-sub {
  font-size: 12px !important;
  color: var(--text-tertiary);
}

.action-grid {
  display: flex;
  justify-content: center;
  gap: 12px;
}
</style>