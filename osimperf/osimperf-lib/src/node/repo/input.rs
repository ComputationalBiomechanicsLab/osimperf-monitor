use std::path::PathBuf;

use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::common::git;
use crate::{Folder, Home, BIO_LAB_URL};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadInputs {
    pub repositories: Vec<ReadInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadInput {
    pub name: String,
    pub branch: String,
    /// Path to repository: defaults to biolab submodule.
    pub path: Option<PathBuf>,
}

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

    pub fn from(read_input: ReadInput, home: &Home) -> Self {
        Self {
            name: read_input.name,
            repo: read_input
                .path
                .unwrap_or(home.path().unwrap().join("software").join("computational-biomechanics-lab")),
            url: BIO_LAB_URL.to_string(),
            branch: read_input.branch,
        }
    }
}
