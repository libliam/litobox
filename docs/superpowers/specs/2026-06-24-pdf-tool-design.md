# PDF 工具集 - 设计文档

**日期**: 2026-06-24
**版本**: v2.12.0 (计划)
**状态**: 已批准

## 概述

新增 PDF 工具集页面，支持 PDF 转图片、图片转 PDF、PDF 文本提取、PDF 合并/拆分 四个功能。全程纯前端处理，无需 Tauri 后端调用，纯本地离线运行。

## 技术选型

| 依赖 | 用途 | 版本 |
|------|------|------|
| `pdf-lib` | PDF 创建/合并/拆分/修改 | ^1.17.1 |
| `pdfjs-dist` | PDF 渲染为 Canvas、文本提取 | ^4.0.0 |

**打包体积影响**: 约 +5MB

## 页面结构

```
PdfTool.vue
├── Tab 栏 (sticky) - 4 个 Tab
├── 文件输入卡片 - 拖拽/上传区域
├── Tab 内容卡片 - 各功能专属操作区
│   ├── PDF转图片: DPI选择 + 逐页预览 + 批量下载
│   ├── 图片转PDF: 图片排序 + 页面方向 + 生成下载
│   ├── PDF文本提取: 提取结果展示 + 复制
│   └── PDF合并/拆分: 多文件列表 + 页码范围输入 + 生成下载
└── 错误提示区
```

## 功能详细设计

### Tab 1: PDF 转图片

**输入**: 单个 PDF 文件
**输出**: 每页导出为 PNG 图片

**交互**:
- 上传 PDF 后显示总页数
- DPI 选择器: 72 (快速) / 150 (标准) / 300 (高清)
- 点击"开始转换"后逐页渲染，显示进度
- 每页预览缩略图，支持单页下载
- 支持"全部下载"打包为 ZIP（或逐个下载）

**工具函数** (`src/utils/pdfUtils.ts`):
- `pdfToImages(pdfFile: File, dpi: number): Promise<Blob[]>` - 使用 pdfjs-dist 渲染每页到 Canvas，导出为 PNG Blob

### Tab 2: 图片转 PDF

**输入**: 多张图片文件
**输出**: 单个 PDF 文件

**交互**:
- 支持多选图片上传，显示图片列表（缩略图 + 文件名）
- 支持拖拽排序图片顺序
- 页面尺寸选择: A4 / A3 / 自定义 / 跟随图片
- 页面方向: 纵向 / 横向 / 自动
- 点击"生成 PDF"后下载

**工具函数**:
- `imagesToPdf(imageFiles: File[], options: PdfOptions): Promise<Blob>` - 使用 pdf-lib 创建 PDF，逐页插入图片

### Tab 3: PDF 文本提取

**输入**: 单个 PDF 文件
**输出**: 纯文本内容

**交互**:
- 上传 PDF 后显示"提取文本"按钮
- 提取后在输出区显示文本，显示字符数
- 支持一键复制
- 支持导出为 .txt 文件

**工具函数**:
- `extractPdfText(pdfFile: File): Promise<string>` - 使用 pdfjs-dist 提取所有页面文本

### Tab 4: PDF 合并/拆分

**输入**: 多个 PDF 文件 + 页码范围
**输出**: 合并/拆分后的 PDF 文件

**交互**:
- 支持多选 PDF 文件上传，显示文件列表
- 每个文件显示页数信息
- 页码范围输入框，语法: `1-3,5,8-10`
  - 支持 `all` 表示全部页
  - 支持 `odd` 表示奇数页，`even` 表示偶数页
- 快捷操作: "合并所有"、"提取奇数页"、"提取偶数页"
- 点击"生成 PDF"后下载

**工具函数**:
- `mergePdf(pdfFiles: File[], pageRanges: string): Promise<Blob>` - 使用 pdf-lib 合并多个 PDF 的指定页
- `parsePageRange(range: string, totalPages: number): number[]` - 解析页码范围语法

## 数据流

```
用户上传文件 → 前端解析 (pdfjs/pdf-lib) → 处理 → 生成 Blob → 下载/复制
```

全程纯前端，无网络请求。

## 文件清单

| 文件 | 用途 |
|------|------|
| `src/views/PdfTool.vue` | 主页面组件 |
| `src/utils/pdfUtils.ts` | PDF 处理工具函数 |
| `src/store/index.ts` | 注册新工具 (TOOL_LIST + 路由) |
| `src/App.vue` | 添加组件导入和路由渲染 |
| `package.json` | 新增 pdf-lib + pdfjs-dist 依赖 |

## 约束

- 纯本地离线运行，无网络请求
- 单文件最大 100MB（PDF 渲染限制）
- 图片转 PDF 最多支持 50 张图片
- 遵循项目 UI 规范（科技风卡片布局、CSS 变量）

## 版本升级

版本号从 `2.11.0` 升级到 `2.12.0`
