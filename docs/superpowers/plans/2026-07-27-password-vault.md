# 密码保管箱 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现本地离线密码保管箱功能，支持主密码保护、加密存储、凭据增删改查和搜索筛选

**Architecture:** 后端使用 Rust + SQLite 实现加密存储和 CRUD 命令，前端使用 Vue 3 + Element Plus 实现表格布局的 UI，加密方案使用现有依赖 sha2 + base64

**Tech Stack:** Vue 3, TypeScript, Element Plus, Tauri 2.0, Rust, SQLite, sha2, base64

---

## 文件结构

| 文件 | 说明 | 状态 |
|------|------|------|
| `src-tauri/src/password_vault.rs` | 后端密码保管箱命令 | 新增 |
| `src/views/PasswordVault.vue` | 前端密码保管箱页面 | 新增 |
| `src-tauri/src/main.rs` | 注册密码保管箱命令 | 修改 |
| `src/store/index.ts` | 添加工具列表条目 | 修改 |
| `src-tauri/src/db.rs` | 数据库迁移（添加表） | 修改 |

---

## Task 1: 数据库迁移（添加 password_vault 表）

**Files:**
- Modify: `src-tauri/src/db.rs:133-273`

- [ ] **Step 1: 在 init_tables 函数中添加 password_vault 表创建语句**

在 `init_tables` 函数的 SQL batch 中添加：
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

- [ ] **Step 2: 验证编译通过**

Run: `cd src-tauri && cargo check`
Expected: PASS

---

## Task 2: 创建后端密码保管箱模块

**Files:**
- Create: `src-tauri/src/password_vault.rs`

- [ ] **Step 1: 创建 password_vault.rs 文件**

