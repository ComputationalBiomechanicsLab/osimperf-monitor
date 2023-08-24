use crate::{Command, CommandTrait, Commit, Date, PipedCommands, RepositoryPath};
use anyhow::{ensure, Context, Result};
use log::{debug, trace};
use std::path::Path;

pub fn read_current_branch(
    repo: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("symbolic-ref");
    cmd.add_arg("--short");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

pub fn read_current_commit(
    repo: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("rev-parse");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

pub fn get_commits_since(repo: &Path, branch: &str, date: &str) -> Result<Vec<Commit>> {
    let mut git_after = Command::new("git");
    git_after.add_arg("-C");
    git_after.add_arg(repo.to_str().unwrap());
    git_after.add_arg("log");
    git_after.add_arg(branch);
    git_after.add_arg(format!("--after={}", date));
    let git_after_output = git_after.run()?;

    let mut echo = Command::new("echo");
    echo.add_arg(git_after_output);

    // Grep the dates.
    let mut grep_date = Command::new("grep");
    grep_date.add_arg("Date");

    let mut awk_date = Command::new("awk");
    awk_date.add_arg("{print $4, $3, $6}");

    // Dates in awkward format: "2023 Aug 02"
    let dates = PipedCommands::new(vec![echo.clone(), grep_date, awk_date]).run()?;

    // Grep the commits
    let mut grep_commit = Command::new("grep");
    grep_commit.add_arg("commit");

    let mut awk_hash = Command::new("awk");
    awk_hash.add_arg("{print $2}");

    let hashes = PipedCommands::new(vec![echo.clone(), grep_commit, awk_hash]).run()?;

    let mut commits = Vec::new();
    for (date, hash) in dates.lines().zip(hashes.lines()) {
        let mut cmd = Command::new("date");
        cmd.add_arg("-d");
        cmd.add_arg(date);
        cmd.add_arg("+%Y_%m_%d");
        let commit = Commit {
            hash: String::from(hash),
            date: String::from(cmd.run_trim()?),
            branch: String::from(branch),
        };
        println!("{:?}", commit);
        commits.push(commit);
    }

    Ok(commits)
}
