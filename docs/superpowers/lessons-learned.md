# 开发经验总结

> 记录每次功能开发中踩过的坑、提炼的最佳实践，让每一次都比上一次更好。

---

## 2026-07-20 视频工具增强（变速/旋转/音量）

### 🏆 本次最佳实践

#### 1. ffmpeg 进度条正确实现方式

**错误做法**：读取 stderr，解析 `frame=... time=HH:MM:SS` 行

```rust
// ❌ 错误：ffmpeg 用 \r 刷新同一行，BufReader::lines() 按 \n 分割会堆积
// 结果：进度条从 0% 直接跳到 100%
let reader = BufReader::new(stderr);
for line in reader.lines() {
    // 永远只有一行（直到进程退出），因为进度更新用 \r 而不是 \n
}
```

**正确做法**：用 `-progress pipe:1` 输出结构化进度到 stdout

```rust
// ✅ 正确：-progress pipe:1 每行用 \n 分隔，可实时逐行读取
// 在输出路径前插入参数：
let mut full_args = args[..args.len()-1].to_vec();
full_args.push("-progress".into());
full_args.push("pipe:1".into());
full_args.push("-nostats".into()); // 禁用 stderr 的默认进度输出
full_args.push(output_path);

// 解析 stdout 中的 out_time_us=（微秒）
for line in BufReader::new(stdout).lines() {
    if let Some(rest) = line?.strip_prefix("out_time_us=") {
        let us = rest.trim().parse::<f64>()?;
        let progress = (us / 1_000_000.0 / duration * 100.0).min(99.9);
    }
}
```

**为什么 out_time_us 比 time= 更好**：
- `out_time_us=1234567` 纯数字，直接 parse 就行
- `time=00:00:05.12` 需要自己拆分时分秒解析
- 微秒精度更高

#### 2. 管道死锁防护

**问题**：同时捕获 stdout 和 stderr 时，如果只读取其中一个，另一个的管道缓冲区满了（Windows 默认 4KB），ffmpeg 会阻塞写入，导致整个进程卡住。

**解决方案**：用独立线程读取其中一个流

```rust
// stderr 用独立线程读完，避免缓冲区满导致死锁
let stderr_handle = std::thread::spawn(move || {
    let mut buf = Vec::new();
    BufReader::new(stderr).read_to_end(&mut buf);
    buf
});

// 主线程读 stdout 处理进度
for line in BufReader::new(stdout).lines() { ... }

// 最后 join 拿到 stderr 内容（用于报错）
let stderr_buf = stderr_handle.join().unwrap_or_default();
```

**适用场景**：任何同时读取子进程 stdout + stderr 的场景，都要考虑这个问题。

#### 3. 视频处理性能分级优化

| 操作 | 视频流 | 音频流 | 速度 |
|------|--------|--------|------|
| 音量调整 | `-c:v copy`（直接复制） | 重编码 | ⚡ 极快（接近复制速度） |
| 静音 | `-c:v copy` | `-an`（移除） | ⚡⚡ 超快（纯流复制） |
| 旋转/变速 | 重编码 + `-preset fast` | copy / 重编码 | 🐢 较慢（必须重编码视频） |

**经验法则**：
- 能 copy 流就 copy，不要重编码
- 必须重编码时，工具类应用用 `-preset fast` 而非默认 `medium`（快 30-50%，画质损失可接受）
- `-preset ultrafast` 速度更快但体积大，适合临时文件

---

### 🐛 踩过的坑

#### 坑 1：Rust 结构体字段不会自动转 camelCase

**现象**：前端传 `keepPitch: true`，后端报错 `missing field 'keep_pitch'`

**规则回顾**（AGENTS.md 第 16 条）：
- **命令函数参数**：Rust snake_case → 前端 camelCase（自动转换）
- **结构体字段**：**不会自动转换**，前端必须传完全一致的 snake_case

```rust
// 命令参数：自动转换，前端传 timePoint
pub async fn some_command(time_point: f64) -> Result<(), String> { ... }

// 结构体字段：不自动转换，前端必须传 keep_pitch
pub struct VideoSpeedOptions {
    pub keep_pitch: bool,  // 前端也要传 keep_pitch，不是 keepPitch
}
```

**怎么避免**：写 invoke 调用时，只要参数是个 object/结构体，就检查一下 Rust 端的字段定义，全部用 snake_case。

#### 坑 2：历史记录用 options 不用 detail

**现象**：TypeScript 报错 `Object literal may only specify known properties, and 'detail' does not exist`

**正确写法**：
```typescript
store.addHistory({
  tool: 'videoTool',
  action: '变速',
  inputPreview: '...',
  outputPreview: '...',
  inputFull: '...',
  outputFull: '...',
  options: { speed: 2.0, keepPitch: true },  // ✅ 用 options，不是 detail
})
```

---

### 💡 设计模式

#### 同类功能整合模式

视频工具里有很多小功能（裁剪、转码、压缩、合并、变速、旋转、音量...），如果每个都占一个主 Tab，Tab 栏会爆炸。

**解决方案**：把同类小功能整合到一个 Tab 内，用内部子 Tab 切换

```
视频工具
├── 裁剪
├── 转码
├── 压缩
├── 合并
├── 画面裁剪
└── 视频调整          ← 一个 Tab 整合三个小功能
    ├── 变速
    ├── 旋转翻转
    └── 音量调整
```

**适用场景**：功能相似、操作流程一致（选文件 → 设参数 → 导出）的小工具集合。

---

### 🔧 可复用代码

#### `run_ffmpeg_with_progress` 通用函数

位置：`src-tauri/src/video_tools.rs`

功能：
- 自动插入 `-progress pipe:1 -nostats` 参数
- 实时解析进度并发送事件
- 独立线程读取 stderr，避免管道死锁
- 失败时返回完整错误信息

```rust
// 用法：把原来的 .output() 替换成调用这个函数
run_ffmpeg_with_progress(
    app_handle,
    &args,           // ffmpeg 参数数组，最后一个是输出路径
    "video-speed-progress",  // 事件名
    duration,        // 输入视频时长（秒），用于计算百分比
)?;
```

后续新增任何 ffmpeg 处理功能都可以直接复用这个函数。

---

### 📊 数据参考

3 分钟 / 48MB MP4 视频（1080p）旋转 90° 大致耗时：
- `-preset medium`：约 45 秒
- `-preset fast`：约 25-30 秒
- 纯 `-c:v copy`（如音量调整）：约 2-3 秒
