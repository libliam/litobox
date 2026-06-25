<!-- src/views/CronTool.vue -->
<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>• 可视化生成 Cron 表达式</p>
                <p>• 支持 5 字段和 6 字段格式</p>
                <p>• 实时预览和校验</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">格式</div>
            <el-radio-group v-model="isSixField" size="small">
              <el-radio-button :label="false">5字段</el-radio-button>
              <el-radio-button :label="true">6字段</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">模板</div>
            <el-select v-model="selectedTemplate" size="small" placeholder="选择模板" style="width: 180px">
              <el-option v-for="template in templates" :key="template.id" :label="template.name" :value="template.id" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleGenerate">生成</el-button>
              <el-button size="small" @click="handleClear">清空</el-button>
              <el-button size="small" @click="handleCopy">复制</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card expression-preview-card">
      <div class="card-header">
        <span class="card-title">表达式预览</span>
      </div>
      <div class="card-body">
        <div class="expression-preview">
          <code class="expression-text">{{ currentExpression }}</code>
          <div class="expression-fields">
            <span v-if="isSixField">秒 分 时 日 月 周</span>
            <span v-else>分 时 日 月 周</span>
          </div>
        </div>
        <div class="next-execution">
          <span class="next-exec-label">执行时间:</span>
          <div class="next-exec-horizontal">
            <div v-if="nextExecutionTimes.length > 0" class="next-exec-items">
              <div v-for="(time, index) in nextExecutionTimes.slice(0, 5)" :key="index" class="next-exec-item">
                <span class="next-exec-index">{{ index + 1 }}</span>
                <span class="next-exec-value">{{ time }}</span>
              </div>
            </div>
            <div v-else class="next-exec-empty">暂无执行时间</div>
          </div>
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">配置</span>
      </div>
      <div class="card-body">
        <div class="cron-config-grid">
          <CronFieldPanel 
            v-if="isSixField" 
            field="second" 
            label="秒" 
            :values="cronState.second" 
            :min="0" 
            :max="59" 
            @update-values="updateFieldValues"
          />
          <CronFieldPanel 
            field="minute" 
            label="分" 
            :values="cronState.minute" 
            :min="0" 
            :max="59" 
            @update-values="updateFieldValues"
          />
          <CronFieldPanel 
            field="hour" 
            label="时" 
            :values="cronState.hour" 
            :min="0" 
            :max="23" 
            @update-values="updateFieldValues"
          />
          <CronFieldPanel 
            field="day" 
            label="日" 
            :values="cronState.day" 
            :min="1" 
            :max="31" 
            @update-values="updateFieldValues"
          />
          <CronFieldPanel 
            field="month" 
            label="月" 
            :values="cronState.month" 
            :min="1" 
            :max="12" 
            @update-values="updateFieldValues"
          />
          <CronFieldPanel 
            field="weekday" 
            label="周" 
            :values="cronState.weekday" 
            :min="0" 
            :max="6" 
            @update-values="updateFieldValues"
          />
        </div>
      </div>
    </div>

    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">校验结果</span>
      </div>
      <div class="card-body">
        <div v-if="validationResult.isValid" class="validation-success">
          ✓ 表达式有效
        </div>
        <div v-else class="validation-error">
          ✗ {{ validationResult.error }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import { buildCronExpression, parseCronExpression, validateCronExpression, getNextExecutionTimes } from '@/utils/cronUtils'
import CronFieldPanel from './CronFieldPanel.vue'

const store = useToolboxStore()

// 是否使用6字段格式
const isSixField = ref(false)

// Cron表达式各字段状态
const cronState = reactive({
  second: new Set<number>(), // 0-59 (6字段时启用)
  minute: new Set<number>(), // 0-59
  hour: new Set<number>(),   // 0-23
  day: new Set<number>(),    // 1-31
  month: new Set<number>(),  // 1-12
  weekday: new Set<number>() // 0-6 (0=周日)
})

// 当前表达式
const currentExpression = computed(() => {
  return buildCronExpression(
    { 
      second: cronState.second, 
      minute: cronState.minute, 
      hour: cronState.hour, 
      day: cronState.day, 
      month: cronState.month, 
      weekday: cronState.weekday 
    }, 
    isSixField.value
  )
})

// 校验结果
const validationResult = computed(() => {
  return validateCronExpression(currentExpression.value, isSixField.value)
})

// 下次执行时间（最近5次）
const nextExecutionTimes = computed(() => {
  const times = getNextExecutionTimes(currentExpression.value, isSixField.value, new Date(), 5)
  return times.map(time => time.toLocaleString('zh-CN', {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
  }))
})

// 模板数据
const templates = [
  { id: 'everyMinute', name: '每分钟', expression5: '* * * * *', expression6: '* * * * * *' },
  { id: 'every5Minutes', name: '每5分钟', expression5: '*/5 * * * *', expression6: '0 */5 * * * *' },
  { id: 'everyHour', name: '每小时', expression5: '0 * * * *', expression6: '0 0 * * * *' },
  { id: 'every2Hours', name: '每2小时', expression5: '0 */2 * * *', expression6: '0 0 */2 * * *' },
  { id: 'dailyMidnight', name: '每天零点', expression5: '0 0 * * *', expression6: '0 0 0 * * *' },
  { id: 'workdays9am', name: '工作日每天9点', expression5: '0 9 * * 1-5', expression6: '0 0 9 * * 1-5' },
  { id: 'workdays9and18', name: '工作日9点和18点', expression5: '0 9,18 * * 1-5', expression6: '0 0 9,18 * * 1-5' },
  { id: 'sundayMidnight', name: '每周日零点', expression5: '0 0 * * 0', expression6: '0 0 0 * * 0' },
  { id: 'monthly1st', name: '每月1号零点', expression5: '0 0 1 * *', expression6: '0 0 0 1 * *' },
  { id: 'yearlyJan1st', name: '每年1月1日零点', expression5: '0 0 1 1 *', expression6: '0 0 0 1 1 *' }
]

// 选中的模板
const selectedTemplate = ref('')

// 更新字段值
const updateFieldValues = (field: keyof typeof cronState, values: Set<number>) => {
  cronState[field] = values
}

// 生成表达式
const handleGenerate = () => {
  if (!validationResult.value.isValid) {
    ElMessage.error(validationResult.value.error || '表达式无效')
    return
  }
  ElMessage.success('表达式生成成功')
  store.addHistory({
    tool: 'cron',
    action: 'Cron表达式生成',
    inputPreview: currentExpression.value,
    outputPreview: nextExecutionTimes.value.length > 0 ? `下次: ${nextExecutionTimes.value[0]}` : '无下次执行时间'
  })
}

// 清空所有字段
const handleClear = () => {
  Object.keys(cronState).forEach(key => {
    const field = key as keyof typeof cronState
    cronState[field].clear()
  })
  selectedTemplate.value = ''
  ElMessage.success('已清空')
}

// 复制表达式
const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(currentExpression.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 监听模板选择变化
watch(selectedTemplate, (newVal) => {
  if (newVal) {
    const template = templates.find(t => t.id === newVal)
    if (template) {
      const expression = isSixField.value ? template.expression6 : template.expression5
      const parsed = parseCronExpression(expression, isSixField.value)
      if (parsed) {
        // 更新所有字段
        Object.keys(parsed).forEach(key => {
          const field = key as keyof typeof parsed
          if (field !== 'second' || isSixField.value) {
            if (parsed[field]) {
              cronState[field] = new Set(parsed[field]!)
            }
          }
        })
        ElMessage.success(`已应用模板: ${template.name}`)
      }
    }
  }
})

// 监听字段变化时更新模板选择（如果匹配某个模板）
watch(currentExpression, () => {
  // 查找匹配的模板
  const matchedTemplate = templates.find(template => {
    const expr = isSixField.value ? template.expression6 : template.expression5
    return expr === currentExpression.value
  })
  
  if (matchedTemplate) {
    selectedTemplate.value = matchedTemplate.id
  } else {
    selectedTemplate.value = '' // 不匹配任何模板时清空选择
  }
})
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
.tool-card:last-child { margin-bottom: 0; }
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
.card-actions { display: flex; align-items: center; gap: 6px; }
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

.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }
.group-label { color: var(--text-secondary); font-size: 13px; white-space: nowrap; }
.group-buttons { display: flex; gap: 6px; }

.expression-preview-card {
  height: 240px;
}
.expression-preview-card .card-body {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  height: calc(100% - 53px);
}

.expression-preview {
  background: var(--bg-input);
  padding: 16px;
  border-radius: 6px;
  text-align: center;
  margin-bottom: 12px;
}
.expression-text {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 18px;
  color: var(--accent-cyan);
  background: transparent;
  padding: 0;
  word-break: break-all;
}
.expression-fields {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 8px;
}

.next-execution {
  padding: 8px 12px;
  background: rgba(0, 212, 255, 0.05);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  min-height: 40px;
}
.next-exec-label { 
  color: var(--text-secondary); 
  display: block;
  margin-bottom: 6px;
  text-align: center;
  font-size: 12px;
}
.next-exec-horizontal {
  display: flex;
  gap: 8px;
  justify-content: center;
  align-items: center;
  min-height: 24px;
}
.next-exec-items {
  display: flex;
  gap: 8px;
  align-items: center;
}
.next-exec-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: rgba(0, 212, 255, 0.03);
  border-radius: 4px;
  white-space: nowrap;
}
.next-exec-index {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 10px;
  font-weight: 600;
}
.next-exec-value { 
  font-weight: 500; 
  color: var(--accent-cyan);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}
.next-exec-empty {
  color: var(--text-secondary);
  text-align: center;
}

.cron-config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
}

.validation-success {
  padding: 8px 12px;
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid #22c55e;
  border-radius: 4px;
  color: #22c55e;
  font-size: 13px;
  text-align: center;
}

.validation-error {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  text-align: center;
}
</style>
