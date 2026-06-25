# 批量OCR 设计文档

**日期:** 2026-06-25
**状态:** 已批准
**关联计划:** [2026-06-25-ocr-extensions.md](../plans/2026-06-25-ocr-extensions.md) Phase 1

---

## 目标

基于现有单图OCR能力，扩展批量图片识别功能，支持一次选择最多20张图片，并行处理，结果以卡片列表形式展示，支持合并/独立查看模式切换。

---

## 需求总结

| 需求 | 决策 |
|------|------|
| 结果展示方式 | 合并/独立可切换 |
| 处理方式 | 并行同时识别 |
| 数量限制 | 最多20张 |
| UI方案 | 方案B：卡片列表式 |

---

## 交互流程

```
用户点击"批量上传" → 选择多张图片（最多20张）
  ↓
显示缩略图网格 + "识别全部"按钮
  ↓
用户点击"识别全部" → 并行处理所有图片
  ↓
显示处理进度（如 3/20 已完成）
  ↓
生成多个结果卡片（每张图一个）+ 顶部合并结果区
```

---

## UI结构

### 1. 操作卡片（置顶）

- 保留原有"粘贴剪贴板"按钮
- "上传文件"按钮改为"批量上传"，支持多选
- 新增"识别全部"按钮（选择图片后出现）
- 显示已选图片数量标签

### 2. 图片预览区

- 改为缩略图网格布局（`display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr))`）
- 每个缩略图卡片包含：
  - 缩略图（80x80px）
  - 文件名（截断显示）
  - 右上角删除按钮（hover时显示）
  - 识别状态图标（pending/recognizing/success/error）

### 3. 结果展示区

**合并模式（默认开启）：**
- 顶部显示合并结果卡片
- 包含所有成功识别的图片结果，用分隔符分隔
- 提供"关闭合并视图"按钮

**独立模式：**
- 垂直排列多个结果卡片
- 每个卡片包含：
  - 标题栏：缩略图 + 文件名 + 状态标签
  - 操作按钮：复制/清理/导出
  - 文本框：识别结果（只读）
  - 错误提示（如有）

---

## 数据结构

```typescript
interface BatchImage {
  id: string                    // 唯一标识（UUID或时间戳）
  file: File | Blob             // 原始文件
  thumbnail: string             // 缩略图 dataURL
  name: string                  // 文件名
  status: 'pending' | 'recognizing' | 'success' | 'error'
  result?: string               // OCR结果文本
  error?: string                // 错误信息
}
```

---

## 核心逻辑

### batchRecognize 函数

```typescript
// src/utils/ocrUtils.ts
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
      // 更新进度
      const completed = images.filter(i => 
        i.status === 'success' || i.status === 'error'
      ).length
      onProgress?.(completed, images.length)
    }
  })
  
  await Promise.all(promises)
}
```

### 合并结果生成

```typescript
function getMergedResult(images: BatchImage[]): string {
  return images
    .filter(i => i.status === 'success' && i.result)
    .map(i => `--- ${i.name} ---\n${i.result}`)
    .join('\n\n')
}
```

---

## 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `src/views/OcrTool.vue` | 添加批量UI、缩略图网格、多结果卡片、合并/独立切换 |
| `src/utils/ocrUtils.ts` | 添加 `batchRecognize()` 和 `getMergedResult()` 函数 |

---

## 与现有功能兼容

| 功能 | 兼容策略 |
|------|----------|
| 单图模式 | 选择1张图时UI与现在完全一致 |
| 历史记录 | 批量识别时，每张图单独保存一条历史 |
| 导出功能 | 支持单个导出和合并导出 |
| 粘贴剪贴板 | 保持原有单图粘贴逻辑不变 |

---

## 错误处理

- 单张图片识别失败不影响其他图片
- 失败图片在结果卡片中显示红色错误提示
- 合并结果中自动跳过失败的图片
- 网络/内存错误时显示全局错误提示

---

## 性能考虑

- 并行处理使用 `Promise.all`，不阻塞UI
- 缩略图生成使用Canvas，限制尺寸80x80px
- 最多20张图片，避免过多导致内存问题
- 识别过程中显示loading状态，防止重复点击
