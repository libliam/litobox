# 自由画布拼图实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将图片拼接功能从排列式升级为 Fabric.js 自由画布，支持拖拽、缩放、旋转、层级调整

**Architecture:** 前端使用 Fabric.js 6.x 创建交互式画布，用户编辑图片位置/缩放/旋转后，将状态传给 Rust 后端，后端使用 image crate 渲染高清 PNG 输出

**Tech Stack:** Vue 3 + TypeScript + Fabric.js 6.x + Tauri 2.0 + Rust image crate

---

## 文件结构

**修改的文件：**
- `package.json` - 添加 fabric 依赖
- `src/views/ImageToolEnhanced.vue` - 重构 merge Tab 为画布模式
- `src-tauri/src/image_tools.rs` - 新增 `image_canvas_merge` 命令
- `src-tauri/src/main.rs` - 注册新命令

---

## Task 1: 安装 Fabric.js 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 fabric 包**

```bash
npm install fabric@^6.0.0
```

Expected: 安装成功，package.json 自动更新

- [ ] **Step 2: 验证安装**

```bash
npm list fabric
```

Expected: 显示 fabric@6.x.x

- [ ] **Step 3: 提交**

```bash
git add package.json package-lock.json
git commit -m "chore: 添加 fabric 依赖"
```

---

## Task 2: 后端 - 定义数据结构和命令签名

**Files:**
- Modify: `src-tauri/src/image_tools.rs:1-50`

- [ ] **Step 1: 在 image_tools.rs 顶部添加 CanvasImageInput 结构体**

在文件顶部（其他结构体定义附近）添加：

```rust
#[derive(serde::Deserialize)]
pub struct CanvasImageInput {
    pub file_path: String,
    pub left: f64,
    pub top: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub angle: f64,
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/src/image_tools.rs
git commit -m "feat: 定义 CanvasImageInput 数据结构"
```

---

## Task 3: 后端 - 实现 image_canvas_merge 命令（自动适应模式）

**Files:**
- Modify: `src-tauri/src/image_tools.rs`

- [ ] **Step 1: 实现自动适应模式的渲染逻辑**

在 `image_tools.rs` 文件末尾添加新函数：

```rust
#[tauri::command]
pub async fn image_canvas_merge(
    images: Vec<CanvasImageInput>,
    canvas_width: Option<u32>,
    canvas_height: Option<u32>,
    bg_color: String,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // 解析背景色
        let bg = parse_color(&bg_color).map_err(|e| format!("背景色解析失败: {}", e))?;

        // 读取所有图片并计算边界
        let mut loaded_images = Vec::new();
        for img_input in &images {
            let img = image::open(&img_input.file_path)
                .map_err(|e| format!("读取图片失败 {}: {}", img_input.file_path, e))?;
            
            // 应用缩放
            let new_width = (img.width() as f64 * img_input.scale_x) as u32;
            let new_height = (img.height() as f64 * img_input.scale_y) as u32;
            let resized = image::imageops::resize(&img, new_width, new_height, image::imageops::FilterType::Lanczos3);
            
            // 应用旋转（使用 imageproc::geometric_transformations::rotate_about_center）
            // 注意：前端传入的是度，需要转为弧度
            let rotated = if img_input.angle.abs() > 0.001 {
                use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
                let angle_rad = (img_input.angle * std::f32::consts::PI / 180.0) as f32;
                rotate_about_center(&resized, angle_rad, Interpolation::Bilinear)
            } else {
                resized
            };
            
            loaded_images.push((rotated, img_input.left, img_input.top));
        }

        // 计算画布尺寸
        let (final_width, final_height, offset_x, offset_y) = if let (Some(w), Some(h)) = (canvas_width, canvas_height) {
            (w, h, 0.0_f64, 0.0_f64)
        } else {
            // 自动适应：计算所有图片的边界框（考虑旋转后的尺寸）
            let mut min_left = f64::MAX;
            let mut min_top = f64::MAX;
            let mut max_right = f64::MIN;
            let mut max_bottom = f64::MIN;
            for (img, left, top) in &loaded_images {
                let right = left + img.width() as f64;
                let bottom = top + img.height() as f64;
                if *left < min_left { min_left = *left; }
                if *top < min_top { min_top = *top; }
                if right > max_right { max_right = right; }
                if bottom > max_bottom { max_bottom = bottom; }
            }
            let width = (max_right - min_left).ceil() as u32;
            let height = (max_bottom - min_top).ceil() as u32;
            (width, height, min_left, min_top)
        };

        // 创建画布并填充背景色
        let mut canvas = image::RgbaImage::new(final_width, final_height);
        for pixel in canvas.pixels_mut() {
            *pixel = bg;
        }

        // 叠加图片（自动适应模式下需要偏移）
        for (img, left, top) in loaded_images {
            let x = (left - offset_x) as i64;
            let y = (top - offset_y) as i64;
            image::imageops::overlay(&mut canvas, &img, x, y);
        }

        // 转换为 base64
        let base64 = image_to_base64_png(&canvas)?;
        
        Ok(MergeResult {
            base64,
            width: final_width,
            height: final_height,
        })
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/src/image_tools.rs
git commit -m "feat: 实现 image_canvas_merge 命令"
```

