# PDF 压缩工具 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 PdfTool.vue 新增"PDF压缩"Tab，纯 Rust 后端（lopdf + image crate）实现 3 档压缩 + 批量处理，Ghostscript 可选增强。

**Architecture:** 新增 `pdf_tools.rs` Rust 模块，在 `spawn_blocking` 中执行 PDF 图片重采样、元数据清理、流压缩。前端新增 Tab 以文件列表 + 压缩设置 + 结果表格呈现。

**Tech Stack:** Rust (lopdf + image crate + flate2), Vue 3 + TypeScript + Element Plus

**Spec:** `docs/superpowers/specs/2026-07-14-pdf-compress-design.md`

---

## 文件结构

| 操作 | 文件 | 职责 |
|------|------|------|
| 修改 | `src-tauri/Cargo.toml` | 新增 `lopdf` 依赖 |
| 新建 | `src-tauri/src/pdf_tools.rs` | PDF 压缩 Rust 后端 |
| 修改 | `src-tauri/src/main.rs` | 注册新模块和命令 |
| 修改 | `src/views/PdfTool.vue` | 新增 "PDF压缩" Tab |
| 修改 | `src/store/index.ts` | TOOL_LIST 更新关键词 |
| 修改 | `package.json` | 版本号 V5.0.0 → V5.1.0 |
| 修改 | `src-tauri/Cargo.toml` | 版本号同步 |
| 修改 | `src-tauri/tauri.conf.json` | 版本号同步 |
| 修改 | `README.md` | 版本号 + 功能记录 |

---

### Task 1: 添加 lopdf 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 添加 lopdf**

```toml
# 在 [dependencies] 中 image = "0.25" 之后添加:
lopdf = "0.34"
```

- [ ] **Step 2: 运行 cargo check 验证依赖**

Run: `cd src-tauri; cargo check`
Expected: 无编译错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: 添加 lopdf 依赖用于 PDF 压缩"
```

---

### Task 2: 创建 Rust 后端 pdf_tools.rs

**Files:**
- Create: `src-tauri/src/pdf_tools.rs`

- [ ] **Step 1: 创建文件骨架**

```rust
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use image::{DynamicImage, ImageFormat, GenericImageView};
use image::imageops::FilterType;
use lopdf::{Document, Object, Stream};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct CompressResult {
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
}

