<script setup lang="ts">
/**
 * D2: 全屏截图选框 + 标注工具（浮层组件）
 *
 * 整体流程：
 * 1. visible=true 时（来自 store.isScreenshotOverlayOpen）
 *    - 如果 store.screenshotDelaySec > 0：先倒计时，期间隐藏浮层，倒计时结束再截
 *    - 否则：先隐藏主窗口（ponytail：可选，当前版本直接截屏不隐藏主窗口，省掉窗口切换抖动）
 *    - 调 invoke('screenshot_capture_fullscreen') 拿到底图 PNG base64
 *    - 显示浮层：Canvas 铺满全屏，底图作为背景层
 * 2. 选框阶段：
 *    - 鼠标按下拖拽产生选框矩形 rect(x,y,w,h)
 *    - 选框外半透明黑色蒙层，选框内清晰
 *    - 选框生成后，支持拖拽调整 8 个控制点重新缩放，拖内部整体移动
 * 3. 标注阶段（选框确定后）：
 *    - 工具栏顶部：矩形/箭头/文字/马赛克/撤销/（颜色+线粗选择器）
 *    - 所有标注记录在 annotations 数组里，撤销 = pop 最后一条
 *    - 马赛克：鼠标绘制的点阵区域，用原图 8x8（可调）平均色块覆盖
 * 4. 完成：
 *    - 「✅ 复制」：选框区域裁剪 + 所有标注重绘 → toDataURL → 调 screenshot_write_clipboard_image
 *    - 「💾 另存为」：调用 tauri-plugin-dialog save → screenshot_save_file
 * 5. ESC：取消
 */
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save as saveDialog } from '@tauri-apps/plugin-dialog'
import { ElMessage } from 'element-plus'
import { useToolboxStore } from '@/store'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ (e: 'update:visible', val: boolean): void }>()

const store = useToolboxStore()

// ===== 渲染状态 =====
const canvasRef = ref<HTMLCanvasElement | null>(null)
/** 显示浮层（延迟截图时先 countdown，再 capture，再 overlay） */
const phase = ref<'hidden' | 'countdown' | 'loading' | 'ready' | 'done'>('hidden')
const countdown = ref(0)
let countdownTimer: number | null = null

/** 底图（Image DOM，用于 Canvas 绘制） */
const bgImage = ref<HTMLImageElement | null>(null)
/** 底图原生宽高（像素） */
const bgW = ref(0)
const bgH = ref(0)
/** 底图在当前屏幕的渲染比例（DPR/缩放后的画布像素比） */
const scaleX = ref(1)
const scaleY = ref(1)
/** 画布的 DOM 尺寸 = 当前视口尺寸（CSS 像素），真实像素要乘 devicePixelRatio */
const canvasCssW = ref(window.innerWidth)
const canvasCssH = ref(window.innerHeight)

// ===== 选框 =====
/** 选框（CSS 像素坐标系：0,0 = canvas 左上角） */
type Rect = { x: number; y: number; w: number; h: number }
const sel = ref<Rect | null>(null)
/** 选框拖拽模式：新建 / 移动 / 缩放 NW,N,NE,W,E,SW,S,SE */
let dragMode: 'new' | 'move' | 'resize' | null = null
let dragHandle: string = ''
let dragStartX = 0
let dragStartY = 0
let selStart: Rect | null = null

// ===== 标注 =====
type AnnoKind = 'rect' | 'arrow' | 'text' | 'mosaic'
interface AnnoRect { kind: 'rect'; x1: number; y1: number; x2: number; y2: number; color: string; width: number }
interface AnnoArrow { kind: 'arrow'; x1: number; y1: number; x2: number; y2: number; color: string; width: number }
interface AnnoText { kind: 'text'; x: number; y: number; text: string; color: string; fontSize: number }
interface AnnoMosaic { kind: 'mosaic'; points: { x: number; y: number }[]; size: number } // 每点一个 8x8 马赛克
type Annotation = AnnoRect | AnnoArrow | AnnoText | AnnoMosaic
const annotations = ref<Annotation[]>([])

/** 正在绘制的新标注（未确认，跟随鼠标） */
const drawing = ref<Annotation | null>(null)
/** 当前工具：select(选框调整) 或标注类型 */
type Tool = 'select' | Exclude<AnnoKind, 'text'>
const currentTool = ref<Tool>('select')
const currentColor = ref('#FF4B4B') // 默认红色框
const currentWidth = ref(3)         // 默认线粗
const currentTextSize = ref(24)
const currentMosaicSize = ref(12)   // 马赛克块像素（底图像素单位）

/** 文字输入框（canvas 内临时浮层） */
const textInputVisible = ref(false)
const textInputPos = ref({ x: 0, y: 0 })
const textInputValue = ref('')
const textInputRef = ref<HTMLInputElement | null>(null)

// ===== 导出 =====
const isExporting = ref(false)

// ============================================================
// 生命周期 & 入口
// ============================================================
watch(() => props.visible, (v) => {
  if (v) start()
})

onBeforeUnmount(() => {
  if (countdownTimer) window.clearInterval(countdownTimer)
  window.removeEventListener('resize', handleResize)
  document.removeEventListener('keydown', handleKey)
})

