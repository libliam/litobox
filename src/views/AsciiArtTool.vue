<template>
  <div class="tool-container fill-height">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="ascii-tabs">
        <el-tab-pane label="文本转 ASCII" name="text" />
        <el-tab-pane label="图片转 ASCII" name="image" />
      </el-tabs>
    </div>

    <!-- Tab 1: 文本转 ASCII -->
    <div v-if="activeTab === 'text'" class="tool-card">
      <div class="card-header">
        <span class="card-title">设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">填充风格</div>
            <el-select v-model="fillStyle" size="small" style="width: 130px">
              <el-option label="像素 #" value="#" />
              <el-option label="方块 █" value="█" />
              <el-option label="深灰 ▓" value="▓" />
              <el-option label="中灰 ▒" value="▒" />
              <el-option label="浅灰 ░" value="░" />
              <el-option label="星点 *" value="*" />
              <el-option label="@ 符" value="@" />
              <el-option label="自定义" value="custom" />
            </el-select>
            <el-input
              v-if="fillStyle === 'custom'"
              v-model="customFill"
              size="small"
              placeholder="单个字符"
              maxlength="1"
              style="width: 80px; margin-left: 8px"
            />
          </div>
          <div class="action-group">
            <div class="group-label">字间距</div>
            <el-switch v-model="charSpacing" size="small" />
          </div>
          <div class="action-group">
            <div class="group-label">反转</div>
            <el-switch v-model="textInvert" size="small" />
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'text'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">文本输入</span>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handlePaste('text')">粘贴</el-button>
          <el-button size="small" @click="tabState.text.input = ''">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-input
          v-model="tabState.text.input"
          type="textarea"
          :rows="3"
          placeholder="输入英文、数字（中文将用占位符）"
          @input="debouncedExecText"
        />
      </div>
    </div>

    <div v-if="activeTab === 'text'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">输出</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>5×7 像素字库，支持字母、数字、常用符号</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" @click="handleCopy('text')">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <pre class="ascii-output">{{ tabState.text.output }}</pre>
      </div>
    </div>

    <!-- Tab 2: 图片转 ASCII -->
    <div v-if="activeTab === 'image'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">输出模式</div>
            <el-select v-model="outputMode" size="small" style="width: 140px">
              <el-option label="灰度字符" value="ascii" />
              <el-option label="彩色 HTML" value="color" />
              <el-option label="盲文 Braille" value="braille" />
              <el-option label="半角方块" value="halfblock" />
            </el-select>
          </div>
          <div v-if="outputMode === 'ascii' || outputMode === 'color'" class="action-group">
            <div class="group-label">字符集</div>
            <el-select v-model="charSet" size="small" style="width: 140px">
              <el-option label="标准灰度" value="standard" />
              <el-option label="细粒度" value="fine" />
              <el-option label="简单" value="simple" />
              <el-option label="区块" value="block" />
              <el-option label="数字" value="number" />
              <el-option label="字母" value="alpha" />
              <el-option label="自定义" value="custom" />
            </el-select>
            <el-input
              v-if="charSet === 'custom'"
              v-model="customChars"
              size="small"
              placeholder="从暗到亮"
              style="width: 120px; margin-left: 8px"
            />
          </div>
          <div class="action-group">
            <div class="group-label">宽度</div>
            <el-input-number v-model="imgWidth" :min="10" :max="400" size="small" style="width: 110px" />
          </div>
          <div class="action-group">
            <div class="group-label">反转</div>
            <el-switch v-model="invert" size="small" />
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="handleImageConvert">转换</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'image'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片预览</span>
      </div>
      <div class="card-body">
        <div
          class="image-drop-zone"
          @click="fileInputRef?.click()"
          @dragover.prevent
          @drop.prevent="handleDrop"
        >
          <input
            ref="fileInputRef"
            type="file"
            accept="image/*"
            style="display: none"
            @change="handleFileSelect"
          />
          <img v-if="imageUrl" :src="imageUrl" class="preview-img" />
          <div v-else class="drop-hint">
            <el-icon :size="40" color="var(--text-tertiary)"><UploadFilled /></el-icon>
            <p>点击或拖拽图片到此处</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'image'" class="tool-card">
      <div class="card-header">
        <span class="card-title">ASCII 输出</span>
        <div class="card-actions">
          <el-button size="small" @click="handleCopyImage">复制</el-button>
        </div>
      </div>
      <div class="card-body">
        <!-- 彩色 HTML 模式：用 div 渲染 -->
        <div
          v-if="outputMode === 'color' && tabState.image.output"
          class="ascii-output color-output"
          v-html="tabState.image.output"
        ></div>
        <!-- 纯文本模式 -->
        <pre v-else class="ascii-output image-output">{{ tabState.image.output }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, UploadFilled } from '@element-plus/icons-vue'
