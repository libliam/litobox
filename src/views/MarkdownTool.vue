<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="markdown-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 实时预览 -->
      <el-tab-pane label="预览" name="preview">
        <!-- 操作卡片 -->
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入 Markdown 文本，实时渲染预览</p>
                    <p>支持常用语法：标题、列表、代码块、表格等</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" type="primary" @click="handleSaveRecord">保存</el-button>
              <el-button size="small" @click="handleClear('preview')">清空</el-button>
              <el-button size="small" @click="handlePaste('preview')">粘贴</el-button>
              <el-button size="small" @click="showTemplateSelect = !showTemplateSelect">模板</el-button>
            </div>
          </div>
          <div class="card-body" v-if="showTemplateSelect">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">模板</div>
                <el-select v-model="previewTemplate" size="small" style="width: 160px" @change="handleTemplateChange">
                  <el-option label="空白" value="blank" />
                  <el-option label="README" value="readme" />
                  <el-option label="表格示例" value="table" />
                  <el-option label="列表示例" value="list" />
                </el-select>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">Markdown 输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('preview')">清空</el-button>
              <el-button size="small" @click="handlePaste('preview')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.preview.input"
              type="textarea"
              :rows="3"
              placeholder="请输入 Markdown 文本..."
              resize="vertical"
              class="markdown-input"
            />
          </div>
        </div>

        <!-- 预览卡片 -->
        <div class="tool-card fill-card">
          <div class="card-header">
            <span class="card-title">预览</span>
            <div class="card-actions">
              <el-button size="small" @click="handleCopyHtml">复制 HTML</el-button>
              <el-button size="small" @click="handleExportHtml">导出 HTML</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="markdown-preview" v-html="renderedHtml"></div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: HTML 转换 -->
      <el-tab-pane label="HTML" name="html">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('html')">清空</el-button>
              <el-button size="small" @click="handlePaste('html')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">方向</div>
                <el-select v-model="htmlDirection" size="small" style="width: 160px">
                  <el-option label="Markdown → HTML" value="md2html" />
                  <el-option label="HTML → Markdown" value="html2md" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleConvert">转换</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('html')">清空</el-button>
              <el-button size="small" @click="handlePaste('html')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.html.input"
              type="textarea"
              :rows="5"
              :placeholder="htmlDirection === 'md2html' ? '请输入 Markdown 文本...' : '请输入 HTML 代码...'"
              resize="vertical"
            />
          </div>
        </div>

        <!-- 输出卡片 -->
        <div class="tool-card fill-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('html')">复制</el-button>
          </div>
          <div class="card-body">
            <el-input
              :model-value="tabState.html.output"
              type="textarea"
              :rows="10"
              readonly
              resize="vertical"
              :class="{ 'error': tabState.html.isError }"
            />
            <div v-if="tabState.html.error" class="error-message">
              {{ tabState.html.error }}
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 统计信息 -->
      <el-tab-pane label="统计" name="stats">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('stats')">清空</el-button>
              <el-button size="small" @click="handlePaste('stats')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleStats">统计</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('stats')">清空</el-button>
              <el-button size="small" @click="handlePaste('stats')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.stats.input"
              type="textarea"
              :rows="4"
              placeholder="请输入 Markdown 文本..."
              resize="vertical"
            />
          </div>
        </div>

        <!-- 统计结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">统计结果</span>
          </div>
          <div class="card-body">
            <div v-if="statsResult" class="stats-grid">
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.chars }}</div>
                <div class="stat-label">字符数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.lines }}</div>
                <div class="stat-label">行数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.words }}</div>
                <div class="stat-label">词数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.headings }}</div>
                <div class="stat-label">标题数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.links }}</div>
                <div class="stat-label">链接数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.images }}</div>
                <div class="stat-label">图片数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ statsResult.readTime }}</div>
                <div class="stat-label">阅读时间(分钟)</div>
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"统计"按钮查看详细信息
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: 保存记录 -->
      <el-tab-pane label="记录" name="records">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">保存记录（{{ records.length }}）</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>非置顶记录最多保留 20 条，超出后自动清理最旧的</p>
                    <p>置顶记录不会被自动清理</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="loadRecords">刷新</el-button>
            </div>
          </div>
        </div>

        <div class="tool-card">
          <div class="card-body records-body">
            <div v-if="records.length === 0" class="records-empty">
              暂无保存记录，在"预览"页点击"保存"添加
            </div>
            <div
              v-for="r in records"
              :key="r.id"
              class="record-item"
              :class="{ pinned: r.pinned }"
            >
              <div class="record-main">
                <div class="record-title">
                  <el-icon v-if="r.pinned" class="pin-icon"><Top /></el-icon>
                  <span class="record-title-text">{{ r.title }}</span>
                </div>
                <div class="record-preview">{{ r.content.slice(0, 120) }}</div>
                <div class="record-time">{{ formatTime(r.updated_at) }}</div>
              </div>
              <div class="record-actions">
                <el-button size="small" type="primary" @click="handleLoadRecord(r)">载入</el-button>
                <el-button size="small" @click="handleTogglePin(r)">{{ r.pinned ? '取消置顶' : '置顶' }}</el-button>
                <el-button size="small" type="danger" @click="handleDeleteRecord(r)">删除</el-button>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, onActivated, onDeactivated } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { QuestionFilled, Top } from '@element-plus/icons-vue'
