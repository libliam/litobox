# 批量OCR 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 扩展OcrTool支持批量图片识别（最多20张），并行处理，结果以卡片列表展示，支持合并/独立模式切换。

**Architecture:** 在现有单图OCR基础上，新增批量图片管理状态（BatchImage数组），通过Promise.all并行调用recognizeImage，结果按图片分组展示为独立卡片，顶部提供合并视图切换。

**Tech Stack:** Vue 3 Composition API, TypeScript, Element Plus, @paddleocr/paddleocr-js

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/utils/ocrUtils.ts` | 修改 | 新增 `batchRecognize()` 和 `getMergedResult()` |
| `src/views/OcrTool.vue` | 修改 | 添加批量UI、缩略图网格、多结果卡片、合并/独立切换 |

---

### Task 1: 新增批量OCR工具函数

**Files:**
- Modify: `src/utils/ocrUtils.ts`

- [ ] **Step 1: 添加 BatchImage 接口和 batchRecognize 函数**

在 `src/utils/ocrUtils.ts` 末尾添加：

```typescript
export interface BatchImage {
  id: string
  file: File | Blob
  thumbnail: string
  name: string
  status: 'pending' | 'recognizing' | 'success' | 'error'
  result?: string
  error?: string
}

/**
 * 批量OCR识别（并行处理）
 */
export async function batchRecognize(
  images: BatchImage[],
  onProgress?: (completed: number, total: number) => void
): Promise<void> {
  const promises = images.map(async (image) => {
    image.status = 'recognizing'
    try {
      const text = await recognizeImage(image.file)
      image.result = text
      image.status = 'success'
    } catch (e: any) {
      image.error = e.message || '识别失败'
      image.status = 'error'
    } finally {
      const completed = images.filter(i =>
        i.status === 'success' || i.status === 'error'
      ).length
      onProgress?.(completed, images.length)
    }
  })

  await Promise.all(promises)
}

/**
 * 获取合并的OCR结果
 */
export function getMergedResult(images: BatchImage[]): string {
  return images
    .filter(i => i.status === 'success' && i.result)
    .map(i => `--- ${i.name} ---\n${i.result}`)
    .join('\n\n')
}
```

- [ ] **Step 2: 验证代码无语法错误**

运行 `npx tsc --noEmit` 确认类型检查通过。

---

### Task 2: 修改OcrTool.vue - 批量图片管理状态

**Files:**
- Modify: `src/views/OcrTool.vue`

- [ ] **Step 1: 添加批量图片状态**

在 `<script setup>` 中，找到现有状态定义区域，添加：

```typescript
import { recognizeImage, cleanText, exportAsTxt, destroyOcr, batchRecognize, getMergedResult, type BatchImage } from '@/utils/ocrUtils'

// 批量图片状态
const batchImages = ref<BatchImage[]>([])
const isBatchMode = computed(() => batchImages.value.length > 1)
const showMergedView = ref(true)
const mergedResult = computed(() => getMergedResult(batchImages.value))
const completedCount = computed(() =>
  batchImages.value.filter(i => i.status === 'success' || i.status === 'error').length
)
const isAllRecognizing = computed(() =>
  batchImages.value.some(i => i.status === 'recognizing')
)
```

- [ ] **Step 2: 修改文件选择处理逻辑**

替换现有的 `handleFileSelect` 函数：

```typescript
// 处理文件选择（支持批量）
const handleFileSelect = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files || files.length === 0) return

  // 限制最多20张
  const maxFiles = 20
  const selectedFiles = Array.from(files).slice(0, maxFiles)
  if (files.length > maxFiles) {
    ElMessage.warning(`最多支持${maxFiles}张图片，已选择前${maxFiles}张`)
  }

  // 清空之前的批量状态
  batchImages.value = []
  imagePreview.value = ''
  resultText.value = ''
  error.value = ''

  // 处理每张图片
  for (const file of selectedFiles) {
    const thumbnail = await generateThumbnail(file)
    batchImages.value.push({
      id: Date.now().toString() + Math.random().toString(36).substring(2, 9),
      file,
      thumbnail,
      name: file.name || `image-${batchImages.value.length + 1}.png`,
      status: 'pending'
    })
  }

  // 如果是单张，直接开始识别
  if (selectedFiles.length === 1) {
    await processImage(selectedFiles[0])
  }

  input.value = ''
}
```

- [ ] **Step 3: 添加批量识别函数**

```typescript
// 批量识别
const handleBatchRecognize = async () => {
  if (batchImages.value.length === 0) return

  error.value = ''

  // 确保模型已加载
  if (!isModelReady.value) {
    isModelLoading.value = true
    try {
      await recognizeImage(batchImages.value[0].file)
      isModelReady.value = true
    } catch (e: any) {
      error.value = e.message || '模型加载失败'
      isModelLoading.value = false
      return
    }
    isModelLoading.value = false
  }

  // 并行识别所有图片
  try {
    await batchRecognize(batchImages.value, (completed, total) => {
      // 进度更新，Vue会自动响应
    })

    // 更新全局历史（只记录一次）
    const successImages = batchImages.value.filter(i => i.status === 'success')
    if (successImages.length > 0) {
      store.addHistory({
        tool: 'ocr',
        action: `批量识别(${successImages.length}张)`,
        inputPreview: `[${successImages.length}张图片]`,
        outputPreview: successImages[0].result?.substring(0, 100) || ''
      })
    }

    ElMessage.success(`批量识别完成，成功${successImages.length}张`)
  } catch (e: any) {
    error.value = e.message || '批量识别失败'
  }
}
```

- [ ] **Step 4: 添加批量管理辅助函数**

```typescript
// 从批量列表中移除图片
const removeBatchImage = (id: string) => {
  batchImages.value = batchImages.value.filter(img => img.id !== id)
  if (batchImages.value.length === 0) {
    imagePreview.value = ''
    resultText.value = ''
  }
}

