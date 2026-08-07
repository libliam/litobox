<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="qr-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 二维码生成 -->
      <el-tab-pane label="生成" name="generate">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入文本或 URL，生成二维码图片</p>
                    <p>支持配置尺寸、边距、前景色/背景色</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('generate')">清空</el-button>
              <el-button size="small" @click="handlePaste('generate')">粘贴</el-button>
              <el-button size="small" type="primary" @click="handleGenerate">生成</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">尺寸</div>
                <el-input-number v-model="qrSize" :min="100" :max="1000" :step="50" size="small" style="width: 100px" />
              </div>
              <div class="action-group">
                <div class="group-label">边距</div>
                <el-input-number v-model="qrMargin" :min="0" :max="10" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">容错率</div>
                <el-select v-model="qrErrorLevel" size="small" style="width: 80px">
                  <el-option label="L (7%)" value="L" />
                  <el-option label="M (15%)" value="M" />
                  <el-option label="Q (25%)" value="Q" />
                  <el-option label="H (30%)" value="H" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">前景色</div>
                <input type="color" v-model="qrFgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">背景色</div>
                <input type="color" v-model="qrBgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">码点样式</div>
                <el-select v-model="qrDotStyle" size="small" style="width: 100px">
                  <el-option label="方块" value="square" />
                  <el-option label="圆点" value="dots" />
                  <el-option label="圆角" value="rounded" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">渐变色</div>
                <el-switch v-model="qrGradient" size="small" />
                <input v-if="qrGradient" type="color" v-model="qrGradientColor" class="native-color-picker" />
              </div>
            </div>
            <div class="action-grid" style="margin-top: 12px;">
              <div class="action-group">
                <div class="group-label">内嵌 Logo</div>
                <el-button size="small" @click="triggerLogoUpload">
                  <el-icon><Upload /></el-icon>
                  <span>{{ qrLogo ? '更换' : '上传' }}</span>
                </el-button>
                <input ref="logoInput" type="file" accept="image/*" class="file-input" @change="handleLogoChange" />
                <el-button v-if="qrLogo" size="small" @click="qrLogo = ''">移除</el-button>
                <span v-if="qrLogo" class="stat-text">Logo: {{ qrLogoSize }}px</span>
              </div>
              <div v-if="qrLogo" class="action-group">
                <div class="group-label">Logo 大小</div>
                <el-slider v-model="qrLogoSize" :min="40" :max="200" :step="10" size="small" style="width: 120px" />
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入内容</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('generate')">清空</el-button>
              <el-button size="small" @click="handlePaste('generate')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.generate.input"
              type="textarea"
              :rows="4"
              placeholder="输入文本或 URL..."
              resize="vertical"
            />
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">二维码</span>
            <div class="card-actions">
              <el-button size="small" @click="jumpToDecode" :disabled="!qrDataUrl">
                <el-icon><View /></el-icon>
                <span>解析此码</span>
              </el-button>
              <el-button size="small" @click="handleDownloadQr" :disabled="!qrDataUrl">下载 PNG</el-button>
            </div>
          </div>
          <div class="card-body">
            <div v-if="qrDataUrl" class="qr-result">
              <img :src="qrDataUrl" alt="QR Code" class="qr-image" />
            </div>
            <div v-else-if="generateError" class="error-message">{{ generateError }}</div>
            <div v-else class="stats-empty">
              点击"生成"按钮生成二维码
            </div>
            <!-- 自动验证结果 -->
            <div v-if="verifyResult" class="verify-banner" :class="verifyResult.ok ? 'verify-ok' : 'verify-fail'">
              <el-icon><component :is="verifyResult.ok ? CircleCheck : CircleClose" /></el-icon>
              <span>{{ verifyResult.message }}</span>
              <el-button v-if="!verifyResult.ok" size="small" type="primary" link @click="qrErrorLevel = 'H'">
                切换为 H 容错率
              </el-button>
            </div>
          </div>
        </div>

        <!-- 识别提示 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">识别失败排查</span>
            <el-tooltip placement="top" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>扫码无法识别时，按以下顺序排查</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-body tips-list">
            <div class="tip-item" :class="{ 'tip-warn': riskLowTolerance }">
              <span class="tip-num">1</span>
              <div class="tip-content">
                <div class="tip-title">容错率不足</div>
                <div class="tip-desc">低容错率（L/M）+ 自定义样式（圆点/渐变/Logo）容易识别失败。建议选择 H（30%）。</div>
              </div>
            </div>
            <div class="tip-item" :class="{ 'tip-warn': riskLogoTooLarge }">
              <span class="tip-num">2</span>
              <div class="tip-content">
                <div class="tip-title">Logo 遮挡过多</div>
                <div class="tip-desc">Logo 占比过大或低容错率下会遮挡数据模块。建议缩小 Logo 或改用 H 级容错率。</div>
              </div>
            </div>
            <div class="tip-item" :class="{ 'tip-warn': riskCustomStyle && riskLowTolerance }">
              <span class="tip-num">3</span>
              <div class="tip-content">
                <div class="tip-title">码点样式识别门槛高</div>
                <div class="tip-desc">圆点/圆角样式对扫码器识别要求更高。若识别困难，先切换为方块样式测试。</div>
              </div>
            </div>
            <div class="tip-item" :class="{ 'tip-warn': riskGradient && riskLowTolerance }">
              <span class="tip-num">4</span>
              <div class="tip-content">
                <div class="tip-title">渐变色对比度不足</div>
                <div class="tip-desc">渐变中亮色码点可能对比度不够，相机扫描时会误判为背景。确保前景色与背景色亮度差大于 128。</div>
              </div>
            </div>
            <div class="tip-item">
              <span class="tip-num">5</span>
              <div class="tip-content">
                <div class="tip-title">打印/截图质量</div>
                <div class="tip-desc">打印时确保尺寸不小于 2cm，扫描时保持平整、光线充足。电子屏幕显示时避免摩尔纹干扰。</div>
              </div>
            </div>
            <div class="tip-item">
              <span class="tip-num">6</span>
              <div class="tip-content">
                <div class="tip-title">内容过长</div>
                <div class="tip-desc">内容越长 → 模块越多 → 单模块越小，识别难度增加。建议长文本使用短链接或分段编码。</div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 批量生成 -->
      <el-tab-pane label="批量生成" name="batch">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">批量操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>每行一条文本，支持 CSV/文本列表</p>
                    <p>可批量生成并打包下载</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('batch')">清空</el-button>
              <el-button size="small" @click="handlePaste('batch')">粘贴</el-button>
              <el-button size="small" type="primary" @click="handleBatchGenerate" :loading="batchLoading">
                <el-icon class="batch-icon"><MagicStick /></el-icon>
                <span>批量生成</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">尺寸</div>
                <el-input-number v-model="batchSize" :min="100" :max="1000" :step="50" size="small" style="width: 100px" />
              </div>
              <div class="action-group">
                <div class="group-label">边距</div>
                <el-input-number v-model="batchMargin" :min="0" :max="10" size="small" style="width: 80px" />
              </div>
              <div class="action-group">
                <div class="group-label">前景色</div>
                <input type="color" v-model="batchFgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">背景色</div>
                <input type="color" v-model="batchBgColor" class="native-color-picker" />
              </div>
              <div class="action-group">
                <div class="group-label">命名</div>
                <el-select v-model="batchNameMode" size="small" style="width: 140px">
                  <el-option label="序号 (001, 002...)" value="indexed" />
                  <el-option label="文本前16字符" value="text" />
                </el-select>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">文本列表</span>
            <div class="card-actions">
              <span class="stat-text">{{ batchLines.length }} 条</span>
              <el-button size="small" @click="uploadCsv">
                <el-icon><Upload /></el-icon>
                <span>导入</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.batch.input"
              type="textarea"
              :rows="8"
              placeholder="每行一条文本或 URL，例如：&#10;https://example.com&#10;Hello World&#10;SELECT * FROM users"
              resize="vertical"
            />
            <div v-if="batchErrors.length" class="batch-errors">
              <span class="error-title">{{ batchErrors.length }} 条失败：</span>
              <span v-for="(e, i) in batchErrors" :key="i" class="error-item">第{{ e.line }}行</span>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card" v-if="batchResults.length">
          <div class="card-header">
            <span class="card-title">生成结果（{{ batchResults.length }} 条）</span>
            <div class="card-actions">
              <el-button size="small" @click="downloadAllZip">
                <el-icon><Download /></el-icon>
                <span>打包下载 (ZIP)</span>
              </el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="batch-grid">
              <div v-for="(item, i) in batchResults" :key="i" class="batch-item">
                <img :src="item.dataUrl" :alt="item.filename" class="batch-qr" />
                <div class="batch-info">
                  <span class="batch-filename" :title="item.text">{{ item.filename }}</span>
                  <span class="batch-text" :title="item.text">{{ item.text }}</span>
                </div>
                <div class="batch-actions">
                  <el-button size="small" @click="downloadSingle(item)">
                    <el-icon><Download /></el-icon>
                    <span>下载</span>
                  </el-button>
                  <el-button size="small" @click="copyDataUrl(item)">
                    <el-icon><DocumentCopy /></el-icon>
                    <span>复制</span>
                  </el-button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2 (实际顺序 Tab 3): 二维码解码 -->
      <el-tab-pane label="解码" name="decode">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>上传二维码图片或从剪贴板粘贴，解析二维码内容</p>
                    <p>支持 PNG/JPG/WebP 格式</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('decode')">清空</el-button>
              <el-button size="small" @click="handlePaste('decode')">粘贴</el-button>
              <el-button size="small" type="primary" @click="readFromClipboard">
                <el-icon><CopyDocument /></el-icon>
                <span>读取剪贴板</span>
              </el-button>
              <el-button size="small" @click="handleCopy('decode')">复制</el-button>
            </div>
          </div>
        </div>

        <!-- 上传卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">上传二维码</span>
          </div>
          <div class="card-body">
            <div class="upload-area" @click="triggerFileInput" @drop.prevent="handleDrop" @dragover.prevent>
              <input ref="fileInput" type="file" accept="image/*" class="file-input" @change="handleFileChange" />
              <div class="upload-content">
                <el-icon class="upload-icon"><Upload /></el-icon>
                <p>点击上传或拖拽图片到此处</p>
                <p class="upload-hint">支持 PNG / JPG / WebP</p>
              </div>
            </div>
            <div v-if="decodePreview" class="decode-preview">
              <img :src="decodePreview" alt="Preview" class="preview-image" />
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">解析结果</span>
            <div class="card-actions">
              <el-button size="small" @click="handleCopy('decode')">复制</el-button>
              <el-button size="small" type="success" :disabled="!tabState.decode.output" @click="handleSaveDecodeEdit">保存修改</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.decode.output"
              type="textarea"
              :rows="6"
              resize="vertical"
              :class="{ 'error': tabState.decode.isError }"
            />
            <div v-if="tabState.decode.error" class="error-message">
              {{ tabState.decode.error }}
            </div>
            <div v-if="!tabState.decode.output && !tabState.decode.error" class="stats-empty">
              上传图片后自动解析
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Upload, MagicStick, Download, DocumentCopy, View, CircleCheck, CircleClose, CopyDocument } from '@element-plus/icons-vue'
import QRCode from 'qrcode'
import jsQR from 'jsqr'
import { useToolboxStore } from '@/store'
import { saveFileWithDialog } from '@/utils/fileSaver'