import MarkdownIt from 'markdown-it'
import { useToolboxStore } from '@/store'
import { saveFileWithDialog } from '@/utils/fileSaver'
import * as db from '@/utils/dbClient'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

const store = useToolboxStore()
const { confirm } = useConfirmDialog()

// 初始化 markdown-it
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true
})

// ============ Tab 状态 ============
const activeTab = ref('preview')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  preview: { input: '', output: '', error: '', isError: false },
  html: { input: '', output: '', error: '', isError: false },
  stats: { input: '', output: '', error: '', isError: false }
})

// 配置
const previewTemplate = ref('blank')
const showTemplateSelect = ref(false)
const htmlDirection = ref('md2html')
const statsResult = ref<StatsResult | null>(null)

interface StatsResult {
  chars: number
  lines: number
  words: number
  headings: number
  links: number
  images: number
  readTime: number
}

// 渲染 HTML（仅预览 Tab）
const renderedHtml = computed(() => {
  const input = tabState.preview.input
  if (!input.trim()) return '<p class="empty-hint">输入 Markdown 文本后，此处将显示渲染结果</p>'
  return md.render(input)
})

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'stats') {
    statsResult.value = null
  }
}

const handlePaste = async (tab: string) => {
  try {
    const text = await navigator.clipboard.readText()
    tabState[tab].input = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

const handleCopy = async (tab: string) => {
  const text = tabState[tab].output || tabState[tab].input
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

const handleCopyHtml = async () => {
  const html = renderedHtml.value
  if (!html || html.includes('empty-hint')) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(html)
    ElMessage.success('HTML 已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleExportHtml = async () => {
  const html = renderedHtml.value
  if (!html || html.includes('empty-hint')) {
    ElMessage.warning('没有可导出的内容')
    return
  }
  
  const fullHtml = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Markdown 导出</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; line-height: 1.6; color: #333; }
    pre { background: #f5f5f5; padding: 16px; border-radius: 4px; overflow-x: auto; }
    code { background: #f5f5f5; padding: 2px 6px; border-radius: 3px; font-size: 0.9em; }
    pre code { background: none; padding: 0; }
    table { border-collapse: collapse; width: 100%; margin: 16px 0; }
    th, td { border: 1px solid #ddd; padding: 8px 12px; text-align: left; }
    th { background: #f5f5f5; }
    blockquote { border-left: 4px solid #ddd; margin: 0; padding: 8px 16px; color: #666; }
    img { max-width: 100%; }
    a { color: #0366d6; }
  </style>
</head>
<body>
${html}
</body>
</html>`

  const blob = new Blob([fullHtml], { type: 'text/html;charset=utf-8' })
  await saveFileWithDialog(blob, 'markdown-export.html', 'html')
}

const addHistory = (action: string) => {
  store.addHistory({
    tool: 'markdown',
    action,
    inputPreview: tabState[activeTab.value].input.slice(0, 50),
    outputPreview: tabState[activeTab.value].output.slice(0, 50),
    inputFull: tabState[activeTab.value].input,
    outputFull: tabState[activeTab.value].output,
  })
}

// ============ 模板 ============
const TEMPLATES: Record<string, string> = {
  blank: '',
  readme: `# 项目名称

## 简介

这是一个示例 README 文件。

## 功能特性

- 功能一：描述
- 功能二：描述
- 功能三：描述

## 安装

\`\`\`bash
npm install
\`\`\`

## 使用

\`\`\`javascript
const app = require('./app')
app.start()
\`\`\`

## 表格示例

| 功能 | 状态 | 备注 |
|------|------|------|
| 功能A | ✅ 已完成 | - |
| 功能B | 🚧 进行中 | 预计下周完成 |

## 链接

- [GitHub](https://github.com)
- [文档](https://example.com)

## 许可证

MIT License`,
  table: `# 表格示例

| 姓名 | 年龄 | 城市 |
|------|------|------|
| 张三 | 25 | 北京 |
| 李四 | 30 | 上海 |
| 王五 | 28 | 广州 |

| 产品 | 价格 | 库存 |
|------|------|------|
| 产品A | ¥99 | 100 |
| 产品B | ¥199 | 50 |`,
  list: `# 列表示例

## 无序列表

- 项目一
  - 子项目 A
  - 子项目 B
- 项目二
- 项目三

## 有序列表

1. 第一步
2. 第二步
3. 第三步

## 任务列表

- [x] 已完成任务
- [ ] 待办任务一
- [ ] 待办任务二`
}

const handleTemplateChange = (val: string) => {
  tabState.preview.input = TEMPLATES[val] || ''
  previewTemplate.value = val
}

// ============ HTML 转换 ============
const handleConvert = () => {
  const input = tabState.html.input.trim()
  if (!input) {
    ElMessage.warning('请输入内容')
    return
  }

  try {
    if (htmlDirection.value === 'md2html') {
      tabState.html.output = md.render(input)
    } else {
      // HTML → Markdown（简化版）
      let result = input
      // 移除 HTML 标签（保留基本结构）
      result = result.replace(/<h([1-6])[^>]*>(.*?)<\/h\1>/gi, (_, level, content) => {
        return '#'.repeat(parseInt(level)) + ' ' + content + '\n\n'
      })
      result = result.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n')
      result = result.replace(/<br\s*\/?>/gi, '\n')
      result = result.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**')
      result = result.replace(/<b[^>]*>(.*?)<\/b>/gi, '**$1**')
      result = result.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*')
      result = result.replace(/<i[^>]*>(.*?)<\/i>/gi, '*$1*')
      result = result.replace(/<a[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>/gi, '[$2]($1)')
      result = result.replace(/<img[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*\/?>/gi, '![$2]($1)')
      result = result.replace(/<img[^>]*src="([^"]*)"[^>]*\/?>/gi, '![]($1)')
      result = result.replace(/<code[^>]*>(.*?)<\/code>/gi, '`$1`')
      result = result.replace(/<pre[^>]*><code[^>]*>(.*?)<\/code><\/pre>/gis, '```\n$1\n```')
      result = result.replace(/<ul[^>]*>(.*?)<\/ul>/gis, '$1')
      result = result.replace(/<ol[^>]*>(.*?)<\/ol>/gis, '$1')
      result = result.replace(/<li[^>]*>(.*?)<\/li>/gi, '- $1\n')
      result = result.replace(/<[^>]+>/g, '')
      result = result.replace(/&nbsp;/g, ' ')
      result = result.replace(/&amp;/g, '&')
      result = result.replace(/&lt;/g, '<')
      result = result.replace(/&gt;/g, '>')
      tabState.html.output = result.trim()
    }
    tabState.html.error = ''
    tabState.html.isError = false
    addHistory(htmlDirection.value === 'md2html' ? 'Markdown转HTML' : 'HTML转Markdown')
    ElMessage.success('转换成功')
  } catch (e: any) {
    tabState.html.output = ''
    tabState.html.error = '转换失败: ' + (e.message || '未知错误')
    tabState.html.isError = true
    ElMessage.error('转换失败')
  }
}

// ============ 统计 ============
const handleStats = () => {
  const input = tabState.stats.input
  if (!input.trim()) {
    ElMessage.warning('请输入内容')
    return
  }

  const chars = input.length
  const lines = input.split('\n').length
  const words = input.trim().split(/\s+/).filter(w => w.length > 0).length
  
  // 统计标题
  const headings = (input.match(/^#{1,6}\s+/gm) || []).length
  
  // 统计链接
  const links = (input.match(/\[([^\]]+)\]\(([^)]+)\)/g) || []).length
  
  // 统计图片
  const images = (input.match(/!\[([^\]]*)\]\(([^)]+)\)/g) || []).length
  
  // 估算阅读时间（中文按 300 字/分钟，英文按 200 词/分钟）
  const chineseChars = (input.match(/[\u4e00-\u9fa5]/g) || []).length
  const readTime = Math.max(1, Math.ceil(chineseChars > 0 ? chineseChars / 300 : words / 200))

  statsResult.value = {
    chars,
    lines,
    words,
    headings,
    links,
    images,
    readTime
  }

  addHistory('统计')
  ElMessage.success('统计完成')
}

// ============ 保存记录 ============
const records = ref<db.MarkdownRecord[]>([])

const loadRecords = async () => {
  try {
    records.value = await db.listMarkdownRecords()
  } catch {
    records.value = []
  }
}

onMounted(loadRecords)

const formatTime = (iso: string) => {
  const d = new Date(iso)
  return isNaN(d.getTime()) ? iso : d.toLocaleString('zh-CN', { hour12: false })
}

const handleSaveRecord = async () => {
  const content = tabState.preview.input
  if (!content.trim()) {
    ElMessage.warning('没有可保存的内容')
    return
  }
  // 默认标题：取首个 Markdown 标题，没有则用时间
  const heading = content.match(/^#{1,6}\s+(.+)$/m)
  const defaultTitle = heading
    ? heading[1].trim().slice(0, 40)
    : `Markdown ${new Date().toLocaleString('zh-CN', { hour12: false })}`
  try {
    const { value } = await ElMessageBox.prompt('请输入记录标题', '保存记录', {
      confirmButtonText: '保存',
      cancelButtonText: '取消',
      inputValue: defaultTitle,
      inputValidator: (v: string) => (v && v.trim() ? true : '标题不能为空')
    })
    const now = new Date().toISOString()
    const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 7)
    await db.saveMarkdownRecord({
      id,
      title: value.trim(),
      content,
      pinned: false,
      created_at: now,
      updated_at: now
    })
    await loadRecords()
    ElMessage.success('已保存到记录')
    addHistory('保存记录')
  } catch {
    // 用户取消
  }
}

// ============ Ctrl+S 快捷键保存 ============
// ponytail: window 级监听 + KeepAlive onActivated/onDeactivated 控制，避免缓存页面在后台误触发
let pageVisible = true

const handleGlobalKeydown = (e: KeyboardEvent) => {
  if (!pageVisible) return
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
    e.preventDefault()
    handleSaveRecord()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown)
})

onActivated(() => {
  pageVisible = true
})

onDeactivated(() => {
  pageVisible = false
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
})

const handleLoadRecord = (record: db.MarkdownRecord) => {
  tabState.preview.input = record.content
  activeTab.value = 'preview'
  ElMessage.success(`已载入「${record.title}」`)
}

const handleTogglePin = async (record: db.MarkdownRecord) => {
  try {
    await db.setMarkdownPin(record.id, !record.pinned)
    await loadRecords()
    ElMessage.success(record.pinned ? '已取消置顶' : '已置顶')
  } catch {
    ElMessage.error('操作失败')
  }
}

const handleDeleteRecord = async (record: db.MarkdownRecord) => {
  const ok = await confirm.ask('删除记录', `确定删除「${record.title}」吗？`, { type: 'danger', confirmText: '删除' })
  if (!ok) return
  try {
    await db.deleteMarkdownRecord(record.id)
    await loadRecords()
    ElMessage.success('已删除')
  } catch {
    ElMessage.error('删除失败')
  }
}

// ============ 自动执行（预览 Tab 实时渲染） ============
// 预览 Tab 不需要自动执行，因为 computed 已经实时渲染
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.markdown-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .markdown-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.markdown-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.markdown-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.markdown-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.markdown-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.markdown-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

/* ===== 让内容撑满 Tab 剩余高度（预览/输出卡片吸收空白） ===== */
.markdown-tool-tabs {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.markdown-tool-tabs :deep(.el-tabs__content) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.markdown-tool-tabs :deep(.el-tab-pane) {
  /* flex 撑满 content，内容超出时由 pane 自身滚动（内容多滚动、内容少撑满） */
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
}

.fill-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.fill-card > .card-header {
  flex-shrink: 0;
}

.fill-card > .card-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.fill-card :deep(.el-textarea) {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.fill-card :deep(.el-textarea__inner) {
  flex: 1;
  height: 100%;
  box-sizing: border-box;
}

/* ===== 工具卡片 ===== */
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

/* Sticky 卡片 */
.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 标题栏 */
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

/* 操作按钮 */
.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: center;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-label {
  color: var(--text-secondary);
  font-size: 13px;
  white-space: nowrap;
}

.group-buttons {
  display: flex;
  gap: 6px;
}

/* 提示图标 */
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

.tooltip-content code {
  background: rgba(0, 212, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 12px;
}

/* 错误提示 */
.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--accent-red);
  border-radius: 4px;
  color: var(--accent-red);
  font-size: 13px;
  line-height: 1.5;
}

:deep(.el-textarea.error .el-textarea__inner) {
  border-color: var(--accent-red);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.1);
}

/* ===== Markdown 预览样式 ===== */
.markdown-preview {
  padding: 16px;
  background: var(--bg-input);
  border-radius: 6px;
  /* 撑满预览卡片剩余高度（由 .fill-card 布局分配），内容超出时内部滚动 */
  flex: 1;
  min-height: 240px;
  overflow-y: auto;
  line-height: 1.7;
  color: var(--text-primary);
}

.markdown-preview :deep(h1),
.markdown-preview :deep(h2),
.markdown-preview :deep(h3),
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
  color: var(--text-primary);
}

.markdown-preview :deep(h1) { font-size: 2em; border-bottom: 1px solid var(--border-color); padding-bottom: 0.3em; }
.markdown-preview :deep(h2) { font-size: 1.5em; border-bottom: 1px solid var(--border-color); padding-bottom: 0.3em; }
.markdown-preview :deep(h3) { font-size: 1.25em; }
.markdown-preview :deep(h4) { font-size: 1em; }

.markdown-preview :deep(p) {
  margin-top: 0;
  margin-bottom: 16px;
}

.markdown-preview :deep(a) {
  color: var(--accent-cyan);
  text-decoration: none;
}

.markdown-preview :deep(a:hover) {
  text-decoration: underline;
}

.markdown-preview :deep(code) {
  background: rgba(0, 212, 255, 0.1);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-size: 0.9em;
  font-family: 'Consolas', 'Monaco', monospace;
}

.markdown-preview :deep(pre) {
  background: rgba(0, 0, 0, 0.3);
  padding: 16px;
  border-radius: 6px;
  overflow-x: auto;
  margin-bottom: 16px;
}

.markdown-preview :deep(pre code) {
  background: none;
  padding: 0;
  font-size: 0.9em;
  line-height: 1.5;
}

.markdown-preview :deep(blockquote) {
  border-left: 4px solid var(--accent-cyan);
  padding: 8px 16px;
  margin: 0 0 16px 0;
  color: var(--text-secondary);
  background: rgba(0, 212, 255, 0.05);
  border-radius: 0 4px 4px 0;
}

.markdown-preview :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin-bottom: 16px;
}

.markdown-preview :deep(th),
.markdown-preview :deep(td) {
  border: 1px solid var(--border-color);
  padding: 8px 12px;
  text-align: left;
}

.markdown-preview :deep(th) {
  background: rgba(0, 212, 255, 0.1);
  font-weight: 600;
}

.markdown-preview :deep(tr:nth-child(even)) {
  background: rgba(0, 0, 0, 0.1);
}

.markdown-preview :deep(ul),
.markdown-preview :deep(ol) {
  padding-left: 2em;
  margin-bottom: 16px;
}

.markdown-preview :deep(li) {
  margin-bottom: 4px;
}

.markdown-preview :deep(img) {
  max-width: 100%;
  border-radius: 4px;
}

.markdown-preview :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 24px 0;
}

.markdown-preview :deep(.empty-hint) {
  color: var(--text-muted);
  text-align: center;
  padding: 40px 0;
  font-style: italic;
}

/* ===== 统计网格 ===== */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 16px;
}

.stat-item {
  text-align: center;
  padding: 16px;
  background: var(--bg-input);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--accent-cyan);
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}

/* ===== 保存记录 ===== */
.records-body {
  padding: 8px 12px;
}

.records-empty {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  font-size: 13px;
}

.record-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid transparent;
  transition: background 0.2s, border-color 0.2s;
  margin-bottom: 4px;
}

.record-item:hover {
  background: rgba(0, 212, 255, 0.05);
}

.record-item.pinned {
  border-color: rgba(0, 212, 255, 0.3);
  background: rgba(0, 212, 255, 0.06);
}

.record-main {
  flex: 1;
  min-width: 0;
}

.record-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.record-title-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pin-icon {
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.record-preview {
  margin-top: 2px;
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.record-time {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-muted);
}

.record-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
</style>
