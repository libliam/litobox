# 计算器工具设计文档

## 概述

为 LitoBox 添加一个计算器工具，覆盖基础数学运算、单位换算、日期计算和时间戳转换四类常见计算场景。采用 2 个 Tab 组织：计算器（表达式 + 单位换算）和日期工具（日期计算 + 时间戳转换）。

## 架构

- **前端**：Vue 3 (Composition API) + TypeScript，基于 `_ToolTemplate.vue` 模板创建
- **计算引擎**：`mathjs` 库解析表达式，纯前端计算，无需后端参与
- **存储**：操作历史记录到 SQLite
- **无后端改动**

## 页面结构

### 工具注册

```ts
{
  id: 'calculator',
  name: '计算器',
  category: 'utility',
  iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <rect x="4" y="2" width="16" height="20" rx="2"/>
    <line x1="8" y1="6" x2="16" y2="6"/>
    <line x1="8" y1="10" x2="16" y2="10"/>
    <line x1="8" y1="14" x2="12" y2="14"/>
    <line x1="14" y1="14" x2="16" y2="14"/>
    <line x1="8" y1="18" x2="12" y2="18"/>
    <line x1="14" y1="18" x2="16" y2="18"/>
  </svg>`,
  description: '表达式计算、单位换算、日期计算、时间戳转换',
  keywords: ['计算器', '计算', '单位换算', '日期', '时间戳'],
  component: 'CalculatorTool'
}
```

### 文件结构

- `src/views/CalculatorTool.vue` — 主页面，基于 `_ToolTemplate.vue`
- 无独立 `utils/` 文件（使用 `mathjs` 和原生 JS）
- 新增依赖：`mathjs`

## Tab 1: 计算器

### 表达式计算（卡片1，sticky操作卡片 + 输入/输出卡片）

**操作卡片**（sticky）：
- 标题 "EXPRESSION"
- 提示图标：hover 显示支持的运算列表

**输入卡片**：
- 单行 `<el-input>`，placeholder: `3 * (4 + 5) / 2`
- 按钮：清空、粘贴
- 集成 `VariablePicker` 组件（工作流变量插入）
- 输入后 300ms 防抖自动计算
- Enter 键触发计算

**输出卡片**：
- 只读显示计算表达式和结果
- 复制按钮
- 错误时红色边框 + "表达式语法错误" 提示

**支持的运算**：

| 类别 | 运算符/函数 | 示例 |
|------|------------|------|
| 基础 | `+` `-` `*` `/` `()` `%` | `3 + 4 * 2` |
| 幂与根 | `^` `pow()` `sqrt()` | `2^10`, `sqrt(16)` |
| 三角函数 | `sin()` `cos()` `tan()` | `sin(pi/2)` |
| 对数 | `log()` `log10()` | `log(100)`, `log10(1000)` |
| 常量 | `pi` `e` | `pi * 2` |
| 绝对值 | `abs()` | `abs(-5)` |

**处理逻辑**：
- 使用 `mathjs` 的 `evaluate()` 解析表达式
- 捕获 `mathjs` 异常，显示对应错误信息
- 表达式历史记录到 SQLite

### 单位换算（卡片2）

**操作卡片**：
- 类别下拉选择：长度、重量、温度、面积、体积、速度、时间、数据存储、角度、压力、能量

**换算区**：
- 布局：源值输入 + 源单位下拉 ↔ 目标值(只读) + 目标单位下拉
- 左右两列，中间有个交换按钮（↔）
- 实时换算：输入数值或切换单位即时出结果

**11类单位覆盖**：

| 类别 | 单位 |
|------|------|
| 长度 | mm, cm, m, km, inch, ft, yd, mile |
| 重量 | mg, g, kg, t, oz, lb |
| 温度 | °C, °F, K |
| 面积 | mm², cm², m², km², ha, acre |
| 体积 | mL, L, m³, gal(US), gal(UK), fl oz |
| 速度 | m/s, km/h, mph, knot |
| 时间 | ms, s, min, h, day, week |
| 数据存储 | B, KB, MB, GB, TB, bit |
| 角度 | deg, rad, grad |
| 压力 | Pa, kPa, MPa, bar, atm, psi |
| 能量 | J, kJ, cal, kcal, Wh, kWh, eV |

**处理逻辑**：
- 每个类别定义基准单位（如长度基准为米）
- 换算公式：`targetValue = sourceValue * (sourceRatio / targetRatio)`
- 温度特殊处理：°C, °F, K 之间需要偏移量公式

## Tab 2: 日期工具

### 日期计算（卡片1）

**模式A — 两个日期之差**：
- 起始日期选择器（`el-date-picker` type="date"）
- 结束日期选择器
- 输出：相差天、月（近似）、周、小时、分钟

**模式B — 日期加减时间**：
- 起始日期选择器
- 运算符：加 / 减
- 数值输入 + 单位选择（天、周、月、年）
- 输出：计算后的日期（YYYY-MM-DD）

### 时间戳转换（卡片2）

**正向 — 时间戳 → 可读时间**：
- 时间戳输入框（自动识别秒/毫秒）
- 按钮：填入当前时间戳、清空
- 输出：YYYY-MM-DD HH:mm:ss
- 复制按钮

**反向 — 可读时间 → 时间戳**：
- 日期选择器 + 时间选择器
- 输出：秒级和毫秒级时间戳
- 复制按钮

## 错误处理

| 场景 | 处理方式 |
|------|----------|
| 表达式语法错误 | 保留上次输出，显示错误信息 |
| 除以零 | 提示"除数不能为0" |
| 单位换算非数字输入 | 忽略，不计算结果 |
| 时间戳超出范围 | 提示"时间戳超出有效范围" |
| 日期格式无效 | el-date-picker 自带校验 |

## 历史记录

- 类型：`calculator`
- 记录字段：表达式、计算结果
- 支持双击回填到输入框
- 遵循现有历史记录管理规范（最多保留10条操作历史）

## 工作流集成

在 `WorkflowView.vue` 的 `executeStep()` 中添加 `calculator` 分支：
- 输入 → mathjs 表达式计算 → 输出结果
- 变量池支持插入变量到表达式

## UI/UX 要点

- 遵循 LitoBox 科技风设计规范，使用 `theme.css` CSS 变量
- 卡片式布局（`.tool-card`），包含标题栏和内容区
- Sticky 操作卡片置顶
- 实时计算使用 300ms 防抖
- 错误提示红色边框+发光效果
- 符合无障碍访问要求
