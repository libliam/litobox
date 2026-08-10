<template>
  <div class="tool-container">
    <!-- 顶部操作卡片 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Git 仓库统计</span>
          <el-tooltip placement="bottom" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 选择本地 Git 仓库目录，分析提交历史</p>
                <p>• 统计贡献者排行、每日提交趋势、文件改动</p>
                <p>• 需要系统已安装 git（git --version 可验证）</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" :loading="analyzing" @click="pickRepo">选择仓库</el-button>
          <el-button v-if="repoPath" size="small" :loading="analyzing" @click="analyze">重新分析</el-button>
        </div>
      </div>
      <div class="card-body">
        <template v-if="repoInfo.valid">
          <div class="repo-path">{{ repoInfo.root }}</div>
          <div class="repo-meta">
            <span v-if="repoInfo.branch" class="branch-tag">分支 {{ repoInfo.branch }}</span>
            <span v-if="repoInfo.last_commit" class="last-commit">{{ repoInfo.last_commit }}</span>
          </div>
        </template>
        <template v-else>
          <div class="empty-tip" v-if="gitAvailable">选择一个本地 Git 仓库目录开始统计</div>
          <div class="error-box" v-else>
            <div class="error-title">未检测到 git</div>
            <div class="error-msg">请先安装 Git（https://git-scm.com/）并确保已加入 PATH，然后重启应用。</div>
          </div>
        </template>
      </div>
    </div>

    <!-- 统计数字卡片 -->
    <div v-if="stats" class="stat-grid">
      <div class="tool-card stat-card">
        <div class="stat-value">{{ formatNumber(stats.total_commits) }}</div>
        <div class="stat-label">总提交数</div>
      </div>
      <div class="tool-card stat-card">
        <div class="stat-value add-color">+{{ formatNumber(stats.total_insertions) }}</div>
        <div class="stat-label">新增行数</div>
      </div>
      <div class="tool-card stat-card">
        <div class="stat-value del-color">-{{ formatNumber(stats.total_deletions) }}</div>
        <div class="stat-label">删除行数</div>
      </div>
      <div class="tool-card stat-card">
        <div class="stat-value">{{ stats.authors.length }}</div>
        <div class="stat-label">贡献者</div>
      </div>
    </div>

    <!-- 提交趋势 -->
    <div v-if="stats" class="tool-card">
      <div class="card-header">
        <span class="card-title">提交趋势{{ trendDays === '0' ? '（全部）' : `（最近 ${trendDays} 天）` }}</span>
        <div class="card-actions">
          <el-radio-group v-model="trendDays" size="small">
            <el-radio-button value="30">30天</el-radio-button>
            <el-radio-button value="90">90天</el-radio-button>
            <el-radio-button value="365">365天</el-radio-button>
            <el-radio-button value="0">全部</el-radio-button>
          </el-radio-group>
        </div>
      </div>
      <div class="card-body">
        <div v-if="trendData.length === 0" class="empty-tip">所选时间范围内无提交</div>
        <svg v-else class="trend-chart" viewBox="0 0 920 250">
          <defs>
            <linearGradient id="trendGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#00d4ff" stop-opacity="0.9" />
              <stop offset="100%" stop-color="#00d4ff" stop-opacity="0.25" />
            </linearGradient>
          </defs>
          <!-- 参考线 -->
          <line v-for="g in gridLines" :key="g.y" :x1="padL" :y1="g.y" :x2="padL + plotW" :y2="g.y" class="grid-line" />
          <text v-for="g in gridLines" :key="'t' + g.y" x="6" :y="g.y - 4" class="grid-label">{{ g.label }}</text>
          <!-- 柱状图 -->
          <g>
            <rect
              v-for="(d, i) in trendData"
              :key="d.date"
              :x="barX(i)"
              :y="barY(d.commits)"
              :width="barWidth"
              :height="barH(d.commits)"
              :rx="barWidth > 3 ? 2 : 0"
              fill="url(#trendGrad)"
              class="trend-bar"
            >
              <title>{{ d.date }}：{{ d.commits }} 次提交（+{{ d.insertions }}/-{{ d.deletions }}）</title>
            </rect>
          </g>
          <!-- 日期标签（首/中/尾，对齐防止溢出裁剪） -->
          <text :x="padL" :y="chartH + 26" text-anchor="start" class="axis-label">{{ trendData[0]?.date }}</text>
          <text :x="padL + plotW / 2" :y="chartH + 26" text-anchor="middle" class="axis-label">{{ trendData[Math.floor(trendData.length / 2)]?.date }}</text>
          <text :x="padL + plotW" :y="chartH + 26" text-anchor="end" class="axis-label">{{ trendData[trendData.length - 1]?.date }}</text>
        </svg>
        <div class="chart-legend">
          <span>柱高 = 当日提交数，悬停查看明细</span>
        </div>
      </div>
    </div>

    <!-- 贡献者排行 -->
    <div v-if="stats" class="tool-card">
      <div class="card-header">
        <span class="card-title">贡献者排行</span>
      </div>
      <div class="card-body">
        <DataTable :data="stats.authors" :max-height="320" class="stats-table">
          <el-table-column label="#" type="index" width="44" />
          <el-table-column prop="name" label="作者" min-width="140" show-overflow-tooltip />
          <el-table-column label="提交数" width="220">
            <template #default="{ row }">
              <div class="bar-cell">
                <div class="bar-track">
                  <div class="bar-fill" :style="{ width: pct(row.commits, maxAuthorCommits) + '%' }" />
                </div>
                <span class="bar-num">{{ formatNumber(row.commits) }}</span>
              </div>
            </template>
          </el-table-column>
          <el-table-column prop="insertions" label="新增" width="90" align="right">
            <template #default="{ row }"><span class="add-color">+{{ formatNumber(row.insertions) }}</span></template>
          </el-table-column>
          <el-table-column prop="deletions" label="删除" width="90" align="right">
            <template #default="{ row }"><span class="del-color">-{{ formatNumber(row.deletions) }}</span></template>
          </el-table-column>
        </DataTable>
      </div>
    </div>

    <!-- 文件改动 Top -->
    <div v-if="stats" class="tool-card">
      <div class="card-header">
        <span class="card-title">文件改动 Top 10</span>
      </div>
      <div class="card-body">
        <DataTable :data="stats.top_files" :max-height="360" class="stats-table">
          <el-table-column label="#" type="index" width="44" />
          <el-table-column prop="path" label="文件路径" min-width="260" show-overflow-tooltip />
          <el-table-column prop="commits" label="提交次数" width="100" align="right" />
          <el-table-column prop="insertions" label="新增" width="90" align="right">
            <template #default="{ row }"><span class="add-color">+{{ formatNumber(row.insertions) }}</span></template>
          </el-table-column>
          <el-table-column prop="deletions" label="删除" width="90" align="right">
            <template #default="{ row }"><span class="del-color">-{{ formatNumber(row.deletions) }}</span></template>
          </el-table-column>
        </DataTable>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import DataTable from '@/components/DataTable.vue'

