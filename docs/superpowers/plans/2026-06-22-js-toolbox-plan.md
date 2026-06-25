# JS 工具箱 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 LitoBox 中新增 JS 工具箱模块，包含沙箱运行器（Rust boa_engine 后端）、代码格式化、代码压缩、JSON→代码生成四个子功能。

**Architecture:** 前端 Vue 多 Tab 页面 + Rust boa_engine 沙箱执行 + 纯前端工具函数。沙箱通过 Tauri invoke 调用 Rust 命令，boa_engine 隔离执行 JS 代码，捕获 console 日志和 return 值。

**Tech Stack:** Vue 3 Composition API, Tauri 2.0, boa_engine 0.21 (Rust JS 引擎), Element Plus

---

## 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/Cargo.toml` | Modify | 新增 boa_engine 依赖 |
| `src-tauri/src/js_executor.rs` | Create | Rust JS 执行引擎，boa_engine 封装 |
| `src-tauri/src/main.rs` | Modify | 注册 execute_js Tauri 命令 |
| `src/views/JSTool.vue` | Create | JS 工具箱主页面（4 个 Tab） |
| `src/utils/jsUtils.ts` | Create | 纯前端工具函数（格式化、压缩、JSON 生成） |
| `src/store/index.ts` | Modify | 新增 JS 工具箱工具注册 |
| `src/App.vue` | Modify | 路由集成 JSTool 组件 |

---

## Task 1: Rust 依赖 — boa_engine

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 boa_engine 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
boa_engine = "0.21"
```

- [ ] **Step 2: 验证依赖可编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过，boa_engine 下载并编译

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat(js): add boa_engine dependency"
```

---

## Task 2: Rust JS 执行引擎

**Files:**
- Create: `src-tauri/src/js_executor.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 创建 js_executor.rs 模块**

```rust
use boa_engine::{Context, JsResult, JsValue, NativeFunction, Source, js_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub success: bool,
    pub result: Option<String>,
    pub logs: Vec<LogEntry>,
    pub error: Option<String>,
    pub line: Option<usize>,
}

#[tauri::command]
pub fn execute_js(code: String, input: String, timeout_ms: u64) -> ExecuteResult {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = run_js(&code, &input);
        let _ = tx.send(result);
    });

    match rx.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(result) => result,
        Err(_) => ExecuteResult {
            success: false,
            result: None,
            logs: vec![],
            error: Some("执行超时（超过 5 秒）".to_string()),
            line: None,
        },
    }
}