// 清空批量列表
const clearBatchImages = () => {
  batchImages.value = []
  imagePreview.value = ''
  resultText.value = ''
  error.value = ''
}

// 加载单张批量图片的结果到主视图
const loadBatchImageResult = (image: BatchImage) => {
  if (image.result) {
    resultText.value = image.result
    imagePreview.value = image.thumbnail
  }
}
```

---

### Task 3: 修改OcrTool.vue - 批量UI模板

**Files:**
- Modify: `src/views/OcrTool.vue`

- [ ] **Step 1: 修改操作卡片按钮**

找到操作卡片中的按钮区域，修改为：

```vue
<div class="action-grid">
  <div class="action-group">
    <span class="group-label">图片输入</span>
    <div class="group-buttons">
      <el-button size="small" type="primary" @click="triggerFileInput">
        批量上传
      </el-button>
      <el-button size="small" @click="handlePaste">
        粘贴剪贴板
      </el-button>
    </div>
  </div>
  <div v-if="batchImages.length > 0" class="action-group">
    <span class="group-label">批量操作</span>
    <div class="group-buttons">
      <el-button
        size="small"
        type="success"
        :disabled="isAllRecognizing || batchImages.length === 0"
        @click="handleBatchRecognize"
      >
        识别全部 ({{ batchImages.length }}张)
      </el-button>
      <el-button size="small" @click="clearBatchImages">
        清空列表
      </el-button>
    </div>
  </div>
</div>
```

- [ ] **Step 2: 替换图片预览区为缩略图网格**

找到 `<div v-if="imagePreview" class="tool-card">` 整个图片预览卡片，替换为：

```vue
<!-- 批量图片列表 -->
<div v-if="batchImages.length > 0" class="tool-card">
  <div class="card-header">
    <span class="card-title">图片列表 ({{ batchImages.length }}/{{ 20 }})</span>
    <div class="card-actions">
      <el-tag v-if="isAllRecognizing" size="small" type="warning">
        识别中 {{ completedCount }}/{{ batchImages.length }}
      </el-tag>
      <el-button size="small" @click="clearBatchImages">清空</el-button>
    </div>
  </div>
  <div class="card-body">
    <div class="thumbnail-grid">
      <div
        v-for="img in batchImages"
        :key="img.id"
        class="thumbnail-item"
        @click="loadBatchImageResult(img)"
      >
        <img :src="img.thumbnail" class="thumbnail-img" />
        <div class="thumbnail-name">{{ img.name }}</div>
        <div class="thumbnail-status">
          <el-tag v-if="img.status === 'success'" size="small" type="success">成功</el-tag>
          <el-tag v-else-if="img.status === 'recognizing'" size="small" type="warning">识别中</el-tag>
          <el-tag v-else-if="img.status === 'error'" size="small" type="danger">失败</el-tag>
          <el-tag v-else size="small" type="info">待识别</el-tag>
        </div>
        <div class="thumbnail-delete" @click.stop="removeBatchImage(img.id)">
          <el-icon><Close /></el-icon>
        </div>
      </div>
    </div>
  </div>
