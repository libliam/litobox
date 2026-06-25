# LitoBox - 栗的百宝箱

<p align="center">
  <strong>零成本 AI 驱动开发的 Windows 桌面工具箱</strong>
</p>

<p align="center">
  <a href="#-功能特性">功能</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-开发故事">开发故事</a> •
  <a href="#%EF%B8%8F-扩展开发">扩展开发</a> •
  <a href="#-技术栈">技术栈</a>
</p>

---

## 🖥️ 项目演示

<p align="center">
  <img src="docs/screenshots/overview.png" alt="LitoBox 主界面" width="800"/>
</p>

> **提示**: 项目演示截图待补充，请将截图放置于 `docs/screenshots/` 目录

---

## ✨ 核心特性

- 🚀 **零网络依赖** — 纯离线运行，无需联网
- 🎨 **深色/浅色主题** — 自动跟随系统，科技风 UI
- 📦 **便携免安装** — 单 exe 文件，双击即用
- 💾 **数据本地化** — 不上传任何数据，无广告无追踪
- ⚡ **极速响应** — 启动 ≤ 1s，操作 ≤ 100ms
- 🧠 **AI 驱动开发** — 全程零成本，免费模型 + 开源工具链

---

## 🧰 功能特性

### 开发工具

| 工具 | 功能 |
|------|------|
| **JSON工具箱** | 格式化/压缩/校验/JSON5 兼容解析（支持注释、尾逗号） |
| **SQL工具箱** | 格式化/压缩/校验/JSON转Insert/字段格式互转/MyBatis日志解析 |
| **JS工具箱** | 在线 JavaScript 代码执行器 |
| **正则测试** | 匹配/替换/高亮/多标志支持 |
| **文本对比** | 行级/字符级差异对比，忽略空白/大小写 |
| **进制转换** | BIN/OCT/DEC/HEX 互转 |

### 文本处理

| 工具 | 功能 |
|------|------|
| **字符串工具** | 去空格/大小写转换/文本清理/拼接分割 |
| **编码工具** | URL编解码/Base64/时间戳转换 |
| **批量文本** | 批量去重/批量编码转换/文件编码检测 |
| **Word计数** | 字数/行数/词数统计 |
| **Markdown** | Markdown 预览渲染 |

### 安全 & 生成

| 工具 | 功能 |
|------|------|
| **加密工具** | AES/RSA/DES 加密解密 |
| **哈希工具** | MD5/SHA1/SHA256/SHA512 |
| **JWT解析** | Token 解码，查看 Header/Payload |
| **密码生成** | 随机密码/密码强度检测 |
| **UUID生成** | 批量生成/多格式输出 |
| **Mock数据** | 自动生成模拟数据 |
| **QR码** | 二维码生成/解析 |

### 文件 & 系统

| 工具 | 功能 |
|------|------|
| **OCR识别** | 图片文字提取（支持 PP-OCRv6） |
| **批量OCR** | 批量图片文字识别 |
| **表格OCR** | 图片表格提取 |
| **PDF工具** | PDF 转图片/合并/提取文字 |
| **图片工具** | 格式转换/压缩/裁剪 |
| **文件编码** | 批量转换文件编码（UTF-8/GBK 等） |
| **HTTP工具** | 简易 HTTP 请求调试 |

### 其他工具

| 工具 | 功能 |
|------|------|
| **Cron表达式** | Cron 表达式解析/可视化/字段面板 |
| **颜色工具** | HEX/RGB/HSL 互转/取色器 |
| **CSS工具** | CSS 压缩/格式化 |
| **CSV工具** | CSV 解析/转换 |
| **XML/YAML** | 格式互转/格式化 |
| **URL工具** | URL 解析/编码 |
| **时间工具** | 时间戳转换/日期计算 |
| **代码片段** | 常用代码片段管理 |
| **历史记录** | 最近操作记录回溯 |
| **剪贴板工具** | 图片识别/内容分析 |
| **十六进制查看** | 二进制数据可视化 |
| **Dev工具** | 开发辅助工具集合 |

---

## 🚀 快速开始

### 一键运行

```bash
# 下载便携版，直接双击 litobox.exe 运行
# 无需安装，无需网络，开箱即用
```

### 开发环境

