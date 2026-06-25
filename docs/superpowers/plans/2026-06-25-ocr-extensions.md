# OCR扩展功能 任务计划

**目标:** 基于已有的 `@paddleocr/paddleocr-js` + `onnxruntime-web` OCR能力，扩展更多实用功能，充分利用已投入的包体积。

**技术基础:**
- OCR引擎: `@paddleocr/paddleocr-js@^0.4.2`
- 推理后端: `onnxruntime-web@^1.27.0`
- 模型: PP-OCRv6 Tiny
- 已有功能: 图片文字识别（单图）

---

## Phase 1: 批量图片OCR

**状态:** pending

**功能描述:** 一次选择/拖拽多张图片，批量进行OCR识别，结果合并展示。

**实现思路:**
- 修改 `OcrTool.vue` 支持多文件选择
- 循环调用 `recognizeImage()` 处理每张图片
- 结果按图片分组展示，支持导出合并文本
- 显示处理进度（如 3/10）

**新增/修改文件:**
- `src/views/OcrTool.vue` - 添加批量选择UI
- `src/utils/ocrUtils.ts` - 添加 `batchRecognize()` 函数

**难度:** 低（纯前端逻辑复用）

---

## Phase 2: 表格识别导出CSV

**状态:** pending

**功能描述:** 识别图片中的表格内容，导出为CSV/Excel格式。

**实现思路:**
- 利用OCR返回的 `box` 坐标信息推断表格结构
- 按行/列分组文字块
- 导出为CSV文件（轻量）或Excel文件（需新依赖）

**新增/修改文件:**
- `src/utils/ocrUtils.ts` - 添加 `extractTableFromOcr()` 函数
- `src/views/OcrTool.vue` - 添加"表格模式"Tab

**难度:** 中（需要坐标分析逻辑）

---

## Phase 3: 图片转Markdown

**状态:** pending

**功能描述:** 识别图片文字后，按段落/标题结构输出Markdown格式。

**实现思路:**
- 利用OCR返回的 `confidence` 和 `box` 信息推断标题（大字/高置信度）
- 按段落间距分组文字
- 输出带标题层级、段落分隔的Markdown

**新增/修改文件:**
- `src/utils/ocrUtils.ts` - 添加 `convertToMarkdown()` 函数
- `src/views/OcrTool.vue` - 添加"Markdown模式"Tab或输出格式选择

**难度:** 中（需要启发式排版推断）

---

## Phase 4: 离线词典翻译（推荐）

**状态:** pending

**功能描述:** 基于自建中英词典，实现离线翻译能力，支持OCR结果翻译和独立文本翻译。

**技术方案: 方案D - 自建词典**
- **数据源**: CC-CEDICT（开源中英词典，约12万词条）或自定义词典JSON
- **包体积**: 压缩后约2-3MB（纯JSON数据）
- **依赖**: 无新增依赖，纯原生实现
- **翻译方式**: 分词匹配 + 词典查找 + 结果排序

**实现思路:**
- 下载/整理CC-CEDICT词典数据，转换为JSON格式（`{"中文": "英文", ...}` 双向）
- 实现简单分词逻辑（基于词典最大匹配）
- 支持中英互译、整句翻译（逐词翻译+语法拼接）
- 可与OCR结果联动，识别后一键翻译
- 支持独立使用（输入文本直接翻译）

**新增/修改文件:**
- `src/utils/dictionary/cedict.json` - 词典数据文件（压缩后约2-3MB）
- `src/utils/translateUtils.ts` - 翻译逻辑（分词、匹配、翻译）
- `src/views/OcrTool.vue` - 添加"翻译"Tab或输出格式选择
- 或新建 `src/views/TranslateTool.vue` - 独立翻译工具页面

**难度:** 中（需要词典数据处理和分词逻辑）

**包体积影响:** +2-3MB（仅词典JSON数据，无新增依赖）

---

## Phase 5: 图片文字校对（可选）

**状态:** pending

**功能描述:** 对比两张图片的OCR结果，高亮显示差异。

**实现思路:**
- 复用已有 `diff` 库（项目已安装）
- 两张图片分别OCR后对比文本差异
- 可视化展示增删改

**新增/修改文件:**
- `src/views/OcrTool.vue` - 添加"对比模式"Tab

**难度:** 低（复用现有diff库）

---

## 排除的功能

| 功能 | 排除原因 |
|------|----------|
| PDF转docx | 需要复杂排版推断，投入产出比低 |
| docx转PDF | 需要完整渲染引擎，纯前端不现实 |
| 手写文字识别 | PP-OCRv6 Tiny主要针对印刷体，需验证模型支持 |
| 名片识别 | 场景太窄，不如通用OCR灵活 |

---

## 建议实施顺序

1. **Phase 1 批量OCR** - 最简单，立即可用
2. **Phase 2 表格识别** - 实用性强，办公场景常用
3. **Phase 3 图片转Markdown** - 开发者/写作者常用
4. **Phase 4/5** - 根据用户需求决定是否实施

---

## 包体积控制原则

- 不新增大型依赖（如Office处理库）
- 优先复用项目已有依赖（`diff`, `js-base64`, `lodash`）
- 新功能基于OCR引擎扩展，不增加额外模型文件
- CSV导出用原生实现，不引入新库