function start() {
  window.addEventListener('resize', handleResize)
  document.addEventListener('keydown', handleKey)

  const delay = store.screenshotDelaySec || 0
  if (delay > 0) {
    // 倒计时：浮层隐藏（不遮挡屏幕），倒计时结束再 capture
    countdown.value = delay
    phase.value = 'countdown'
    countdownTimer = window.setInterval(() => {
      countdown.value -= 1
      if (countdown.value <= 0) {
        if (countdownTimer) window.clearInterval(countdownTimer)
        countdownTimer = null
        doCapture()
      }
    }, 1000)
  } else {
    // 立即截图：为了截到浮层后面的真实屏幕，先把浮层画布留空，等 capture 返回
    doCapture()
  }
}

async function doCapture() {
  phase.value = 'loading'
  try {
    const res: { base64: string; width: number; height: number } =
      await invoke('screenshot_capture_fullscreen')
    const img = new Image()
    img.onload = async () => {
      bgImage.value = img
      bgW.value = res.width
      bgH.value = res.height
      canvasCssW.value = window.innerWidth
      canvasCssH.value = window.innerHeight
      // 计算缩放：底图虚拟屏 vs 当前视口
      scaleX.value = canvasCssW.value / bgW.value
      scaleY.value = canvasCssH.value / bgH.value
      // 重置选框/标注
      sel.value = null
      annotations.value = []
      drawing.value = null
      currentTool.value = 'select'
      phase.value = 'ready'
      await nextTick()
      redraw()
    }
    img.src = `data:image/png;base64,${res.base64}`
  } catch (e: any) {
    ElMessage.error('截图失败：' + (e?.message || String(e)))
    closeIt()
  }
}

function handleResize() {
  canvasCssW.value = window.innerWidth
  canvasCssH.value = window.innerHeight
  if (bgW.value) {
    scaleX.value = canvasCssW.value / bgW.value
    scaleY.value = canvasCssH.value / bgH.value
  }
  nextTick(redraw)
}

function handleKey(e: KeyboardEvent) {
  if (!props.visible) return
  if (e.key === 'Escape') {
    // 优先取消文字输入，再取消选框，最后取消整个浮层
    if (textInputVisible.value) {
      textInputVisible.value = false
      return
    }
    if (drawing.value) {
      drawing.value = null
      redraw()
      return
    }
    if (sel.value && currentTool.value !== 'select') {
      currentTool.value = 'select'
      return
    }
    if (sel.value) {
      sel.value = null
      redraw()
      return
    }
    closeIt()
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
    e.preventDefault()
    undo()
  }
}

function closeIt() {
  emit('update:visible', false)
  phase.value = 'hidden'
  bgImage.value = null
  if (countdownTimer) window.clearInterval(countdownTimer)
  countdownTimer = null
}

// ============================================================
// 坐标系转换：CSS 像素 <-> 底图像素
// ============================================================
function cssToBg(cx: number, cy: number) {
  return { x: cx / scaleX.value, y: cy / scaleY.value }
}
function bgToCss(bx: number, by: number) {
  return { x: bx * scaleX.value, y: by * scaleY.value }
}

// ============================================================
// Canvas 重绘（每帧：底图 + 蒙层 + 选框 + 全部标注 + drawing）
// ============================================================
function redraw() {
  const c = canvasRef.value
  const img = bgImage.value
  if (!c || !img) return
  const dpr = window.devicePixelRatio || 1
  c.width = canvasCssW.value * dpr
  c.height = canvasCssH.value * dpr
  const ctx = c.getContext('2d')!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.imageSmoothingEnabled = true

  // 1. 底图按 CSS 尺寸绘制
  ctx.drawImage(img, 0, 0, canvasCssW.value, canvasCssH.value)

  // 2. 选框外蒙层（半透明黑色）
  if (sel.value) {
    const { x, y, w, h } = sel.value
    ctx.save()
    ctx.fillStyle = 'rgba(0,0,0,0.55)'
    ctx.beginPath()
    ctx.rect(0, 0, canvasCssW.value, canvasCssH.value)
    ctx.rect(x + w, y, -w, h) // 逆时针挖洞
    ctx.fill('evenodd')
    ctx.restore()

    // 3. 选框边框 + 四角 8 控制点
    ctx.save()
    ctx.strokeStyle = '#00E5FF'
    ctx.lineWidth = 1.5
    ctx.strokeRect(x + 0.5, y + 0.5, w, h)
    // 左上/上/右上/左/右/左下/下/右下
    const pts = [
      [x, y], [x + w / 2, y], [x + w, y],
      [x, y + h / 2], [x + w, y + h / 2],
      [x, y + h], [x + w / 2, y + h], [x + w, y + h],
    ]
    for (const [px, py] of pts) {
      ctx.fillStyle = '#FFFFFF'
      ctx.strokeStyle = '#00E5FF'
      ctx.lineWidth = 1
      const s = 7
      ctx.fillRect(px - s / 2, py - s / 2, s, s)
      ctx.strokeRect(px - s / 2 + 0.5, py - s / 2 + 0.5, s, s)
    }
    // 尺寸文字
    ctx.font = '12px "Segoe UI", sans-serif'
    ctx.fillStyle = '#00E5FF'
    const { x: bx, y: by } = cssToBg(w, h)
    const sizeTxt = `${Math.round(bx)} × ${Math.round(by)}`
    ctx.fillText(sizeTxt, x + 4, y - 6)
    ctx.restore()
  }

  // 4. 标注（底图像素坐标 → 按 CSS 缩放画）
  for (const a of annotations.value) drawAnno(ctx, a)
  if (drawing.value) drawAnno(ctx, drawing.value)

  // 5. 文字输入框指示位置（画一个光标）
  if (textInputVisible.value) {
    ctx.save()
    ctx.strokeStyle = '#FFFF00'
    ctx.setLineDash([3, 3])
    ctx.strokeRect(textInputPos.value.x, textInputPos.value.y - currentTextSize.value, 200, currentTextSize.value + 8)
    ctx.restore()
  }
}

