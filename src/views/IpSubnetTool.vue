<template>
  <div class="tool-container">
    <!-- Tab 栏（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="ip-subnet-tabs">
        <el-tab-pane label="子网计算" name="calc" />
        <el-tab-pane label="子网划分" name="divide" />
        <el-tab-pane label="IP 范围合并" name="range" />
        <el-tab-pane label="IP↔整数" name="int" />
      </el-tabs>
    </div>

    <!-- Tab 1: 子网计算 -->
    <div v-if="activeTab === 'calc'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持两种格式：</p>
                <p><code>192.168.1.0/24</code>（CIDR）</p>
                <p><code>192.168.1.0 255.255.255.0</code>（IP+掩码）</p>
                <p>仅 IP 默认按 /32 单主机计算</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <VariablePicker @select="insertVariable($event, 'calc')" />
          <el-button size="small" @click="calcInput = ''">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="calcInput"
          placeholder="192.168.1.0/24"
          size="default"
          clearable
        />
      </div>
    </div>

    <div v-if="activeTab === 'calc' && calcResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">计算结果</span>
        <el-button size="small" @click="copyAll(calcResultText)">复制全部</el-button>
      </div>
      <div class="card-body">
        <div class="result-grid">
          <div v-for="item in calcResultItems" :key="item.label" class="result-item">
            <span class="result-label">{{ item.label }}</span>
            <code class="result-value" :class="{ mono: item.mono }">{{ item.value }}</code>
            <el-button size="small" text @click="copy(item.value)">复制</el-button>
          </div>
        </div>
        <!-- 二进制可视化 -->
        <div class="binary-block">
          <div class="binary-row">
            <span class="binary-label">IP  </span>
            <span class="binary-str">
              <span v-for="(b, i) in calcResult.binaryIp.split('')" :key="'ip'+i"
                :class="['bit', i < calcResult.cidr ? 'bit-net' : 'bit-host']">{{ b }}</span>
            </span>
          </div>
          <div class="binary-row">
            <span class="binary-label">掩码</span>
            <span class="binary-str">
              <span v-for="(b, i) in calcResult.binaryMask.split('')" :key="'mk'+i"
                :class="['bit', i < calcResult.cidr ? 'bit-net' : 'bit-host']">{{ b }}</span>
            </span>
          </div>
          <div class="binary-legend">
            <span class="legend-item"><i class="legend-dot bit-net"></i>网络位 ({{ calcResult.cidr }})</span>
            <span class="legend-item"><i class="legend-dot bit-host"></i>主机位 ({{ 32 - calcResult.cidr }})</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 2: 子网划分 -->
    <div v-if="activeTab === 'divide'" class="tool-card">
      <div class="card-header">
        <span class="card-title">划分参数</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-input v-model="divideInput" placeholder="原网络 192.168.1.0/24" style="width: 240px" />
          </div>
          <div class="action-group">
            <span class="group-label">方式</span>
            <el-radio-group v-model="divideMode" size="small">
              <el-radio-button label="count">按子网数量</el-radio-button>
              <el-radio-button label="hosts">按主机数量</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <el-input-number v-model="divideValue" :min="1" :max="1000000" size="small" style="width: 130px" />
            <el-button type="primary" size="small" @click="handleDivide">计算</el-button>
          </div>
        </div>
        <div v-if="divideError" class="error-message">{{ divideError }}</div>
      </div>
    </div>

    <div v-if="activeTab === 'divide' && divideResult.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">划分结果（{{ divideResult.length }} 个子网）</span>
        <el-button size="small" @click="copyTable(divideResult, ['network','cidr','firstHost','lastHost','broadcast','hostCount'])">复制全部</el-button>
      </div>
      <div class="card-body">
        <DataTable :data="divideResult" max-height="500">
          <el-table-column prop="index" label="#" width="50" />
          <el-table-column prop="network" label="网络地址" min-width="130" />
          <el-table-column prop="cidr" label="前缀" width="70" />
          <el-table-column prop="firstHost" label="首主机" min-width="130" />
          <el-table-column prop="lastHost" label="末主机" min-width="130" />
          <el-table-column prop="broadcast" label="广播地址" min-width="130" />
          <el-table-column prop="hostCount" label="可用主机" width="100" />
        </DataTable>
      </div>
    </div>

    <!-- Tab 3: IP 范围合并 -->
    <div v-if="activeTab === 'range'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">IP 范围</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>输入起止 IP，自动计算覆盖该范围的最少 CIDR 块</p>
                <p>例：<code>192.168.1.1</code> ~ <code>192.168.1.254</code></p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-input v-model="rangeStart" placeholder="起始 IP" style="width: 180px" />
            <span class="group-label">~</span>
            <el-input v-model="rangeEnd" placeholder="结束 IP" style="width: 180px" />
            <el-button type="primary" size="small" @click="handleRange">计算</el-button>
          </div>
        </div>
        <div v-if="rangeError" class="error-message">{{ rangeError }}</div>
      </div>
    </div>

    <div v-if="activeTab === 'range' && rangeResult.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">CIDR 块（{{ rangeResult.length }} 个）</span>
        <el-button size="small" @click="copyTable(rangeResult, ['cidr','network','broadcast','hostCount'])">复制全部</el-button>
      </div>
      <div class="card-body">
        <DataTable :data="rangeResult" max-height="500">
          <el-table-column type="index" label="#" width="50" />
          <el-table-column prop="cidr" label="CIDR" min-width="180" />
          <el-table-column prop="network" label="网络地址" min-width="130" />
          <el-table-column prop="broadcast" label="广播地址" min-width="130" />
          <el-table-column prop="hostCount" label="可用主机" width="100" />
        </DataTable>
      </div>
    </div>

    <!-- Tab 4: IP ↔ 整数 -->
    <div v-if="activeTab === 'int'" class="tool-card">
      <div class="card-header">
        <span class="card-title">输入</span>
        <div class="card-actions">
          <el-button size="small" @click="intInput = ''">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="intInput"
          placeholder="输入 IPv4 地址或整数（0~4294967295）"
          size="default"
          clearable
        />
      </div>
    </div>

    <div v-if="activeTab === 'int' && intResult" class="tool-card">
      <div class="card-header">
        <span class="card-title">转换结果</span>
        <el-button size="small" @click="copyAll(intResultText)">复制全部</el-button>
      </div>
      <div class="card-body">
        <div class="result-grid">
          <div v-for="item in intResultItems" :key="item.label" class="result-item">
            <span class="result-label">{{ item.label }}</span>
            <code class="result-value mono">{{ item.value }}</code>
            <el-button size="small" text @click="copy(item.value)">复制</el-button>
          </div>
        </div>
        <div v-if="intResult.binary" class="binary-block">
          <div class="binary-row">
            <span class="binary-label">二进制</span>
            <span class="binary-str">
              <span v-for="(b, i) in intResult.binary.split('')" :key="i"
                :class="['bit', i < 8 ? 'bit-net' : i < 16 ? 'bit-host' : i < 24 ? 'bit-net' : 'bit-host']">{{ b }}</span>
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import VariablePicker from '@/components/VariablePicker.vue'
import DataTable from '@/components/DataTable.vue'
import {
  calcSubnet, divideSubnets, ipRangeToCidrs,
  ipToInt, intToIp, intToBinary,
  type SubnetInfo, type SubnetDivision, type CidrRange,
} from '@/utils/ipSubnetUtils'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('calc')