// ============ 类型（与 Rust snake_case 一致） ============
interface RepoInfo {
  valid: boolean
  branch: string
  root: string
  last_commit: string
}
interface AuthorStat { name: string; commits: number; insertions: number; deletions: number }
interface DayStat { date: string; commits: number; insertions: number; deletions: number }
interface FileStat { path: string; commits: number; insertions: number; deletions: number }
interface GitStats {
  branch: string
  total_commits: number
  total_insertions: number
  total_deletions: number
  earliest_date: string
  authors: AuthorStat[]
  daily: DayStat[]
  top_files: FileStat[]
}

// ============ 状态 ============
const gitAvailable = ref(true)
const repoPath = ref('')
const repoInfo = ref<RepoInfo>({ valid: false, branch: '', root: '', last_commit: '' })
const stats = ref<GitStats | null>(null)
const analyzing = ref(false)
const trendDays = ref<string>('90')

// ============ 选择仓库 ============
const pickRepo = async () => {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({ directory: true, multiple: false })
  if (typeof selected !== 'string' || !selected) return
  repoPath.value = selected
  await analyze()
}

// ============ 分析 ============
const analyze = async () => {
  if (!repoPath.value) return
  analyzing.value = true
  try {
    const info = await invoke<RepoInfo>('git_repo_info', { path: repoPath.value })
    repoInfo.value = info
    if (!info.valid) {
      stats.value = null
      ElMessage.warning('所选目录不是 Git 仓库')
      return
    }
    const result = await invoke<GitStats>('git_analyze', { path: repoPath.value })
    stats.value = result
    ElMessage.success(`分析完成：${result.total_commits} 次提交`)
  } catch (e: any) {
    ElMessage.error('分析失败: ' + (e.message || e))
  } finally {
    analyzing.value = false
  }
}

