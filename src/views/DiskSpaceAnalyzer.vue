<template>
  <div class="tool-container">
    <!-- 扫描配置卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">磁盘空间分析</span>
        <div class="card-actions">
          <el-button size="small" @click="loadLastScanPath">上次路径</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group" style="flex: 1">
            <div class="group-label">扫描路径</div>
            <el-input
              v-model="scanPath"
              placeholder="选择或输入要扫描的目录路径"
              size="small"
              clearable
            >
              <template #append>
                <el-button size="small" @click="selectFolder">浏览</el-button>
              </template>
            </el-input>
          </div>
          <div class="action-group">
            <div class="group-label">选项</div>
            <el-checkbox v-model="opts.includeHidden">包含隐藏</el-checkbox>
            <el-checkbox v-model="opts.detectDuplicates">检测重复</el-checkbox>
            <el-checkbox v-model="opts.followSymlinks">跟随链接</el-checkbox>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!scanPath || scanning"
                :loading="scanning"
                @click="startScan"
              >
                开始扫描
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 扫描进度卡片（仅扫描中显示） -->
    <div v-if="scanning" class="tool-card">
      <div class="card-header">
        <span class="card-title">扫描中</span>
        <div class="card-actions">
          <el-button size="small" type="danger" @click="cancelScan">取消扫描</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="progress-info">
          <div>当前路径: {{ progress?.currentPath || '准备中...' }}</div>
          <div>
            已扫描: {{ progress?.filesScanned || 0 }} 文件 |
            {{ formatBytes(progress?.bytesScanned || 0) }} |
            耗时 {{ formatDuration(scanElapsedMs) }}
          </div>
        </div>
        <el-progress :percentage="scanPercentage" :stroke-width="14" :show-text="false" stripe />
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="scanError" class="tool-card">
      <div class="card-body">
        <div class="error-message">{{ scanError }}</div>
      </div>
    </div>

    <!-- 结果展示卡片（仅完成后显示） -->
    <div v-if="scanCompleted && summary" class="tool-card">
      <div class="card-header">
        <span class="card-title">扫描结果</span>
        <div class="card-actions">
          <span class="summary-text">
            {{ summary.totalFiles }} 文件 | {{ formatBytes(summary.totalSize) }} |
            耗时 {{ formatDuration(summary.durationMs) }}
            <span v-if="summary.skippedCount > 0" class="warn-text">
              (跳过 {{ summary.skippedCount }} 个无权限目录)
            </span>
          </span>
        </div>
      </div>
      <div class="card-body">
        <el-tabs v-model="activeTab">
          <el-tab-pane label="文件夹大小" name="folders">
            <FoldersTab v-if="activeTab === 'folders'" :scan-id="scanId" :root-path="scanPath" />
          </el-tab-pane>
          <el-tab-pane label="大文件 Top N" name="topFiles">
            <TopFilesTab v-if="activeTab === 'topFiles'" :scan-id="scanId" />
          </el-tab-pane>
          <el-tab-pane label="按类型" name="extensions">
            <ExtensionsTab v-if="activeTab === 'extensions'" :scan-id="scanId" />
          </el-tab-pane>
          <el-tab-pane
            v-if="opts.detectDuplicates"
            label="重复文件"
            name="duplicates"
          >
            <DuplicatesTab v-if="activeTab === 'duplicates'" :scan-id="scanId" />
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'
import { formatBytes } from '@/utils/systemInfoClient'
import {
  diskScanStart,
  diskScanCancel,
  diskClearScan,
} from '@/utils/diskAnalyzerClient'
import type { ScanOptions, ScanSummary, ScanProgress } from '@/utils/diskAnalyzerTypes'
import FoldersTab from './disk-analyzer-tabs/FoldersTab.vue'
import TopFilesTab from './disk-analyzer-tabs/TopFilesTab.vue'
import ExtensionsTab from './disk-analyzer-tabs/ExtensionsTab.vue'
import DuplicatesTab from './disk-analyzer-tabs/DuplicatesTab.vue'

const store = useToolboxStore()

const scanPath = ref(localStorage.getItem('diskAnalyzer.lastPath') || 'C:\\')
const opts = reactive<ScanOptions>({
  includeHidden: false,
  detectDuplicates: false,
  maxFiles: null,
  followSymlinks: false,
})
const scanning = ref(false)
const scanCompleted = ref(false)
const scanError = ref('')
const scanId = ref('')
const summary = ref<ScanSummary | null>(null)
const progress = ref<ScanProgress | null>(null)
const scanStartTime = ref(0)
const scanElapsedMs = ref(0)
const activeTab = ref('folders')

