<template>
  <div class="tool-container">
    <!-- Tab 栏 -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="image-tabs">
        <el-tab-pane label="图片压缩" name="compress" />
        <el-tab-pane label="尺寸缩放" name="resize" />
        <el-tab-pane label="图片转Base64" name="base64" />
      </el-tabs>
    </div>

    <!-- 公共上传区域 -->
    <div class="tool-card">
      <div class="card-header">
        <span class="card-title">图片输入</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerFileInput">上传文件</el-button>
          <el-button v-if="imageFile" size="small" @click="handleClearImage">移除</el-button>
        </div>
      </div>
      <div
        class="card-body upload-area"
        :class="{ 'drag-over': isDragging }"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <input
          ref="fileInputRef"
          type="file"
          accept="image/png,image/jpeg,image/webp,image/bmp"
          style="display: none"
          @change="handleFileSelect"
        />
        <div v-if="imageFile" class="image-info">
          <span class="info-name">{{ imageFile.name }}</span>
          <span class="info-size">{{ formatFileSize(imageInfo?.size || 0) }}</span>
          <span class="info-dimensions">{{ imageInfo?.width }}×{{ imageInfo?.height }}</span>
        </div>
        <div v-else class="upload-hint">
          点击「上传文件」或拖拽图片到此处
        </div>
      </div>
    </div>

    <!-- Tab 1: 图片压缩 -->
    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩设置</span>
      </div>
      <div class="card-body">
        <div class="compress-controls">
          <label class="control-label">压缩质量: {{ compressQuality }}%</label>
          <input
            type="range"
            v-model.number="compressQuality"
            min="10"
            max="100"
            step="5"
            class="quality-slider"
          />
          <div class="slider-labels">
            <span>10%</span><span>50%</span><span>100%</span>
          </div>
        </div>
        <div class="action-grid">
          <div class="action-group">
            <el-button size="small" type="primary" :disabled="!imageFile" @click="handleCompress">
              开始压缩
            </el-button>
            <el-button size="small" :disabled="!compressedBlob" @click="handleDownloadCompressed">
              下载压缩图
            </el-button>
            <el-button size="small" :disabled="!compressedBlob" @click="handleCompressToBase64">
              转Base64
            </el-button>
          </div>
        </div>
        <div v-if="compressedBlob" class="result-info">
          <span>原图: {{ formatFileSize(imageInfo?.size || 0) }}</span>
          <span class="arrow">→</span>
          <span>压缩后: {{ formatFileSize(compressedBlob.size) }}</span>
          <span class="ratio">({{ ((compressedBlob.size / (imageInfo?.size || 1)) * 100).toFixed(1) }}%)</span>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 2: 尺寸缩放 -->
    <div v-if="activeTab === 'resize'" class="tool-card">
      <div class="card-header">
        <span class="card-title">缩放设置</span>
      </div>
      <div class="card-body">
        <div class="resize-controls">
          <div class="input-row">
            <label>宽度 (px)</label>
            <input
              type="number"
              v-model.number="resizeWidth"
              :disabled="lockAspect && !resizeTarget"
              class="resize-input"
              @input="handleWidthChange"
            />
          </div>
          <div class="input-row">
            <label>高度 (px)</label>
            <input
              type="number"
              v-model.number="resizeHeight"
              :disabled="lockAspect && !resizeTarget"
              class="resize-input"
              @input="handleHeightChange"
            />
          </div>
          <div class="input-row">
            <label>百分比 (%)</label>
            <input
              type="number"
              v-model.number="resizePercent"
              class="resize-input"
              @input="handlePercentChange"
            />
          </div>
          <div class="lock-row">
            <el-checkbox v-model="lockAspect">等比例缩放</el-checkbox>
          </div>
        </div>
        <div class="action-grid">
          <div class="action-group">
            <el-button size="small" type="primary" :disabled="!imageFile" @click="handleResize">
              开始缩放
            </el-button>
            <el-button size="small" :disabled="!resizedBlob" @click="handleDownloadResized">
              下载缩放图
            </el-button>
          </div>
        </div>
        <div v-if="resizedBlob" class="result-info">
          <span>原尺寸: {{ imageInfo?.width }}×{{ imageInfo?.height }}</span>
          <span class="arrow">→</span>
          <span>新尺寸: {{ resizeWidth }}×{{ resizeHeight }}</span>
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>

    <!-- Tab 3: 图片转Base64 -->
    <div v-if="activeTab === 'base64'" class="tool-card">
      <div class="card-header">
        <span class="card-title">Base64 输出</span>
        <div class="card-actions">
          <el-button size="small" :disabled="!base64Result" @click="handleCopyBase64">复制</el-button>
          <el-button size="small" :disabled="!base64Result" @click="handleDownloadBase64">下载.txt</el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <el-button size="small" type="primary" :disabled="!imageFile" @click="handleToBase64">
              生成Base64
            </el-button>
          </div>
        </div>
        <div v-if="base64Result" class="base64-info">
          <span>大小: {{ formatFileSize(base64Result.length) }}</span>
          <span>格式: {{ imageInfo?.type }}</span>
        </div>
        <el-input
          v-model="base64Result"
          type="textarea"
          :rows="10"
          placeholder="生成Base64后在此显示..."
          readonly
        />
        <div v-if="error" class="error-message">{{ error }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElLoading } from 'element-plus'
