<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="color-tool-tabs" @tab-click="handleTabClick">

      <!-- Tab 1: 颜色选择器 -->
      <el-tab-pane label="选择器" name="picker">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>选择或输入颜色，实时查看各格式值</p>
                    <p>支持 HEX / RGB / HSL / HSB 格式</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleRandomColor">随机颜色</el-button>
              <el-button size="small" @click="handleCopy('picker')">复制</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">复制格式</div>
                <el-select v-model="pickerCopyFormat" size="small" style="width: 120px">
                  <el-option label="HEX" value="hex" />
                  <el-option label="RGB" value="rgb" />
                  <el-option label="HSL" value="hsl" />
                </el-select>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">颜色输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handlePaste('picker')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="picker-input-row">
              <input type="color" v-model="pickerColor" class="native-color-picker" />
              <el-input v-model="tabState.picker.input" placeholder="输入颜色值，如 #00d4ff、rgb(0,212,255)" @input="handlePickerInput" />
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">颜色值</span>
          </div>
          <div class="card-body">
            <div v-if="pickerResult" class="color-preview-section">
              <div class="color-swatch-large" :style="{ background: pickerResult.hex }"></div>
              <div class="color-values">
                <div class="color-value-item" @click="copyValue(pickerResult.hex)">
                  <span class="label">HEX</span>
                  <span class="value">{{ pickerResult.hex }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(pickerResult.rgb)">
                  <span class="label">RGB</span>
                  <span class="value">{{ pickerResult.rgb }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(pickerResult.hsl)">
                  <span class="label">HSL</span>
                  <span class="value">{{ pickerResult.hsl }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(pickerResult.hsv)">
                  <span class="label">HSV</span>
                  <span class="value">{{ pickerResult.hsv }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
              </div>
            </div>
            <div v-else class="stats-empty">
              输入颜色值后自动显示结果
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 颜色转换 -->
      <el-tab-pane label="转换" name="convert">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('convert')">清空</el-button>
              <el-button size="small" @click="handlePaste('convert')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">输入格式</div>
                <el-select v-model="convertInputFormat" size="small" style="width: 120px">
                  <el-option label="自动识别" value="auto" />
                  <el-option label="HEX" value="hex" />
                  <el-option label="RGB" value="rgb" />
                  <el-option label="HSL" value="hsl" />
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
              <el-button size="small" @click="handleClear('convert')">清空</el-button>
              <el-button size="small" @click="handlePaste('convert')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input
              v-model="tabState.convert.input"
              type="textarea"
              :rows="4"
              placeholder="输入颜色值，如 #00d4ff、rgb(0,212,255)、hsl(191,100%,50%)"
              resize="vertical"
            />
          </div>
        </div>

        <!-- 输出卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopy('convert')">复制</el-button>
          </div>
          <div class="card-body">
            <div v-if="convertResult" class="color-preview-section">
              <div class="color-swatch-large" :style="{ background: convertResult.hex }"></div>
              <div class="color-values">
                <div class="color-value-item" @click="copyValue(convertResult.hex)">
                  <span class="label">HEX</span>
                  <span class="value">{{ convertResult.hex }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(convertResult.rgb)">
                  <span class="label">RGB</span>
                  <span class="value">{{ convertResult.rgb }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(convertResult.hsl)">
                  <span class="label">HSL</span>
                  <span class="value">{{ convertResult.hsl }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
                <div class="color-value-item" @click="copyValue(convertResult.hsv)">
                  <span class="label">HSV</span>
                  <span class="value">{{ convertResult.hsv }}</span>
                  <span class="copy-hint">点击复制</span>
                </div>
              </div>
            </div>
            <div v-else-if="tabState.convert.error" class="error-message">
              {{ tabState.convert.error }}
            </div>
            <div v-else class="stats-empty">
              点击"转换"按钮查看结果
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 色板生成 -->
      <el-tab-pane label="色板" name="palette">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>输入基准色，自动生成配套色板</p>
                    <p>支持互补色、三角色、类似色、分裂互补色</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('palette')">清空</el-button>
              <el-button size="small" @click="handlePaste('palette')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">色板类型</div>
                <el-select v-model="paletteType" size="small" style="width: 140px">
                  <el-option label="互补色" value="complementary" />
                  <el-option label="三角色" value="triadic" />
                  <el-option label="类似色" value="analogous" />
                  <el-option label="分裂互补" value="split-complementary" />
                  <el-option label="四角色" value="tetradic" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handlePalette">生成色板</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">基准色</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('palette')">清空</el-button>
              <el-button size="small" @click="handlePaste('palette')">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="picker-input-row">
              <input type="color" v-model="paletteBaseColor" class="native-color-picker" />
              <el-input v-model="tabState.palette.input" placeholder="输入基准色，如 #00d4ff" @input="handlePaletteInput" />
            </div>
          </div>
        </div>

        <!-- 色板结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">色板结果</span>
            <el-button size="small" @click="handleCopyPalette">复制全部</el-button>
          </div>
          <div class="card-body">
            <div v-if="paletteResult.length > 0" class="palette-grid">
              <div v-for="(color, idx) in paletteResult" :key="idx" class="palette-item" @click="copyValue(color.hex)">
                <div class="palette-swatch" :style="{ background: color.hex }"></div>
                <div class="palette-info">
                  <span class="palette-hex">{{ color.hex }}</span>
                </div>
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"生成色板"查看结果
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: 对比度检查 -->
      <el-tab-pane label="对比度" name="contrast">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>检查前景色和背景色的对比度是否符合 WCAG 2.1 标准</p>
                    <p>AA 级：正常文本 ≥ 4.5:1，大文本 ≥ 3:1</p>
                    <p>AAA 级：正常文本 ≥ 7:1，大文本 ≥ 4.5:1</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('contrast')">清空</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleContrast">检查对比度</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">颜色输入</span>
          </div>
          <div class="card-body">
            <div class="contrast-inputs">
              <div class="contrast-input-item">
                <span class="contrast-label">前景色</span>
                <input type="color" v-model="contrastFg" class="native-color-picker" />
                <el-input v-model="contrastFgText" placeholder="#000000" @input="handleContrastFgInput" />
              </div>
              <div class="contrast-input-item">
                <span class="contrast-label">背景色</span>
                <input type="color" v-model="contrastBg" class="native-color-picker" />
                <el-input v-model="contrastBgText" placeholder="#ffffff" @input="handleContrastBgInput" />
              </div>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">对比度结果</span>
          </div>
          <div class="card-body">
            <div v-if="contrastResult" class="contrast-result">
              <div class="contrast-preview" :style="{ backgroundColor: contrastResult.bg, color: contrastResult.fg }">
                预览文本 Preview Text
              </div>
              <div class="contrast-ratio">
                <span class="ratio-value">{{ contrastResult.ratio }}:1</span>
                <span class="ratio-label">对比度</span>
              </div>
              <div class="contrast-standards">
                <div class="standard-item" :class="{ pass: contrastResult.aaNormal }">
                  <span class="standard-name">AA 正常文本</span>
                  <span class="standard-status">{{ contrastResult.aaNormal ? '✓ 通过' : '✗ 未通过' }}</span>
                  <span class="standard-require">≥ 4.5:1</span>
                </div>
                <div class="standard-item" :class="{ pass: contrastResult.aaLarge }">
                  <span class="standard-name">AA 大文本</span>
                  <span class="standard-status">{{ contrastResult.aaLarge ? '✓ 通过' : '✗ 未通过' }}</span>
                  <span class="standard-require">≥ 3:1</span>
                </div>
                <div class="standard-item" :class="{ pass: contrastResult.aaaNormal }">
                  <span class="standard-name">AAA 正常文本</span>
                  <span class="standard-status">{{ contrastResult.aaaNormal ? '✓ 通过' : '✗ 未通过' }}</span>
                  <span class="standard-require">≥ 7:1</span>
                </div>
                <div class="standard-item" :class="{ pass: contrastResult.aaaLarge }">
                  <span class="standard-name">AAA 大文本</span>
                  <span class="standard-status">{{ contrastResult.aaaLarge ? '✓ 通过' : '✗ 未通过' }}</span>
                  <span class="standard-require">≥ 4.5:1</span>
                </div>
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"检查对比度"查看结果
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 5: 渐变生成 -->
      <el-tab-pane label="渐变" name="gradient">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <div class="header-left">
              <span class="card-title">操作</span>
              <el-tooltip placement="top" effect="dark">
                <template #content>
                  <div class="tooltip-content">
                    <p>选择渐变类型和颜色，生成 CSS 渐变代码</p>
                    <p>支持线性渐变和径向渐变</p>
                  </div>
                </template>
                <el-icon class="hint-icon"><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
            <div class="card-actions">
              <el-button size="small" @click="handleClear('gradient')">清空</el-button>
              <el-button size="small" @click="handleCopy('gradient')">复制 CSS</el-button>
            </div>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">渐变类型</div>
                <el-select v-model="gradientType" size="small" style="width: 120px" @change="handleGradientTypeChange">
                  <el-option label="线性" value="linear" />
                  <el-option label="径向" value="radial" />
                </el-select>
              </div>
              <div class="action-group">
                <div class="group-label">角度</div>
                <el-input-number v-model="gradientAngle" :min="0" :max="360" size="small" style="width: 100px" v-if="gradientType === 'linear'" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <div class="group-buttons">
                  <el-button type="primary" size="small" @click="handleGradient">生成渐变</el-button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 颜色输入卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">渐变颜色</span>
          </div>
          <div class="card-body">
            <div class="gradient-colors">
              <div v-for="(_, idx) in gradientColors" :key="idx" class="gradient-color-item">
                <input type="color" v-model="gradientColors[idx]" class="native-color-picker" />
                <el-input v-model="gradientColors[idx]" size="small" style="width: 120px" />
                <el-button size="small" @click="removeGradientColor(idx)" :disabled="gradientColors.length <= 2">删除</el-button>
              </div>
              <el-button size="small" @click="addGradientColor" :disabled="gradientColors.length >= 5">+ 添加颜色</el-button>
            </div>
          </div>
        </div>

        <!-- 结果卡片 -->
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">渐变预览</span>
          </div>
          <div class="card-body">
            <div v-if="gradientResult" class="gradient-result">
              <div class="gradient-preview" :style="{ background: gradientResult.css }"></div>
              <div class="gradient-code">
                <el-input :model-value="gradientResult.css" readonly size="small" />
              </div>
            </div>
            <div v-else class="stats-empty">
              点击"生成渐变"查看结果
            </div>
          </div>
        </div>
      </el-tab-pane>

    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled } from '@element-plus/icons-vue'
import { colord, extend } from 'colord'
import mixPlugin from 'colord/plugins/mix'
import namesPlugin from 'colord/plugins/names'
import a11yPlugin from 'colord/plugins/a11y'

extend([mixPlugin, namesPlugin, a11yPlugin])

// ============ Tab 状态 ============
const activeTab = ref('picker')

const tabState = reactive<Record<string, { input: string; output: string; error: string; isError: boolean }>>({
  picker: { input: '#00d4ff', output: '', error: '', isError: false },
  convert: { input: '', output: '', error: '', isError: false },
  palette: { input: '#00d4ff', output: '', error: '', isError: false },
  contrast: { input: '', output: '', error: '', isError: false },
  gradient: { input: '', output: '', error: '', isError: false }
})

// ============ 选择器 Tab ============
const pickerColor = ref('#00d4ff')
const pickerCopyFormat = ref('hex')

interface PickerResult {
  hex: string
  rgb: string
  hsl: string
  hsv: string
}

const pickerResult = ref<PickerResult | null>(null)

const handlePickerInput = () => {
  const input = tabState.picker.input.trim()
  if (!input) {
    pickerResult.value = null
    return
  }
  const c = colord(input)
  if (!c.isValid()) {
    pickerResult.value = null
    return
  }
  pickerColor.value = c.toHex()
  updatePickerResult(c)
}

const updatePickerResult = (c: any) => {
  const rgb = c.toRgb()
  const hsl = c.toHsl()
  const hsv = c.toHsv()
  pickerResult.value = {
    hex: c.toHex(),
    rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
    hsl: `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
    hsv: `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`
  }
}

const handleRandomColor = () => {
  const hex = '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0')
  tabState.picker.input = hex
  pickerColor.value = hex
  const c = colord(hex)
  updatePickerResult(c)
}

// 初始化
const initPicker = () => {
  const c = colord(tabState.picker.input)
  if (c.isValid()) {
    pickerColor.value = c.toHex()
    updatePickerResult(c)
  }
}
initPicker()

// ============ 转换 Tab ============
const convertInputFormat = ref('auto')
const convertResult = ref<PickerResult | null>(null)

const handleConvert = () => {
  const input = tabState.convert.input.trim()
  if (!input) {
    ElMessage.warning('请输入颜色值')
    return
  }

  const c = colord(input)
  if (!c.isValid()) {
    tabState.convert.output = ''
    tabState.convert.error = '无效的颜色值，请检查输入格式'
    tabState.convert.isError = true
    convertResult.value = null
    return
  }

  const rgb = c.toRgb()
  const hsl = c.toHsl()
  const hsv = c.toHsv()
  convertResult.value = {
    hex: c.toHex(),
    rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
    hsl: `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
    hsv: `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`
  }
  tabState.convert.output = convertResult.value.hex
  tabState.convert.error = ''
  tabState.convert.isError = false
  ElMessage.success('转换成功')
}

// ============ 色板 Tab ============
const paletteType = ref('complementary')
const paletteBaseColor = ref('#00d4ff')
const paletteResult = ref<PickerResult[]>([])

const handlePaletteInput = () => {
  const input = tabState.palette.input.trim()
  if (!input) return
  const c = colord(input)
  if (c.isValid()) {
    paletteBaseColor.value = c.toHex()
  }
}

const handlePalette = () => {
  const input = tabState.palette.input.trim() || paletteBaseColor.value
  const c = colord(input)
  if (!c.isValid()) {
    ElMessage.error('无效的颜色值')
    return
  }

  let colors: any[] = []
  switch (paletteType.value) {
    case 'complementary':
      colors = [c, c.rotate(180)]
      break
    case 'triadic':
      colors = [c, c.rotate(120), c.rotate(240)]
      break
    case 'analogous':
      colors = [c.rotate(-30), c, c.rotate(30)]
      break
    case 'split-complementary':
      colors = [c, c.rotate(150), c.rotate(210)]
      break
    case 'tetradic':
      colors = [c, c.rotate(90), c.rotate(180), c.rotate(270)]
      break
  }

  paletteResult.value = colors.map(color => {
    const rgb = color.toRgb()
    const hsl = color.toHsl()
    const hsv = color.toHsv()
    return {
      hex: color.toHex(),
      rgb: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
      hsl: `hsl(${Math.round(hsl.h)}, ${Math.round(hsl.s)}%, ${Math.round(hsl.l)}%)`,
      hsv: `hsv(${Math.round(hsv.h)}, ${Math.round(hsv.s)}%, ${Math.round(hsv.v)}%)`
    }
  })

  ElMessage.success('色板生成成功')
}

// ============ 对比度 Tab ============
const contrastFg = ref('#000000')
const contrastBg = ref('#ffffff')
const contrastFgText = ref('#000000')
const contrastBgText = ref('#ffffff')

interface ContrastResult {
  fg: string
  bg: string
  ratio: string
  aaNormal: boolean
  aaLarge: boolean
  aaaNormal: boolean
  aaaLarge: boolean
}

const contrastResult = ref<ContrastResult | null>(null)

const handleContrastFgInput = () => {
  const c = colord(contrastFgText.value)
  if (c.isValid()) {
    contrastFg.value = c.toHex()
  }
}

const handleContrastBgInput = () => {
  const c = colord(contrastBgText.value)
  if (c.isValid()) {
    contrastBg.value = c.toHex()
  }
}

const handleContrast = () => {
  const fg = colord(contrastFg.value)
  const bg = colord(contrastBg.value)

  if (!fg.isValid() || !bg.isValid()) {
    ElMessage.error('无效的颜色值')
    return
  }

  const ratio = fg.contrast(bg)
  contrastResult.value = {
    fg: fg.toHex(),
    bg: bg.toHex(),
    ratio: ratio.toFixed(2),
    aaNormal: ratio >= 4.5,
    aaLarge: ratio >= 3,
    aaaNormal: ratio >= 7,
    aaaLarge: ratio >= 4.5
  }

  ElMessage.success('对比度检查完成')
}

// ============ 渐变 Tab ============
const gradientType = ref('linear')
const gradientAngle = ref(135)
const gradientColors = ref(['#00d4ff', '#0891b2'])
const gradientResult = ref<{ css: string } | null>(null)

const handleGradientTypeChange = () => {}

const addGradientColor = () => {
  if (gradientColors.value.length < 5) {
    const randomColor = '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0')
    gradientColors.value.push(randomColor)
  }
}

const removeGradientColor = (idx: number) => {
  if (gradientColors.value.length > 2) {
    gradientColors.value.splice(idx, 1)
  }
}

const handleGradient = () => {
  if (gradientColors.value.length < 2) {
    ElMessage.warning('至少需要 2 个颜色')
    return
  }

  let css: string
  if (gradientType.value === 'linear') {
    css = `linear-gradient(${gradientAngle.value}deg, ${gradientColors.value.join(', ')})`
  } else {
    css = `radial-gradient(circle, ${gradientColors.value.join(', ')})`
  }

  gradientResult.value = { css }
  tabState.gradient.output = css
  ElMessage.success('渐变生成成功')
}

// ============ 通用方法 ============
const handleTabClick = () => {}

const handleClear = (tab: string) => {
  tabState[tab].input = ''
  tabState[tab].output = ''
  tabState[tab].error = ''
  tabState[tab].isError = false
  if (tab === 'convert') convertResult.value = null
  if (tab === 'palette') {
    paletteResult.value = []
    tabState.palette.input = '#00d4ff'
    paletteBaseColor.value = '#00d4ff'
  }
  if (tab === 'contrast') {
    contrastResult.value = null
    contrastFg.value = '#000000'
    contrastBg.value = '#ffffff'
    contrastFgText.value = '#000000'
    contrastBgText.value = '#ffffff'
  }
  if (tab === 'gradient') {
    gradientResult.value = null
    gradientColors.value = ['#00d4ff', '#0891b2']
  }
  if (tab === 'picker') {
    pickerResult.value = null
    pickerColor.value = '#00d4ff'
    tabState.picker.input = '#00d4ff'
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
  let text = tabState[tab].output || tabState[tab].input
  if (tab === 'picker' && pickerCopyFormat.value && pickerResult.value) {
    text = pickerResult.value[pickerCopyFormat.value as keyof PickerResult]
  }
  if (tab === 'gradient' && gradientResult.value) {
    text = gradientResult.value.css
  }
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

const copyValue = async (value: string) => {
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success(`已复制: ${value}`)
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleCopyPalette = async () => {
  if (paletteResult.value.length === 0) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  const text = paletteResult.value.map(c => c.hex).join('\n')
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('色板已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
/* ===== 一级 Tab 样式 ===== */
.color-tool-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

html.light .color-tool-tabs :deep(.el-tabs__header) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.color-tool-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.color-tool-tabs :deep(.el-tabs__item) {
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.color-tool-tabs :deep(.el-tabs__item.is-active) {
  color: var(--accent-cyan);
}

.color-tool-tabs :deep(.el-tabs__active-bar) {
  background-color: var(--accent-cyan);
}

.color-tool-tabs :deep(.el-tabs__nav-wrap::after) {
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

/* ===== 颜色选择器 ===== */
.picker-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.native-color-picker {
  width: 48px;
  height: 36px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px;
  background: var(--bg-input);
  flex-shrink: 0;
}

/* 颜色预览 */
.color-preview-section {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.color-swatch-large {
  width: 100px;
  height: 100px;
  border-radius: 8px;
  border: 2px solid var(--border-color);
  flex-shrink: 0;
}

.color-values {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.color-value-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.color-value-item:hover {
  background: rgba(0, 212, 255, 0.1);
}

.color-value-item .label {
  font-weight: 600;
  color: var(--accent-cyan);
  min-width: 40px;
  font-size: 13px;
}

.color-value-item .value {
  flex: 1;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  color: var(--text-primary);
}

.color-value-item .copy-hint {
  color: var(--text-muted);
  font-size: 12px;
}

/* ===== 色板网格 ===== */
.palette-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 12px;
}

.palette-item {
  cursor: pointer;
  transition: transform 0.2s;
}

.palette-item:hover {
  transform: translateY(-2px);
}

.palette-swatch {
  width: 100%;
  height: 80px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.palette-info {
  padding: 6px 0;
  text-align: center;
}

.palette-hex {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
}

/* ===== 对比度 ===== */
.contrast-inputs {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.contrast-input-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.contrast-label {
  min-width: 60px;
  color: var(--text-secondary);
  font-size: 13px;
}

.contrast-result {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.contrast-preview {
  padding: 24px;
  border-radius: 8px;
  text-align: center;
  font-size: 18px;
  font-weight: 600;
  border: 1px solid var(--border-color);
}

.contrast-ratio {
  text-align: center;
}

.ratio-value {
  font-size: 36px;
  font-weight: 700;
  color: var(--accent-cyan);
}

.ratio-label {
  display: block;
  color: var(--text-secondary);
  font-size: 13px;
  margin-top: 4px;
}

.contrast-standards {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.standard-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg-input);
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.standard-item.pass {
  border-color: rgba(34, 197, 94, 0.3);
  background: rgba(34, 197, 94, 0.05);
}

.standard-name {
  font-size: 13px;
  color: var(--text-primary);
}

.standard-status {
  font-size: 13px;
  font-weight: 600;
}

.standard-item.pass .standard-status {
  color: #22c55e;
}

.standard-item:not(.pass) .standard-status {
  color: var(--accent-red);
}

.standard-require {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* ===== 渐变 ===== */
.gradient-colors {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.gradient-color-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gradient-result {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.gradient-preview {
  width: 100%;
  height: 120px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.gradient-code {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* ===== 空状态 ===== */
.stats-empty {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-style: italic;
}
</style>
