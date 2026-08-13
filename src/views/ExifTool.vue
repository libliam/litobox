<template>
  <div class="tool-container">
    <div class="tool-card sticky-card">
      <div class="card-header">
        <span class="card-title">图片EXIF</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerInput">选择图片</el-button>
          <el-button v-if="file" size="small" @click="handleClear">移除</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="inputRef"
          type="file"
          accept="image/*"
          style="display: none"
          @change="handleSelect"
        />
        <div v-if="file" class="file-info">
          <span class="file-name">{{ file.name }}</span>
          <span class="file-size">{{ formatFileSize(file.size) }}</span>
          <span v-if="exif" class="file-format">{{ exif.format.toUpperCase() }}</span>
        </div>
        <div v-else class="upload-hint" @dragover.prevent @drop="handleDrop">
          点击「选择图片」或拖拽图片到此处
        </div>
      </div>
    </div>

    <div v-if="exif" class="tool-card">
      <div class="card-header">
        <span class="card-title">EXIF 信息</span>
        <span v-if="isReading" class="reading-tip">解析中...</span>
      </div>
      <div class="card-body exif-body">
        <div v-if="previewUrl" class="exif-preview">
          <img :src="previewUrl" alt="预览" />
        </div>
        <div v-if="exif.hasGps" class="gps-bar">
          <span class="gps-label">GPS 定位</span>
          <span class="gps-value">{{ exif.gpsText }}</span>
          <el-button size="small" type="primary" plain @click="copyGps">复制坐标</el-button>
        </div>
        <DataTable v-if="exif.items.length" :data="exif.items" max-height="420">
          <el-table-column prop="label" label="字段" width="180" />
          <el-table-column prop="value" label="值" />
        </DataTable>
        <div v-else class="no-exif">未发现 EXIF 信息（可能是截图或已剥离过元数据）</div>
      </div>
    </div>

    <div v-if="file" class="tool-card">
      <div class="card-header">
        <span class="card-title">清除隐私信息</span>
        <div class="card-actions">
          <el-button
            type="danger"
            size="small"
            :loading="isStripping"
            @click="handleStrip"
          >
            清除并另存
          </el-button>
        </div>
      </div>
      <div class="card-body">
        <div class="strip-info">
          移除 EXIF / GPS / IPTC 等隐私元数据。JPEG 无损剥离不重编码、不损失画质；
          PNG / WebP 通过重绘剥离。
          <span v-if="strippedSize !== null" class="strip-result">
            已生成无 EXIF 版本（{{ formatFileSize(strippedSize) }}，原 {{ formatFileSize(file.size) }}）
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import DataTable from '@/components/DataTable.vue'
import { readExif, stripExif, type ExifResult } from '@/utils/exifUtils'
import { saveFileWithDialog } from '@/utils/fileSaver'

const inputRef = ref<HTMLInputElement>()
const file = ref<File>()
const previewUrl = ref('')
const exif = ref<ExifResult | null>(null)
const isReading = ref(false)
const isStripping = ref(false)
const strippedSize = ref<number | null>(null)

const formatFileSize = (size: number): string => {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(2)} MB`
}

const triggerInput = (): void => inputRef.value?.click()

const loadFile = async (f: File): Promise<void> => {
  file.value = f
  strippedSize.value = null
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
  previewUrl.value = URL.createObjectURL(f)
  isReading.value = true
  try {
    exif.value = await readExif(f)
  } catch (err) {
    exif.value = null
    ElMessage.error(err instanceof Error ? err.message : '读取 EXIF 失败')
  } finally {
    isReading.value = false
  }
}

const handleSelect = async (e: Event): Promise<void> => {
  const input = e.target as HTMLInputElement
  const f = input.files?.[0]
  if (!f) return
  await loadFile(f)
  input.value = ''
}

const handleDrop = async (e: DragEvent): Promise<void> => {
  e.preventDefault()
  const f = e.dataTransfer?.files?.[0]
  if (f) await loadFile(f)
}

const handleClear = (): void => {
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
  previewUrl.value = ''
  file.value = undefined
  exif.value = null
  strippedSize.value = null
}

const handleStrip = async (): Promise<void> => {
  if (!file.value) return
  isStripping.value = true
  try {
    const blob = await stripExif(file.value)
    strippedSize.value = blob.size
    const name = file.value.name.replace(/\.[^.]+$/, '') + '_无EXIF'
    await saveFileWithDialog(blob, name, 'jpg')
  } catch (err) {
    ElMessage.error(err instanceof Error ? err.message : '清除失败')
  } finally {
    isStripping.value = false
  }
}

const copyGps = async (): Promise<void> => {
  if (!exif.value?.gpsText) return
  try {
    await navigator.clipboard.writeText(exif.value.gpsText)
    ElMessage.success('GPS 坐标已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}
</script>

<style scoped>
.exif-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.exif-preview img {
  max-width: 260px;
  max-height: 200px;
  border-radius: 6px;
  border: 1px solid var(--border-color, #dcdfe6);
}

.gps-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(245, 158, 11, 0.08);
  border: 1px solid rgba(245, 158, 11, 0.35);
}

.gps-label {
  font-weight: 600;
  color: #f59e0b;
  white-space: nowrap;
}

.gps-value {
  flex: 1;
  font-family: monospace;
  word-break: break-all;
}

.strip-info {
  color: var(--text-secondary, #909399);
  font-size: 13px;
  line-height: 1.6;
}

.strip-result {
  display: block;
  margin-top: 8px;
  color: #10b981;
}

.no-exif {
  color: var(--text-secondary, #909399);
  padding: 12px 0;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.file-name {
  font-weight: 500;
}

.file-size,
.file-format {
  color: var(--text-secondary, #909399);
}

.reading-tip {
  color: var(--text-secondary, #909399);
  font-size: 12px;
}

.upload-hint {
  padding: 20px;
  text-align: center;
  color: var(--text-secondary, #909399);
  border: 1px dashed var(--border-color, #dcdfe6);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s;
}

.upload-hint:hover {
  border-color: var(--color-primary, #409eff);
}
</style>