function drawAnno(ctx: CanvasRenderingContext2D, a: Annotation) {
  if (a.kind === 'rect') {
    const p1 = bgToCss(a.x1, a.y1)
    const p2 = bgToCss(a.x2, a.y2)
    ctx.save()
    ctx.strokeStyle = a.color
    ctx.lineWidth = a.width
    ctx.strokeRect(Math.min(p1.x, p2.x), Math.min(p1.y, p2.y), Math.abs(p2.x - p1.x), Math.abs(p2.y - p1.y))
    ctx.restore()
  } else if (a.kind === 'arrow') {
    const p1 = bgToCss(a.x1, a.y1)
    const p2 = bgToCss(a.x2, a.y2)
    drawArrow(ctx, p1.x, p1.y, p2.x, p2.y, a.color, a.width)
  } else if (a.kind === 'text') {
    const p = bgToCss(a.x, a.y)
    const sizeCss = a.fontSize * scaleY.value
    ctx.save()
    ctx.fillStyle = a.color
    ctx.font = `bold ${sizeCss}px "Microsoft YaHei", "PingFang SC", sans-serif`
    ctx.fillText(a.text, p.x, p.y)
    ctx.restore()
  } else if (a.kind === 'mosaic') {
    const img = bgImage.value
    if (!img) return
    for (const pt of a.points) {
      drawOneMosaic(ctx, img, pt.x, pt.y, a.size)
    }
  }
}

function drawArrow(ctx: CanvasRenderingContext2D, x1: number, y1: number, x2: number, y2: number, color: string, width: number) {
  ctx.save()
  ctx.strokeStyle = color
  ctx.fillStyle = color
  ctx.lineWidth = width
  ctx.lineCap = 'round'
  ctx.beginPath()
  ctx.moveTo(x1, y1)
  ctx.lineTo(x2, y2)
  ctx.stroke()
  // 箭头
  const ang = Math.atan2(y2 - y1, x2 - x1)
  const len = Math.max(10, width * 4)
  ctx.beginPath()
  ctx.moveTo(x2, y2)
  ctx.lineTo(x2 - len * Math.cos(ang - Math.PI / 6), y2 - len * Math.sin(ang - Math.PI / 6))
  ctx.lineTo(x2 - len * Math.cos(ang + Math.PI / 6), y2 - len * Math.sin(ang + Math.PI / 6))
  ctx.closePath()
  ctx.fill()
  ctx.restore()
}

function drawOneMosaic(ctx: CanvasRenderingContext2D, img: HTMLImageElement, cx: number, cy: number, size: number) {
  // 1. 从底图取 size*size 的像素（CSS 坐标换算到底图像素）
  // 马赛克是圆圈画笔：以鼠标半径=size/2 扫过画一块一块
  const half = Math.max(4, size / 2)
  // 每个马赛克块大小 = size (底图像素)
  const bx0 = Math.max(0, Math.floor((cx - half) / size) * size)
  const by0 = Math.max(0, Math.floor((cy - half) / size) * size)
  const bxN = Math.min(bgW.value, bx0 + Math.ceil(half * 2 / size) * size + size)
  const byN = Math.min(bgH.value, by0 + Math.ceil(half * 2 / size) * size + size)
  // 对每个 size*size 方块求平均色，然后画到当前 canvas（CSS 坐标）
  // 用一个离屏 canvas 采样
  const tmp = document.createElement('canvas')
  tmp.width = 1
  tmp.height = 1
  const tctx = tmp.getContext('2d')!
  for (let by = by0; by < byN; by += size) {
    for (let bx = bx0; bx < bxN; bx += size) {
      const bw = Math.min(size, bgW.value - bx)
      const bh = Math.min(size, bgH.value - by)
      if (bw <= 0 || bh <= 0) continue
      tctx.clearRect(0, 0, 1, 1)
      tctx.drawImage(img, bx, by, bw, bh, 0, 0, 1, 1)
      const rgba = tctx.getImageData(0, 0, 1, 1).data
      const [r, g, b] = rgba
      ctx.fillStyle = `rgb(${r},${g},${b})`
      const p = bgToCss(bx, by)
      const s = bgToCss(bw, bh)
      ctx.fillRect(p.x, p.y, s.x, s.y)
    }
  }
}

