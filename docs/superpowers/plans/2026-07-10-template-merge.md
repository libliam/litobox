# 模板拼图 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将图片拼接从 Fabric.js 自由画布重构为模板拼图，使用 CSS Grid + 原生 HTML5 Drag & Drop，零新依赖

**Architecture:** 前端用 CSS Grid 渲染模板预览，HTML5 Drag & Drop 交换槽位图片；后端用 `image_template_merge` 按槽位坐标渲染（cover 模式 + 居中裁剪）。输出尺寸固定 1200×800px。

**Tech Stack:** Vue 3 + CSS Grid + HTML5 Drag & Drop API + Rust image crate

---

## File Structure

| 文件 | 职责 | 操作 |
|------|------|------|
| `package.json` | 移除 fabric 依赖 | 修改 |
| `src/views/ImageToolEnhanced.vue` | 模板拼图 UI（模板选择器、预览区、拖拽交换） | 重写 merge Tab 部分 |
| `src-tauri/src/image_tools.rs` | 新命令 `image_template_merge` | 替换 `image_canvas_merge` |
| `src-tauri/src/main.rs` | 命令注册 | 修改 |

---

### Task 1: 移除 fabric 依赖

**Files:**
- Modify: `package.json`
- Modify: `src/views/ImageToolEnhanced.vue:250`

- [ ] **Step 1: 卸载 fabric 包**

```powershell
npm uninstall fabric
```

- [ ] **Step 2: 移除 import 语句**

在 `src/views/ImageToolEnhanced.vue` 第 250 行，删除：
```typescript
import * as fabric from 'fabric'
```

同时移除不再需要的 `watch`、`onUnmounted`、`nextTick` 导入（如果 merge Tab 是最后一次使用）。当前其他 Tab 不使用这些，所以从第 245 行改为：

```typescript
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
```

改为：

```typescript
import { ref, computed, onMounted } from 'vue'
```

- [ ] **Step 3: 验证编译**

```powershell
npm run dev
```

预期：Vite 编译成功，无 fabric 相关错误。

- [ ] **Step 4: 提交**

```powershell
git add package.json package-lock.json src/views/ImageToolEnhanced.vue
git commit -m "chore: 移除 fabric 依赖，为模板拼图方案做准备"
```

---

### Task 2: 后端 - 替换为模板拼图渲染命令

**Files:**
- Modify: `src-tauri/src/image_tools.rs:250-359`

- [ ] **Step 1: 替换 CanvasImageInput 为 MergeSlotInput**

在 `src-tauri/src/image_tools.rs` 第 252-260 行，将：

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

替换为：

```rust
#[derive(serde::Deserialize)]
pub struct MergeSlotInput {
    pub file_path: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

- [ ] **Step 2: 替换 image_canvas_merge 命令**

将第 262-274 行：

```rust
#[tauri::command]
pub async fn image_canvas_merge(
    images: Vec<CanvasImageInput>,
    canvas_width: Option<u32>,
    canvas_height: Option<u32>,
    bg_color: String,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_canvas_merge(images, canvas_width, canvas_height, bg_color)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
```

替换为：

```rust
#[tauri::command]
pub async fn image_template_merge(
    images: Vec<MergeSlotInput>,
    canvas_width: u32,
    canvas_height: u32,
    bg_color: String,
    gap: u32,
) -> Result<MergeResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_image_template_merge(images, canvas_width, canvas_height, bg_color, gap)
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
```

- [ ] **Step 3: 替换 do_image_canvas_merge 为 do_image_template_merge**

将第 276-359 行（`fn do_image_canvas_merge` 整个函数）替换为：

```rust
fn do_image_template_merge(
    images: Vec<MergeSlotInput>,
    canvas_width: u32,
    canvas_height: u32,
    bg_color: String,
    gap: u32,
) -> Result<MergeResult, String> {
    if images.is_empty() {
        return Err("请至少选择一张图片".into());
    }

    let is_transparent = bg_color.is_empty() || bg_color == "transparent";
    let bg = if is_transparent {
        Rgba([0, 0, 0, 0])
    } else {
        parse_color(&bg_color)?
    };

    let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, bg);

    for img_input in &images {
        let bytes = std::fs::read(&img_input.file_path)
            .map_err(|e| format!("读取文件失败 ({}): {}", img_input.file_path, e))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("无法解码图片 ({}): {}", img_input.file_path, e))?;

        // cover 模式：等比缩放填满槽位，居中裁剪
        let slot_w = img_input.width;
        let slot_h = img_input.height;
        let img_w = img.width();
        let img_h = img.height();

        let scale = (slot_w as f64 / img_w as f64).max(slot_h as f64 / img_h as f64);
        let scaled_w = (img_w as f64 * scale) as u32;
        let scaled_h = (img_h as f64 * scale) as u32;

        let resized = img.resize(scaled_w, scaled_h, FilterType::Lanczos3);

        // 居中裁剪到槽位尺寸
        let crop_x = (scaled_w.saturating_sub(slot_w) / 2) as u32;
        let crop_y = (scaled_h.saturating_sub(slot_h) / 2) as u32;
        let cropped = resized.crop_imm(crop_x, crop_y, slot_w, slot_h);

        imageops::overlay(&mut canvas, &cropped, img_input.x as i64, img_input.y as i64);
    }

    let merged = DynamicImage::ImageRgba8(canvas);
    let base64 = image_to_base64_png(&merged)?;

    Ok(MergeResult {
        base64,
        width: canvas_width,
        height: canvas_height,
    })
}
```

- [ ] **Step 4: 检查编译**

```powershell
cd src-tauri; cargo check
```

预期：编译成功，无错误（预存 warning 忽略）。

- [ ] **Step 5: 提交**

```powershell
git add src-tauri/src/image_tools.rs
git commit -m "refactor: 将 image_canvas_merge 替换为 image_template_merge（槽位坐标渲染）"
```

---

### Task 3: 更新 main.rs 命令注册

**Files:**
- Modify: `src-tauri/src/main.rs:140-141`

- [ ] **Step 1: 替换命令注册**

在 `src-tauri/src/main.rs` 第 140-141 行，将：

```rust
            image_tools::image_merge,
            image_tools::image_canvas_merge,