import { useToolboxStore } from '@/store'
import { ASCII_FONT } from '@/utils/asciiFont'

const store = useToolboxStore()
const activeTab = ref('text')

const tabState = reactive<Record<string, { input: string; output: string; error: string }>>({
  text: { input: '', output: '', error: '' },
  image: { input: '', output: '', error: '' }
})

// ============ 文本转 ASCII ============
const fillStyle = ref('#')
const customFill = ref('█')
const charSpacing = ref(true)
const textInvert = ref(false)

const activeFill = computed(() => {
  if (fillStyle.value === 'custom') return customFill.value || '█'
  return fillStyle.value
})

let textTimer: ReturnType<typeof setTimeout> | null = null
const debouncedExecText = () => {
  if (textTimer) clearTimeout(textTimer)
  textTimer = setTimeout(execText, 300)
}

// 填充风格 / 反转 / 字间距变化时重新渲染
watch([activeFill, charSpacing, textInvert], () => execText())

const execText = () => {
  const input = tabState.text.input
  if (!input.trim()) {
    tabState.text.output = ''
    return
  }
  const fill = activeFill.value || '#'
  const blank = textInvert.value ? fill : ' '
  const marked = textInvert.value ? ' ' : fill
  const lines = input.split('\n')
  const result = lines.map(line => renderTextLine(line, marked, blank)).join('\n')
  tabState.text.output = result
  if (input.trim()) addHistory('文本转ASCII')
}

const renderTextLine = (line: string, fill: string, blank: string): string => {
  const height = 7
  const rows: string[] = Array(height).fill('')
  const sep = charSpacing.value ? ' ' : ''
  for (const ch of line.toUpperCase()) {
    const glyph = ASCII_FONT[ch] || ASCII_FONT['?']
    for (let i = 0; i < height; i++) {
      // 把字库中的 # 替换为填充字符，空格替换为空白字符
      const rendered = glyph[i].replace(/#/g, fill).replace(/ /g, blank)
      rows[i] += rendered + sep
    }
  }
  return rows.join('\n')
}

// ============ 图片转 ASCII ============
const outputMode = ref('ascii')
const charSet = ref('standard')
const customChars = ref('')
const imgWidth = ref(80)
const invert = ref(false)
const imageUrl = ref('')
const fileInputRef = ref<HTMLInputElement>()

const CHAR_SETS: Record<string, string> = {
  standard: '@%#*+=-:. ',
  fine: '$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,"^`\'. ',
  simple: '#. ',
  block: '█▓▒░ ',
  number: '0123456789 ',
  alpha: 'abcdefghijklmnopqrstuvwxyz '
}

const activeChars = computed(() => {
  if (charSet.value === 'custom') return customChars.value || CHAR_SETS.standard
  return CHAR_SETS[charSet.value] || CHAR_SETS.standard
})

const handleFileSelect = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) loadImage(file)
}

const handleDrop = (e: DragEvent) => {
  const file = e.dataTransfer?.files?.[0]
  if (file && file.type.startsWith('image/')) loadImage(file)
}

const loadImage = (file: File) => {
  const reader = new FileReader()
  reader.onload = () => {
    imageUrl.value = reader.result as string
    // 图片加载后自动转换一次
    setTimeout(() => handleImageConvert(true), 0)
  }
  reader.readAsDataURL(file)
}

// 把像素数据按模式渲染
const handleImageConvert = (silent = false) => {
  if (!imageUrl.value) {
    if (!silent) ElMessage.warning('请先选择图片')
    return
  }

  if ((outputMode.value === 'ascii' || outputMode.value === 'color') && activeChars.value.length < 2) {
    if (!silent) ElMessage.warning('字符集至少需要 2 个字符')
    return
  }

  const img = new Image()
  img.onload = () => {
    try {
      let result = ''
      switch (outputMode.value) {
        case 'ascii':
          result = renderAscii(img)
          break
        case 'color':
          result = renderColor(img)
          break
        case 'braille':
          result = renderBraille(img)
          break
        case 'halfblock':
          result = renderHalfBlock(img)
          break
      }
      tabState.image.output = result
      if (!silent) {
        addHistory('图片转ASCII-' + outputMode.value)
        ElMessage.success('转换完成')
      }
    } catch (e: any) {
      if (!silent) ElMessage.error('转换失败: ' + (e.message || e))
    }
  }
  img.onerror = () => {
    if (!silent) ElMessage.error('图片加载失败')
  }
  img.src = imageUrl.value
}

