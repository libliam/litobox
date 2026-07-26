<template>
  <div class="tool-container">
    <!-- 统计卡片 -->
    <div v-if="scanResult" class="stats-row">
      <div class="stat-card memory" :class="{ empty: scanResult.memory_total === 0 }" @click="scanResult.memory_total > 0 && (showMemoryDetail = !showMemoryDetail)">
        <div class="card-top">
          <span class="stat-icon">💾</span>
          <span class="stat-number">{{ scanResult.memory_total === 0 ? '—' : formatBytes(scanResult.memory_total) }}</span>
          <span class="stat-label">可释放内存</span>
          <span class="stat-arrow">{{ showMemoryDetail ? '▼' : '▶' }}</span>
        </div>
        <div class="card-bottom">
          <el-button
            type="primary"
            size="small"
            class="card-clean-btn"
            :loading="executingItem === 'memory'"
            :disabled="!!executingItem || scanResult.memory_total === 0"
            @click.stop="handleCleanMemory"
          >
            🧹 释放内存
          </el-button>
        </div>
      </div>
      <div class="stat-card temp" :class="{ empty: scanResult.temp_size === 0 }" @click="scanResult.temp_size > 0 && (showTempDetail = !showTempDetail)">
        <div class="card-top">
          <span class="stat-icon">📁</span>
          <span class="stat-number">{{ scanResult.temp_size === 0 ? '—' : formatBytes(scanResult.temp_size) }}</span>
          <span class="stat-label">临时文件 ({{ scanResult.temp_file_count }} 个)</span>
          <span class="stat-arrow">{{ showTempDetail ? '▼' : '▶' }}</span>
        </div>
        <div class="card-bottom">
          <el-button
            type="primary"
            size="small"
            class="card-clean-btn"
            :loading="executingItem === 'temp'"
            :disabled="!!executingItem || scanResult.temp_size === 0"
            @click.stop="handleCleanTemp"
          >
            🧹 清理
          </el-button>
        </div>
      </div>
      <div class="stat-card recycle" :class="{ empty: scanResult.recycle_size === 0 }" @click="scanResult.recycle_size > 0 && (showRecycleDetail = !showRecycleDetail)">
        <div class="card-top">
          <span class="stat-icon">🗑️</span>
          <span class="stat-number">{{ scanResult.recycle_size === 0 ? '—' : formatBytes(scanResult.recycle_size) }}</span>
          <span class="stat-label">回收站 ({{ scanResult.recycle_items.length }} 项)</span>
          <span class="stat-arrow">{{ showRecycleDetail ? '▼' : '▶' }}</span>
        </div>
        <div class="card-bottom">
          <el-button
            type="danger"
            size="small"
            class="card-clean-btn"
            :loading="executingItem === 'recycle'"
            :disabled="!!executingItem || scanResult.recycle_size === 0"
            @click.stop="handleCleanRecycle"
          >
            🗑️ 清空
          </el-button>
        </div>
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

    <!-- 临时文件详情表格 -->
    <div v-if="showTempDetail && scanResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">临时文件详情</span>
        <span class="card-sub">
          显示 {{ displayedTempItems.length }} / {{ scanResult.temp_items.length }} 项
        </span>
      </div>
      <div class="card-body">
        <el-table :data="displayedTempItems" max-height="400" size="small" border stripe>
          <el-table-column prop="name" label="文件名" min-width="200">
            <template #default="{ row }">
              <el-tooltip :content="row.name" placement="top" :show-after="300">
                <span class="cell-text">{{ row.name }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
          <el-table-column prop="size" label="大小" width="120" align="right">
            <template #default="{ row }">{{ row.size === 0 ? '—' : formatBytes(row.size) }}</template>
          </el-table-column>
          <el-table-column prop="path" label="路径" min-width="300">
            <template #default="{ row }">
              <el-tooltip :content="row.path" placement="top" :show-after="300">
                <span class="cell-text">{{ row.path }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="scanResult.temp_items.length === 0" class="empty-tip">
          暂无临时文件
        </div>
        <div v-else-if="hasMoreTempItems" class="load-more-wrap">
          <el-button type="primary" plain size="small" @click="loadMoreTemp">
            加载更多 (还剩 {{ scanResult.temp_items.length - displayedTempItems.length }} 项)
          </el-button>
        </div>
      </div>
    </div>

    <!-- 回收站详情表格 -->
    <div v-if="showRecycleDetail && scanResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">回收站详情</span>
        <span class="card-sub">
          显示 {{ displayedRecycleItems.length }} / {{ scanResult.recycle_items.length }} 项
        </span>
      </div>
      <div class="card-body">
        <el-table :data="displayedRecycleItems" max-height="400" size="small" border stripe>
          <el-table-column prop="name" label="名称" min-width="200">
            <template #default="{ row }">
              <el-tooltip :content="row.name" placement="top" :show-after="300">
                <span class="cell-text">{{ row.name }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
          <el-table-column prop="size" label="大小" width="120" align="right">
            <template #default="{ row }">{{ row.size === 0 ? '—' : formatBytes(row.size) }}</template>
          </el-table-column>
          <el-table-column prop="path" label="原路径" min-width="300">
            <template #default="{ row }">
              <el-tooltip :content="row.path" placement="top" :show-after="300">
                <span class="cell-text">{{ row.path }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="scanResult.recycle_items.length === 0" class="empty-tip">
          回收站为空
        </div>
        <div v-else-if="hasMoreRecycleItems" class="load-more-wrap">
          <el-button type="primary" plain size="small" @click="loadMoreRecycleItems">
            加载更多 (还剩 {{ scanResult.recycle_items.length - displayedRecycleItems.length }} 项)
          </el-button>
        </div>
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
            <el-button type="primary" @click="handleScan" :loading="scanning || executing" :disabled="executing || !!executingItem">
              🔍 {{ scanning ? '扫描中...' : '扫描' }}
            </el-button>
            <el-button type="success" @click="handleExecute" :loading="executing" :disabled="!!executingItem">
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
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { boostScan, boostExecute } from '../utils/systemInfoClient'
import type { BoostScanResult, BoostExecuteResult, RecycleItemInfo, TempFileInfo } from '../utils/systemInfoClient'
import { useConfirmDialog } from '../composables/useConfirmDialog'

const { confirm } = useConfirmDialog()

const scanning = ref(false)
const executing = ref(false)
const executingItem = ref<string | null>(null)  // 当前正在单独清理的项
const scanResult = ref<BoostScanResult | null>(null)
const executeResult = ref<BoostExecuteResult | null>(null)
const error = ref('')
const showMemoryDetail = ref(false)
const showTempDetail = ref(false)
const showRecycleDetail = ref(false)
const recyclePageSize = 100
const recyclePage = ref(1)
const tempPageSize = 100
const tempPage = ref(1)

const displayedRecycleItems = computed<RecycleItemInfo[]>(() => {
  if (!scanResult.value) return []
  const end = recyclePage.value * recyclePageSize
  return scanResult.value.recycle_items.slice(0, end)
})

const hasMoreRecycleItems = computed(() => {
  if (!scanResult.value) return false
  return recyclePage.value * recyclePageSize < scanResult.value.recycle_items.length
})

const displayedTempItems = computed<TempFileInfo[]>(() => {
  if (!scanResult.value) return []
  const end = tempPage.value * tempPageSize
  return scanResult.value.temp_items.slice(0, end)
})

const hasMoreTempItems = computed(() => {
  if (!scanResult.value) return false
  return tempPage.value * tempPageSize < scanResult.value.temp_items.length
})

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
  recyclePage.value = 1
  tempPage.value = 1
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
  console.log('[Boost] handleExecute 开始')
  if (!scanResult.value) {
    console.log('[Boost] 无扫描结果，先执行扫描')
    await handleScan()
    if (!scanResult.value) {
      console.log('[Boost] 扫描后仍无结果，退出')
      return
    }
  }
  console.log('[Boost] recycle_items.length =', scanResult.value.recycle_items.length)
  if (scanResult.value.recycle_items.length > 0) {
    console.log('[Boost] 弹出确认弹窗')
    const ok = await confirm.ask(
      '确认清空回收站',
      `回收站中有 ${scanResult.value.recycle_items.length} 项（${formatBytes(scanResult.value.recycle_size)}），清空后无法恢复，确定继续吗？`,
      { type: 'danger', confirmText: '确定清空' }
    )
    console.log('[Boost] confirm.ask 返回:', ok)
    if (!ok) {
      console.log('[Boost] 用户取消，退出')
      return
    }
  }
  console.log('[Boost] 开始执行 boostExecute')
  executing.value = true
  error.value = ''
  try {
    executeResult.value = await boostExecute()
    console.log('[Boost] boostExecute 完成:', executeResult.value)
    ElMessage.success(`加速完成，总计释放 ${formatBytes(executeResult.value.total_freed)}`)
    await handleScan()
  } catch (e) {
    console.error('[Boost] boostExecute 失败:', e)
    error.value = String(e)
    ElMessage.error('加速失败: ' + String(e))
  } finally {
    executing.value = false
  }
}

async function handleCleanMemory() {
  executingItem.value = 'memory'
  error.value = ''
  try {
    const result = await boostExecute(['memory'])
    ElMessage.success(`内存释放完成，释放 ${formatBytes(result.total_freed)}`)
    await handleScan()
  } catch (e) {
    console.error('[Boost] 内存释放失败:', e)
    ElMessage.error('内存释放失败: ' + String(e))
  } finally {
    executingItem.value = null
  }
}

async function handleCleanTemp() {
  executingItem.value = 'temp'
  error.value = ''
  try {
    const result = await boostExecute(['temp'])
    const item = result.items.find(i => i.name === '临时文件清理')
    const msg = item
      ? `删除 ${formatBytes(result.total_freed)}（${item.message}）`
      : `释放 ${formatBytes(result.total_freed)}`
    ElMessage.success(`临时文件清理完成，${msg}`)
    await handleScan()
  } catch (e) {
    console.error('[Boost] 临时文件清理失败:', e)
    ElMessage.error('临时文件清理失败: ' + String(e))
  } finally {
    executingItem.value = null
  }
}

async function handleCleanRecycle() {
  if (!scanResult.value) return
  const ok = await confirm.ask(
    '确认清空回收站',
    `回收站中有 ${scanResult.value.recycle_items.length} 项（${formatBytes(scanResult.value.recycle_size)}），清空后无法恢复，确定继续吗？`,
    { type: 'danger', confirmText: '确定清空' }
  )
  console.log('[Boost] handleCleanRecycle confirm.ask 返回:', ok)
  if (!ok) return
  executingItem.value = 'recycle'
  error.value = ''
  try {
    const result = await boostExecute(['recycle'])
    ElMessage.success(`回收站已清空，释放 ${formatBytes(result.total_freed)}`)
    await handleScan()
  } catch (e) {
    console.error('[Boost] 回收站清空失败:', e)
    ElMessage.error('回收站清空失败: ' + String(e))
  } finally {
    executingItem.value = null
  }
}

function loadMoreRecycleItems() {
  recyclePage.value++
}

function loadMoreTemp() {
  tempPage.value++
}
</script>

<style scoped>
.stats-row {
  position: sticky;
  top: 0;
  z-index: 10;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 16px;
  padding-top: 2px;
  padding-bottom: 4px;
  background: var(--bg-primary);
}

.stat-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 180px;
  padding: 16px 14px;
  border-radius: 8px;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  transition: border-color 0.2s, transform 0.2s;
}

.stat-card.memory,
.stat-card.recycle {
  cursor: pointer;
}

.stat-card.memory:hover,
.stat-card.recycle:hover {
  border-color: var(--accent-color);
}

.stat-card.empty {
  opacity: 0.55;
}

.stat-card.empty:hover {
  border-color: var(--border-color);
  cursor: default;
}

.card-top {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.card-bottom {
  width: 100%;
  margin-top: auto;
  padding-top: 12px;
}

.stat-icon {
  font-size: 28px;
  margin-bottom: 10px;
}

.stat-number {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  font-family: 'Consolas', 'Courier New', monospace;
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 6px;
}

.stat-arrow {
  font-size: 12px;
  color: var(--accent-color);
  margin-top: 6px;
}

.card-clean-btn {
  width: 100%;
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

.cell-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-tip {
  text-align: center;
  padding: 20px;
  color: var(--text-secondary);
  font-size: 13px;
}

.load-more-wrap {
  text-align: center;
  padding: 12px 0 4px;
}
</style>