// 还原期间禁止写历史（避免从历史跳转回来时自动计算产生重复记录）
let isRestoring = false
let restoreTimer: ReturnType<typeof setTimeout> | null = null
const blockHistory = () => {
  isRestoring = true
  if (restoreTimer) clearTimeout(restoreTimer)
  restoreTimer = setTimeout(() => { isRestoring = false }, 500)
}

// ============ Tab 1: 子网计算 ============
const calcInput = ref('')
const calcResult = ref<SubnetInfo | null>(null)
const calcError = ref('')

const calcResultItems = computed(() => {
  if (!calcResult.value) return []
  const s = calcResult.value
  return [
    { label: '网络地址', value: s.networkAddress, mono: true },
    { label: '广播地址', value: s.broadcastAddress, mono: true },
    { label: '子网掩码', value: s.mask, mono: true },
    { label: '反掩码', value: s.wildcard, mono: true },
    { label: '首主机', value: s.firstHost, mono: true },
    { label: '末主机', value: s.lastHost, mono: true },
    { label: '可用主机数', value: s.hostCount.toLocaleString(), mono: true },
    { label: 'CIDR', value: `/${s.cidr}`, mono: true },
    { label: 'IP 类别', value: s.ipClass, mono: false },
    { label: '私网地址', value: s.isPrivate ? '是' : '否', mono: false },
    { label: '环回地址', value: s.isLoopback ? '是' : '否', mono: false },
  ]
})

