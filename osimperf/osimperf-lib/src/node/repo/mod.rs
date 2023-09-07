use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::hash::Hash;

mod id;
mod input;
mod commit;
mod source;

pub use id::Id;
pub use input::{ReadInput, Input, ReadInputs};
pub use commit::Commit;
pub use source::Source;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct Repository {
    /// For nicer folder and results identifiers.
    pub name: String,
    /// Path to repository.
    pub path: PathBuf,
    /// For checking that path is correct.
    pub url: String,
    /// The branch the commit should belong to.
    pub branch: String,
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

impl Repository {
    pub fn new(input: Input, params: Commit) -> anyhow::Result<Self> {
        input.verify_url()?;
        Ok(Self {
            name: input.name,
            path: input.repo,
            url: input.url,
            branch: input.branch,
            hash: params.hash,
            date: params.date,
        })
    }

    pub fn source<'a>(&'a self) -> Source<'a> {
        Source {
            branch: &self.branch,
            hash: &self.hash,
            url: &self.url,
            repo: &self.path,
        }
    }
}
