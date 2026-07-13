<template>
  <div class="tool-container">
    <!-- 操作卡片（sticky 置顶） -->
    <div class="tool-card sticky-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">图标生成</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>将一张图片生成多尺寸 PNG 图标和 .ico 文件</p>
                <p>支持 Favicon / App Icon 等常见尺寸预设</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">图片</div>
            <div class="group-buttons">
              <el-button type="primary" size="small" @click="selectImage">
                <el-icon><FolderOpened /></el-icon>
                选择图片
              </el-button>
              <el-button
                v-if="sourcePath"
                size="small"
                @click="clearImage"
              >
                清除
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">预设</div>
            <div class="group-buttons">
              <el-button
                v-for="preset in PRESETS"
                :key="preset.label"
                size="small"
                :type="currentPreset === preset.label ? 'primary' : 'default'"
                @click="applyPreset(preset)"
              >
                {{ preset.label }}
              </el-button>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">尺寸</div>
            <div class="group-buttons">
              <el-checkbox-group v-model="selectedSizes" size="small">
                <el-checkbox
                  v-for="s in ALL_SIZES"
                  :key="s"
                  :label="s"
                  :value="s"
                  border
                >
                  {{ s }}×{{ s }}
                </el-checkbox>
              </el-checkbox-group>
            </div>
          </div>
          <div class="action-group">
            <div class="group-label">生成</div>
            <div class="group-buttons">
              <el-button
                type="success"
                size="small"
                :disabled="!canGenerate"
                :loading="generating"
                @click="doGenerate"
              >
                生成图标
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 源图预览 -->
    <div v-if="sourcePreview" class="tool-card">
      <div class="card-header">
        <span class="card-title">源图</span>
        <span class="source-path">{{ fileName }}</span>
      </div>
      <div class="card-body">
        <div class="source-preview">
          <img :src="sourcePreview" alt="源图预览" />
        </div>
      </div>
    </div>

    <!-- 图标预览 -->
    <div v-if="result" class="tool-card">
      <div class="card-header">
        <span class="card-title">生成结果</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="downloadIco">
            <el-icon><Download /></el-icon>
            下载 .ico
          </el-button>
          <el-button size="small" @click="downloadAllPng">
            <el-icon><Download /></el-icon>
            下载全部 PNG
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="preview-grid">
          <div
            v-for="preview in result.previews"
            :key="preview.size"
            class="preview-item"
          >
            <div class="preview-img-wrap">
              <img
                :src="'data:image/png;base64,' + preview.base64"
                :alt="preview.size + 'x' + preview.size"
              />
            </div>
            <div class="preview-label">{{ preview.size }}×{{ preview.size }}</div>
            <el-button
              size="small"
              text
              type="primary"
              @click="downloadSinglePng(preview)"
            >
              下载
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, FolderOpened, Download } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { saveFileWithDialog } from '@/utils/fileSaver'

// ============ 常量 ============
const ALL_SIZES = [16, 24, 32, 48, 64, 128, 256]

const PRESETS = [
  { label: 'Favicon', sizes: [16, 32, 48] },
  { label: 'App Icon', sizes: [16, 32, 48, 64, 128, 256] },
  { label: '自定义', sizes: [] },
]

// ============ 状态 ============
const sourcePath = ref('')
const sourcePreview = ref('')
const fileName = ref('')
const selectedSizes = ref<number[]>([16, 32, 48, 64, 128, 256])
const currentPreset = ref('App Icon')
const generating = ref(false)
const error = ref('')

interface IconPreview {
  size: number
  base64: string
}

interface IconResult {
  previews: IconPreview[]
  ico_base64: string
}

const result = ref<IconResult | null>(null)

// ============ 计算属性 ============
const canGenerate = computed(() => {
  return sourcePath.value && selectedSizes.value.length > 0
})

// ============ 方法 ============
const applyPreset = (preset: { label: string; sizes: number[] }) => {
  currentPreset.value = preset.label
  if (preset.sizes.length > 0) {
    selectedSizes.value = [...preset.sizes]
  }
}

const selectImage = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: '图片文件',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'ico', 'svg'],
      }],
    })
    if (!selected) return

    const path = selected as string
    sourcePath.value = path
    fileName.value = path.split(/[/\\]/).pop() || ''

    // 通过后端读取文件并 base64 预览
    try {
      const base64 = await invoke<string>('read_file_base64', { filePath: path })
      sourcePreview.value = 'data:image/png;base64,' + base64
    } catch (e: any) {
      console.warn('预览读取失败:', e)
    }

    error.value = ''
    result.value = null
  } catch (e: any) {
    error.value = `选择图片失败: ${e}`
  }
}

const clearImage = () => {
  sourcePath.value = ''
  sourcePreview.value = ''
  fileName.value = ''
  result.value = null
  error.value = ''
}

const doGenerate = async () => {
  if (!canGenerate.value) return
  generating.value = true
  error.value = ''

  try {
    const res = await invoke<IconResult>('generate_icon', {
      filePath: sourcePath.value,
      sizes: selectedSizes.value,
    })
    result.value = res
    ElMessage.success(`成功生成 ${res.previews.length} 个尺寸的图标`)
  } catch (e: any) {
    error.value = `生成失败: ${e}`
    ElMessage.error(`生成失败: ${e}`)
  } finally {
    generating.value = false
  }
}

const downloadSinglePng = async (preview: IconPreview) => {
  const blob = base64ToBlob(preview.base64, 'image/png')
  await saveFileWithDialog(blob, `icon-${preview.size}x${preview.size}.png`, 'png')
}

const downloadIco = async () => {
  if (!result.value) return
  const baseName = fileName.value.replace(/\.[^.]+$/, '') || 'icon'
  const blob = base64ToBlob(result.value.ico_base64, 'application/octet-stream')
  await saveFileWithDialog(blob, `${baseName}.ico`, 'ico')
}

const downloadAllPng = async () => {
  if (!result.value) return
  const baseName = fileName.value.replace(/\.[^.]+$/, '') || 'icon'
  for (const preview of result.value.previews) {
    const blob = base64ToBlob(preview.base64, 'image/png')
    await saveFileWithDialog(blob, `${baseName}-${preview.size}x${preview.size}.png`, 'png')
  }
}

const base64ToBlob = (base64: string, mimeType: string): Blob => {
  const byteChars = atob(base64)
  const byteNumbers = new Uint8Array(byteChars.length)
  for (let i = 0; i < byteChars.length; i++) {
    byteNumbers[i] = byteChars.charCodeAt(i)
  }
  return new Blob([byteNumbers], { type: mimeType })
}
</script>

<style scoped>
.source-preview {
  max-width: 200px;
  max-height: 200px;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--border-color);
}

.source-preview img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.source-path {
  font-size: 12px;
  color: var(--text-muted);
  margin-left: auto;
}

.preview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-input);
  min-width: 100px;
}

.preview-img-wrap {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-conic-gradient(#334155 0% 25%, #1e293b 0% 50%) 50% / 16px 16px;
  border-radius: 4px;
  overflow: hidden;
}

.preview-img-wrap img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.preview-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

:deep(.el-checkbox-group) {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

:deep(.el-checkbox.is-bordered) {
  margin-right: 0;
  padding: 4px 10px;
}
</style>