```

替换为：

```rust
            image_tools::image_merge,
            image_tools::image_template_merge,
```

注意：`image_merge` 是旧版线性拼接命令（其他工具在用），不能删除。

- [ ] **Step 2: 检查编译**

```powershell
cd src-tauri; cargo check
```

预期：编译成功。

- [ ] **Step 3: 提交**

```powershell
git add src-tauri/src/main.rs
git commit -m "refactor: 注册 image_template_merge 替换 image_canvas_merge"
```

---

### Task 4: 前端 - 重写 merge Tab 模板

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue:70-137`

- [ ] **Step 1: 替换 merge Tab 的 HTML 模板**

将第 70-137 行（从 `<!-- Tab 2: 自由画布拼图 -->` 到 `</div>` 截止于输出设置区域末尾）替换为：

```html
    <!-- Tab 2: 模板拼图 -->
    <div v-if="activeTab === 'merge'" class="tool-card">
      <div class="card-header">
        <span class="card-title">图片选择</span>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="selectMergeFiles">选择图片</el-button>
          <el-button v-if="mergeImages.length" size="small" @click="clearMergeImages">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <div v-if="mergeImages.length" class="merge-file-list">
          <div v-for="(f, i) in mergeImages" :key="i" class="merge-file-item">
            <img :src="f.thumb" class="merge-thumb" />
            <span class="file-name">{{ f.name }}</span>
            <span class="file-size">{{ formatBytes(f.size) }}</span>
            <el-button size="small" text type="danger" @click="removeMergeImage(i)">移除</el-button>
          </div>
        </div>
        <div v-else class="upload-hint">选择图片开始拼图</div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && availableTemplates.length" class="tool-card">
      <div class="card-header"><span class="card-title">拼图模板</span></div>
      <div class="card-body">
        <div class="template-grid">
          <div
            v-for="tpl in availableTemplates"
            :key="tpl.id"
            class="template-item"
            :class="{ 'template-active': currentTemplate?.id === tpl.id }"
            @click="selectTemplate(tpl)"
          >
            <div class="template-preview" :style="templatePreviewStyle(tpl)">
              <div
                v-for="(slot, si) in tpl.grid"
                :key="si"
                class="template-slot"
                :style="slotStyle(slot)"
              ></div>
            </div>
            <span class="template-name">{{ tpl.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && currentTemplate" class="tool-card">
      <div class="card-header"><span class="card-title">拼图预览</span></div>
      <div class="card-body">
        <div class="merge-preview-grid" :style="mergeGridStyle">
          <div
            v-for="(slot, si) in currentTemplate.grid"
            :key="si"
            class="merge-slot"
            :class="{
              'merge-slot-drag-over': dragOverSlot === si,
              'merge-slot-empty': slotMap[si] === null,
              'merge-slot-dragging': dragFromSlot === si,
            }"
            :style="slotStyle(slot)"
            @dragover.prevent="onDragOver(si)"
            @dragleave="onDragLeave"
            @drop="onDrop(si)"
          >
            <img
              v-if="slotMap[si] !== null && mergeImages[slotMap[si]!]"
              :src="mergeImages[slotMap[si]!].thumb"
              class="merge-slot-img"
              draggable="true"
              @dragstart="onDragStart(si)"
              @dragend="onDragEnd"
            />
            <span v-else class="merge-slot-placeholder">拖入图片</span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'merge' && currentTemplate" class="tool-card">
      <div class="card-header"><span class="card-title">输出设置</span></div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <span class="group-label">背景色</span>
            <el-color-picker v-model="mergeBgColor" size="small" show-alpha />
            <el-button size="small" @click="mergeBgColor = ''" style="margin-left: 8px">透明</el-button>
          </div>
          <div class="action-group">
            <span class="group-label">间距</span>
            <el-input-number v-model="mergeGap" :min="0" :max="100" size="small" controls-position="right" style="width: 90px" />
            <span style="font-size: 12px; color: var(--text-secondary)">px</span>
          </div>
        </div>
        <div class="action-group" style="margin-top: 12px">
          <el-button size="small" type="primary" :disabled="!hasFilledSlots" :loading="mergeLoading" @click="handleTemplateMerge">生成拼图</el-button>
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

```powershell
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 重写 merge Tab 为模板拼图 HTML 模板"
```

---

### Task 5: 前端 - 重写 merge Tab 脚本逻辑

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue:373-765`