---

## Task 4: 后端 - 注册命令

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 在 main.rs 中导入新命令**

在 `main.rs` 的导入区域添加：

```rust
use crate::image_tools::image_canvas_merge;
```

- [ ] **Step 2: 在 invoke_handler 中注册命令**

在 `invoke_handler` 的命令列表中添加 `image_canvas_merge`：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    image_canvas_merge,
])
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri
cargo check
```

Expected: 编译成功，无错误

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: 注册 image_canvas_merge 命令"
```

---

## Task 5: 前端 - 重构 merge Tab 的 UI 结构

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 替换 merge Tab 的模板部分**

找到 `<!-- Tab 2: 图片拼接 -->` 部分，替换为：

```vue
<!-- Tab 2: 自由画布拼图 -->
<div v-if="activeTab === 'merge'" class="tool-card">
  <div class="card-header">
    <span class="card-title">图片操作</span>
    <div class="card-actions">
      <el-button size="small" type="primary" @click="selectMergeFiles">选择图片</el-button>
      <el-button v-if="canvasImages.length" size="small" @click="clearCanvasImages">清空</el-button>
    </div>
  </div>
  <div class="card-body">
    <div v-if="canvasImages.length" class="merge-file-list">
      <div v-for="(f, i) in canvasImages" :key="f.id" class="merge-file-item">
        <img v-if="f.thumb" :src="f.thumb" class="merge-thumb" />
        <span class="file-name">{{ f.name }}</span>
        <span class="file-size">{{ formatBytes(f.size) }}</span>
        <el-button size="small" text type="danger" @click="removeCanvasImage(i)">移除</el-button>
      </div>
    </div>
    <div v-else class="upload-hint">选择图片添加到画布</div>
  </div>
</div>

<div v-if="activeTab === 'merge'" class="tool-card">
  <div class="card-header"><span class="card-title">画布</span></div>
  <div class="card-body">
    <canvas ref="fabricCanvasRef" class="fabric-canvas"></canvas>
  </div>
</div>

<div v-if="activeTab === 'merge'" class="tool-card">
  <div class="card-header"><span class="card-title">输出设置</span></div>
  <div class="card-body">
    <div class="action-grid">
      <div class="action-group">
        <span class="group-label">画布尺寸</span>
        <div class="group-buttons">
          <el-button size="small" :type="canvasSizeMode === 'auto' ? 'primary' : ''" @click="canvasSizeMode = 'auto'">自动适应</el-button>
          <el-button size="small" :type="canvasSizeMode === 'manual' ? 'primary' : ''" @click="canvasSizeMode = 'manual'">手动指定</el-button>
        </div>
      </div>
      <div v-if="canvasSizeMode === 'manual'" class="action-group">
        <span class="group-label">尺寸 (px)</span>
        <el-input-number v-model="manualCanvasWidth" :min="100" :max="8000" size="small" placeholder="宽" controls-position="right" style="width: 100px" />
        <span>×</span>
        <el-input-number v-model="manualCanvasHeight" :min="100" :max="8000" size="small" placeholder="高" controls-position="right" style="width: 100px" />
      </div>
      <div class="action-group">
        <span class="group-label">背景色</span>
        <el-color-picker v-model="mergeBgColor" size="small" />
      </div>
    </div>
    <div class="action-group" style="margin-top: 12px">
      <el-button size="small" type="primary" :disabled="canvasImages.length === 0" :loading="mergeLoading" @click="handleCanvasMerge">生成拼图</el-button>
      <el-button size="small" :disabled="!mergeResult" @click="downloadMergeResult">下载结果</el-button>
    </div>
    <div v-if="mergeResult" class="preview-area">
      <img :src="mergeResultUrl" class="merge-preview" />
    </div>
    <div v-if="error" class="error-message">{{ error }}</div>
  </div>
</div>
```

