use crate::{Command, CommandTrait, PipedCommands};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::io::Lines;
use std::path::Path;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
// Can be created from the [Repository]
pub struct Commit {
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

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

pub fn commit_merged_to(repo: &Path, hash: &str) -> Result<String> {
    PipedCommands::parse(&format!(
        r#"git -C {} branch --contains {} --no-color|sed -E s/\*//"#,
        repo.to_str().unwrap(),
        hash
    ))
    .run()
}

pub fn was_commit_merged_to_branch(repo: &Path, branch: &str, hash: &str) -> Result<bool> {
    let output = commit_merged_to(repo, hash)?;
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

pub fn checkout_commit(repo: &Path, hash: &str) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("checkout");
    cmd.add_arg(&hash);
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

/// Returns date of commit.
pub fn get_date(repo: &Path, hash: &str) -> Result<String> {
    let path: &str = repo.to_str().unwrap();
    let cmd = Command::parse(&format!("git -C {path} log {hash} --pretty=format:%cs"));
    let output = cmd.run()?;
    Ok(output.lines().next().unwrap().to_string())
}

/// returns Vec<(hash, date)>
pub fn get_last_commit(repo: &Path, branch: &str) -> Result<(String, String)> {
    let path: &str = repo.to_str().unwrap();
    let cmd = Command::parse(&format!(
        "git -C {path} log {branch} --pretty=format:%H,%cs"
    ));
    let output = cmd.run()?;

    let mut split = output.lines().next().unwrap().split(',');
    let hash = String::from(split.next().context("failed to read hash")?);
    let date = String::from(
        split
            .next()
            .context("failed to read date")?
            .replace('-', "_"),
    );
    Ok((hash, date))
}

/// returns Vec<(hash, date)>
pub fn get_commits_since(
    repo: &Path,
    branch: &str,
    after_date: Option<&str>,
    before_date: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let path: &str = repo.to_str().unwrap();
    let mut cmd = Command::parse(&format!(
        "git -C {path} log {branch} --pretty=format:%H,%cs"
    ));
    if let Some(date) = after_date {
        cmd.add_arg(format!("--after={}", date));
    }
    if let Some(date) = before_date {
        cmd.add_arg(format!("--before={}", date));
    }
    let output = cmd.run()?;

    let mut commits = Vec::new();
    for line in output.lines() {
        let mut split = line.split(',');
        let hash = String::from(split.next().context("failed to read hash")?);
        let date = String::from(
            split
                .next()
                .context("failed to read date")?
                .replace('-', "_"),
        );
        commits.push((hash, date));
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

pub fn pull(repo: &Path) -> Result<String> {
    Command::parse(&format!("git -C {} pull", repo.to_str().unwrap())).run_trim()
}
