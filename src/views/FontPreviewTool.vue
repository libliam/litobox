<template>
  <div class="tool-container fill-height">
    <!-- 预览设置 -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">预览设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group grow">
            <span class="group-label">预览文本</span>
            <el-input v-model="previewText" size="small" placeholder="输入预览文本..." />
          </div>
          <div class="action-group">
            <span class="group-label">字号</span>
            <el-input-number v-model="fontSize" size="small" :min="10" :max="120" :step="2" controls-position="right" style="width: 110px" />
          </div>
          <div class="action-group">
            <span class="group-label">粗细</span>
            <el-select v-model="fontWeight" size="small" style="width: 110px">
              <el-option label="细体" value="300" />
              <el-option label="常规" value="400" />
              <el-option label="中等" value="500" />
              <el-option label="粗体" value="700" />
              <el-option label="特粗" value="900" />
            </el-select>
          </div>
          <div class="action-group">
            <span class="group-label">搜索</span>
            <el-input v-model="searchFont" size="small" placeholder="过滤字体名..." clearable style="width: 160px" />
          </div>
          <div class="action-group">
            <span class="group-label">仅显示可用</span>
            <el-switch v-model="onlyAvailable" size="small" />
          </div>
        </div>
        <div class="font-count">
          共 {{ availableFonts.length }} 个字体可用 / {{ detectedFonts.length }} 个已检测
        </div>
      </div>
    </div>

    <!-- 字体列表 -->
    <div class="tool-card font-list-card">
      <div class="card-body">
        <div v-if="displayFonts.length === 0" class="empty-tip">未找到匹配的字体</div>
        <div v-else class="font-grid">
          <div
            v-for="font in displayFonts"
            :key="font.name"
            class="font-item"
            :class="{ unavailable: !font.available }"
            @click="copyFontName(font.name)"
          >
            <div class="font-preview" :style="fontStyle(font)">
              {{ previewText || 'The quick brown fox 你好世界' }}
            </div>
            <div class="font-info">
              <span class="font-name">{{ font.name }}</span>
              <el-tag v-if="!font.available" size="small" type="info" effect="plain">未安装</el-tag>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'

// 常用字体列表（Windows 系统自带 + 常见编程/设计字体）
const FONT_LIST: string[] = [
  // 西文无衬线
  'Arial', 'Arial Black', 'Arial Narrow', 'Helvetica', 'Helvetica Neue',
  'Segoe UI', 'Segoe UI Light', 'Segoe UI Semibold', 'Segoe UI Symbol',
  'Tahoma', 'Trebuchet MS', 'Verdana', 'Geneva', 'Impact',
  'Roboto', 'Roboto Mono', 'Open Sans', 'Lato', 'Montserrat', 'Poppins',
  'Inter', 'Noto Sans', 'Noto Sans SC', 'Noto Sans JP', 'Noto Sans KR',
  // 西文衬线
  'Times New Roman', 'Georgia', 'Palatino Linotype', 'Book Antiqua',
  'Garamond', 'Cambria', 'Cambria Math', 'Constantia', 'Didot',
  'Noto Serif', 'Noto Serif SC',
  // 等宽字体
  'Consolas', 'Courier New', 'Courier', 'Lucida Console', 'Monaco',
  'Menlo', 'Cascadia Code', 'Cascadia Mono', 'Fira Code', 'Fira Mono',
  'Source Code Pro', 'JetBrains Mono', 'Inconsolata', 'Anonymous Pro',
  'DejaVu Sans Mono', 'Liberation Mono',
  // 中文字体
  'SimSun', '宋体', 'NSimSun', '新宋体', 'SimHei', '黑体',
  'Microsoft YaHei', '微软雅黑', 'Microsoft YaHei UI',
  'KaiTi', '楷体', 'FangSong', '仿宋', 'STXihei', '华文细黑',
  'STKaiti', '华文楷体', 'STFangsong', '华文仿宋', 'STSong', '华文宋体',
  'STZhongsong', '华文中宋', 'STHupo', '华文琥珀', 'STLiti', '华文隶书',
  'STXingkai', '华文行楷', 'STXinwei', '华文新魏', 'STCaiyun', '华文彩云',
  'LiSu', '隶书', 'YouYuan', '幼圆', 'MingLiU', '细明体', 'PMingLiU',
  'DFKai-SB', '標楷體', 'PingFang SC', 'PingFang TC', 'PingFang HK',
  'Hiragino Sans GB', 'Source Han Sans SC', 'Source Han Serif SC',
  'WenQuanYi Micro Hei', 'WenQuanYi Zen Hei',
  // 手写/装饰
  'Comic Sans MS', 'Brush Script MT', 'Script MT Bold', 'French Script MT',
  'Edwardian Script ITC', 'Lucida Handwriting', 'Curlz MT',
  // 符号
  'Wingdings', 'Wingdings 2', 'Wingdings 3', 'Webdings', 'Symbol',
  'MT Extra', 'Marlett',
  // 其他
  'Century', 'Century Gothic', 'Century Schoolbook', 'Gill Sans',
  'Gill Sans MT', 'Calibri', 'Candara', 'Corbel', 'Optima',
  'Baskerville', 'Bodoni MT', 'Rockwell', 'Rockwell Condensed',
  'Cooper Black', 'Stencil', 'Playbill', 'Onyx', 'Wide Latin'
]

