# Hosts 文件管理器设计文档

## 目标

在 LitoBox 中新增 Hosts 文件管理器，提供表格化编辑、多环境 profile 切换、自动备份恢复功能，替代手动编辑 `C:\Windows\System32\drivers\etc\hosts` 的传统方式。

## 功能范围

### 包含
- Hosts 条目表格化展示与行内编辑（IP / 域名 / 启用状态 / 备注）
- 启用/禁用条目（复选框切换，等价于注释 `#` 切换）
- 多环境 profile 管理（创建 / 切换 / 删除）
- 自动备份（每次保存前备份，保留最近 20 份）+ 列表恢复
- 管理员权限检测与提示

### 不包含
- 语法高亮（表格模式无需）
- 纯文本编辑模式
- DNS 缓存刷新（用户手动执行 `ipconfig /flushdns`）
- 非 Windows 系统支持

## 页面结构

多 Tab 布局，遵循 `_ToolTemplate.vue` 规范：

### Tab 1: Hosts 编辑

```
┌─ admin-banner（非管理员时显示）──────────────────────────┐
├─ sticky-card: Tab 栏 ──────────────────────────────────┤
├─ 操作栏 ─── [添加条目] [保存] [刷新] [搜索框] ──────────┤
├─ Hosts 条目表格 ────────────────────────────────────────┤
│ ☑ │ IP            │ 域名              │ 备注           │
│ ☑ │ 127.0.0.1    │ localhost         │ 本地回环       │
│ ☐ │ 192.168.1.1  │ api.test.com      │ 测试服务器     │
│   │              │ web.test.com      │                │
│ ☑ │ 0.0.0.0      │ ads.example.com   │ 屏蔽广告       │
└────────────────────────────────────────────────────────┘
```

- **admin-banner**：参考 `ServiceListView.vue`，非管理员时显示「编辑 hosts 需要管理员权限」
- **操作栏**：添加条目按钮、保存按钮（非管理员置灰）、刷新、搜索框
- **表格**：
  - 第一列：复选框（启用/禁用）
  - 第二列：IP 地址（行内编辑，支持 IPv4/IPv6）
  - 第三列：域名（多域名合并展示，一行多域名用空格分隔；编辑时 textarea）
  - 第四列：备注（行内编辑）
  - 第五列：操作列（删除按钮）
  - 表格底部：统计「共 N 条，启用 M 条」

### Tab 2: Profile 管理

```
┌─ Profile 列表 ──────────────────────────────────────────┐
│ 名称      │ 条目数 │ 更新时间       │ 操作              │
│ 默认      │ 12     │ 2026-07-21    │ [切换] [删除]    │
│ dev       │ 8      │ 2026-07-20    │ [切换] [删除]    │
│ test      │ 15     │ 2026-07-19    │ [切换] [删除]    │
├───────────────────────────────────────────────────────┤
│ [新建 Profile] [从当前 Hosts 保存为 Profile]           │
└───────────────────────────────────────────────────────┘
```

- **默认 profile**：不存储为文件，`hosts_profile_list` 返回时动态加入"默认"项（直接读取当前系统 hosts）。不可删除
- **切换 profile**：将选中 profile 的条目写入系统 hosts（保存前自动备份当前 hosts）。切换前若编辑 Tab 有未保存修改，弹窗提示用户选择「保存当前修改」或「丢弃」
- **新建 profile**：弹窗输入名称，创建空 profile（entries 为空数组）
- **从当前 Hosts 保存为 Profile**：弹窗输入名称，将当前 hosts 条目保存为新 profile。若名称已存在则提示覆盖确认
- `hosts_profile_save`：name 已存在则覆盖，不存在则创建

### Tab 3: 备份恢复

```
┌─ 备份列表 ──────────────────────────────────────────────┐
│ 时间                 │ 大小    │ 操作                   │
│ 2026-07-21 14:30:25 │ 2.1 KB │ [预览] [恢复] [删除]  │
│ 2026-07-21 10:15:00 │ 1.8 KB │ [预览] [恢复] [删除]  │
├───────────────────────────────────────────────────────┤
│ [刷新列表] [立即备份]                                  │
└───────────────────────────────────────────────────────┘
```

- **预览**：弹窗显示备份文件内容
- **恢复**：确认后用备份覆盖当前 hosts（恢复前再自动备份一次当前 hosts）
- **立即备份**：手动触发一次备份
- 自动清理：超过 20 份时自动删除最旧的

## 数据结构

### HostsEntry（Rust struct，snake_case 字段）

