<template>
  <div class="tool-container">
    <!-- 操作卡 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">截屏操作</span>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">截屏模式</div>
            <el-button-group>
              <el-button :type="!fullscreenOnly ? 'primary' : 'default'" size="small" @click="fullscreenOnly = false">
                🎯 区域截图
              </el-button>
              <el-button :type="fullscreenOnly ? 'primary' : 'default'" size="small" @click="fullscreenOnly = true">
                🖥 全屏截图
              </el-button>
            </el-button-group>
          </div>

          <div class="action-group">
            <div class="group-label">延时 (秒)</div>
            <el-radio-group v-model="delaySec" size="small">
              <el-radio-button :value="0">立即</el-radio-button>
              <el-radio-button :value="3">3s</el-radio-button>
              <el-radio-button :value="5">5s</el-radio-button>
              <el-radio-button :value="10">10s</el-radio-button>
            </el-radio-group>
          </div>

          <div class="action-group">
            <div class="group-label">开始截图</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="startCapture">
                📸 开始截图
              </el-button>
            </div>
          </div>
        </div>

        <div class="tips">
          <p>
            <b>快捷键：</b>全局 <kbd>Alt</kbd>+<kbd>Shift</kbd>+<kbd>A</kbd> 任意界面一键呼出
            · 选框阶段 <kbd>ESC</kbd> 取消 · <kbd>Ctrl+Z</kbd> 撤销标注 · 双击选框内部=复制到剪贴板
          </p>
          <p>
            <b>标注工具：</b>选框外半透明遮挡，支持矩形框、箭头、文字、马赛克，可调整颜色和粗细
          </p>
          <p>
            <b>输出：</b>完成后一键复制到剪贴板（可直接粘贴到微信/文档/画图），或另存为 PNG
          </p>
        </div>
      </div>
    </div>

    <!-- 设置卡 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">保存设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group action-group-full">
            <div class="group-label">默认保存目录</div>
            <el-input v-model="saveDir" size="small" readonly style="flex: 1">
              <template #append>
                <el-button @click="pickDir">📁 选择</el-button>
              </template>
            </el-input>
          </div>

          <div class="action-group">
            <div class="group-label">默认颜色</div>
            <el-color-picker v-model="defaultColor" size="small" />
          </div>

          <div class="action-group">
            <div class="group-label">默认线粗</div>
            <el-slider v-model="defaultWidth" :min="1" :max="12" style="width: 160px" />
            <span style="color: var(--color-accent); min-width: 24px; text-align: right;">
              {{ defaultWidth }}px
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 最近截图 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">最近截图</span>
          <span class="count-badge">{{ recentList.length }}</span>
        </div>
        <el-button link size="small" @click="clearRecent">清空</el-button>
      </div>
      <div class="card-body">
        <div v-if="!recentList.length" class="empty">
          <div class="empty-icon">🖼</div>
          <p>暂无截图记录，点上方按钮来一张吧~</p>
        </div>
        <div v-else class="recent-grid">
          <div
            v-for="(item, i) in recentList"
            :key="i"
            class="recent-item"
            @click="openPath(item.path)"
          >
            <div class="recent-thumb">
              <img v-if="item.path" :src="asFileUrl(item.path)" :alt="item.path" />
              <div v-else class="clip-indicator">📋 剪贴板</div>
            </div>
            <div class="recent-meta">
              <div class="recent-size">{{ item.size }}</div>
              <div class="recent-time">{{ item.time }}</div>
            </div>
            <div class="recent-actions" @click.stop>
              <el-button v-if="item.path" link size="small" @click="openPath(item.path)">
                打开
              </el-button>
              <el-button link size="small" @click="removeRecent(i)">
                删除
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { open as openPathDialog } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

const fullscreenOnly = ref(false)
const delaySec = ref(0)
const saveDir = ref('')
const defaultColor = ref('#FF4B4B')
const defaultWidth = ref(3)

// 最近截图列表（localStorage 存元数据）
type RecentItem = {
  path: string
  size: string
  time: string
}
const recentList = ref<RecentItem[]>([])
const RECENT_KEY = 'screenshot_recent'