let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null
let unlistenWarning: (() => void) | null = null
let elapsedTimer: ReturnType<typeof setInterval> | null = null

const scanPercentage = computed(() => {
  // 无准确百分比，用文件数模 1000 模拟进度条动画（ponytail: 仅视觉反馈）
  const n = progress.value?.filesScanned || 0
  return Math.min(95, (n % 1000) / 10 + 5)
})

const formatDuration = (ms: number) => {
  if (ms < 1000) return '0s'
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  return `${m}m ${s % 60}s`
}

const loadLastScanPath = () => {
  const last = localStorage.getItem('diskAnalyzer.lastPath')
  if (last) {
    scanPath.value = last
    ElMessage.info('已加载上次路径')
  }
}

const selectFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === 'string') {
      scanPath.value = selected
      localStorage.setItem('diskAnalyzer.lastPath', selected)
    }
  } catch (e) {
    ElMessage.error(`选择文件夹失败: ${e}`)
  }
}

const startScan = async () => {
  if (!scanPath.value) {
    ElMessage.warning('请先选择扫描路径')
    return
  }
  // 清理上次扫描
  if (scanId.value) {
    try {
      await diskClearScan(scanId.value)
    } catch {
      /* 忽略旧 scan_id 已过期 */
    }
  }
  scanning.value = true
  scanCompleted.value = false
  scanError.value = ''
  summary.value = null
  progress.value = null
  scanStartTime.value = Date.now()
  scanElapsedMs.value = 0
  localStorage.setItem('diskAnalyzer.lastPath', scanPath.value)

  elapsedTimer = setInterval(() => {
    scanElapsedMs.value = Date.now() - scanStartTime.value
  }, 1000)

  try {
    scanId.value = await diskScanStart(scanPath.value, { ...opts })
    ElMessage.success('扫描已启动')
  } catch (e) {
    scanning.value = false
    scanError.value = `启动扫描失败: ${e}`
    if (elapsedTimer) clearInterval(elapsedTimer)
  }
}

const cancelScan = async () => {
  if (!scanId.value) return
  try {
    await diskScanCancel(scanId.value)
    ElMessage.info('已请求取消，等待扫描停止...')
  } catch (e) {
    ElMessage.error(`取消失败: ${e}`)
  }
}

const handleScanComplete = async (event: { payload: { scanId: string; summary: ScanSummary } }) => {
  if (event.payload.scanId !== scanId.value) return
  scanning.value = false
  scanCompleted.value = true
  summary.value = event.payload.summary
  if (elapsedTimer) {
    clearInterval(elapsedTimer)
    elapsedTimer = null
  }
  scanElapsedMs.value = event.payload.summary.durationMs

  // 写入历史记录（AGENTS.md 强制要求 inputFull/outputFull）
  store.addHistory({
    tool: 'diskAnalyzer',
    action: '扫描磁盘',
    inputPreview: scanPath.value.slice(0, 50),
    outputPreview: `${event.payload.summary.totalFiles} 文件 ${formatBytes(event.payload.summary.totalSize)}`.slice(0, 50),
    inputFull: scanPath.value,
    outputFull: JSON.stringify(event.payload.summary),
  })
}

const handleScanProgress = (event: { payload: ScanProgress }) => {
  if (event.payload.scanId !== scanId.value) return
  progress.value = event.payload
}

const handleScanWarning = (event: { payload: { scanId: string; message: string } }) => {
  if (event.payload.scanId !== scanId.value) return
  ElMessage.warning(event.payload.message)
}

onMounted(async () => {
  unlistenProgress = await listen('disk-scan-progress', handleScanProgress as any)
  unlistenComplete = await listen('disk-scan-complete', handleScanComplete as any)
  unlistenWarning = await listen('disk-scan-warning', handleScanWarning as any)
})

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress()
  if (unlistenComplete) unlistenComplete()
  if (unlistenWarning) unlistenWarning()
  if (elapsedTimer) clearInterval(elapsedTimer)
  // 离开页面时释放 Rust 端结果
  if (scanId.value) {
    diskClearScan(scanId.value).catch(() => {})
  }
})
</script>

<style scoped>
.progress-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
  font-size: 13px;
  color: var(--text-secondary, #888);
  word-break: break-all;
}
.summary-text {
  font-size: 13px;
  color: var(--text-secondary, #888);
}
.warn-text {
  color: var(--warning-color, #e6a23c);
  margin-left: 8px;
}
</style>