```rust
pub struct HostsEntry {
    pub enabled: bool,           // 启用状态（false = 行首有 #）
    pub ip: String,              // IP 地址
    pub domains: Vec<String>,    // 域名列表（一行可多域名）
    pub comment: String,         // 行内注释（# 后的内容，不含 #）
}
```

### HostsFile（解析后的完整文件）

```rust
pub struct HostsFile {
    pub entries: Vec<HostsEntry>,  // 解析出的条目
    pub raw_lines: Vec<String>,    // 无法解析的行（原样保留）
    pub path: String,              // hosts 文件路径
}
```

### Profile（存储格式，JSON）

```json
{
  "name": "dev",
  "entries": [
    {"enabled": true, "ip": "127.0.0.1", "domains": ["localhost"], "comment": "本地回环"},
    {"enabled": false, "ip": "192.168.1.100", "domains": ["api.dev.com"], "comment": "开发服务器"}
  ],
  "created_at": "2026-07-21T10:00:00Z",
  "updated_at": "2026-07-21T14:30:00Z"
}
```

### BackupInfo（备份列表项）

```rust
pub struct BackupInfo {
    pub filename: String,      // hosts_20260721_143025
    pub timestamp: String,     // 2026-07-21 14:30:25
    pub size: u64,             // 字节数
    pub path: String,          // 完整路径
}
```

## Hosts 解析规则

### 解析逻辑（逐行处理）

1. **空行**：跳过（不加入 entries，但计入 raw_lines 保留原样）
2. **纯注释行**（以 `#` 开头，且去除 `#` 后无 IP 模式）：保留到 raw_lines
3. **禁用条目**（`# IP domain...`）：解析为 `enabled: false` 的 entry
4. **启用条目**（`IP domain...`）：解析为 `enabled: true` 的 entry
5. **行内注释**（`IP domain # comment`）：comment 字段存储 `#` 后内容
6. **无法解析的行**：保留到 raw_lines

### 回写逻辑

1. 先写 raw_lines 中的前导行（注释块等）
2. 再写 entries（禁用的加 `#` 前缀）
3. 每行格式：`[IP] [domain1] [domain2]... [# comment]`

### 多域名处理

- 解析时：同一行的多个域名合并为一个 entry 的 `domains` 数组
- 展示时：`domains.join(" ")` 合并显示
- 编辑时：textarea，每行一个域名或空格分隔

## 后端设计

### 新增文件：`src-tauri/src/hosts_manager.rs`

### Tauri 命令

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `hosts_read` | 无 | `Result<HostsFile, String>` | 读取系统 hosts |
| `hosts_save` | `entries: Vec<HostsEntry>` | `Result<(), String>` | 保存到 hosts（自动备份 + 原子写入） |
| `hosts_check_admin` | 无 | `Result<bool, String>` | 检测管理员权限 |
| `hosts_list_backups` | 无 | `Result<Vec<BackupInfo>, String>` | 列出备份 |
| `hosts_preview_backup` | `filename: String` | `Result<String, String>` | 预览备份内容 |
| `hosts_restore_backup` | `filename: String` | `Result<(), String>` | 恢复备份（恢复前再备份一次） |
| `hosts_delete_backup` | `filename: String` | `Result<(), String>` | 删除指定备份 |
| `hosts_create_backup` | 无 | `Result<BackupInfo, String>` | 立即创建备份 |
| `hosts_profile_list` | 无 | `Result<Vec<ProfileMeta>, String>` | 列出所有 profile |
| `hosts_profile_load` | `name: String` | `Result<Vec<HostsEntry>, String>` | 加载 profile 条目 |
| `hosts_profile_save` | `name: String, entries: Vec<HostsEntry>` | `Result<(), String>` | 保存为 profile（覆盖） |
| `hosts_profile_delete` | `name: String` | `Result<(), String>` | 删除 profile（默认不可删） |
| `hosts_profile_apply` | `name: String` | `Result<(), String>` | 将 profile 写入系统 hosts（自动备份当前） |

### 关键函数

