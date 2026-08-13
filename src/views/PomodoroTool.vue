<template>
  <div class="tool-container">
    <!-- 状态卡片 -->
    <div class="tool-card pomo-status-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">🍅 番茄钟</span>
          <span class="phase-badge" :class="`phase-${phase}`">{{ phaseLabel }}</span>
        </div>
        <div class="header-actions">
          <span class="cycle-info">已完成 {{ cycleFocusDone }} 轮专注</span>
        </div>
      </div>
      <div class="card-body pomo-body">
        <div class="pomo-time" :class="{ paused: !running && phase !== 'idle' }">{{ timeText }}</div>
        <div class="pomo-progress">
          <div class="pomo-progress-bar" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="pomo-controls">
          <el-button type="primary" size="large" @click="onToggle">
            {{ running ? '暂停' : phase === 'idle' ? '开始' : '继续' }}
          </el-button>
          <el-button size="large" @click="onSkip">跳过</el-button>
          <el-button size="large" @click="onReset">重置</el-button>
        </div>
        <div v-if="phase === 'idle'" class="pomo-hint">点击「开始」进入第一轮专注，每轮完成后自动切换休息</div>
        <div v-else-if="!running" class="pomo-hint">
          {{ phase === 'focus' ? '本轮专注已就绪' : '休息时间已就绪' }}，点击「继续」开始
        </div>
      </div>
    </div>

    <!-- 设置卡片 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">时长设置</span>
        </div>
        <div class="header-actions">
          <el-button type="primary" size="small" @click="onSaveSettings">保存设置</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="settings-grid">
          <div class="setting-item">
            <span class="setting-label">专注时长</span>
            <el-input-number v-model="focusMinutes" :min="1" :max="180" size="small" controls-position="right" />
            <span class="setting-unit">分钟</span>
          </div>
          <div class="setting-item">
            <span class="setting-label">短休息</span>
            <el-input-number v-model="shortBreakMinutes" :min="1" :max="60" size="small" controls-position="right" />
            <span class="setting-unit">分钟</span>
          </div>
          <div class="setting-item">
            <span class="setting-label">长休息</span>
            <el-input-number v-model="longBreakMinutes" :min="1" :max="120" size="small" controls-position="right" />
            <span class="setting-unit">分钟</span>
          </div>
          <div class="setting-item">
            <span class="setting-label">长休间隔</span>
            <el-input-number v-model="longBreakInterval" :min="1" :max="12" size="small" controls-position="right" />
            <span class="setting-unit">轮专注后</span>
          </div>
        </div>
        <div class="pomo-desc">设置保存后立即生效，重新开始计时时按新时长计算</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

interface Snapshot {
  phase: string
  running: boolean
  remainingSecs: number
  totalSecs: number
  cycleFocusDone: number
  focusSecs: number
  shortBreakSecs: number
  longBreakSecs: number
  longBreakInterval: number
}

const store = useToolboxStore()

const phase = ref('idle')
const running = ref(false)
const remainingSecs = ref(0)
const totalSecs = ref(1)
const cycleFocusDone = ref(0)

const focusMinutes = ref(25)
const shortBreakMinutes = ref(5)
const longBreakMinutes = ref(15)
const longBreakInterval = ref(4)

let tickUnlisten: UnlistenFn | null = null
let finishUnlisten: UnlistenFn | null = null
let shortcutUnlisten: UnlistenFn | null = null

const PHASE_META: Record<string, { label: string }> = {
  focus: { label: '专注中' },
  short_break: { label: '短休息' },
  long_break: { label: '长休息' },
  idle: { label: '待机' },
}

const phaseLabel = computed(() => PHASE_META[phase.value]?.label || '待机')

