use crate::{Command, CommandTrait, Commit, Date, PipedCommands, RepositoryPath};
use anyhow::{ensure, Context, Result};
use log::{debug, trace};
use std::path::Path;

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";
pub static BIO_LAB_URL: &str =
"git@github.com:ComputationalBiomechanicsLab/osimperf-monitor.git";

pub fn read_current_branch(repo: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("symbolic-ref");
    cmd.add_arg("--short");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

pub fn read_current_commit(repo: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.add_arg("-C");
    cmd.add_arg(repo.to_str().unwrap());
    cmd.add_arg("rev-parse");
    cmd.add_arg("HEAD");
    Ok(cmd.run_trim()?)
}

pub fn verify_current_branch(repo: &Path, branch: &str) -> Result<bool> {
    Ok(read_current_branch(repo)? == branch)
}

pub fn verify_current_commit(repo: &Path, commit: &str) -> Result<bool> {
    Ok(read_current_commit(repo)? == commit)
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
    cmd.add_arg("main");
    let _res = cmd.run()?;
    Ok(())
}

pub fn switch_branch_checked(repo: &Path, branch: &str) -> Result<()> {
    if !verify_current_branch(repo, branch)
        .context("failed to verify branch before attempting to switch branch")?
    {
        switch_branch(repo, branch)?;
        ensure!(
            verify_current_branch(repo, branch).context("failed to verify switching branch")?,
            "Verification returned false: we did not switch to branch"
        );
    }
    Ok(())
}

pub fn checkout_commit_checked(repo: &Path, commit: &str) -> Result<()> {
    if !verify_current_commit(repo, &commit)
        .context("failed to verify commit before attempting to checkout")?
    {
        checkout_commit(repo, commit)?;
        ensure!(
            verify_current_commit(repo, &commit)?,
            format!("failed to checkout {:?}", commit)
        );
    }
    Ok(())
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

pub fn read_repo_url(repo: &Path) -> Result<String> {
    ensure!(repo.exists(), "repo does not exist: path = {:?}", repo);

    let mut git_remote_v = Command::new("git");
    git_remote_v.add_arg("-C");
    git_remote_v.add_arg(repo.to_str().unwrap());
    git_remote_v.add_arg("remote");
    git_remote_v.add_arg("-v");

    let mut grep = Command::new("grep");
    grep.add_arg("fetch");

    let mut awk = Command::new("awk");
    awk.add_arg("{print $2}");

    PipedCommands::new(vec![git_remote_v, grep, awk]).run_trim()
}

pub fn verify_repository(repo: &Path, expected_url: &str) -> Result<()> {
    let url = read_repo_url(repo)?;
    Some(()).filter(|_| url == expected_url)
        .with_context(|| format!("repo path = {:?}", repo))
        .with_context(|| format!("repo url = {:?}", url))
        .with_context(|| format!("expected url = {:?}", expected_url))
        .context("failed to verify path points to correct repo")?;
    Ok(())
}
