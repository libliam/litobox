import type { CheatSheetData } from './index'

export const powerShellCommands: CheatSheetData = {
  id: 'powershell',
  name: 'PowerShell 命令',
  description: 'Windows PowerShell 高频命令速查，Get-Process/Invoke-WebRequest/文件操作/注册表等，含 Linux 命令对照',
  columns: [
    { key: 'command', label: '命令', width: '220px', copyable: true },
    { key: 'category', label: '分类', width: '110px' },
    { key: 'params', label: '常用参数', width: '240px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 基础/别名 ===
    { command: 'Get-Command gci', category: '基础', params: 'Get-Alias; alias: gci=Get-ChildItem, ls, dir; gi=Get-Item; cd=Set-Location', description: '查看命令别名（Linux 风格缩写对照）', examples: ['Get-Alias | Where-Object {$_.Name -eq "gci"}', 'Set-Alias ll ls', 'Get-Command | Measure-Object'] },
    { command: 'Get-Help 命令 [-Online]', category: '基础', params: '-Full 完整; -Examples 仅示例; -Parameter 某参数; -Online 浏览器打开MS文档', description: '内置帮助（比Linux man直接在控制台更友好）', examples: ['Get-Help Get-ChildItem -Full', 'Get-Help Get-ChildItem -Examples', 'Get-Help Invoke-RestMethod -Online'] },
    // === 文件操作 ===
    { command: 'Get-ChildItem (gci/ls)', category: '文件', params: '-Path 路径; -Recurse 递归; -File/-Directory 过滤; -Filter *.log; -Force 含隐藏; -Name 仅名字; -Depth N', description: '列出目录内容（对照 Linux ls/find）', examples: ['Get-ChildItem -Recurse -Filter *.log | Select-Object FullName, Length', 'gci -Force .', 'Get-ChildItem /var/log/nginx -File | Sort-Object Length -Descending | Select-Object -First 20'] },
    { command: 'Set-Location / Push-Location / Pop-Location', category: '文件', params: 'Set-Location (-Path / -LiteralPath); Push-Location (cd+stack); Pop-Location (cd-); Get-Location (pwd)', description: '切换目录（支持路径栈）', examples: ['Set-Location D:\\project', 'Push-Location D:\\temp; Pop-Location', 'cd ..'] },
    { command: 'Copy-Item / Move-Item / Remove-Item', category: '文件', params: '-Recurse 目录递归; -Force 强制覆盖/删只读; -Filter; -Include/-Exclude; -Destination', description: '复制/移动/删除（cp/mv/rm 对照）', examples: ['Copy-Item -Recurse -Force ./dist D:\\deploy\\', 'Remove-Item -Recurse -Force .\\node_modules', 'Move-Item -Path old.txt -Destination new.txt'] },
    { command: 'New-Item / New-Item -ItemType File', category: '文件', params: '-ItemType File/Directory/SymbolicLink; -Force 已存在不报错; -Path 路径; -Name 名字', description: '创建文件/目录/链接（mkdir/touch）', examples: ['New-Item -ItemType Directory -Path D:\\logs -Force', 'New-Item -ItemType File -Path config.ini', 'New-Item -ItemType SymbolicLink -Link "C:\\Users\\Public\\Shortcut" -Target "D:\\Project"'] },
    { command: 'Get-Item / Get-Content / Set-Content', category: '文件', params: '-Recurse; -File; -Encoding UTF8/GBK/UTF8BOM; -ReadCount 批量读; -TotalCount 前N行; -Tail 末N行(PW 7+)', description: '读取文件/元数据（cat/head/tail）', examples: ['Get-Content .\\error.log -Tail 50', 'Get-Content .\\huge.log -ReadCount 1000 | ForEach-Object { $_ }', 'Set-Content -Path .\\out.txt -Encoding UTF8 -Value $content'] },
    { command: 'Select-String (sls)', category: '文件', params: '-Path/*.log; -Pattern "regex"; -SimpleMatch 非正则; -CaseSensitive; -Context N,M 上下文; -List 仅列首个匹配', description: '文本搜索（grep 替代，原生正则）', examples: ['Select-String -Path *.log -Pattern "ERROR" -Context 2,2', 'gci -Recurse *.ts | Select-String "import.*from"', 'Select-String -Pattern "TODO|FIXME" -Path .\\src\\**\\*.ts -List'] },
    { command: 'Get-Content file | Select-Object -Skip N -First M', category: '文件', params: 'Select-Object -First 10 前10行; -Skip 跳过; -Last 5 后5行; -Unique 去重; -ExpandProperty', description: '管道处理（head/tail/filter 组合）', examples: ['Get-Content file.txt | Select-Object -First 50', 'Get-Content file.txt | Select-Object -Skip 50 -First 100'] },
    { command: 'Compare-Object / Get-FileHash', category: '文件', params: 'Compare-Object -ReferencePath / -DifferencePath; SyncWindow; Get-FileHash file -Algorithm SHA256/MD5', description: '文件对比/哈希校验（diff/md5sum）', examples: ['Compare-Object (gc a.txt) (gc b.txt)', 'Get-FileHash package.json -Algorithm SHA256'] },
    // === 进程/服务 ===
    { command: 'Get-Process / Stop-Process / Start-Process', category: '进程', params: '-Name/-Id; -Force 强制; -PassThru 返回对象; -Wait 等待; -ArgumentList "arg1","arg2"', description: '进程管理（ps/kill）', examples: ['Get-Process -Name chrome | Select-Object Id, CPU, WS | Sort-Object CPU -Descending | Select -First 10', 'Stop-Process -Name calc -Force', 'Start-Process -FilePath node -ArgumentList "server.js" -NoNewWindow -PassThru'] },
    { command: 'Get-Service / Stop-Service / Restart-Service', category: '进程', params: '-Name/-DisplayName; -Status Running/Stopped; -DependentServices; -RequiredServices', description: 'Windows 服务管理', examples: ['Get-Service | Where-Object {$_.Status -eq "Running"}', 'Restart-Service -Name w3svc -Force', 'Get-Service -Name mysql -DependentServices'] },
    { command: 'tasklist / taskkill (cmd)', category: '进程', params: '/FI "IMAGENAME eq chrome.exe" 过滤; /PID PID; /F 强制', description: 'cmd 版本进程管理（比 PS 快）', examples: ['tasklist /FI "IMAGENAME eq chrome.exe"', 'taskkill /F /PID 1234', 'taskkill /IM notepad.exe /F'] },
    // === 网络 ===
    { command: 'Invoke-WebRequest / Invoke-RestMethod', category: '网络', params: '-Uri URL; -Method GET/POST/PUT; -Headers @{}; -Body; -ContentType; -OutFile; -UseBasicParsing; -TimeoutSec', description: 'HTTP 请求（curl/Python requests）', examples: ['Invoke-RestMethod -Uri https://api.github.com/users/microsoft/repos | Select-Object -First 3 name', 'Invoke-WebRequest -Uri https://example.com/file.zip -OutFile .\\file.zip', '$body = @{k="v"} | ConvertTo-Json; Invoke-RestMethod -Uri api.io/data -Method Post -ContentType "application/json" -Body $body'] },
    { command: 'Test-NetConnection / Test-Connection', category: '网络', params: '-ComputerName 主机; -Port 端口; -InformationLevel Detailed; Test-Connection = ping', description: '网络连通性测试（ping/telnet）', examples: ['Test-NetConnection google.com -Port 443', 'Test-Connection google.com -Count 4', 'Test-NetConnection 192.168.1.10 -Port 3306'] },
    { command: 'netstat -ano / Get-NetTCPConnection', category: '网络', params: '-ano 显示PID; Get-NetTCPConnection -State Listen; -LocalPort', description: '端口占用查询（ss/netstat）', examples: ['netstat -ano | findstr :8080', 'Get-NetTCPConnection -State Listen | Select-Object LocalAddress, LocalPort, OwningProcess', 'Get-NetTCPConnection -LocalPort 3306 | Select OwningProcess'] },
    // === 管道/Pipeline ===
    { command: 'cmd1 | cmd2 | cmd3', category: '管道', params: 'PS 管道传递对象(不是文本); $_ 当前对象; ForEach-Object / %; Where-Object / ?; Select-Object / Select', description: 'PowerShell 管道是对象流而非文本流', examples: ['Get-Process | Where-Object {$_.CPU -gt 10} | Sort-Object CPU | Select -First 5', 'Get-ChildItem *.log | ForEach-Object { Get-Content $_.FullName -Tail 100 }', '1..10 | ForEach-Object { "num: $_" }'] },
    { command: 'Measure-Object / Group-Object / Select-Object -Unique', category: '管道', params: '-Property; -Sum/-Avg/-Count/-Min/-Max; Group-Object -AsHashTable', description: '统计/分组', examples: ['Get-Process | Measure-Object -Property WS -Sum', 'Get-ChildItem | Group-Object Extension | Sort-Object Count -Descending', '1..10 | Group-Object { $_ % 3 }'] },
    // === 注册表 ===
    { command: 'Get-ItemProperty / Set-ItemProperty HKLM:\\...', category: '注册表', params: 'HKLM = HKEY_LOCAL_MACHINE; HKCU = HKEY_CURRENT_USER; HKCR; HKCC; Get-ChildItem 递归', description: '读写 Windows 注册表', examples: ['Get-ItemProperty "HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion" | Select ProductName, DisplayVersion', 'Set-ItemProperty -Path "HKCU:\\Software\\MyApp" -Name "Theme" -Value "dark" -Type String', 'Get-ChildItem "HKLM:\\Software" | Select PSChildName'] },
    // === 环境变量 ===
    { command: '$env:VAR / [Environment]::GetEnvironmentVariable', category: '环境', params: '$env:PATH = "新值"; [Environment]::SetEnvironmentVariable("VAR", "val", "User|Machine|Process")', description: '读写环境变量（永久/临时）', examples: ['$env:PATH -split ";"', '[Environment]::SetEnvironmentVariable("JAVA_HOME", "C:\\java-17", "User")', '$env:JAVA_HOME'] },
    // === 日志/事件 ===
    { command: 'Get-WinEvent -LogName Application', category: '系统', params: '-FilterXPath; -FilterHashtable @{LogName=""; StartTime=; Id=}; -MaxEvents N', description: '查看 Windows 事件日志（journalctl）', examples: ['Get-WinEvent -LogName Application -MaxEvents 50 | Select TimeCreated, Id, LevelDisplayName, Message', 'Get-WinEvent -FilterHashtable @{LogName="System"; Level=2; StartTime=(Get-Date).AddDays(-1)}'] },
    { command: 'Write-Host / Write-Warning / Write-Error', category: '系统', params: '-ForegroundColor/-BackgroundColor; Write-Host 仅控制台; Write-Output 管道; Write-Information', description: '输出分级', examples: ['Write-Host "Success!" -ForegroundColor Green', 'Write-Warning "配置缺失，使用默认值"', 'Write-Error "连接失败" -RecommendedAction "检查网络"'] },
    // === 计划任务 ===
    { command: 'schtasks / Create / Query / Delete / Run', category: '系统', params: '/SC MINUTE/HOURLY/DAILY/WEEKLY/MONTHLY/ONLOGON/ONSTART; /TN 任务名; /TR "命令"; /ST HH:MM', description: 'Windows 计划任务（crontab 对照）', examples: ['schtasks /Create /SC DAELY /TN "DailyBackup" /TR "C:\\backup.bat" /ST 02:00', 'schtasks /Query /TN "DailyBackup" /V /FO LIST', 'schtasks /Delete /TN "DailyBackup" /F'] },
    // === 压缩 ===
    { command: 'Compress-Archive / Expand-Archive', category: '文件', params: '-Path 源路径; -DestinationPath 目标; -Force; -CompressionLevel Optimal/Fastest/NoCompression', description: 'ZIP 压缩/解压（无需额外工具）', examples: ['Compress-Archive -Path ./dist -DestinationPath ./dist.zip -Force', 'Expand-Archive -Path archive.zip -DestinationPath ./out -Force'] },
    // === JSON/CSV ===
    { command: 'ConvertTo-Json / ConvertFrom-Json', category: '文件', params: '-Depth N; -Compress 单行; ConvertFrom-Json -AsHashtable (PW 7+); -InputObject', description: 'JSON 序列化', examples: ['@{name="张三"; age=18; tags=@("a","b")} | ConvertTo-Json -Depth 3', 'Get-Content .\\data.json -Raw | ConvertFrom-Json'] },
    { command: 'Export-Csv / Import-Csv', category: '文件', params: '-Path; -NoTypeInformation; -Encoding UTF8; -Header; -Delimiter; -Append', description: 'CSV 读写', examples: ['Get-Process | Select Name,Id,CPU | Export-Csv -Path .\\processes.csv -NoTypeInformation -Encoding UTF8', 'Import-Csv .\\data.csv | Where-Object {$_.age -gt 30}'] },
    // === 常用一行流 ===
    { command: 'ls file.ts -Recurse | sls "TODO"', category: '一行流', params: 'gci+sls=find . -name "*.ts" -exec grep TODO {}', description: '查找待办', examples: ['Get-ChildItem -Recurse -Include *.ts,*.vue | Select-String -Pattern "TODO|FIXME|HACK"'] },
    { command: 'ls -Recurse *.tmp | rm', category: '一行流', params: '清理临时文件', description: '清理临时文件', examples: ['Get-ChildItem -Recurse -Include *.tmp,*.log -File | Remove-Item -Force'] },
    { command: 'Get-Process | Sort CPU -Descending | Select -First 10', category: '一行流', params: 'top 10 最耗 CPU', description: 'top 命令替代', examples: ['Get-Process | Sort-Object CPU -Descending | Select-Object -First 10 Name,@{N="CPU%";E={[math]::Round($_.CPU,1)}},@{N="MemMB";E={[math]::Round($_.WS/1MB,1)}}'] },
  ],
}