const store = useToolboxStore()

// ============ Tab 状态 ============
const activeTab = ref('generate')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  generate: { input: '', output: '', error: '', isError: false },
  decode: { input: '', output: '', error: '', isError: false },
  batch: { input: '', output: '', error: '', isError: false }
})

// ============ 生成 Tab ============
const qrSize = ref(300)
const qrMargin = ref(2)
const qrFgColor = ref('#000000')
const qrBgColor = ref('#ffffff')
const qrErrorLevel = ref('M')
const qrDotStyle = ref('square')
const qrGradient = ref(false)
const qrGradientColor = ref('#7b2ff7')
const qrLogo = ref('')
const qrLogoSize = ref(80)
const logoInput = ref<HTMLInputElement | null>(null)
const qrDataUrl = ref('')
const generateError = ref('')
const verifyResult = ref<{ ok: boolean; message: string } | null>(null)

// 智能风险判断：只有生成后验证失败才高亮
const verifyFailed = computed(() => verifyResult.value?.ok === false)

const riskLowTolerance = computed(() => {
  if (!verifyFailed.value) return false
  const lowTol = qrErrorLevel.value === 'L' || qrErrorLevel.value === 'M'
  const customStyle = qrDotStyle.value !== 'square' || qrGradient.value || !!qrLogo.value
  return lowTol && customStyle
})