fn run_js(code: &str, input_json: &str) -> ExecuteResult {
    let mut context = Context::default();
    let mut logs: Vec<LogEntry> = Vec::new();

    // 注入 console 对象
    let console = {
        let logs_ref = &mut logs;
        let log_fn = NativeFunction::from_fn_ptr(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_ref.push(LogEntry {
                level: "log".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        });

        let warn_fn = NativeFunction::from_fn_ptr(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_ref.push(LogEntry {
                level: "warn".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        });

        let error_fn = NativeFunction::from_fn_ptr(move |_this, args, _ctx| {
            let msg = args
                .iter()
                .map(|v| v.display().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            logs_ref.push(LogEntry {
                level: "error".to_string(),
                message: msg,
            });
            Ok(JsValue::undefined())
        });

        boa_engine::object::ObjectInitializer::new(&mut context)
            .function(log_fn, js_string!("log"), 0)
            .function(warn_fn, js_string!("warn"), 0)
            .function(error_fn, js_string!("error"), 0)
            .build()
    };

    context
        .register_global_property(js_string!("console"), console, boa_engine::property::Attribute::all())
        .unwrap();

    // 注入 input 全局变量
    if !input_json.trim().is_empty() {
        match serde_json::from_str::<serde_json::Value>(input_json) {
            Ok(json_val) => {
                let js_val = json_to_js_value(&json_val, &mut context);
                context
                    .register_global_property(js_string!("input"), js_val, boa_engine::property::Attribute::all())
                    .unwrap();
            }
            Err(e) => {
                return ExecuteResult {
                    success: false,
                    result: None,
                    logs,
                    error: Some(format!("入参 JSON 格式错误: {}", e)),
                    line: None,
                };
            }
        }
    }

    // 执行代码
    match context.eval(Source::from_bytes(code)) {
        Ok(result) => ExecuteResult {
            success: true,
            result: Some(result.display().to_string()),
            logs,
            error: None,
            line: None,
        },
        Err(e) => {
            let error_msg = e.to_string();
            // 尝试提取行号
            let line = extract_line_number(&error_msg);
            ExecuteResult {
                success: false,
                result: None,
                logs,
                error: Some(error_msg),
                line,
            }
        }
    }
}

fn json_to_js_value(json: &serde_json::Value, context: &mut Context) -> JsValue {
    match json {
        serde_json::Value::Null => JsValue::null(),
        serde_json::Value::Bool(b) => JsValue::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::from(i)
            } else if let Some(f) = n.as_f64() {
                JsValue::from(f)
            } else {
                JsValue::null()
            }
        }
        serde_json::Value::String(s) => JsValue::from(js_string!(s.as_str())),
        serde_json::Value::Array(arr) => {
            let js_arr = boa_engine::object::ArrayBuilder::new(context);
            let js_arr = arr.iter().fold(js_arr, |builder, item| {
                builder.push(json_to_js_value(item, context))
            });
            js_arr.build().into()
        }
        serde_json::Value::Object(obj) => {
            let js_obj = boa_engine::object::ObjectInitializer::new(context);
            let js_obj = obj.iter().fold(js_obj, |builder, (key, value)| {
                builder.property(
                    js_string!(key.as_str()),
                    json_to_js_value(value, context),
                    boa_engine::property::Attribute::all(),
                )
            });
            js_obj.build().into()
        }
    }
}

fn extract_line_number(error_msg: &str) -> Option<usize> {
    // boa 错误格式: "ReferenceError: xxx is not defined\n  at <anonymous>:3:1"
    for line in error_msg.lines() {
        if line.contains(":") && line.contains("at") {
            if let Some(pos) = line.rfind(':') {
                if let Ok(num) = line[pos + 1..].trim().parse::<usize>() {
                    return Some(num);
                }
            }
        }
    }
    None
}
```

- [ ] **Step 2: 在 main.rs 中注册模块和命令**

在 `src-tauri/src/main.rs` 中添加：

```rust
mod js_executor;
```

在 `.invoke_handler()` 中添加：

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing handlers ...
    js_executor::execute_js,
])
```

- [ ] **Step 3: 验证 Rust 编译**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/js_executor.rs src-tauri/src/main.rs
git commit -m "feat(js): add Rust JS executor with boa_engine"
```

---

## Task 3: 纯前端工具函数

**Files:**
- Create: `src/utils/jsUtils.ts`

- [ ] **Step 1: 创建 jsUtils.ts**

```typescript
// JS 代码格式化
export interface JsFormatOptions {
  indent: 2 | 4
  semicolons: boolean
  quotes: 'single' | 'double'
  maxWidth: 80 | 100 | 120
}

export function formatJs(code: string, options: JsFormatOptions): string {
  if (!code.trim()) return ''

  const indentStr = ' '.repeat(options.indent)
  const lines = code.split('\n')
  const result: string[] = []
  let indentLevel = 0

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed) {
      result.push('')
      continue
    }

    // 减少缩进：} 或 ]; 或 ); 开头的行
    if (/^[}\]);]/.test(trimmed)) {
      indentLevel = Math.max(0, indentLevel - 1)
    }

    result.push(indentStr.repeat(indentLevel) + trimmed)

    // 增加缩进：{ 或 [ 或 ( 结尾的行
    if (/[{\[(]$/.test(trimmed.replace(/\/\/.*$/, '').replace(/\/\*.*\*\//, ''))) {
      indentLevel++
    }
  }

  let formatted = result.join('\n')

  // 处理分号
  if (!options.semicolons) {
    formatted = formatted.replace(/;$/gm, '')
  }

  // 处理引号
  if (options.quotes === 'single') {
    formatted = formatted.replace(/"(?![^"]*'[^\"]*"[^"]*')/g, "'")
  } else {
    formatted = formatted.replace(/'(?![^']*"[^']*'[^']*')/g, '"')
  }

  return formatted
}

// JS 代码压缩
export interface JsCompressOptions {
  mangle: boolean
  keepComments: boolean
  esVersion: 'es5' | 'es6'
}

export function compressJs(code: string, options: JsCompressOptions): string {
  if (!code.trim()) return ''

  let result = code

  // 移除注释
  if (!options.keepComments) {
    // 移除多行注释
    result = result.replace(/\/\*[\s\S]*?\*\//g, '')
    // 移除单行注释
    result = result.replace(/\/\/.*$/gm, '')
  }

  // 移除多余空白
  result = result.replace(/\s+/g, ' ')
  result = result.replace(/\s*([{}();,=<>!+\-*/&|?:])\s*/g, '$1')
  result = result.trim()

  // 变量名混淆（简单实现：将局部变量名缩短为 a, b, c...）
  if (options.mangle) {
    result = mangleVariables(result)
  }

  return result
}

function mangleVariables(code: string): string {
  // 简单混淆：匹配 let/const/var 声明的变量名，替换为短名
  const varNames = new Set<string>()
  const declRegex = /(?:let|const|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)/g
  let match
  while ((match = declRegex.exec(code)) !== null) {
    varNames.add(match[1])
  }

  // 按长度排序，长的先替换
  const sorted = Array.from(varNames).sort((a, b) => b.length - a.length)
  const shortNames = 'abcdefghijklmnopqrstuvwxyz'
  let result = code

  sorted.forEach((name, i) => {
    if (name.length > 1) {
      const shortName = shortNames[i % 26]
      const regex = new RegExp(`\\b${name}\\b`, 'g')
      result = result.replace(regex, shortName)
    }
  })

  return result
}

// JSON → 解构代码
export function jsonToDestruct(json: string): string {
  let obj: any
  try {
    obj = JSON.parse(json)
  } catch {
    return '// JSON 格式错误'
  }

  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    return '// 请输入 JSON 对象'
  }

  const keys = Object.keys(obj)
  if (keys.length === 0) return 'const {} = data'

  const destructKeys = keys.map(k => {
    const validKey = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/.test(k) ? k : `'${k}'`
    return validKey
  }).join(', ')

  return `const { ${destructKeys} } = data`
}

// JSON → TS 类型声明
export function jsonToInterface(json: string, interfaceName: string = 'Data'): string {
  let obj: any
  try {
    obj = JSON.parse(json)
  } catch {
    return '// JSON 格式错误'
  }

  return `interface ${interfaceName} {\n${generateInterfaceBody(obj, '  ')}\n}`
}

function generateInterfaceBody(obj: any, indent: string): string {
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    return indent + '// 非对象类型'
  }

  const lines: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const validKey = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/.test(key) ? key : `'${key}'`
    const type = getTsType(value)
    lines.push(`${indent}${validKey}: ${type};`)
  }

  return lines.join('\n')
}

