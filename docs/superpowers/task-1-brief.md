# Task 1 Brief: 新增批量OCR工具函数

## 上下文

这是批量OCR功能（Phase 1）的第一个Task。需要在现有OCR工具文件中添加批量处理函数。

## 需求

在 `src/utils/ocrUtils.ts` 末尾添加：

1. **BatchImage 接口** - 批量图片数据结构
2. **batchRecognize() 函数** - 并行批量识别
3. **getMergedResult() 函数** - 合并结果

## 完整代码

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

## 注意事项

- 不要修改现有代码
- 只在文件末尾添加新代码
- `recognizeImage` 已在文件中定义，直接复用
- 完成后运行 `npx tsc --noEmit` 验证类型检查通过

## 报告文件

完成后将报告写入：`d:\work\trae_use\desktop-tools\docs\superpowers\task-1-report.md`

报告格式：
- 状态：DONE / DONE_WITH_CONCERNS / BLOCKED
- 提交哈希：`git rev-parse --short HEAD`
- 测试结果：类型检查通过/失败
- 关注点：（如有）