// ============================================================
// 鼠标交互
// ============================================================
/** 返回命中的 handle：'' 没命中 / move / resize 方位 NW,N,NE,W,E,SW,S,SE */
function hitHandle(mx: number, my: number): string {
  if (!sel.value) return ''
  const { x, y, w, h } = sel.value
  const t = 6
  // 内部移动：比 handle 大，优先判 handle
  const inHandle = (hx: number, hy: number) => Math.abs(mx - hx) < 7 && Math.abs(my - hy) < 7
  if (inHandle(x, y)) return 'NW'
  if (inHandle(x + w / 2, y)) return 'N'
  if (inHandle(x + w, y)) return 'NE'
  if (inHandle(x, y + h / 2)) return 'W'
  if (inHandle(x + w, y + h / 2)) return 'E'
  if (inHandle(x, y + h)) return 'SW'
  if (inHandle(x + w / 2, y + h)) return 'S'
  if (inHandle(x + w, y + h)) return 'SE'
  if (mx >= x + t && mx <= x + w - t && my >= y + t && my <= y + h - t) return 'move'
  return ''
}

function onMouseDown(e: MouseEvent) {
  if (phase.value !== 'ready' || !bgImage.value) return
  const rect = canvasRef.value!.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  dragStartX = mx
  dragStartY = my

  // 工具：文字（点击即弹输入）
  if (currentTool.value === 'rect' || currentTool.value === 'arrow') {
    const { x, y } = cssToBg(mx, my)
    if (currentTool.value === 'rect') {
      drawing.value = { kind: 'rect', x1: x, y1: y, x2: x, y2: y, color: currentColor.value, width: currentWidth.value }
    } else {
      drawing.value = { kind: 'arrow', x1: x, y1: y, x2: x, y2: y, color: currentColor.value, width: currentWidth.value }
    }
    dragMode = 'new'
    return
  }
  if (currentTool.value === 'mosaic') {
    const { x, y } = cssToBg(mx, my)
    drawing.value = { kind: 'mosaic', points: [{ x, y }], size: currentMosaicSize.value }
    dragMode = 'new'
    return
  }

  // select 模式
  const h = hitHandle(mx, my)
  if (h) {
    dragMode = h === 'move' ? 'move' : 'resize'
    dragHandle = h
    selStart = { ...sel.value! }
    return
  }
  // 选框外：新建选框
  dragMode = 'new'
  sel.value = { x: mx, y: my, w: 0, h: 0 }
  annotations.value = []
}

function onMouseMove(e: MouseEvent) {
  if (phase.value !== 'ready') return
  const rect = canvasRef.value!.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top

  // 鼠标指针
  if (!drawing.value && currentTool.value === 'select' && !dragMode && sel.value) {
    const h = hitHandle(mx, my)
    const mapCursor: Record<string, string> = {
      NW: 'nwse-resize', SE: 'nwse-resize',
      NE: 'nesw-resize', SW: 'nesw-resize',
      N: 'ns-resize', S: 'ns-resize',
      W: 'ew-resize', E: 'ew-resize',
      move: 'move',
    }
    canvasRef.value!.style.cursor = mapCursor[h] || 'crosshair'
  } else if (currentTool.value !== 'select') {
    canvasRef.value!.style.cursor = 'crosshair'
  } else {
    canvasRef.value!.style.cursor = 'crosshair'
  }

  if (!dragMode) return

  // drawing 更新
  if (drawing.value && (drawing.value.kind === 'rect' || drawing.value.kind === 'arrow')) {
    const { x, y } = cssToBg(mx, my)
    drawing.value.x2 = x
    drawing.value.y2 = y
    redraw()
    return
  }
  if (drawing.value && drawing.value.kind === 'mosaic') {
    const { x, y } = cssToBg(mx, my)
    // 连续点阵：每像素差超过 size/3 就补一个点，避免高速拖动漏空
    const last = drawing.value.points[drawing.value.points.length - 1]
    const gap = Math.max(1, drawing.value.size / 4)
    const dx = x - last.x, dy = y - last.y
    const d = Math.hypot(dx, dy)
    const steps = Math.ceil(d / gap)
    for (let i = 1; i <= steps; i++) {
      drawing.value.points.push({
        x: last.x + (dx * i) / steps,
        y: last.y + (dy * i) / steps,
      })
    }
    redraw()
    return
  }

  // 选框
  if (dragMode === 'new') {
    if (sel.value) {
      const w = mx - dragStartX
      const h = my - dragStartY
      sel.value = {
        x: w >= 0 ? dragStartX : mx,
        y: h >= 0 ? dragStartY : my,
        w: Math.abs(w),
        h: Math.abs(h),
      }
      redraw()
    }
    return
  }
  if (dragMode === 'move' && sel.value && selStart) {
    const dx = mx - dragStartX
    const dy = my - dragStartY
    sel.value.x = Math.max(0, Math.min(canvasCssW.value - selStart.w, selStart.x + dx))
    sel.value.y = Math.max(0, Math.min(canvasCssH.value - selStart.h, selStart.y + dy))
    redraw()
    return
  }
  if (dragMode === 'resize' && sel.value && selStart) {
    const dx = mx - dragStartX
    const dy = my - dragStartY
    let { x, y, w, h } = selStart
    if (dragHandle.includes('E')) w += dx
    if (dragHandle.includes('W')) { x += dx; w -= dx }
    if (dragHandle.includes('S')) h += dy
    if (dragHandle.includes('N')) { y += dy; h -= dy }
    // 防止翻转
    if (w < 4) { w = 4; x = selStart.x + selStart.w - 4 }
    if (h < 4) { h = 4; y = selStart.y + selStart.h - 4 }
    // 防越界
    x = Math.max(0, x); y = Math.max(0, y)
    w = Math.min(canvasCssW.value - x, w)
    h = Math.min(canvasCssH.value - y, h)
    sel.value = { x, y, w, h }
    redraw()
    return
  }
}

