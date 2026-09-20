<template>
  <div class="tool-container">
    <!-- 权限矩阵 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">权限设置</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>R = 读取 (4)</p>
                <p>W = 写入 (2)</p>
                <p>X = 执行 (1)</p>
                <p>每组权限 = 三者之和</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="resetAll">重置</el-button>
          <el-button size="small" type="primary" @click="setAll('777')">777</el-button>
          <el-button size="small" type="success" @click="setAll('755')">755</el-button>
          <el-button size="small" type="warning" @click="setAll('644')">644</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="perm-matrix">
          <div class="perm-row perm-header">
            <div class="perm-cell perm-label">权限</div>
            <div class="perm-cell perm-group">所有者 (Owner)</div>
            <div class="perm-cell perm-group">组 (Group)</div>
            <div class="perm-cell perm-group">其他 (Other)</div>
          </div>
          <div class="perm-row">
            <div class="perm-cell perm-label">
              <span class="perm-badge perm-read">R</span>
              <span class="perm-weight">4</span>
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="owner.r" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="group.r" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="other.r" />
            </div>
          </div>
          <div class="perm-row">
            <div class="perm-cell perm-label">
              <span class="perm-badge perm-write">W</span>
              <span class="perm-weight">2</span>
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="owner.w" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="group.w" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="other.w" />
            </div>
          </div>
          <div class="perm-row">
            <div class="perm-cell perm-label">
              <span class="perm-badge perm-exec">X</span>
              <span class="perm-weight">1</span>
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="owner.x" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="group.x" />
            </div>
            <div class="perm-cell">
              <el-checkbox v-model="other.x" />
            </div>
          </div>
          <div class="perm-row perm-subtotal">
            <div class="perm-cell perm-label">小计</div>
            <div class="perm-cell">
              <span class="perm-num">{{ ownerValue }}</span>
            </div>
            <div class="perm-cell">
              <span class="perm-num">{{ groupValue }}</span>
            </div>
            <div class="perm-cell">
              <span class="perm-num">{{ otherValue }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 结果展示 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">结果</span>
      </div>
      <div class="card-body">
        <div class="result-grid">
          <div class="result-item">
            <span class="result-label">数字权限</span>
            <div class="result-value-row">
              <el-input
                v-model="numericInput"
                class="numeric-input"
                placeholder="如 755"
                maxlength="3"
                @input="handleNumericInput"
              />
              <el-button size="small" @click="copyResult(numeric)">复制</el-button>
            </div>
          </div>
          <div class="result-item">
            <span class="result-label">符号权限</span>
            <div class="result-value-row">
              <span class="symbol-text">{{ rwxString }}</span>
              <el-button size="small" @click="copyResult(rwxString)">复制</el-button>
            </div>
          </div>
          <div class="result-item result-command">
            <span class="result-label">chmod 命令</span>
            <div class="result-value-row">
              <code class="command-text">chmod {{ numeric }} file</code>
              <el-button size="small" @click="copyResult(`chmod ${numeric} file`)">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 常见权限参考 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">常见权限参考</span>
      </div>
      <div class="card-body">
        <div class="ref-grid">
          <div v-for="ref in refs" :key="ref.numeric" class="ref-item" @click="setAll(ref.numeric)">
            <div class="ref-numeric">{{ ref.numeric }}</div>
            <div class="ref-rwx">{{ ref.rwx }}</div>
            <div class="ref-desc">{{ ref.desc }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

interface PermGroup {
  r: boolean
  w: boolean
  x: boolean
}

const owner = reactive<PermGroup>({ r: true, w: true, x: true })
const group = reactive<PermGroup>({ r: true, w: false, x: true })
const other = reactive<PermGroup>({ r: true, w: false, x: true })

const numericInput = ref('755')

const groupValue = (g: PermGroup): number => (g.r ? 4 : 0) + (g.w ? 2 : 0) + (g.x ? 1 : 0)

const ownerValue = computed(() => groupValue(owner))
const groupValueC = computed(() => groupValue(group))
const otherValue = computed(() => groupValue(other))

const numeric = computed(() => `${ownerValue.value}${groupValueC.value}${otherValue.value}`)

const groupRwx = (g: PermGroup): string => (g.r ? 'r' : '-') + (g.w ? 'w' : '-') + (g.x ? 'x' : '-')
const rwxString = computed(() => `${groupRwx(owner)}${groupRwx(group)}${groupRwx(other)}`)

// 监听 checkboxes 变化，同步数字输入框
watch([owner, group, other], () => {
  numericInput.value = numeric.value
  addHistory()
}, { deep: true })

const handleNumericInput = () => {
  const val = numericInput.value.replace(/[^0-7]/g, '').slice(0, 3)
  numericInput.value = val
  if (val.length === 3) {
    applyNumeric(val)
  }
}

const applyNumeric = (num: string) => {
  const digits = num.split('').map(Number)
  if (digits.length !== 3 || digits.some(d => isNaN(d) || d > 7)) return
  const applyGroup = (g: PermGroup, d: number) => {
    g.r = !!(d & 4)
    g.w = !!(d & 2)
    g.x = !!(d & 1)
  }
  applyGroup(owner, digits[0])
  applyGroup(group, digits[1])
  applyGroup(other, digits[2])
}

const setAll = (num: string) => {
  applyNumeric(num)
  numericInput.value = num
}

const resetAll = () => setAll('755')

const refs = [
  { numeric: '777', rwx: 'rwxrwxrwx', desc: '所有人可读写执行' },
  { numeric: '755', rwx: 'rwxr-xr-x', desc: '所有者可读写执行，其他只读执行（常用目录/脚本）' },
  { numeric: '644', rwx: 'rw-r--r--', desc: '所有者可读写，其他只读（常用文件）' },
  { numeric: '700', rwx: 'rwx------', desc: '仅所有者可读写执行' },
  { numeric: '600', rwx: 'rw-------', desc: '仅所有者可读写（私钥/敏感文件）' },
  { numeric: '711', rwx: 'rwx--x--x', desc: '所有者可读写执行，其他仅执行' },
]

const copyResult = async (text: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success(`已复制: ${text}`)
  } catch {
    ElMessage.error('复制失败')
  }
}

let lastHistory = ''
const addHistory = () => {
  const sig = `${numeric.value}|${rwxString.value}`
  if (sig === lastHistory) return
  lastHistory = sig
  store.addHistory({
    tool: 'chmod',
    action: '权限计算',
    inputPreview: rwxString.value,
    outputPreview: numeric.value,
    inputFull: rwxString.value,
    outputFull: `chmod ${numeric.value}  # ${rwxString.value}`,
  })
}
</script>

<style scoped>
.perm-matrix {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.perm-row {
  display: grid;
  grid-template-columns: 120px 1fr 1fr 1fr;
  gap: 8px;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--border-color);
}

.perm-row:last-child {
  border-bottom: none;
}

.perm-row.perm-header {
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 13px;
}

.perm-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.perm-label {
  justify-content: flex-start;
  padding-left: 8px;
  gap: 8px;
}

.perm-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  font-weight: 700;
  font-size: 13px;
  color: #fff;
}

.perm-read { background: #10b981; }
.perm-write { background: #f59e0b; }
.perm-exec { background: #8b5cf6; }

.perm-weight {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}

.perm-subtotal .perm-cell {
  font-weight: 600;
}

.perm-num {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  color: var(--accent-cyan);
  font-weight: 700;
}

.result-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.result-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.result-label {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.result-value-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.numeric-input {
  max-width: 200px;
}

:deep(.numeric-input .el-input__inner) {
  font-family: 'JetBrains Mono', monospace;
  font-size: 20px;
  font-weight: 700;
  color: var(--accent-cyan);
  text-align: center;
}

.symbol-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 2px;
  padding: 8px 16px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.command-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 15px;
  padding: 8px 16px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  color: var(--accent-cyan);
}

.ref-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
}

.ref-item {
  padding: 12px 16px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.ref-item:hover {
  border-color: var(--accent-cyan);
  transform: translateY(-1px);
}

.ref-numeric {
  font-family: 'JetBrains Mono', monospace;
  font-size: 22px;
  font-weight: 700;
  color: var(--accent-cyan);
}

.ref-rwx {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.ref-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 6px;
  line-height: 1.4;
}
</style>
