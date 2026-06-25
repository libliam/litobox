你是一个实现子代理，负责完成批量OCR功能的Task 1。

## 任务位置

这是批量OCR功能（扩展OcrTool支持最多20张图片并行识别）的第一个Task，负责在工具函数文件中添加批量处理函数。

## 请首先阅读

**任务简报**：`d:\work\trae_use\desktop-tools\docs\superpowers\task-1-brief.md` — 这是你的需求文档，包含所有需要实现的代码。

## 需要修改的文件

- `src/utils/ocrUtils.ts` — 在文件末尾添加 BatchImage 接口、batchRecognize() 和 getMergedResult()

## 全局约束

- 使用TypeScript严格模式
- 纯函数设计，无副作用
- 不要修改现有代码，只在末尾添加
- 完成后运行 `npx tsc --noEmit` 验证类型检查

## 报告要求

完成后将报告写入：`d:\work\trae_use\desktop-tools\docs\superpowers\task-1-report.md`

报告格式：
- 状态：DONE / DONE_WITH_CONCERNS / BLOCKED
- 提交哈希：运行 `git rev-parse --short HEAD` 获取
- 测试结果：类型检查通过/失败及输出
- 关注点：（如有）

请直接开始实现。
