<template>
  <div class="tool-container mindmap-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">思维导图</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>用 Markdown 标题/列表自动生成脑图</p>
                <p>支持缩放、拖拽、折叠节点</p>
                <p>导出 PNG / SVG</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="action-bar">
          <el-button size="small" @click="loadExample">示例</el-button>
          <el-button size="small" @click="fitView">适配视图</el-button>
          <el-button size="small" @click="zoomIn">放大 +</el-button>
          <el-button size="small" @click="zoomOut">缩小 -</el-button>
          <el-button size="small" @click="exportSvg">导出 SVG</el-button>
          <el-button size="small" @click="exportPng" :loading="exporting">导出 PNG</el-button>
        </div>
      </div>
    </div>

    <!-- 主体：左编辑右画布 -->
    <div class="tool-card mindmap-main">
      <div class="card-body mindmap-body">
        <!-- 左侧编辑器 -->
        <div class="editor-pane">
          <div class="pane-title">Markdown 源码</div>
          <el-input
            v-model="source"
            type="textarea"
            :rows="30"
            placeholder="# 根节点&#10;## 子节点 A&#10;- 内容 1&#10;- 内容 2&#10;## 子节点 B&#10;### 更深层级"
            resize="none"
            class="source-input"
          />
          <div class="pane-footer">
            <el-button size="small" text @click="source = ''">清空</el-button>
            <span>{{ source.length }} 字符</span>
          </div>
        </div>

        <!-- 右侧画布 -->
        <div class="canvas-pane">
          <div class="canvas-header">
            <span class="pane-title">脑图画布</span>
            <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
          </div>
          <div
            ref="svgContainer"
            class="svg-container"
            :class="{ 'markmap-dark': darkMode }"
            :style="containerStyle"
            @wheel.prevent="onWheel"
            @mousedown="onMouseDown"
          >
            <svg ref="svgEl" class="mindmap-svg markmap" xmlns="http://www.w3.org/2000/svg"></svg>
            <div v-if="!source.trim()" class="empty-overlay">
              <p>输入 Markdown 后自动生成脑图</p>
              <p class="hint">支持 # 标题、- 列表、* 列表</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 样式配置 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">样式配置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">明暗主题</div>
            <el-switch v-model="darkMode" active-text="深色" inactive-text="浅色" />
          </div>
          <div class="action-group">
            <div class="group-label">主题卡片</div>
            <el-switch v-model="showBgCard" active-text="开启" inactive-text="关闭" />
          </div>
          <div class="action-group">
            <div class="group-label">配色方案</div>
            <el-select v-model="palette" size="small" style="width: 140px">
              <el-option label="青色" value="cyan" />
              <el-option label="紫色" value="purple" />
              <el-option label="绿色" value="green" />
              <el-option label="橙色" value="orange" />
              <el-option label="红粉" value="rose" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">文字颜色</div>
            <el-color-picker v-model="textColor" size="small" :predefine="textPresets" />
          </div>
          <div class="action-group">
            <div class="group-label">字体大小</div>
            <el-input-number v-model="fontSize" :min="12" :max="32" :step="2" size="small" controls-position="right" style="width: 110px" />
          </div>
          <div class="action-group">
            <div class="group-label">水平间距</div>
            <el-input-number v-model="spacingH" :min="40" :max="200" :step="10" size="small" controls-position="right" style="width: 110px" />
          </div>
          <div class="action-group">
            <div class="group-label">垂直间距</div>
            <el-input-number v-model="spacingV" :min="2" :max="30" :step="1" size="small" controls-position="right" style="width: 110px" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { Transformer } from 'markmap-lib'
import { Markmap } from 'markmap-view'
import domtoimage from 'dom-to-image-more'
import { saveFileWithDialog } from '@/utils/fileSaver'

// ============ 状态 ============
const source = ref(`# LitoBox 功能图谱

## 文本处理
- JSON 工具
- 字符串工具
- 正则测试
- 编码转换

## 开发工具
- HTTP 请求
- WebSocket
- SQL 工具
- Git 统计

## 实用工具
- PDF 工具
- 图片处理
- 二维码生成
- 思维导图

## 系统工具
- 系统信息
- 进程管理
- 网络查看
- 密码保管箱`)

const svgEl = ref<SVGSVGElement | null>(null)
const svgContainer = ref<HTMLElement | null>(null)
let mm: Markmap | null = null
const transformer = new Transformer()

const darkMode = ref(true)
const showBgCard = ref(true)
const palette = ref('cyan')
const textColor = ref('#e0e0e0')
const fontSize = ref(16)
const spacingH = ref(80)
const spacingV = ref(5)
const maxWidth = ref(0)