</div>
```

在script中添加Close图标导入：
```typescript
import { QuestionFilled, Close } from '@element-plus/icons-vue'
```

- [ ] **Step 3: 添加合并结果卡片**

在结果展示卡片之前（独立结果卡片之前），添加：

```vue
<!-- 合并结果卡片 -->
<div v-if="isBatchMode && showMergedView && mergedResult" class="tool-card">
  <div class="card-header">
    <div class="header-left">
      <span class="card-title">合并结果</span>
    </div>
    <div class="card-actions">
      <el-button size="small" @click="handleCopyMerged">复制全部</el-button>
      <el-button size="small" @click="handleExportMerged">导出合并txt</el-button>
      <el-button size="small" @click="showMergedView = false">切换单图</el-button>
    </div>
  </div>
  <div class="card-body">
    <el-input
      v-model="mergedResult"
      type="textarea"
      :rows="12"
      readonly
      class="result-textarea"
    />
  </div>
</div>
```

添加合并结果操作函数：

```typescript
// 复制合并结果
const handleCopyMerged = async () => {
  try {
    await navigator.clipboard.writeText(mergedResult.value)
    ElMessage.success('已复制全部结果')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 导出合并结果
const handleExportMerged = async () => {
  await exportAsTxt(mergedResult.value, 'ocr-batch-result.txt')
}
```

- [ ] **Step 4: 修改现有结果卡片，添加模式切换**

找到现有结果卡片的标题栏，修改为：

```vue
<div class="card-header">
  <div class="header-left">
    <span class="card-title">识别结果</span>
    <el-button
      v-if="isBatchMode && !showMergedView"
      size="small"
      @click="showMergedView = true"
    >
      切换合并
    </el-button>
  </div>
  <div class="card-actions">
    <el-button size="small" :disabled="!resultText" @click="handleCopy">复制</el-button>
    <el-button size="small" :disabled="!resultText" @click="handleCleanText">清理空行</el-button>
    <el-button size="small" :disabled="!resultText" @click="handleExport">导出txt</el-button>
  </div>
</div>
```

---

### Task 4: 添加批量相关样式

**Files:**
- Modify: `src/views/OcrTool.vue`

- [ ] **Step 1: 在 `<style scoped>` 末尾添加批量相关样式**

```css
/* 缩略图网格 */
.thumbnail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 12px;
}

.thumbnail-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.2s;
}
.thumbnail-item:hover {
  border-color: var(--accent-cyan);
}

.thumbnail-img {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
}

.thumbnail-name {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 6px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.thumbnail-status {
  margin-top: 4px;
}

.thumbnail-delete {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(239, 68, 68, 0.8);
  color: white;
  border-radius: 50%;
  font-size: 12px;
  opacity: 0;
  transition: opacity 0.2s;
  cursor: pointer;
}
.thumbnail-item:hover .thumbnail-delete {
  opacity: 1;
}
```

---

### Task 5: 验证和测试

**Files:**
- No file changes

- [ ] **Step 1: 类型检查**

运行：`npx tsc --noEmit`
预期：无错误

- [ ] **Step 2: 启动开发服务器**

运行：`npm run dev`
预期：无编译错误

- [ ] **Step 3: 功能测试清单**

| 测试场景 | 预期结果 |
|----------|----------|
| 选择1张图片 | 直接识别，UI与原来一致 |
| 选择2-20张图片 | 显示缩略图网格，点击"识别全部"并行处理 |
| 选择超过20张图片 | 提示"最多支持20张"，只取前20张 |
| 识别过程中 | 显示进度标签，按钮禁用 |
| 部分图片识别失败 | 失败卡片显示红色错误，其他正常 |
| 合并视图 | 显示所有成功结果，用分隔符分隔 |
| 单图视图 | 点击缩略图切换对应结果 |
| 复制/导出合并结果 | 功能正常 |
| 删除缩略图 | 从列表中移除，不影响其他图片 |
| 清空列表 | 重置所有状态 |

---

## 自审检查

- [x] 所有需求都有对应Task实现
- [x] 无TBD/TODO占位符
- [x] 类型定义在各Task中保持一致（BatchImage接口）
- [x] 每个Task可独立编译验证
- [x] 遵循现有代码风格（Composition API + script setup）
- [x] 单图模式完全兼容
- [x] 错误处理覆盖主要场景
