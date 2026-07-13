# PDF 压缩工具 - 设计文档

**日期**: 2026-07-14
**版本**: V5.1
**类型**: 新增功能（PdfTool.vue 新增 Tab）

---

## 1. 背景

LitoBox 已有 PDF 工具（转图片/文本提取/转 Markdown/合并拆分），但缺少 PDF 压缩能力。用户需要缩小 PDF 文件体积（如提交表单、邮件附件限制），无需安装 Adobe Acrobat 等重型软件。

**目标**：纯 Rust 实现 PDF 压缩，支持 3 档预设级别 + 批量处理，Ghostscript 作为可选增强引擎。

---

## 2. 方案概述

**核心思路**：纯 Rust 后端（lopdf + image crate）+ 前端批量文件管理 UI。

**技术栈**：
- PDF 解析/修改：`lopdf`（纯 Rust，新增依赖）
- 图片重采样：`image` crate（已有，v0.25）
- 流压缩：`flate2`（lopdf 已有依赖）
- Ghostscript 增强：运行时探测 `gswin64c.exe`，有则用于极限压缩档位

**体积增量**：~1-2 MB（lopdf）

---

## 3. 功能范围

### 3.1 包含

- 3 档预设压缩级别（快速/标准/极限）
- 批量处理：一次拖入多个 PDF，逐个压缩
- 压缩前后大小对比，显示压缩率
- 元数据清理（XMP/注释/文档信息）
- 图像重采样（降低 DPI 和 JPEG 质量）
- 内容流压缩（flate2）
- Ghostscript 自动探测，有则用于极限压缩增强
- 加密 PDF 检测，给出明确提示
- 操作记录到历史（store.addHistory）

### 3.2 不包含

- 自定义 DPI/质量参数（保持预设档位简洁）
- PDF 加密/解密（已有 F7 规划，不在此范围）
- 压缩后自动覆盖原文件（统一通过保存对话框手动保存）

---

## 4. 架构设计

```
PdfTool.vue（新增 "PDF压缩" Tab，name: compress）
  │
  ├─ 前端：文件拖入/选择、压缩档位选择、批量进度、结果展示
  │
  └─ Tauri invoke ──► Rust 后端
                         │
                         ├─ pdf_tools.rs (新增)
                         │   ├─ compress_pdf()          ← 核心压缩
                         │   │   ├─ 图像重采样           ← image crate
                         │   │   ├─ 流压缩               ← flate2
                         │   │   └─ 元数据清理           ← lopdf
                         │   ├─ detect_ghostscript()     ← 探测 gs
                         │   └─ get_pdf_page_count()     ← 获取页数
                         │
                         └─ Cargo.toml 新增
                             └─ lopdf = "0.34"
```

**关键决策**：
- 压缩操作在 `tokio::task::spawn_blocking` 中执行，避免阻塞 UI
- Ghostscript 探测结果前端缓存，首次调用后不再重复探测
- 批量处理在前端循环调用 `compress_pdf`，每个文件独立压缩

---

## 5. 压缩级别设计

| 档位 | 标签 | 图片 DPI | JPEG 质量 | 元数据清理 | 流压缩 | Ghostscript |
|------|------|----------|-----------|-----------|--------|-------------|
| 1 | 快速压缩 | 150 | 85% | 仅 XMP/注释 | 是 | 否 |
| 2 | 标准压缩 | 150 | 70% | 全部清除 | 是 | 否 |
| 3 | 极限压缩 | 72 | 50% | 全部清除 | 是 | 有则用，无则纯 Rust |

**规则**：
- 原始 DPI < 目标 DPI 时**不放大**，保留原图
- 每档压缩后显示：原始大小 → 压缩后大小 → 压缩率
- 极限压缩时，Ghostscript 结果与纯 Rust 结果比较，取更小的

---

## 6. 前端 UI 设计

### 6.1 布局（3 张卡片 + 结果区）

**文件选择卡片**：
- 上传按钮 + 拖入区域
- 已选文件列表（文件名 + 大小），支持逐个移除
- 单文件最大 100MB

**压缩设置卡片**：
- 3 档 Radio 选择，切换时显示该档位说明
- "开始压缩" 按钮（含 loading 状态）
- 进度提示：当前处理文件名

**压缩结果卡片**：
- 表格：文件名、原始大小、压缩后大小、压缩率、保存按钮
- 底部合计行：总原始大小 → 总压缩后大小，总压缩率
- "全部保存" 按钮