function onMouseUp(e: MouseEvent) {
  if (!dragMode) return
  dragMode = null
  if (drawing.value) {
    annotations.value.push(drawing.value as Annotation)
    drawing.value = null
    redraw()
    return
  }
  // 新建选框成功后，自动把工具切换到 select（允许继续调整）
  if (sel.value && sel.value.w < 3 && sel.value.h < 3) {
    // 太小算误点 → 如果是「文字」工具：点一下打开文字输入框
    if (e.shiftKey) {
      // 预留：shift 点做某事
    }
  }
  redraw()
}

// 双击选框内 = 确认 + 复制（快捷键）
function onDblClick(e: MouseEvent) {
  if (!sel.value) return
  const rect = canvasRef.value!.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  const { x, y, w, h } = sel.value
  if (mx >= x && mx <= x + w && my >= y && my <= y + h) {
    void doCopy()
  }
}

// 画布点击空白：切换文字工具时弹输入
function onClick(e: MouseEvent) {
  if (phase.value !== 'ready') return
  if (currentTool.value !== ('text' as unknown as Tool)) return
  // 点击画布任意位置 → 出现输入框
  const rect = canvasRef.value!.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top
  textInputPos.value = { x: mx, y: my }
  textInputValue.value = ''
  textInputVisible.value = true
  nextTick(() => {
    textInputRef.value?.focus()
  })
}

function confirmText() {
  if (!textInputValue.value.trim()) {
    textInputVisible.value = false
    return
  }
  const { x, y } = cssToBg(textInputPos.value.x, textInputPos.value.y)
  annotations.value.push({
    kind: 'text',
    x,
    y, // 文字基线（底部）
    text: textInputValue.value,
    color: currentColor.value,
    fontSize: currentTextSize.value,
  })
  textInputVisible.value = false
  redraw()
}

function undo() {
  annotations.value.pop()
  redraw()
}

