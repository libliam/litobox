<template>
  <div class="tool-container slide-container">
    <!-- 编辑区 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">Markdown 幻灯片</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>用 --- 分割每张幻灯片</p>
                <p>支持 Markdown 语法、代码块、表格</p>
                <p>快捷键：← / → / Space 翻页，F 全屏</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="action-bar">
          <el-button size="small" @click="loadExample">示例</el-button>
          <el-button size="small" @click="exportHtml">导出 HTML</el-button>
          <el-button size="small" type="primary" @click="openPreview">预览 (F5)</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="split-layout">
          <!-- 左侧源码 -->
          <div class="split-pane">
            <div class="pane-title">Markdown 源码</div>
            <el-input
              v-model="source"
              type="textarea"
              :rows="20"
              placeholder="用 --- 分割每张幻灯片&#10;&#10;# 第一张&#10;这里是内容&#10;&#10;---&#10;&#10;# 第二张&#10;更多内容"
              resize="none"
              class="source-input"
            />
            <div class="source-footer">
              <span>{{ slides.length }} 张幻灯片</span>
              <span>{{ source.length }} 字符</span>
            </div>
          </div>
          <!-- 右侧预览缩略 -->
          <div class="split-pane preview-pane">
            <div class="pane-title">幻灯片预览（{{ currentIndex + 1 }} / {{ slides.length }}）</div>
            <div class="slide-thumb-wrap">
              <div class="slide-thumb" :style="thumbScaleStyle">
                <div v-if="currentSlide" class="slide-frame" v-html="renderedCurrentSlide"></div>
                <div v-else class="empty-hint">输入 Markdown 后预览</div>
              </div>
            </div>
            <div class="thumb-nav">
              <el-button size="small" :disabled="currentIndex === 0" @click="prevSlide">←</el-button>
              <el-button size="small" :disabled="currentIndex >= slides.length - 1" @click="nextSlide">→</el-button>
              <el-input-number
                v-model="currentIndex"
                :min="0"
                :max="Math.max(0, slides.length - 1)"
                size="small"
                controls-position="right"
                style="width: 90px"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 快捷配置 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">样式配置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">主题</div>
            <el-select v-model="theme" size="small" style="width: 160px">
              <el-option label="深色 (Dark)" value="dark" />
              <el-option label="浅色 (Light)" value="light" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">代码高亮</div>
            <el-select v-model="codeTheme" size="small" style="width: 160px">
              <el-option label="GitHub Dark" value="github-dark" />
              <el-option label="GitHub Light" value="github-light" />
              <el-option label="Dracula" value="dracula" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">幻灯片比例</div>
            <el-select v-model="aspectRatio" size="small" style="width: 140px">
              <el-option label="16:9" value="16:9" />
              <el-option label="4:3" value="4:3" />
            </el-select>
          </div>
          <div class="action-group">
            <div class="group-label">字号</div>
            <el-input-number v-model="fontSize" :min="14" :max="36" :step="2" size="small" controls-position="right" style="width: 110px" />
          </div>
        </div>
      </div>
    </div>

    <!-- 全屏预览遮罩 -->
    <teleport to="body">
      <div v-if="showPreview" class="preview-overlay" :class="theme" @click.self="closePreview">
        <div class="preview-toolbar" @click.stop>
          <span>{{ currentIndex + 1 }} / {{ slides.length }}</span>
          <el-button size="small" text @click="prevSlide" :disabled="currentIndex === 0">←</el-button>
          <el-button size="small" text @click="nextSlide" :disabled="currentIndex >= slides.length - 1">→</el-button>
          <el-button size="small" text @click="closePreview">ESC 退出</el-button>
        </div>
        <div class="slide-full" :class="[theme, 'ratio-' + aspectRatio.replace(':', '')]">
          <div class="slide-content" :style="{ fontSize: fontSize + 'px' }" v-html="renderedCurrentSlide"></div>
          <div class="slide-footer">
            <span class="slide-num">{{ currentIndex + 1 }} / {{ slides.length }}</span>
          </div>
        </div>
      </div>
    </teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import MarkdownIt from 'markdown-it'
import { saveFileWithDialog } from '@/utils/fileSaver'

// ============ markdown-it 实例 ============
const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true
})