```rust
// 管理员检测（windows-sys）
fn is_admin() -> bool {
    // OpenProcessToken + GetTokenInformation(TokenElevation)
}

// hosts 路径
const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

// Profile 存储目录
fn profiles_dir() -> PathBuf {
    // %APPDATA%/com.dev.toolbox/hosts_profiles/
}

// 备份存储目录
fn backups_dir() -> PathBuf {
    // %APPDATA%/com.dev.toolbox/hosts_backups/
}

// 原子写入
fn atomic_write_hosts(content: &str) -> Result<(), String> {
    // 1. 写入同目录临时文件（hosts.tmp）
    // 2. std::fs::rename 替换（同分区保证原子性）
}

// 自动备份 + 清理
fn auto_backup() -> Result<(), String> {
    // 1. 复制 hosts 到 backups_dir，命名 hosts_YYYYMMDD_HHmmss
    // 2. 读取 backups_dir，按时间排序
    // 3. 超过 20 份则删除最旧的
}

// 解析 hosts
fn parse_hosts(content: &str) -> HostsFile { ... }

// 序列化回 hosts 格式
fn serialize_hosts(file: &HostsFile) -> String { ... }
```

### 依赖变更

需要在 `Cargo.toml` 的 `windows-sys` features 中添加：
```toml
"Win32_Security",  # TokenElevation, TOKEN_QUERY
```

## 前端设计

### 新增文件：`src/views/HostsView.vue`

### 页面结构

```vue
<template>
  <div class="tool-container">
    <!-- admin-banner -->
    <div v-if="!isAdmin" class="admin-banner">...</div>

    <!-- Tab 栏（sticky） -->
    <div class="tool-card sticky-card">
      <el-tabs v-model="activeTab" class="hosts-tabs">
        <el-tab-pane label="Hosts 编辑" name="editor" />
        <el-tab-pane label="Profile 管理" name="profiles" />
        <el-tab-pane label="备份恢复" name="backups" />
      </el-tabs>
    </div>

    <!-- Tab 1: 编辑 -->
    <div v-if="activeTab === 'editor'" class="tool-card">
      <!-- 操作栏 -->
      <!-- 表格 -->
    </div>

    <!-- Tab 2: Profile -->
    <div v-if="activeTab === 'profiles'" class="tool-card">...</div>

    <!-- Tab 3: 备份 -->
    <div v-if="activeTab === 'backups'" class="tool-card">...</div>
  </div>
</template>
```

### 状态管理

- 不加入 Pinia store（hosts 数据量大，且无需跨页面共享）
- 组件内维护：`entries`, `profiles`, `backups`, `isAdmin`, `activeTab`
- KeepAlive 缓存：切换 tab 后保留状态（AGENTS 经验 12，用 `onActivated` 重新加载）

### 表格交互

- 复选框列：`el-table-column type="selection"` 或自定义复选框
- IP 列：`el-input` 行内编辑
- 域名列：`el-input` 行内编辑（多域名时 textarea）
- 备注列：`el-input` 行内编辑
- 操作列：删除按钮
- 添加条目：表格末尾新增空行

## 存储路径

| 用途 | 路径 |
|------|------|
| 系统 hosts | `C:\Windows\System32\drivers\etc\hosts` |
| Profile 存储 | `%APPDATA%\com.dev.toolbox\hosts_profiles\<name>.json` |
| 备份存储 | `%APPDATA%\com.dev.toolbox\hosts_backups\hosts_YYYYMMDD_HHmmss` |

## 错误处理

- **读取 hosts 失败**：显示错误信息，可能是文件不存在或权限不足
- **写入 hosts 失败**：保留临时文件，提示用户手动恢复
- **Profile 不存在**：返回错误，前端提示
- **备份目录不存在**：自动创建
- **磁盘空间不足**：写入前检查，提示用户

## 安全考虑

- 所有写入操作前自动备份（可回滚）
- 原子写入（rename，避免半写入状态）
- 管理员权限检测（避免静默失败）
- Profile 删除二次确认
- 备份恢复二次确认

## 测试策略

### 单元测试（Rust）

- `parse_hosts`：各种格式行（空行、纯注释、禁用条目、启用条目、多域名、行内注释、无法解析的行）
- `serialize_hosts`：解析 → 序列化 → 解析，数据一致
- `auto_backup`：备份创建 + 超过 20 份清理
- `is_admin`：在管理员/非管理员环境下返回正确值

### 手动验证

- 非管理员运行：banner 显示，保存按钮置灰
- 管理员运行：可读取、编辑、保存 hosts
- 启用/禁用切换：对应 `#` 注释切换
- 多域名展示：一行多域名合并显示
- Profile 切换：hosts 内容更新
- 备份恢复：hosts 内容恢复到备份时状态
- KeepAlive：切换 tab 再回来保留状态

## 版本规划

- 版本号：5.8.0 → 5.9.0（新增工具菜单项，按 semver minor）
- 侧边栏分类：系统工具
- README 更新：V5.9 功能记录
