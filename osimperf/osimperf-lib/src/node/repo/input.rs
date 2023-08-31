use std::path::PathBuf;

use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::common::git;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    /// For nicer folder and results identifiers.
    pub name: String,
    /// Path to repository.
    pub repo: PathBuf,
    /// For checking that path is correct.
    pub url: String,
    /// The branch the commit should belong to.
    pub branch: String,
}

impl Input {
    pub fn verify_url(&self) -> anyhow::Result<()> {
        let read_url = git::read_repo_url(&self.repo)?;
        ensure!(
            read_url.contains(&self.url),
            format!(
                "path to repository reads-url {} does not math given url = {}",
                read_url, self.url
            )
        );
        Ok(())
    }
}