const timeText = computed(() => {
  const s = remainingSecs.value
  const m = Math.floor(s / 60)
  const sec = s % 60
  return `${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
})

const progress = computed(() => {
  if (!totalSecs.value) return 0
  return Math.max(0, Math.min(100, (remainingSecs.value / totalSecs.value) * 100))
})

const applySnapshot = (s: Snapshot) => {
  phase.value = s.phase || 'idle'
  running.value = s.running
  remainingSecs.value = s.remainingSecs
  totalSecs.value = s.totalSecs || 1
  cycleFocusDone.value = s.cycleFocusDone || 0
  if (s.focusSecs) focusMinutes.value = Math.round(s.focusSecs / 60)
  if (s.shortBreakSecs) shortBreakMinutes.value = Math.round(s.shortBreakSecs / 60)
  if (s.longBreakSecs) longBreakMinutes.value = Math.round(s.longBreakSecs / 60)
  if (s.longBreakInterval) longBreakInterval.value = s.longBreakInterval
}

/** 蜂鸣提醒 */
function beep() {
  try {
    const Ctx = window.AudioContext || (window as any).webkitAudioContext
    const ctx = new Ctx()
    const now = ctx.currentTime
    for (let i = 0; i < 3; i++) {
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.type = 'sine'
      osc.frequency.value = 880
      osc.connect(gain)
      gain.connect(ctx.destination)
      const t = now + i * 0.4
      gain.gain.setValueAtTime(0.0001, t)
      gain.gain.exponentialRampToValueAtTime(0.4, t + 0.02)
      gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.3)
      osc.start(t)
      osc.stop(t + 0.32)
    }
  } catch {
    /* ignore */
  }
}

const onToggle = () => {
  invoke('pomodoro_toggle').catch((e: any) => ElMessage.error(`操作失败: ${e?.message || e || '未知错误'}`))
}

const onSkip = () => {
  invoke('pomodoro_skip').catch((e) => ElMessage.error(e))
}

const onReset = () => {
  invoke('pomodoro_reset').catch((e) => ElMessage.error(e))
}

const onSaveSettings = async () => {
  try {
    await invoke('pomodoro_set_settings', {
      focusSecs: focusMinutes.value * 60,
      shortBreakSecs: shortBreakMinutes.value * 60,
      longBreakSecs: longBreakMinutes.value * 60,
      longBreakInterval: longBreakInterval.value,
    })
    ElMessage.success('设置已保存')
  } catch (e: any) {
    ElMessage.error(e?.message || '保存失败')
  }
}

onMounted(async () => {
  try {
    const s = await invoke<Snapshot>('pomodoro_state')
    applySnapshot(s)
  } catch {
    /* ignore */
  }

  tickUnlisten = await listen<Snapshot>('pomodoro-tick', (e) => {
    applySnapshot(e.payload)
  })

  // 到点提醒
  finishUnlisten = await listen('pomodoro-finished', () => {
    if (store.activeTabId === 'pomodoro') {
      beep()
    }
  })

  // 全局快捷键：Ctrl+Alt+Shift+P 切换开始/暂停
  shortcutUnlisten = await listen<string>('global-shortcut-triggered', (e) => {
    if (e.payload === 'pomodoro') {
      onToggle()
    }
  })
})

onBeforeUnmount(() => {
  tickUnlisten?.()
  finishUnlisten?.()
  shortcutUnlisten?.()
})
</script>

<style scoped>
.phase-badge {
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 10px;
  margin-left: 10px;
  color: #fff;
}
.phase-focus {
  background: #ff6b6b;
}
.phase-short_break {
  background: #4ecdc4;
}
.phase-long_break {
  background: #5b8def;
}
.phase-idle {
  background: #8a8a96;
}

.cycle-info {
  font-size: 13px;
  color: var(--text-secondary, #888);
}

.pomo-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 20px 16px;
}

.pomo-time {
  font-size: 72px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: 2px;
  line-height: 1;
  color: var(--text-primary, #e8e8ef);
}
.pomo-time.paused {
  color: #ffd166;
}

.pomo-progress {
  width: 100%;
  max-width: 480px;
  height: 8px;
  border-radius: 4px;
  background: rgba(127, 127, 127, 0.18);
  overflow: hidden;
}
.pomo-progress-bar {
  height: 100%;
  border-radius: 4px;
  background: linear-gradient(90deg, #ff6b6b, #ffa07a);
  transition: width 1s linear;
}

.pomo-controls {
  display: flex;
  gap: 8px;
}
.pomo-controls .el-button + .el-button {
  margin-left: 0;
}

.pomo-hint {
  font-size: 13px;
  color: var(--text-secondary, #888);
}

.pomo-desc {
  font-size: 13px;
  color: var(--text-secondary, #888);
  line-height: 1.6;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 12px;
}
.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
}
.setting-label {
  font-size: 13px;
  color: var(--text-primary, #e8e8ef);
  white-space: nowrap;
}
.setting-unit {
  font-size: 12px;
  color: var(--text-secondary, #888);
  white-space: nowrap;
}
</style>