static GS_AVAILABLE: AtomicBool = AtomicBool::new(false);
static GS_CHECKED: AtomicBool = AtomicBool::new(false);
```

- [ ] **Step 2: 实现 detect_ghostscript 命令**

```rust
#[tauri::command]
pub fn detect_ghostscript() -> bool {
    if GS_CHECKED.load(Ordering::Relaxed) {
        return GS_AVAILABLE.load(Ordering::Relaxed);
    }
    let available = Command::new("where")
        .arg("gswin64c.exe")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("where")
            .arg("gswin32c.exe")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    GS_AVAILABLE.store(available, Ordering::Relaxed);
    GS_CHECKED.store(true, Ordering::Relaxed);
    available
}
```

- [ ] **Step 3: 实现 get_pdf_page_count 命令**

```rust
#[tauri::command]
pub fn get_pdf_page_count(file_path: String) -> Result<u32, String> {
    let doc = Document::load(&file_path).map_err(|e| format!("PDF 加载失败: {}", e))?;
    Ok(doc.get_pages().len() as u32)
}
```

- [ ] **Step 4: 实现 compress_pdf 核心逻辑**

```rust
#[tauri::command]
pub async fn compress_pdf(
    file_path: String,
    level: u8,
    gs_available: bool,
) -> Result<CompressResult, String> {
    let original_size = fs::metadata(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?
        .len();

    let result = tauri::async_runtime::spawn_blocking(move || {
        do_compress_pdf(&file_path, level, gs_available)
    })
    .await
    .map_err(|e| format!("压缩线程异常: {}", e))??;

    Ok(CompressResult {
        compressed_size: fs::metadata(&result.output_path)
            .map_err(|e| format!("读取输出文件失败: {}", e))?
            .len(),
        original_size,
        ..result
    })
}

fn do_compress_pdf(file_path: &str, level: u8, gs_available: bool) -> Result<CompressResult, String> {
    let mut doc = Document::load(file_path)
        .map_err(|e| format!("PDF 加载失败: {}", e))?;

    // 检查是否加密
    if doc.is_encrypted() {
        return Err("不支持加密 PDF".into());
    }

    // 压缩参数
    let (target_dpi, jpeg_quality) = match level {
        1 => (150.0, 85u8),
        2 => (150.0, 70u8),
        _ => (72.0, 50u8),
    };

    // 遍历所有页面，处理图片 XObject
    let page_ids: Vec<u32> = doc.get_pages().keys().copied().collect();
    for page_id in &page_ids {
        if let Ok(page) = doc.get_page(*page_id) {
            let resources = page.resources.clone();
            // 递归查找 XObject 图片
            process_page_resources(&mut doc, &resources, target_dpi, jpeg_quality)?;
        }
    }

    // 元数据清理
    if level >= 1 {
        // 移除 XMP 元数据流
        doc.objects.retain(|_id, obj| {
            if let Ok(stream) = obj.as_stream() {
                if let Ok(dict) = stream.dict.try_borrow() {
                    if dict.get(b"Type").ok().map(|o| o.as_name().ok() == Some(b"Metadata")) == Some(true) {
                        return false;
                    }
                    if dict.get(b"Subtype").ok().map(|o| o.as_name().ok() == Some(b"XML")) == Some(true) {
                        return false;
                    }
                }
            }
            true
        });
        // 移除文档注释/标记内容
        doc.trailer.remove(b"MarkInfo");
    }
    if level >= 2 {
        // 清除文档信息
        doc.trailer.remove(b"Info");
    }

    // 保存临时文件
    let temp_dir = std::env::temp_dir();
    let file_stem = PathBuf::from(file_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let temp_output = temp_dir.join(format!("{}_compressed.pdf", file_stem));
    doc.save(&temp_output).map_err(|e| format!("保存 PDF 失败: {}", e))?;

    // 极限压缩：尝试 Ghostscript
    if level == 3 && gs_available {
        let gs_output = temp_dir.join(format!("{}_gs_compressed.pdf", file_stem));
        let gs_result = Command::new("gswin64c.exe")
            .args([
                "-sDEVICE=pdfwrite",
                "-dPDFSETTINGS=/screen",
                "-dNOPAUSE",
                "-dQUIET",
                "-dBATCH",
                &format!("-sOutputFile={}", gs_output.to_string_lossy()),
                &temp_output.to_string_lossy().replace('\\', "/"),
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        if let Ok(output) = gs_result {
            if output.status.success() && gs_output.exists() {
                let rust_size = fs::metadata(&temp_output).map(|m| m.len()).unwrap_or(0);
                let gs_size = fs::metadata(&gs_output).map(|m| m.len()).unwrap_or(0);
                if gs_size < rust_size {
                    let _ = fs::remove_file(&temp_output);
                    let _ = fs::rename(&gs_output, &temp_output);
                } else {
                    let _ = fs::remove_file(&gs_output);
                }
            }
        }
    }

    Ok(CompressResult {
        output_path: temp_output.to_string_lossy().to_string(),
        original_size: 0, // 由上层填充
        compressed_size: 0, // 由上层填充
    })
}

fn process_page_resources(
    doc: &mut Document,
    resources: &lopdf::Object,
    target_dpi: f64,
    jpeg_quality: u8,
) -> Result<(), String> {
    let dict = match resources.as_dict() {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };

    // 处理 XObject
    if let Ok(xobjects) = dict.get(b"XObject") {
        if let Ok(xobj_dict) = xobjects.as_dict() {
            for (_, obj_id) in xobj_dict.iter() {
                if let Ok(obj_id) = obj_id.as_reference() {
                    if let Ok(obj) = doc.get_object_mut(obj_id) {
                        if let Ok(stream) = obj.as_stream() {
                            if let Ok(sdict) = stream.dict.try_borrow() {
                                if sdict.get(b"Subtype").ok().map(|o| o.as_name().ok()) == Some(Some(b"Image")) {
                                    // 提取图片数据并重采样
                                    let (width, height) = (
                                        sdict.get(b"Width").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0),
                                        sdict.get(b"Height").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0),
                                    );
                                    if width > 0 && height > 0 {
                                        let raw_data = stream.content.clone();
                                        if let Ok(img) = image::load_from_memory(&raw_data) {
                                            let new_img = resize_image(&img, width as u32, height as u32, target_dpi);
                                            let mut buf = Vec::new();
                                            let _ = new_img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg);
                                            // 重建 stream
                                            let mut new_stream = Stream::new(
                                                lopdf::Dictionary::new(),
                                                buf,
                                            );
                                            new_stream.compress();
                                            *obj = Object::Stream(new_stream);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn resize_image(img: &DynamicImage, orig_width: u32, orig_height: u32, target_dpi: f64) -> DynamicImage {
    // 假设原始 DPI 为 300，计算目标尺寸
    let assumed_dpi = 300.0;
    if target_dpi >= assumed_dpi {
        return img.clone(); // 不放大
    }
    let scale = target_dpi / assumed_dpi;
    let new_w = (orig_width as f64 * scale) as u32;
    let new_h = (orig_height as f64 * scale) as u32;
    if new_w < 1 || new_h < 1 {
        return img.clone();
    }
    img.resize_exact(new_w, new_h, FilterType::Lanczos3)
}
```

- [ ] **Step 5: 运行 cargo check 验证**

Run: `cd src-tauri; cargo check`
Expected: 无编译错误

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/pdf_tools.rs
git commit -m "feat: 新增 PDF 压缩 Rust 后端（lopdf + image crate）"
```

---

### Task 3: 注册新模块到 main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 添加模块声明和命令注册**

在 `mod audio_tools;` 之后添加：
```rust
mod pdf_tools;
```

在 `invoke_handler` 中 `audio_tools::get_audio_preview,` 之后添加：
```rust
            pdf_tools::detect_ghostscript,
            pdf_tools::compress_pdf,
            pdf_tools::get_pdf_page_count,
```

- [ ] **Step 2: 运行 cargo check 验证**

Run: `cd src-tauri; cargo check`
Expected: 无编译错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: 注册 pdf_tools 模块和命令"
```

---

### Task 4: 前端新增 "PDF压缩" Tab

**Files:**
- Modify: `src/views/PdfTool.vue`

- [ ] **Step 1: 在模板 Tab 栏添加新 Tab**

在 `<el-tab-pane label="PDF合并/拆分" name="mergeSplit" />` 之后添加：
```html
        <el-tab-pane label="PDF压缩" name="compress" />
```

- [ ] **Step 2: 添加压缩 Tab 的模板（文件选择卡片）**

在 `<!-- Tab 4: PDF合并/拆分 -->` 的最后一个 `</div>` 之后、`</template>` 之前添加：

```html
    <!-- Tab 6: PDF压缩 -->
    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">PDF 输入</span>
          <el-tooltip placement="top" effect="dark">
            <template #content>
              <div class="tooltip-content">
                <p>支持拖入多个 PDF 文件批量压缩</p>
                <p>单文件最大 100MB</p>
              </div>
            </template>
            <el-icon class="hint-icon"><QuestionFilled /></el-icon>
          </el-tooltip>
        </div>
        <div class="card-actions">
          <el-button size="small" type="primary" @click="triggerCompressInput">添加 PDF</el-button>
          <el-button v-if="compressFiles.length" size="small" @click="handleClearCompressFiles">清空</el-button>
        </div>
      </div>
      <div class="card-body">
        <input
          ref="compressInputRef"
          type="file"
          accept=".pdf"
          multiple
          style="display: none"
          @change="handleCompressFileSelect"
        />
        <div
          class="compress-drop-zone"
          @dragover.prevent="isDragOver = true"
          @dragleave.prevent="isDragOver = false"
          @drop.prevent="handleCompressDrop"
          :class="{ 'drag-over': isDragOver }"
        >
          <div v-if="compressFiles.length" class="compress-file-list">
            <div v-for="(file, idx) in compressFiles" :key="idx" class="compress-file-item">
              <span class="file-index">{{ idx + 1 }}</span>
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <el-button size="small" type="danger" link @click="handleRemoveCompressFile(idx)">移除</el-button>
            </div>
          </div>
          <div v-else class="upload-hint">点击「添加 PDF」或拖入 PDF 文件</div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress'" class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩设置</span>
      </div>
      <div class="card-body">
        <div class="action-grid">
          <div class="action-group">
            <div class="group-label">压缩级别</div>
            <el-radio-group v-model="compressLevel" size="small">
              <el-radio-button :value="1">快速压缩</el-radio-button>
              <el-radio-button :value="2">标准压缩</el-radio-button>
              <el-radio-button :value="3">极限压缩</el-radio-button>
            </el-radio-group>
          </div>
          <div class="action-group">
            <div class="group-label">执行</div>
            <div class="group-buttons">
              <el-button
                type="primary"
                size="small"
                :disabled="!compressFiles.length || isCompressing"
                :loading="isCompressing"
                @click="handleCompress"
              >
                开始压缩
              </el-button>
            </div>
          </div>
        </div>
        <div class="compress-level-hint">
          {{ compressLevelHint }}
        </div>
        <div v-if="gsAvailable" class="gs-hint">
          <el-icon><Check /></el-icon>
          <span>已检测到 Ghostscript，「极限压缩」将获得更佳效果</span>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress' && compressResults.length" class="tool-card">
      <div class="card-header">
        <span class="card-title">压缩结果</span>
        <div class="card-actions">
          <el-button size="small" @click="handleSaveAllCompressed">全部保存</el-button>
        </div>
      </div>
      <div class="card-body">
        <el-table :data="compressResults" stripe size="small" class="compress-table">
          <el-table-column prop="fileName" label="文件" min-width="200" />
          <el-table-column label="原始大小" width="110">
            <template #default="{ row }">
              <span>{{ formatFileSize(row.originalSize) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="压缩后" width="110">
            <template #default="{ row }">
              <span>{{ formatFileSize(row.compressedSize) }}</span>
            </template>
          </el-table-column>
          <el-table-column label="压缩率" width="90">
            <template #default="{ row }">
              <span :class="row.ratio > 0 ? 'ratio-positive' : 'ratio-negative'">
                {{ row.ratio > 0 ? `-${row.ratio}%` : `+${Math.abs(row.ratio)}%` }}
              </span>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="90">
            <template #default="{ row, $index }">
              <el-button size="small" @click="handleSaveSingleCompressed($index)">保存</el-button>
            </template>
          </el-table-column>
        </el-table>
        <div class="compress-summary">
          <span>合计：{{ formatFileSize(totalOriginalSize) }} → {{ formatFileSize(totalCompressedSize) }}，</span>
          <span :class="totalRatio > 0 ? 'ratio-positive' : 'ratio-negative'">
            缩小 {{ totalRatio }}%
          </span>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'compress' && compressError" class="error-message">{{ compressError }}</div>
```

- [ ] **Step 3: 添加 script 逻辑**

在 `<script setup lang="ts">` 的 import 区域添加：
```typescript
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Check } from '@element-plus/icons-vue'
```

在 `const activeTab = ref('pdfToImages')` 之后添加压缩相关状态：

```typescript
// ============ Tab 6: PDF压缩 ============
const compressInputRef = ref<HTMLInputElement | null>(null)
const compressFiles = ref<File[]>([])
const compressLevel = ref(2)
const isCompressing = ref(false)
const isDragOver = ref(false)
const compressError = ref('')
const gsAvailable = ref(false)

interface CompressResultItem {
  fileName: string
  originalSize: number
  compressedSize: number
  ratio: number
  outputPath: string
}

const compressResults = ref<CompressResultItem[]>([])

const compressLevelHint = computed(() => {
  switch (compressLevel.value) {
    case 1: return '快速压缩：图片150DPI/85%质量，清除XMP元数据，速度最快'
    case 2: return '标准压缩：图片150DPI/70%质量，清除全部元数据，平衡体积与质量'
    case 3: return '极限压缩：图片72DPI/50%质量，清除全部元数据，最小体积（有Ghostscript效果更佳）'
    default: return ''
  }
})

const totalOriginalSize = computed(() =>
  compressResults.value.reduce((sum, r) => sum + r.originalSize, 0)
)

const totalCompressedSize = computed(() =>
  compressResults.value.reduce((sum, r) => sum + r.compressedSize, 0)
)

const totalRatio = computed(() => {
  const orig = totalOriginalSize.value
  const comp = totalCompressedSize.value
  if (orig === 0) return 0
  return Math.round((1 - comp / orig) * 100)
})

const triggerCompressInput = () => compressInputRef.value?.click()

const handleCompressFileSelect = (e: Event) => {
  const input = e.target as HTMLInputElement
  const files = input.files
  if (!files) return
  addCompressFiles(Array.from(files))
  input.value = ''
}

const handleCompressDrop = (e: DragEvent) => {
  isDragOver.value = false
  const files = e.dataTransfer?.files
  if (!files) return
  addCompressFiles(Array.from(files))
}

const addCompressFiles = (files: File[]) => {
  compressError.value = ''
  const pdfFiles = files.filter(f => f.type === 'application/pdf' || f.name.endsWith('.pdf'))
  if (pdfFiles.length === 0) {
    compressError.value = '请选择 PDF 文件'
    return
  }
  const maxSize = 100 * 1024 * 1024
  const oversize = pdfFiles.find(f => f.size > maxSize)
  if (oversize) {
    compressError.value = `文件 "${oversize.name}" 超过 100MB 限制`
    return
  }
  compressFiles.value = [...compressFiles.value, ...pdfFiles]
  compressResults.value = []
}

const handleRemoveCompressFile = (idx: number) => {
  compressFiles.value.splice(idx, 1)
  compressResults.value = []
}

const handleClearCompressFiles = () => {
  compressFiles.value = []
  compressResults.value = []
  compressError.value = ''
  if (compressInputRef.value) compressInputRef.value.value = ''
}

const handleCompress = async () => {
  if (!compressFiles.value.length) return
  compressError.value = ''
  compressResults.value = []
  isCompressing.value = true

  try {
    for (let i = 0; i < compressFiles.value.length; i++) {
      const file = compressFiles.value[i]
      // 读取文件为 base64 传给后端
      const buffer = await file.arrayBuffer()
      const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)))
      // 先通过 save_temp_file 保存到临时路径（需要新增命令）
      const tempPath: string = await invoke('save_temp_file', { data: base64, filename: file.name })

      const result: { output_path: string; original_size: number; compressed_size: number } =
        await invoke('compress_pdf', {
          filePath: tempPath,
          level: compressLevel.value,
          gsAvailable: gsAvailable.value,
        })

      const originalSize = result.original_size
      const compressedSize = result.compressed_size
      const ratio = originalSize > 0
        ? Math.round((1 - compressedSize / originalSize) * 100)
        : 0

      compressResults.value.push({
        fileName: file.name,
        originalSize,
        compressedSize,
        ratio,
        outputPath: result.output_path,
      })
    }
    ElMessage.success(`压缩完成，共 ${compressResults.value.length} 个文件`)
    store.addHistory({
      tool: 'pdf',
      action: `PDF压缩 (${compressLevel.value === 1 ? '快速' : compressLevel.value === 2 ? '标准' : '极限'})`,
      inputPreview: `${compressFiles.value.length} 个文件`,
      outputPreview: `缩小 ${totalRatio.value}%`,
      inputFull: compressFiles.value.map(f => f.name).join('\n'),
      outputFull: `${formatFileSize(totalOriginalSize.value)} → ${formatFileSize(totalCompressedSize.value)}，缩小 ${totalRatio.value}%`,
    })
  } catch (e: any) {
    compressError.value = typeof e === 'string' ? e : (e.message || '压缩失败')
  } finally {
    isCompressing.value = false
  }
}

const handleSaveSingleCompressed = async (idx: number) => {
  const result = compressResults.value[idx]
  if (!result) return
  // 读取临时文件为 base64，通过对话框保存
  try {
    const base64: string = await invoke('read_file_base64', { path: result.outputPath })
    const originalName = result.fileName
    const baseName = originalName.replace(/\.pdf$/i, '')
    await invoke('save_file_with_dialog', {
      dataBase64: base64,
      filename: `${baseName}_compressed.pdf`,
      defaultExt: 'pdf',
    })
    ElMessage.success('已保存')
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : '保存失败')
  }
}

const handleSaveAllCompressed = async () => {
  for (let i = 0; i < compressResults.value.length; i++) {
    await handleSaveSingleCompressed(i)
  }
}

// 检测 Ghostscript
onMounted(async () => {
  try {
    gsAvailable.value = await invoke('detect_ghostscript')
  } catch {
    gsAvailable.value = false
  }
})
```

- [ ] **Step 4: 添加 scoped 样式**

在 `<style scoped>` 末尾（`</style>` 之前）添加：

```css
/* ===== PDF压缩 ===== */
.compress-drop-zone {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  padding: 16px;
  transition: border-color 0.3s, background-color 0.3s;
  min-height: 60px;
  display: flex;
  align-items: center;
}

.compress-drop-zone.drag-over {
  border-color: var(--accent-cyan);
  background: rgba(0, 212, 255, 0.05);
}

.compress-file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  max-height: 300px;
  overflow-y: auto;
}

.compress-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
}

.file-index {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-cyan);
  color: var(--bg-card);
  border-radius: 50%;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.compress-level-hint {
  margin-top: 12px;
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  line-height: 1.5;
}

.gs-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--accent-green);
  padding: 6px 12px;
  background: rgba(16, 185, 129, 0.08);
  border-radius: 4px;
}

.compress-table {
  width: 100%;
}

.ratio-positive {
  color: var(--accent-green);
  font-weight: 600;
}

.ratio-negative {
  color: var(--accent-orange);
  font-weight: 600;
}

.compress-summary {
  margin-top: 12px;
  padding: 8px 12px;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-secondary);
  text-align: right;
}
```

- [ ] **Step 5: 运行 npm run build 验证**

Run: `npm run build`
Expected: 无编译错误

- [ ] **Step 6: Commit**

```bash
git add src/views/PdfTool.vue
git commit -m "feat: PdfTool 新增 PDF压缩 Tab（批量压缩 + 3档预设）"
```

---

### Task 5: 新增 save_temp_file 命令（文件传输用）

**Files:**
- Modify: `src-tauri/src/pdf_tools.rs`

- [ ] **Step 1: 添加 save_temp_file 命令**

在 `pdf_tools.rs` 文件末尾添加：

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[tauri::command]
pub fn save_temp_file(data: String, filename: String) -> Result<String, String> {
    let bytes = STANDARD
        .decode(&data)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(&filename);
    fs::write(&temp_path, &bytes)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;
    Ok(temp_path.to_string_lossy().to_string())
}
```

- [ ] **Step 2: 在 main.rs 注册 save_temp_file**

在 `pdf_tools::get_pdf_page_count,` 之后添加：
```rust
            pdf_tools::save_temp_file,
```

- [ ] **Step 3: 运行 cargo check 验证**

Run: `cd src-tauri; cargo check`
Expected: 无编译错误

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/pdf_tools.rs src-tauri/src/main.rs
git commit -m "feat: 新增 save_temp_file 命令用于前端上传文件到后端"
```

---

### Task 6: 更新 TOOL_LIST 和版本号

**Files:**
- Modify: `src/store/index.ts`
- Modify: `package.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`

- [ ] **Step 1: 更新 TOOL_LIST 中 PDF 工具描述**

在 `src/store/index.ts` 第 68 行，将 PDF 工具的 description 和 keywords 更新：
```typescript
// 将:
description: 'PDF转图片、图片转PDF、文本提取、合并拆分',
// 改为:
description: 'PDF转图片、图片转PDF、文本提取、合并拆分、压缩',
// 将 keywords 中的:
keywords: ['pdf', '转换', '合并', '拆分', '提取'],
// 改为:
keywords: ['pdf', '转换', '合并', '拆分', '提取', '压缩'],
```

- [ ] **Step 2: 更新版本号到 V5.1.0**

`package.json`:
```json
"version": "5.1.0"
```

`src-tauri/Cargo.toml`:
```toml
version = "5.1.0"
```

`src-tauri/tauri.conf.json`:
```json
"version": "5.1.0"
```

- [ ] **Step 3: 更新 README.md**

在功能阶段记录中添加：
```markdown
| V5.1 | ✅ | PDF 压缩（3档预设 + 批量处理 + Ghostscript增强） | 2026-07-14 |
```

- [ ] **Step 4: 运行 npm run build 验证**

Run: `npm run build`
Expected: 无编译错误

- [ ] **Step 5: Commit**

```bash
git add src/store/index.ts package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json README.md
git commit -m "chore: 版本号 V5.1.0，更新 PDF 工具描述和 README"
```

---

### Task 7: 端到端验证

- [ ] **Step 1: 启动 Tauri 开发模式**

Run: `npm run tauri dev`
Expected: 应用正常启动

- [ ] **Step 2: 手动验证**

1. 打开 PDF 工具 → 切换到 "PDF压缩" Tab
2. 添加一个图片密集的 PDF 文件
3. 选择"标准压缩"，点击"开始压缩"
4. 验证：显示压缩结果（原始大小、压缩后大小、压缩率）
5. 点击"保存"，验证文件正常保存
6. 用 PDF 阅读器打开压缩后的文件，验证内容完整
7. 添加多个 PDF 文件，验证批量压缩
8. 尝试加密 PDF，验证错误提示
9. 切换 3 档压缩级别，验证体积差异

- [ ] **Step 3: 验证完成，关闭 Tauri**

---

## 自审

- Spec 覆盖：所有章节（架构、压缩级别、UI、Rust 后端、数据流、错误处理）均有对应 Task
- 无占位符：所有步骤均包含具体代码
- 类型一致性：`CompressResult` 结构体在前端接口和 Rust 后端定义一致，字段名使用 snake_case（遵循项目规范）