const scale = ref(1)
const exporting = ref(false)

let isDragging = false
let dragStart = { x: 0, y: 0 }

const textPresets = [
  '#e0e0e0', '#ffffff', '#222222',
  '#00d4ff', '#a78bfa', '#34d399',
  '#fbbf24', '#fb7185'
]

// ============ 配色方案 ============
// ponytail: 每个 palette 包含 accent（连线/主题色）+ 分支颜色数组
// 分支颜色用 d3's colorFn 的方式，按节点 path hash 取色
const palettes: Record<string, { accent: string; branches: string[] }> = {
  cyan: { accent: '#00d4ff', branches: ['#00d4ff', '#3b82f6', '#06b6d4', '#0ea5e9', '#22d3ee', '#0284c7'] },
  purple: { accent: '#a78bfa', branches: ['#a78bfa', '#8b5cf6', '#c084fc', '#d946ef', '#ec4899', '#f472b6'] },
  green: { accent: '#34d399', branches: ['#34d399', '#10b981', '#22c55e', '#84cc16', '#14b8a6', '#06b6d4'] },
  orange: { accent: '#fbbf24', branches: ['#fbbf24', '#f97316', '#f59e0b', '#eab308', '#f43f5e', '#ef4444'] },
  rose: { accent: '#fb7185', branches: ['#fb7185', '#f472b6', '#e879f9', '#a78bfa', '#c084fc', '#d946ef'] }
}

// ============ 容器样式（注入正确的 --markmap-* 变量） ============
const containerStyle = computed(() => {
  const p = palettes[palette.value]
  const textCol = textColor.value
  const bgCol = darkMode.value ? '#1a1a2e' : '#ffffff'
  const codeBg = darkMode.value ? '#2a2a3e' : '#f0f0f0'
  const codeText = darkMode.value ? '#c9d1d9' : '#24292e'
  const circleBg = darkMode.value ? '#333' : '#fff'
  const accent = p.accent

  // 关键：所有 markmap 识别的 CSS 变量
  return {
    '--markmap-font': `400 ${fontSize.value}px/1.5 "Segoe UI", "Microsoft YaHei", sans-serif`,
    '--markmap-text-color': textCol,
    '--markmap-a-color': accent,
    '--markmap-a-hover-color': accent,
    '--markmap-code-bg': codeBg,
    '--markmap-code-color': codeText,
    '--markmap-highlight-bg': accent + '44',
    '--markmap-highlight-node-bg': accent + '22',
    '--markmap-circle-open-bg': circleBg,
    '--mm-accent': accent,
    '--mm-bg': bgCol
  } as Record<string, string>
})

// ============ 渲染 ============
const renderMindmap = async () => {
  if (!svgEl.value) return
  if (!source.value.trim()) return

  try {
    const { root } = transformer.transform(source.value)
    if (!root) return

    while (svgEl.value.firstChild) {
      svgEl.value.removeChild(svgEl.value.firstChild)
    }

    const p = palettes[palette.value]

    mm = Markmap.create(svgEl.value, {
      autoFit: true,
      spacingHorizontal: spacingH.value,
      spacingVertical: spacingV.value,
      maxWidth: maxWidth.value,
      duration: 0,
      // 用 color 选项覆盖默认的 d3 scheme 颜色
      // markmap 会根据节点 path hash 来取颜色，保证同层级节点颜色稳定
      color: (node: any) => {
        const path = node.state?.path || ''
        // 用 path 字符串 hash 取值，保证同一分支颜色一致
        let hash = 0
        for (let i = 0; i < path.length; i++) hash = (hash * 31 + path.charCodeAt(i)) | 0
        const idx = Math.abs(hash) % p.branches.length
        return p.branches[idx]
      }
    }, root)

    await nextTick()
    applyNodeStyles()
  } catch (e: any) {
    console.error('Markmap render error:', e)
  }
}