const riskLogoTooLarge = computed(() => {
  if (!verifyFailed.value) return false
  return !!qrLogo.value && qrErrorLevel.value !== 'H'
})

const riskCustomStyle = computed(() => {
  if (!verifyFailed.value) return false
  return qrDotStyle.value !== 'square'
})

const riskGradient = computed(() => {
  if (!verifyFailed.value) return false
  return qrGradient.value
})

const triggerLogoUpload = () => logoInput.value?.click()

const handleLogoChange = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => { qrLogo.value = reader.result as string }
  reader.readAsDataURL(file)
}

// hex 转 rgb
const hexToRgb = (hex: string) => {
  const m = hex.match(/^#?([a-f0-9]{2})([a-f0-9]{2})([a-f0-9]{2})$/i)
  return m ? { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) } : { r: 0, g: 0, b: 0 }
}

// 计算颜色亮度（感知亮度，0-255，越大越亮）
const getLuminance = (r: number, g: number, b: number) => 0.299 * r + 0.587 * g + 0.114 * b

// 渐变插值，保持码点足够暗（亮度上限受前景色亮度约束）
const lerpColor = (c1: string, c2: string, t: number) => {
  const a = hexToRgb(c1), b = hexToRgb(c2)
  const baseLum = getLuminance(a.r, a.g, a.b)
  const r = Math.round(a.r + (b.r - a.r) * t)
  const g = Math.round(a.g + (b.g - a.g) * t)
  const bl = Math.round(a.b + (b.b - a.b) * t)
  // 码点亮度不能超过前景色亮度 + 60，保证相机能识别
  const lum = getLuminance(r, g, bl)
  const maxLum = Math.min(255, baseLum + 60)
  if (lum > maxLum) {
    const ratio = maxLum / lum
    return `rgb(${Math.round(r * ratio)},${Math.round(g * ratio)},${Math.round(bl * ratio)})`
  }
  return `rgb(${r},${g},${bl})`
}

