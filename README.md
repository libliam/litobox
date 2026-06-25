# LitoBox - 栗的百宝箱

轻量化、无网络、无广告、常驻本地的 Windows 桌面集成工具箱，统一收纳高频开发小工具。

## 🎯 功能特性

### 核心工具
- **JSON工具箱** - 格式化/压缩/校验/兼容解析（支持注释、尾逗号）
- **字符串工具箱** - 空格处理/拼接分割/大小写转换/文本清理
- **基础编码工具** - URL编解码/Base64/时间戳转换
- **正则测试工具** - 匹配/替换/高亮/多标志支持
- **进制转换工具** - BIN/OCT/DEC/HEX互转
- **UUID生成工具** - 批量生成/格式选项

### 进阶工具
- **SQL工具箱** - 格式化美化/压缩单行化/语法校验/JSON转Insert/字段格式互转/注释批量操作/SQL IN转换/快捷模板/MyBatis日志解析
- **JS工具箱** - 在线JavaScript代码执行器
- **Mock数据生成** - 自动生成模拟数据
- **OCR文字识别** - 图片文字提取
- **文件编码转换** - 批量转换文件编码格式
- **文本对比** - 行级/字符级差异对比，支持自动执行、忽略空白/大小写

### 系统能力
- 多标签页并行使用
- 深色/浅色主题适配
- 系统托盘常驻
- 全局热键唤起
- 操作历史记录（最近10条）
- 窗口置顶功能

---

## 🛠️ 开发指南

### 开发技能（Skills）

本项目在开发过程中使用了以下关键技能来提升开发效率和质量：

| Skill | 用途 | 使用场景 |
|-------|------|----------|
| **superpowers:brainstorming** | 需求探索 | 与用户对话确认功能细节，输出设计文档到 specs 目录 |
| **superpowers:subagent-driven-development** | 子代理驱动开发 | 每个任务独立子代理执行，任务间审查，快速迭代 |
| **superpowers:executing-plans** | 计划执行 | 在当前会话批量执行带检查点的任务 |
| **superpowers:writing-plans** | 计划编写 | 编写详细的实施计划文档，指导开发流程 |
| **superpowers:frontend-design** | 前端设计规范 | 确保UI设计遵循科技风规范，卡片式布局 |
| **superpowers:fullstack-developer** | 全栈开发 | Vue 3 + Tauri 2.0 全栈实现 |
| **superpowers:requirements-analyst** | 需求分析 | PRD文档编写，需求拆解 |
| **superpowers:systematic-debugging** | 系统调试 | 定位和修复复杂问题 |

### 开发流程与技能配合

#### 标准开发流程（三段式协作）

本项目采用 **需求确认 → 计划拆解 → 并行执行** 的三段式技能协作流程：

1. **需求确认** → 使用 `brainstorming` 与用户对话，确认功能细节和边界条件
   - 多轮对话明确需求范围
   - 设计文档输出到 `docs/superpowers/specs/`

2. **计划拆解** → 使用 `writing-plans` 将设计文档转化为可执行的实施计划
   - 拆分为多个独立任务
   - 明确每个任务的输入/输出/验收标准
   - 计划文档输出到 `docs/superpowers/plans/`

3. **并行执行** → 使用 `subagent-driven-development` 按任务分配子代理执行
   - 每个任务独立子代理执行（brief → implement → report 闭环）
   - 任务间自动审查，快速迭代
   - 每个任务完成后自动推进下一个

#### V1.0 基础版本开发流程

1. **需求分析** → 使用 `requirements-analyst` 编写 PRD 和技术设计文档
2. **计划制定** → 使用 `writing-plans` 在 `docs/superpowers/plans/` 创建详细实施计划
3. **任务执行** → 使用 `subagent-driven-development` 按任务分解逐步实现
   - 项目初始化与基础配置
   - 工具函数层实现（TDD）
   - 公共组件实现
   - 状态管理与样式
   - 功能页面实现
   - Tauri配置与Rust底层
4. **验证测试** → 手动测试验证所有功能

#### V1.2 进阶版本开发流程

1. **复杂度排序** → 按复杂度递增顺序实现：窗口置顶 → 自定义快捷键 → 批量文本处理 → 文件编码转换
2. **前端实现** → 使用 `frontend-design` 确保UI一致性
3. **Rust集成** → 使用 `fullstack-developer` 实现跨语言调用
4. **调试优化** → 使用 `systematic-debugging` 解决集成问题