- [ ] **Step 1: 替换 merge Tab 的所有脚本代码**

将第 373-765 行（从 `// ============ Tab 2: 自由画布拼图 ============` 到 `const downloadMergeResult` 结束）替换为：

```typescript
// ============ Tab 2: 模板拼图 ============

// 模板定义
interface TemplateSlot {
  colStart: number
  colEnd: number
  rowStart: number
  rowEnd: number
}

interface Template {
  id: string
  name: string
  count: number
  grid: TemplateSlot[]
}

const TEMPLATES: Template[] = [
  { id: 'h2', name: '左右2列', count: 2, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'v2', name: '上下2行', count: 2, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l2', name: '左大右小', count: 2, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 'h3', name: '三等分', count: 3, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
  ]},
  { id: 't3', name: '上1下2', count: 3, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l3', name: '左1右2', count: 3, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'g4', name: '四宫格', count: 4, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'l4', name: '左大右3', count: 4, grid: [
    { colStart: 1, colEnd: 3, rowStart: 1, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 3, rowEnd: 4 },
  ]},
  { id: 'h5', name: '五宫格', count: 5, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
  { id: 'g6', name: '六宫格', count: 6, grid: [
    { colStart: 1, colEnd: 2, rowStart: 1, rowEnd: 2 },
    { colStart: 2, colEnd: 3, rowStart: 1, rowEnd: 2 },
    { colStart: 3, colEnd: 4, rowStart: 1, rowEnd: 2 },
    { colStart: 1, colEnd: 2, rowStart: 2, rowEnd: 3 },
    { colStart: 2, colEnd: 3, rowStart: 2, rowEnd: 3 },
    { colStart: 3, colEnd: 4, rowStart: 2, rowEnd: 3 },
  ]},
]

interface MergeImage {
  path: string
  name: string
  size: number
  thumb: string
}

const mergeImages = ref<MergeImage[]>([])
const slotMap = ref<(number | null)[]>([])
const currentTemplate = ref<Template | null>(null)
const mergeBgColor = ref('#ffffff')
const mergeGap = ref(4)
const mergeResult = ref<{ base64: string; width: number; height: number } | null>(null)
const mergeLoading = ref(false)

// 拖拽状态
const dragFromSlot = ref<number | null>(null)
const dragOverSlot = ref<number | null>(null)

const mergeResultUrl = computed(() =>
  mergeResult.value ? 'data:image/png;base64,' + mergeResult.value.base64 : ''
)

// 根据图片数量过滤可用模板
const availableTemplates = computed(() =>
  TEMPLATES.filter(t => t.count <= mergeImages.value.length)
)

// 至少有 1 个槽位有图片
const hasFilledSlots = computed(() =>
  slotMap.value.some(s => s !== null)
)

// 模板预览图（缩略版 CSS Grid）
const templatePreviewStyle = (tpl: Template) => {
  const cols = Math.max(...tpl.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...tpl.grid.map(s => s.rowEnd)) - 1
  return {
    gridTemplateColumns: `repeat(${cols}, 1fr)`,
    gridTemplateRows: `repeat(${rows}, 1fr)`,
  }
}

// 拼图预览 Grid（固定 1200x800 比例）
const mergeGridStyle = computed(() => {
  if (!currentTemplate.value) return {}
  const cols = Math.max(...currentTemplate.value.grid.map(s => s.colEnd)) - 1
  const rows = Math.max(...currentTemplate.value.grid.map(s => s.rowEnd)) - 1
  return {
    gridTemplateColumns: `repeat(${cols}, 1fr)`,
    gridTemplateRows: `repeat(${rows}, 1fr)`,
    aspectRatio: '1200 / 800',
    gap: mergeGap.value + 'px',
  }
})

const slotStyle = (slot: TemplateSlot) => ({
  gridColumn: `${slot.colStart} / ${slot.colEnd}`,
  gridRow: `${slot.rowStart} / ${slot.rowEnd}`,
})

// 选择模板
const selectTemplate = (tpl: Template) => {
  currentTemplate.value = tpl
  // 保留已有图片映射，多余槽位留空，不足的用 null 填充
  const newSlots: (number | null)[] = new Array(tpl.grid.length).fill(null)
  for (let i = 0; i < Math.min(tpl.grid.length, slotMap.value.length); i++) {
    newSlots[i] = slotMap.value[i]
  }
  slotMap.value = newSlots
}

// 选择图片
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

    let thumb = ''
    try {
      const thumbBase64 = await invoke<string>('get_thumbnail', { filePath: path })
      thumb = 'data:image/jpeg;base64,' + thumbBase64
    } catch { /* ignore */ }

    mergeImages.value.push({ path, name, size, thumb })
  }

  // 自动选择首个匹配模板
  if (!currentTemplate.value && availableTemplates.value.length > 0) {
    selectTemplate(availableTemplates.value[0])
  } else if (currentTemplate.value) {
    // 更新 slotMap：新图片填充到空槽位
    const tpl = currentTemplate.value
    let imgIdx = 0
    const newSlots: (number | null)[] = new Array(tpl.grid.length).fill(null)
    for (let i = 0; i < tpl.grid.length && imgIdx < mergeImages.value.length; i++) {
      newSlots[i] = imgIdx++
    }
    slotMap.value = newSlots
  }

  error.value = ''
}

// 删除图片
const removeMergeImage = (index: number) => {
  mergeImages.value.splice(index, 1)
  // 从 slotMap 中移除，并重新映射
  slotMap.value = slotMap.value.map(s => {
    if (s === null) return null
    if (s === index) return null
    return s > index ? s - 1 : s
  })
  // 检查模板是否仍可用
  if (currentTemplate.value && currentTemplate.value.count > mergeImages.value.length) {
    currentTemplate.value = null
    slotMap.value = []
  }
}

// 清空
const clearMergeImages = () => {
  mergeImages.value = []
  slotMap.value = []
  currentTemplate.value = null
  mergeResult.value = null
  error.value = ''
}

// 拖拽开始
const onDragStart = (slotIndex: number) => {
  dragFromSlot.value = slotIndex
}

// 拖拽结束
const onDragEnd = () => {
  dragFromSlot.value = null
  dragOverSlot.value = null
}

// 拖拽经过
const onDragOver = (slotIndex: number) => {
  dragOverSlot.value = slotIndex
}

// 拖拽离开
const onDragLeave = () => {
  dragOverSlot.value = null
}

// 放置
const onDrop = (targetSlot: number) => {
  if (dragFromSlot.value === null) return
  const from = dragFromSlot.value
  // 交换两个槽位的图片
  const temp = slotMap.value[from]
  slotMap.value[from] = slotMap.value[targetSlot]
  slotMap.value[targetSlot] = temp
  dragFromSlot.value = null
  dragOverSlot.value = null
}

// 生成拼图
const handleTemplateMerge = async () => {
  if (!currentTemplate.value || !hasFilledSlots.value) return
  error.value = ''
  mergeLoading.value = true

  try {
    const cols = Math.max(...currentTemplate.value.grid.map(s => s.colEnd)) - 1
    const rows = Math.max(...currentTemplate.value.grid.map(s => s.rowEnd)) - 1
    const canvasWidth = 1200
    const canvasHeight = 800
    const gap = mergeGap.value

    const slotWidth = (canvasWidth - gap * (cols - 1)) / cols
    const slotHeight = (canvasHeight - gap * (rows - 1)) / rows

    const images: { file_path: string; x: number; y: number; width: number; height: number }[] = []

    currentTemplate.value.grid.forEach((slot, si) => {
      const imgIdx = slotMap.value[si]
      if (imgIdx === null) return
      const img = mergeImages.value[imgIdx]
      if (!img) return

      const col = slot.colStart - 1
      const row = slot.rowStart - 1
      const colSpan = slot.colEnd - slot.colStart
      const rowSpan = slot.rowEnd - slot.rowStart

      images.push({
        file_path: img.path,
        x: Math.round(col * (slotWidth + gap)),
        y: Math.round(row * (slotHeight + gap)),
        width: Math.round(slotWidth * colSpan + gap * (colSpan - 1)),
        height: Math.round(slotHeight * rowSpan + gap * (rowSpan - 1)),
      })
    })

    const result = await invoke<{ base64: string; width: number; height: number }>('image_template_merge', {
      images,
      canvasWidth,
      canvasHeight,
      bgColor: mergeBgColor.value,
      gap,
    })

    mergeResult.value = result
    ElMessage.success('拼图生成完成')
  } catch (e: any) {
    error.value = e
  } finally {
    mergeLoading.value = false
  }
}

const downloadMergeResult = async () => {
  if (!mergeResult.value) return
  const blob = base64ToBlob(mergeResult.value.base64)
  await saveFileWithDialog(blob, 'merged.png', 'png')
}
```

