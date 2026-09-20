<template>
  <div class="tool-container svg-tool">
    <!-- 输入区 -->
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">SVG 源码</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>粘贴 SVG 代码或上传 .svg 文件</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="header-right">
          <span v-if="input" class="stat-text">{{ input.length }} 字符</span>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">操作</div>
            <div class="group-buttons">
              <el-button size="small" @click="loadSample">
                <el-icon><MagicStick /></el-icon>
                <span>示例</span>
              </el-button>
              <el-button size="small" @click="uploadFile">
                <el-icon><Upload /></el-icon>
                <span>上传</span>
              </el-button>
              <el-button size="small" @click="pasteFromClipboard">
                <el-icon><DocumentCopy /></el-icon>
                <span>粘贴</span>
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">清理</div>
            <div class="group-buttons">
              <el-button size="small" @click="input = ''; output = ''; error = ''">
                <el-icon><Delete /></el-icon>
                <span>清空</span>
              </el-button>
            </div>
          </div>
        </div>

        <div class="editor-section">
          <div class="editor-wrapper">
            <textarea
              v-model="input"
              class="svg-input"
              spellcheck="false"
              placeholder='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">...'
            />
            <div v-if="error" class="error-overlay">
              <el-icon><WarningFilled /></el-icon>
              <span>{{ error }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="svg-tabs">
        <el-tab-pane label="实时预览" name="preview" />
        <el-tab-pane label="优化压缩" name="optimize" />
        <el-tab-pane label="转 PNG" name="convert" />
      </el-tabs>
    </div>

    <!-- Tab 1: 实时预览 -->
    <div v-if="activeTab === 'preview'" class="tool-card preview-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">SVG 预览</span>
        </div>
        <div class="header-right">
          <el-button-group size="small">
            <el-button :disabled="!svgValid" @click="zoomOut">
              <el-icon><ZoomOut /></el-icon>
            </el-button>
            <el-button :disabled="!svgValid" @click="zoomReset">
              {{ Math.round(scale * 100) }}%
            </el-button>
            <el-button :disabled="!svgValid" @click="zoomIn">
              <el-icon><ZoomIn /></el-icon>
            </el-button>
            <el-button :disabled="!svgValid" @click="zoomFit" title="适应窗口">
              <el-icon><FullScreen /></el-icon>
            </el-button>
          </el-button-group>
          <el-divider direction="vertical" />
          <el-select v-model="bgColor" size="small" style="width: 120px">
            <el-option label="透明" value="transparent" />
            <el-option label="白色" value="#ffffff" />
            <el-option label="黑色" value="#000000" />
            <el-option label="浅灰" value="#f5f5f5" />
          </el-select>
        </div>
      </div>
      <div class="card-body preview-body">
        <div
          class="preview-area"
          :style="{ background: bgColor }"
          @wheel.prevent="onWheel"
          ref="previewAreaRef"
        >
          <div v-if="svgValid" class="svg-preview" :style="{ zoom: scale }">
            <div v-html="input" class="svg-inner"></div>
          </div>
          <div v-else class="empty-hint">
            <el-icon :size="48"><Picture /></el-icon>
            <p>输入有效的 SVG 代码以预览</p>
          </div>
        </div>
        <div v-if="svgMeta" class="meta-info">
          <span>📐 {{ svgMeta.width }} × {{ svgMeta.height }}</span>
          <span>📏 原始: {{ svgMeta.viewBox }}</span>
          <span>🔍 {{ Math.round(scale * 100) }}%</span>
        </div>
      </div>
    </div>

    <!-- Tab 2: 优化压缩 -->
    <div v-if="activeTab === 'optimize'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">SVG 优化</span>
          <span class="stat-badge" v-if="optimizeResult">
            <span :class="optimizeResult.savedPct > 0 ? 'positive' : 'negative'">
              {{ optimizeResult.savedPct > 0 ? '−' : '+' }}{{ Math.abs(optimizeResult.savedPct) }}%
            </span>
          </span>
        </div>
        <div class="header-right">
          <el-button type="primary" size="small" :disabled="!input" @click="doOptimize">
            <el-icon><Lightning /></el-icon>
            <span>优化</span>
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">优化选项</div>
            <div class="option-list">
              <el-checkbox v-model="opts.removeComments" label="移除注释" />
              <el-checkbox v-model="opts.removeMetadata" label="移除 metadata" />
              <el-checkbox v-model="opts.indent" label="紧凑输出（移除多余空格）" />
              <el-checkbox v-model="opts.mergePaths" label="合并 <path>（简化 d 属性）" />
            </div>
          </div>
        </div>

        <div v-if="optimizeResult" class="result-section">
          <div class="size-comparison">
            <div class="size-box original">
              <span class="label">原始</span>
              <span class="value">{{ optimizeResult.originalSize }} B</span>
            </div>
            <div class="arrow">→</div>
            <div class="size-box optimized">
              <span class="label">优化后</span>
              <span class="value">{{ optimizeResult.optimizedSize }} B</span>
            </div>
          </div>

          <div class="editor-section">
            <div class="output-toolbar">
              <span class="output-title">优化后 SVG</span>
              <div class="output-actions">
                <el-button size="small" @click="copyText(optimizeResult.code)">
                  <el-icon><CopyDocument /></el-icon>
                  <span>复制</span>
                </el-button>
                <el-button size="small" @click="downloadText(optimizeResult.code, 'optimized.svg')">
                  <el-icon><Download /></el-icon>
                  <span>下载</span>
                </el-button>
                <el-button size="small" @click="input = optimizeResult.code">
                  <el-icon><Edit /></el-icon>
                  <span>替换输入</span>
                </el-button>
              </div>
            </div>
            <textarea class="svg-input output-area" spellcheck="false" :value="optimizeResult.code" readonly />
          </div>
        </div>
      </div>
    </div>

    <!-- Tab 3: 转 PNG -->
    <div v-if="activeTab === 'convert'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">SVG → PNG</span>
        </div>
        <div class="header-right">
          <el-button type="primary" size="small" :disabled="!input" @click="doConvert">
            <el-icon><MagicStick /></el-icon>
            <span>转换</span>
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">尺寸</div>
            <div class="group-buttons dimension-group">
              <el-input-number v-model="convertWidth" :min="10" :max="8000" size="small" controls-position="right" />
              <span class="dim-sep">×</span>
              <el-input-number v-model="convertHeight" :min="10" :max="8000" size="small" controls-position="right" />
              <el-checkbox v-model="keepRatio" label="保持比例" size="small" />
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">背景</div>
            <div class="group-buttons">
              <el-color-picker v-model="convertBg" :show-alpha="true" size="small" />
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">操作</div>
            <div class="group-buttons">
              <el-button size="small" @click="downloadPng">
                <el-icon><Download /></el-icon>
                <span>下载 PNG</span>
              </el-button>
              <el-button size="small" @click="copyPngToClipboard">
                <el-icon><DocumentCopy /></el-icon>
                <span>复制到剪贴板</span>
              </el-button>
            </div>
          </div>
        </div>

        <div v-if="convertPreview" class="preview-section">
          <div class="preview-label">预览（{{ convertWidth }} × {{ convertHeight }} px）</div>
          <div class="preview-area png-preview" :style="{ background: convertBg }">
            <img :src="convertPreview" alt="PNG preview" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { saveFileWithDialog } from '@/utils/fileSaver'
import {
  QuestionFilled, MagicStick, Upload, DocumentCopy, Delete,
  WarningFilled, Picture, Lightning, CopyDocument, Download, Edit,
  ZoomIn, ZoomOut, FullScreen
} from '@element-plus/icons-vue'

const input = ref('')
const output = ref('')
const error = ref('')
const activeTab = ref('preview')
const bgColor = ref('transparent')

// 预览相关
const svgValid = ref(false)
const svgMeta = ref<{ width: number; height: number; viewBox: string } | null>(null)
const scale = ref(1)
const previewAreaRef = ref<HTMLElement | null>(null)

const zoomIn = () => { scale.value = Math.min(scale.value * 1.2, 10) }
const zoomOut = () => { scale.value = Math.max(scale.value / 1.2, 0.1) }
const zoomReset = () => { scale.value = 1 }
const onWheel = (e: WheelEvent) => {
  if (e.deltaY < 0) zoomIn()
  else zoomOut()
}
const zoomFit = () => {
  // ponytail: 适应窗口 = 让 SVG 缩放到刚好放进预览区，取宽高比的较小值
  if (!previewAreaRef.value) return
  const svg = previewAreaRef.value.querySelector('svg')
  if (!svg) return
  const svgW = svg.getBoundingClientRect().width / scale.value
  const svgH = svg.getBoundingClientRect().height / scale.value
  const areaW = previewAreaRef.value.clientWidth - 48
  const areaH = previewAreaRef.value.clientHeight - 48
  if (svgW > 0 && svgH > 0) {
    scale.value = Math.min(areaW / svgW, areaH / svgH, 1)
  }
}

// 优化相关
interface OptimizeResult {
  originalSize: number
  optimizedSize: number
  savedPct: number
  code: string
}
const optimizeResult = ref<OptimizeResult | null>(null)

const opts = ref({
  removeComments: true,
  removeMetadata: true,
  indent: false,
  mergePaths: false
})

// 转换相关
const convertWidth = ref(400)
const convertHeight = ref(400)
const keepRatio = ref(true)
const convertBg = ref('#ffffff00')
const convertPreview = ref('')

// 上传文件
const uploadFile = () => {
  const inputEl = document.createElement('input')
  inputEl.type = 'file'
  inputEl.accept = '.svg,image/svg+xml'
  inputEl.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      input.value = reader.result as string
      ElMessage.success('文件已加载')
    }
    reader.readAsText(file)
  }
  inputEl.click()
}