// ============ 状态 ============
const source = ref(`# 欢迎使用 LitoBox 幻灯片

**Markdown 幻灯片工具**

---

# 快速上手

- 用 \`---\` 分割幻灯片
- 支持 **Markdown 语法**
- 支持 \`代码块\`、表格、列表等

---

# 代码示例

\`\`\`javascript
function hello(name) {
  return \`Hello, \${name}!\`
}
console.log(hello('World'))
\`\`\`

---

# 表格支持

| 功能 | 状态 |
|------|------|
| 代码高亮 | ✅ |
| 导出 HTML | ✅ |
| 全屏预览 | ✅ |

---

# 快捷键

| 按键 | 功能 |
|------|------|
| ← / ↑ | 上一页 |
| → / Space | 下一页 |
| ESC | 退出全屏 |

---

# 谢谢！

💡 用 Markdown 写你的下一份演示吧`)

const currentIndex = ref(0)
const showPreview = ref(false)
const theme = ref<'dark' | 'light'>('dark')
const codeTheme = ref('github-dark')
const aspectRatio = ref('16:9')
const fontSize = ref(18)

// ============ 计算属性 ============
// 按 --- 分割（必须独占一行，前后允许空行）
const slides = computed(() => {
  if (!source.value.trim()) return []
  const parts = source.value
    .split(/\n\s*---\s*\n/)
    .map(s => s.trim())
    .filter(Boolean)
  return parts.length > 0 ? parts : [source.value.trim()]
})

// 当分割后索引越界时修正
watch(slides, (val) => {
  if (currentIndex.value >= val.length) {
    currentIndex.value = Math.max(0, val.length - 1)
  }
})

const currentSlide = computed(() => slides.value[currentIndex.value] || '')
const renderedCurrentSlide = computed(() => {
  if (!currentSlide.value) return ''
  return md.render(currentSlide.value)
})

// 缩略预览缩放
const thumbScaleStyle = computed(() => {
  const baseW = aspectRatio.value === '16:9' ? 800 : 720
  const baseH = aspectRatio.value === '16:9' ? 450 : 540
  const containerW = 360
  const containerH = 220
  const scale = Math.min(containerW / baseW, containerH / baseH) * 0.95
  return {
    width: baseW + 'px',
    height: baseH + 'px',
    transform: `scale(${scale})`,
    transformOrigin: 'top left'
  }
})

// ============ 方法 ============
const prevSlide = () => {
  if (currentIndex.value > 0) currentIndex.value--
}

const nextSlide = () => {
  if (currentIndex.value < slides.value.length - 1) currentIndex.value++
}

const openPreview = () => {
  if (slides.value.length === 0) {
    ElMessage.warning('没有可预览的幻灯片')
    return
  }
  showPreview.value = true
}

const closePreview = () => {
  showPreview.value = false
}