onMounted(async () => {
  saveDir.value = await invoke<string>('screenshot_get_default_dir')
  // 恢复设置
  const saved = localStorage.getItem('screenshot_defaults')
  if (saved) {
    try {
      const d = JSON.parse(saved)
      if (d.color) defaultColor.value = d.color
      if (d.width) defaultWidth.value = d.width
    } catch {}
  }
  loadRecent()

  // 监听保存事件，自动追加到最近
  unlistenSaved = await listen('screenshot://saved', (e: any) => {
    const p = e.payload?.path as string
    const s = e.payload?.size as number
    if (p) appendRecent({
      path: p,
      size: formatBytes(s || 0),
      time: formatTime(new Date()),
    })
  })
})

let unlistenSaved: (() => void) | null = null
onBeforeUnmount(() => {
  if (unlistenSaved) unlistenSaved()
})

function loadRecent() {
  try {
    const raw = localStorage.getItem(RECENT_KEY)
    if (raw) recentList.value = JSON.parse(raw).slice(0, 20)
  } catch {}
}
function persistRecent() {
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recentList.value.slice(0, 20)))
  } catch {}
}
function appendRecent(item: RecentItem) {
  recentList.value.unshift(item)
  recentList.value = recentList.value.slice(0, 20)
  persistRecent()
}
function removeRecent(i: number) {
  recentList.value.splice(i, 1)
  persistRecent()
}
function clearRecent() {
  recentList.value = []
  persistRecent()
}

// 监听设置变化
watch([defaultColor, defaultWidth], () => {
  localStorage.setItem('screenshot_defaults', JSON.stringify({
    color: defaultColor.value,
    width: defaultWidth.value,
  }))
}, { immediate: false })

async function pickDir() {
  const picked = await openPathDialog({
    directory: true,
    multiple: false,
    defaultPath: saveDir.value,
  })
  if (picked) saveDir.value = picked as string
}

function startCapture() {
  // 设置后，ScreenshotOverlay 组件再读
  store.openScreenshotOverlay(delaySec.value)
}

function formatBytes(n: number) {
  if (n < 1024) return n + ' B'
  if (n < 1024 * 1024) return (n / 1024).toFixed(1) + ' KB'
  return (n / 1024 / 1024).toFixed(2) + ' MB'
}
function formatTime(d: Date) {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}
function asFileUrl(p: string) {
  // Tauri asset scope 限制，改用后端 read_file_base64 或 convertFileSrc
  // 优先用 convertFileSrc
  try {
    const { convertFileSrc } = require('@tauri-apps/api/core')
    return convertFileSrc(p)
  } catch {
    return 'file:///' + p.replace(/\\/g, '/')
  }
}
async function openPath(p: string) {
  try {
    await invoke('ql_open_file', { path: p })
  } catch (e: any) {
    ElMessage.error('打开失败：' + (e?.message || String(e)))
  }
}
</script>

<style scoped>
.tool-container {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tips {
  margin-top: 16px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-left: 3px solid var(--color-accent);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 12.5px;
  line-height: 1.75;
}
.tips p { margin: 0; }
.tips b { color: var(--color-accent); }
.tips kbd {
  font-family: 'Consolas', monospace;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 1px 6px;
  font-size: 11.5px;
  margin: 0 2px;
  color: var(--text-primary);
}

.action-group-full { flex: 1 1 100%; }

.count-badge {
  margin-left: 8px;
  padding: 1px 8px;
  background: var(--color-accent);
  color: #0b1220;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.empty {
  padding: 40px 0;
  text-align: center;
  color: var(--text-muted);
}
.empty-icon {
  font-size: 48px;
  opacity: 0.5;
  margin-bottom: 10px;
}

.recent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.recent-item {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
}
.recent-item:hover {
  border-color: var(--color-accent);
  box-shadow: 0 4px 16px rgba(0, 229, 255, 0.1);
  transform: translateY(-1px);
}

.recent-thumb {
  height: 120px;
  background: var(--bg-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-bottom: 1px solid var(--border-color);
}
.recent-thumb img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.clip-indicator {
  color: var(--color-accent);
  font-size: 14px;
  font-weight: 600;
}

.recent-meta {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  justify-content: space-between;
}
.recent-size {
  color: var(--color-accent);
  font-weight: 600;
}
.recent-time {
  opacity: 0.8;
  font-variant-numeric: tabular-nums;
}

.recent-actions {
  padding: 0 8px 8px;
  display: flex;
  justify-content: flex-end;
  gap: 4px;
}
</style>