### 6.2 交互要点

- 文件列表支持拖入添加
- 压缩中按钮 loading，进度条显示当前文件
- 结果按文件独立展示，支持单独保存
- 遵循项目样式规范：CSS 变量、`.tool-card`、`.card-header` 等全局类名

### 6.3 Tab 定义

```typescript
// PdfTool.vue 新增 Tab
const tabs = [
  // ... 现有 5 个 Tab
  { name: 'compress', label: 'PDF压缩' }
]
```

---

## 7. Rust 后端设计

### 7.1 新增文件

`src-tauri/src/pdf_tools.rs`

### 7.2 Tauri 命令

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `compress_pdf` | `file_path: String, level: u8, gs_available: bool` | `{ output_path, original_size, compressed_size }` | 核心压缩，spawn_blocking |
| `detect_ghostscript` | 无 | `bool` | 探测 gswin64c.exe / gswin32c.exe |
| `get_pdf_page_count` | `file_path: String` | `u32` | 获取 PDF 页数 |

### 7.3 compress_pdf 内部流程

```
1. lopdf::Document::load(input_path)
2. 遍历所有页面 → 提取图片 XObject
3. 对每个图片：
   ├─ 解码（JPEG/PNG/...）
   ├─ 按 level 指定的 DPI/质量重采样
   └─ 重新编码后替换原 XObject 流
4. 元数据清理：
   ├─ level >= 1: 移除 XMP 元数据、注释
   └─ level >= 2: 移除文档信息（作者/标题/创建者）
5. 流压缩：lopdf 自动用 flate2 压缩内容流
6. 如果 level == 3 && gs_available:
   ├─ 先保存纯 Rust 压缩结果
   ├─ 调用 Ghostscript 压缩
   └─ 比较两个结果，返回更小的
7. 保存到临时输出文件，返回路径和大小
```

### 7.4 Ghostscript 探测

- 用 `std::process::Command` 在 PATH 中搜索 `gswin64c.exe` / `gswin32c.exe`
- 前端启动时调用 `detect_ghostscript`，结果缓存
- Ghostscript 调用命令：`gswin64c.exe -sDEVICE=pdfwrite -dPDFSETTINGS=/screen -dNOPAUSE -dQUIET -dBATCH -sOutputFile=output.pdf input.pdf`
- 子进程加 `CREATE_NO_WINDOW` 标志

### 7.5 错误处理

| 场景 | 处理 |
|------|------|
| 加密 PDF | 返回 `Err("不支持加密 PDF")` |
| 损坏 PDF | lopdf 加载失败，返回解析错误 |
| 图片解码失败 | 跳过该图片，记录警告日志 |
| Ghostscript 调用失败 | 降级到纯 Rust 结果 |
| 压缩后体积变大 | 仍返回压缩结果，前端显示负压缩率（如 +5%） |

---

## 8. 数据流

```
用户选择文件 → 前端维护文件列表
     │
     ▼
点击"开始压缩" → 前端循环调用 compress_pdf
     │
     ▼
compress_pdf(file_path, level, gs_available)
     │
     ├─ 加载 PDF → 提取图片 → 重采样 → 替换流
     ├─ 清理元数据 → 流压缩 → 保存
     └─ 返回 { output_path, original_size, compressed_size }
     │
     ▼
前端汇总结果 → 显示表格 → 用户逐个/批量保存
     │
     ▼
保存：调用 save_file_with_dialog → 复制临时文件到目标路径
```

---

## 9. 测试策略

### 9.1 手动测试场景

- 文本为主的 PDF（压缩效果有限）
- 图片密集的 PDF（压缩效果明显）
- 混合内容 PDF
- 加密 PDF（验证错误提示）
- 大文件（>50MB，验证 spawn_blocking 不卡 UI）
- 批量 5+ 文件同时压缩
- 无 Ghostscript 环境（验证纯 Rust 降级）
- 有 Ghostscript 环境（验证极限压缩增强）

### 9.2 验收标准

- 压缩后 PDF 可用常用阅读器正常打开
- 页面顺序和内容完整性不变
- 3 档之间有明显的体积差异
- 批量处理不卡 UI
- 加密 PDF 给出明确错误提示

---

## 10. 版本规划

- 版本号：V5.1（新增工具 Tab，按 semver 规则为 minor 版本升级）
- 更新 `TOOL_LIST` 中 PdfTool 的描述，新增"压缩"关键词
- README 同步更新功能记录