// 粘贴
const pasteFromClipboard = async () => {
  try {
    const text = await navigator.clipboard.readText()
    if (text) {
      input.value = text
      ElMessage.success('已粘贴剪贴板内容')
    } else {
      ElMessage.warning('剪贴板为空')
    }
  } catch {
    ElMessage.error('无法读取剪贴板')
  }
}

// 示例 SVG
const loadSample = () => {
  input.value = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
  <!-- 示例：渐变圆形 -->
  <defs>
    <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00d4ff"/>
      <stop offset="100%" stop-color="#7b2ff7"/>
    </linearGradient>
  </defs>
  <circle cx="100" cy="100" r="80" fill="url(#grad)"/>
  <text x="100" y="110" font-size="24" fill="white" text-anchor="middle" font-family="sans-serif">LitoBox</text>
</svg>`
  ElMessage.success('已加载示例')
}

// 验证 SVG
const validate = () => {
  error.value = ''
  svgValid.value = false
  svgMeta.value = null
  convertPreview.value = ''
  optimizeResult.value = null
  scale.value = 1

  if (!input.value.trim()) return

  try {
    const parser = new DOMParser()
    const doc = parser.parseFromString(input.value, 'image/svg+xml')
    const parseError = doc.querySelector('parsererror')
    if (parseError) {
      error.value = 'SVG 解析错误，请检查代码'
      return
    }
    const svg = doc.querySelector('svg')
    if (!svg) {
      error.value = '未找到 <svg> 根元素'
      return
    }

    svgValid.value = true

    // 提取尺寸
    const width = parseFloat(svg.getAttribute('width') || '100')
    const height = parseFloat(svg.getAttribute('height') || '100')
    const viewBox = svg.getAttribute('viewBox') || `0 0 ${width} ${height}`
    svgMeta.value = { width, height, viewBox }
  } catch (e: any) {
    error.value = e.message || '解析失败'
  }
}

watch(input, validate, { immediate: true })

// 优化
const doOptimize = () => {
  if (!input.value.trim()) {
    ElMessage.warning('请输入 SVG 代码')
    return
  }

  let code = input.value

  // 1. 移除注释
  if (opts.value.removeComments) {
    code = code.replace(/<!--[\s\S]*?-->/g, '')
  }

  // 2. 移除 metadata
  if (opts.value.removeMetadata) {
    code = code.replace(/<metadata[\s\S]*?<\/metadata>/gi, '')
  }

  // 3. 紧凑输出
  if (opts.value.indent) {
    code = code.replace(/>\s+</g, '><')
    code = code.replace(/\n+/g, '')
    code = code.replace(/\s{2,}/g, ' ')
    code = code.trim()
  } else {
    // 清理多余空行
    code = code.replace(/\n{3,}/g, '\n\n')
  }

  // 4. 简化 path d 属性（移除无用小数点后缀零）
  if (opts.value.mergePaths) {
    code = code.replace(/d="([^"]*)"/g, (_, d) => {
      // 移除路径中不必要的空格和重复的小数点
      return 'd="' + d.replace(/\s+/g, ' ').replace(/(\d)\.0+/g, '$1').trim() + '"'
    })
  }

  const originalSize = new Blob([input.value]).size
  const optimizedSize = new Blob([code]).size
  const savedPct = originalSize > 0 ? Math.round((1 - optimizedSize / originalSize) * 100) : 0

  optimizeResult.value = {
    originalSize,
    optimizedSize,
    savedPct,
    code
  }

  ElMessage.success(savedPct > 0 ? `压缩了 ${savedPct}%` : '已优化（此文件可能已压缩过）')
}

// 内联 SVG 中的外部图片为 base64，避免 canvas 被污染（tainted）
const inlineSvgImages = async (svg: Element) => {
  const images = svg.querySelectorAll('image')
  for (const imgEl of Array.from(images)) {
    const href = imgEl.getAttribute('href') || imgEl.getAttribute('xlink:href') || ''
    if (!href || href.startsWith('data:')) continue
    try {
      const resp = await fetch(href)
      const blob = await resp.blob()
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onload = () => resolve(reader.result as string)
        reader.onerror = reject
        reader.readAsDataURL(blob)
      })
      imgEl.setAttribute('href', dataUrl)
      imgEl.removeAttribute('xlink:href')
    } catch {
      // 无法加载的图片保留原样，可能导致导出失败
    }
  }
}

// 转换为 PNG
const svgToPngBlob = async (): Promise<Blob> => {
  const parser = new DOMParser()
  const doc = parser.parseFromString(input.value, 'image/svg+xml')
  const svg = doc.querySelector('svg')
  if (!svg) throw new Error('无效的 SVG')

  // 确保命名空间正确
  svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
  svg.setAttribute('xmlns:xlink', 'http://www.w3.org/1999/xlink')
  // 移除 script 标签，避免安全问题
  svg.querySelectorAll('script').forEach((s) => s.remove())

  // 内联外部图片，防止 canvas tainted
  await inlineSvgImages(svg)

  const vb = svg.getAttribute('viewBox')?.split(/[\s,]+/)
  let vbX = 0, vbY = 0, vbW = 0, vbH = 0
  if (vb && vb.length === 4) {
    vbX = parseFloat(vb[0])
    vbY = parseFloat(vb[1])
    vbW = parseFloat(vb[2])
    vbH = parseFloat(vb[3])
  }

  // 内容可能超出 viewBox（如 Mermaid mindmap 根节点低于 viewBox 底部），<img> 栅格化会按
  // viewBox 裁剪导致节点/文字被切掉，而预览内联渲染溢出可见，两者表现不一致。
  // 仅在内容超出原 viewBox 时扩展之，保留正常 SVG 自带的边距
  // 注：getBBox 需要元素处于渲染树中，游离文档返回 0，故先挂到屏幕外再计算
  let contentBox: { x0: number; y0: number; x1: number; y1: number } | null = null
  try {
    const probe = document.createElement('div')
    probe.style.cssText = 'position:fixed;left:-9999px;top:-9999px;width:800px;height:600px;'
    document.body.appendChild(probe)
    probe.appendChild(svg)
    const bb = svg.getBBox()
    svg.remove()
    probe.remove()
    if (bb.width > 0 && bb.height > 0) {
      contentBox = { x0: bb.x, y0: bb.y, x1: bb.x + bb.width, y1: bb.y + bb.height }
    }
  } catch {
    // getBBox 不可用时保持原 viewBox
  }
  if (contentBox) {
    const pad = 2
    const c = { x0: contentBox.x0 - pad, y0: contentBox.y0 - pad, x1: contentBox.x1 + pad, y1: contentBox.y1 + pad }
    const v = {
      x0: vbW > 0 ? vbX : c.x0,
      y0: vbH > 0 ? vbY : c.y0,
      x1: vbW > 0 ? vbX + vbW : c.x1,
      y1: vbH > 0 ? vbY + vbH : c.y1,
    }
    // 无 viewBox 的 SVG 直接以内容包围盒作为 viewBox
    if (vbW === 0 || c.x0 < v.x0 || c.y0 < v.y0 || c.x1 > v.x1 || c.y1 > v.y1) {
      vbX = Math.min(c.x0, v.x0)
      vbY = Math.min(c.y0, v.y0)
      vbW = Math.max(c.x1, v.x1) - vbX
      vbH = Math.max(c.y1, v.y1) - vbY
      svg.setAttribute('viewBox', `${vbX} ${vbY} ${vbW} ${vbH}`)
    }
  }

  // width/height 为百分比（如 Mermaid 导出的 width="100%"）时无法作为实际尺寸，
  // 以 viewBox 尺寸为准，避免比例被算成 100 × 422
  const isNum = (s: string | null) => s != null && /^\d+(\.\d+)?(px)?$/.test(s.trim())
  const wAttr = isNum(svg.getAttribute('width')) ? parseFloat(svg.getAttribute('width')!) : 0
  const hAttr = isNum(svg.getAttribute('height')) ? parseFloat(svg.getAttribute('height')!) : 0
  const origW = vbW > 0 ? vbW : (wAttr || 100)
  const origH = vbH > 0 ? vbH : (hAttr || 100)

  // 规整根 svg 尺寸为 viewBox 尺寸：width="100%"（且无 height）的 SVG 作为 <img> 加载时
  // 固有尺寸会退化为默认 150 高、宽度按比例算出（243×150），导致内容被裁剪、文字错位
  if (vbW > 0 && vbH > 0) {
    svg.setAttribute('width', String(vbW))
    svg.setAttribute('height', String(vbH))
  }

  let w = convertWidth.value
  let h = convertHeight.value
  if (keepRatio.value && origW > 0 && origH > 0) {
    const ratio = origH / origW
    h = Math.round(w * ratio)
    convertHeight.value = h
  }

  // ponytail: 用 data URL 而非 Blob URL，某些浏览器对 Blob URL 的 SVG 有更严格的安全限制
  const svgStr = new XMLSerializer().serializeToString(svg)
  const dataUrl = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgStr)

  return await new Promise<Blob>((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas')
        canvas.width = w
        canvas.height = h
        const ctx = canvas.getContext('2d')!

        // 背景色
        if (convertBg.value === '#ffffff00' || convertBg.value === 'transparent') {
          ctx.clearRect(0, 0, w, h)
        } else {
          ctx.fillStyle = convertBg.value
          ctx.fillRect(0, 0, w, h)
        }

        ctx.drawImage(img, 0, 0, w, h)

        canvas.toBlob((blob) => {
          if (blob) {
            resolve(blob)
          } else {
            reject(new Error('PNG 生成失败，SVG 可能包含不支持的元素（如 foreignObject、外部字体等）'))
          }
        }, 'image/png')
      } catch (e: any) {
        reject(new Error(e.message || 'canvas 渲染失败'))
      }
    }
    img.onerror = () => {
      reject(new Error('SVG 图片加载失败'))
    }
    img.src = dataUrl
  })
}

const doConvert = async () => {
  try {
    const blob = await svgToPngBlob()
    const url = URL.createObjectURL(blob)
    convertPreview.value = url
    ElMessage.success('转换完成')
  } catch (e: any) {
    ElMessage.error(e.message || '转换失败')
  }
}

const downloadPng = async () => {
  try {
    const blob = await svgToPngBlob()
    // 弹原生保存对话框（saveFileWithDialog 内部处理成功/取消/降级提示）
    await saveFileWithDialog(blob, `svg_${Date.now()}.png`, 'png')
  } catch (e: any) {
    ElMessage.error(e.message || '下载失败')
  }
}

const copyPngToClipboard = async () => {
  try {
    const blob = await svgToPngBlob()
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      ElMessage.success('已复制图片到剪贴板')
    } else {
      ElMessage.warning('当前浏览器不支持图片剪贴板操作')
    }
  } catch (e: any) {
    ElMessage.error(e.message || '复制失败')
  }
}

// 工具方法
const copyText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const downloadText = async (text: string, filename: string) => {
  try {
    const savedPath = await invoke<string>('save_text_with_dialog', { content: text, filename })
    if (savedPath === 'cancelled') {
      ElMessage.info('已取消保存')
      return
    }
    ElMessage.success(`文件已保存至: ${savedPath}`)
  } catch (e: any) {
    ElMessage.error(e.message || '保存失败')
  }
}

onMounted(() => {
  // 默认加载一个简单示例，方便用户直接看到效果
  if (!input.value) loadSample()
})
</script>

<style scoped>
/* 整体布局：flex 列，填满窗口高度，输入区固定、预览区自适应 */
.svg-tool {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px;
}

.svg-tool > .tool-card {
  margin-bottom: 12px;
  flex-shrink: 0;
}

.svg-tool > .sticky-card {
  position: relative;
  top: auto;
}

.svg-tool > .preview-card {
  flex: 1;
  min-height: 0;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
}

.svg-tool > .preview-card > .card-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 12px 16px;
}

.svg-tabs {
  width: 100%;
  padding: 4px 20px 0;
}

.svg-tabs :deep(.el-tabs__item) {
  letter-spacing: 1px;
  text-transform: uppercase;
}

.editor-section {
  margin-top: 16px;
}

.svg-input {
  width: 100%;
  min-height: 120px;
  max-height: 300px;
  padding: 12px;
  background: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
  tab-size: 2;
}

.svg-input:focus {
  border-color: var(--border-active);
  box-shadow: 0 0 0 2px rgba(0, 212, 255, 0.15);
}

.svg-input.output-area {
  min-height: 180px;
}

.editor-wrapper {
  position: relative;
}

.error-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(255, 77, 79, 0.15);
  color: #ff4d4f;
  border-radius: 4px;
  font-size: 12px;
  pointer-events: none;
}

.stat-text {
  color: var(--text-secondary);
  font-size: 12px;
}

.stat-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  margin-left: 8px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 600;
  background: var(--bg-input);
}

.stat-badge .positive {
  color: #52c41a;
}

.stat-badge .negative {
  color: #ff4d4f;
}

.preview-area {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: auto;
  padding: 24px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-image:
    linear-gradient(45deg, var(--bg-input) 25%, transparent 25%),
    linear-gradient(-45deg, var(--bg-input) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, var(--bg-input) 75%),
    linear-gradient(-45deg, transparent 75%, var(--bg-input) 75%);
  background-size: 20px 20px;
  background-position: 0 0, 0 10px, 10px -10px, 10px 0;
  transition: background 0.2s ease;
}

:deep(html.light) .preview-area {
  background-image:
    linear-gradient(45deg, #d0d7de 25%, transparent 25%),
    linear-gradient(-45deg, #d0d7de 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #d0d7de 75%),
    linear-gradient(-45deg, transparent 75%, #d0d7de 75%);
}

:deep(html.light) .preview-area[style*="transparent"] {
  background-image:
    linear-gradient(45deg, #d0d7de 25%, transparent 25%),
    linear-gradient(-45deg, #d0d7de 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #d0d7de 75%),
    linear-gradient(-45deg, transparent 75%, #d0d7de 75%);
}

.preview-area[style*="transparent"] {
  background-image:
    linear-gradient(45deg, var(--bg-input) 25%, transparent 25%),
    linear-gradient(-45deg, var(--bg-input) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, var(--bg-input) 75%),
    linear-gradient(-45deg, transparent 75%, var(--bg-input) 75%);
}

.svg-preview {
  /* ponytail: 用 zoom 缩放，会影响布局尺寸，overflow:auto 才能正确出滚动条 */
}

.svg-preview :deep(svg) {
  display: block;
  max-width: none;
  max-height: none;
}

.empty-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
}

.meta-info {
  display: flex;
  gap: 16px;
  margin-top: 12px;
  color: var(--text-secondary);
  font-size: 13px;
}

.option-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.option-list :deep(.el-checkbox__label) {
  font-size: 13px;
}

.size-comparison {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.size-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 24px;
  border-radius: 6px;
  min-width: 120px;
}

.size-box.original {
  background: var(--bg-input);
  border: 1px solid var(--border-color);
}

.size-box.optimized {
  background: rgba(82, 196, 26, 0.1);
  border: 1px solid #52c41a;
}

.size-box .label {
  font-size: 12px;
  color: var(--text-secondary);
}

.size-box .value {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 4px;
}

.size-box.optimized .value {
  color: #52c41a;
}

.arrow {
  font-size: 24px;
  color: var(--accent-cyan);
}

.result-section {
  margin-top: 16px;
}

.output-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.output-title {
  font-size: 13px;
  color: var(--text-secondary);
  letter-spacing: 1px;
  text-transform: uppercase;
}

.output-actions {
  display: flex;
  gap: 8px;
}

.preview-section {
  margin-top: 16px;
}

.preview-label {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.png-preview img {
  max-width: 100%;
  max-height: 500px;
  image-rendering: -webkit-optimize-contrast;
}

.dimension-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dim-sep {
  color: var(--text-secondary);
}
</style>