import {
  compressImage,
  resizeImage,
  imageToBase64,
  getImageInfo,
  saveFileWithDialog,
  copyBase64,
  formatFileSize,
  type ImageInfo
} from '@/utils/imageUtils'

const fileInputRef = ref<HTMLInputElement | null>(null)
const activeTab = ref('compress')
const imageFile = ref<File | null>(null)
const imageInfo = ref<ImageInfo | null>(null)
const error = ref('')

// 压缩
const compressQuality = ref(80)
const compressedBlob = ref<Blob | null>(null)

// 缩放
const resizeWidth = ref(0)
const resizeHeight = ref(0)
const resizePercent = ref(100)
const lockAspect = ref(true)
const resizeTarget = ref<'width' | 'height' | 'percent'>('percent')
const resizedBlob = ref<Blob | null>(null)

// Base64
const base64Result = ref('')

const triggerFileInput = () => {
  fileInputRef.value?.click()
}

const handleFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  await loadImage(file)
  input.value = ''
}

const isDragging = ref(false)

const handleDragOver = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = true
}

const handleDragLeave = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
}

const handleDrop = (e: DragEvent) => {
  e.preventDefault()
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (!files || files.length === 0) return
  const file = files[0]
  if (!file.type.startsWith('image/')) {
    ElMessage.warning('请拖拽图片文件')
    return
  }
  loadImage(file)
}

const loadImage = async (file: File) => {
  error.value = ''
  const maxSize = 50 * 1024 * 1024 // 50MB
  if (file.size > maxSize) {
    error.value = '图片过大，建议小于 50MB'
    return
  }
  imageFile.value = file
  compressedBlob.value = null
  resizedBlob.value = null
  base64Result.value = ''

  try {
    imageInfo.value = await getImageInfo(file)
    resizeWidth.value = imageInfo.value.width
    resizeHeight.value = imageInfo.value.height
    resizePercent.value = 100
  } catch (e: any) {
    error.value = e.message || '图片加载失败'
  }
}

const handleClearImage = () => {
  imageFile.value = null
  imageInfo.value = null
  compressedBlob.value = null
  resizedBlob.value = null
  base64Result.value = ''
  error.value = ''
  if (fileInputRef.value) fileInputRef.value.value = ''
}

// 压缩
const handleCompress = async () => {
  if (!imageFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: '正在压缩图片，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    compressedBlob.value = await compressImage(imageFile.value, compressQuality.value)
    ElMessage.success('压缩完成')
  } catch (e: any) {
    error.value = e.message || '压缩失败'
  } finally {
    loading.close()
  }
}

const handleDownloadCompressed = async () => {
  if (!compressedBlob.value) return
  await saveFileWithDialog(compressedBlob.value, 'compressed.jpg', 'jpg')
}

const handleCompressToBase64 = async () => {
  if (!compressedBlob.value) return
  const file = new File([compressedBlob.value], 'compressed.jpg', { type: 'image/jpeg' })
  try {
    base64Result.value = await imageToBase64(file)
    activeTab.value = 'base64'
    ElMessage.success('已转为Base64')
  } catch (e: any) {
    error.value = e.message || '转换失败'
  }
}

