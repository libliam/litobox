# 密码保管箱 - 设计文档

## 1. 功能概述

本地离线密码保管箱，主密码保护，凭据数据加密存储于 SQLite，支持网站/用户名/密码/备注的增删改查和搜索筛选。

## 2. 核心功能

| 功能 | 说明 |
|------|------|
| 主密码保护 | 首次使用设置主密码，后续打开时验证 |
| 加密存储 | 使用现有依赖（sha2 + base64）实现加密，不新增依赖 |
| 凭据管理 | 网站/用户名/密码/备注的增删改查 |
| 搜索筛选 | 按网站名或用户名搜索 |
| 快捷复制 | 点击复制密码到剪贴板 |

## 3. 加密方案

**原则**：不新增依赖，使用现有 `sha2` 和 `base64` crate

### 3.1 密钥派生
- 主密码 + 随机 salt（16 字节）
- 通过 SHA-256 迭代 10000 次生成 32 字节密钥

### 3.2 加密方式
- XOR 流密码：密钥循环与明文异或
- 密文使用 base64 编码存储

### 3.3 存储格式
```
salt (base64) + ":" + encrypted_password (base64)
```

## 4. UI 布局（表格布局）

### 4.1 页面结构
- **顶部操作栏**：搜索框 + 添加按钮 + 主密码设置/验证区域
- **表格列表**：网站、用户名、密码（星号隐藏，点击眼睛切换）、备注、操作（复制/编辑/删除）
- **编辑弹窗**：表单填写凭据信息

### 4.2 交互流程
1. 进入页面 → 检测是否已设置主密码
2. 未设置 → 弹出设置主密码弹窗
3. 已设置 → 弹出验证主密码弹窗
4. 验证通过 → 加载并解密凭据列表
5. 用户操作 → 搜索/添加/编辑/删除/复制

## 5. 数据库设计

### 5.1 凭据表
```sql
CREATE TABLE IF NOT EXISTS password_vault (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT DEFAULT '',
    username TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,
    notes TEXT DEFAULT '',
    salt TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_password_vault_name ON password_vault(name);
```

### 5.2 主密码配置
存储于 `config` 表：
- `password_vault_master_hash`: 主密码哈希（SHA-256）
- `password_vault_salt`: 主密码盐

## 6. 后端命令设计

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `pv_set_master_password` | `password: string` | `Result<(), string>` | 设置主密码 |
| `pv_verify_master_password` | `password: string` | `Result<boolean, string>` | 验证主密码 |
| `pv_has_master_password` | 无 | `Result<boolean, string>` | 检查是否已设置主密码 |
| `pv_list_credentials` | `master_password: string` | `Result<Vec<Credential>, string>` | 获取凭据列表 |
| `pv_search_credentials` | `master_password: string, query: string` | `Result<Vec<Credential>, string>` | 搜索凭据 |
| `pv_add_credential` | `master_password: string, credential: Credential` | `Result<i64, string>` | 添加凭据 |
| `pv_update_credential` | `master_password: string, credential: Credential` | `Result<(), string>` | 更新凭据 |
| `pv_delete_credential` | `id: i64` | `Result<(), string>` | 删除凭据 |

### 6.1 Credential 结构
```rust
#[derive(Serialize, Deserialize, Debug)]
struct Credential {
    id: Option<i64>,
    name: String,
    url: String,
    username: String,
    password: String,
    notes: String,
}
```

## 7. 文件结构

| 文件 | 说明 | 状态 |
|------|------|------|
| `src-tauri/src/password_vault.rs` | 后端命令（新文件） | 新增 |
| `src/views/PasswordVault.vue` | 前端页面（新文件） | 新增 |
| `src-tauri/src/main.rs` | 注册命令（更新） | 修改 |
| `src/store/index.ts` | 添加工具列表（更新） | 修改 |
| `src-tauri/src/db.rs` | 数据库迁移（更新） | 修改 |

## 8. 安全性考虑

- 主密码仅用于派生密钥，不在内存中长期存储
- 解密后的密码仅在 UI 展示时临时使用
- 所有操作通过后端命令完成，前端不处理加密逻辑
- 本地离线存储，无网络传输