```rust
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use base64::{encode, decode};
use rand::Rng;

use crate::db::{with_conn, params};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credential {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultCredential {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

fn derive_key(master_password: &str, salt: &str) -> Vec<u8> {
    let mut key = master_password.as_bytes().to_vec();
    key.extend_from_slice(salt.as_bytes());
    
    for _ in 0..10000 {
        let mut hasher = Sha256::new();
        hasher.update(&key);
        key = hasher.finalize().to_vec();
    }
    
    key
}

fn encrypt(password: &str, key: &[u8]) -> String {
    let bytes = password.as_bytes();
    let mut encrypted = Vec::with_capacity(bytes.len());
    
    for (i, &byte) in bytes.iter().enumerate() {
        encrypted.push(byte ^ key[i % key.len()]);
    }
    
    encode(&encrypted)
}

fn decrypt(encrypted: &str, key: &[u8]) -> Result<String, String> {
    let bytes = decode(encrypted).map_err(|e| e.to_string())?;
    let mut decrypted = Vec::with_capacity(bytes.len());
    
    for (i, &byte) in bytes.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }
    
    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

fn generate_salt() -> String {
    let mut rng = rand::thread_rng();
    let salt: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    encode(&salt)
}

#[tauri::command]
pub fn pv_has_master_password() -> Result<bool, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_master_hash'")
            .map_err(|e| e.to_string())?;
        let result: Option<String> = stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result.is_some())
    })
}

#[tauri::command]
pub fn pv_set_master_password(password: String) -> Result<(), String> {
    if password.len() < 4 {
        return Err("密码长度至少4位".to_string());
    }
    
    let salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let hash = encode(&hasher.finalize());
    
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO config (key, value) VALUES ('password_vault_master_hash', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![hash],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO config (key, value) VALUES ('password_vault_salt', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![salt],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_verify_master_password(password: String) -> Result<bool, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_master_hash'")
            .map_err(|e| e.to_string())?;
        let hash: Option<String> = stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        let salt: Option<String> = stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        
        match (hash, salt) {
            (Some(h), Some(s)) => {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                hasher.update(s.as_bytes());
                let computed = encode(&hasher.finalize());
                Ok(computed == h)
            }
            _ => Ok(false),
        }
    })
}

#[tauri::command]
pub fn pv_list_credentials(master_password: String) -> Result<Vec<VaultCredential>, String> {
    let salt = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, url, username, encrypted_password, notes, created_at, updated_at
                      FROM password_vault ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                let encrypted_password: String = row.get(4)?;
                let password = decrypt(&encrypted_password, &key)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(4, rusqlite::types::Type::Text, Box::new(e)))?;
                
                Ok(VaultCredential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    username: row.get(3)?,
                    password,
                    notes: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn pv_search_credentials(master_password: String, query: String) -> Result<Vec<VaultCredential>, String> {
    let salt = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    with_conn(|conn| {
        let like = format!("%{}%", query);
        let mut stmt = conn
            .prepare("SELECT id, name, url, username, encrypted_password, notes, created_at, updated_at
                      FROM password_vault
                      WHERE name LIKE ?1 OR username LIKE ?1 OR url LIKE ?1
                      ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], |row| {
                let encrypted_password: String = row.get(4)?;
                let password = decrypt(&encrypted_password, &key)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(4, rusqlite::types::Type::Text, Box::new(e)))?;
                
                Ok(VaultCredential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    username: row.get(3)?,
                    password,
                    notes: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn pv_add_credential(master_password: String, credential: Credential) -> Result<i64, String> {
    let salt = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    let credential_salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(&key);
    hasher.update(credential_salt.as_bytes());
    let credential_key = hasher.finalize().to_vec();
    
    let encrypted_password = encrypt(&credential.password, &credential_key);
    
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO password_vault (name, url, username, encrypted_password, notes, salt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![credential.name, credential.url, credential.username, encrypted_password, credential.notes, credential_salt],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    })
}

#[tauri::command]
pub fn pv_update_credential(master_password: String, credential: Credential) -> Result<(), String> {
    let id = credential.id.ok_or("凭据ID不能为空".to_string())?;
    
    let salt = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    let credential_salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(&key);
    hasher.update(credential_salt.as_bytes());
    let credential_key = hasher.finalize().to_vec();
    
    let encrypted_password = encrypt(&credential.password, &credential_key);
    
    with_conn(|conn| {
        conn.execute(
            "UPDATE password_vault SET name = ?1, url = ?2, username = ?3, 
             encrypted_password = ?4, notes = ?5, salt = ?6, updated_at = datetime('now')
             WHERE id = ?7",
            params![credential.name, credential.url, credential.username, encrypted_password, credential.notes, credential_salt, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_delete_credential(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM password_vault WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}
```

- [ ] **Step 2: 添加 rand 依赖到 Cargo.toml**

在 `[dependencies]` 中添加：
```toml
rand = "0.8"
```

- [ ] **Step 3: 验证编译通过**

Run: `cd src-tauri && cargo check`
Expected: PASS

---

## Task 3: 注册后端命令到 main.rs

**Files:**
- Modify: `src-tauri/src/main.rs:1-155`

- [ ] **Step 1: 添加模块声明**

在 `mod cert_reader;` 后添加：
```rust
mod password_vault;
```

- [ ] **Step 2: 注册命令**

在 `invoke_handler` 的 `generate_handler!` 宏中添加：
```rust
password_vault::pv_has_master_password,
password_vault::pv_set_master_password,
password_vault::pv_verify_master_password,
password_vault::pv_list_credentials,
password_vault::pv_search_credentials,
password_vault::pv_add_credential,
password_vault::pv_update_credential,
password_vault::pv_delete_credential,
```

- [ ] **Step 3: 验证编译通过**

Run: `cd src-tauri && cargo check`
Expected: PASS

---

## Task 4: 添加工具列表条目到 store

**Files:**
- Modify: `src/store/index.ts:45-104`

- [ ] **Step 1: 添加工具条目**