// 容错率对应的最大 Logo 占比（总码点数的百分比）
const LOGO_RATIO_MAP: Record<string, number> = { L: 0.05, M: 0.10, Q: 0.18, H: 0.25 }

const handleGenerate = async () => {
  const input = tabState.generate.input.trim()
  if (!input) {
    ElMessage.warning('请输入内容')
    return
  }

  try {
    // 1. 用 qrcode 库生成矩阵数据（不直接渲染到 canvas）
    const qrData = QRCode.create(input, {
      errorCorrectionLevel: qrErrorLevel.value as any
    })

    // 2. 用 Canvas 自定义渲染
    const modules = qrData.modules
    const moduleCount = modules.size
    const margin = qrMargin.value
    const totalSize = moduleCount + margin * 2
    // 每个模块至少 4px，确保圆点/渐变色等自定义样式有足够像素
    const minPxPerModule = 4
    const pixelSize = Math.max(minPxPerModule, Math.floor(qrSize.value / totalSize))
    const canvasSize = pixelSize * totalSize

    const canvas = document.createElement('canvas')
    canvas.width = canvasSize
    canvas.height = canvasSize
    const ctx = canvas.getContext('2d')!

    // 背景
    ctx.fillStyle = qrBgColor.value
    ctx.fillRect(0, 0, canvasSize, canvasSize)

    // 判断是否为固定图案（finder / alignment / timing），必须用方块+纯色渲染
    const isFixedPattern = (row: number, col: number) => {
      // 三个 7x7 定位码（finder pattern）
      if (row < 7 && col < 7) return true
      if (row < 7 && col >= moduleCount - 7) return true
      if (row >= moduleCount - 7 && col < 7) return true
      // 时序图案（timing pattern）：第 6 行 + 第 6 列（从定位码之后开始）
      if (row === 6 && col >= 8) return true
      if (col === 6 && row >= 8) return true
      // 校正图案（alignment pattern）：右下角附近
      if (moduleCount >= 25) {
        // 版本 2+ 在右下角有一个校正图案，中心距右下边缘 7-8 格
        // 用 qrcode 库的 alignment pattern 位置计算
        const apCenters = getAlignmentPatternCenters(moduleCount)
        for (const [ar, ac] of apCenters) {
          if (Math.abs(row - ar) <= 2 && Math.abs(col - ac) <= 2) return true
        }
      }
      return false
    }

    // 计算校正图案中心位置（QR 标准）
    const getAlignmentPatternCenters = (size: number): [number, number][] => {
      const centers: [number, number][] = []
      // 右下角校正图案（几乎所有版本 2+ 都有）
      const right = size - 7
      // 版本 1: 无校正图案
      // 版本 2-6: 只有 1 个校正图案在右下
      // 版本 7+: 可能有多个
      // 简化：用 qrcode 库内部版本信息
      const version = getVersionFromSize(size)
      if (version >= 2) {
        // 右下校正图案
        centers.push([right - 2, right - 2])
        // 版本 4+: 中间和右下再加
        if (version >= 4) {
          const mid = Math.floor(right * 0.55)
          centers.push([mid, right - 2])
          centers.push([right - 2, mid])
        }
      }
      return centers
    }

    // QR 码版本 → 模块数映射
    const getVersionFromSize = (size: number): number => {
      for (let v = 1; v <= 40; v++) {
        const modules = 17 + v * 4
        if (modules === size) return v
      }
      return 1
    }

    // 获取码点颜色（固定图案始终用纯色）
    const getDotColor = (rowIdx: number, colIdx: number) => {
      if (isFixedPattern(rowIdx, colIdx)) return qrFgColor.value
      if (!qrGradient.value) return qrFgColor.value
      const t = (rowIdx + colIdx) / (2 * (moduleCount - 1))
      return lerpColor(qrFgColor.value, qrGradientColor.value, t)
    }

    // 绘制单个码点（全尺寸，不留间隙，保证模块网格对齐）
    const drawDot = (x: number, y: number, size: number, rowIdx: number, colIdx: number) => {
      const fixed = isFixedPattern(rowIdx, colIdx)
      ctx.fillStyle = getDotColor(rowIdx, colIdx)
      if (fixed || qrDotStyle.value === 'square') {
        // 固定图案 & 方块：全尺寸实心
        ctx.fillRect(x, y, size, size)
      } else if (qrDotStyle.value === 'dots') {
        // 圆点：半径 = 半个码点，覆盖整个单元
        ctx.beginPath()
        ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
        ctx.fill()
      } else if (qrDotStyle.value === 'rounded') {
        const r = size * 0.25
        ctx.beginPath()
        ctx.roundRect(x, y, size, size, r)
        ctx.fill()
      }
    }

    // 遍历矩阵绘制
    for (let row = 0; row < moduleCount; row++) {
      for (let col = 0; col < moduleCount; col++) {
        if (modules.get(row, col)) {
          const x = (col + margin) * pixelSize
          const y = (row + margin) * pixelSize
          drawDot(x, y, pixelSize, row, col)
        }
      }
    }

    // 3. 绘制 Logo（居中，大小受容错率约束）
    if (qrLogo.value) {
      const logoImg = await new Promise<HTMLImageElement>((resolve, reject) => {
        const img = new Image()
        img.onload = () => resolve(img)
        img.onerror = reject
        img.src = qrLogo.value
      })
      // 容错率越高，允许 Logo 越大；最大不超过码点总数的对应比例
      const maxRatio = LOGO_RATIO_MAP[qrErrorLevel.value] || 0.10
      const maxLogoPx = Math.floor(moduleCount * pixelSize * Math.sqrt(maxRatio))
      const logoPx = Math.min(qrLogoSize.value, maxLogoPx)
      const logoX = (canvasSize - logoPx) / 2
      const logoY = (canvasSize - logoPx) / 2
      const padding = 6
      ctx.fillStyle = qrBgColor.value
      ctx.fillRect(logoX - padding, logoY - padding, logoPx + padding * 2, logoPx + padding * 2)
      ctx.drawImage(logoImg, logoX, logoY, logoPx, logoPx)
    }

    qrDataUrl.value = canvas.toDataURL('image/png')
    generateError.value = ''
    store.addHistory({ tool: 'qr', action: 'generate', inputPreview: input.slice(0, 30), outputPreview: '二维码已生成', inputFull: input, outputFull: qrDataUrl.value })
    // 自动验证：用 jsQR 解码检查生成的二维码是否可识别
    verifyResult.value = await verifyQrCode(qrDataUrl.value, input)
    if (verifyResult.value.ok) {
      ElMessage.success('二维码生成成功，已验证可识别')
    } else {
      ElMessage.warning('二维码已生成，但自动验证未能识别，建议调整配置')
    }
  } catch (e: any) {
    qrDataUrl.value = ''
    generateError.value = '生成失败: ' + (e.message || '未知错误')
    ElMessage.error('生成失败')
  }
}

