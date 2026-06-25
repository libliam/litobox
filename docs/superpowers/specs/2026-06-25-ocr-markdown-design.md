# Phase 3: 图片转Markdown 设计文档

**日期:** 2026-06-25
**状态:** 已批准

---

## 概述

在OCR工具中新增"Markdown转换"Tab，实现图片文字识别后按段落/标题结构输出Markdown格式，支持源码查看、渲染预览和导出.md文件。

---

## 架构设计

### 位置
在 `src/views/OcrTool.vue` 中新增 `el-tab-pane`，与现有的"文字识别"和"表格识别"Tab并列。

### 核心流程
```
用户上传图片 
  → OCR识别（获取原始OcrResult[]，含box坐标和confidence）
  → convertToMarkdown() 处理
  → 输出Markdown文本
  → 左侧显示源码，右侧渲染预览
  → 支持导出.md文件
```

---

## 组件设计

### OcrTool.vue 新增状态变量
- `markdownImagePreview: ref<string>('')` - Markdown模式图片预览
- `markdownMdText: ref<string>('')` - Markdown输出文本
- `markdownError: ref<string>('')` - 错误信息
- `isMarkdownRecognizing: ref<boolean>(false)` - 识别中状态
- `markdownFileInputRef: ref<HTMLInputElement | null>(null)` - 文件输入引用

### UI结构
1. **操作卡片**: 上传文件、粘贴剪贴板按钮
2. **图片预览卡片**: 显示待转换的图片
3. **输出卡片**: 分两栏布局
   - 左侧：Markdown源码（textarea，readonly）
   - 右侧：Markdown渲染预览（HTML渲染）
4. **操作按钮**: 复制Markdown、导出.md文件

### ocrUtils.ts 新增函数

#### `convertToMarkdown(ocrResults: OcrResult[]): string`

**功能**: 将OCR原始结果转换为Markdown格式

**实现逻辑**:
1. 计算所有文字块的平均高度
2. 基于高度阈值判断标题层级：
   - 高度 > 平均高度 × 1.5 → `# 一级标题`
   - 高度 > 平均高度 × 1.2 → `## 二级标题`
   - 高度 > 平均高度 × 1.0 → `### 三级标题`
   - 其他 → 普通段落
3. 按Y坐标排序文字块
4. 按段落间距分组（间距大于阈值则插入空行）
5. 输出Markdown字符串

**参数**:
- `ocrResults: OcrResult[]` - OCR识别结果数组

**返回**:
- `string` - Markdown格式文本

#### `recognizeMarkdown(image: Blob | File): Promise<string>`

**功能**: 识别图片并返回Markdown格式文本

**实现**:
1. 调用 `initOcr()` 获取引擎
2. 调用 `engine.predict(image)` 获取原始OcrResult[]
3. 调用 `convertToMarkdown(results)` 转换
4. 返回Markdown字符串

---

## 数据流

```
┌─────────────┐
│ 用户上传图片 │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ OCR识别引擎  │ → OcrResult[] (text, confidence, box)
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│ convertToMarkdown│ → Markdown字符串
└──────┬───────────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│ 左侧: 源码   │     │ 右侧: 预览   │
└─────────────┘     └─────────────┘
       │
       ▼
┌─────────────┐
│ 导出.md文件  │
└─────────────┘
```

---

## 标题推断逻辑

### 基于box高度推断

```typescript
// 计算平均高度
const avgHeight = results.reduce((sum, r) => {
  const h = Math.max(...r.box.map(p => p[1])) - Math.min(...r.box.map(p => p[1]))
  return sum + h
}, 0) / results.length

// 判断层级
const height = Math.max(...item.box.map(p => p[1])) - Math.min(...item.box.map(p => p[1]))
if (height > avgHeight * 1.5) return '# '
if (height > avgHeight * 1.2) return '## '
if (height > avgHeight * 1.0) return '### '
return ''
```

### 段落分组

按Y坐标间距判断段落分隔：
- 间距 > 平均行高 × 2 → 插入空行（新段落）
- 否则 → 同一段落内换行

---

## 导出功能

复用项目已有的 `saveFileWithDialog` 工具函数：

```typescript
import { saveFileWithDialog } from '@/utils/fileSaver'

const blob = new Blob([markdownMdText.value], { type: 'text/markdown;charset=utf-8' })
await saveFileWithDialog(blob, 'markdown-result.md', 'md')
```

---

## 错误处理

| 场景 | 处理方式 |
|------|----------|
| OCR识别失败 | 显示错误提示到 `markdownError` |
| 未识别到文字 | 提示"未识别到文字，请检查图片是否清晰" |
| Markdown转换异常 | 降级为纯文本输出（每行前加空行分隔） |

---

## 新增/修改文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/views/OcrTool.vue` | 修改 | 新增"Markdown转换"Tab及UI |
| `src/utils/ocrUtils.ts` | 修改 | 新增 `convertToMarkdown()` 和 `recognizeMarkdown()` 函数 |

---

## 包体积影响

- **无新增依赖** - 纯原生实现
- **代码量**: 约100行新增代码（ocrUtils.ts + OcrTool.vue）

---

## 自审检查

- [x] 无TBD/TODO/不完整部分
- [x] 架构与功能描述一致，无矛盾
- [x] 范围聚焦，仅实现图片转Markdown
- [x] 需求明确：基于字体大小推断标题、双栏输出、导出.md文件
