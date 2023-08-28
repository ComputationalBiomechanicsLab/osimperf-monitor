use std::path::PathBuf;

use serde::{Serialize, Deserialize};

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

impl Default for Input {
    fn default() -> Self {
        todo!()
    }
}

impl Input {
    pub fn new() -> Self {
        todo!()
    }
}