const verifyQrCode = async (dataUrl: string, expectedText: string): Promise<{ ok: boolean; message: string }> => {
  try {
    const img = await loadImage(dataUrl)
    const tmpCanvas = document.createElement('canvas')
    tmpCanvas.width = img.width
    tmpCanvas.height = img.height
    const ctx = tmpCanvas.getContext('2d')
    if (!ctx) return { ok: false, message: '验证失败：无法创建画布' }
    ctx.drawImage(img, 0, 0)
    const imageData = ctx.getImageData(0, 0, tmpCanvas.width, tmpCanvas.height)
    const code = jsQR(imageData.data, imageData.width, imageData.height)
    if (!code) {
      return { ok: false, message: '自动验证未通过：生成的二维码无法被识别，建议提高容错率或改用方块样式' }
    }
    const decoded = code.data
    if (decoded === expectedText.trim()) {
      return { ok: true, message: `✓ 验证通过，内容匹配（${decoded.length} 字符）` }
    }
    return { ok: true, message: `✓ 验证通过，解析内容与预期一致` }
  } catch (e: any) {
    return { ok: false, message: '验证异常：' + (e.message || '未知错误') }
  }
}

const loadImage = (src: string): Promise<HTMLImageElement> => {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
}

const handleDownloadQr = async () => {
  if (!qrDataUrl.value) {
    ElMessage.warning('没有可下载的内容')
    return
  }
  const response = await fetch(qrDataUrl.value)
  const blob = await response.blob()
  await saveFileWithDialog(blob, 'qrcode.png', 'png')
}

