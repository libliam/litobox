# 模板拼图 - 设计文档

**日期**: 2026-07-10
**版本**: V4.9.0
**类型**: 重构

---

## 1. 背景

当前"图片拼接"功能（Fabric.js 自由画布）存在以下问题：
- 大图 base64 加载导致画布拖拽卡顿，几乎无法使用
- Fabric.js 自由画布方案过重，用户实际需求是模板化的拼图排列
- 300KB 的 Fabric.js 依赖对简单的拼图场景来说性价比低

**目标**：将"图片拼接"重构为"模板拼图"，使用预设布局模板 + 原生拖拽交换，实现轻量、流畅的拼图体验。

---

## 2. 方案概述

**核心改动**：移除 Fabric.js，改用 CSS Grid 预设布局模板 + 原生 HTML5 Drag & Drop 交换图片位置。

**技术栈**：
- 前端：CSS Grid + HTML5 Drag & Drop API（零新依赖）
- 后端：简化版 `image_canvas_merge`（去掉旋转/缩放/自由定位，改为槽位坐标渲染）

**对比**：

| 方面 | 旧方案（Fabric.js） | 新方案（模板拼图） |
|------|-------------------|-------------------|
| 依赖 | fabric ~300KB | 无 |
| 交互 | 自由拖拽/缩放/旋转 | 拖拽交换槽位 |
| 性能 | 大图卡顿 | 缩略图预览，流畅 |
| 学习成本 | 需理解画布操作 | 直观，选模板即可 |
| 适用场景 | 自由创作 | 快速拼图 |

---

## 3. 模板系统

### 3.1 模板定义

```typescript
interface Template {
  id: string
  name: string      // 显示名称
  count: number     // 所需图片数
  grid: TemplateSlot[]
}

interface TemplateSlot {
  colStart: number
  colEnd: number
  rowStart: number
  rowEnd: number
}
```

### 3.2 模板列表

所有模板输出尺寸固定为 1200×800 像素。

| ID | 名称 | 图片数 | 布局描述 |
|----|------|--------|---------|
| h2 | 左右 2 列 | 2 | 1×2 均分 |
| v2 | 上下 2 行 | 2 | 2×1 均分 |
| l2 | 左大右小 | 2 | 左 2/3，右 1/3 |
| h3 | 三等分 | 3 | 1×3 均分 |
| t3 | 上 1 下 2 | 3 | 上 1 大图，下 2 小图 |
| l3 | 左 1 右 2 | 3 | 左 1 大图，右 2 小图叠放 |
| g4 | 四宫格 | 4 | 2×2 均分 |
| l4 | 左大右 3 | 4 | 左 2/3 大图，右 3 小图叠放 |
| h5 | 五宫格-横 | 5 | 上 2 下 3 |
| g6 | 六宫格 | 6 | 2×3 均分 |

### 3.3 模板过滤规则

- 图片数 ≥ 模板所需数：显示该模板
- 图片数 > 模板所需数：多余的图片暂不显示，后续可拖入空槽位
- 图片数 < 模板所需数：隐藏该模板

---

## 4. 前端设计

### 4.1 页面结构

```
┌─ sticky-card ─────────────────────────────┐
│  [批量压缩/转换] [图片拼图] [加水印] [调色板]  │
└───────────────────────────────────────────┘

┌─ tool-card: 图片选择 ──────────────────────┐
│  [选择图片] [清空]                           │
│  图片列表（缩略图+文件名+删除）                │
└───────────────────────────────────────────┘

┌─ tool-card: 拼图模板 ──────────────────────┐
│  模板缩略图选择器（CSS Grid 小示意图）          │
│  根据图片数量自动过滤                          │
└───────────────────────────────────────────┘

┌─ tool-card: 拼图预览 ──────────────────────┐
│  CSS Grid 实时预览，槽位可拖拽交换              │
└───────────────────────────────────────────┘

┌─ tool-card: 输出设置 ──────────────────────┐
│  背景色: [■] [透明]  间距: [___px]            │
│  [生成拼图] [下载结果]                        │
└───────────────────────────────────────────┘
```

### 4.2 拖拽交换

