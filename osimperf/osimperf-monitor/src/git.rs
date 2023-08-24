use crate::{cmd::pipe_commands, folders, Command, Commit, Folders};
use anyhow::{ensure, Context, Result};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{path::Path, str};

fn verify_current_branch(repo: &Path, branch: &str) -> Result<bool> {
    let mut verify = Command::new("git");
    verify.add_arg("-C");
    verify.add_arg(repo.to_str().unwrap());
    verify.add_arg("symbolic-ref");
    verify.add_arg("--short");
    verify.add_arg("HEAD");
    let read_branch = verify
        .run()
        .context(format!("Failed to verify main branch"))?;
    Ok(read_branch == branch)
}

fn verify_current_commit(repo: &Path, commit: &str) -> Result<bool> {
    let mut verify = Command::new("git");
    verify.add_arg("-C");
    verify.add_arg(repo.to_str().unwrap());
    verify.add_arg("rev-parse");
    verify.add_arg("HEAD");
    let read_commit = verify
        .run()
        .context(format!("Failed to verify current commit"));
    Ok(read_commit? == commit)
}

pub fn switch_opensim_core_to_main(folders: &Folders) -> Result<()> {
    let repo = Path::new(&folders.opensim_core);
    is_the_opensim_core_repository(repo).context("failed to verify opensim-core")?;
    if !verify_current_branch(repo, "main").unwrap_or(false) {
        debug!("Opensim core repository is not on main branch: attempt to switch to main now.");
        let mut switch = Command::new("git");
        switch.add_arg("-C");
        switch.add_arg(repo.to_str().unwrap());
        switch.add_arg("switch");
        switch.add_arg("main");
        let res = switch.run();
        trace!(
            "Executed command to switch opensim-core to main: output=\n{:?}",
            res
        );
        ensure!(
            verify_current_branch(repo, "main")
                .context("failed to execute branch verification after switching to main")?,
            "Verification returned false: we did not switch to main branch"
        );
    }
    debug!("Succesfully switched opensim-core to main");
    Ok(())
}

pub fn checkout_commit(repo: &Path, commit: &Commit) -> Result<()> {
    is_the_opensim_core_repository(repo).context("failed to verify opensim-core")?;
    if !verify_current_commit(repo, &commit.hash).unwrap_or(false) {
        // Checkout commit.
        let mut checkout = Command::new("git");
        checkout.add_arg("-C");
        checkout.add_arg(repo.to_str().unwrap());
        checkout.add_arg("checkout");
        checkout.add_arg(&commit.hash);
        // Switching gives a warning to stderr?
        let res = checkout.run();
        println!("checkout: {:?}", res);
        ensure!(
            verify_current_commit(repo, &commit.hash)?,
            format!("failed to checkout {:?}", commit)
        );
    }
    println!("succesfully switched opensim-core to: {:?}", commit);
    Ok(())
}

/// This function verifies that the given path is indeed the "opensim-core" repository.
///  This can be used to prevent accidentally performing git actions on an unwanted repository.
pub fn is_the_opensim_core_repository(repo: &Path) -> Result<()> {
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

    let output = pipe_commands(&[git_remote_v, grep, awk])?;

    let url = "https://github.com/opensim-org/opensim-core.git";
    ensure!(
        url == output,
        format!(
            "Path to source does not look like the opensim repository:\npath = {:?}\noutput={:?}",
            repo, output
        )
    );

    Ok(())
}