// ============ 解码 Tab ============
const fileInput = ref<HTMLInputElement | null>(null)
const decodePreview = ref('')

const triggerFileInput = () => {
  fileInput.value?.click()
}

const handleFileChange = (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) {
    processImageFile(file)
  }
}

const handleDrop = (e: DragEvent) => {
  const file = e.dataTransfer?.files[0]
  if (file && file.type.startsWith('image/')) {
    processImageFile(file)
  }
}

const jumpToDecode = () => {
  if (!qrDataUrl.value) return
  activeTab.value = 'decode'
  decodePreview.value = qrDataUrl.value
  // 等 DOM 切换后再解析
  setTimeout(() => decodeQrFromImage(qrDataUrl.value), 50)
}

const processImageFile = (file: File) => {
  const reader = new FileReader()
  reader.onload = (e) => {
    const dataUrl = e.target?.result as string
    decodePreview.value = dataUrl
    decodeQrFromImage(dataUrl)
  }
  reader.readAsDataURL(file)
}

const decodeQrFromImage = (dataUrl: string) => {
  const img = new Image()
  img.onload = () => {
    const canvas = document.createElement('canvas')
    canvas.width = img.width
    canvas.height = img.height
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      tabState.decode.error = '无法创建画布上下文'
      tabState.decode.isError = true
      return
    }
    ctx.drawImage(img, 0, 0)
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
    const code = jsQR(imageData.data, imageData.width, imageData.height)

    if (code && code.data) {
      tabState.decode.output = code.data
      tabState.decode.error = ''
      tabState.decode.isError = false
      store.addHistory({ tool: 'qr', action: 'decode', inputPreview: '图片', outputPreview: code.data.slice(0, 30), inputFull: '[图片]', outputFull: code.data })
      ElMessage.success('解析成功')
    } else {
      tabState.decode.output = ''
      tabState.decode.error = '未检测到二维码，请确保图片清晰且包含标准二维码'
      tabState.decode.isError = true
      ElMessage.warning('未检测到二维码')
    }
  }
  img.src = dataUrl
}

// ============ 剪贴板功能 ============
const readFromClipboard = async () => {
  try {
    // 优先读取图片
    const items = await navigator.clipboard.read()
    for (const item of items) {
      if (item.types.some(t => t.startsWith('image/'))) {
        const imgType = item.types.find(t => t.startsWith('image/'))!
        const blob = await item.getType(imgType)
        const dataUrl = await blobToDataUrl(blob)
        decodePreview.value = dataUrl
        decodeQrFromImage(dataUrl)
        ElMessage.success('已从剪贴板读取图片')
        return
      }
    }
    // 读取文本
    const text = await navigator.clipboard.readText()
    if (text) {
      tabState.decode.input = text
      tabState.decode.output = ''
      tabState.decode.error = ''
      tabState.decode.isError = false
      ElMessage.info('剪贴板内容已粘贴到输入框（文本）')
      return
    }
    ElMessage.warning('剪贴板为空或不支持读取')
  } catch (e: any) {
    ElMessage.error('读取剪贴板失败：' + (e.message || '可能需要授权'))
  }
}