// ============================================================
// 导出：裁剪 + 重绘所有标注到离屏 canvas（底图像素尺寸）
// ============================================================
async function buildFinalPng(): Promise<{ base64: string; width: number; height: number } | null> {
  const img = bgImage.value
  if (!img) { console.warn('[screenshot] buildFinalPng: bgImage is null'); return null }
  // 没选框 → 按全图
  const selCss = sel.value || { x: 0, y: 0, w: canvasCssW.value, h: canvasCssH.value }
  const sx = Math.floor(selCss.x / scaleX.value)
  const sy = Math.floor(selCss.y / scaleY.value)
  const sw = Math.max(1, Math.ceil(selCss.w / scaleX.value))
  const sh = Math.max(1, Math.ceil(selCss.h / scaleY.value))
  console.log(`[screenshot] buildFinalPng: selCss=${JSON.stringify(selCss)}, bgCoords=(${sx},${sy},${sw},${sh}), bgSize=(${bgW.value}x${bgH.value})`)
  if (sx + sw > bgW.value || sy + sh > bgH.value || sw <= 0 || sh <= 0) {
    console.error('[screenshot] buildFinalPng: WARNING - crop region out of bounds!', {sx,sy,sw,sh,bgW:bgW.value,bgH:bgH.value})
  }

  const out = document.createElement('canvas')
  out.width = sw
  out.height = sh
  const ctx = out.getContext('2d')!
  ctx.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh)

  // 标注坐标是底图像素级全局坐标 → 转输出 canvas（平移 -sx,-sy）
  for (const a of annotations.value) {
    if (a.kind === 'rect') {
      ctx.strokeStyle = a.color
      ctx.lineWidth = a.width
      ctx.strokeRect(a.x1 - sx, a.y1 - sy, a.x2 - a.x1, a.y2 - a.y1)
    } else if (a.kind === 'arrow') {
      drawArrowBg(ctx, a.x1 - sx, a.y1 - sy, a.x2 - sx, a.y2 - sy, a.color, a.width)
    } else if (a.kind === 'text') {
      ctx.fillStyle = a.color
      ctx.font = `bold ${a.fontSize}px "Microsoft YaHei", "PingFang SC", sans-serif`
      ctx.fillText(a.text, a.x - sx, a.y - sy)
    } else if (a.kind === 'mosaic') {
      for (const pt of a.points) {
        drawMosaicOnBg(ctx, img, sx, sy, sw, sh, pt.x, pt.y, a.size)
      }
    }
  }
  return {
    base64: out.toDataURL('image/png'),
    width: sw,
    height: sh,
  }
}
function drawArrowBg(ctx: CanvasRenderingContext2D, x1: number, y1: number, x2: number, y2: number, color: string, w: number) {
  ctx.save()
  ctx.strokeStyle = color
  ctx.fillStyle = color
  ctx.lineWidth = w
  ctx.lineCap = 'round'
  ctx.beginPath()
  ctx.moveTo(x1, y1)
  ctx.lineTo(x2, y2)
  ctx.stroke()
  const ang = Math.atan2(y2 - y1, x2 - x1)
  const len = Math.max(8, w * 4)
  ctx.beginPath()
  ctx.moveTo(x2, y2)
  ctx.lineTo(x2 - len * Math.cos(ang - Math.PI / 6), y2 - len * Math.sin(ang - Math.PI / 6))
  ctx.lineTo(x2 - len * Math.cos(ang + Math.PI / 6), y2 - len * Math.sin(ang + Math.PI / 6))
  ctx.closePath()
  ctx.fill()
  ctx.restore()
}
function drawMosaicOnBg(ctx: CanvasRenderingContext2D, img: HTMLImageElement, sx: number, sy: number, sw: number, sh: number, cx: number, cy: number, size: number) {
  const half = Math.max(4, size / 2)
  const bx0 = Math.max(0, Math.floor((cx - half) / size) * size)
  const by0 = Math.max(0, Math.floor((cy - half) / size) * size)
  const bxN = Math.min(sx + sw, bx0 + Math.ceil(half * 2 / size) * size + size)
  const byN = Math.min(sy + sh, by0 + Math.ceil(half * 2 / size) * size + size)
  const tmp = document.createElement('canvas')
  tmp.width = 1
  tmp.height = 1
  const tctx = tmp.getContext('2d')!
  for (let by = by0; by < byN; by += size) {
    for (let bx = bx0; bx < bxN; bx += size) {
      const bw = Math.min(size, (sx + sw) - bx)
      const bh = Math.min(size, (sy + sh) - by)
      if (bw <= 0 || bh <= 0) continue
      tctx.clearRect(0, 0, 1, 1)
      tctx.drawImage(img, bx, by, bw, bh, 0, 0, 1, 1)
      const rgba = tctx.getImageData(0, 0, 1, 1).data
      ctx.fillStyle = `rgb(${rgba[0]},${rgba[1]},${rgba[2]})`
      ctx.fillRect(bx - sx, by - sy, bw, bh)
    }
  }
}

// 完成：复制到剪贴板
async function doCopy() {
  if (isExporting.value) { console.warn('[screenshot] doCopy blocked: already exporting'); return }
  isExporting.value = true
  console.log('[screenshot] doCopy: start, bgImage=', !!bgImage.value, 'bgW=', bgW.value, 'bgH=', bgH.value, 'scaleX=', scaleX.value, 'scaleY=', scaleY.value)
  try {
    const t1 = performance.now()
    const out = await buildFinalPng()
    const t2 = performance.now()
    console.log(`[screenshot] buildFinalPng: ${(t2-t1).toFixed(0)}ms, out=`, out ? `size=${out.width}x${out.height}, base64Len=${out.base64.length}` : 'NULL')
    if (!out) { ElMessage.warning('还未捕获到底图'); return }
    // base64: data:image/png;base64,xxx → 去掉前缀
    const b64 = out.base64.slice(out.base64.indexOf(',') + 1)
    const t3 = performance.now()
    console.log(`[screenshot] invoking screenshot_write_clipboard_image, b64Len=${b64.length}`)
    await invoke('screenshot_write_clipboard_image', { base64Png: b64 })
    const t4 = performance.now()
    console.log(`[screenshot] invoke done in ${(t4-t3).toFixed(0)}ms`)
    // 记录操作历史
    store.addHistory({
      tool: 'screenshot',
      action: '截图复制剪贴板',
      inputPreview: `截图 ${out.width}x${out.height}`,
      outputPreview: '已复制到剪贴板',
      inputFull: '',
      outputFull: out.base64,
    })
    ElMessage.success(`已复制 (${out.width}×${out.height})`)
    closeIt()
  } catch (e: any) {
    console.error('[screenshot] doCopy ERROR:', e)
    ElMessage.error('复制失败：' + (e?.message || String(e)))
  } finally {
    isExporting.value = false
    console.log('[screenshot] doCopy: done, isExporting=false')
  }
}

