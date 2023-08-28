use std::path::{Path, PathBuf};

use crate::{Archive, Folder};

use super::Repository;

/// For folder and file name generation.
pub struct Id<'a> {
    /// For nicer folder and results identifiers.
    pub name: &'a str,
    /// The branch the commit should belong to.
    pub branch: &'a str,
    /// The commit we are checking out.
    pub hash: &'a str,
    /// The date is for ordering results.
    pub date: &'a str,
    /// Archive subfolder.
    pub path: &'a Path,
}

impl<'a> Id<'a> {
    pub fn folder(&self) -> PathBuf {
        self.path.join(format!(
            "{}-{}-{}-{}",
            self.name, self.branch, self.date, self.hash,
        ))
    }

    pub fn folder_string(&self) -> String {
        String::from(self.folder().to_str().unwrap())
    }
}