#### 1. 预装工具（开发前必须安装）

| 工具 | 说明 | 安装方式 |
|------|------|----------|
| **Node.js** | >= 16，前端构建环境 | [官网下载](https://nodejs.org/) |
| **Visual Studio Build Tools** | C++ 编译工具链（Tauri/Rust 编译依赖） | 安装时勾选 **"使用 C++ 的桌面开发"** 工作负载 |
| **WebView2** | Windows 11 自带；Windows 10 需[手动安装](https://developer.microsoft.com/microsoft-edge/webview2/) | 系统自带或官网下载 |
| **Git** | 版本控制 | [官网下载](https://git-scm.com/) |

> **重要**: Visual Studio Build Tools 是 Tauri 项目编译的**必须依赖**，缺少会导致 Rust 编译失败。不需要安装完整版 Visual Studio，只需 [Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 即可。

#### 2. 安装 Rust 工具链

```bash
# 下载并运行 rustup 安装器
# https://rustup.rs/ 或执行项目中的 rustup-init.exe

# 验证安装
rustc --version
cargo --version
```

#### 3. 克隆 & 运行

```bash
# 克隆仓库
git clone https://github.com/libliam/litobox.git
cd litobox

# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 完整环境清单

| 依赖 | 版本 | 说明 |
|------|------|------|
| Windows | >= 10 | 运行系统 |
| Node.js | >= 16 | 前端构建 |
| Rust | >= 1.75 | Tauri 底层编译 |
| VS Build Tools | 2019/2022 | C++ 编译工具链 |
| WebView2 | 最新版 | Web 视图渲染引擎 |
| Git | 任意版本 | 版本控制 |

---

## 🧠 Vibe Coding 开发实践

> **本项目是 Vibe Coding 的完整实践案例** — 从需求到上线，全程零手写代码，通过 AI 对话驱动开发。

### 什么是 Vibe Coding？

Vibe Coding 是一种全新的开发范式：**你描述意图，AI 完成编码**。开发者不再纠结于语法细节，而是专注于"要做什么"，让 AI 处理"怎么做"。

本项目证明了：一个完整的桌面应用，从 0 到 1，可以**完全通过对话完成**。

---

### 🔧 开发工具链

| 工具 | 作用 | 费用 |
|------|------|------|
| **[Trae](https://www.trae.ai/)** (CN 版) | AI 原生 IDE，内置 Agent 技能系统 | 免费 |
| **[Qwen](https://qianwen.aliyun.com/)** (通义千问) | 底层 AI 模型，代码生成 & 逻辑推理 | 免费额度 |
| **豆包** | 初版需求文档分析 | 免费 |
| **GitHub** | 版本控制 & 代码托管 | 免费 |

**全程零花费**：每天利用免费额度迭代，额度用完就第二天继续，或多账号交替使用。

---

### 📋 Vibe Coding 开发流程

```
你描述想法 → AI 引导提问 → 确认方案 → 生成设计文档 → 生成实施计划 → AI 自动写代码 → 运行验证
```

#### 第一步：需求确认（brainstorming）

在 Trae 中调用 `brainstorming` 技能，用自然语言描述你想加的功能：

> "我想加一个 SQL 格式化工具，能美化 SQL 语句，支持压缩和校验"

AI 会：
- 引导你思考功能边界（支持哪些 SQL 方言？要不要语法树解析？）
- 提供多种技术方案供你选择
- 最终输出**设计文档**到 `docs/superpowers/specs/`

#### 第二步：计划拆解（writing-plans）

确认方案后，调用 `writing-plans` 技能：
- AI 将设计文档拆分为**可执行的实施计划**
- 每个任务有明确的输入/输出/验收标准
- 计划文档输出到 `docs/superpowers/plans/`

#### 第三步：自动开发（subagent-driven-development）

计划确认后，调用 `subagent-driven-development` 技能：
- AI 按任务分配子代理并行执行
- 每个任务：brief → implement → report 闭环
- 任务间自动审查，快速迭代
- **大多数新功能，一步到位直接搞定**

---

### 🛡️ 经验固化机制（AGENTS.md）

Vibe Coding 不是一次性的对话，而是**持续进化的工程实践**：

| 阶段 | 做法 |
|------|------|
| **踩坑** | 开发中遇到编译错误、样式问题、性能瓶颈 |
| **记录** | 将解决方案写入 `AGENTS.md` |
| **复用** | 下次同类问题，AI 自动参考历史记录 |
| **模板化** | 通用页面结构沉淀为 `_ToolTemplate.vue` |

结果：**越早开发的功能踩的坑越多，越到后面开发越快**，因为经验已经固化到项目记忆中。

---

### 🎯 实际开发成果

| 指标 | 数据 |
|------|------|
| 开发工具 | 100% 免费 |
| 代码手写比例 | ≈ 0%（全部 AI 生成） |
| 开发周期 | 持续迭代，每天推进 |
| 工具数量 | 30+ 个功能工具 |
| 技术栈 | Vue 3 + Tauri 2.0 + TypeScript |

---

### 📚 如何参与或学习

如果你想**复现这种开发模式**：

1. **下载 [Trae CN 版](https://www.trae.ai/)** — 免费 AI IDE
2. **配置免费模型** — 选择通义千问或其他免费模型
3. **克隆本项目** — 参考 `AGENTS.md` 中的开发规范
4. **从模板开始** — 复制 `_ToolTemplate.vue`，调用 brainstorming 技能
5. **开始你的第一个 Vibe Coding 项目**

如果你想**为本项目添加新工具**，只需：
1. 在 Trae 中打开项目
2. 告诉 AI："我想加一个 XXX 工具"
3. 跟随引导完成设计 → 计划 → 开发 → 提交 PR

---

## 🛠️ 扩展开发

### 新增工具页面（3 步搞定）

1. **复制模板**：`src/views/_ToolTemplate.vue` → `src/views/YourTool.vue`
2. **修改逻辑**：替换命名，按需添加功能
3. **注册导航**：在 `src/store/index.ts` 添加工具配置

### 代码规范

- TypeScript 严格模式
- Vue 3 Composition API + `<script setup>`
- 组件命名：`PascalCase`
- 工具函数：`camelCase`
- 所有颜色使用 CSS 变量，禁止硬编码

### 关键文件

| 文件 | 作用 |
|------|------|
| `AGENTS.md` | AI Agent 行为规范 + 项目经验沉淀 |
| `_ToolTemplate.vue` | 新工具页面模板（直接复制使用） |
| `theme.css` | 科技风主题变量（深色/浅色） |
| `src/store/index.ts` | 工具注册中心 |

---

## � 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 | Vue 3 + TypeScript + Vite | Composition API，类型安全 |
| UI | Element Plus | 桌面端组件库 |
| 桌面 | Tauri 2.0 | Rust 底层，系统级调用 |
| 状态 | Pinia | 轻量状态管理 |
| 存储 | localStorage | 配置与历史缓存 |
| OCR | PaddleOCR.js | PP-OCRv6 离线识别 |
| PDF | pdf-lib + pdfjs-dist | PDF 处理与渲染 |
| 加密 | crypto-js | AES/RSA/DES 等算法 |

---

## 📊 性能指标

| 指标 | 值 |
|------|-----|
| 启动时间 | ≤ 1s |
| 操作响应 | ≤ 100ms |
| 空闲内存 | ≤ 50MB |
| 大文本处理 | 10w 字符无卡顿 |
| 便携版体积 | ≤ 50MB |

---

## 🔒 安全承诺

- ✅ 纯本地离线运行，无网络请求
- ✅ 仅保留必要权限：剪贴板/窗口控制/热键/存储
- ✅ 所有数据本地存储，不上传
- ✅ 无广告、无后台、无数据采集

---

## �️ 版本路线

| 版本 | 状态 | 内容 |
|------|------|------|
| V1.0 | ✅ | JSON/字符串/编码/正则/进制/UUID |
| V1.1 | ✅ | 操作历史/窗口置顶/主题切换 |
| V1.2 | ✅ | SQL/JS执行器/Mock/OCR/文件编码 |
| V1.3 | ✅ | 文本对比工具 |
| V2.x | 🔄 | 更多工具持续加入... |

---

## 🤝 参与贡献

欢迎 PR！如果你想添加工具或改进功能：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

---

## 📝 License

[MIT License](LICENSE)
