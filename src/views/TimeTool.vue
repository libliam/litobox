<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="time-tabs" @tab-click="handleTabClick">
      <el-tab-pane label="时间戳转换" name="timestamp" />
      <el-tab-pane label="时区转换" name="timezone" />
      <el-tab-pane label="时间差计算" name="diff" />
      <el-tab-pane label="格式化代码" name="format" />
      <el-tab-pane label="倒计时" name="countdown" />
      <el-tab-pane label="相对时间" name="relative" />
    </el-tabs>

    <!-- 当前时间显示 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">当前时间</span>
      </div>
      <div class="card-body">
        <div class="current-time">
          <div class="time-display">{{ currentTime }}</div>
          <div class="timestamp-row">
            <span class="ts-label">毫秒戳：</span>
            <span class="ts-value">{{ currentTimestampMs }}</span>
            <el-button size="small" @click="handleCopy(currentTimestampMs)">复制</el-button>
          </div>
          <div class="timestamp-row">
            <span class="ts-label">秒级戳：</span>
            <span class="ts-value">{{ currentTimestampS }}</span>
            <el-button size="small" @click="handleCopy(currentTimestampS)">复制</el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 时间戳转换 Tab -->
    <div v-if="activeTab === 'timestamp'" class="tool-card">
      <div class="card-header">
        <span class="card-title">时间戳 → 日期时间</span>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariable" />
          <el-button size="small" @click="handleClear">清空</el-button>
          <el-button size="small" @click="handlePaste">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>时间戳：</label>
          <el-input v-model="tsInput" placeholder="输入时间戳..." />
          <el-radio-group v-model="tsMode" size="small">
            <el-radio-button label="ms">毫秒</el-radio-button>
            <el-radio-button label="s">秒</el-radio-button>
          </el-radio-group>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'timestamp'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleTsToDatetime">转换</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="tsResult" readonly type="textarea" :rows="2" resize="vertical" />
        <div class="copy-row">
          <el-button size="small" @click="handleCopy(tsResult)">复制</el-button>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'timestamp'" class="tool-card">
      <div class="card-header">
        <span class="card-title">日期时间 → 时间戳</span>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>日期时间：</label>
          <el-input v-model="dtInput" placeholder="如 2026-06-23 12:00:00" />
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'timestamp'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleDatetimeToTs">转换</el-button>
      </div>
      <div class="card-body">
        <div class="ts-result-row" v-if="dtResultMs">
          <span class="ts-label">毫秒：</span>
          <span class="ts-value">{{ dtResultMs }}</span>
          <el-button size="small" @click="handleCopy(dtResultMs)">复制</el-button>
        </div>
        <div class="ts-result-row" v-if="dtResultS">
          <span class="ts-label">秒：</span>
          <span class="ts-value">{{ dtResultS }}</span>
          <el-button size="small" @click="handleCopy(dtResultS)">复制</el-button>
        </div>
        <div v-if="dtError" class="error-message">{{ dtError }}</div>
      </div>
    </div>

    <!-- 时区转换 Tab -->
    <div v-if="activeTab === 'timezone'" class="tool-card">
      <div class="card-header">
        <span class="card-title">时区转换</span>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>日期时间：</label>
          <el-input v-model="tzDatetime" placeholder="如 2026-06-23 12:00:00" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>从时区：</label>
          <el-select v-model="tzFrom" style="width: 180px">
            <el-option label="UTC+8 北京" value="8" />
            <el-option label="UTC+0 伦敦" value="0" />
            <el-option label="UTC-5 纽约" value="-5" />
            <el-option label="UTC-8 洛杉矶" value="-8" />
            <el-option label="UTC+9 东京" value="9" />
            <el-option label="UTC+5:30 印度" value="5.5" />
          </el-select>
          <label style="margin-left: 16px">到时区：</label>
          <el-select v-model="tzTo" style="width: 180px">
            <el-option label="UTC+8 北京" value="8" />
            <el-option label="UTC+0 伦敦" value="0" />
            <el-option label="UTC-5 纽约" value="-5" />
            <el-option label="UTC-8 洛杉矶" value="-8" />
            <el-option label="UTC+9 东京" value="9" />
            <el-option label="UTC+5:30 印度" value="5.5" />
          </el-select>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'timezone'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleTimezoneConvert">转换</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="tzResult" readonly type="textarea" :rows="2" resize="vertical" />
        <div class="copy-row">
          <el-button size="small" @click="handleCopy(tzResult)">复制</el-button>
        </div>
      </div>
    </div>

    <!-- 时间差计算 Tab -->
    <div v-if="activeTab === 'diff'" class="tool-card">
      <div class="card-header">
        <span class="card-title">时间差计算</span>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>开始时间：</label>
          <el-input v-model="diffStart" placeholder="如 2026-01-01 00:00:00" />
        </div>
        <div class="input-row" style="margin-top: 12px">
          <label>结束时间：</label>
          <el-input v-model="diffEnd" placeholder="如 2026-06-23 12:00:00" />
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'diff'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleTimeDiff">计算</el-button>
      </div>
      <div class="card-body">
        <div v-if="diffResult" class="diff-results">
          <div class="diff-item"><span class="diff-label">总天数：</span>{{ diffResult.days }} 天</div>
          <div class="diff-item"><span class="diff-label">总小时：</span>{{ diffResult.hours }} 小时</div>
          <div class="diff-item"><span class="diff-label">总分钟：</span>{{ diffResult.minutes }} 分钟</div>
          <div class="diff-item"><span class="diff-label">总秒数：</span>{{ diffResult.seconds }} 秒</div>
          <div class="diff-item"><span class="diff-label">详细：</span>{{ diffResult.detail }}</div>
        </div>
        <div v-if="diffError" class="error-message">{{ diffError }}</div>
      </div>
    </div>

    <!-- 格式化代码 Tab -->
    <div v-if="activeTab === 'format'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">日期格式化代码</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 生成各语言日期格式化代码</p>
                <p>• 支持 JavaScript、Python、Java 等</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>语言：</label>
          <el-select v-model="formatLang" style="width: 150px">
            <el-option label="JavaScript" value="js" />
            <el-option label="Python" value="python" />
            <el-option label="Java" value="java" />
            <el-option label="C#" value="csharp" />
            <el-option label="Go" value="go" />
            <el-option label="Rust" value="rust" />
          </el-select>
          <label style="margin-left: 16px">格式：</label>
          <el-select v-model="formatPattern" style="width: 200px">
            <el-option label="YYYY-MM-DD HH:mm:ss" value="full" />
            <el-option label="YYYY-MM-DD" value="date" />
            <el-option label="HH:mm:ss" value="time" />
            <el-option label="YYYY/MM/DD" value="slash" />
            <el-option label="Unix 时间戳" value="timestamp" />
          </el-select>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'format'" class="tool-card">
      <div class="card-header">
        <span class="card-title">代码</span>
        <el-button size="small" @click="handleCopy(formatCode)">复制</el-button>
      </div>
      <div class="card-body">
        <el-input :model-value="formatCode" readonly type="textarea" :rows="6" resize="vertical" class="code-input" />
      </div>
    </div>

    <!-- 倒计时 Tab -->
    <div v-if="activeTab === 'countdown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">倒计时</span>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>目标时间：</label>
          <el-input v-model="countdownTarget" placeholder="如 2026-12-31 23:59:59" />
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'countdown'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleCountdownStart">{{ countdownRunning ? '停止' : '开始' }}</el-button>
      </div>
      <div class="card-body">
        <div class="countdown-display">
          <div class="countdown-item">
            <span class="countdown-value">{{ countdownDays }}</span>
            <span class="countdown-label">天</span>
          </div>
          <div class="countdown-sep">:</div>
          <div class="countdown-item">
            <span class="countdown-value">{{ countdownHours }}</span>
            <span class="countdown-label">时</span>
          </div>
          <div class="countdown-sep">:</div>
          <div class="countdown-item">
            <span class="countdown-value">{{ countdownMinutes }}</span>
            <span class="countdown-label">分</span>
          </div>
          <div class="countdown-sep">:</div>
          <div class="countdown-item">
            <span class="countdown-value">{{ countdownSeconds }}</span>
            <span class="countdown-label">秒</span>
          </div>
        </div>
        <div v-if="countdownExpired" class="countdown-expired">倒计时已结束</div>
      </div>
    </div>

    <!-- 相对时间 Tab -->
    <div v-if="activeTab === 'relative'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">相对时间</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 将时间戳或日期转换为相对时间描述</p>
                <p>• 如 "3天前"、"2小时后"、"刚刚"</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="handleInsertVariableRelative" />
          <el-button size="small" @click="handleClearRelative">清空</el-button>
          <el-button size="small" @click="handlePasteRelative">粘贴</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="input-row">
          <label>时间戳/日期：</label>
          <el-input v-model="relativeInput" placeholder="输入时间戳或日期，如 1719100800000、2026-06-23 12:00:00" />
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'relative'" class="tool-card">
      <div class="card-header">
        <span class="card-title">操作</span>
        <el-button size="small" type="primary" @click="handleRelativeTime">转换</el-button>
      </div>
      <div class="card-body">
        <div v-if="relativeResult" class="relative-result">
          <div class="relative-main">{{ relativeResult.text }}</div>
          <div class="relative-detail">{{ relativeResult.detail }}</div>
        </div>
        <div v-if="relativeError" class="error-message">{{ relativeError }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'

const store = useToolboxStore()
const activeTab = ref('timestamp')

// 当前时间
const currentTime = ref('')
const currentTimestampMs = ref('')
const currentTimestampS = ref('')
let timeInterval: number | null = null

const updateTime = () => {
  const now = new Date()
  currentTime.value = now.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
  currentTimestampMs.value = String(now.getTime())
  currentTimestampS.value = String(Math.floor(now.getTime() / 1000))
}

// 时间戳转换
const tsInput = ref('')
const tsMode = ref<'ms' | 's'>('ms')
const tsResult = ref('')

// 日期 → 时间戳
const dtInput = ref('')
const dtResultMs = ref('')
const dtResultS = ref('')
const dtError = ref('')

// 时区转换
const tzDatetime = ref('')
const tzFrom = ref('8')
const tzTo = ref('0')
const tzResult = ref('')

// 时间差
const diffStart = ref('')
const diffEnd = ref('')
const diffResult = ref<{ days: number; hours: number; minutes: number; seconds: number; detail: string } | null>(null)
const diffError = ref('')

// 格式化代码
const formatLang = ref('js')
const formatPattern = ref('full')
const formatCode = computed(() => {
  const patterns: Record<string, Record<string, string>> = {
    js: {
      full: "new Date().toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })",
      date: "new Date().toISOString().split('T')[0]",
      time: "new Date().toLocaleTimeString('zh-CN', { hour12: false })",
      slash: "new Date().toLocaleDateString('zh-CN').replace(/\\//g, '/')",
      timestamp: 'Date.now()'
    },
    python: {
      full: "datetime.now().strftime('%Y-%m-%d %H:%M:%S')",
      date: "datetime.now().strftime('%Y-%m-%d')",
      time: "datetime.now().strftime('%H:%M:%S')",
      slash: "datetime.now().strftime('%Y/%m/%d')",
      timestamp: 'int(time.time())'
    },
    java: {
      full: 'LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss"))',
      date: 'LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)',
      time: 'LocalTime.now().format(DateTimeFormatter.ISO_LOCAL_TIME)',
      slash: 'LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyy/MM/dd"))',
      timestamp: 'System.currentTimeMillis()'
    },
    csharp: {
      full: 'DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss")',
      date: 'DateTime.Now.ToString("yyyy-MM-dd")',
      time: 'DateTime.Now.ToString("HH:mm:ss")',
      slash: 'DateTime.Now.ToString("yyyy/MM/dd")',
      timestamp: 'new DateTimeOffset(DateTime.Now).ToUnixTimeMilliseconds()'
    },
    go: {
      full: 'time.Now().Format("2006-01-02 15:04:05")',
      date: 'time.Now().Format("2006-01-02")',
      time: 'time.Now().Format("15:04:05")',
      slash: 'time.Now().Format("2006/01/02")',
      timestamp: 'time.Now().UnixMilli()'
    },
    rust: {
      full: 'chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()',
      date: 'chrono::Local::now().format("%Y-%m-%d").to_string()',
      time: 'chrono::Local::now().format("%H:%M:%S").to_string()',
      slash: 'chrono::Local::now().format("%Y/%m/%d").to_string()',
      timestamp: 'chrono::Local::now().timestamp_millis()'
    }
  }
  return patterns[formatLang.value]?.[formatPattern.value] || ''
})

// 倒计时
const countdownTarget = ref('')
const countdownRunning = ref(false)
const countdownDays = ref('00')
const countdownHours = ref('00')
const countdownMinutes = ref('00')
const countdownSeconds = ref('00')
const countdownExpired = ref(false)
let countdownInterval: number | null = null

const handleTabClick = () => {}

const handleClear = () => {
  tsInput.value = ''
  tsResult.value = ''
}

const handlePaste = async () => {
  try {
    const text = await navigator.clipboard.readText()
    tsInput.value = text
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariable = (value: string) => {
  tsInput.value = value
}

const handleCopy = async (text: string) => {
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleTsToDatetime = () => {
  const ts = Number(tsInput.value)
  if (isNaN(ts)) {
    ElMessage.warning('请输入有效的时间戳')
    return
  }
  const ms = tsMode.value === 's' ? ts * 1000 : ts
  const date = new Date(ms)
  if (isNaN(date.getTime())) {
    tsResult.value = '无效的时间戳'
    return
  }
  tsResult.value = date.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
  store.addHistory({ tool: 'time', action: 'timestamp-to-date', inputPreview: tsInput.value, outputPreview: tsResult.value })
  ElMessage.success('转换完成')
}

const handleDatetimeToTs = () => {
  const date = new Date(dtInput.value)
  if (isNaN(date.getTime())) {
    dtError.value = '无效的日期时间格式'
    dtResultMs.value = ''
    dtResultS.value = ''
    return
  }
  dtError.value = ''
  dtResultMs.value = String(date.getTime())
  dtResultS.value = String(Math.floor(date.getTime() / 1000))
  store.addHistory({ tool: 'time', action: 'date-to-timestamp', inputPreview: dtInput.value, outputPreview: dtResultMs.value })
  ElMessage.success('转换完成')
}

const handleTimezoneConvert = () => {
  const date = new Date(tzDatetime.value)
  if (isNaN(date.getTime())) {
    tzResult.value = '无效的日期时间格式'
    return
  }
  const fromOffset = Number(tzFrom.value) * 3600000
  const toOffset = Number(tzTo.value) * 3600000
  const utc = date.getTime() - fromOffset
  const target = new Date(utc + toOffset)
  tzResult.value = target.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
  store.addHistory({ tool: 'time', action: 'timezone', inputPreview: tzDatetime.value, outputPreview: tzResult.value })
  ElMessage.success('转换完成')
}

const handleTimeDiff = () => {
  const start = new Date(diffStart.value)
  const end = new Date(diffEnd.value)
  if (isNaN(start.getTime()) || isNaN(end.getTime())) {
    diffError.value = '请输入有效的日期'
    diffResult.value = null
    return
  }
  diffError.value = ''
  const diffMs = Math.abs(end.getTime() - start.getTime())
  const totalSeconds = Math.floor(diffMs / 1000)
  const totalMinutes = Math.floor(totalSeconds / 60)
  const totalHours = Math.floor(totalMinutes / 60)
  const totalDays = Math.floor(totalHours / 24)

  const remainingHours = totalHours % 24
  const remainingMinutes = totalMinutes % 60
  const remainingSeconds = totalSeconds % 60

  diffResult.value = {
    days: totalDays,
    hours: totalHours,
    minutes: totalMinutes,
    seconds: totalSeconds,
    detail: `${totalDays}天 ${remainingHours}小时 ${remainingMinutes}分钟 ${remainingSeconds}秒`
  }
  store.addHistory({ tool: 'time', action: 'diff', inputPreview: `${diffStart.value} ~ ${diffEnd.value}`, outputPreview: diffResult.value.detail })
  ElMessage.success('计算完成')
}

// 相对时间
const relativeInput = ref('')
const relativeResult = ref<{ text: string; detail: string } | null>(null)
const relativeError = ref('')

const handleCountdownStart = () => {
  if (countdownRunning.value) {
    countdownRunning.value = false
    if (countdownInterval) clearInterval(countdownInterval)
    countdownInterval = null
    return
  }
  const target = new Date(countdownTarget.value)
  if (isNaN(target.getTime())) {
    ElMessage.warning('请输入有效的目标时间')
    return
  }
  countdownRunning.value = true
  countdownExpired.value = false

  const updateCountdown = () => {
    const now = new Date().getTime()
    const diff = target.getTime() - now
    if (diff <= 0) {
      countdownDays.value = '00'
      countdownHours.value = '00'
      countdownMinutes.value = '00'
      countdownSeconds.value = '00'
      countdownExpired.value = true
      countdownRunning.value = false
      if (countdownInterval) clearInterval(countdownInterval)
      return
    }
    const days = Math.floor(diff / (1000 * 60 * 60 * 24))
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
    const seconds = Math.floor((diff % (1000 * 60)) / 1000)
    countdownDays.value = String(days).padStart(2, '0')
    countdownHours.value = String(hours).padStart(2, '0')
    countdownMinutes.value = String(minutes).padStart(2, '0')
    countdownSeconds.value = String(seconds).padStart(2, '0')
  }

  updateCountdown()
  countdownInterval = window.setInterval(updateCountdown, 1000)
}

onMounted(() => {
  updateTime()
  timeInterval = window.setInterval(updateTime, 1000)
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
  if (countdownInterval) clearInterval(countdownInterval)
})

// 相对时间处理
const getRelativeTime = (targetDate: Date): { text: string; detail: string } => {
  const now = new Date()
  const diffMs = targetDate.getTime() - now.getTime()
  const absDiffMs = Math.abs(diffMs)
  const isFuture = diffMs > 0
  const suffix = isFuture ? '后' : '前'

  const seconds = Math.floor(absDiffMs / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  const months = Math.floor(days / 30)
  const years = Math.floor(days / 365)

  let text: string
  if (seconds < 60) {
    text = seconds < 10 ? '刚刚' : `${seconds}秒${suffix}`
  } else if (minutes < 60) {
    text = `${minutes}分钟${suffix}`
  } else if (hours < 24) {
    text = `${hours}小时${suffix}`
  } else if (days < 30) {
    text = `${days}天${suffix}`
  } else if (days < 365) {
    text = `${months}个月${suffix}`
  } else {
    text = `${years}年${suffix}`
  }

  const detail = targetDate.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })

  return { text, detail }
}

const handleRelativeTime = () => {
  const input = relativeInput.value.trim()
  if (!input) {
    relativeError.value = '请输入时间戳或日期'
    relativeResult.value = null
    return
  }

  let date: Date

  // 尝试解析为时间戳
  const ts = Number(input)
  if (!isNaN(ts)) {
    // 自动判断秒/毫秒
    date = new Date(ts < 1e12 ? ts * 1000 : ts)
  } else {
    // 尝试解析为日期字符串
    date = new Date(input)
  }

  if (isNaN(date.getTime())) {
    relativeError.value = '无效的时间格式，请输入有效的时间戳或日期（如 2026-06-23 12:00:00）'
    relativeResult.value = null
    return
  }

  relativeError.value = ''
  relativeResult.value = getRelativeTime(date)
  store.addHistory({ tool: 'time', action: 'relative', inputPreview: input, outputPreview: relativeResult.value.text })
  ElMessage.success('转换完成')
}

const handleClearRelative = () => {
  relativeInput.value = ''
  relativeResult.value = null
  relativeError.value = ''
}

const handlePasteRelative = async () => {
  try {
    const text = await navigator.clipboard.readText()
    relativeInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.warning('无法读取剪贴板')
  }
}

const handleInsertVariableRelative = (value: string) => {
  relativeInput.value = value
}
</script>

<style scoped>
/* 二级 Tab（子功能切换） */
.time-tabs {
  margin-bottom: 8px;
  margin-top: -4px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px 8px 4px 12px;
}

.time-tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
  padding-left: 0;
  border-bottom: none;
}

.time-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 0;
}

.time-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.time-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 400;
  padding: 0 12px;
  height: 28px;
  line-height: 28px;
  border-radius: 4px;
  margin-right: 4px;
  transition: all 0.2s;
}

