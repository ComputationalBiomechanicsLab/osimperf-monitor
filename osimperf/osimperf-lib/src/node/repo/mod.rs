use std::path::{PathBuf, Path};

mod id;
mod input;
mod params;
mod source;

use params::Params;
use input::Input;
use source::Source;

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
    pub fn new(
        input: Input,
        params: Params,
    ) -> Self {
        todo!()
    }

    pub fn source<'a>(&'a self) -> Source<'a> {
        todo!()
    }
}