// 另存为
async function doSave() {
  isExporting.value = true
  try {
    const out = await buildFinalPng()
    if (!out) { ElMessage.warning('还未捕获到底图'); return }
    const defaultDir = await invoke<string>('screenshot_get_default_dir')
    const fname = `截图_${Date.now()}.png`
    const picked = await saveDialog({
      defaultPath: `${defaultDir}\\${fname}`,
      filters: [{ name: 'PNG 图片', extensions: ['png'] }],
    })
    if (!picked) return
    // 拆 dir + filename
    const lastSlash = Math.max(picked.lastIndexOf('/'), picked.lastIndexOf('\\'))
    const dir = lastSlash >= 0 ? picked.slice(0, lastSlash) : defaultDir
    const base = lastSlash >= 0 ? picked.slice(lastSlash + 1).replace(/\.png$/i, '') : fname.replace(/\.png$/i, '')
    const b64 = out.base64.slice(out.base64.indexOf(',') + 1)
    const saved = await invoke<string>('screenshot_save_file', {
      base64Png: b64,
      options: { dir, filename: base },
    })
    store.addHistory({
      tool: 'screenshot',
      action: '截图另存',
      inputPreview: `截图 ${out.width}x${out.height}`,
      outputPreview: saved,
      inputFull: '',
      outputFull: saved,
    })
    ElMessage.success(`已导出到: ${saved}`)
    closeIt()
  } catch (e: any) {
    ElMessage.error('保存失败：' + (e?.message || String(e)))
  } finally {
    isExporting.value = false
  }
}

// ============================================================
// 计算属性
// ============================================================
const selSizeCss = computed(() => {
  if (!sel.value) return ''
  const { x, y } = cssToBg(sel.value.w, sel.value.h)
  return `${Math.round(x)} × ${Math.round(y)}`
})
</script>

<template>
  <!-- 倒计时遮罩 -->
  <div v-if="visible && phase === 'countdown'" class="sc-countdown">
    <div class="sc-countdown-num">{{ countdown }}</div>
    <div class="sc-countdown-tip">秒后开始截图...</div>
  </div>

  <!-- 加载遮罩 -->
  <div v-if="visible && phase === 'loading'" class="sc-loading">
    <div class="spinner"></div>
    <div>正在捕获屏幕...</div>
  </div>

  <!-- 主界面：canvas 铺满整屏 -->
  <div
    v-if="visible && (phase === 'ready' || phase === 'done')"
    class="sc-root"
  >
    <canvas
      ref="canvasRef"
      class="sc-canvas"
      :style="{ width: canvasCssW + 'px', height: canvasCssH + 'px' }"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @dblclick="onDblClick"
      @click="onClick"
    />

    <!-- 顶部工具栏 -->
    <div v-if="phase === 'ready'" class="sc-toolbar top">
      <div class="sc-tb-group">
        <button :class="{ active: currentTool === 'select' }" @click="currentTool = 'select'" title="移动/调整选框">
          📐 选框
        </button>
        <button :class="{ active: currentTool === 'rect' }" @click="currentTool = 'rect'" title="矩形标注">
          ▭ 矩形
        </button>
        <button :class="{ active: currentTool === 'arrow' }" @click="currentTool = 'arrow'" title="箭头">
          ➡ 箭头
        </button>
        <button :class="{ active: currentTool === ('text' as unknown as Tool) }" @click="currentTool = ('text' as unknown as Tool)" title="文字 (点击画布任意位置输入)">
          🔤 文字
        </button>
        <button :class="{ active: currentTool === 'mosaic' }" @click="currentTool = 'mosaic'" title="马赛克（按住拖动画笔）">
          🟦 马赛克
        </button>
        <button @click="undo" :disabled="!annotations.length" title="撤销 (Ctrl+Z)">
          ↶ 撤销
        </button>
      </div>

      <div class="sc-tb-group">
        <label class="sc-label">
          颜色
          <input type="color" v-model="currentColor" />
        </label>
        <label class="sc-label" v-if="currentTool !== 'mosaic' && currentTool !== 'text' as unknown as Tool">
          线粗
          <input type="range" min="1" max="12" v-model.number="currentWidth" class="sc-range" />
          <span class="sc-num">{{ currentWidth }}</span>
        </label>
        <label class="sc-label" v-if="currentTool === ('text' as unknown as Tool)">
          字号
          <input type="range" min="12" max="72" step="1" v-model.number="currentTextSize" class="sc-range" />
          <span class="sc-num">{{ currentTextSize }}</span>
        </label>
        <label class="sc-label" v-if="currentTool === 'mosaic'">
          块大小
          <input type="range" min="6" max="40" step="2" v-model.number="currentMosaicSize" class="sc-range" />
          <span class="sc-num">{{ currentMosaicSize }}</span>
        </label>
      </div>

      <div class="sc-tb-group">
        <span v-if="selSizeCss" class="sc-sizes">{{ selSizeCss }}</span>
        <button class="sc-cancel" @click="closeIt" title="ESC 取消">✖ 取消</button>
        <button class="sc-ok" :loading="isExporting" @click="doCopy" title="复制到剪贴板">
          ✅ 复制
        </button>
        <button class="sc-save" :loading="isExporting" @click="doSave" title="另存为 PNG">
          💾 另存为
        </button>
      </div>
    </div>

    <!-- 底部帮助提示 -->
    <div v-if="phase === 'ready'" class="sc-hint">
      <template v-if="!sel">
        拖拽鼠标框选区域 · ESC 取消 · 双击全选后复制
      </template>
      <template v-else-if="currentTool === 'select'">
        拖动边框调整 / 内部移动 · 切换上方工具开始标注 · ESC 取消选框
      </template>
      <template v-else-if="currentTool === ('text' as unknown as Tool)">
        点击画布任意位置输入文字 · ESC 取消
      </template>
      <template v-else-if="currentTool === 'mosaic'">
        按住左键拖动画笔涂抹 · Ctrl+Z 撤销
      </template>
      <template v-else>
        拖动鼠标画出 {{ currentTool === 'rect' ? '矩形' : '箭头' }} · Ctrl+Z 撤销
      </template>
    </div>

    <!-- 文字输入浮层（跟随点击位置） -->
    <div
      v-if="textInputVisible"
      class="sc-text-input"
      :style="{ left: textInputPos.x + 'px', top: (textInputPos.y - currentTextSize - 4) + 'px' }"
    >
      <input
        ref="textInputRef"
        v-model="textInputValue"
        class="sc-text-box"
        :style="{ color: currentColor, fontSize: currentTextSize + 'px', height: (currentTextSize + 14) + 'px' }"
        placeholder="输入文字后按回车"
        @keydown.enter.prevent="confirmText"
        @keydown.esc.prevent="textInputVisible = false"
        @blur="confirmText"
      />
    </div>
  </div>
