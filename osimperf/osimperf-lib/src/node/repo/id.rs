use std::path::{Path, PathBuf};

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
    pub fn subfolder_name(&self) -> String {
        format!("{}-{}-{}", self.name, self.date, self.hash,)
    }

    pub fn path(&self) -> PathBuf {
        self.path.join(self.subfolder_name())
    }

    pub fn path_str(&self) -> String {
        String::from(self.path().to_str().unwrap())
    }
}