在 `TOOL_LIST` 数组中（boost 条目之后）添加：
```typescript
{ id: 'passwordVault', name: '密码保管箱', icon: '🔐', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/><path d="M12 15a1 1 0 100-2 1 1 0 000 2z"/></svg>`, description: '本地密码保管箱，主密码保护，加密存储凭据', keywords: ['密码保管箱', 'password', 'vault', '凭据', '加密', '安全'], category: 'security' },
```

---

## Task 5: 创建前端密码保管箱页面

**Files:**
- Create: `src/views/PasswordVault.vue`

- [ ] **Step 1: 创建 PasswordVault.vue 文件**

```vue
<template>
  <div class="tool-container">
    <div v-if="!isUnlocked" class="lock-screen">
      <div class="lock-card">
        <div class="lock-icon">🔐</div>
        <h2>{{ hasMasterPassword ? '验证主密码' : '设置主密码' }}</h2>
        <p class="lock-hint">{{ hasMasterPassword ? '请输入主密码解锁密码保管箱' : '请设置主密码以保护您的凭据' }}</p>
        <el-input
          v-model="masterPassword"
          type="password"
          placeholder="输入密码"
          show-password
          @keyup.enter="handleUnlock"
          style="margin-bottom: 16px"
        />
        <el-input
          v-if="!hasMasterPassword"
          v-model="confirmPassword"
          type="password"
          placeholder="确认密码"
          show-password
          @keyup.enter="handleUnlock"
          style="margin-bottom: 16px"
        />
        <el-button type="primary" @click="handleUnlock" :loading="isLoading">
          {{ hasMasterPassword ? '解锁' : '设置' }}
        </el-button>
        <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
      </div>
    </div>

    <div v-else class="vault-content">
      <div class="tool-card sticky-card">
        <div class="card-header">
          <div class="header-left">
            <span class="card-title">密码保管箱</span>
            <el-tooltip placement="top" effect="dark">
              <template #content>
                <div class="tooltip-content">
                  <p>本地加密存储您的网站凭据</p>
                  <p>所有数据仅存储在本地，不会上传</p>
                </div>
              </template>
              <el-icon class="hint-icon"><QuestionFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="card-actions">
            <el-button size="small" @click="handleAdd">添加凭据</el-button>
            <el-button size="small" type="danger" @click="handleLock">🔒 锁定</el-button>
          </div>
        </div>
        <div class="card-body">
          <div class="search-bar">
            <el-input
              v-model="searchQuery"
              placeholder="搜索网站、用户名..."
              clearable
              @input="handleSearch"
              style="width: 300px"
            />
          </div>
        </div>
      </div>

      <div class="tool-card">
        <div class="card-header">
          <span class="card-title">凭据列表</span>
          <span class="count-badge">{{ credentials.length }} 条</span>
        </div>
        <div class="card-body">
          <DataTable :data="credentials" max-height="600">
            <el-table-column prop="name" label="网站" min-width="150" />
            <el-table-column prop="url" label="网址" min-width="200" />
            <el-table-column prop="username" label="用户名" min-width="120" />
            <el-table-column prop="password" label="密码" min-width="150">
              <template #default="scope">
                <div class="password-cell">
                  <span v-if="!showPasswords[scope.row.id]">{{ '*'.repeat(scope.row.password.length) }}</span>
                  <code v-else>{{ scope.row.password }}</code>
                  <el-button
                    size="small"
                    @click="togglePassword(scope.row.id)"
                    class="toggle-btn"
                  >
                    <el-icon>{{ showPasswords[scope.row.id] ? <EyeOff /> : <Eye /> }}</el-icon>
                  </el-button>
                </div>
              </template>
            </el-table-column>
            <el-table-column prop="notes" label="备注" min-width="150" />
            <el-table-column label="操作" width="120" fixed="right">
              <template #default="scope">
                <div class="action-buttons">
                  <el-button size="small" @click="handleCopy(scope.row.password)">复制</el-button>
                  <el-button size="small" @click="handleEdit(scope.row)">编辑</el-button>
                  <el-button size="small" type="danger" @click="handleDelete(scope.row.id)">删除</el-button>
                </div>
              </template>
            </el-table-column>
          </DataTable>
        </div>
      </div>

      <el-dialog
        v-model="dialogVisible"
        :title="isEditing ? '编辑凭据' : '添加凭据'"
        width="500px"
      >
        <el-form :model="formData" label-width="80px">
          <el-form-item label="网站名称" required>
            <el-input v-model="formData.name" placeholder="如：GitHub" />
          </el-form-item>
          <el-form-item label="网址">
            <el-input v-model="formData.url" placeholder="如：https://github.com" />
          </el-form-item>
          <el-form-item label="用户名" required>
            <el-input v-model="formData.username" placeholder="输入用户名" />
          </el-form-item>
          <el-form-item label="密码" required>
            <el-input v-model="formData.password" type="password" show-password placeholder="输入密码" />
          </el-form-item>
          <el-form-item label="备注">
            <el-input v-model="formData.notes" type="textarea" :rows="3" placeholder="备注信息" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSave">保存</el-button>
        </template>
      </el-dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { QuestionFilled, Eye, EyeOff } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { useToolboxStore } from '@/store'
