# SQL 工具设计文档

**日期**: 2026-06-18  
**主题**: SQL IN 查询条件转换工具

## 概述

新增独立的 SQL 工具页面，提供将换行分隔的字符串列表转换为 SQL IN 查询条件的功能。

## 架构

### 新增文件
- `src/views/SqlTool.vue` - SQL 工具页面组件
- `src/utils/sqlUtils.ts` - SQL 转换工具函数

### 修改文件
- `src/store/index.ts` - 在 TOOL_LIST 中添加 SQL 工具项
- `src/App.vue` - 添加 SqlTool 组件的路由渲染

## 功能设计

### sqlUtils.ts

```typescript
// 将换行分隔的列表转为 SQL IN 条件
export function convertToSqlIn(
  text: string, 
  quoteType: 'single' | 'double' | 'none' = 'single'
): string
```

**输入**: 换行分隔的字符串（如 `"111\n222\n333"`）  
**输出**: SQL IN 格式字符串

**引号类型**:
- `single`: `('111','222','333')` - 单引号（MySQL/PostgreSQL）
- `double`: `("111","222","333")` - 双引号
- `none`: `(111,222,333)` - 无引号（数字类型）

**处理逻辑**:
1. 按换行符分割
2. 过滤空行
3. 去除每行首尾空白
4. 根据引号类型包裹
5. 用逗号连接并添加括号

### SqlTool.vue

遵循 AGENTS.md 中的科技风卡片布局模板：

1. **操作卡片**: 引号类型选择器（el-select），转换按钮
2. **输入卡片**: 多行文本输入，带清空/粘贴按钮
3. **输出卡片**: 只读结果展示，带复制按钮

## 数据流

```
用户输入文本 → 选择引号类型 → 点击转换 → sqlUtils.convertToSqlIn() → 显示结果 → 可选复制
```

## 错误处理

- 空输入时提示"请输入内容"
- 自动过滤空行，不报错
