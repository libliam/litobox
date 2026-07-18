# Task 2 报告：实现 ffprobe 调用与 JSON 解析

**状态**: DONE

**执行时间**: 2026-07-18

## 完成内容

### 1. 代码实现

在 `src-tauri/src/media_info.rs` 中添加了以下内容：

#### 导入声明
```rust
use std::process::Command;
use serde_json::Value;
```

#### 常量定义
```rust
const CREATE_NO_WINDOW: u32 = 0x08000000;
```

#### 主要函数（11 个）

1. **get_media_info** - Tauri 命令，调用 ffprobe 获取媒体信息
   - 使用 `Command::new("ffprobe")` 执行外部命令
   - 参数：`-v quiet -print_format json -show_format -show_streams -show_chapters`
   - 应用 `CREATE_NO_WINDOW` 标志避免弹出黑框
   - 解析 JSON 输出并返回结构化数据

2. **parse_media_info** - 解析完整的媒体信息 JSON
   - 提取 format、streams、metadata、chapters
   - 根据 codec_type 分类流（video/audio/subtitle/other）

3. **parse_format** - 解析格式信息
   - 提取 format_name、duration、bitrate、stream_count 等

4. **parse_video_stream** - 解析视频流信息
   - 提取分辨率、帧率、编码格式、色彩空间等
   - 处理 extra 字段（未在 known_keys 中的字段）

5. **parse_audio_stream** - 解析音频流信息
   - 提取采样率、声道数、位深度等
   - 处理 extra 字段

6. **parse_subtitle_stream** - 解析字幕流信息
   - 提取编码格式和标签

7. **parse_other_stream** - 解析其他类型流
   - 提取 codec_type 和标签

8. **parse_chapter** - 解析章节信息
   - 提取 start_time、end_time、title

9. **parse_frame_rate** - 解析帧率字符串
   - 处理 "30/1" 格式的分数

10. **parse_disposition** - 解析 disposition 标志
    - 提取值为 1 的标志位

11. **parse_tags** - 解析标签字典
    - 将 JSON 对象转换为 Vec<KeyValue>

12. **value_to_string** - JSON 值转字符串
    - 处理 String、Number、Bool、Null 等类型

### 2. 编译验证

```bash
cargo check
```

**结果**: 编译通过，无错误

**警告**: 存在 53 个项目原有警告（与本次修改无关）
- unused_variables: volume
- dead_code: do_note_get_by_id, parse_extension_filter 等
- non_snake_case: 多个结构体字段名使用 camelCase

这些警告均来自项目其他文件，不影响本次提交。

### 3. 代码提交

```bash
git add src-tauri/src/media_info.rs
git commit -m "feat(media-info): 实现 ffprobe 调用与 JSON 解析"
```

**Commit Hash**: `617b58b`

**变更统计**: 1 file changed, 304 insertions(+)

## 技术要点

### 子进程调用规范
- ✅ 使用 `CREATE_NO_WINDOW` 标志（0x08000000）避免弹出黑框
- ✅ 使用 `std::process::Command` 调用外部命令
- ✅ 错误处理完整，包含文件访问检查、命令执行失败、JSON 解析失败等场景

### JSON 解析策略
- ✅ 使用 `serde_json::Value` 动态解析
- ✅ 通过索引访问字段（`json["format"]`），避免类型转换错误
- ✅ 使用 `unwrap_or` 提供默认值，保证健壮性
- ✅ 分离 known_keys 和 extra 字段，保留完整信息

### 数据结构映射
- ✅ 完整映射 ffprobe 输出的所有字段
- ✅ 使用 Option 类型处理可选字段（level、bit_depth）
- ✅ 使用 Vec<KeyValue> 存储动态标签和额外字段

## 下一步

Task 3: 注册命令到 Tauri 应用
- 在 `src-tauri/src/main.rs` 中添加 `mod media_info;`
- 在 `invoke_handler` 中注册 `media_info::get_media_info`