### 扩展开发建议

当需要新增功能时，建议遵循以下步骤：

1. **需求澄清** → 使用 `requirements-analyst` 明确需求范围
2. **设计规划** → 在 `docs/superpowers/plans/` 创建实施计划
3. **代码实现** → 使用 `subagent-driven-development` 按任务执行
4. **UI规范** → 遵循 `frontend-design` 的科技风设计规范
5. **测试验证** → 功能验证后提交

### 技术栈

| 层级 | 技术 | 版本 | 说明 |
|------|------|------|------|
| 前端框架 | Vue 3 | ^3.4.0 | Composition API |
| 构建工具 | Vite | ^5.1.0 | 极速构建与热更新 |
| 类型系统 | TypeScript | ^5.3.0 | 统一代码规范 |
| UI组件库 | Element Plus | ^2.5.0 | 简洁桌面端组件 |
| 桌面内核 | Tauri | ^2.0.0 | Rust底层，原生系统调用 |
| 状态管理 | Pinia | ^2.1.0 | 轻量状态管理 |
| JSON处理 | json5 + prettier | ^2.2.3 | 非标准JSON兼容解析 |
| 字符串处理 | lodash | ^4.17.21 | 高效批量处理 |
| 编解码 | js-base64 | ^3.7.6 | Base64编解码 |

### 项目结构

```plain
litobox/
├── src/                    # 前端Vue源码
│   ├── components/         # 公共组件
│   │   ├── SidebarNav.vue  # 侧边导航
│   │   ├── ToolActions.vue # 操作按钮组件
│   │   ├── ToolInput.vue   # 通用输入框组件
│   │   └── ToolOutput.vue  # 结果展示组件
│   ├── views/              # 功能页面
│   │   ├── JsonTool.vue    # JSON工具箱
│   │   ├── StringTool.vue  # 字符串工具箱
│   │   ├── SqlTool.vue     # SQL工具箱
│   │   ├── JSTool.vue      # JS执行器
│   │   └── ...             # 其他工具页面
│   ├── utils/              # 核心工具方法
│   │   ├── jsonUtils.ts    # JSON格式化/压缩/校验
│   │   ├── sqlUtils.ts     # SQL处理工具
│   │   └── ...             # 其他工具函数
│   ├── store/              # Pinia状态管理
│   ├── style/              # 样式文件
│   ├── App.vue             # 根页面
│   └── main.ts             # 入口文件
├── src-tauri/              # Tauri Rust底层
│   ├── src/
│   │   ├── main.rs         # Rust入口
│   │   ├── js_executor.rs  # JS执行引擎封装
│   │   └── file_encoding.rs # 文件编码转换
│   ├── Cargo.toml          # Rust依赖
│   └── tauri.conf.json     # Tauri配置
└── package.json            # 前端依赖
```

### 开发流程

#### 1. 环境准备

```bash
# 安装 Node.js >= 16
# 安装 Rust (通过 rustup-init.exe)
.\rustup-init.exe -y

# 配置 Rust 环境
source $HOME/.cargo/env
```

#### 2. 依赖安装

```bash
# 安装前端依赖
npm install

# 安装 Tauri CLI（如果未安装）
npm install -D @tauri-apps/cli
```

#### 3. 开发调试

```bash
# 启动开发服务器（前端热更新）
npm run dev

# 启动 Tauri 开发模式（需要另开终端）
npm run tauri dev
```

#### 4. 代码规范

- 使用 TypeScript 严格模式
- Vue 3 Composition API + `<script setup>` 语法
- 组件命名使用 PascalCase
- 工具函数使用 camelCase
- 常量使用 UPPER_SNAKE_CASE

#### 5. 新增功能页面流程

1. 在 `src/views/` 创建新的 Vue 组件
2. 在 `src/utils/` 添加对应的工具函数
3. 在 `src/components/SidebarNav.vue` 注册导航
4. 在 `src/App.vue` 中添加路由/视图引用

#### 6. 扩展开发提示

**新增工具页面模板：**

```vue
<template>
  <div class="tool-container">
    <div class="tool-card">
      <div class="card-header">
        <div class="header-left">
          <span class="card-title">操作</span>
          <el-tooltip content="使用说明">
            <QuestionFilled class="hint-icon" />
          </el-tooltip>
        </div>
      </div>
      <div class="card-body">
        <!-- 操作按钮 -->
      </div>
    </div>
    <!-- 输入区域 -->
    <!-- 输出区域 -->
  </div>
</template>
```