function getTsType(value: any): string {
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  if (typeof value === 'boolean') return 'boolean'
  if (typeof value === 'number') return 'number'
  if (typeof value === 'string') return 'string'
  if (Array.isArray(value)) {
    if (value.length === 0) return 'any[]'
    const elemType = getTsType(value[0])
    return `${elemType}[]`
  }
  if (typeof value === 'object') {
    return `{\n${generateInterfaceBody(value, '  ')}\n}`
  }
  return 'any'
}

// JSON → 默认值模板
export function jsonToDefaultTemplate(json: string): string {
  let obj: any
  try {
    obj = JSON.parse(json)
  } catch {
    return '// JSON 格式错误'
  }

  return `const defaultData = ${JSON.stringify(obj, null, 2)};`
}
```

- [ ] **Step 2: Commit**

```bash
git add src/utils/jsUtils.ts
git commit -m "feat(js): add frontend utility functions"
```

---

## Task 4: JSTool.vue 页面

**Files:**
- Create: `src/views/JSTool.vue`

- [ ] **Step 1: 创建 JSTool.vue 页面**

页面包含 4 个 Tab，遵循现有 SQL 工具的多 Tab 模式。完整代码见下方。

```vue
<template>
  <div class="tool-container">
    <el-tabs v-model="activeTab" class="js-tabs">
      <!-- Tab 1: 沙箱运行器 -->
      <el-tab-pane label="沙箱运行器" name="sandbox">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">超时 (秒)</div>
                <el-input-number v-model="timeout" :min="1" :max="10" size="small" style="width: 100px" />
              </div>
              <div class="action-group">
                <div class="group-label">入参 JSON</div>
                <el-button size="small" @click="showInputPanel = !showInputPanel">
                  {{ showInputPanel ? '隐藏' : '设置入参' }}
                </el-button>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleExecute">执行</el-button>
              </div>
            </div>
          </div>
        </div>

        <!-- 入参面板 -->
        <div v-if="showInputPanel" class="tool-card">
          <div class="card-header">
            <span class="card-title">入参 (JSON)</span>
            <el-button size="small" @click="handleClearInput">清空</el-button>
          </div>
          <div class="card-body">
            <el-input v-model="inputJson" type="textarea" :rows="4" placeholder='{"users": [{"name": "张三"}]}' resize="vertical" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">代码</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClearCode">清空</el-button>
              <el-button size="small" @click="handlePasteCode">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="code" type="textarea" :rows="8" placeholder="const result = input.users.map(u => u.name);&#10;console.log(result);&#10;return result;" resize="vertical" class="code-editor" />
          </div>
        </div>

        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出结果</span>
            <el-button size="small" @click="handleCopyResult">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="result" type="textarea" :rows="4" readonly resize="vertical" :class="{ 'error': isError }" />
            <div v-if="errorMsg" class="error-message">{{ errorMsg }}</div>
          </div>
        </div>

        <div v-if="logs.length > 0" class="tool-card">
          <div class="card-header">
            <span class="card-title">日志 ({{ logs.length }})</span>
            <el-button size="small" @click="logs = []">清空</el-button>
          </div>
          <div class="card-body">
            <div v-for="(log, idx) in logs" :key="idx" class="log-entry" :class="log.level">
              <span class="log-level">[{{ log.level.toUpperCase() }}]</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 2: 代码格式化 -->
      <el-tab-pane label="代码格式化" name="format">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">缩进</div>
                <el-radio-group v-model="formatOptions.indent" size="small">
                  <el-radio-button :label="2">2空格</el-radio-button>
                  <el-radio-button :label="4">4空格</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">分号</div>
                <el-radio-group v-model="formatOptions.semicolons" size="small">
                  <el-radio-button :label="true">保留</el-radio-button>
                  <el-radio-button :label="false">移除</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">引号</div>
                <el-radio-group v-model="formatOptions.quotes" size="small">
                  <el-radio-button label="single">单引号</el-radio-button>
                  <el-radio-button label="double">双引号</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleFormat">格式化</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClearFormat">清空</el-button>
              <el-button size="small" @click="handlePasteFormat">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="formatInput" type="textarea" :rows="8" placeholder="请输入JS代码..." resize="vertical" class="code-editor" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopyFormat">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="formatOutput" type="textarea" :rows="8" readonly resize="vertical" class="code-editor" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 3: 代码压缩 -->
      <el-tab-pane label="代码压缩" name="compress">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">变量混淆</div>
                <el-switch v-model="compressOptions.mangle" size="small" />
              </div>
              <div class="action-group">
                <div class="group-label">保留注释</div>
                <el-switch v-model="compressOptions.keepComments" size="small" />
              </div>
              <div class="action-group">
                <div class="group-label">ES版本</div>
                <el-radio-group v-model="compressOptions.esVersion" size="small">
                  <el-radio-button label="es5">ES5</el-radio-button>
                  <el-radio-button label="es6">ES6</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleCompress">压缩</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClearCompress">清空</el-button>
              <el-button size="small" @click="handlePasteCompress">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="compressInput" type="textarea" :rows="8" placeholder="请输入JS代码..." resize="vertical" class="code-editor" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopyCompress">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="compressOutput" type="textarea" :rows="8" readonly resize="vertical" class="code-editor" />
          </div>
        </div>
      </el-tab-pane>

      <!-- Tab 4: JSON→代码 -->
      <el-tab-pane label="JSON→代码" name="jsonCode">
        <div class="tool-card sticky-card">
          <div class="card-header">
            <span class="card-title">操作</span>
          </div>
          <div class="card-body">
            <div class="action-grid">
              <div class="action-group">
                <div class="group-label">生成类型</div>
                <el-radio-group v-model="jsonCodeMode" size="small">
                  <el-radio-button label="destruct">解构代码</el-radio-button>
                  <el-radio-button label="interface">TS类型</el-radio-button>
                  <el-radio-button label="template">默认值模板</el-radio-button>
                </el-radio-group>
              </div>
              <div class="action-group">
                <div class="group-label">接口名</div>
                <el-input v-if="jsonCodeMode === 'interface'" v-model="interfaceName" placeholder="Data" size="small" style="width: 120px" />
              </div>
              <div class="action-group">
                <div class="group-label">执行</div>
                <el-button type="primary" size="small" @click="handleJsonToCode">生成</el-button>
              </div>
            </div>
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输入 (JSON)</span>
            <div class="card-actions">
              <el-button size="small" @click="handleClearJsonCode">清空</el-button>
              <el-button size="small" @click="handlePasteJsonCode">粘贴</el-button>
            </div>
          </div>
          <div class="card-body">
            <el-input v-model="jsonCodeInput" type="textarea" :rows="8" placeholder='{"name": "张三", "age": 25}' resize="vertical" />
          </div>
        </div>
        <div class="tool-card">
          <div class="card-header">
            <span class="card-title">输出</span>
            <el-button size="small" @click="handleCopyJsonCode">复制</el-button>
          </div>
          <div class="card-body">
            <el-input :model-value="jsonCodeOutput" type="textarea" :rows="8" readonly resize="vertical" class="code-editor" />
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { formatJs, compressJs, jsonToDestruct, jsonToInterface, jsonToDefaultTemplate, type JsFormatOptions, type JsCompressOptions } from '@/utils/jsUtils'
import { useToolboxStore } from '@/store'

