//! Git 仓库统计：通过系统 git 命令解析提交历史，生成贡献者/趋势/文件统计
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub valid: bool,
    pub branch: String,
    pub root: String,
    pub last_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorStat {
    pub name: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStat {
    pub date: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    pub path: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStats {
    pub branch: String,
    pub total_commits: u32,
    pub total_insertions: u32,
    pub total_deletions: u32,
    pub earliest_date: String,
    pub authors: Vec<AuthorStat>,
    pub daily: Vec<DayStat>,
    pub top_files: Vec<FileStat>,
}

/// 执行 git 命令，成功返回 stdout 文本
fn run_git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("git 执行失败（请确认已安装并加入 PATH）: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug_log!("git 命令失败 {:?} => {}", args, stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 检查系统 git 是否可用
#[tauri::command]
pub fn git_check_available() -> Result<bool, String> {
    match run_git(&["--version"]) {
        Ok(v) => {
            debug_log!("git 可用: {}", v.trim());
            Ok(true)
        }
        Err(e) => {
            debug_log!("git 不可用: {}", e);
            Ok(false)
        }
    }
}

/// 验证路径是否为 git 仓库，返回分支/根路径/最近提交
#[tauri::command]
pub fn git_repo_info(path: String) -> Result<RepoInfo, String> {
    let root = match run_git(&["-C", &path, "rev-parse", "--show-toplevel"]) {
        Ok(r) => r.trim().to_string(),
        Err(_) => return Ok(RepoInfo { valid: false, branch: String::new(), root: path, last_commit: String::new() }),
    };
    let branch = run_git(&["-C", &path, "branch", "--show-current"])
        .map(|b| b.trim().to_string())
        .unwrap_or_default();
    let last_commit = run_git(&["-C", &path, "log", "-1", "--pretty=format:%h %s"])
        .map(|c| c.trim().to_string())
        .unwrap_or_default();
    debug_log!("git_repo_info valid root={} branch={}", root, branch);
    Ok(RepoInfo { valid: true, branch, root, last_commit })
}

/// 解析 git log --numstat 输出为统计（纯函数，便于单元测试）
/// 提交头行：__C__hash\t作者\t日期\t主题；numstat 行：新增\t删除\t路径；二进制文件新增/删除为 -
fn parse_log(log: &str) -> (HashMap<String, (u32, u32, u32)>, HashMap<String, (u32, u32, u32)>, HashMap<String, (u32, u32, u32)>) {
    let mut authors: HashMap<String, (u32, u32, u32)> = HashMap::new();
    let mut daily: HashMap<String, (u32, u32, u32)> = HashMap::new();
    let mut files: HashMap<String, (u32, u32, u32)> = HashMap::new();
    let mut cur_author = String::new();
    let mut cur_date = String::new();

    for line in log.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("__C__") {
            let mut parts = rest.splitn(4, '\t');
            let _hash = parts.next().unwrap_or("");
            cur_author = parts.next().unwrap_or("").to_string();
            cur_date = parts.next().unwrap_or("").to_string();
            let entry = authors.entry(cur_author.clone()).or_insert((0, 0, 0));
            entry.0 += 1;
            let d = daily.entry(cur_date.clone()).or_insert((0, 0, 0));
            d.0 += 1;
            continue;
        }
        // numstat 行：add\tdel\tpath（二进制文件为 -）
        let mut parts = line.splitn(3, '\t');
        let add_str = parts.next().unwrap_or("0");
        let del_str = parts.next().unwrap_or("0");
        let fpath = parts.next().unwrap_or("").to_string();
        if fpath.is_empty() {
            continue;
        }
        let add: u32 = add_str.parse().unwrap_or(0);
        let del: u32 = del_str.parse().unwrap_or(0);
        if let Some(a) = authors.get_mut(&cur_author) {
            a.1 += add;
            a.2 += del;
        }
        if let Some(d) = daily.get_mut(&cur_date) {
            d.1 += add;
            d.2 += del;
        }
        let f = files.entry(fpath).or_insert((0, 0, 0));
        f.0 += 1;
        f.1 += add;
        f.2 += del;
    }
    (authors, daily, files)
}

/// 核心统计：解析全量提交历史（numstat），生成贡献者/每日趋势/文件统计
fn do_analyze(path: &str) -> Result<GitStats, String> {
    let total_str = run_git(&["-C", path, "rev-list", "--count", "HEAD"])?;
    let total_commits: u32 = total_str.trim().parse().unwrap_or(0);

    let log = run_git(&[
        "-C", path, "log",
        "--numstat",
        "--date=short",
        "--pretty=format:__C__%h%x09%an%x09%ad%x09%s",
        "--no-merges",
    ])?;

    let (authors, daily, files) = parse_log(&log);

    let mut total_insertions = 0u32;
    let mut total_deletions = 0u32;
    let mut author_list: Vec<AuthorStat> = authors
        .into_iter()
        .map(|(name, (commits, ins, del))| {
            total_insertions += ins;
            total_deletions += del;
            AuthorStat { name, commits, insertions: ins, deletions: del }
        })
        .collect();
    author_list.sort_by(|a, b| b.commits.cmp(&a.commits));

    let mut daily_list: Vec<DayStat> = daily
        .into_iter()
        .map(|(date, (commits, ins, del))| DayStat { date, commits, insertions: ins, deletions: del })
        .collect();
    daily_list.sort_by(|a, b| a.date.cmp(&b.date));

    let mut file_list: Vec<FileStat> = files
        .into_iter()
        .map(|(path, (commits, ins, del))| FileStat { path, commits, insertions: ins, deletions: del })
        .collect();
    file_list.sort_by(|a, b| b.commits.cmp(&a.commits));
    file_list.truncate(10);

    let earliest_date = daily_list.first().map(|d| d.date.clone()).unwrap_or_default();

    debug_log!("git_analyze 提交数={} 作者数={} 最早={}", total_commits, author_list.len(), earliest_date);
    Ok(GitStats {
        branch: String::new(),
        total_commits,
        total_insertions,
        total_deletions,
        earliest_date,
        authors: author_list,
        daily: daily_list,
        top_files: file_list,
    })
}

#[tauri::command]
pub async fn git_analyze(path: String) -> Result<GitStats, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // 先取分支信息填充
        let mut stats = do_analyze(&path)?;
        stats.branch = run_git(&["-C", &path, "branch", "--show-current"])
            .map(|b| b.trim().to_string())
            .unwrap_or_default();
        Ok(stats)
    })
    .await
    .map_err(|e| format!("分析线程异常: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::parse_log;

    #[test]
    fn parse_log_counts_commits_authors_daily_files() {
        let log = "\
__C__abc123\t张三\t2026-08-01\t初始提交
1\t2\tsrc/main.rs
3\t0\tREADME.md

__C__def456\t李四\t2026-08-02\t修复bug
-\t-\tassets/logo.png
5\t1\tsrc/main.rs
";
        let (authors, daily, files) = parse_log(log);
        // 作者
        assert_eq!(authors.get("张三"), Some(&(1, 4, 2)));
        assert_eq!(authors.get("李四"), Some(&(1, 5, 1)));
        // 每日
        assert_eq!(daily.get("2026-08-01"), Some(&(1, 4, 2)));
        assert_eq!(daily.get("2026-08-02"), Some(&(1, 5, 1)));
        // 文件（二进制文件 -\t- 计为 0 改动，但算一次提交）
        assert_eq!(files.get("src/main.rs"), Some(&(2, 6, 3)));
        assert_eq!(files.get("README.md"), Some(&(1, 3, 0)));
        assert_eq!(files.get("assets/logo.png"), Some(&(1, 0, 0)));
    }
}
