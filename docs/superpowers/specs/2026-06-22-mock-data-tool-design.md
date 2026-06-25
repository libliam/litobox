# 随机假数据工具设计文档

**日期**: 2026-06-22
**作者**: liam
**状态**: 已确认

## 需求概述

新增"随机假数据"工具，可随机生成多种类型的模拟数据，用于开发测试、数据填充等场景。

## 数据类别

| 序号 | 数据类型 | 数量范围 | 额外选项 |
|------|---------|---------|---------|
| 1 | 姓名 | 1-50 | 性别（男/女/随机） |
| 2 | 身份证 | 1-20 | 性别、年龄范围（成年/老年/随机） |
| 3 | 手机号 | 1-50 | 运营商（移动/联通/电信/随机） |
| 4 | 邮箱 | 1-50 | 域名（qq/163/gmail/随机） |
| 5 | IP地址 | 1-50 | 类型（IPv4/IPv6） |
| 6 | 网址 | 1-50 | 协议（http/https/随机） |
| 7 | 国内地址 | 1-20 | 省份（随机/指定） |
| 8 | 银行卡号 | 1-20 | 类型（储蓄卡/信用卡） |
| 9 | 统一社会信用代码 | 1-20 | 无 |
| 10 | 车架号 | 1-20 | 无 |
| 11 | 车牌号 | 1-20 | 省份（随机/指定） |

## 架构设计

### 文件结构

新增文件：
- `src/views/MockDataTool.vue` — 页面组件
- `src/utils/mockDataUtils.ts` — 数据生成工具函数

修改文件：
- `src/store/index.ts` — 添加 `mockData` 到 `TOOL_LIST`
- `src/App.vue` — 添加路由

### 工具函数设计

`mockDataUtils.ts` 包含以下纯函数：

```typescript
// 姓名生成
generateName(options: { count: number; gender?: 'male' | 'female' | 'random' }): string[]

// 身份证生成
generateIdCard(options: { count: number; gender?: 'male' | 'female' | 'random'; ageRange?: 'adult' | 'elder' | 'random' }): string[]

// 手机号生成
generatePhone(options: { count: number; carrier?: 'mobile' | 'unicom' | 'telecom' | 'random' }): string[]

// 邮箱生成
generateEmail(options: { count: number; domain?: 'qq' | '163' | 'gmail' | 'random' }): string[]

// IP地址生成
generateIP(options: { count: number; type?: 'ipv4' | 'ipv6' }): string[]

// 网址生成
generateURL(options: { count: number; protocol?: 'http' | 'https' | 'random' }): string[]

// 国内地址生成
generateAddress(options: { count: number; province?: string }): string[]

// 银行卡号生成
generateBankCard(options: { count: number; type?: 'debit' | 'credit' }): string[]

// 统一社会信用代码生成
generateCreditCode(options: { count: number }): string[]

// 车架号生成
generateVIN(options: { count: number }): string[]

// 车牌号生成
generatePlate(options: { count: number; province?: string }): string[]
```

### 页面布局

卡片网格布局，每行 3 个卡片：

```
┌─────────────────────────────────────────────────┐
│  随机假数据工具                                    │
├─────────────────────────────────────────────────┤
│  [姓名]        [身份证]      [手机号]              │
│  [邮箱]        [IP地址]      [网址]               │
│  [国内地址]    [银行卡号]    [统一社会信用代码]    │
│  [车架号]      [车牌号]                          │
└─────────────────────────────────────────────────┘
```

每个卡片结构：
- 标题栏：数据类型名称 + 生成按钮
- 选项区：数量输入框 + 特定选项
- 结果区：生成的数据列表，每项可单独复制

### 样式规范

- 使用项目现有的科技风 UI 规范
- 卡片网格布局：`display: grid; grid-template-columns: repeat(3, 1fr)`
- 每个卡片遵循 `.tool-card` 样式规范
- 结果项使用 `.data-item` 样式（类似 UUIDTool 的 `.uuid-item`）
- 响应式：小屏幕时自动调整为 2 列或 1 列

## 数据模板

### 姓氏库
常见百家姓，约 100 个常见姓氏。

### 名字库
常用汉字名字，分男女两组。

### 手机号段
- 移动：134-139, 150-152, 157-159, 182-184, 187-188
- 联通：130-132, 155-156, 185-186
- 电信：133, 153, 180-181, 189

### 邮箱域名
qq.com, 163.com, 126.com, gmail.com, outlook.com, sina.com

### 地址库
省份、城市、区县、街道模板。

## 实现要点

1. **身份证校验**：符合 GB 11643-1999 标准，包含校验位计算
2. **银行卡号校验**：符合 Luhn 算法
3. **统一社会信用代码**：符合 GB 32100-2015 标准
4. **车架号（VIN）**：符合 ISO 3779 标准，17 位字符
5. **车牌号**：符合国内车牌格式

## 性能要求

- 单次生成 50 条数据响应时间 < 100ms
- 无网络请求，纯本地生成
- 内存占用 < 5MB