- 每个槽位同时是 `draggable` 和 `dropzone`
- HTML5 Drag & Drop API：`dragstart` / `dragover` / `drop`
- 拖拽时源槽位半透明（opacity: 0.5），目标槽位高亮边框（青色虚线）
- 拖拽到另一个槽位 → 交换 `slotMap` 中两个位置的图片索引
- 拖拽到空槽位 → 把图片移过去，源槽位清空

### 4.3 状态管理

```typescript
interface MergeImage {
  path: string
  name: string
  size: number
  thumb: string  // base64 缩略图
}

const images = ref<MergeImage[]>([])         // 已选图片列表
const slotMap = ref<(number | null)[]>([])   // 槽位 → 图片索引，null=空
const currentTemplate = ref<Template | null>(null)
```

### 4.4 操作流程

1. **选择图片** → 推入 `images[]` → 自动选择首个匹配模板 → 按顺序填充 `slotMap`
2. **切换模板** → 保留已有图片映射，多余槽位留空
3. **拖拽交换** → 交换 `slotMap` 中两个槽位的值
4. **删除图片** → 从 `images[]` 移除 → 清理 `slotMap` → 重新检查模板兼容性
5. **生成拼图** → 将 `slotMap` + `images` + 模板信息传给后端

---

## 5. 后端设计

### 5.1 接口

```rust
#[tauri::command]
pub async fn image_template_merge(
    images: Vec<MergeSlotInput>,
    canvas_width: u32,
    canvas_height: u32,
    bg_color: String,
    gap: u32,  // 间距（像素）
) -> Result<MergeResult, String>
```

```rust
struct MergeSlotInput {
    file_path: String,
    x: u32,     // 槽位 X 坐标
    y: u32,     // 槽位 Y 坐标
    width: u32, // 槽位宽度
    height: u32,// 槽位高度
}
```

### 5.2 渲染逻辑

1. 创建指定尺寸的画布（RGBA），填充背景色（支持透明）
2. 对每个槽位：
   - 读取图片
   - 按槽位宽高等比缩放（cover 模式，居中裁剪）
   - 叠加到画布的 `(x, y)` 位置
3. 输出 PNG base64

### 5.3 移除逻辑

- 不再需要旋转计算（`rotate_about_center`）
- 不再需要自由缩放（`scale_x/scale_y`）
- 不再需要自动/手动画布尺寸模式（改为固定模板尺寸）
- 不再需要 `CanvasImageInput` 中的 `left/top/scale_x/scale_y/angle`

---

## 6. 代码变更清单

### 6.1 移除

- `package.json`: 移除 `fabric` 依赖，执行 `npm uninstall fabric`
- `src/views/ImageToolEnhanced.vue`: 移除 Fabric.js 画布、对齐线、右键菜单、CanvasImage 接口等所有自由画布相关代码
- `src-tauri/src/image_tools.rs`: 移除 `CanvasImageInput` 和 `image_canvas_merge`，替换为 `MergeSlotInput` 和 `image_template_merge`
- `src-tauri/src/main.rs`: 将 `image_canvas_merge` 注册替换为 `image_template_merge`

### 6.2 新增

- `src/views/ImageToolEnhanced.vue`: 模板选择器、CSS Grid 预览区、拖拽交换逻辑
- `src-tauri/src/image_tools.rs`: `image_template_merge` 命令

### 6.3 保留

- 图片选择/删除/清空按钮
- 背景色设置（含透明）
- 下载结果
- 预览展示
- 错误提示

---

## 7. 错误处理

| 场景 | 处理 |
|------|------|
| 图片加载失败 | 槽位显示错误图标，提示"图片加载失败" |
| 图片数不足模板要求 | 自动过滤模板，不显示不可用的 |
| 后端渲染失败 | 捕获错误，显示在错误区域 |
| 空槽位生成 | 跳过空槽位，不渲染 |

---

## 8. 测试要点

- [ ] 选择 2-6 张图片，验证模板自动过滤
- [ ] 拖拽交换图片位置
- [ ] 切换模板，验证图片保留
- [ ] 删除图片，验证模板重新过滤
- [ ] 设置背景色/透明 + 间距
- [ ] 生成拼图，验证输出正确
- [ ] 下载结果，验证 PNG 文件
- [ ] 工作流集成（变量池 + 历史记录）