const calcResultText = computed(() => {
  if (!calcResult.value) return ''
  const s = calcResult.value
  return `IP: ${s.ip}/${s.cidr}
网络地址: ${s.networkAddress}
广播地址: ${s.broadcastAddress}
子网掩码: ${s.mask}
反掩码: ${s.wildcard}
首主机: ${s.firstHost}
末主机: ${s.lastHost}
可用主机数: ${s.hostCount}
IP类别: ${s.ipClass}
私网: ${s.isPrivate ? '是' : '否'}
环回: ${s.isLoopback ? '是' : '否'}`
})

const runCalc = () => {
  if (!calcInput.value.trim()) {
    calcResult.value = null
    calcError.value = ''
    return
  }
  try {
    calcResult.value = calcSubnet(calcInput.value)
    calcError.value = ''
    if (!isRestoring) {
      store.addHistory({
        tool: 'ipSubnet',
        action: '子网计算',
        inputPreview: calcInput.value.slice(0, 50),
        outputPreview: `${calcResult.value.networkAddress}/${calcResult.value.cidr} · 主机${calcResult.value.hostCount}`,
        inputFull: calcInput.value,
        outputFull: calcResultText.value,
      })
    }
  } catch (e: any) {
    calcResult.value = null
    calcError.value = e.message
  }
}

watch(calcInput, () => runCalc())

// ============ Tab 2: 子网划分 ============
const divideInput = ref('')
const divideMode = ref<'count' | 'hosts'>('count')
const divideValue = ref(4)
const divideResult = ref<SubnetDivision[]>([])
const divideError = ref('')

const handleDivide = () => {
  if (!divideInput.value.trim()) {
    ElMessage.warning('请输入原网络')
    return
  }
  try {
    divideResult.value = divideSubnets(divideInput.value, divideMode.value, divideValue.value)
    divideError.value = ''
    store.addHistory({
      tool: 'ipSubnet',
      action: '子网划分',
      inputPreview: `${divideInput.value} · ${divideMode.value === 'count' ? divideValue.value + '个子网' : '每子网' + divideValue.value + '主机'}`,
      outputPreview: `${divideResult.value.length} 个子网`,
      inputFull: `${divideInput.value} | ${divideMode.value} | ${divideValue.value}`,
      outputFull: divideResult.value.map(d => `${d.network}${d.cidr}`).join('\n'),
    })
  } catch (e: any) {
    divideResult.value = []
    divideError.value = e.message
  }
}

// ============ Tab 3: IP 范围合并 ============
const rangeStart = ref('')
const rangeEnd = ref('')
const rangeResult = ref<CidrRange[]>([])
const rangeError = ref('')

const handleRange = () => {
  if (!rangeStart.value.trim() || !rangeEnd.value.trim()) {
    ElMessage.warning('请输入起止 IP')
    return
  }
  try {
    rangeResult.value = ipRangeToCidrs(rangeStart.value, rangeEnd.value)
    rangeError.value = ''
    store.addHistory({
      tool: 'ipSubnet',
      action: 'IP范围合并',
      inputPreview: `${rangeStart.value} ~ ${rangeEnd.value}`,
      outputPreview: `${rangeResult.value.length} 个 CIDR 块`,
      inputFull: `${rangeStart.value} ~ ${rangeEnd.value}`,
      outputFull: rangeResult.value.map(r => r.cidr).join('\n'),
    })
  } catch (e: any) {
    rangeResult.value = []
    rangeError.value = e.message
  }
}

// ============ Tab 4: IP ↔ 整数 ============
const intInput = ref('')
const intResult = ref<{ ip: string; int: string; binary: string; hex: string } | null>(null)
const intError = ref('')

const intResultItems = computed(() => {
  if (!intResult.value) return []
  return [
    { label: 'IPv4 地址', value: intResult.value.ip },
    { label: '十进制整数', value: intResult.value.int },
    { label: '十六进制', value: intResult.value.hex },
  ]
})

const intResultText = computed(() => {
  if (!intResult.value) return ''
  return `IPv4: ${intResult.value.ip}
整数: ${intResult.value.int}
十六进制: ${intResult.value.hex}
二进制: ${intResult.value.binary}`
})