// ============ 快捷键 ============
const handleKeydown = (e: KeyboardEvent) => {
  // 拦截浏览器/Tauri 刷新快捷键
  if ((e.key === 'F5' || ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'r')) && slides.value.length > 0) {
    e.preventDefault()
    if (!showPreview.value) {
      openPreview()
    }
    return
  }

  if (!showPreview.value) return
  switch (e.key) {
    case 'ArrowLeft':
    case 'ArrowUp':
      prevSlide()
      break
    case 'ArrowRight':
    case ' ':
    case 'Enter':
    case 'PageDown':
      e.preventDefault()
      nextSlide()
      break
    case 'Escape':
      closePreview()
      break
    case 'Home':
      currentIndex.value = 0
      break
    case 'End':
      currentIndex.value = slides.value.length - 1
      break
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// ============ 导出 HTML ============
const exportHtml = async () => {
  if (slides.value.length === 0) {
    ElMessage.warning('没有可导出的幻灯片')
    return
  }

  // 生成每一页的 HTML
  const slideHtmls = slides.value.map((s, i) =>
    `<section class="slide" data-index="${i}">${md.render(s)}</section>`
  ).join('\n')

  const html = buildStandaloneHtml(slideHtmls, {
    theme: theme.value,
    codeTheme: codeTheme.value,
    aspectRatio: aspectRatio.value,
    fontSize: fontSize.value
  })

  const blob = new Blob([html], { type: 'text/html;charset=utf-8' })
  await saveFileWithDialog(blob, 'slides.html', 'html')
}

const buildStandaloneHtml = (slidesHtml: string, opts: { theme: string; codeTheme: string; aspectRatio: string; fontSize: number }) => {
  const darkMode = opts.theme === 'dark'
  const bg = darkMode ? '#1a1a2e' : '#ffffff'
  const fg = darkMode ? '#e0e0e0' : '#222222'
  const accent = '#00d4ff'
  const ratio = opts.aspectRatio === '16:9' ? '56.25%' : '75%'
  const darkBgOuter = darkMode ? '#0d0d1a' : '#e8e8e8'
  const darkBlockBg = darkMode ? '#2a2a3e' : '#f0f0f0'
  const darkBorder = darkMode ? '#444' : '#ddd'
  const darkQuoteColor = darkMode ? '#999' : '#555'
  const darkFooterColor = darkMode ? '#666' : '#aaa'
  const darkToolbarBg = darkMode ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.08)'
  const darkToolbarBtnBorder = darkMode ? '#555' : '#ccc'
  const darkToolbarBtnColor = darkMode ? '#fff' : '#333'
  const darkToolbarCounter = darkMode ? '#aaa' : '#666'

  let codeBlockCss: string
  if (opts.codeTheme === 'dracula') {
    codeBlockCss = '    .standalone pre { background: #282a36; color: #f8f8f2; } .standalone code { color: #f8f8f2; }'
  } else if (opts.codeTheme === 'github-light') {
    codeBlockCss = '    .standalone pre { background: #f6f8fa; color: #24292e; } .standalone code { color: #24292e; }'
  } else {
    codeBlockCss = '    .standalone pre { background: #161b22; color: #c9d1d9; } .standalone code { color: #c9d1d9; }'
  }

  const aspectExpr = opts.aspectRatio === '16:9' ? '16 / 9' : '4 / 3'

  const css = `* { margin: 0; padding: 0; box-sizing: border-box; }
  body { background: ${darkBgOuter}; color: ${fg}; font-family: -apple-system, 'Segoe UI', 'Microsoft YaHei', sans-serif; overflow: hidden; }
  .standalone { width: 100vw; height: 100vh; display: flex; align-items: center; justify-content: center; padding: 40px; }
  .slides-wrapper { width: 100%; max-width: calc(100vh * ${aspectExpr}); }
  .slides-inner { position: relative; width: 100%; padding-top: ${ratio}; }
  .slide { position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: ${bg}; color: ${fg}; border-radius: 12px; padding: 60px 80px; font-size: ${opts.fontSize}px; line-height: 1.7; overflow: auto; display: none; box-shadow: 0 20px 60px rgba(0,0,0,0.3); }
  .slide.active { display: block; }
  .slide h1 { font-size: 2.2em; margin-bottom: 0.6em; color: ${accent}; }
  .slide h2 { font-size: 1.6em; margin: 0.8em 0 0.4em; color: ${accent}; }
  .slide h3 { font-size: 1.2em; margin: 0.6em 0 0.3em; color: ${accent}; }
  .slide p { margin: 0.5em 0; }
  .slide ul, .slide ol { margin: 0.5em 0; padding-left: 2em; }
  .slide li { margin: 0.3em 0; }
  .slide code { background: rgba(0,0,0,0.1); padding: 2px 6px; border-radius: 4px; font-family: Consolas, monospace; font-size: 0.9em; }
  .slide pre { padding: 16px 20px; border-radius: 8px; overflow-x: auto; margin: 1em 0; }
  .slide pre code { background: none; padding: 0; font-size: 0.85em; }
${codeBlockCss}
  .slide blockquote { border-left: 4px solid ${accent}; padding-left: 16px; color: ${darkQuoteColor}; margin: 0.8em 0; }
  .slide table { border-collapse: collapse; width: 100%; margin: 0.8em 0; }
  .slide th, .slide td { border: 1px solid ${darkBorder}; padding: 8px 12px; text-align: left; }
  .slide th { background: ${darkBlockBg}; }
  .slide hr { border: none; border-top: 2px solid ${accent}; margin: 1em 0; }
  .slide a { color: ${accent}; }
  .slide img { max-width: 100%; height: auto; }
  .slide-num { position: absolute; bottom: 24px; right: 32px; font-size: 0.8em; color: ${darkFooterColor}; }
  .toolbar { position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); display: flex; gap: 8px; background: ${darkToolbarBg}; padding: 8px 16px; border-radius: 24px; backdrop-filter: blur(8px); z-index: 100; }
  .toolbar button { background: transparent; border: 1px solid ${darkToolbarBtnBorder}; color: ${darkToolbarBtnColor}; padding: 4px 12px; border-radius: 12px; cursor: pointer; font-size: 14px; }
  .toolbar button:hover { background: rgba(128,128,128,0.2); }
  .toolbar .counter { color: ${darkToolbarCounter}; padding: 4px 12px; font-size: 14px; }`

  const slideCount = slidesHtml.split('<section').length - 1

  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>LitoBox Slides</title>
<style>
${css}
</style>
</head>
<body>
<div class="standalone">
  <div class="slides-wrapper">
    <div class="slides-inner" id="slides">${slidesHtml}</div>
  </div>
</div>
<div class="toolbar">
  <button onclick="prev()">&larr; 上一页</button>
  <span class="counter"><span id="cur">1</span> / <span id="total">${slideCount}</span></span>
  <button onclick="next()">下一页 &rarr;</button>
</div>
<script>
(function(){
  var slides = document.querySelectorAll('.slide');
  var idx = 0;
  var curEl = document.getElementById('cur');
  function show(){
    slides.forEach(function(s){ s.classList.remove('active'); });
    slides[idx].classList.add('active');
    curEl.textContent = idx + 1;
  }
  function prev(){ if(idx > 0){ idx--; show(); } }
  function next(){ if(idx < slides.length - 1){ idx++; show(); } }
  document.addEventListener('keydown', function(e){
    if(e.key === 'ArrowLeft' || e.key === 'ArrowUp') prev();
    if(e.key === 'ArrowRight' || e.key === ' ' || e.key === 'Enter' || e.key === 'PageDown'){ e.preventDefault(); next(); }
    if(e.key === 'Home'){ idx = 0; show(); }
    if(e.key === 'End'){ idx = slides.length - 1; show(); }
  });
  show();
})();
<\/script>
</body>
</html>`
}

// ============ 示例 ============
const loadExample = () => {
  source.value = `# 快速上手

欢迎使用 **LitoBox 幻灯片**！

---

# 支持的内容

- 文本、**粗体**、*斜体*
- 列表和表格
- \`行内代码\` 和代码块
- 引用块、分隔线、链接

---

# 代码块

\`\`\`python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

print(fibonacci(10))
\`\`\`

---

# 快捷键

| 按键 | 功能 |
|------|------|
| ← → | 翻页 |
| Space | 下一页 |
| F5 | 预览 |
| ESC | 退出 |

---

# 开始吧 ✨

用 Markdown 写你的下一份演示！`
}
</script>