// ============ 趋势图数据 ============
const trendData = computed<DayStat[]>(() => {
  if (!stats.value) return []
  const all = stats.value.daily
  const days = Number(trendDays.value)
  if (days === 0) return all
  return all.slice(-days)
})

// ============ SVG 图表计算 ============
const chartW = 920
const chartH = 200
// 左侧留出参考线数字标签空间，右侧留出边距
const padL = 42
const padR = 8
const plotW = chartW - padL - padR
const maxTrend = computed(() => Math.max(...trendData.value.map(d => d.commits), 1))
const barSlot = computed(() => trendData.value.length ? plotW / trendData.value.length : 1)
const barWidth = computed(() => Math.max(Math.min(barSlot.value * 0.72, 18), 1))
const barX = (i: number) => padL + i * barSlot.value + (barSlot.value - barWidth.value) / 2
const barH = (commits: number) => Math.max((commits / maxTrend.value) * chartH, 2)
const barY = (commits: number) => chartH - barH(commits)
const gridLines = computed(() => {
  const max = maxTrend.value
  const step = Math.max(Math.ceil(max / 4), 1)
  return [1, 2, 3, 4].map(i => {
    const val = step * i
    // 限制 y 最小值，避免顶部参考线标签越界被裁剪
    const y = Math.max(chartH - (val / max) * chartH, 16)
    return { y, label: formatNumber(val) }
  })
})

// ============ 工具函数 ============
const formatNumber = (n: number): string => n.toLocaleString('en-US')
const maxAuthorCommits = computed(() => Math.max(...(stats.value?.authors.map(a => a.commits) || [1]), 1))
const pct = (v: number, max: number): number => Math.round((v / max) * 100)

// ============ 初始化 ============
const init = async () => {
  try {
    gitAvailable.value = await invoke<boolean>('git_check_available')
    if (!gitAvailable.value) {
      ElMessage.warning('未检测到 git，请先安装')
    }
  } catch {
    gitAvailable.value = false
  }
}
init()
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

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
.card-actions { display: flex; align-items: center; gap: 8px; }
.header-left { display: flex; align-items: center; gap: 8px; }
.card-body { padding: 16px 20px; }

.hint-icon {
  font-size: 15px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.2s;
  flex-shrink: 0;
}
.hint-icon:hover { color: var(--accent-cyan); }
.tooltip-content { max-width: 320px; line-height: 1.6; }
.tooltip-content p { margin: 2px 0; }

.repo-path {
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}
.repo-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
  flex-wrap: wrap;
}
.branch-tag {
  font-size: 12px;
  color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.12);
  border: 1px solid rgba(0, 212, 255, 0.3);
  padding: 2px 10px;
  border-radius: 10px;
}
.last-commit {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', Consolas, monospace;
}

/* 统计数字卡片 */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}
.stat-card {
  text-align: center;
  padding: 20px 12px;
}
.stat-value {
  font-size: 26px;
  font-weight: 700;
  font-family: 'JetBrains Mono', Consolas, monospace;
  color: var(--text-primary);
}
.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 6px;
}
.add-color { color: #10b981; }
.del-color { color: #ef4444; }

/* 趋势图 */
.trend-chart {
  width: 100%;
  height: auto;
  display: block;
}
.grid-line {
  stroke: var(--border-color);
  stroke-width: 0.5;
  stroke-dasharray: 4 4;
}
.grid-label {
  fill: var(--text-muted);
  font-size: 10px;
}
.axis-label {
  fill: var(--text-muted);
  font-size: 11px;
  font-family: 'JetBrains Mono', Consolas, monospace;
}
.trend-bar {
  transition: opacity 0.2s;
}
.trend-bar:hover { opacity: 0.7; }
.chart-legend {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

/* 表格内进度条 */
.bar-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bar-track {
  flex: 1;
  height: 8px;
  background: rgba(0, 0, 0, 0.25);
  border-radius: 4px;
  overflow: hidden;
}
.bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #00d4ff, #0090ff);
  border-radius: 4px;
  transition: width 0.4s;
}
.bar-num {
  min-width: 44px;
  text-align: right;
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  color: var(--text-secondary);
}

.stats-table {
  width: 100%;
}

.error-box {
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.25);
  border-radius: 6px;
}
.error-title {
  color: #ef4444;
  font-weight: 600;
  font-size: 13px;
  margin-bottom: 6px;
}
.error-msg {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.empty-tip {
  text-align: center;
  color: var(--text-muted);
  padding: 24px 0;
  font-size: 13px;
}

@media (max-width: 1000px) {
  .stat-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
