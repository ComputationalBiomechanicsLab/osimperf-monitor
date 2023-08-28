use std::path::PathBuf;

use anyhow::Result;
use osimperf_lib::*;

fn main() -> Result<()> {
    let repo = PathBuf::from(".");
    let branch = String::from("main");
    let date = String::from("2023-08-03");

    println!("commits since = {:#?}", git::get_commits_since(&repo, &branch, Some(&date), None)?);
    println!("current branch = {}", git::read_current_branch(&repo)?);
    println!("current commit = {}", git::read_current_commit(&repo)?);

    git::verify_repository(&repo, OPENSIM_CORE_URL)?;

    Ok(())
}
