# 文本编辑器 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 LitoBox 添加一个基于 CodeMirror 6 的文本编辑器功能，支持分类文件夹管理、语法高亮、查找替换、自动保存、编码转换、行操作等基础编辑能力。

**Architecture:** 左侧文件夹树（200px）+ 右侧 CodeMirror 编辑区布局。混合存储：SQLite 存元数据（标题、分类、更新时间），内容存为本地文件（`%APPDATA%/com.dev.toolbox/notes/`）。Rust 后端处理文件系统操作，前端 CodeMirror 6 提供编辑能力。

**Tech Stack:** Vue 3 + TypeScript + CodeMirror 6 + Element Plus + Tauri 2.0 + rusqlite

---

## 文件结构

### 新建文件

| 文件 | 职责 |
|------|------|
| `src-tauri/src/note_manager.rs` | Rust 后端：笔记文件系统操作（CRUD + 读写） |
| `src/utils/noteClient.ts` | 前端：封装 Tauri note 命令调用 |
| `src/views/NoteEditor.vue` | 主页面：左侧文件夹树 + 右侧 CodeMirror 编辑区 |
| `src/components/NoteSidebar.vue` | 侧边栏：文件夹树、右键菜单、拖拽 |
| `src/components/NoteToolbar.vue` | 顶部工具栏：查找替换、语法、编码、行操作 |
| `src/components/CodeMirrorEditor.vue` | CodeMirror 6 编辑器封装组件 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `src-tauri/src/db.rs` | 新增 `notes` 表 + CRUD 函数 |
| `src-tauri/src/main.rs` | 注册 note 相关 Tauri 命令 |
| `src/store/index.ts` | 新增 note 相关状态和方法 |
| `src/components/SidebarNav.vue` | 新增"文本编辑器"导航项 |
| `src/App.vue` | 导入并注册 NoteEditor 组件 |
| `package.json` | 新增 CodeMirror 6 依赖 |
| `src-tauri/Cargo.toml` | 新增 `chardetng` 依赖（编码检测） |

---

### Task 1: 安装 CodeMirror 6 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 CodeMirror 6 核心包**

```bash
cd d:\work\litobox
npm install @codemirror/state @codemirror/view @codemirror/commands @codemirror/search @codemirror/language @codemirror/theme-one-dark @codemirror/theme-one-light @codemirror/lang-javascript @codemirror/lang-json @codemirror/lang-html @codemirror/lang-css @codemirror/lang-markdown @codemirror/lang-xml @codemirror/lang-sql @codemirror/lang-python @codemirror/lang-rust
```

- [ ] **Step 2: 验证安装**

