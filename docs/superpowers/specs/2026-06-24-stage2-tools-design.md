# 第二阶段工具设计文档

**日期**: 2026-06-24
**状态**: 已批准

## 概述

实现3个第二阶段工具：CSS工具、JWT解析、字数统计。采用方案A（独立工具页面），每个工具创建独立Vue页面，在侧边栏独立展示。

## 架构

### 新增文件

| 文件 | 说明 |
|------|------|
| `src/views/CssTool.vue` | CSS工具页面 |
| `src/views/JwtTool.vue` | JWT解析页面 |
| `src/views/WordCountTool.vue` | 字数统计页面 |

### 修改文件

| 文件 | 修改内容 |
|------|---------|
| `src/store/index.ts` | TOOL_LIST添加3个新工具项，添加快捷键配置 |
| `src/App.vue` | 导入3个组件，添加路由条件 |
| `src/components/SidebarNav.vue` | 版本号 v2.0 → v2.2 |
| `package.json` | 版本号 2.1.0 → 2.2.0 |

## 工具详情

### 1. CSS工具 (CssTool.vue)

**功能模块**:
- 颜色格式转换：Hex ↔ RGB ↔ HSL 互相转换
- 单位换算：px ↔ rem ↔ em ↔ vw/vh
- CSS 压缩/格式化

**页面结构**:
- 操作卡片：功能切换（颜色转换/单位换算/CSS压缩）
- 输入卡片：CSS代码或颜色值输入
- 输出卡片：转换结果展示

**技术实现**:
- 纯前端颜色计算
- 正则表达式解析CSS属性
- 无需新依赖

### 2. JWT解析 (JwtTool.vue)

**功能模块**:
- 解析 JWT token 三段结构
- 展示 Header（算法、类型）
- 展示 Payload（用户信息、过期时间）
- 高亮过期字段

**页面结构**:
- 操作卡片：解析按钮
- 输入卡片：JWT token 输入
- 输出卡片：Header/Payload 分段展示（JSON格式化）

**技术实现**:
- Base64解码（复用已有encodeUtils）
- JSON解析
- 时间戳格式化

### 3. 字数统计 (WordCountTool.vue)

**功能模块**:
- 字符数（含/不含空格）
- 单词数（英文）/ 字数（中文）
- 行数、段落数
- 阅读时间估算（中文/英文不同速度）

**页面结构**:
- 操作卡片：统计按钮
- 输入卡片：文本输入
- 输出卡片：统计数据展示

**技术实现**:
- 正则表达式分词
- 简单统计算法

## 集成细节

### TOOL_LIST 新增项

```typescript
{ id: 'css', name: 'CSS工具', category: 'devtools', ... }
{ id: 'jwt', name: 'JWT解析', category: 'devtools', ... }
{ id: 'wordCount', name: '字数统计', category: 'devtools', ... }
```

### 快捷键配置

```typescript
shortcuts: {
  css: 'CmdOrCtrl+Alt+C',
  jwt: 'CmdOrCtrl+Alt+J',
  wordCount: 'CmdOrCtrl+Alt+W'
}
```

## 版本号

- `package.json`: `2.1.0` → `2.2.0`
- `SidebarNav.vue`: `v2.0` → `v2.2`

## 设计原则

1. 遵循现有工具页面模板（操作/输入/输出卡片结构）
2. 纯本地离线运行，无网络请求
3. 使用科技风UI规范（CSS变量、卡片式布局）
4. 复用已有工具函数（encodeUtils等）
5. 不引入新依赖