**核心样式清单（必须包含）：**

```css
.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}
.card-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}
```

---

## 🚀 部署说明

### 环境要求

| 依赖 | 版本要求 | 说明 |
|------|----------|------|
| Node.js | >= 16.0.0 | 前端构建 |
| Rust | >= 1.75.0 | Tauri 构建 |
| Windows | >= 10 1903 | 运行系统 |

### 生产打包

```bash
# 执行打包命令
npm run tauri build
```

### 打包产物

打包完成后，产物位于：
- **便携版**: `src-tauri/target/release/litobox.exe`（免安装，双击运行）
- **安装包**: 由于配置中 `bundle.targets` 为空数组，不生成安装包

### 运行方式

1. **便携版**：直接双击 `litobox.exe` 即可运行，无需安装
2. **启动时间**：≤ 1秒
3. **内存占用**：空闲时 ≤ 50MB

---

## 📖 使用说明

### 界面布局

```
┌─────────────────────────────────────────────────────────┐
│  [侧边导航]          [主内容区]                          │
│  ├─ JSON工具箱        │                                 │
│  ├─ 字符串工具        │  ┌─────────────────────────┐    │
│  ├─ 编码工具          │  │ 标题栏 (操作按钮)        │    │
│  ├─ SQL工具箱         │  ├─────────────────────────┤    │
│  ├─ ...              │  │ 输入区域                 │    │
│  │                   │  ├─────────────────────────┤    │
│  │                   │  │ 输出区域                 │    │
│  └─ 历史记录          │  └─────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 通用操作

| 操作 | 说明 |
|------|------|
| **粘贴** | 从剪贴板粘贴内容到输入框 |
| **清空** | 清空输入/输出区域 |
| **复制** | 将结果复制到剪贴板 |
| **拖拽** | 支持拖拽文件到输入区域 |

### 工具使用指南

#### JSON工具箱
- **格式化**：将压缩的JSON美化成可读格式
- **压缩**：将格式化的JSON转为单行
- **校验**：实时检测JSON语法错误
- **兼容解析**：支持带注释和尾逗号的非标准JSON

#### SQL工具箱
- **格式化**：美化SQL语句，添加缩进
- **压缩**：将多行SQL转为单行
- **JSON转Insert**：将JSON数组转为INSERT语句
- **MyBatis日志解析**：解析MyBatis的SQL日志，还原完整SQL

#### 字符串工具箱
- **空格处理**：首尾去空格、全局去空格、保留换行
- **大小写转换**：全大写、全小写、首字母大写、驼峰转换
- **文本清理**：去除换行符、制表符、空行

#### 编码工具
- **URL编码/解码**：处理URL参数
- **Base64编码/解码**：二进制数据编解码
- **时间戳转换**：秒级/毫秒级时间戳与北京时间互转

### 系统功能

- **主题切换**：自动跟随系统主题，或手动切换深色/浅色模式
- **托盘常驻**：最小化后隐藏到系统托盘，双击唤起
- **全局热键**：默认 `Ctrl+Shift+L` 唤起窗口（可自定义）
- **窗口置顶**：点击置顶按钮保持窗口在最上层

---

## ⚡ 性能指标

| 指标 | 目标值 |
|------|--------|
| 启动时间 | ≤ 1秒 |
| 操作响应 | ≤ 100ms |
| 空闲内存 | ≤ 50MB |
| 大文本处理 | 10w字符无卡顿 |
| 打包体积 | 便携版 ≤ 15MB |

---

## 🔒 安全说明

- ✅ 纯本地离线运行，无网络请求
- ✅ 仅保留必要权限：剪贴板、窗口控制、全局热键、本地存储
- ✅ 所有数据本地存储，不上传任何服务器
- ✅ 无广告、无后台推送、无数据采集

---

## 📋 版本规划

| 版本 | 状态 | 功能 |
|------|------|------|
| V1.0 | ✅ 完成 | JSON格式化/压缩/校验、字符串处理、编码工具、主题适配、托盘常驻 |
| V1.1 | ✅ 完成 | 正则测试工具、进制转换、UUID生成、操作历史记录 |
| V1.2 | ✅ 完成 | SQL工具箱、JS执行器、Mock数据生成、OCR工具、文件编码转换 |
| V1.3 | ✅ 完成 | 文本对比工具（行级/字符级差异高亮） |

---

## 📝 License

MIT License