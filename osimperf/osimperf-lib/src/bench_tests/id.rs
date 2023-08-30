use std::path::PathBuf;

use crate::{ResultsFolder, Id};

pub struct TestId<'a> {
    folder: &'a ResultsFolder,
    outer_subfolder: &'a str,
    inner_subfolder: &'a str,
}

impl TestId {
    pub fn new(
        folder: &ResultsFolder,
        node_id: &Id,
        name: &str) -> Self {
        todo!()
    }

    /// Path ot the test node file.
    pub fn path_to_node(&self) -> PathBuf {
        todo!()
    }
}