<style scoped>
.slide-container .split-layout {
  display: grid;
  grid-template-columns: 1fr 380px;
  gap: 16px;
}

.slide-container .split-pane {
  display: flex;
  flex-direction: column;
  min-height: 300px;
}

.slide-container .pane-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 8px;
}

.slide-container .source-input textarea {
  font-family: 'Consolas', 'Menlo', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.slide-container .source-footer {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 6px;
}

.slide-container .preview-pane {
  border-left: 1px solid var(--border-color);
  padding-left: 16px;
}

.slide-container .slide-thumb-wrap {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  height: 230px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.slide-container .slide-thumb {
  position: relative;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-primary);
}

.slide-container .slide-frame {
  width: 100%;
  height: 100%;
  padding: 24px 32px;
  overflow: auto;
  font-size: 14px;
  line-height: 1.6;
}

.slide-container .slide-frame :deep(h1) { font-size: 1.8em; color: var(--accent-cyan); margin-bottom: 0.4em; }
.slide-container .slide-frame :deep(h2) { font-size: 1.4em; color: var(--accent-cyan); margin: 0.5em 0; }
.slide-container .slide-frame :deep(h3) { font-size: 1.2em; color: var(--accent-cyan); margin: 0.4em 0; }
.slide-container .slide-frame :deep(pre) { background: rgba(0,0,0,0.1); padding: 8px 12px; border-radius: 4px; font-size: 12px; }
.slide-container .slide-frame :deep(code) { font-family: Consolas, monospace; background: rgba(0,0,0,0.1); padding: 1px 4px; border-radius: 3px; }
.slide-container .slide-frame :deep(pre code) { background: none; padding: 0; }
.slide-container .slide-frame :deep(table) { border-collapse: collapse; width: 100%; }
.slide-container .slide-frame :deep(th),
.slide-container .slide-frame :deep(td) { border: 1px solid var(--border-color); padding: 4px 8px; }
.slide-container .slide-frame :deep(blockquote) { border-left: 3px solid var(--accent-cyan); padding-left: 8px; color: var(--text-secondary); margin: 0.5em 0; }
.slide-container .slide-frame :deep(hr) { border: none; border-top: 1px solid var(--border-color); margin: 0.5em 0; }

.slide-container .thumb-nav {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  margin-top: 10px;
}

/* ========== 全屏预览遮罩 ========== */
.preview-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.92);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
}

