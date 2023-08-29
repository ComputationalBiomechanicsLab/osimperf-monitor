use std::path::{Path, PathBuf};

use anyhow::ensure;

use crate::{git, node::Focus, Archive, Folder};

use super::Repository;

/// For folder and file name generation.
pub struct Source<'a> {
    /// For nicer folder and results identifiers.
    pub repo: &'a Path,
    /// The branch the commit should belong to.
    pub branch: &'a str,
    /// The commit we are checking out.
    pub hash: &'a str,
    /// The url of the repo.
    pub url: &'a str,
}

impl<'a> Source<'a> {
    pub fn checkout(&self) -> anyhow::Result<()> {
        git::was_commit_merged_to_branch(self.repo, self.branch, self.hash)?;
        let hash = git::read_current_commit(self.repo)?;
        if &hash != self.hash {
            git::checkout_commit(self.repo, self.hash)?;
            ensure!(
                git::read_current_commit(self.repo)? == self.hash,
                "checkout failed"
            );
        }
        Ok(())
    }

    pub fn path(&self) -> anyhow::Result<&'a Path> {
        Ok(self.repo)
    }
}
