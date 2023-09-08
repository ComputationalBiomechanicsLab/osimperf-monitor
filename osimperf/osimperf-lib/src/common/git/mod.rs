mod commands;

pub use commands::{
    checkout_commit, pull, read_current_commit, read_repo_url, was_commit_merged_to_branch,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::Path;

pub type Date = chrono::NaiveDate;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
// Can be created from the [Repository]
pub struct Commit {
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

pub fn get_last_commit(repo: &Path, branch: &str) -> Result<Commit> {
    commands::get_last_commit(repo, branch).map(|x| Commit::from_hash_and_date_tuple(x))
}

/// returns Vec<(hash, date)>
pub fn get_commits_since(
    repo: &Path,
    branch: &str,
    after_date: Option<&str>,
    before_date: Option<&str>,
) -> Result<Vec<Commit>> {
    Ok(
        commands::get_commits_since(repo, branch, after_date, before_date)?
            .drain(..)
            .map(|x| Commit::from_hash_and_date_tuple(x))
            .collect(),
    )
}

impl Commit {
    pub(crate) fn from_hash_and_date_tuple(hash_and_date: (String, String)) -> Self {
        Self {
            hash: hash_and_date.0,
            date: hash_and_date.1,
        }
    }

    // TODO keep as chrono in struct itself.
    pub fn date(&self) -> anyhow::Result<Date> {
        Ok(Date::parse_from_str(&self.date, "%Y_%m_%d")
            .with_context(|| format!("failed to parse date {} to NaiveDate", self.date))?)
    }
}