interface FontInfo {
  name: string
  available: boolean
}

const previewText = ref('The quick brown fox 你好世界 123')
const fontSize = ref(24)
const fontWeight = ref('400')
const searchFont = ref('')
const onlyAvailable = ref(true)
const detectedFonts = ref<FontInfo[]>([])

// 检测字体是否可用
const checkFontAvailable = (font: string): boolean => {
  const testString = 'mmmmmmmmmmlli'
  const baseFonts = ['monospace', 'sans-serif', 'serif']
  const span = document.createElement('span')
  span.style.fontSize = '72px'
  span.style.position = 'absolute'
  span.style.left = '-9999px'
  span.style.visibility = 'hidden'
  span.style.fontFamily = 'monospace'
  span.textContent = testString
  document.body.appendChild(span)
  const baseWidth = span.offsetWidth
  const baseHeight = span.offsetHeight
  let available = false
  for (const base of baseFonts) {
    span.style.fontFamily = `'${font}', ${base}`
    if (span.offsetWidth !== baseWidth || span.offsetHeight !== baseHeight) {
      available = true
      break
    }
  }
  document.body.removeChild(span)
  return available
}

onMounted(() => {
  // 异步检测，避免阻塞 UI
  detectedFonts.value = FONT_LIST.map(name => ({ name, available: false }))
  let i = 0
  const detectNext = () => {
    if (i < FONT_LIST.length) {
      const name = FONT_LIST[i]
      detectedFonts.value[i].available = checkFontAvailable(name)
      i++
      requestAnimationFrame(detectNext)
    }
  }
  requestAnimationFrame(detectNext)
})

const availableFonts = computed(() => detectedFonts.value.filter(f => f.available))

const displayFonts = computed(() => {
  let list = detectedFonts.value
  if (onlyAvailable.value) {
    list = list.filter(f => f.available)
  }
  if (searchFont.value.trim()) {
    const kw = searchFont.value.trim().toLowerCase()
    list = list.filter(f => f.name.toLowerCase().includes(kw))
  }
  return list
})

const fontStyle = (font: FontInfo) => ({
  fontFamily: `'${font.name}', sans-serif`,
  fontSize: `${fontSize.value}px`,
  fontWeight: fontWeight.value
})

const copyFontName = async (name: string) => {
  try {
    await navigator.clipboard.writeText(name)
    ElMessage.success(`已复制字体名: ${name}`)
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.tool-container {
  padding: 20px;
  overflow-y: auto;
}

.action-group.grow {
  flex: 1;
  min-width: 200px;
}

.font-count {
  margin-top: 12px;
  font-size: 12px;
  color: var(--text-muted);
}

.font-list-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.font-list-card .card-body {
  flex: 1;
  overflow-y: auto;
}

.font-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
}

.font-item {
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-input);
  cursor: pointer;
  transition: border-color 0.2s, transform 0.1s;
}

.font-item:hover {
  border-color: var(--accent-cyan);
  transform: translateY(-1px);
}

.font-item:active {
  transform: translateY(0);
}

.font-item.unavailable {
  opacity: 0.4;
}

.font-preview {
  color: var(--text-primary);
  word-break: break-all;
  line-height: 1.4;
  min-height: 40px;
  margin-bottom: 8px;
}

.font-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid var(--border-color);
  padding-top: 8px;
}

.font-name {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.empty-tip {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