// 缩放
const handleWidthChange = () => {
  if (lockAspect.value && imageInfo.value) {
    resizeTarget.value = 'width'
    resizeHeight.value = Math.round(resizeWidth.value * (imageInfo.value.height / imageInfo.value.width))
    resizePercent.value = Math.round((resizeWidth.value / imageInfo.value.width) * 100)
  }
}

const handleHeightChange = () => {
  if (lockAspect.value && imageInfo.value) {
    resizeTarget.value = 'height'
    resizeWidth.value = Math.round(resizeHeight.value * (imageInfo.value.width / imageInfo.value.height))
    resizePercent.value = Math.round((resizeHeight.value / imageInfo.value.height) * 100)
  }
}

const handlePercentChange = () => {
  if (imageInfo.value) {
    resizeTarget.value = 'percent'
    resizeWidth.value = Math.round(imageInfo.value.width * (resizePercent.value / 100))
    resizeHeight.value = Math.round(imageInfo.value.height * (resizePercent.value / 100))
  }
}

const handleResize = async () => {
  if (!imageFile.value || !resizeWidth.value || !resizeHeight.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: '正在缩放图片，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    resizedBlob.value = await resizeImage(imageFile.value, resizeWidth.value, resizeHeight.value)
    ElMessage.success('缩放完成')
  } catch (e: any) {
    error.value = e.message || '缩放失败'
  } finally {
    loading.close()
  }
}

const handleDownloadResized = async () => {
  if (!resizedBlob.value) return
  const name = imageFile.value?.name.replace(/\.[^.]+$/, '') || 'image'
  const ext = imageFile.value?.type === 'image/png' ? 'png' : 'jpg'
  await saveFileWithDialog(resizedBlob.value, `${name}_resized.${ext}`, ext)
}

// Base64
const handleToBase64 = async () => {
  if (!imageFile.value) return
  error.value = ''
  const loading = ElLoading.service({
    lock: true,
    text: '正在转换为 Base64，请稍候...',
    background: 'rgba(0, 0, 0, 0.7)',
  })
  try {
    base64Result.value = await imageToBase64(imageFile.value)
    ElMessage.success('Base64生成完成')
  } catch (e: any) {
    error.value = e.message || '转换失败'
  } finally {
    loading.close()
  }
}

const handleCopyBase64 = async () => {
  try {
    await copyBase64(base64Result.value)
    ElMessage.success('已复制Base64')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleDownloadBase64 = async () => {
  const blob = new Blob([base64Result.value], { type: 'text/plain' })
  await saveFileWithDialog(blob, 'image-base64.txt', 'txt')
}
</script>

<style scoped>
.tool-container {
  height: 100vh;
  overflow-y: auto;
  padding: 20px;
  background: var(--bg-primary);
}

/* 工具卡片 */
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
.card-actions { display: flex; align-items: center; gap: 6px; }
.card-body { padding: 16px 20px; }

/* Tab 样式 */
.image-tabs :deep(.el-tabs__header) {
  margin: 0;
  padding-left: 16px;
}
.image-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

/* 上传区域 */
.image-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
}
.info-name { color: var(--text-primary); font-weight: 500; }
.upload-hint {
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
  padding: 20px;
  transition: color 0.2s;
}

/* 拖拽上传 */
.upload-area {
  transition: background 0.2s;
}
.upload-area.drag-over {
  background: rgba(0, 212, 255, 0.05);
}
.upload-area.drag-over .upload-hint {
  color: var(--accent-cyan);
}

/* 压缩控制 */
.compress-controls { margin-bottom: 16px; }
.control-label {
  display: block;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}
.quality-slider {
  width: 100%;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border-color);
  border-radius: 3px;
  outline: none;
}
.quality-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent-cyan);
  cursor: pointer;
}
.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}

/* 缩放控制 */
.resize-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
}
.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.input-row label {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}
.resize-input {
  width: 100px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 6px 8px;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}
.resize-input:focus {
  border-color: var(--accent-cyan);
}
.resize-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.lock-row { width: 100%; }

/* 操作按钮 */
.action-grid { display: flex; flex-wrap: wrap; gap: 16px; align-items: center; }
.action-group { display: flex; align-items: center; gap: 8px; }

/* 结果信息 */
.result-info {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.arrow { color: var(--accent-cyan); }
.ratio { color: var(--accent-cyan); font-weight: 600; }

.base64-info {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
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
</style>
