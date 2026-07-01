# 文本编辑器设计文档

## 概述

为 LitoBox 添加一个基于 CodeMirror 6 的文本编辑器功能，定位为草稿本/便签，支持分类文件夹管理、语法高亮、查找替换、自动保存、编码转换、行操作等基础编辑能力。与现有 30+ 工具箱形成互补——工具箱做"处理"，编辑器做"创作"。

## 架构

- **前端**：Vue 3 + CodeMirror 6 + Element Plus，左侧文件夹树 + 右侧编辑区布局
- **后端**：Tauri Rust 命令处理文件系统操作，SQLite 存储元数据
- **存储**：混合模式 — SQLite 存元数据（标题、分类、更新时间），内容存为本地文件（`%APPDATA%/com.dev.toolbox/notes/`）

## 数据库设计

### notes 表

```sql
CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER DEFAULT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'file',
    file_path TEXT,
    language TEXT DEFAULT 'plaintext',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_notes_parent ON notes(parent_id);
```

### 文件存储路径

```
%APPDATA%/com.dev.toolbox/notes/
├── 工作/
│   ├── 会议记录.md
│   └── 需求草稿.txt
├── 学习/
│   ── Rust笔记.md
└── 未分类/
    └── 临时草稿.txt
```

## Rust 后端命令

| 命令 | 功能 |
|------|------|
| `note_list(parent_id)` | 获取指定文件夹下的子项 |
| `note_create(name, type, parent_id)` | 创建文件夹或文件 |
| `note_rename(id, new_name)` | 重命名 |
| `note_delete(id)` | 删除（文件夹级联删除） |
| `note_read(file_path)` | 读取文件内容 |
| `note_write(file_path, content)` | 写入文件内容 |
| `note_move(id, new_parent_id)` | 移动到其他文件夹 |

## UI 设计

### 整体布局

```
┌─────────────────────────────────────────────────────┐
│  📝 文本编辑器                              [新建]  │
├──────────┬──────────────────────────────────────────┤
│ 侧边栏    │           CodeMirror 编辑区              │
│ 200px    │                                          │
│          │  ┌────────────────────────────────────  │
│ 📁 工作   │  │  1 | function hello() {           │  │
│ 📁 学习   │  │  2 |   console.log("hi");         │  │
│ 📄 草稿   │  │  3 | }                            │  │
│          │  │                                    │  │
│          │  ────────────────────────────────────┘  │
│          │                                          │
│          │  [行号] [查找] [替换] [语法] [编码] [行操作]│
└──────────┴──────────────────────────────────────────┘
```

### 侧边栏交互

- **右键菜单**：新建文件夹/新建文件、重命名、删除、移动
- **拖拽排序**：支持拖拽文件到其他文件夹
- **点击文件**：在右侧编辑区打开
- **未保存提示**：文件有未保存修改时，文件名旁显示 `●` 标记

### 顶部工具栏

| 按钮 | 功能 |
|------|------|
| 查找 | `Ctrl+F` 打开查找面板 |
| 替换 | `Ctrl+H` 打开替换面板 |
| 语法 | 下拉选择语言（自动检测/手动切换） |
| 编码 | UTF-8 / GBK / Base64 切换 |
| 行操作 | 排序 / 去重 / 反转 / 转大写 / 转小写 |
| 格式化 | 根据语言自动格式化（JSON/JS/HTML 等） |

### 自动保存

- 停止输入 1 秒后自动写入文件
- 关闭应用时自动保存所有未保存文件
- 编辑区底部显示状态：`已保存` / `未保存 ●` / `保存中...`

## CodeMirror 6 集成

### 扩展选择

```typescript
import { EditorState, StateEffect } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { bracketMatching, foldGutter, foldKeymap } from '@codemirror/language'
import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
```

### 主题适配

- 深色模式：使用 `oneDark` 主题，与现有科技风一致
- 浅色模式：使用 `oneLight` 或自定义浅色主题
- 跟随系统主题切换时自动更新编辑器主题

### 语法高亮语言包

按需加载，不一次性全量引入：

```typescript
import { javascript } from '@codemirror/lang-javascript'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { xml } from '@codemirror/lang-xml'
import { sql } from '@codemirror/lang-sql'
import { python } from '@codemirror/lang-python'
import { rust } from '@codemirror/lang-rust'
```

### 多光标与选择

- CodeMirror 原生支持 `Ctrl+Click` 多光标
- `Ctrl+D` 选择下一个相同词
- `Alt+Click` 列选择模式

### 查找替换面板

- 使用 CodeMirror 内置 `@codemirror/search` 扩展
- 支持正则表达式、大小写敏感、全词匹配
- 支持"全部替换"和"逐个替换"

## 错误处理与边界情况

### 文件冲突处理

- 如果外部编辑器修改了同一文件，下次打开时提示"文件已被外部修改，是否重新加载？"
- 使用文件修改时间戳检测冲突

### 大文件处理

- 超过 1MB 的文件加载时提示"文件较大，加载可能较慢"
- CodeMirror 有虚拟滚动，理论上支持大文件，但语法高亮可能影响性能
- 超过 5MB 的文件禁用语法高亮，纯文本模式

### 非法字符处理

- 读取文件时检测编码，如果解码失败，提示"文件编码无法识别，尝试以二进制模式打开？"
- 使用 Rust 的 `chardetng` crate 自动检测编码

### 文件夹操作限制

- 文件夹名称不能包含 `\ / : * ? " < > |`
- 文件名不能为空
- 删除文件夹时二次确认，提示"将删除 X 个文件"

### 自动保存失败处理

- 如果写入文件失败（磁盘满、权限问题），编辑区顶部显示红色警告条
- 内容保留在内存中，不丢失，用户可以手动"另存为"到其他位置

### 快捷键冲突

- 编辑器快捷键与 Tauri 全局快捷键冲突时，编辑器内优先
- 使用 CodeMirror 的 `keymap` 系统，在编辑区聚焦时拦截