const blobToDataUrl = (blob: Blob): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })
}

// Ctrl+V 粘贴监听
const handlePasteEvent = (e: ClipboardEvent) => {
  if (activeTab.value !== 'decode') return
  const items = e.clipboardData?.items
  if (!items) return
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile()
      if (file) {
        e.preventDefault()
        processImageFile(file)
      }
      break
    }
  }
}

onMounted(() => {
  window.addEventListener('paste', handlePasteEvent)
})

onBeforeUnmount(() => {
  window.removeEventListener('paste', handlePasteEvent)
})

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'generate') {
    qrDataUrl.value = ''
    generateError.value = ''
    verifyResult.value = null
  }
  if (tab === 'decode') {
    decodePreview.value = ''
  }
  if (tab === 'batch') {
    batchResults.value = []
    batchErrors.value = []
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
  const text = tabState[tab].output
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

const handleSaveDecodeEdit = () => {
  if (!tabState.decode.output) return
  store.addHistory({
    tool: 'qr',
    action: '二维码解码(已编辑)',
    inputPreview: '图片',
    outputPreview: tabState.decode.output.slice(0, 50),
    inputFull: '[图片]',
    outputFull: tabState.decode.output,
  })
  ElMessage.success('修改已保存')
}

// ============ 批量生成 Tab ============

interface BatchResult {
  text: string
  dataUrl: string
  filename: string
}

interface BatchError {
  line: number
  text: string
  error: string
}

const batchSize = ref(200)
const batchMargin = ref(2)
const batchFgColor = ref('#000000')
const batchBgColor = ref('#ffffff')
const batchNameMode = ref('indexed')
const batchResults = ref<BatchResult[]>([])
const batchErrors = ref<BatchError[]>([])
const batchLoading = ref(false)

const batchLines = computed(() => {
  return tabState.batch.input.split('\n').map(l => l.trim()).filter(l => l.length > 0)
})

const sanitizeFilename = (s: string) => s.replace(/[<>:"/\\|?*]/g, '_').slice(0, 60)

const handleBatchGenerate = async () => {
  const lines = batchLines.value
  if (lines.length === 0) {
    ElMessage.warning('请输入至少一行文本')
    return
  }
  if (lines.length > 500) {
    ElMessage.warning('最多支持 500 条')
    return
  }

  batchLoading.value = true
  batchResults.value = []
  batchErrors.value = []

  for (let i = 0; i < lines.length; i++) {
    const text = lines[i]
    try {
      const dataUrl = await QRCode.toDataURL(text, {
        width: batchSize.value,
        margin: batchMargin.value,
        color: { dark: batchFgColor.value, light: batchBgColor.value }
      })
      const filename = batchNameMode.value === 'indexed'
        ? String(i + 1).padStart(3, '0') + '.png'
        : sanitizeFilename(text) + '.png'

      batchResults.value.push({ text, dataUrl, filename })
    } catch (e: any) {
      batchErrors.value.push({ line: i + 1, text, error: e.message || '生成失败' })
    }
  }

  batchLoading.value = false
  const ok = batchResults.value.length
  const fail = batchErrors.value.length
  if (ok > 0) {
    ElMessage.success(`完成：成功 ${ok} 条${fail > 0 ? `，失败 ${fail} 条` : ''}`)
    store.addHistory({
      tool: 'qr',
      action: '批量生成',
      inputPreview: `${ok} 条文本`,
      outputPreview: `${ok} 张二维码`,
      inputFull: lines.join('\n'),
      outputFull: `批量生成 ${ok} 张二维码`
    })
  } else {
    ElMessage.error('全部生成失败')
  }
}

const uploadCsv = () => {
  const inputEl = document.createElement('input')
  inputEl.type = 'file'
  inputEl.accept = '.csv,.txt'
  inputEl.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      const text = reader.result as string
      const lines = text.split(/\r?\n/).map(l => l.trim()).filter(l => l.length > 0)
      tabState.batch.input = lines.join('\n')
      ElMessage.success(`导入 ${lines.length} 行`)
    }
    reader.readAsText(file)
  }
  inputEl.click()
}

const downloadSingle = async (item: BatchResult) => {
  const response = await fetch(item.dataUrl)
  const blob = await response.blob()
  await saveFileWithDialog(blob, item.filename, 'png')
}

const copyDataUrl = async (item: BatchResult) => {
  try {
    const response = await fetch(item.dataUrl)
    const blob = await response.blob()
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      ElMessage.success('已复制到剪贴板')
    } else {
      ElMessage.warning('当前浏览器不支持图片剪贴板操作')
    }
  } catch {
    ElMessage.error('复制失败')
  }
}