import DataTable from '@/components/DataTable.vue'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

const store = useToolboxStore()
const confirm = useConfirmDialog()

interface Credential {
  id: number
  name: string
  url: string
  username: string
  password: string
  notes: string
  created_at: string
  updated_at: string
}

const isUnlocked = ref(true)
const hasMasterPassword = ref(false)
const masterPassword = ref('')
const confirmPassword = ref('')
const isLoading = ref(false)
const errorMessage = ref('')

const credentials = ref<Credential[]>([])
const searchQuery = ref('')
const showPasswords = reactive<Record<number, boolean>>({})

const dialogVisible = ref(false)
const isEditing = ref(false)
const formData = reactive({
  id: 0,
  name: '',
  url: '',
  username: '',
  password: '',
  notes: ''
})

const loadCredentials = async () => {
  try {
    credentials.value = await invoke<Credential[]>('pv_list_credentials', {
      masterPassword: masterPassword.value
    })
  } catch (e) {
    ElMessage.error('加载凭据失败: ' + String(e))
  }
}

const handleUnlock = async () => {
  if (!masterPassword.value.trim()) {
    errorMessage.value = '请输入密码'
    return
  }

  if (!hasMasterPassword.value && masterPassword.value !== confirmPassword.value) {
    errorMessage.value = '两次输入的密码不一致'
    return
  }

  isLoading.value = true
  errorMessage.value = ''

  try {
    if (!hasMasterPassword.value) {
      await invoke('pv_set_master_password', { password: masterPassword.value })
      ElMessage.success('主密码设置成功')
    } else {
      const verified = await invoke<boolean>('pv_verify_master_password', {
        password: masterPassword.value
      })
      if (!verified) {
        errorMessage.value = '密码错误'
        isLoading.value = false
        return
      }
    }

    isUnlocked.value = false
    await loadCredentials()
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    isLoading.value = false
  }
}

const handleLock = async () => {
  const ok = await confirm.ask('锁定确认', '确定要锁定密码保管箱吗？', { type: 'warning' })
  if (!ok) return
  
  isUnlocked.value = true
  masterPassword.value = ''
  confirmPassword.value = ''
  credentials.value = []
  ElMessage.success('已锁定')
}

const handleSearch = async () => {
  if (!searchQuery.value.trim()) {
    await loadCredentials()
    return
  }

  try {
    credentials.value = await invoke<Credential[]>('pv_search_credentials', {
      masterPassword: masterPassword.value,
      query: searchQuery.value
    })
  } catch (e) {
    ElMessage.error('搜索失败: ' + String(e))
  }
}

const togglePassword = (id: number) => {
  showPasswords[id] = !showPasswords[id]
}

const handleCopy = async (password: string) => {
  try {
    await navigator.clipboard.writeText(password)
    ElMessage.success('密码已复制')
  } catch {
    ElMessage.error('复制失败')
  }
}

const handleAdd = () => {
  isEditing.value = false
  formData.id = 0
  formData.name = ''
  formData.url = ''
  formData.username = ''
  formData.password = ''
  formData.notes = ''
  dialogVisible.value = true
}

const handleEdit = (credential: Credential) => {
  isEditing.value = true
  formData.id = credential.id
  formData.name = credential.name
  formData.url = credential.url
  formData.username = credential.username
  formData.password = credential.password
  formData.notes = credential.notes
  dialogVisible.value = true
}