// 参数变化时自动重新转换（防抖）
let autoConvertTimer: ReturnType<typeof setTimeout> | null = null
const scheduleAutoConvert = () => {
  if (!imageUrl.value) return
  if (autoConvertTimer) clearTimeout(autoConvertTimer)
  autoConvertTimer = setTimeout(() => handleImageConvert(true), 200)
}
watch([outputMode, charSet, customChars, imgWidth, invert], scheduleAutoConvert)

// 计算单像素灰度（0-255），应用反转
const getGray = (r: number, g: number, b: number, a: number): number => {
  let gray = (0.299 * r + 0.587 * g + 0.114 * b) * (a / 255)
  if (invert.value) gray = 255 - gray
  return gray
}

// 模式1: 灰度字符
const renderAscii = (img: HTMLImageElement): string => {
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  const width = imgWidth.value
  const height = Math.round((img.height / img.width) * width / 2)
  canvas.width = width
  canvas.height = height
  ctx.drawImage(img, 0, 0, width, height)
  const data = ctx.getImageData(0, 0, width, height).data
  const chars = activeChars.value
  let result = ''
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4
      const gray = getGray(data[i], data[i + 1], data[i + 2], data[i + 3])
      const idx = Math.floor((gray / 255) * (chars.length - 1))
      result += chars[idx]
    }
    result += '\n'
  }
  return result.trimEnd()
}

// 模式2: 彩色 HTML
const renderColor = (img: HTMLImageElement): string => {
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  const width = imgWidth.value
  const height = Math.round((img.height / img.width) * width / 2)
  canvas.width = width
  canvas.height = height
  ctx.drawImage(img, 0, 0, width, height)
  const data = ctx.getImageData(0, 0, width, height).data
  const chars = activeChars.value
  let result = '<div style="font-family:Consolas,monospace;font-size:12px;line-height:1;white-space:pre;">'
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4
      const r = data[i], g = data[i + 1], b = data[i + 2], a = data[i + 3]
      const gray = getGray(r, g, b, a)
      const idx = Math.floor((gray / 255) * (chars.length - 1))
      // 用原色着色，保留彩色信息
      result += `<span style="color:rgb(${r},${g},${b})">${chars[idx]}</span>`
    }
    result += '<br/>'
  }
  result += '</div>'
  return result
}

// 模式3: Braille 盲文（2x4 像素块 → 1 个盲文字符）
const renderBraille = (img: HTMLImageElement): string => {
  const cellW = 2, cellH = 4
  const width = imgWidth.value
  const height = Math.round((img.height / img.width) * width / 2)
  const cols = Math.ceil(width / cellW)
  const rows = Math.ceil(height / cellH)

  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  canvas.width = width
  canvas.height = height
  ctx.drawImage(img, 0, 0, width, height)
  const data = ctx.getImageData(0, 0, width, height).data

  // 盲文点的 bit 位映射（Unicode U+2800 起）
  // 点排列: 1(0,0) 4(1,0)
  //          2(0,1) 5(1,1)
  //          3(0,2) 6(1,2)
  //          7(0,3) 8(1,3)
  const dotMap = [
    [0, 1 << 0], [1, 1 << 3],
    [0, 1 << 1], [1, 1 << 4],
    [0, 1 << 2], [1, 1 << 5],
    [0, 1 << 6], [1, 1 << 7],
  ]

  let result = ''
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      let bits = 0
      // 按顺序遍历 8 个点，d>>1 得到行偏移 (0,0,1,1,2,2,3,3)
      for (let d = 0; d < dotMap.length; d++) {
        const [dx, bit] = dotMap[d]
        const px = col * cellW + dx
        const py = row * cellH + (d >> 1)
        if (px < width && py < height) {
          const i = (py * width + px) * 4
          const gray = getGray(data[i], data[i + 1], data[i + 2], data[i + 3])
          if (gray > 128) bits |= bit
        }
      }
      result += String.fromCharCode(0x2800 + bits)
    }
    result += '\n'
  }
  return result.trimEnd()
}