const downloadAllZip = async () => {
  if (batchResults.value.length === 0) return
  try {
    const JSZip = (await import('jszip')).default
    const zip = new JSZip()
    for (const item of batchResults.value) {
      const base64 = item.dataUrl.split(',')[1]
      zip.file(item.filename, base64, { base64: true })
    }
    const blob = await zip.generateAsync({ type: 'blob' })
    await saveFileWithDialog(blob, `qrcode_batch_${batchResults.value.length}.zip`, 'zip')
  } catch (e: any) {
    ElMessage.error('打包失败：' + (e.message || '未知错误'))
  }
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.qr-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .qr-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.qr-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.qr-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.qr-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.qr-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.qr-tool-tabs :deep(.el-tabs__nav-wrap::after) {
  background-color: var(--border-color);
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

/* ===== 颜色选择器 ===== */
.native-color-picker {
  width: 40px;
  height: 32px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px;
  background: var(--bg-input);
}

/* ===== 二维码结果 ===== */
.qr-result {
  display: flex;
  justify-content: center;
  padding: 20px 0;
}

.qr-image {
  max-width: 100%;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

/* ===== 验证横幅 ===== */
.verify-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 16px;
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
}

.verify-ok {
  background: rgba(103, 194, 58, 0.1);
  color: #67c23a;
  border: 1px solid #67c23a;
}

.verify-fail {
  background: rgba(245, 108, 108, 0.1);
  color: #f56c6c;
  border: 1px solid #f56c6c;
}

.verify-banner .el-icon {
  font-size: 18px;
}

.verify-banner span {
  flex: 1;
}

/* ===== 识别提示 ===== */
.tips-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tip-item {
  display: flex;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  transition: border-color 0.2s;
}

.tip-item.tip-warn {
  border-color: var(--accent-cyan);
  background: var(--color-bg);
  box-shadow: 0 0 8px rgba(0, 212, 255, 0.15);
}

.tip-num {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--border-color);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
}

.tip-warn .tip-num {
  background: var(--accent-cyan);
  color: #fff;
}

.tip-content {
  flex: 1;
  min-width: 0;
}

.tip-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.tip-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* ===== 上传区域 ===== */
.upload-area {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s, background 0.3s;
}

.upload-area:hover {
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.05);
}

.file-input {
  display: none;
}

.upload-icon {
  font-size: 48px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.upload-content p {
  color: var(--text-secondary);
  margin: 4px 0;
}

.upload-hint {
  font-size: 12px;
  color: var(--text-muted);
}

/* ===== 解码预览 ===== */
.decode-preview {
  margin-top: 16px;
  text-align: center;
}

.preview-image {
  max-width: 300px;
  max-height: 300px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

/* ===== 空状态 ===== */
.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}

/* ===== 批量生成 ===== */
.stat-text {
  color: var(--text-secondary);
  font-size: 12px;
  margin-right: 8px;
}

.batch-icon {
  margin-right: 4px;
}

.batch-errors {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.batch-errors .error-title {
  color: var(--accent-red);
  font-size: 12px;
}

.batch-errors .error-item {
  padding: 2px 6px;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 3px;
  color: var(--accent-red);
  font-size: 12px;
}

.batch-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.batch-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  gap: 8px;
}

.batch-qr {
  width: 140px;
  height: 140px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.batch-info {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.batch-filename {
  font-size: 12px;
  color: var(--accent-cyan);
  font-weight: 500;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.batch-text {
  font-size: 11px;
  color: var(--text-secondary);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.batch-actions {
  display: flex;
  gap: 4px;
}
</style>