// ============ 给 SVG 注入自定义样式 ============
// ponytail: markmap 的 globalCSS 会注入到 svg > style 里，我们追加自己的覆盖规则
// foreignObject 里的 div 不在 Vue scoped 里，只能直接写进 svg 的 style 标签
const injectCustomStyles = () => {
  if (!svgEl.value) return

  const size = fontSize.value
  const textCol = textColor.value
  const accent = palettes[palette.value].accent
  const rootSize = size + 10   // 根节点差 +10，拉开层级
  const l2Size = size + 5      // 二级 +5
  const l3Size = size + 2      // 三级 +2

  // 基础样式：只设字号/颜色/字体，不设置加粗
  let customCSS = `
/* ===== LitoBox 自定义覆盖 ===== */
.markmap-foreign > div {
  font-size: ${size}px !important;
  color: ${textCol} !important;
  font-family: "Segoe UI", "Microsoft YaHei", sans-serif !important;
  line-height: 1.5 !important;
  font-weight: 400 !important;
}
/* 根节点 depth=1 */
.markmap-node .markmap-foreign > div[data-depth="1"] {
  font-size: ${rootSize}px !important;
  font-weight: 700 !important;
}
/* 二级节点 depth=2 */
.markmap-node .markmap-foreign > div[data-depth="2"] {
  font-size: ${l2Size}px !important;
  font-weight: 600 !important;
}
/* 三级节点 depth=3 */
.markmap-node .markmap-foreign > div[data-depth="3"] {
  font-size: ${l3Size}px !important;
  font-weight: 500 !important;
}
`.trim()

  // 主题卡片：可开关
  if (showBgCard.value) {
    customCSS += `
/* 根节点卡片 */
.markmap-node .markmap-foreign > div[data-depth="1"] > div:first-child {
  background: ${accent}22 !important;
  border: 2px solid ${accent} !important;
  border-radius: 10px !important;
  padding: 6px 14px !important;
  margin: -6px -6px !important;
  box-shadow: 0 4px 16px rgba(0,0,0,0.2) !important;
}
/* 二级节点竖条 */
.markmap-node .markmap-foreign > div[data-depth="2"] > div:first-child {
  background: ${accent}11 !important;
  border-left: 3px solid ${accent} !important;
  padding: 3px 10px !important;
  border-radius: 0 6px 6px 0 !important;
}
`.trim()
  }

  const existing = svgEl.value.querySelector('style.markmap-custom')
  if (existing) {
    existing.textContent = customCSS
  } else {
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style')
    style.classList.add('markmap-custom')
    style.textContent = customCSS
    svgEl.value.appendChild(style)
  }
}

// ============ 给 DOM 节点打 depth 标记（替代 walk state.data） ============
// ponytail: markmap state.data 的层级关系 = DOM 中 .markmap-node 的嵌套
// 根节点在 g.markmap > g.markmap-node，子节点嵌套更深
const markNodeDepth = () => {
  if (!svgEl.value) return

  const roots = svgEl.value.querySelectorAll('g.markmap-node')
  // 计算每个节点的 depth（祖先 .markmap-node 数量）
  roots.forEach((node) => {
    let depth = 1
    let parent = node.parentElement
    while (parent && !parent.classList.contains('markmap')) {
      if (parent.classList.contains('markmap-node')) depth++
      parent = parent.parentElement
    }
    const div = node.querySelector('.markmap-foreign > div')
    if (div) {
      div.setAttribute('data-depth', String(depth))
    }
  })
}

// 合并：渲染完成后一次性应用
const applyNodeStyles = () => {
  if (!svgEl.value) return
  markNodeDepth()
  injectCustomStyles()
}

// ============ 交互 ============
const fitView = () => {
  scale.value = 1
  if (mm) mm.fit()
}

const zoomIn = () => {
  scale.value = Math.min(scale.value * 1.2, 5)
  if (mm) mm.rescale(scale.value)
}

const zoomOut = () => {
  scale.value = Math.max(scale.value / 1.2, 0.1)
  if (mm) mm.rescale(scale.value)
}

const onWheel = (e: WheelEvent) => {
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  scale.value = Math.max(0.1, Math.min(5, scale.value * delta))
  if (mm) mm.rescale(scale.value)
}

const onMouseDown = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (target.closest('g.markmap-node')) return

  isDragging = true
  dragStart = { x: e.clientX, y: e.clientY }

  const onMove = (ev: MouseEvent) => {
    if (!isDragging || !mm) return
    const dx = ev.clientX - dragStart.x
    const dy = ev.clientY - dragStart.y
    dragStart = { x: ev.clientX, y: ev.clientY }
    ;(mm as any).setTransform(dx, dy, scale.value)
  }

  const onUp = () => {
    isDragging = false
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }

  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}

// ============ 导出 SVG ============
const exportSvg = async () => {
  if (!svgEl.value) return
  const svgData = new XMLSerializer().serializeToString(svgEl.value)
  const blob = new Blob([svgData], { type: 'image/svg+xml' })
  await saveFileWithDialog(blob, 'mindmap.svg', 'svg')
}