.preview-overlay.light { background: rgba(200, 200, 200, 0.95); }

.preview-toolbar {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  padding: 6px 20px;
  backdrop-filter: blur(8px);
  font-size: 13px;
  color: #aaa;
}

.preview-overlay.light .preview-toolbar {
  background: rgba(0, 0, 0, 0.08);
  color: #333;
}

.slide-full {
  background: #1a1a2e;
  color: #e0e0e0;
  border-radius: 12px;
  box-shadow: 0 40px 100px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.slide-full.light {
  background: #ffffff;
  color: #222;
}

.slide-full.ratio-169 {
  width: min(90vw, calc(90vh * 16 / 9));
  height: min(50.625vw, 90vh);
}

.slide-full.ratio-43 {
  width: min(80vw, calc(80vh * 4 / 3));
  height: min(60vw, 80vh);
}

.slide-content {
  flex: 1;
  padding: 48px 64px;
  overflow: auto;
  line-height: 1.7;
}

.slide-content :deep(h1) { font-size: 2.2em; color: #00d4ff; margin-bottom: 0.6em; }
.slide-content :deep(h2) { font-size: 1.6em; color: #00d4ff; margin: 0.8em 0 0.4em; }
.slide-content :deep(h3) { font-size: 1.3em; color: #00d4ff; margin: 0.6em 0; }
.slide-content :deep(pre) { background: #161b22; padding: 16px 20px; border-radius: 8px; overflow-x: auto; margin: 1em 0; font-size: 0.9em; }
.slide-content :deep(code) { background: rgba(0,0,0,0.2); padding: 2px 8px; border-radius: 4px; font-family: Consolas, monospace; }
.slide-content :deep(pre code) { background: none; padding: 0; }
.slide-content :deep(blockquote) { border-left: 4px solid #00d4ff; padding-left: 16px; color: #999; margin: 0.8em 0; }
.slide-content :deep(table) { border-collapse: collapse; width: 100%; margin: 0.8em 0; }
.slide-content :deep(th), .slide-content :deep(td) { border: 1px solid #444; padding: 10px 16px; text-align: left; }
.slide-content :deep(th) { background: #2a2a3e; }
.slide-content :deep(hr) { border: none; border-top: 2px solid #00d4ff; margin: 1em 0; }
.slide-content :deep(a) { color: #00d4ff; }
.slide-content :deep(img) { max-width: 100%; height: auto; }

.slide-full.light .slide-content :deep(pre) { background: #f6f8fa; }
.slide-full.light .slide-content :deep(code) { background: rgba(0,0,0,0.08); }
.slide-full.light .slide-content :deep(th) { background: #f0f0f0; }
.slide-full.light .slide-content :deep(th),
.slide-full.light .slide-content :deep(td) { border-color: #ddd; }
.slide-full.light .slide-content :deep(blockquote) { color: #555; }

.slide-footer {
  padding: 12px 24px;
  font-size: 12px;
  color: #555;
  text-align: right;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.slide-full.light .slide-footer {
  border-top-color: #eee;
  color: #aaa;
}
</style>