- [ ] **Step 2: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 重构 merge Tab UI 结构"
```

---

## Task 6: 前端 - 集成 Fabric.js 画布

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 导入 Fabric.js**

在 `<script setup>` 顶部添加：

```typescript
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import * as fabric from 'fabric'
```

- [ ] **Step 2: 定义画布相关状态**

在 merge Tab 的状态区域添加：

```typescript
interface CanvasImage {
  id: string
  path: string
  name: string
  size: number
  thumb?: string
  left: number
  top: number
  scaleX: number
  scaleY: number
  angle: number
}

const fabricCanvasRef = ref<HTMLCanvasElement>()
const fabricCanvas = ref<fabric.Canvas>()
const canvasImages = ref<CanvasImage[]>([])
const canvasSizeMode = ref<'auto' | 'manual'>('auto')
const manualCanvasWidth = ref(800)
const manualCanvasHeight = ref(600)
```

- [ ] **Step 3: 初始化 Fabric.js 画布**

添加生命周期钩子：

```typescript
onMounted(() => {
  if (fabricCanvasRef.value) {
    fabricCanvas.value = new fabric.Canvas(fabricCanvasRef.value, {
      width: 800,
      height: 600,
      backgroundColor: '#ffffff',
      selection: true,
    })
    
    // 监听对象修改事件，同步状态
    fabricCanvas.value.on('object:modified', (e) => {
      if (e.target) {
        const obj = e.target as fabric.FabricImage
        const id = (obj as any).customId
        const imgData = canvasImages.value.find(img => img.id === id)
        if (imgData) {
          imgData.left = obj.left || 0
          imgData.top = obj.top || 0
          imgData.scaleX = obj.scaleX || 1
          imgData.scaleY = obj.scaleY || 1
          imgData.angle = obj.angle || 0
        }
      }
    })
  }
})

onUnmounted(() => {
  fabricCanvas.value?.dispose()
})
```

- [ ] **Step 4: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 集成 Fabric.js 画布"
```

---

## Task 7: 前端 - 实现图片添加和画布交互

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 实现 selectMergeFiles 函数**

替换原有的 `selectMergeFiles` 函数：

```typescript
const selectMergeFiles = async () => {
  const selected = await open({
    multiple: true,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  
  for (const path of paths) {
    const name = path.split(/[/\\]/).pop() || ''
    let size = 0
    try {
      const info = await invoke<{ size: number }>('get_file_info', { filePath: path })
      size = info.size
    } catch { /* ignore */ }
    
    let thumb: string | undefined
    try {
      const thumbBase64 = await invoke<string>('get_thumbnail', { filePath: path })
      thumb = 'data:image/jpeg;base64,' + thumbBase64
    } catch { /* ignore */ }
    
    const id = `img_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const imgData: CanvasImage = {
      id,
      path,
      name,
      size,
      thumb,
      left: 100 + canvasImages.value.length * 20,
      top: 100 + canvasImages.value.length * 20,
      scaleX: 1,
      scaleY: 1,
      angle: 0,
    }
    
    canvasImages.value.push(imgData)
    
    // 添加到 Fabric 画布
    if (fabricCanvas.value && thumb) {
      fabric.Image.fromURL(thumb, (img) => {
        img.set({
          left: imgData.left,
          top: imgData.top,
          scaleX: imgData.scaleX,
          scaleY: imgData.scaleY,
          angle: imgData.angle,
        })
        ;(img as any).customId = id
        fabricCanvas.value!.add(img)
        fabricCanvas.value!.renderAll()
      })
    }
  }
  error.value = ''
}
```

- [ ] **Step 2: 实现 removeCanvasImage 函数**

```typescript
const removeCanvasImage = (index: number) => {
  const imgData = canvasImages.value[index]
  if (!imgData) return
  
  // 从 Fabric 画布移除
  if (fabricCanvas.value) {
    const objects = fabricCanvas.value.getObjects()
    const fabricObj = objects.find(obj => (obj as any).customId === imgData.id)
    if (fabricObj) {
      fabricCanvas.value.remove(fabricObj)
      fabricCanvas.value.renderAll()
    }
  }
  
  // 从状态移除
  canvasImages.value.splice(index, 1)
}
```

- [ ] **Step 3: 实现 clearCanvasImages 函数**

```typescript
const clearCanvasImages = () => {
  if (fabricCanvas.value) {
    fabricCanvas.value.clear()
    fabricCanvas.value.backgroundColor = '#ffffff'
    fabricCanvas.value.renderAll()
  }
  canvasImages.value = []
  mergeResult.value = null
  error.value = ''
}
```

- [ ] **Step 4: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 实现图片添加和画布交互"
```

---