// ============ 导出 PNG（dom-to-image-more） ============
// ponytail: dom-to-image-more 专门处理 SVG + foreignObject 场景
// 会自动把 computed style inline 到每个元素上，不依赖离屏 iframe 的 CSS 解析
const exportPng = async () => {
  if (!svgContainer.value || !mm) return
  exporting.value = true

  try {
    fitView()
    await new Promise(r => setTimeout(r, 300))

    const bgColor = darkMode.value ? '#1a1a2e' : '#ffffff'
    // toPng 返回 data URL，自动处理 foreignObject 和 computed style
    const dataUrl = await domtoimage.toPng(svgContainer.value, {
      bgcolor: bgColor,
      scale: 2,
      quality: 1
    })

    // data URL → Blob → 保存
    const base64 = dataUrl.split(',')[1]
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    const blob = new Blob([bytes], { type: 'image/png' })
    await saveFileWithDialog(blob, 'mindmap.png', 'png')
  } catch (e: any) {
    console.error('PNG export error:', e)
    ElMessage.error('PNG 导出失败: ' + e.message)
  } finally {
    exporting.value = false
  }
}

// ============ 示例 ============
const loadExample = () => {
  source.value = `# 项目架构

## 前端
- Vue 3 + TypeScript
- Vite 构建
- Element Plus

## 后端
- Tauri 2.0
- Rust
- SQLite

## 工具链
- npm / pnpm
- Git
- VS Code

## 核心模块
### 文本工具
- JSON / Markdown
- 正则 / 编码

### 开发工具
- HTTP / WebSocket
- SQL / Git

### 系统工具
- 进程 / 网络
- 文件 / 磁盘`
}

// ============ 自动重新渲染 ============
let renderTimer: ReturnType<typeof setTimeout> | null = null
watch([source, palette, darkMode, spacingH, spacingV, maxWidth], () => {
  if (renderTimer) clearTimeout(renderTimer)
  renderTimer = setTimeout(renderMindmap, 400)
})

// 字体/颜色/卡片开关变化：只更新样式不需要重渲染
watch([fontSize, textColor, showBgCard], () => {
  applyNodeStyles()
})

watch(() => (mm as any)?.state?.transform?.k, (v: any) => {
  if (typeof v === 'number') scale.value = v
})

onMounted(() => {
  nextTick(renderMindmap)
})

onUnmounted(() => {
  if (renderTimer) clearTimeout(renderTimer)
})
</script>

<style scoped>
.mindmap-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.mindmap-container .mindmap-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.mindmap-container .mindmap-main :deep(.card-body) {
  flex: 1;
  display: grid;
  grid-template-columns: 340px 1fr;
  min-height: 0;
  padding: 0;
  gap: 0;
}

.mindmap-container .editor-pane {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  min-height: 0;
}

.mindmap-container .editor-pane .pane-title,
.mindmap-container .canvas-header .pane-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 12px 16px 6px;
}

.mindmap-container .editor-pane :deep(.el-textarea) {
  flex: 1;
  padding: 0 16px;
  min-height: 0;
}

.mindmap-container .editor-pane :deep(.el-textarea__inner) {
  font-family: 'Consolas', 'Menlo', monospace;
  font-size: 13px;
  line-height: 1.6;
  height: 100%;
  min-height: 0;
}

.mindmap-container .pane-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  font-size: 12px;
  color: var(--text-secondary);
  border-top: 1px solid var(--border-color);
}

.mindmap-container .canvas-pane {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.mindmap-container .canvas-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.mindmap-container .zoom-info {
  font-size: 12px;
  color: var(--text-secondary);
  padding-right: 16px;
}

.mindmap-container .svg-container {
  flex: 1;
  background: var(--mm-bg, #1a1a2e);
  overflow: hidden;
  cursor: grab;
  position: relative;
  min-height: 0;
}

.mindmap-container .svg-container:active {
  cursor: grabbing;
}

.mindmap-container .mindmap-svg {
  width: 100%;
  height: 100%;
  display: block;
}

/* ============ 根节点 & 二级节点特殊样式 ============ */
.mindmap-container :deep(.mm-root) > div:first-child {
  background: var(--markmap-highlight-node-bg, rgba(0, 212, 255, 0.25));
  border: 2px solid var(--mm-accent, #00d4ff);
  border-radius: 10px;
  padding: 8px 16px;
  margin: -6px -6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.mindmap-container :deep(.mm-l2) > div:first-child {
  background: var(--markmap-highlight-node-bg, rgba(0, 212, 255, 0.15));
  border-left: 3px solid var(--mm-accent, #00d4ff);
  padding: 3px 12px;
  border-radius: 0 6px 6px 0;
}

.mindmap-container :deep(.mm-l3) {
  font-weight: 500;
}

/* ============ 空状态 ============ */
.mindmap-container .empty-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  pointer-events: none;
  font-size: 14px;
  gap: 6px;
}

.mindmap-container .empty-overlay .hint {
  font-size: 12px;
  opacity: 0.6;
}
</style>
