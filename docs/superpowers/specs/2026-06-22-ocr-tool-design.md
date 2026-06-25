# OCR文字识别工具 - 设计文档

## 概述

新增OCR文字识别工具，基于 `@paddleocr/paddleocr-js` + `onnxruntime-web` 实现纯前端离线文字识别，支持本地图片上传和剪贴板粘贴两种输入方式。

## 技术选型

| 组件 | 技术 | 说明 |
|------|------|------|
| OCR引擎 | `@paddleocr/paddleocr-js@0.4.2` | 浏览器端PaddleOCR SDK |
| 推理后端 | `onnxruntime-web` | Wasm推理引擎 |
| 模型 | PP-OCRv6 Tiny | 中英文文字识别，体积小精度高 |

## 架构设计

### 新增文件

| 文件 | 职责 |
|------|------|
| `src/views/OcrTool.vue` | OCR工具页面，卡片式布局 |
| `src/utils/ocrUtils.ts` | OCR引擎初始化、识别、清理等纯函数 |
| `public/assets/ocr/` | 存放PP-OCRv6 Tiny模型文件（det/rec） |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `src/store/index.ts` | 新增OCR历史记录接口和存储逻辑 |
| `src/App.vue` | 新增OCR路由 |
| `package.json` | 新增依赖 `@paddleocr/paddleocr-js`、`onnxruntime-web` |

## 数据流

1. **图片输入**：文件选择器上传 / 剪贴板粘贴（监听 `paste` 事件）
2. **模型懒加载**：首次进入OCR页面时初始化 `PaddleOCR` 实例，后续复用
3. **识别执行**：调用 `ocr.predict(imageBlob)` 返回结构化结果
4. **结果处理**：提取 `items[].text` 拼接为完整文本，保留原始分行
5. **历史记录**：保存最近10条识别记录到 `localStorage`

## UI布局

遵循科技风卡片布局规范（`tool-card` + `card-header` + `card-body`）：

- **操作卡片**：上传文件、粘贴剪贴板、模型状态指示器、清空历史、清除当前结果
- **图片预览区**：支持拖拽上传，显示图片缩略图
- **识别结果区**：文本展示（保留分行），操作按钮（复制文本、清理空行、导出txt、转发至其他工具）
- **识别历史区**：最近10条记录，缩略图+文本预览

## 性能指标

- 模型体积：约 1.5-2MB
- onnxruntime-web：约 300KB
- 首次识别加载：200-500ms
- 单张识别耗时：100-150ms
- 内存占用：30-50MB（Wasm）

## 错误处理

- 模型加载失败：提示用户重新加载
- 图片格式不支持：提示支持 PNG/JPG/WebP/BMP
- 识别超时：10s 超时提示重试
- 空图片：提示图片可能为空或损坏

## 安全约束

- 全程离线，图片与识别数据仅本地处理
- 不上传网络
- 仅用户手动选择图片，禁止自动遍历本地文件
