<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="calculator-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 计算器 -->
      <el-tab-pane label="计算器" name="calculator">
        <!-- 表达式计算 -->
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">表达式</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>基础运算: + - * / ( ) %</p>
                    <p>幂与根: ^ sqrt()</p>
                    <p>三角函数: sin() cos() tan()</p>
                    <p>对数: log() log10()</p>
                    <p>常量: pi, e</p>
                    <p>绝对值: abs()</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <VariablePicker @select="handleVarSelect" />
              <el-button size="small" @click="calcInput = ''; calcResult = null; calcError = ''">清空</el-button>
              <el-button size="small" @click="handlePaste">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="expr-input-row">
              <el-input
                v-model="calcInput"
                placeholder="输入表达式，如 3 * (4 + 5) / 2"
                size="large"
                clearable
                @keyup.enter="handleCalculate"
              >
                <template #prefix>
                  <span class="expr-prefix">=</span>
                </template>
              </el-input>
              <el-button type="primary" size="large" @click="handleCalculate" class="calc-btn">
                计算
              </el-button>
            </div>
          </div>
        </div>

        <!-- 计算结果 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">结果</span>
            <div class="card-actions">
              <el-button size="small" @click="handleCopyResult">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div v-if="calcResult === null && !calcError" class="result-placeholder">
              输入表达式后点击"计算"或按 Enter
            </div>
            <div v-if="calcResult !== null" class="result-display">
              <span class="result-equal">{{ calcInput }} = </span>
              <span class="result-value">{{ calcResult }}</span>
            </div>
            <div v-if="calcError" class="error-message">{{ calcError }}</div>
          </div>
        </div>

        <!-- 单位换算 -->
        <div class="tool-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">单位换算</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>支持长度、重量、温度等 11 类单位实时换算</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
          <div class="card-body">
            <div class="convert-config">
              <el-select v-model="convertCategory" size="small" style="width: 140px" @change="handleCategoryChange">
                <el-option v-for="cat in categories" :key="cat.id" :label="cat.name" :value="cat.id" />
              </el-select>
            </div>
            <div class="convert-row">
              <div class="convert-side">
                <el-input
                  v-model.number="convertFromValue"
                  type="number"
                  placeholder="输入数值"
                  @input="handleConvert"
                />
                <el-select v-model="convertFromUnit" size="small" @change="handleConvert">
                  <el-option v-for="u in currentFromUnits" :key="u" :label="u" :value="u" />
                </el-select>
              </div>
              <button class="swap-btn" @click="handleSwap" title="交换单位">
                <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M7 16l-4-4 4-4"/>
                  <path d="M17 8l4 4-4 4"/>
                  <path d="M3 12h18"/>
                </svg>
              </button>
              <div class="convert-side">
                <el-input
                  :model-value="convertToValue"
                  placeholder="结果"
                  readonly
                />
                <el-select v-model="convertToUnit" size="small" @change="handleConvert">
                  <el-option v-for="u in currentToUnits" :key="u" :label="u" :value="u" />
                </el-select>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 日期工具 -->
      <el-tab-pane label="日期工具" name="date">
        <!-- 日期计算 -->
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">日期计算</span>
          </div>
          <div class="card-body">
            <div class="date-mode-switch">
              <el-radio-group v-model="dateMode" size="small">
                <el-radio-button value="diff">两个日期之差</el-radio-button>
                <el-radio-button value="add">日期加减时间</el-radio-button>
              </el-radio-group>
            </div>

            <!-- 模式A: 日期差 -->
            <div v-if="dateMode === 'diff'" class="date-diff-section">
              <div class="date-picker-row">
                <div class="date-picker-item">
                  <span class="date-label">起始</span>
                  <el-date-picker v-model="dateStart" type="date" placeholder="选择起始日期" size="small" value-format="YYYY-MM-DD" />
                </div>
                <div class="date-picker-item">
                  <span class="date-label">结束</span>
                  <el-date-picker v-model="dateEnd" type="date" placeholder="选择结束日期" size="small" value-format="YYYY-MM-DD" />
                </div>
              </div>
              <el-button type="primary" size="small" @click="handleDateDiff" class="date-calc-btn">计算差值</el-button>
              <div v-if="dateDiffResult !== null" class="date-result">
                相差 {{ dateDiffResult.days }} 天（约 {{ dateDiffResult.months }} 个月，{{ dateDiffResult.weeks }} 周）
              </div>
            </div>

            <!-- 模式B: 日期加减 -->
            <div v-if="dateMode === 'add'" class="date-add-section">
              <div class="date-picker-row">
                <div class="date-picker-item">
                  <span class="date-label">起始</span>
                  <el-date-picker v-model="dateAddStart" type="date" placeholder="选择起始日期" size="small" value-format="YYYY-MM-DD" />
                </div>
              </div>
              <div class="date-add-controls">
                <el-radio-group v-model="dateAddOp" size="small">
                  <el-radio-button value="add">加</el-radio-button>
                  <el-radio-button value="sub">减</el-radio-button>
                </el-radio-group>
                <el-input-number v-model="dateAddNum" :min="1" :max="999" size="small" style="width: 100px" />
                <el-select v-model="dateAddUnit" size="small" style="width: 80px">
                  <el-option label="天" value="day" />
                  <el-option label="周" value="week" />
                  <el-option label="月" value="month" />
                  <el-option label="年" value="year" />
                </el-select>
              </div>
              <el-button type="primary" size="small" @click="handleDateAdd" class="date-calc-btn">计算</el-button>
              <div v-if="dateAddResult" class="date-result">
                结果：{{ dateAddResult }}
              </div>
            </div>
          </div>
        </div>

        <!-- 时间戳转换 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">时间戳转换</span>
          </div>
          <div class="card-body">
            <div class="ts-convert-mode">
              <el-radio-group v-model="tsMode" size="small">
                <el-radio-button value="toDate">时间戳 → 日期</el-radio-button>
                <el-radio-button value="toTs">日期 → 时间戳</el-radio-button>
              </el-radio-group>
            </div>

            <!-- 时间戳 → 日期 -->
            <div v-if="tsMode === 'toDate'" class="ts-to-date">
              <div class="ts-input-row">
                <el-input v-model="tsInput" placeholder="输入时间戳（秒或毫秒自动识别）" size="small" @input="handleTsToDate" />
                <el-button size="small" @click="fillCurrentTs">当前时间戳</el-button>
                <el-button size="small" @click="tsInput = ''; tsDateResult = ''">清空</el-button>
              </div>
              <div v-if="tsDateResult" class="ts-result">
                {{ tsDateResult }}
                <el-button size="small" text @click="handleCopy(tsDateResult)">复制</el-button>
              </div>
            </div>

            <!-- 日期 → 时间戳 -->
            <div v-if="tsMode === 'toTs'" class="ts-to-ts">
              <div class="ts-input-row">
                <el-date-picker
                  v-model="tsDate"
                  type="datetime"
                  placeholder="选择日期时间"
                  size="small"
                  value-format="YYYY-MM-DD HH:mm:ss"
                  style="flex: 1"
                />
              </div>
              <div v-if="tsDate" class="ts-result-row">
                <div class="ts-result-item">
                  <span class="ts-label">秒级时间戳</span>
                  <code class="ts-value">{{ tsSeconds }}</code>
                  <el-button size="small" text @click="handleCopy(String(tsSeconds))">复制</el-button>
                </div>
                <div class="ts-result-item">
                  <span class="ts-label">毫秒级时间戳</span>
                  <code class="ts-value">{{ tsMilliseconds }}</code>
                  <el-button size="small" text @click="handleCopy(String(tsMilliseconds))">复制</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import VariablePicker from '@/components/VariablePicker.vue'