const store = useToolboxStore()

// Tab 状态
const activeTab = ref('sandbox')

// ===== Tab 1: 沙箱运行器 =====
const code = ref('')
const inputJson = ref('')
const result = ref('')
const errorMsg = ref('')
const isError = ref(false)
const logs = ref<{ level: string; message: string }[]>([])
const timeout = ref(5)
const showInputPanel = ref(false)

const handleExecute = async () => {
  if (!code.value.trim()) {
    ElMessage.warning('请输入代码')
    return
  }

  // 验证入参 JSON
  if (inputJson.value.trim() && inputJson.value.trim() !== '{}') {
    try {
      JSON.parse(inputJson.value)
    } catch {
      ElMessage.error('入参 JSON 格式错误')
      return
    }
  }

  logs.value = []
  errorMsg.value = ''
  isError.value = false
  result.value = ''

  try {
    const res = await invoke('execute_js', {
      code: code.value,
      input: inputJson.value.trim() || '{}',
      timeoutMs: timeout.value * 1000
    }) as { success: boolean; result?: string; logs?: { level: string; message: string }[]; error?: string; line?: number }

    if (res.success) {
      result.value = res.result || ''
      logs.value = res.logs || []
      ElMessage.success('执行成功')
    } else {
      errorMsg.value = res.error || '执行失败'
      isError.value = true
      logs.value = res.logs || []
      if (res.line) {
        errorMsg.value += ` (第 ${res.line} 行)`
      }
    }
  } catch (err: any) {
    errorMsg.value = `调用失败: ${err.message || err}`
    isError.value = true
  }

  store.addHistory({
    tool: 'js',
    action: '沙箱执行',
    inputPreview: code.value.slice(0, 50),
    outputPreview: result.value.slice(0, 50)
  })
}

