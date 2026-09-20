import type { CheatSheetData } from './index'

export const gitCommands: CheatSheetData = {
  id: 'git',
  name: 'Git 命令',
  description: '版本控制高频 Git 命令速查，含分支/合并/rebase/远程/撤销',
  columns: [
    { key: 'command', label: '命令', width: '260px', copyable: true },
    { key: 'category', label: '分类', width: '100px' },
    { key: 'params', label: '常用参数', width: '220px' },
    { key: 'description', label: '说明' },
  ],
  rows: [
    // === 基础 ===
    { command: 'git init [--bare]', category: '基础', params: '--bare 裸仓库(无工作区)', description: '初始化空仓库', examples: ['git init', 'git init --bare /srv/repos/myapp.git'] },
    { command: 'git clone [选项] URL [目录]', category: '基础', params: '--depth 1 浅克隆; -b 分支; --recurse-submodules', description: '克隆远程仓库', examples: ['git clone --depth 1 -b dev https://github.com/user/repo.git', 'git clone --recurse-submodules https://github.com/user/repo.git'] },
    { command: 'git status [选项]', category: '基础', params: '-s 短格式; -b 分支状态; --porcelain 机器可读', description: '查看工作区/暂存区状态', examples: ['git status -sb'] },
    { command: 'git add [选项] [文件/路径]', category: '基础', params: '-A 全部(= git add . + git add -u); -u 已跟踪文件; -p 交互式分块添加; -f 强制忽略.gitignore', description: '添加到暂存区', examples: ['git add -A', 'git add -p src/App.vue', 'git add . && git status -s'] },
    { command: 'git commit [选项]', category: '基础', params: '-m "msg" 消息; -a 自动 add 已跟踪文件; --amend 修改最后一次; --no-edit 保留原消息', description: '提交到本地仓库', examples: ['git commit -m "fix: resolve null pointer"', 'git commit -a -m "wip"', 'git commit --amend --no-edit'] },
    { command: 'git log [选项]', category: '基础', params: '--oneline 单行; --graph ASCII图; --all 所有分支; -n N 最近N条; -p 带diff; -S "text" 搜索改动; --since/--until', description: '查看提交历史', examples: ['git log --oneline --graph --all -20', 'git log -p -S "TODO" --all', 'git log --since="3 days ago" --author="Lito"'] },
    { command: 'git diff [选项] [a] [b]', category: '基础', params: '无参: 工作区 vs 暂存区; --staged 暂存区 vs HEAD; branch1..branch2 两分支; --stat 统计; -W 单词级别差异', description: '查看差异', examples: ['git diff', 'git diff --staged', 'git diff main..feature', 'git diff --stat HEAD~5'] },
    { command: 'git show [选项] 提交/对象', category: '基础', params: '-p 带diff; --stat 统计; --name-only 文件列表; HEAD~N 上N次', description: '查看某次提交/对象详情', examples: ['git show HEAD', 'git show --stat HEAD~3', 'git show HEAD~1:src/main.ts'] },
    { command: 'git stash [子命令]', category: '基础', params: 'save/list/pop/drop/show; -u 含未跟踪; -m "msg" 备注; pop = apply + drop', description: '临时保存/恢复工作进度', examples: ['git stash save -u "wip: experiment"', 'git stash list', 'git stash pop stash@{0}', 'git stash drop'] },
    // === 分支 ===
    { command: 'git branch [选项]', category: '分支', params: '无参 列出本地; -a 全部; -r 远程; -d 安全删除; -D 强制删除; -m 重命名; -vv 带跟踪信息', description: '分支管理', examples: ['git branch -avv', 'git branch feature-x', 'git branch -d feature-x', 'git branch -m old-name new-name'] },
    { command: 'git checkout / switch [分支/提交]', category: '分支', params: 'git checkout: -b 新建并切换; -- 文件 恢复文件; git switch: -c 新建; - 上一个分支', description: '切换分支/恢复文件（switch 为新语法）', examples: ['git switch -c feature-login', 'git switch main', 'git checkout -- src/App.vue', 'git switch -'] },
    { command: 'git merge [选项] 分支', category: '分支', params: '--no-ff 禁用快进; --ff-only 仅快进; -m "msg" 合并消息; --abort 中止合并', description: '合并分支', examples: ['git checkout main && git merge --no-ff feature', 'git merge --ff-only hotfix', 'git merge --abort'] },
    { command: 'git rebase [选项] 基准分支', category: '分支', params: '-i 交互式(pick/reword/drop/squash/fixup); --onto NEW_BASE OLD_BASE; --abort; --continue; --skip', description: '变基（线性历史）', examples: ['git rebase main', 'git rebase -i HEAD~5', 'git rebase --onto main old-base feature', 'git rebase --continue'] },
    { command: 'git cherry-pick [选项] 提交', category: '分支', params: '-n 不自动提交; -x 标注来源; --continue; --abort', description: '挑选特定提交应用到当前分支', examples: ['git cherry-pick a1b2c3d', 'git cherry-pick -x a1b2c3d^..a1b2c3d', 'git cherry-pick --continue'] },
    { command: 'git revert [选项] 提交', category: '分支', params: '-n 批量后手动提交; -m 父分支号(撤销merge); --continue/--abort', description: '反向提交（生成新提交撤销，不修改历史）', examples: ['git revert HEAD', 'git revert -n a1b2c3d e4f5g6h', 'git revert -m 1 merge_commit'] },
    // === 远程 ===
    { command: 'git remote [子命令]', category: '远程', params: 'add/remove/rename/set-url/get-url/show -v', description: '远程仓库管理', examples: ['git remote -v', 'git remote add origin https://github.com/user/repo.git', 'git remote set-url origin git@github.com:user/repo.git'] },
    { command: 'git fetch [选项] [远程] [分支]', category: '远程', params: '--all 所有远程; -p 清理已删远程分支; --depth N 浅拉取; --tags', description: '获取远程更新（不改工作区）', examples: ['git fetch --all -p', 'git fetch --depth 1 origin main'] },
    { command: 'git pull [选项] [远程] [分支]', category: '远程', params: '--rebase 拉取后rebase(推荐); --no-rebase; --ff-only; -X theirs ours 冲突策略', description: '拉取并合并/变基（= fetch + merge/rebase）', examples: ['git pull --rebase origin main', 'git pull', 'git pull -X theirs --rebase'] },
    { command: 'git push [选项] [远程] [分支]', category: '远程', params: '-u 设置上游跟踪; --force-with-lease 安全强推; --tags; --all; -o 推送选项', description: '推送本地提交到远程', examples: ['git push -u origin main', 'git push --force-with-lease origin feature', 'git push --tags'] },
    // === 撤销 ===
    { command: 'git reset [选项] [HEAD/提交]', category: '撤销', params: '--soft 仅改HEAD(保留暂存); --mixed 默认(改HEAD+清空暂存); --hard 全部丢弃(危险!); HEAD~N 回退N次', description: '重置 HEAD 指针（三种模式影响各异）', examples: ['git reset --soft HEAD~1', 'git reset HEAD  # 取消 git add', 'git reset --hard HEAD~3', 'git reset --hard origin/main'] },
    { command: 'git restore [选项] [文件]', category: '撤销', params: '--staged 恢复暂存区; --source=提交 指定来源; -W 工作区(默认); 可替代 checkout --', description: '恢复工作区/暂存区文件（checkout 的新语法）', examples: ['git restore src/App.vue', 'git restore --staged file.txt', 'git restore --source=HEAD~1 config.json'] },
    { command: 'git clean [选项]', category: '撤销', params: '-n 预览; -f 执行; -d 含未跟踪目录; -X 仅删忽略文件; -x 含忽略+未忽略', description: '删除未跟踪文件', examples: ['git clean -nd', 'git clean -fd', 'git clean -fdx'] },
    { command: 'git reflog [选项]', category: '撤销', params: '-n N 最近N条; show/expire/prune; reflog expire --expire=now --all', description: '查看 HEAD 操作历史（救命稻草）', examples: ['git reflog -20', 'git reflog show HEAD', 'git reset --hard HEAD@{5}'] },
    { command: 'git checkout [提交] -- [文件]', category: '撤销', params: '--source=提交 指定来源（新语法用 restore）', description: '从指定提交恢复文件', examples: ['git checkout HEAD~3 -- src/config.ts', 'git restore --source=HEAD~3 src/config.ts'] },
    // === 标签 ===
    { command: 'git tag [选项] [名称] [提交]', category: '标签', params: '-a 附注标签; -m "msg" 附注消息; -d 删除; -l 列表; -n 显示附注; -s GPG 签名', description: '创建/管理标签', examples: ['git tag -a v1.2.0 -m "release 1.2.0"', 'git tag -d v1.1.0', 'git tag -l "v1.*" -n'] },
    { command: 'git push [远程] [选项] tag', category: '标签', params: '--tags 推送所有标签; --follow-tags 推送带commit的tag; -d 删除远程tag', description: '推送/删除远程标签', examples: ['git push origin v1.2.0', 'git push origin --tags', 'git push origin :refs/tags/v1.1.0'] },
    // === 临时/高级 ===
    { command: 'git bisect [子命令]', category: '高级', params: 'start/bad/good/reset/visualize/log; run ./script.sh 自动', description: '二分定位引入 bug 的提交', examples: ['git bisect start HEAD v1.0.0', 'git bisect run npm test', 'git bisect reset'] },
    { command: 'git blame [选项] 文件', category: '高级', params: '-L start,end 指定行; -C/M/D 跨文件/移动/重命名; -e 邮箱; -p 机器友好', description: '逐行查看是谁改的（查锅神器）', examples: ['git blame -L 100,120 src/App.vue', 'git blame -C config.json'] },
    { command: 'git submodule [子命令]', category: '高级', params: 'add/init/update/sync/foreach/status; update --init --recursive', description: '子模块管理', examples: ['git submodule add https://github.com/lib/ui vendor/ui', 'git submodule update --init --recursive', 'git submodule foreach git pull origin main'] },
    { command: 'git worktree [子命令]', category: '高级', params: 'add/list/remove/prune; add 路径 分支; add -b 路径 分支', description: '多个工作目录（同时开发多分支）', examples: ['git worktree add -b feature-test ../repo-test main', 'git worktree list', 'git worktree remove ../repo-test'] },
    { command: 'git config [选项] key [value]', category: '高级', params: '--global 用户级; --system 系统级; --local 仓库级(默认); --list 列出; -e 编辑', description: 'Git 配置管理', examples: ['git config --global user.name "Lito"', 'git config --global user.email "lito@example.com"', 'git config --list | grep remote'] },
  ],
}
