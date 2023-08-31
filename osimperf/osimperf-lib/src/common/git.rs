use crate::{Command, CommandTrait, PipedCommands};
use anyhow::{ensure, Context, Result};
use std::path::Path;

pub fn read_current_branch(repo: &Path) -> Result<String> {
    // git rev-parse --abbrev-ref HEAD
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("rev-parse");
    cmd.add_arg("--abbrev-ref");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

pub fn commit_merged_to(repo: &Path, commit: &str) -> Result<String> {
    PipedCommands::parse(&format!(
        r#"git -C {} branch --contains {} --no-color|sed -E s/\*//"#,
        repo.to_str().unwrap(),
        commit
    ))
    .run()
}

pub fn was_commit_merged_to_branch(repo: &Path, branch: &str, commit: &str) -> Result<bool> {
    let output = commit_merged_to(repo, commit)?;
    Ok(output.lines().any(|line| line.trim() == branch))
}

pub fn read_current_commit(repo: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("rev-parse");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

fn date_of_commit_unformatted(repo: &Path, hash: &str) -> Result<String> {
    let path: &str = repo.to_str().unwrap();
    Command::parse(&format!("git -C {} show -s --format=%cs {}", path, hash)).run_trim()
}

fn fmt_date(unformatted_date: &str) -> Result<String> {
    Command::parse(&format!("date -d {} +%Y_%m_%d", unformatted_date)).run_trim()
}

pub fn date_of_commit(repo: &Path, hash: &str) -> Result<String> {
    date_of_commit_unformatted(repo, hash).and_then(|date| fmt_date(&date))
}

pub fn checkout_commit(repo: &Path, commit: &str) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("checkout");
    cmd.add_arg(&commit);
    cmd.run()?;
    Ok(())
}

pub fn switch_branch(repo: &Path, branch: &str) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("switch");
    cmd.add_arg(branch);
    let _res = cmd.run()?;
    Ok(())
}

/// returns Vec<commit-hash>
pub fn get_commits_since(
    repo: &Path,
    branch: &str,
    after_date: Option<&str>,
    before_date: Option<&str>,
) -> Result<Vec<String>> {
    let path: &str = repo.to_str().unwrap();
    let mut cmd = Command::parse(&format!("git -C {path} log {branch} --pretty=format:%H"));
    if let Some(date) = after_date {
        cmd.add_arg(format!("--after={}", date));
    }
    if let Some(date) = before_date {
        cmd.add_arg(format!("--before={}", date));
    }
    let output = cmd.run()?;

    let mut commits = Vec::new();
    for line in output.lines() {
        commits.push(String::from(line));
    }

    Ok(commits)
}

pub fn read_repo_url(repo: &Path) -> Result<String> {
    ensure!(repo.exists(), "repo does not exist: path = {:?}", repo);

    let mut git_remote_v = Command::new("git");
    git_remote_v.add_arg("-C");
    git_remote_v.add_arg(repo.to_str().unwrap());
    git_remote_v.add_arg("remote");
    git_remote_v.add_arg("-v");

    let mut grep = Command::new("grep");
    grep.add_arg("fetch");

    PipedCommands::new(vec![git_remote_v, grep]).run_trim()
}

pub fn verify_repository(repo: &Path, expected_url: &str) -> Result<()> {
    let url = read_repo_url(repo)?;
    Some(())
        .filter(|_| url == expected_url)
        .with_context(|| format!("repo path = {:?}", repo))
        .with_context(|| format!("repo url = {:?}", url))
        .with_context(|| format!("expected url = {:?}", expected_url))
        .context("failed to verify path points to correct repo")?;
    Ok(())
}