const handleDelete = async (id: number) => {
  const ok = await confirm.ask('删除确认', '确定要删除这条凭据吗？', { type: 'danger' })
  if (!ok) return

  try {
    await invoke('pv_delete_credential', { id })
    await loadCredentials()
    ElMessage.success('删除成功')
  } catch (e) {
    ElMessage.error('删除失败: ' + String(e))
  }
}

const handleSave = async () => {
  if (!formData.name.trim()) {
    ElMessage.warning('请输入网站名称')
    return
  }
  if (!formData.username.trim()) {
    ElMessage.warning('请输入用户名')
    return
  }
  if (!formData.password.trim()) {
    ElMessage.warning('请输入密码')
    return
  }

  try {
    if (isEditing.value) {
      await invoke('pv_update_credential', {
        masterPassword: masterPassword.value,
        credential: {
          id: formData.id,
          name: formData.name,
          url: formData.url,
          username: formData.username,
          password: formData.password,
          notes: formData.notes
        }
      })
      ElMessage.success('更新成功')
    } else {
      await invoke('pv_add_credential', {
        masterPassword: masterPassword.value,
        credential: {
          name: formData.name,
          url: formData.url,
          username: formData.username,
          password: formData.password,
          notes: formData.notes
        }
      })
      ElMessage.success('添加成功')
    }
    dialogVisible.value = false
    await loadCredentials()
  } catch (e) {
    ElMessage.error('保存失败: ' + String(e))
  }
}

onMounted(async () => {
  try {
    hasMasterPassword.value = await invoke<boolean>('pv_has_master_password')
    isUnlocked.value = !hasMasterPassword.value
  } catch (e) {
    ElMessage.error('初始化失败: ' + String(e))
  }
})
</script>

<style scoped>
.lock-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.lock-card {
  width: 400px;
  padding: 40px;
  background: var(--bg-card);
  border-radius: 12px;
  text-align: center;
  border: 1px solid var(--border-color);
}

.lock-icon {
  font-size: 64px;
  margin-bottom: 20px;
}

.lock-card h2 {
  color: var(--accent-cyan);
  font-size: 24px;
  margin-bottom: 8px;
}

.lock-hint {
  color: var(--text-secondary);
  font-size: 14px;
  margin-bottom: 24px;
}

.error-message {
  color: var(--accent-red);
  font-size: 13px;
  margin-top: 12px;
}

.search-bar {
  display: flex;
  gap: 12px;
}

.count-badge {
  font-size: 13px;
  color: var(--text-secondary);
  background: var(--bg-input);
  padding: 4px 12px;
  border-radius: 12px;
}

.password-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.password-cell code {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
}

.toggle-btn {
  padding: 4px;
}

.action-buttons {
  display: flex;
  gap: 4px;
}

.tooltip-content {
  max-width: 300px;
}
</style>
```

---

## Task 6: 更新版本号和 README

**Files:**
- Modify: `src-tauri/Cargo.toml:3`
- Modify: `package.json`
- Modify: `README.md`

- [ ] **Step 1: 更新 Cargo.toml 版本号**

将 `version = "6.6.0"` 改为 `version = "6.7.0"`

- [ ] **Step 2: 更新 package.json 版本号**

读取 package.json，更新 version 字段

- [ ] **Step 3: 更新 README.md**

在已完成版本表格中添加一行：
```
| V6.7 | ✅ | 密码保管箱（主密码保护、加密存储、凭据增删改查、搜索筛选） |
```

---

## Self-Review

**1. Spec coverage:**
- ✅ 主密码保护 (Task 2)
- ✅ 加密存储 (Task 2)
- ✅ 凭据管理增删改查 (Task 2)
- ✅ 搜索筛选 (Task 2)
- ✅ 表格布局 UI (Task 5)
- ✅ 数据库设计 (Task 1)

**2. Placeholder scan:** 无占位符

**3. Type consistency:** Credential 结构前后一致
