use std::path::{Path, PathBuf};

use crate::{Archive, Folder};

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

impl<'a> Id<'a> {
    fn checkout(&self) -> anyhow::Result<()> {
        // Checkout
        git::verify_url(&self.path, &repo.url)?;
        git::checkout(&self.path, self.branch)?;
        todo!()
    }

    fn source(self) -> anyhow::Result<PathBuf> {
        // Checkout.

        // Eat self, and give path:

        todo!()
    }

    pub fn opensim_core_source(self) -> anyhow::Result<PathBuf> {
        todo!()
    }

    pub fn dependencies_source(self) -> anyhow::Result<PathBuf> {
        todo!()
    }
}