- [ ] **Step 2: 提交**

```powershell
git add src/views/ImageToolEnhanced.vue
git commit -m "feat: 重写 merge Tab 脚本为模板拼图逻辑"
```

---

### Task 6: 前端 - 添加模板拼图样式

**Files:**
- Modify: `src/views/ImageToolEnhanced.vue` (scoped style 区域)

- [ ] **Step 1: 替换 merge 相关样式**

在 `<style scoped>` 区域，将现有的 merge 相关样式（第 950-963 行 `merge-file-list`、`merge-file-item`、`merge-thumb`）保留，并删除以下样式（从 ~1076 行开始）：

- `.fabric-canvas` (1076-1082)
- `.canvas-context-menu` (1084-1090)

然后在样式末尾添加模板拼图新样式。在 `</style>` 之前追加：

```css
/* 模板选择器 */
.template-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.template-item {
  cursor: pointer;
  padding: 6px;
  border: 2px solid transparent;
  border-radius: 6px;
  transition: border-color 0.2s;
  text-align: center;
}
.template-item:hover {
  border-color: var(--color-accent);
}
.template-active {
  border-color: var(--color-accent);
  background: rgba(0, 255, 255, 0.05);
}
.template-preview {
  display: grid;
  width: 80px;
  height: 60px;
  gap: 2px;
  margin-bottom: 4px;
}
.template-slot {
  background: var(--border-color);
  border-radius: 2px;
}
.template-name {
  font-size: 11px;
  color: var(--text-secondary);
}

/* 拼图预览 */
.merge-preview-grid {
  display: grid;
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
  background: var(--bg-input);
  border-radius: 4px;
  overflow: hidden;
}
.merge-slot {
  position: relative;
  background: var(--border-color);
  border: 2px dashed transparent;
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  min-height: 60px;
  transition: border-color 0.2s, opacity 0.2s;
}
.merge-slot-drag-over {
  border-color: var(--color-accent);
}
.merge-slot-dragging {
  opacity: 0.4;
}
.merge-slot-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: grab;
}
.merge-slot-img:active {
  cursor: grabbing;
}
.merge-slot-placeholder {
  font-size: 12px;
  color: var(--text-secondary);
}
```

- [ ] **Step 2: 提交**

```powershell
git add src/views/ImageToolEnhanced.vue
git commit -m "style: 添加模板拼图样式（模板选择器、预览区、拖拽状态）"
```

---

### Task 7: 端到端验证

- [ ] **Step 1: 启动应用**

```powershell
npm run tauri dev
```

- [ ] **Step 2: 功能测试**

1. 切换到"图片工具" → "图片拼图" Tab
2. 选择 2-3 张图片 → 验证模板自动过滤并显示
3. 点击不同模板 → 验证预览区切换
4. 拖拽图片到另一个槽位 → 验证交换成功
5. 删除一张图片 → 验证模板重新过滤
6. 设置背景色 + 间距
7. 点击"生成拼图" → 验证预览出现
8. 点击"下载结果" → 验证 PNG 文件正确

- [ ] **Step 3: 提交**（如有小修复）

```powershell
git add -A
git commit -m "fix: 模板拼图端到端测试修复"
```