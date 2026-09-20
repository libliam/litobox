---
name: "litobox-devops"
description: "LitoBox 项目一键启动 dev 或打包 release 并部署桌面。Invoke when user says 启动/打包/发版/release/deploy/部署, or asks to start dev server / build release."
---

# LitoBox DevOps Skill

LitoBox 项目高频发布流水线，覆盖三条路径：启动 dev、release 打包、git 提交推送。所有命令在 PowerShell 5 环境执行，工作目录 `d:\work\litobox`。

---

## 公共前置：BOM 检查

`package.json` 和 `src-tauri/tauri.conf.json` 反复被写回 UTF-8 BOM（`EF BB BF`），Tauri build 会因此失败。**每次 dev / build 前必须先跑这段**：

```powershell
$files = @('package.json', 'src-tauri\tauri.conf.json')
foreach ($f in $files) {
  $bytes = [System.IO.File]::ReadAllBytes($f)
  if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
    $content = [System.IO.File]::ReadAllText($f)
    [System.IO.File]::WriteAllText($f, $content, [System.Text.UTF8Encoding]::new($false))
    Write-Output "$f : BOM 已移除"
  } else {
    Write-Output "$f : OK"
  }
}
```

---

## 路径 A：启动 dev

触发词：`启动` / `dev` / `开发模式`

1. 先跑 BOM 检查（公共前置）
2. 启动（后台运行，`blocking: false`）：

```
npm run tauri dev
```

3. 第一次 wait 20s，然后用 `CheckCommandStatus` 轮询。**成功判据**（两者都要看到）：
   - `Finished dev profile [unoptimized + debuginfo]`
   - `Running target\debug\litobox.exe`
4. 回报：Vite 本地 URL（`http://localhost:1420/`）+ 编译耗时。快捷键注册失败是被其他进程占用，不影响使用，只在日志里正常跳过。

---

## 路径 B：打包 release

触发词：`打包` / `release` / `发版` / `正式版` / `build`

1. 先跑 BOM 检查（公共前置）
2. 启动（后台运行，`blocking: false`，这是长任务）：

```
npm run tauri build
```

3. 第一次 wait 15s，然后轮询。**成功判据**：
   - `Finished release profile [optimized]`
   - `Built application at: D:\work\litobox\src-tauri\target\release\litobox.exe`
4. 回报：前端耗时 + Rust release 编译耗时。

### 可选：复制到桌面

用户说"复制到桌面"时执行：

```powershell
Copy-Item -Path "D:\work\litobox\src-tauri\target\release\litobox.exe" `
  -Destination "$env:USERPROFILE\OneDrive\Desktop\litobox.exe" -Force
```

注意：沙箱可能拦截 OneDrive 路径。失败时提示用户手动执行，不要自行改路径。

---

## 路径 C：Git 提交推送

触发词：`提交` / `commit` / `推送` / `push`

1. 先看改了啥：

```
git status --short
git diff --stat
```

2. 写中文 commit message，覆盖这几类（按实际变更挑选，不要瞎写）：
   - `feat:` 新增功能 / 工具页
   - `fix:` Bug 修复
   - `chore:` 依赖升级 / 配置调整 / 重构无行为变化
   - `docs:` 文档

3. 提交：

```
git add -A
git commit -m "<中文 message>"
```

4. 推送：**必须重试一次**。GitHub SSL/TLS 偶发握手失败（`schannel: failed to receive handshake`），第二次通常就通了：

```
git push
# 失败就立刻再跑一次
```

---

## 已知坑

| 现象 | 处理 |
|------|------|
| BOM 头反复出现 | 每次 dev/build 前硬检查，根治需要后续定位谁在写 BOM |
| `schannel: failed to receive handshake` | 不是配置问题，重试一次即可 |
| `resvg rendersvg feature 不存在` | 删 `src-tauri/Cargo.lock` 让它重新解析依赖 |
| release 产物路径 | 固定是 `src-tauri\target\release\litobox.exe`，`bundle.targets` 是空数组（AGENTS.md 约束：不生成安装包） |
| OneDrive 桌面复制被拦 | 不要绕过，提示手动命令 |