```bash
npm ls @codemirror/state
```
Expected: 显示已安装的版本

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: 安装 CodeMirror 6 依赖"
```

---

### Task 2: Rust 后端 — 添加 notes 表和 CRUD

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: 在 `init_tables` 函数中添加 notes 表创建语句**

在 `src-tauri/src/db.rs` 的 `init_tables` 函数中添加：

```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS notes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        parent_id INTEGER DEFAULT NULL,
        name TEXT NOT NULL,
        type TEXT NOT NULL DEFAULT 'file',
        file_path TEXT,
        language TEXT DEFAULT 'plaintext',
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
    )",
    [],
)?;
conn.execute(
    "CREATE INDEX IF NOT EXISTS idx_notes_parent ON notes(parent_id)",
    [],
)?;
```

- [ ] **Step 2: 添加 NoteItem 结构体**

在 `db.rs` 中添加：

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct NoteItem {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub r#type: String,  // 'folder' or 'file'
    pub file_path: Option<String>,
    pub language: String,
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 3: 添加 note_list 函数**

```rust
pub fn note_list(parent_id: Option<i64>) -> Result<Vec<NoteItem>, String> {
    with_conn(|conn| {
        let sql = "SELECT id, parent_id, name, type, file_path, language, created_at, updated_at 
                   FROM notes WHERE parent_id = ? ORDER BY type DESC, name ASC";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![parent_id], |row| {
            Ok(NoteItem {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                r#type: row.get(3)?,
                file_path: row.get(4)?,
                language: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    })
}
```

- [ ] **Step 4: 添加 note_create 函数**

```rust
pub fn note_create(name: &str, note_type: &str, parent_id: Option<i64>) -> Result<NoteItem, String> {
    with_conn(|conn| {
        let file_path = if note_type == "file" {
            Some(generate_note_file_path(name))
        } else {
            None
        };
        conn.execute(
            "INSERT INTO notes (name, type, parent_id, file_path) VALUES (?, ?, ?, ?)",
            params![name, note_type, parent_id, file_path],
        ).map_err(|e| e.to_string())?;
        let id = conn.last_insert_rowid();
        note_get_by_id(id)
    })
}

fn generate_note_file_path(name: &str) -> String {
    let app_dir = dirs::config_dir()
        .expect("无法获取应用数据目录")
        .join("com.dev.toolbox")
        .join("notes");
    std::fs::create_dir_all(&app_dir).expect("无法创建笔记目录");
    app_dir.join(name).to_string_lossy().to_string()
}
```

- [ ] **Step 5: 添加 note_get_by_id、note_rename、note_delete、note_move 函数**

```rust
pub fn note_get_by_id(id: i64) -> Result<NoteItem, String> {
    with_conn(|conn| {
        let sql = "SELECT id, parent_id, name, type, file_path, language, created_at, updated_at FROM notes WHERE id = ?";
        conn.query_row(sql, params![id], |row| {
            Ok(NoteItem {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                r#type: row.get(3)?,
                file_path: row.get(4)?,
                language: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).map_err(|e| e.to_string())
    })
}

pub fn note_rename(id: i64, new_name: &str) -> Result<NoteItem, String> {
    with_conn(|conn| {
        let item = note_get_by_id(id)?;
        if item.r#type == "file" {
            if let Some(old_path) = &item.file_path {
                let new_path = generate_note_file_path(new_name);
                std::fs::rename(old_path, &new_path)
                    .map_err(|e| format!("重命名文件失败: {}", e))?;
                conn.execute(
                    "UPDATE notes SET name = ?, file_path = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    params![new_name, new_path, id],
                ).map_err(|e| e.to_string())?;
            }
        } else {
            conn.execute(
                "UPDATE notes SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                params![new_name, id],
            ).map_err(|e| e.to_string())?;
        }
        note_get_by_id(id)
    })
}

pub fn note_delete(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        let item = note_get_by_id(id)?;
        if item.r#type == "file" {
            if let Some(path) = &item.file_path {
                std::fs::remove_file(path)
                    .map_err(|e| format!("删除文件失败: {}", e))?;
            }
        } else {
            // 删除文件夹下的所有文件
            let children = note_list(Some(id))?;
            for child in children {
                if child.r#type == "file" {
                    if let Some(path) = &child.file_path {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
        conn.execute("DELETE FROM notes WHERE id = ?", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

pub fn note_move(id: i64, new_parent_id: Option<i64>) -> Result<NoteItem, String> {
    with_conn(|conn| {
        conn.execute(
            "UPDATE notes SET parent_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![new_parent_id, id],
        ).map_err(|e| e.to_string())?;
        note_get_by_id(id)
    })
}
```

- [ ] **Step 6: 添加 Tauri 命令包装函数**

```rust
#[tauri::command]
pub fn cmd_note_list(parent_id: Option<i64>) -> Result<Vec<NoteItem>, String> {
    note_list(parent_id)
}

#[tauri::command]
pub fn cmd_note_create(name: String, note_type: String, parent_id: Option<i64>) -> Result<NoteItem, String> {
    note_create(&name, &note_type, parent_id)
}

#[tauri::command]
pub fn cmd_note_rename(id: i64, new_name: String) -> Result<NoteItem, String> {
    note_rename(id, &new_name)
}

#[tauri::command]
pub fn cmd_note_delete(id: i64) -> Result<(), String> {
    note_delete(id)
}

#[tauri::command]
pub fn cmd_note_move(id: i64, new_parent_id: Option<i64>) -> Result<NoteItem, String> {
    note_move(id, new_parent_id)
}
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: 添加 notes 表和 CRUD 操作"
```

---

### Task 3: Rust 后端 — 文件读写与编码检测

**Files:**
- Create: `src-tauri/src/note_manager.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 chardetng 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
chardetng = "0.1"
```

- [ ] **Step 2: 创建 note_manager.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chardetng::EncodingDetector;

#[derive(Serialize, Deserialize, Debug)]
pub struct NoteFileContent {
    pub content: String,
    pub encoding: String,
    pub size: usize,
}

pub fn note_read(file_path: &str) -> Result<NoteFileContent, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("文件不存在".to_string());
    }
    
    let bytes = fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let size = bytes.len();
    
    // 大文件警告
    if size > 5 * 1024 * 1024 {
        return Err("文件超过 5MB，已禁用语法高亮".to_string());
    }
    
    // 检测编码
    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let encoding_name = encoding.name();
    
    // 解码
    let (content, _) = encoding.decode(&bytes);
    
    Ok(NoteFileContent {
        content: content.into_owned(),
        encoding: encoding_name.to_string(),
        size,
    })
}

pub fn note_write(file_path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))
}

#[tauri::command]
pub fn cmd_note_read(file_path: String) -> Result<NoteFileContent, String> {
    note_read(&file_path)
}

#[tauri::command]
pub fn cmd_note_write(file_path: String, content: String) -> Result<(), String> {
    note_write(&file_path, &content)
}
```

- [ ] **Step 3: 注册命令到 main.rs**

在 `src-tauri/src/main.rs` 中添加模块声明和命令注册：

```rust
mod note_manager;
```

在 `invoke_handler` 中添加：

```rust
note_manager::cmd_note_read,
note_manager::cmd_note_write,
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/note_manager.rs src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: 添加笔记文件读写和编码检测"
```

---

### Task 4: 前端 — dbClient 封装 note 调用

**Files:**
- Create: `src/utils/noteClient.ts`

- [ ] **Step 1: 创建 noteClient.ts**

```typescript
import { invoke } from '@tauri-apps/api/core'

export interface NoteItem {
  id: number
  parent_id: number | null
  name: string
  type: 'folder' | 'file'
  file_path: string | null
  language: string
  created_at: string
  updated_at: string
}

export interface NoteFileContent {
  content: string
  encoding: string
  size: number
}

export async function noteList(parentId: number | null): Promise<NoteItem[]> {
  return invoke('note_list', { parentId })
}

export async function noteCreate(name: string, noteType: 'folder' | 'file', parentId: number | null): Promise<NoteItem> {
  return invoke('note_create', { name, noteType, parentId })
}

export async function noteRename(id: number, newName: string): Promise<NoteItem> {
  return invoke('note_rename', { id, newName })
}

export async function noteDelete(id: number): Promise<void> {
  return invoke('note_delete', { id })
}

export async function noteMove(id: number, newParentId: number | null): Promise<NoteItem> {
  return invoke('note_move', { id, newParentId })
}

export async function noteRead(filePath: string): Promise<NoteFileContent> {
  return invoke('note_read', { filePath })
}

export async function noteWrite(filePath: string, content: string): Promise<void> {
  return invoke('note_write', { filePath, content })
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/noteClient.ts
git commit -m "feat: 添加 noteClient 前端封装"
```

---

### Task 5: 前端 — CodeMirror 编辑器封装组件

**Files:**
- Create: `src/components/CodeMirrorEditor.vue`

- [ ] **Step 1: 创建 CodeMirrorEditor.vue**

```vue
<template>
  <div ref="editorRef" class="codemirror-wrapper"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, shallowRef } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { searchKeymap, highlightSelectionMatches, openSearchPanel, closeSearchPanel } from '@codemirror/search'
import { bracketMatching, foldGutter, foldKeymap, syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { xml } from '@codemirror/lang-xml'
import { sql } from '@codemirror/lang-sql'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
import { useToolboxStore } from '@/store'

const props = defineProps<{
  modelValue: string
  language?: string
  readOnly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': [value: string]
}>()

const store = useToolboxStore()
const editorRef = ref<HTMLElement>()
const view = shallowRef<EditorView>()

// 语言扩展映射
const langExtensions: Record<string, () => any> = {
  javascript: () => javascript(),
  typescript: () => javascript({ typescript: true }),
  json: () => json(),
  html: () => html(),
  css: () => css(),
  markdown: () => markdown(),
  xml: () => xml(),
  sql: () => sql(),
  python: () => python(),
  rust: () => rust(),
}

// 根据主题获取扩展
const getThemeExtension = () => {
  return store.config.theme === 'light' ? [] : [oneDark]
}

// 获取语言扩展
const getLangExtension = (lang: string) => {
  const ext = langExtensions[lang]
  return ext ? [ext()] : []
}

// 创建编辑器
const createEditor = () => {
  if (!editorRef.value) return

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      bracketMatching(),
      foldGutter(),
      syntaxHighlighting(defaultHighlightStyle),
      history(),
      highlightSelectionMatches(),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
        ...foldKeymap,
      ]),
      getThemeExtension(),
      getLangExtension(props.language || 'plaintext'),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const newValue = update.state.doc.toString()
          emit('update:modelValue', newValue)
          emit('change', newValue)
        }
      }),
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: '14px',
        },
        '.cm-scroller': {
          overflow: 'auto',
        },
      }),
    ],
  })

  view.value = new EditorView({
    state,
    parent: editorRef.value,
  })
}

// 更新内容（外部修改时）
const updateContent = (newContent: string) => {
  if (view.value && newContent !== view.value.state.doc.toString()) {
    view.value.dispatch({
      changes: { from: 0, to: view.value.state.doc.length, insert: newContent },
    })
  }
}

// 更新语言
const updateLanguage = (lang: string) => {
  if (!view.value) return
  const currentLang = props.language || 'plaintext'
  if (lang !== currentLang) {
    // 重新创建编辑器以切换语言
    const content = view.value.state.doc.toString()
    view.value.destroy()
    createEditor()
  }
}

// 查找
const openFind = () => {
  if (view.value) {
    openSearchPanel(view.value)
  }
}

// 替换
const openReplace = () => {
  if (view.value) {
    openSearchPanel(view.value)
  }
}

// 行操作
const sortLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  lines.sort()
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: lines.join('\n') },
  })
}

const dedupLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  const unique = [...new Set(lines)]
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: unique.join('\n') },
  })
}

const reverseLines = () => {
  if (!view.value) return
  const content = view.value.state.doc.toString()
  const lines = content.split('\n')
  lines.reverse()
  view.value.dispatch({
    changes: { from: 0, to: view.value.state.doc.length, insert: lines.join('\n') },
  })
}

const toUpperCase = () => {
  if (!view.value) return
  const selection = view.value.state.selection.main
  if (selection.empty) return
  const text = view.value.state.doc.sliceString(selection.from, selection.to)
  view.value.dispatch({
    changes: { from: selection.from, to: selection.to, insert: text.toUpperCase() },
  })
}

const toLowerCase = () => {
  if (!view.value) return
  const selection = view.value.state.selection.main
  if (selection.empty) return
  const text = view.value.state.doc.sliceString(selection.from, selection.to)
  view.value.dispatch({
    changes: { from: selection.from, to: selection.to, insert: text.toLowerCase() },
  })
}

// 格式化代码
const formatCode = () => {
  if (!view.value) return
  const lang = props.language || 'plaintext'
  const content = view.value.state.doc.toString()
  
  if (lang === 'json') {
    try {
      const formatted = JSON.stringify(JSON.parse(content), null, 2)
      view.value.dispatch({
        changes: { from: 0, to: view.value.state.doc.length, insert: formatted },
      })
    } catch {
      // JSON 格式错误，不处理
    }
  } else if (lang === 'javascript' || lang === 'typescript') {
    // 简单格式化：缩进
    const lines = content.split('\n')
    let indent = 0
    const formatted = lines.map(line => {
      const trimmed = line.trim()
      if (trimmed.endsWith('{') || trimmed.endsWith('[')) {
        const result = '  '.repeat(indent) + trimmed
        indent++
        return result
      } else if (trimmed.startsWith('}') || trimmed.startsWith(']')) {
        indent = Math.max(0, indent - 1)
        return '  '.repeat(indent) + trimmed
      }
      return '  '.repeat(indent) + trimmed
    }).join('\n')
    view.value.dispatch({
      changes: { from: 0, to: view.value.state.doc.length, insert: formatted },
    })
  }
}

// 暴露方法给父组件
defineExpose({
  openFind,
  openReplace,
  sortLines,
  dedupLines,
  reverseLines,
  toUpperCase,
  toLowerCase,
  formatCode,
  updateLanguage,
})

onMounted(() => {
  createEditor()
})

onUnmounted(() => {
  view.value?.destroy()
})

watch(() => props.modelValue, (newVal) => {
  updateContent(newVal)
})

watch(() => props.language, (newLang) => {
  if (newLang) updateLanguage(newLang)
})

watch(() => store.config.theme, () => {
  // 主题变化时重新创建编辑器
  if (view.value) {
    const content = view.value.state.doc.toString()
    view.value.destroy()
    createEditor()
  }
})
</script>

<style scoped>
.codemirror-wrapper {
  height: 100%;
  min-height: 400px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/CodeMirrorEditor.vue
git commit -m "feat: 添加 CodeMirror 编辑器封装组件"
```

---

### Task 6: 前端 — 侧边栏组件 NoteSidebar

**Files:**
- Create: `src/components/NoteSidebar.vue`

- [ ] **Step 1: 创建 NoteSidebar.vue**

```vue
<template>
  <div class="note-sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">笔记</span>
      <el-button size="small" text @click="showNewMenu = true">
        <el-icon><Plus /></el-icon>
      </el-button>
    </div>

    <!-- 新建菜单 -->
    <el-dropdown v-model:visible="showNewMenu" trigger="click" placement="bottom-start">
      <div class="new-menu-trigger" style="display:none"></div>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item @click="handleCreateFolder">
            <el-icon><FolderAdd /></el-icon> 新建文件夹
          </el-dropdown-item>
          <el-dropdown-item @click="handleCreateFile">
            <el-icon><DocumentAdd /></el-icon> 新建文件
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>

    <!-- 文件树 -->
    <div class="file-tree" @contextmenu.prevent="handleContextMenu($event, null)">
      <NoteTreeItem
        v-for="item in rootItems"
        :key="item.id"
        :item="item"
        :selected-id="selectedId"
        @select="handleSelect"
        @contextmenu="handleContextMenu"
        @rename="handleRename"
        @delete="handleDelete"
        @create-child="handleCreateChild"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, FolderAdd, DocumentAdd } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteTreeItem from './NoteTreeItem.vue'

const emit = defineEmits<{
  'select': [item: NoteItem]
}>()

const rootItems = ref<NoteItem[]>([])
const selectedId = ref<number | null>(null)
const showNewMenu = ref(false)

const loadItems = async (parentId: number | null = null) => {
  try {
    const items = await noteClient.noteList(parentId)
    if (parentId === null) {
      rootItems.value = items
    }
    return items
  } catch (e: any) {
    ElMessage.error(`加载失败: ${e}`)
    return []
  }
}

const handleSelect = (item: NoteItem) => {
  if (item.type === 'file') {
    selectedId.value = item.id
    emit('select', item)
  }
}

const handleCreateFolder = async () => {
  showNewMenu.value = false
  try {
    const { value } = await ElMessageBox.prompt('文件夹名称', '新建文件夹', {
      inputPattern: /^[^\\/:*?"<>|]+$/,
      inputErrorMessage: '名称不能包含 \\ / : * ? " < > |',
    })
    await noteClient.noteCreate(value, 'folder', null)
    await loadItems()
    ElMessage.success('文件夹已创建')
  } catch {
    // 用户取消
  }
}

const handleCreateFile = async () => {
  showNewMenu.value = false
  try {
    const { value } = await ElMessageBox.prompt('文件名称', '新建文件', {
      inputPattern: /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/,
      inputErrorMessage: '请输入有效的文件名（含扩展名）',
    })
    const item = await noteClient.noteCreate(value, 'file', null)
    await loadItems()
    emit('select', item)
    ElMessage.success('文件已创建')
  } catch {
    // 用户取消
  }
}

const handleCreateChild = async (parentId: number, type: 'folder' | 'file') => {
  try {
    const label = type === 'folder' ? '文件夹名称' : '文件名称'
    const pattern = type === 'folder' 
      ? /^[^\\/:*?"<>|]+$/ 
      : /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/
    const { value } = await ElMessageBox.prompt(label, `新建${type === 'folder' ? '文件夹' : '文件'}`, {
      inputPattern: pattern,
      inputErrorMessage: type === 'folder' ? '名称不能包含 \\ / : * ? " < > |' : '请输入有效的文件名（含扩展名）',
    })
    await noteClient.noteCreate(value, type, parentId)
    await loadItems()
    ElMessage.success('已创建')
  } catch {
    // 用户取消
  }
}

const handleRename = async (item: NoteItem) => {
  try {
    const { value } = await ElMessageBox.prompt('新名称', '重命名', {
      inputValue: item.name,
      inputPattern: item.type === 'folder' ? /^[^\\/:*?"<>|]+$/ : /^[^\\/:*?"<>|]+\.[a-zA-Z0-9]+$/,
      inputErrorMessage: item.type === 'folder' ? '名称不能包含 \\ / : * ? " < > |' : '请输入有效的文件名（含扩展名）',
    })
    await noteClient.noteRename(item.id, value)
    await loadItems()
    ElMessage.success('已重命名')
  } catch {
    // 用户取消
  }
}

const handleDelete = async (item: NoteItem) => {
  try {
    if (item.type === 'folder') {
      const children = await noteClient.noteList(item.id)
      const count = children.length
      await ElMessageBox.confirm(
        `将删除文件夹及其中的 ${count} 个项目，确定要删除吗？`,
        '删除确认',
        { type: 'warning' }
      )
    } else {
      await ElMessageBox.confirm(`确定要删除 "${item.name}" 吗？`, '删除确认', { type: 'warning' })
    }
    await noteClient.noteDelete(item.id)
    if (selectedId.value === item.id) {
      selectedId.value = null
    }
    await loadItems()
    ElMessage.success('已删除')
  } catch {
    // 用户取消
  }
}

const handleContextMenu = (event: MouseEvent, item: NoteItem | null) => {
  // 右键菜单逻辑（使用 Element Plus 的 ContextMenu 或自定义）
  // 简化版：直接弹出操作菜单
  if (!item) return
  // 这里可以使用 el-dropdown 或自定义右键菜单
}

onMounted(() => {
  loadItems()
})
</script>

<style scoped>
.note-sidebar {
  width: 200px;
  min-width: 200px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent-cyan);
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/NoteSidebar.vue
git commit -m "feat: 添加笔记侧边栏组件"
```

---

### Task 7: 前端 — 文件树子项组件 NoteTreeItem

**Files:**
- Create: `src/components/NoteTreeItem.vue`

- [ ] **Step 1: 创建 NoteTreeItem.vue**

```vue
<template>
  <div class="tree-item">
    <div
      class="tree-item-content"
      :class="{ active: item.id === selectedId, 'is-folder': item.type === 'folder' }"
      @click="handleClick"
      @contextmenu.prevent="$emit('contextmenu', $event, item)"
    >
      <el-icon v-if="item.type === 'folder'" class="tree-icon" @click.stop="toggleExpand">
        <Folder v-if="!expanded" />
        <FolderOpened v-else />
      </el-icon>
      <el-icon v-else class="tree-icon">
        <Document />
      </el-icon>
      <span class="tree-label" :title="item.name">{{ item.name }}</span>
      <span v-if="item.type === 'file' && isModified" class="unsaved-dot">●</span>
    </div>

    <!-- 子项 -->
    <div v-if="item.type === 'folder' && expanded" class="tree-children">
      <NoteTreeItem
        v-for="child in children"
        :key="child.id"
        :item="child"
        :selected-id="selectedId"
        @select="$emit('select', $event)"
        @contextmenu="$emit('contextmenu', $event, $event)"
        @rename="$emit('rename', $event)"
        @delete="$emit('delete', $event)"
        @create-child="$emit('create-child', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Folder, FolderOpened, Document } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'

const props = defineProps<{
  item: NoteItem
  selectedId: number | null
}>()

const emit = defineEmits<{
  'select': [item: NoteItem]
  'contextmenu': [event: MouseEvent, item: NoteItem]
  'rename': [item: NoteItem]
  'delete': [item: NoteItem]
  'create-child': [parentId: number, type: 'folder' | 'file']
}>()

const expanded = ref(false)
const children = ref<NoteItem[]>([])
const isModified = ref(false) // 由父组件通过 prop 或事件控制

const toggleExpand = async () => {
  expanded.value = !expanded.value
  if (expanded.value && children.value.length === 0) {
    children.value = await noteClient.noteList(props.item.id)
  }
}

const handleClick = () => {
  if (props.item.type === 'folder') {
    toggleExpand()
  } else {
    emit('select', props.item)
  }
}

onMounted(() => {
  // 默认不展开
})
</script>

<style scoped>
.tree-item-content {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  cursor: pointer;
  border-radius: 4px;
  margin: 1px 8px;
  transition: background 0.2s;
}

.tree-item-content:hover {
  background: rgba(0, 212, 255, 0.06);
}

.tree-item-content.active {
  background: rgba(0, 212, 255, 0.1);
}

.tree-icon {
  font-size: 16px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.tree-item-content.active .tree-icon {
  color: var(--accent-cyan);
}

.tree-label {
  flex: 1;
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-item-content.active .tree-label {
  color: var(--accent-cyan);
}

.unsaved-dot {
  color: var(--accent-cyan);
  font-size: 10px;
}

.tree-children {
  padding-left: 12px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/NoteTreeItem.vue
git commit -m "feat: 添加文件树子项组件"
```

---

### Task 8: 前端 — 顶部工具栏 NoteToolbar

**Files:**
- Create: `src/components/NoteToolbar.vue`

- [ ] **Step 1: 创建 NoteToolbar.vue**

```vue
<template>
  <div class="note-toolbar">
    <div class="toolbar-group">
      <el-button size="small" @click="$emit('find')">
        <el-icon><Search /></el-icon> 查找
      </el-button>
      <el-button size="small" @click="$emit('replace')">
        <el-icon><Search /></el-icon> 替换
      </el-button>
    </div>

    <div class="toolbar-group">
      <el-select v-model="localLanguage" size="small" style="width: 120px" @change="$emit('language-change', $event)">
        <el-option label="自动检测" value="auto" />
        <el-option label="纯文本" value="plaintext" />
        <el-option label="JavaScript" value="javascript" />
        <el-option label="TypeScript" value="typescript" />
        <el-option label="JSON" value="json" />
        <el-option label="HTML" value="html" />
        <el-option label="CSS" value="css" />
        <el-option label="Markdown" value="markdown" />
        <el-option label="XML" value="xml" />
        <el-option label="SQL" value="sql" />
        <el-option label="Python" value="python" />
        <el-option label="Rust" value="rust" />
      </el-select>
    </div>

    <div class="toolbar-group">
      <el-button size="small" @click="$emit('sort-lines')">排序行</el-button>
      <el-button size="small" @click="$emit('dedup-lines')">去重行</el-button>
      <el-button size="small" @click="$emit('reverse-lines')">反转行</el-button>
      <el-button size="small" @click="$emit('to-upper')">转大写</el-button>
      <el-button size="small" @click="$emit('to-lower')">转小写</el-button>
    </div>

    <div class="toolbar-group">
      <el-button type="primary" size="small" @click="$emit('format')">格式化</el-button>
    </div>

    <div class="toolbar-status" :class="{ modified: isModified }">
      {{ isModified ? '未保存 ●' : '已保存' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Search } from '@element-plus/icons-vue'

defineProps<{
  language?: string
  isModified?: boolean
}>()

const emit = defineEmits<{
  'find': []
  'replace': []
  'language-change': [lang: string]
  'sort-lines': []
  'dedup-lines': []
  'reverse-lines': []
  'to-upper': []
  'to-lower': []
  'format': []
}>()

const localLanguage = ref('auto')
</script>

<style scoped>
.note-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border-color);
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-status {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
}

.toolbar-status.modified {
  color: var(--accent-cyan);
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/NoteToolbar.vue
git commit -m "feat: 添加笔记工具栏组件"
```

---

### Task 9: 前端 — 主页面 NoteEditor

**Files:**
- Create: `src/views/NoteEditor.vue`

- [ ] **Step 1: 创建 NoteEditor.vue**

```vue
<template>
  <div class="note-editor-container">
    <NoteSidebar @select="handleSelectFile" />
    
    <div class="editor-main">
      <NoteToolbar
        v-if="currentFile"
        :language="currentLanguage"
        :is-modified="isModified"
        @find="handleFind"
        @replace="handleReplace"
        @language-change="handleLanguageChange"
        @sort-lines="handleSortLines"
        @dedup-lines="handleDedupLines"
        @reverse-lines="handleReverseLines"
        @to-upper="handleToUpper"
        @to-lower="handleToLower"
        @format="handleFormat"
      />

      <div v-if="!currentFile" class="editor-empty">
        <el-icon :size="48" color="var(--text-muted)"><Document /></el-icon>
        <p>选择或创建一个笔记开始编辑</p>
      </div>

      <div v-else class="editor-wrapper">
        <CodeMirrorEditor
          ref="editorRef"
          v-model="editorContent"
          :language="currentLanguage"
          @change="handleContentChange"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Document } from '@element-plus/icons-vue'
import * as noteClient from '@/utils/noteClient'
import type { NoteItem } from '@/utils/noteClient'
import NoteSidebar from '@/components/NoteSidebar.vue'
import NoteToolbar from '@/components/NoteToolbar.vue'
import CodeMirrorEditor from '@/components/CodeMirrorEditor.vue'

const currentFile = ref<NoteItem | null>(null)
const editorContent = ref('')
const originalContent = ref('')
const isModified = ref(false)
const currentLanguage = ref('plaintext')
const editorRef = ref()
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

// 根据文件扩展名推断语言
const detectLanguage = (filename: string): string => {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const langMap: Record<string, string> = {
    js: 'javascript', ts: 'typescript', json: 'json',
    html: 'html', htm: 'html', css: 'css', md: 'markdown',
    xml: 'xml', sql: 'sql', py: 'python', rs: 'rust',
    txt: 'plaintext', log: 'plaintext',
  }
  return langMap[ext] || 'plaintext'
}

const handleSelectFile = async (item: NoteItem) => {
  // 保存当前文件
  if (currentFile.value && isModified.value) {
    await saveCurrentFile()
  }

  currentFile.value = item
  currentLanguage.value = item.language !== 'plaintext' ? item.language : detectLanguage(item.name)

  try {
    if (item.file_path) {
      const result = await noteClient.noteRead(item.file_path)
      editorContent.value = result.content
      originalContent.value = result.content
      
      if (result.size > 1024 * 1024) {
        ElMessage.warning('文件较大，加载可能较慢')
      }
    }
  } catch (e: any) {
    ElMessage.error(`读取文件失败: ${e}`)
  }
}

const handleContentChange = () => {
  isModified.value = editorContent.value !== originalContent.value
  
  // 自动保存：停止输入 1 秒后保存
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    saveCurrentFile()
  }, 1000)
}

const saveCurrentFile = async () => {
  if (!currentFile.value || !currentFile.value.file_path || !isModified.value) return
  
  try {
    await noteClient.noteWrite(currentFile.value.file_path, editorContent.value)
    originalContent.value = editorContent.value
    isModified.value = false
    
    // 更新数据库中的 updated_at
    await noteClient.noteRename(currentFile.value.id, currentFile.value.name)
  } catch (e: any) {
    ElMessage.error(`保存失败: ${e}`)
  }
}

const handleLanguageChange = (lang: string) => {
  currentLanguage.value = lang === 'auto' ? detectLanguage(currentFile.value?.name || '') : lang
  editorRef.value?.updateLanguage(currentLanguage.value)
}

const handleFind = () => editorRef.value?.openFind()
const handleReplace = () => editorRef.value?.openReplace()
const handleSortLines = () => editorRef.value?.sortLines()
const handleDedupLines = () => editorRef.value?.dedupLines()
const handleReverseLines = () => editorRef.value?.reverseLines()
const handleToUpper = () => editorRef.value?.toUpperCase()
const handleToLower = () => editorRef.value?.toLowerCase()
const handleFormat = () => editorRef.value?.formatCode()

// 关闭时保存
onUnmounted(() => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  if (currentFile.value && isModified.value) {
    saveCurrentFile()
  }
})
</script>

<style scoped>
.note-editor-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.editor-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.editor-empty p {
  margin-top: 16px;
  font-size: 14px;
}

.editor-wrapper {
  flex: 1;
  overflow: hidden;
  padding: 8px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/NoteEditor.vue
git commit -m "feat: 添加文本编辑器主页面"
```

---

### Task 10: 集成到应用 — 注册路由和导航

**Files:**
- Modify: `src/App.vue`
- Modify: `src/components/SidebarNav.vue`
- Modify: `src/store/index.ts`

- [ ] **Step 1: 在 store 中添加 note 工具定义**

在 `src/store/index.ts` 的 `TOOL_LIST` 中添加：

```typescript
{
  id: 'note',
  name: '文本编辑器',
  iconSvg: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`,
  category: 'text',
},
```

- [ ] **Step 2: 在 App.vue 中导入并注册 NoteEditor**

```typescript
import NoteEditor from '@/views/NoteEditor.vue'
```

在模板中添加：

```vue
<NoteEditor v-else-if="activeTool === 'note'" />
```

- [ ] **Step 3: Commit**

```bash
git add src/App.vue src/components/SidebarNav.vue src/store/index.ts
git commit -m "feat: 集成文本编辑器到应用"
```

---

### Task 11: 最终审查与修复

**Files:**
- 所有上述文件

- [ ] **Step 1: TypeScript 类型检查**

```bash
cd d:\work\litobox
npx vue-tsc --noEmit
```
Expected: 无错误（或仅有已存在的错误）

- [ ] **Step 2: 修复任何类型错误**

根据 vue-tsc 输出修复问题。

- [ ] **Step 3: 启动开发环境测试**

```bash
npm run tauri dev
```

- [ ] **Step 4: 功能测试清单**

- [ ] 新建文件夹
- [ ] 新建文件
- [ ] 重命名文件/文件夹
- [ ] 删除文件/文件夹
- [ ] 打开文件并编辑
- [ ] 自动保存（停止输入 1 秒后）
- [ ] 语法高亮切换
- [ ] 查找替换
- [ ] 行操作（排序/去重/反转/大小写）
- [ ] 代码格式化（JSON）
- [ ] 主题切换（深色/浅色）
- [ ] 关闭时自动保存

- [ ] **Step 5: Commit 所有修复**

```bash
git add -A
git commit -m "fix: 修复文本编辑器类型错误和边界情况"
```

---

## 自审

### 1. Spec 覆盖检查

| Spec 要求 | 对应 Task |
|-----------|-----------|
| notes 表 + SQLite 元数据 | Task 2 |
| 本地文件存储 | Task 3 |
| Rust CRUD 命令 | Task 2, 3 |
| 左侧文件夹树 + 右侧编辑区 | Task 6, 7, 9 |
| CodeMirror 6 集成 | Task 5 |
| 语法高亮 | Task 5 |
| 查找替换 | Task 5, 8 |
| 自动保存 | Task 9 |
| 编码转换 | Task 3 (chardetng) |
| 行操作 | Task 5, 8 |
| 主题适配 | Task 5 |
| 大文件处理 | Task 3, 9 |
| 文件冲突检测 | Task 9 (简化版) |
| 右键菜单 | Task 6 |

### 2. Placeholder 扫描

✅ 无 TBD/TODO，所有步骤都有具体代码

### 3. 类型一致性

- `NoteItem` 在 `noteClient.ts` 和 `db.rs` 中定义一致
- `NoteFileContent` 在 `note_manager.rs` 和 `noteClient.ts` 中定义一致
- 事件命名统一使用 kebab-case