</template>

<style scoped>
.sc-root {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: transparent;
}
.sc-canvas {
  display: block;
  position: absolute;
  left: 0;
  top: 0;
  user-select: none;
}

/* 工具栏 */
.sc-toolbar {
  position: fixed;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  background: rgba(20, 28, 40, 0.92);
  backdrop-filter: blur(10px);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 8px 10px;
  display: flex;
  gap: 10px;
  align-items: center;
  box-shadow: 0 6px 24px rgba(0,0,0,0.5);
}
.sc-toolbar.top { top: 16px; }

.sc-tb-group {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  border-right: 1px solid rgba(128,128,128,0.3);
}
.sc-tb-group:last-child { border-right: none; }

.sc-toolbar button {
  background: transparent;
  color: #c9d6ff;
  border: 1px solid transparent;
  padding: 6px 10px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s ease;
  white-space: nowrap;
}
.sc-toolbar button:hover:not(:disabled) {
  border-color: var(--color-accent);
  color: var(--color-accent);
}
.sc-toolbar button.active {
  background: var(--color-accent);
  color: #0b1220;
  font-weight: 600;
}
.sc-toolbar button:disabled { opacity: 0.4; cursor: not-allowed; }

.sc-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #c9d6ff;
  font-size: 12px;
}
.sc-label input[type="color"] {
  width: 26px;
  height: 26px;
  border: none;
  padding: 0;
  background: transparent;
  cursor: pointer;
  border-radius: 4px;
}
.sc-range {
  width: 80px;
  accent-color: var(--color-accent);
}
.sc-num {
  color: var(--color-accent);
  min-width: 22px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.sc-sizes {
  color: #00E5FF;
  font-size: 12px;
  margin-right: 6px;
  letter-spacing: 0.3px;
}

.sc-cancel {
  border: 1px solid rgba(255,100,100,0.4) !important;
  color: #ffb8b8 !important;
}
.sc-cancel:hover { border-color: #ff6464 !important; color: #ff6464 !important; }

.sc-ok {
  background: #1fab71 !important;
  color: #ffffff !important;
  font-weight: 600;
}
.sc-ok:hover:not(:disabled) { background: #21c980 !important; color: #fff !important; }

.sc-save {
  background: #2f67ff !important;
  color: #fff !important;
  font-weight: 600;
}
.sc-save:hover:not(:disabled) { background: #4b80ff !important; color: #fff !important; }

/* 帮助提示 */
.sc-hint {
  position: fixed;
  left: 50%;
  bottom: 20px;
  transform: translateX(-50%);
  z-index: 10000;
  background: rgba(20, 28, 40, 0.9);
  color: #c9d6ff;
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid var(--border-color);
  font-size: 12px;
  letter-spacing: 0.3px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.4);
}

/* 文字输入 */
.sc-text-input {
  position: fixed;
  z-index: 10001;
}
.sc-text-box {
  background: rgba(15, 20, 30, 0.95);
  border: 1px dashed #00E5FF;
  border-radius: 6px;
  padding: 4px 8px;
  outline: none;
  color: #fff;
  width: 260px;
  font-family: "Microsoft YaHei", sans-serif;
  font-weight: bold;
}

/* 倒计时 */
.sc-countdown {
  position: fixed;
  inset: 0;
  z-index: 9998;
  background: rgba(0,0,0,0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  pointer-events: none;
  color: #fff;
}
.sc-countdown-num {
  font-size: 160px;
  font-weight: 800;
  line-height: 1;
  text-shadow: 0 8px 32px rgba(0,0,0,0.5);
  color: var(--color-accent);
  animation: pulse 1s ease-in-out infinite;
}
.sc-countdown-tip {
  margin-top: 20px;
  font-size: 18px;
  letter-spacing: 2px;
  opacity: 0.85;
}
@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.15); opacity: 0.8; }
}

/* loading */
.sc-loading {
  position: fixed;
  inset: 0;
  z-index: 9998;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 18px;
  color: #fff;
  font-size: 14px;
}
.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255,255,255,0.2);
  border-top-color: var(--color-accent);
  border-radius: 50%;
  animation: spin 0.9s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