const runInt = () => {
  const v = intInput.value.trim()
  if (!v) {
    intResult.value = null
    intError.value = ''
    return
  }
  try {
    let ip: string
    // 判断输入是 IP 还是整数
    if (v.includes('.')) {
      ip = v
      const int = ipToInt(v)
      intResult.value = { ip: v, int: int.toString(), binary: intToBinary(int), hex: '0x' + int.toString(16).toUpperCase().padStart(8, '0') }
    } else {
      const int = Number(v)
      ip = intToIp(int)
      intResult.value = { ip, int: int.toString(), binary: intToBinary(int), hex: '0x' + int.toString(16).toUpperCase().padStart(8, '0') }
    }
    intError.value = ''
    if (!isRestoring) {
      store.addHistory({
        tool: 'ipSubnet',
        action: 'IP↔整数',
        inputPreview: v.slice(0, 50),
        outputPreview: intResult.value.ip,
        inputFull: v,
        outputFull: intResultText.value,
      })
    }
  } catch (e: any) {
    intResult.value = null
    intError.value = e.message
  }
}

watch(intInput, () => runInt())

// ============ 历史还原 ============
watch(() => store.pendingHistoryRestore, (restore) => {
  if (!restore || restore.tool !== 'ipSubnet') return
  blockHistory()
  const action = restore.action
  if (action === '子网计算') {
    activeTab.value = 'calc'
    calcInput.value = restore.input || ''
  } else if (action === '子网划分') {
    activeTab.value = 'divide'
    // input 格式: "原网络 | count | 4"
    const parts = (restore.input || '').split(' | ')
    divideInput.value = parts[0] || ''
    if (parts[1]) divideMode.value = parts[1] as 'count' | 'hosts'
    if (parts[2]) divideValue.value = Number(parts[2]) || 4
    if (divideInput.value) handleDivide()
  } else if (action === 'IP范围合并') {
    activeTab.value = 'range'
    const parts = (restore.input || '').split(' ~ ')
    rangeStart.value = parts[0] || ''
    rangeEnd.value = parts[1] || ''
    if (rangeStart.value && rangeEnd.value) handleRange()
  } else if (action === 'IP↔整数') {
    activeTab.value = 'int'
    intInput.value = restore.input || ''
  }
  store.clearHistoryRestore()
})

// ============ 通用方法 ============
const copy = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const copyAll = async (text: string) => {
  if (!text) return
  await copy(text)
}

const copyTable = async (rows: any[], fields: string[]) => {
  if (!rows.length) return
  const header = fields.join('\t')
  const body = rows.map(r => fields.map(f => r[f]).join('\t')).join('\n')
  await copy(header + '\n' + body)
}

const insertVariable = (varName: string, tab: string) => {
  if (tab === 'calc') {
    calcInput.value += `{{${varName}}}`
  }
}
</script>

<style scoped>
/* ===== Tab 样式 ===== */
.ip-subnet-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
html.light .ip-subnet-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.ip-subnet-tabs :deep(.el-tabs__nav-wrap) { padding-left: 4px; }
.ip-subnet-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}
.ip-subnet-tabs :deep(.el-tabs__item.is-active) { color: var(--accent-cyan); }
.ip-subnet-tabs :deep(.el-tabs__active-bar) { background-color: var(--accent-cyan); }
.ip-subnet-tabs :deep(.el-tabs__nav-wrap::after) { background-color: var(--border-color); }

/* ===== 结果网格 ===== */
.result-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  transition: border-color 0.3s;
}
.result-item:hover { border-color: var(--accent-cyan); }
.result-label {
  font-size: 12px;
  color: var(--accent-cyan);
  font-weight: 600;
  min-width: 90px;
  text-transform: uppercase;
  letter-spacing: 1px;
}
.result-value {
  flex: 1;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
}
.result-value.mono {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* ===== 二进制可视化 ===== */
.binary-block {
  margin-top: 16px;
  padding: 12px 14px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
}
.binary-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}
.binary-row:last-of-type { margin-bottom: 12px; }
.binary-label {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 40px;
}
.binary-str {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  letter-spacing: 1px;
  display: flex;
  flex-wrap: wrap;
}
.binary-str .bit:nth-child(8n) { margin-right: 6px; }
.bit {
  display: inline-block;
  width: 12px;
  text-align: center;
}
.bit-net { color: var(--accent-cyan); }
.bit-host { color: var(--text-muted); }
.binary-legend {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}
.legend-item { display: flex; align-items: center; gap: 6px; }
.legend-dot {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 2px;
}
.legend-dot.bit-net { background: var(--accent-cyan); }
.legend-dot.bit-host { background: var(--text-muted); }
</style>