const handleClearCode = () => {
  code.value = ''
  result.value = ''
  errorMsg.value = ''
  isError.value = false
  logs.value = []
}

const handleClearInput = () => {
  inputJson.value = ''
}

const handlePasteCode = async () => {
  try {
    const text = await navigator.clipboard.readText()
    code.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败')
  }
}

const handleCopyResult = async () => {
  const text = result.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ===== Tab 2: 代码格式化 =====
const formatInput = ref('')
const formatOutput = ref('')
const formatOptions = reactive<JsFormatOptions>({
  indent: 2,
  semicolons: true,
  quotes: 'single',
  maxWidth: 80
})

const handleFormat = () => {
  if (!formatInput.value.trim()) {
    ElMessage.warning('请输入代码')
    return
  }
  formatOutput.value = formatJs(formatInput.value, formatOptions)
  ElMessage.success('格式化成功')
  store.addHistory({
    tool: 'js',
    action: '代码格式化',
    inputPreview: formatInput.value.slice(0, 50),
    outputPreview: formatOutput.value.slice(0, 50)
  })
}

const handleClearFormat = () => {
  formatInput.value = ''
  formatOutput.value = ''
}

const handlePasteFormat = async () => {
  try {
    const text = await navigator.clipboard.readText()
    formatInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败')
  }
}

const handleCopyFormat = async () => {
  const text = formatOutput.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ===== Tab 3: 代码压缩 =====
const compressInput = ref('')
const compressOutput = ref('')
const compressOptions = reactive<JsCompressOptions>({
  mangle: false,
  keepComments: false,
  esVersion: 'es6'
})

const handleCompress = () => {
  if (!compressInput.value.trim()) {
    ElMessage.warning('请输入代码')
    return
  }
  compressOutput.value = compressJs(compressInput.value, compressOptions)
  ElMessage.success('压缩成功')
  store.addHistory({
    tool: 'js',
    action: '代码压缩',
    inputPreview: compressInput.value.slice(0, 50),
    outputPreview: compressOutput.value.slice(0, 50)
  })
}

const handleClearCompress = () => {
  compressInput.value = ''
  compressOutput.value = ''
}

const handlePasteCompress = async () => {
  try {
    const text = await navigator.clipboard.readText()
    compressInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败')
  }
}

const handleCopyCompress = async () => {
  const text = compressOutput.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

// ===== Tab 4: JSON→代码 =====
const jsonCodeInput = ref('')
const jsonCodeOutput = ref('')
const jsonCodeMode = ref<'destruct' | 'interface' | 'template'>('destruct')
const interfaceName = ref('Data')

const handleJsonToCode = () => {
  if (!jsonCodeInput.value.trim()) {
    ElMessage.warning('请输入 JSON')
    return
  }
  switch (jsonCodeMode.value) {
    case 'destruct':
      jsonCodeOutput.value = jsonToDestruct(jsonCodeInput.value)
      break
    case 'interface':
      jsonCodeOutput.value = jsonToInterface(jsonCodeInput.value, interfaceName.value)
      break
    case 'template':
      jsonCodeOutput.value = jsonToDefaultTemplate(jsonCodeInput.value)
      break
  }
  ElMessage.success('生成成功')
  store.addHistory({
    tool: 'js',
    action: 'JSON→代码',
    inputPreview: jsonCodeInput.value.slice(0, 50),
    outputPreview: jsonCodeOutput.value.slice(0, 50)
  })
}

const handleClearJsonCode = () => {
  jsonCodeInput.value = ''
  jsonCodeOutput.value = ''
}

const handlePasteJsonCode = async () => {
  try {
    const text = await navigator.clipboard.readText()
    jsonCodeInput.value = text
    ElMessage.success('粘贴成功')
  } catch {
    ElMessage.error('粘贴失败')
  }
}

const handleCopyJsonCode = async () => {
  const text = jsonCodeOutput.value
  if (!text) {
    ElMessage.warning('没有可复制的内容')
    return
  }
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('复制成功')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 监听代码变化，粘贴后自动执行（沙箱 Tab）
let autoExecTimer: ReturnType<typeof setTimeout> | null = null
watch(
  () => code.value,
  (value) => {
    if (!value.trim()) {
      result.value = ''
      errorMsg.value = ''
      isError.value = false
      return
    }
    if (autoExecTimer) clearTimeout(autoExecTimer)
    autoExecTimer = setTimeout(() => {
      handleExecute()
    }, 500)
  }
)
</script>

<style scoped>
.js-tabs :deep(.el-tabs__header) {
  margin-bottom: 16px;
  padding-left: 8px;
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.js-tabs :deep(.el-tabs__nav-wrap) {
  padding-left: 4px;
}

.tool-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 16px;
  overflow: hidden;
  transition: border-color 0.3s;
}

.sticky-card {
  position: sticky;
  top: 0;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tool-card:hover {
  border-color: rgba(0, 212, 255, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-bottom: 1px solid var(--border-color);
}

.card-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--accent-cyan);
  text-transform: uppercase;
  letter-spacing: 1px;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  padding: 20px;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: flex-end;
}

.action-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-label {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.code-editor :deep(.el-textarea__inner) {
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.error :deep(.el-textarea__inner) {
  border-color: var(--accent-red) !important;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.2) !important;
}

.error-message {
  color: var(--accent-red);
  font-size: 12px;
  margin-top: 10px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
}

.log-entry {
  display: flex;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 3px;
  font-size: 12px;
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  margin-bottom: 2px;
}

.log-entry.log {
  background: rgba(0, 212, 255, 0.05);
  color: var(--text-primary);
}

.log-entry.warn {
  background: rgba(234, 179, 8, 0.08);
  color: #eab308;
}

.log-entry.error {
  background: rgba(239, 68, 68, 0.08);
  color: var(--accent-red);
}

.log-level {
  font-weight: 600;
  white-space: nowrap;
  min-width: 50px;
}

.log-message {
  word-break: break-all;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/JSTool.vue
git commit -m "feat(js): add JSTool.vue page with 4 tabs"
```

---

## Task 5: Store 和路由集成

**Files:**
- Modify: `src/store/index.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 在 store/index.ts 中注册 JS 工具箱**

在 `TOOL_LIST` 数组中添加：

```typescript
{ id: 'js', name: 'JS工具', icon: 'JS', iconSvg: `<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V6a2 2 0 00-2-2z"/><path d="M8 16v-4M12 16v-6M16 16v-3"/></svg>`, description: 'JS沙箱运行、格式化、压缩、JSON生成代码', keywords: ['js', 'javascript', '沙箱', '格式化', '压缩'] },
```

在 `shortcuts` 中添加：

```typescript
js: 'CmdOrCtrl+Alt+Y'
```

- [ ] **Step 2: 在 App.vue 中集成路由**

在 import 区域添加：

```typescript
import JSTool from '@/views/JSTool.vue'
```

在 main 区域添加：

```vue
<JSTool v-else-if="activeTool === 'js'" />
```

- [ ] **Step 3: Commit**

```bash
git add src/store/index.ts src/App.vue
git commit -m "feat(js): register JS tool in store and router"
```

---

## Task 6: 端到端验证

- [ ] **Step 1: 启动开发服务器**

Run: `npm run tauri dev`
Expected: Tauri 窗口打开，侧边栏显示 "JS工具" 入口

- [ ] **Step 2: 测试沙箱运行器**

在沙箱 Tab 输入：
```js
const result = input.users.map(u => u.name);
console.log(result);
return result;
```
入参 JSON 输入：`{"users": [{"name": "张三"}, {"name": "李四"}]}`
点击执行，Expected: 输出 `["张三", "李四"]`，日志面板显示 `[LOG] ["张三", "李四"]`

- [ ] **Step 3: 测试代码格式化**

在格式化 Tab 输入压缩的 JS 代码，点击格式化，Expected: 代码按缩进和换行格式化

- [ ] **Step 4: 测试代码压缩**

在压缩 Tab 输入正常 JS 代码，开启变量混淆，点击压缩，Expected: 代码被压缩为一行，变量名缩短

- [ ] **Step 5: 测试 JSON→代码**

在 JSON→代码 Tab 输入 `{"name": "张三", "age": 25, "active": true}`，分别测试三种模式，Expected: 生成对应的解构/类型/模板代码

- [ ] **Step 6: Commit**

```bash
git add .
git commit -m "feat(js): JS toolbox complete"
```