.time-tabs :deep(.el-tabs__item:hover) {
  color: var(--text-primary);
  background: rgba(0, 212, 255, 0.05);
}

.time-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
  font-weight: 500;
  background: rgba(0, 212, 255, 0.1);
}

.time-tabs :deep(.el-tabs__active-bar) {
  display: none;
}

html.light .time-tabs {
  background: var(--bg-card);
  border-color: var(--border-color);
}

html.light .time-tabs :deep(.el-tabs__item:hover) {
  background: rgba(8, 145, 178, 0.05);
}

html.light .time-tabs :deep(.el-tabs__item.is-active) {
  background: rgba(8, 145, 178, 0.1);
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:last-child {
  margin-bottom: 0;
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-body {
  padding: 16px 20px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.input-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  min-width: 70px;
}

.current-time {
  text-align: center;
}

.time-display {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent-cyan);
  letter-spacing: 2px;
  margin-bottom: 12px;
}

.timestamp-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 6px;
}

.ts-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.ts-value {
  font-size: 14px;
  color: var(--text-primary);
  font-family: 'Courier New', monospace;
}

.copy-row {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.ts-result-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.diff-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.diff-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.diff-label {
  color: var(--text-secondary);
  min-width: 70px;
}

.code-input :deep(.el-textarea__inner) {
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.countdown-display {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px 0;
}

.countdown-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  min-width: 70px;
}

.countdown-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--accent-cyan);
  font-family: 'Courier New', monospace;
}

.countdown-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.countdown-sep {
  font-size: 28px;
  color: var(--text-secondary);
  font-weight: 700;
}

.countdown-expired {
  text-align: center;
  color: var(--accent-red);
  font-size: 18px;
  font-weight: 600;
  padding: 16px;
}

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}

.hint-icon:hover {
  color: var(--accent-cyan);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tooltip-content {
  max-width: 320px;
  line-height: 1.6;
}

.tooltip-content p {
  margin: 2px 0;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
}

/* 相对时间结果 */
.relative-result {
  padding: 20px;
  text-align: center;
}

.relative-main {
  font-size: 36px;
  font-weight: 700;
  color: var(--accent-cyan);
  margin-bottom: 8px;
}

.relative-detail {
  font-size: 14px;
  color: var(--text-secondary);
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