import { create, all } from 'mathjs'
import { useToolboxStore } from '@/store'

const math = create(all, {})
const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('calculator')
const handleTabClick = () => {}

// ============ 表达式计算 ============
const calcInput = ref('')
const calcResult = ref<number | null>(null)
const calcError = ref('')

const handleCalculate = () => {
  const expr = calcInput.value.trim()
  if (!expr) {
    ElMessage.warning('请输入表达式')
    return
  }
  try {
    calcResult.value = math.evaluate(expr)
    calcError.value = ''

    store.addHistory({
      tool: 'calculator',
      action: '表达式计算',
      inputPreview: expr,
      outputPreview: String(calcResult.value),
      inputFull: expr,
      outputFull: String(calcResult.value),
    })
  } catch (e: any) {
    calcResult.value = null
    calcError.value = `表达式错误: ${e.message}`
  }
}

const handleCopyResult = async () => {
  if (calcResult.value === null) return
  await handleCopy(String(calcResult.value))
}

const handleVarSelect = (varName: string) => {
  calcInput.value += varName
}

const handlePaste = async () => {
  try {
    calcInput.value = await navigator.clipboard.readText()
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

// ============ 单位换算 ============
interface UnitCategory {
  id: string
  name: string
  base: string
  units: Record<string, number | ((v: number) => number)>
}

const categories: UnitCategory[] = [
  {
    id: 'length', name: '长度', base: 'm',
    units: { mm: 0.001, cm: 0.01, m: 1, km: 1000, inch: 0.0254, ft: 0.3048, yd: 0.9144, mile: 1609.344 }
  },
  {
    id: 'weight', name: '重量', base: 'kg',
    units: { mg: 0.000001, g: 0.001, kg: 1, t: 1000, oz: 0.0283495, lb: 0.453592 }
  },
  {
    id: 'temperature', name: '温度', base: '°C',
    units: {
      '°C': (v: number) => v,
      '°F': (v: number) => (v - 32) * 5 / 9,
      'K': (v: number) => v - 273.15
    }
  },
  {
    id: 'area', name: '面积', base: 'm²',
    units: { 'mm²': 0.000001, 'cm²': 0.0001, 'm²': 1, 'km²': 1000000, ha: 10000, acre: 4046.856 }
  },
  {
    id: 'volume', name: '体积', base: 'L',
    units: { mL: 0.001, L: 1, 'm³': 1000, 'gal(US)': 3.78541, 'gal(UK)': 4.54609, 'fl oz': 0.0295735 }
  },
  {
    id: 'speed', name: '速度', base: 'm/s',
    units: { 'm/s': 1, 'km/h': 0.277778, mph: 0.44704, knot: 0.514444 }
  },
  {
    id: 'timeUnits', name: '时间', base: 's',
    units: { ms: 0.001, s: 1, min: 60, h: 3600, day: 86400, week: 604800 }
  },
  {
    id: 'data', name: '数据存储', base: 'B',
    units: { B: 1, KB: 1024, MB: 1048576, GB: 1073741824, TB: 1099511627776, bit: 0.125 }
  },
  {
    id: 'angle', name: '角度', base: 'deg',
    units: { deg: 1, rad: 57.2958, grad: 0.9 }
  },
  {
    id: 'pressure', name: '压力', base: 'Pa',
    units: { Pa: 1, kPa: 1000, MPa: 1000000, bar: 100000, atm: 101325, psi: 6894.76 }
  },
  {
    id: 'energy', name: '能量', base: 'J',
    units: { J: 1, kJ: 1000, cal: 4.184, kcal: 4184, Wh: 3600, kWh: 3600000, eV: 1.602176634e-19 }
  }
]

const convertCategory = ref('length')
const convertFromValue = ref<number | null>(null)
const convertFromUnit = ref('m')
const convertToUnit = ref('km')
const convertToValue = ref('')

const currentCategory = computed(() => categories.find(c => c.id === convertCategory.value)!)
const currentUnits = computed(() => Object.keys(currentCategory.value.units))
const currentFromUnits = computed(() => currentUnits.value)
const currentToUnits = computed(() => currentUnits.value.filter(u => u !== convertFromUnit.value))

watch(convertFromUnit, () => {
  if (convertToUnit.value === convertFromUnit.value) {
    const others = currentUnits.value.filter(u => u !== convertFromUnit.value)
    if (others.length > 0) convertToUnit.value = others[0]
  }
})

const handleCategoryChange = () => {
  const units = currentUnits.value
  convertFromUnit.value = units[0] || ''
  convertToUnit.value = units.length > 1 ? units[1] : units[0] || ''
  convertToValue.value = ''
  convertFromValue.value = null
}

const handleConvert = () => {
  if (convertFromValue.value === null || convertFromValue.value === undefined || isNaN(Number(convertFromValue.value))) {
    convertToValue.value = ''
    return
  }

  const cat = currentCategory.value
  const fromVal = Number(convertFromValue.value)

  let baseVal: number
  if (convertCategory.value === 'temperature') {
    const fromFn = cat.units[convertFromUnit.value]
    baseVal = typeof fromFn === 'function' ? (fromFn as (v: number) => number)(fromVal) : fromVal * (fromFn as number)
  } else {
    const fromRatio = cat.units[convertFromUnit.value] as number
    baseVal = fromVal * fromRatio
  }

  let result: number
  if (convertCategory.value === 'temperature') {
    const toFn = cat.units[convertToUnit.value]
    if (typeof toFn === 'function') {
      const invMap: Record<string, (v: number) => number> = {
        '°C': (v: number) => v,
        '°F': (v: number) => v * 9 / 5 + 32,
        'K': (v: number) => v + 273.15
      }
      result = (invMap[convertToUnit.value] || ((v: number) => v))(baseVal)
    } else {
      result = baseVal / (toFn as number)
    }
  } else {
    const toRatio = cat.units[convertToUnit.value] as number
    result = baseVal / toRatio
  }

  convertToValue.value = formatNumber(result)
}

const handleSwap = () => {
  const tmp = convertFromUnit.value
  convertFromUnit.value = convertToUnit.value
  convertToUnit.value = tmp
  handleConvert()
}

const formatNumber = (n: number): string => {
  if (Math.abs(n) < 0.000001 || Math.abs(n) > 999999999) {
    return n.toExponential(6)
  }
  return Number(n.toPrecision(10)).toString()
}

handleCategoryChange()

// ============ 日期计算 ============
const dateMode = ref('diff')
const dateStart = ref('')
const dateEnd = ref('')
const dateDiffResult = ref<{ days: number; months: number; weeks: number } | null>(null)

const dateAddStart = ref('')
const dateAddOp = ref('add')
const dateAddNum = ref(1)
const dateAddUnit = ref('day')
const dateAddResult = ref('')

const handleDateDiff = () => {
  if (!dateStart.value || !dateEnd.value) {
    ElMessage.warning('请选择起始和结束日期')
    return
  }
  const d1 = new Date(dateStart.value)
  const d2 = new Date(dateEnd.value)
  const diffMs = Math.abs(d2.getTime() - d1.getTime())
  const days = Math.round(diffMs / (24 * 60 * 60 * 1000))
  const months = Math.round(days / 30.44)
  const weeks = Math.round(days / 7)
  dateDiffResult.value = { days, months, weeks }

  store.addHistory({
    tool: 'calculator',
    action: '日期差计算',
    inputPreview: `${dateStart.value} ~ ${dateEnd.value}`,
    outputPreview: `${days} 天`,
    inputFull: `${dateStart.value} ~ ${dateEnd.value}`,
    outputFull: `${days} 天 (${weeks} 周, ${months} 月)`,
  })
}

const handleDateAdd = () => {
  if (!dateAddStart.value) {
    ElMessage.warning('请选择起始日期')
    return
  }
  const d = new Date(dateAddStart.value)
  const num = dateAddOp.value === 'sub' ? -dateAddNum.value : dateAddNum.value

  switch (dateAddUnit.value) {
    case 'day': d.setDate(d.getDate() + num); break
    case 'week': d.setDate(d.getDate() + num * 7); break
    case 'month': d.setMonth(d.getMonth() + num); break
    case 'year': d.setFullYear(d.getFullYear() + num); break
  }

  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  dateAddResult.value = `${y}-${m}-${day}`

  store.addHistory({
    tool: 'calculator',
    action: '日期加减',
    inputPreview: `${dateAddStart.value} ${dateAddOp.value} ${dateAddNum.value}${dateAddUnit.value}`,
    outputPreview: dateAddResult.value,
    inputFull: `${dateAddStart.value} ${dateAddOp.value} ${dateAddNum.value}${dateAddUnit.value}`,
    outputFull: dateAddResult.value,
  })
}

// ============ 时间戳转换 ============
const tsMode = ref('toDate')
const tsInput = ref('')
const tsDateResult = ref('')
const tsDate = ref('')
const tsSeconds = ref(0)
const tsMilliseconds = ref(0)

const fillCurrentTs = () => {
  tsInput.value = String(Date.now())
  handleTsToDate()
}

const handleTsToDate = () => {
  if (!tsInput.value.trim()) {
    tsDateResult.value = ''
    return
  }
  const ts = Number(tsInput.value.trim())
  if (isNaN(ts)) {
    tsDateResult.value = '无效的时间戳'
    return
  }
  const d = ts > 1e12 ? new Date(ts) : new Date(ts * 1000)
  if (isNaN(d.getTime())) {
    tsDateResult.value = '时间戳超出有效范围'
    return
  }
  tsDateResult.value = formatDate(d)
}

watch(tsDate, (val) => {
  if (val) {
    const d = new Date(val)
    tsSeconds.value = Math.floor(d.getTime() / 1000)
    tsMilliseconds.value = d.getTime()
  }
})

const formatDate = (d: Date): string => {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const min = String(d.getMinutes()).padStart(2, '0')
  const s = String(d.getSeconds()).padStart(2, '0')
  return `${y}-${m}-${day} ${h}:${min}:${s}`
}

// ============ 通用方法 ============
const handleCopy = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ============ 300ms 防抖自动执行 ============
let calcTimer: ReturnType<typeof setTimeout> | null = null
watch(calcInput, (val) => {
  if (!val.trim()) {
    calcResult.value = null
    calcError.value = ''
    return
  }
  if (calcTimer) clearTimeout(calcTimer)
  calcTimer = setTimeout(() => {
    handleCalculate()
  }, 300)
})
</script>

<style scoped>
.calculator-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .calculator-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.calculator-tabs :deep(.el-tabs__nav-wrap) { padding-left: 4px; }
.calculator-tabs :deep(.el-tabs__item) { color: var(--text-secondary); font-size: 14px; font-weight: 500; }
.calculator-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.calculator-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.calculator-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.tool-card:hover { border-color: rgba(0, 212, 255, 0.3); }

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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

.header-left { display: flex; align-items: center; gap: 8px; }
.card-actions { display: flex; align-items: center; gap: 6px; }
.card-body { padding: 16px 20px; }

.hint-icon { font-size: 15px; color: var(--text-secondary); cursor: pointer; }
.hint-icon:hover { color: var(--accent-cyan); }

.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.expr-input-row { display: flex; gap: 8px; align-items: center; }
.expr-prefix { font-family: 'JetBrains Mono', monospace; color: var(--accent-cyan); font-weight: 700; }
.calc-btn { flex-shrink: 0; }

.result-placeholder { color: var(--text-muted); font-size: 13px; text-align: center; padding: 12px; }
.result-display { padding: 12px 16px; background: var(--bg-input); border-radius: 6px; border: 1px solid var(--border-color); }
.result-equal { font-size: 13px; color: var(--text-secondary); }
.result-value { font-family: 'JetBrains Mono', monospace; font-size: 20px; font-weight: 700; color: var(--accent-cyan); }

.error-message {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
}

.convert-config { margin-bottom: 12px; }
.convert-row { display: flex; gap: 12px; align-items: center; }
.convert-side { flex: 1; display: flex; flex-direction: column; gap: 8px; }

.swap-btn {
  width: 36px; height: 36px;
  border: 1px solid var(--border-color);
  border-radius: 50%;
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;
}
.swap-btn:hover { border-color: var(--accent-cyan); color: var(--accent-cyan); }

.date-mode-switch { margin-bottom: 12px; }
.date-picker-row { display: flex; gap: 12px; margin-bottom: 12px; }
.date-picker-item { display: flex; align-items: center; gap: 8px; }
.date-label { font-size: 13px; color: var(--text-secondary); min-width: 40px; }
.date-calc-btn { margin-bottom: 12px; }
.date-result { padding: 8px 12px; background: var(--bg-input); border-radius: 6px; font-size: 13px; color: var(--accent-cyan); border: 1px solid var(--border-color); }
.date-add-controls { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }

.ts-convert-mode { margin-bottom: 12px; }
.ts-input-row { display: flex; gap: 8px; margin-bottom: 12px; }
.ts-result { padding: 8px 12px; background: var(--bg-input); border-radius: 6px; font-family: 'JetBrains Mono', monospace; font-size: 14px; color: var(--accent-cyan); border: 1px solid var(--border-color); display: flex; align-items: center; justify-content: space-between; }
.ts-result-row { display: flex; flex-direction: column; gap: 8px; }
.ts-result-item { display: flex; align-items: center; gap: 8px; padding: 8px 12px; background: var(--bg-input); border-radius: 6px; border: 1px solid var(--border-color); }
.ts-label { font-size: 12px; color: var(--text-muted); min-width: 80px; }
.ts-value { flex: 1; font-family: 'JetBrains Mono', monospace; font-size: 13px; color: var(--text-primary); }

:deep(.error .el-textarea__inner) { border-color: var(--accent-red); box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1); }
</style>