// 模式4: 半角方块（上半 ▀ / 下半 ▄ / 全 █ / 空）
// 两个垂直像素合并到一个字符，垂直分辨率翻倍
const renderHalfBlock = (img: HTMLImageElement): string => {
  const width = imgWidth.value
  // 半角方块一个字符占 2 行像素，所以高度不除以 2
  const height = Math.round((img.height / img.width) * width)

  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')!
  canvas.width = width
  canvas.height = height
  ctx.drawImage(img, 0, 0, width, height)
  const data = ctx.getImageData(0, 0, width, height).data

  const threshold = 128
  let result = ''
  for (let y = 0; y < height; y += 2) {
    for (let x = 0; x < width; x++) {
      const iTop = (y * width + x) * 4
      const topGray = getGray(data[iTop], data[iTop + 1], data[iTop + 2], data[iTop + 3])
      const topOn = topGray > threshold

      let bottomOn = false
      if (y + 1 < height) {
        const iBot = ((y + 1) * width + x) * 4
        const botGray = getGray(data[iBot], data[iBot + 1], data[iBot + 2], data[iBot + 3])
        bottomOn = botGray > threshold
      }

      let ch = ' '
      if (topOn && bottomOn) ch = '█'
      else if (topOn && !bottomOn) ch = '▀'
      else if (!topOn && bottomOn) ch = '▄'
      result += ch
    }
    result += '\n'
  }
  return result.trimEnd()
}

// ============ 通用方法 ============
const handlePaste = async (tab: string) => {
  try {
    tabState[tab].input = await navigator.clipboard.readText()
  } catch (e: any) {
    ElMessage.error('粘贴失败: ' + (e.message || e))
  }
}

const handleCopy = async (tab: string) => {
  try {
    await navigator.clipboard.writeText(tabState[tab].output)
    ElMessage.success('已复制到剪贴板')
  } catch (e: any) {
    ElMessage.error('复制失败: ' + (e.message || e))
  }
}

// 图片复制：color 模式复制 HTML，其他复制纯文本
const handleCopyImage = async () => {
  try {
    if (outputMode.value === 'color') {
      // 用 Clipboard API 写入 HTML
      const html = tabState.image.output
      const blob = new Blob([html], { type: 'text/html' })
      await navigator.clipboard.write([new ClipboardItem({ 'text/html': blob })])
      ElMessage.success('已复制彩色 HTML（粘贴到富文本编辑器）')
    } else {
      await navigator.clipboard.writeText(tabState.image.output)
      ElMessage.success('已复制到剪贴板')
    }
  } catch (e: any) {
    // 回退到纯文本复制
    try {
      await navigator.clipboard.writeText(tabState.image.output)
      ElMessage.success('已复制（纯文本）')
    } catch {
      ElMessage.error('复制失败: ' + (e.message || e))
    }
  }
}

const addHistory = (action: string) => {
  const state = tabState[activeTab.value]
  store.addHistory({
    tool: 'asciiArt',
    action,
    inputPreview: state.input.slice(0, 50),
    outputPreview: state.output.slice(0, 50),
    inputFull: state.input,
    outputFull: state.output,
  })
}

watch(activeTab, () => {})
</script>

<style scoped>
.ascii-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .ascii-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.ascii-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.ascii-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.ascii-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.ascii-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.ascii-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
}

.ascii-output {
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.1;
  white-space: pre;
  overflow: auto;
  max-height: 60vh;
  margin: 0;
  color: var(--text-primary);
  background: var(--bg-secondary);
  padding: 12px;
  border-radius: 4px;
}

.image-output {
  letter-spacing: 0;
}

.color-output {
  line-height: 1;
  background: #000;
  padding: 12px;
  border-radius: 4px;
  overflow: auto;
  max-height: 60vh;
}

.color-output :deep(span) {
  display: inline;
}

.image-drop-zone {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  min-height: 200px;
  width: 100%;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: border-color 0.2s;
  overflow: hidden;
}

.image-drop-zone:hover {
  border-color: var(--accent-cyan);
}

.drop-hint {
  text-align: center;
  color: var(--text-tertiary);
}

.preview-img {
  display: block;
  max-width: 100%;
  max-height: 300px;
  margin: 0 auto;
}
</style>
