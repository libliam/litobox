# 证书/密钥查看器 (A11) 实现计划

## Context

新增 Windows 证书查看器，支持浏览系统证书存储（个人/受信任的根/中间CA）和解析证书文件（.cer/.crt/.pfx），展示 SSL 证书详情（主题/颁发者/有效期/SAN/指纹）。

**版本**: 6.5.0 → 6.6.0（新增菜单项，minor 版本号升）

## 技术方案

纯 PowerShell + .NET `X509Certificate2` 实现，**无需新增任何 Rust crate 依赖**，无需修改 `Cargo.toml` 或 `windows-sys` features。

## 关键沙箱避坑

PowerShell 脚本中**禁止使用 `ForEach-Object { [PSCustomObject]@{...} }`**，必须用 `foreach` 循环 + `New-Object PSObject -Property @{...}` 替代。`Where-Object` 过滤也改为 `foreach` + `if` 条件。

## 后端实现

### 新增文件：`src-tauri/src/cert_reader.rs`

**数据结构**：
- `CertInfo` — 证书列表项（subject, issuer, not_before, not_after, serial_number, thumbprint, store_name, has_private_key, is_expired）
- `CertStoreList` — 三个存储的证书列表（personal, root, ca）
- `CertDetail` — 证书详情（+ version, thumbprint_sha256, san, key_usage, enhanced_key_usage, basic_constraints, signature_algorithm, public_key, raw_pem, days_until_expiry）

**Tauri 命令**（同步 `#[tauri::command] fn`，与 env_vars/scheduled_tasks 风格一致）：
- `read_cert_store()` → `CertStoreList` — 读取 CurrentUser 三个证书存储
- `get_cert_detail(thumbprint, store_name)` → `CertDetail` — 按指纹获取详情
- `parse_cert_file(file_path, password)` → `CertDetail` — 解析证书文件

**PowerShell 脚本**：
- 复用 `run_powershell()` + `CREATE_NO_WINDOW` + `GBK` 解码模式
- 存储查询：`Get-ChildItem -Path Cert:\CurrentUser\My|Root|CA`，用 `foreach` 循环收集
- 详情查询：按 `Thumbprint` 匹配，用 `foreach` + `if` 替代 `Where-Object` 提取扩展信息
- 文件解析：`New-Object X509Certificate2($path, $password)`

**JSON 反序列化**：中间结构体 `PsCertStoreRaw` / `PsCertInfo` / `PsCertDetail` 匹配 PowerShell 输出字段名（PascalCase），再映射到 CamelCase 输出结构体。

### 修改文件：`src-tauri/src/main.rs`

```rust
mod cert_reader;  // 在 mod boost; 之后

// invoke_handler 中新增：
cert_reader::read_cert_store,
cert_reader::get_cert_detail,
cert_reader::parse_cert_file,
```

## 前端实现

### 新增文件：`src/views/CertViewer.vue`

**布局**：左右分栏（左侧 45% 列表 + 右侧 55% 详情面板）

**Tab 栏**：
- Tab 1: 证书存储 — 子 Tab（个人/受信任的根/中间CA）+ 搜索框 + 证书列表 + 刷新按钮
- Tab 2: 文件解析 — 选择文件按钮 + 密码输入（仅 pfx）+ 详情面板

**证书列表**：左侧卡片式列表，每项显示主题（截断）、颁发者、有效期、过期状态标签（红色"已过期"/黄色"即将过期"/绿色"有效"）

**详情面板**：使用 `el-descriptions` 组件，结构化展示：
- 基本信息：主题、颁发者、版本、序列号
- 有效期：生效日期 → 到期日期，过期天数标签
- 指纹：SHA1 + SHA256（可复制）
- SAN：域名/IP/邮箱列表
- 密钥用法 / 增强密钥用法
- 签名算法 / 公钥信息
- 完整 PEM 文本（可复制）

**操作按钮**：复制 PEM、导出证书（保存为 .cer 文件）

### 修改文件

**`src/utils/systemInfoClient.ts`**：新增类型定义 + invoke 函数

**`src/store/index.ts`**：在 `TOOL_LIST` 中 `envVars` 条目之后添加 `certViewer` 条目，`category: 'system'`

**`src/App.vue`**：添加 import 和 `componentMap` 映射

**`src/views/WorkflowView.vue`**：`executeStep()` 中添加 `certViewer` 分支

## 实现步骤

1. 创建 `src-tauri/src/cert_reader.rs`（数据结构 + 3 个 PowerShell 脚本 + 3 个 Tauri 命令 + 单元测试）
2. 修改 `src-tauri/src/main.rs`（注册模块和命令）
3. 创建 `src/views/CertViewer.vue`（完整前端页面）
4. 修改 `src/utils/systemInfoClient.ts`（类型 + invoke 函数）
5. 修改 `src/store/index.ts`（TOOL_LIST 注册）
6. 修改 `src/App.vue`（路由注册）
7. 修改 `src/views/WorkflowView.vue`（工作流集成）
8. 更新 `README.md`（V6.6 功能记录）

## 验证

1. `cargo check` / `cargo test` — Rust 编译和测试通过
2. `npx vue-tsc --noEmit` — TypeScript 类型检查通过
3. 手动测试：证书存储浏览、详情查看、文件解析、过期警告、PEM 复制导出