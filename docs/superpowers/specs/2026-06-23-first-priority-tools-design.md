# 第一优先级工具扩展设计文档

**日期**: 2026-06-23
**状态**: 待审查

## 概述

为 LitoBox 添加 4 个第一优先级的高频开发工具：CSV/Excel 工具、哈希计算、XML/YAML 工具、文本去重。所有工具均为纯前端实现，无需新增依赖。

## 工具清单

### 1. CSV/Excel 工具 (CsvTool.vue)

**功能**:
- CSV 文本解析为表格预览
- 支持自定义分隔符（逗号、分号、Tab、竖线）
- 列筛选与排序
- 导出为 JSON 或 SQL INSERT 语句
- 支持文件上传（.csv 文件）

**技术实现**:
- 纯前端 CSV 解析（处理引号转义、换行等边界情况）
- Element Plus Table 组件展示
- 文件读取使用 FileReader API

**页面结构**:
- 操作卡片：分隔符选择、导入文件、导出 JSON、导出 SQL
- 输入卡片：CSV 文本输入区
- 输出卡片：表格预览 + 导出结果

**分类归属**: `fileprocessing` (文件处理)

### 2. 哈希计算工具 (HashTool.vue)

**功能**:
- 文本的 MD5、SHA-1、SHA-256、SHA-512 计算
- 文件哈希计算（拖拽上传）
- HMAC 计算（可选密钥）
- 一键复制哈希值

**技术实现**:
- Web Crypto API (SubtleCrypto) 计算 SHA 系列
- MD5 使用轻量实现（内联函数或小型库，约 2KB）
- 文件哈希使用流式读取避免大文件内存溢出

**页面结构**:
- 操作卡片：算法选择（MD5/SHA-1/SHA-256/SHA-512）、HMAC 密钥输入、计算按钮
- 输入卡片：文本输入或文件拖拽
- 输出卡片：哈希结果展示（支持同时显示多种算法结果）

**分类归属**: `devtools` (开发工具)

### 3. XML/YAML 工具 (XmlYamlTool.vue)

**功能**:
- XML 格式化/压缩/校验
- YAML 格式化/校验
- XML ↔ JSON 互转
- YAML ↔ JSON 互转

**技术实现**:
- XML 处理使用浏览器内置 DOMParser 和 XMLSerializer
- YAML 处理使用已安装的 `js-yaml` 库（如未安装则内联简易解析器）
- JSON 互转使用原生 JSON.parse/stringify

**页面结构**:
- 操作卡片：格式化、压缩、校验、XML→JSON、JSON→XML、YAML→JSON、JSON→YAML
- 输入卡片：文本输入 + 粘贴按钮
- 输出卡片：格式化结果 + 错误提示

**Tab 设计**: 两个 Tab（XML 工具 / YAML 工具），每个 Tab 独立输入值

**分类归属**: `devtools` (开发工具)

### 4. 文本去重工具 (DedupTool.vue)

**功能**:
- 按行去重（保留首次出现 / 保留末次出现）
- 忽略大小写去重
- 忽略首尾空格去重
- 统计去重前后的行数变化
- 显示重复行及其出现次数

**技术实现**:
- 纯前端 Set/Map 数据结构
- 实时统计展示

**页面结构**:
- 操作卡片：去重模式选择（保留首次/末次）、选项（忽略大小写/忽略空格）、执行去重、复制结果
- 输入卡片：文本输入 + 粘贴按钮
- 输出卡片：去重结果 + 统计信息（原始行数、去重后行数、重复行数）

**分类归属**: `fileprocessing` (文件处理)

## 架构设计

### 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/views/CsvTool.vue` | 新增 | CSV/Excel 工具页面 |
| `src/views/HashTool.vue` | 新增 | 哈希计算工具页面 |
| `src/views/XmlYamlTool.vue` | 新增 | XML/YAML 工具页面 |
| `src/views/DedupTool.vue` | 新增 | 文本去重工具页面 |
| `src/utils/csvUtils.ts` | 新增 | CSV 解析工具函数 |
| `src/utils/hashUtils.ts` | 新增 | 哈希计算工具函数 |
| `src/utils/xmlYamlUtils.ts` | 新增 | XML/YAML 处理函数 |
| `src/utils/dedupUtils.ts` | 新增 | 文本去重工具函数 |
| `src/App.vue` | 修改 | 注册新工具路由 |
| `src/store/index.ts` | 修改 | 添加新工具到 TOOL_LIST |
| `src/components/SidebarNav.vue` | 无需修改 | 自动从 TOOL_LIST 读取 |

### 依赖变更

- **不新增依赖**：所有功能使用浏览器内置 API 或内联实现
- 如 `js-yaml` 未安装，YAML 解析使用简易正则实现（覆盖 90% 场景）

### UI 规范

所有新工具页面严格遵循 AGENTS.md 中的科技风 UI 规范：
- 使用 `.tool-card` 卡片式布局
- 使用 CSS 变量（禁止硬编码颜色）
- 操作按钮使用 `.action-grid` + `.action-group` 分组
- 输入/输出区域包含标题栏和操作按钮
- 错误提示使用红色边框 + 发光效果

## 实施顺序

1. **CsvTool** → 2. **HashTool** → 3. **XmlYamlTool** → 4. **DedupTool**

每个工具独立实现，互不依赖。