## Task 8: 前端 - 实现生成和下载功能

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 实现 handleCanvasMerge 函数**

```typescript
const handleCanvasMerge = async () => {
  if (canvasImages.value.length === 0) return
  error.value = ''
  mergeLoading.value = true
  
  try {
    const images = canvasImages.value.map(img => ({
      file_path: img.path,
      left: img.left,
      top: img.top,
      scale_x: img.scaleX,
      scale_y: img.scaleY,
      angle: img.angle,
    }))
    
    const result = await invoke<{ base64: string; width: number; height: number }>('image_canvas_merge', {
      images,
      canvasWidth: canvasSizeMode.value === 'manual' ? manualCanvasWidth.value : null,
      canvasHeight: canvasSizeMode.value === 'manual' ? manualCanvasHeight.value : null,
      bgColor: mergeBgColor.value,
    })
    
    mergeResult.value = result
    ElMessage.success('拼图生成完成')
  } catch (e: any) {
    error.value = e
  } finally {
    mergeLoading.value = false
  }
}
```

- [ ] **Step 2: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 实现生成拼图功能"
```

---

## Task 9: 前端 - 添加右键菜单（置顶/置底）

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 添加右键菜单状态和模板**

在模板的画布卡片中添加右键菜单：

```vue
<!-- 右键菜单 -->
<div v-if="contextMenuVisible" class="canvas-context-menu" :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }">
  <div class="context-menu-item" @click="bringToFront">置顶</div>
  <div class="context-menu-item" @click="sendToBack">置底</div>
</div>
```

- [ ] **Step 2: 添加右键菜单逻辑**

在 script 中添加：

```typescript
const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })
const contextMenuTargetId = ref<string | null>(null)

// 在 onMounted 的画布初始化中添加右键事件
fabricCanvas.value.on('contextmenu', (e: any) => {
  if (e.target) {
    e.e.preventDefault()
    contextMenuTargetId.value = (e.target as any).customId || null
    contextMenuPos.value = { x: e.e.clientX, y: e.e.clientY }
    contextMenuVisible.value = true
  }
})

// 点击其他地方关闭菜单
document.addEventListener('click', () => {
  contextMenuVisible.value = false
})

const bringToFront = () => {
  if (!fabricCanvas.value || !contextMenuTargetId.value) return
  const obj = fabricCanvas.value.getObjects().find(o => (o as any).customId === contextMenuTargetId.value)
  if (obj) {
    fabricCanvas.value.bringObjectToFront(obj)
    fabricCanvas.value.renderAll()
  }
  contextMenuVisible.value = false
}

const sendToBack = () => {
  if (!fabricCanvas.value || !contextMenuTargetId.value) return
  const obj = fabricCanvas.value.getObjects().find(o => (o as any).customId === contextMenuTargetId.value)
  if (obj) {
    fabricCanvas.value.sendObjectToBack(obj)
    fabricCanvas.value.renderAll()
  }
  contextMenuVisible.value = false
}
```

- [ ] **Step 3: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 添加画布右键菜单（置顶/置底）"
```

---

## Task 10: 前端 - 添加画布样式

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue`

- [ ] **Step 1: 添加画布样式**

在 `<style scoped>` 中添加：

```css
.fabric-canvas {
  border: 1px solid var(--border-color);
  border-radius: 4px;
  display: block;
  margin: 0 auto;
  max-width: 100%;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 添加画布样式"
```

---

## Task 10: 测试验证

- [ ] **Step 1: 启动开发服务器**

```bash
npm run tauri dev
```

- [ ] **Step 2: 测试功能**

1. 切换到"图片工具" → "图片拼接" Tab
2. 点击"选择图片"，添加多张图片
3. 验证图片是否出现在画布上
4. 测试拖拽移动图片
5. 测试拖拽角点缩放图片
6. 测试拖拽旋转手柄旋转图片
7. 测试"自动适应"和"手动指定"两种画布尺寸模式
8. 点击"生成拼图"，验证结果
9. 点击"下载结果"，验证保存功能

- [ ] **Step 3: 提交最终版本**

```bash
git add .
git commit -m "feat: 完成自由画布拼图功能"
```

---

## 完成标准

- [ ] Fabric.js 依赖安装成功
- [ ] 后端 `image_canvas_merge` 命令实现并注册
- [ ] 前端画布初始化成功
- [ ] 图片可以添加到画布并显示
- [ ] 拖拽、缩放、旋转功能正常
- [ ] 生成拼图功能正常
- [ ] 下载功能正常
- [ ] 无 TypeScript 编译错误
- [ ] 无 Rust 编译错误
