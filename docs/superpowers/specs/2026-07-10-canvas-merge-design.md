# 自由画布拼图设计

## 概述

将现有的"图片拼接"Tab 升级为"自由画布拼图"，从排列式拼接（纵向/横向+对齐+间距）升级为自由画布交互，支持拖拽定位、缩放、旋转、层级调整，像拼贴画一样自由组合图片。

## 技术方案

- **前端**：引入 Fabric.js 6.x（~300KB），在 Vue 组件中创建交互式画布
- **后端**：新增 Rust 命令 `image_canvas_merge`，使用 `image` crate 渲染高清输出
- **执行方式**：后端使用 `spawn_blocking` 在后台线程执行（遵循项目规范）

## 架构

### 前端（ImageToolEnhanced.vue - merge Tab）

**页面结构**：
1. **图片操作卡片**：选择图片、清空、图片列表（缩略图+文件名+删除按钮）
2. **画布卡片**：Fabric.js 画布区域，核心交互区
3. **输出设置卡片**：画布尺寸模式、背景色、生成/下载按钮

**交互流程**：
1. 点击"选择图片"添加图片 → 读取缩略图 → 创建 Fabric.Image 对象添加到画布（默认居中堆叠，每张偏移 20px）
2. 用户在画布上拖拽移动、拖拽角点缩放、拖拽旋转手柄旋转
3. 列表中可删除图片（同步从画布移除）
4. 画布尺寸默认"自动适应"，可切换为手动输入固定宽高
5. 点击"生成拼图" → 收集所有图片状态 → 调用 Rust 后端渲染 → 展示预览
6. 点击"下载结果"保存 PNG

**Fabric.js 交互能力**：
- 单击选中图片，显示控制手柄（缩放、旋转）
- 拖拽移动位置
- 拖拽角点缩放
- 拖拽顶部旋转手柄旋转
- Ctrl+A 全选 / Delete 删除选中
- 右键菜单（自定义 contextmenu 事件弹出）：置顶（bringToFront）/ 置底（sendToBack）

**状态同步**：
- Fabric.js 的 `object:modified` 事件触发时，更新 `canvasImages` 数组中对应图片的位置/缩放/旋转参数

### 后端（image_tools.rs）

**新增命令**：

```rust
#[tauri::command]
pub async fn image_canvas_merge(
    images: Vec<CanvasImageInput>,
    canvas_width: Option<u32>,
    canvas_height: Option<u32>,
    bg_color: String,
) -> Result<MergeResult, String>
```

**数据结构**：

```rust
struct CanvasImageInput {
    file_path: String,
    left: f64,
    top: f64,
    scale_x: f64,
    scale_y: f64,
    angle: f64,  // 弧度（前端传度，前端负责转弧度后传入）
}
```

**渲染逻辑**：
1. 若 `canvas_width/height` 为 None，遍历所有图片计算旋转后包围盒，确定画布尺寸
2. 创建指定尺寸 RGBA 画布，填充背景色
3. 按添加顺序（后添加的在上层）依次处理每张图片
4. 对每张图片：读取 → 缩放到 `scale_x/scale_y` → 旋转 `angle` → 叠加到 `(left, top)` 位置
5. 输出 PNG base64

### 前端数据结构

```typescript
interface CanvasImage {
  id: string          // 唯一标识
  path: string        // 文件路径
  name: string        // 文件名
  size: number        // 文件大小
  thumb?: string      // 缩略图 base64
  left: number        // X 坐标
  top: number         // Y 坐标
  scaleX: number      // 水平缩放
  scaleY: number      // 垂直缩放
  angle: number       // 旋转角度（度）
}
```

## 画布尺寸

- **自动适应**（默认）：根据所有图片的位置和变换计算边界框，画布刚好包裹所有内容
- **手动指定**：用户输入固定宽高（像素），超出部分在生成时被裁剪

## 变更范围

### 保留
- 背景色设置
- 下载结果（PNG）
- 图片选择/缩略图/文件列表

### 移除
- 方向选择（纵向/横向）
- 对齐方式（居左/居中/居右）
- 间距设置
- ↑↓ 排序按钮

### 新增
- Fabric.js 画布区域
- 画布尺寸模式选择（自动/手动）
- 后端 `image_canvas_merge` 命令
- 右键菜单（置顶/置底）

## 错误处理

| 场景 | 处理方式 |
|------|---------|
| 图片文件不存在/无法读取 | 添加时失败，显示错误提示 |
| 生成时某图片丢失 | 跳过该图片，完成后提示 |
| 画布尺寸过大（>8000px） | 弹出确认对话框 |
| 后端渲染失败 | 捕获错误，显示在页面错误区域 |
| 空画布 | 禁用"生成拼图"按钮 |

## 边界情况

- 旋转后的边界计算：自动适应模式需计算旋转后的包围盒
- 图片超出画布：编辑时允许超出，生成时超出部分裁剪
- 缩放限制：最小 0.1x，最大 10x

## 依赖

```json
"fabric": "^6.0.0"
```

Fabric.js 6.x 支持 ES modules，与 Vite 兼容良好。
