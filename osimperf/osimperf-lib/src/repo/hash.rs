use std::path::PathBuf;

use crate::{git, Archive, Command, CommandTrait, Folder, Repository, ResultsFolder};
use anyhow::{ensure, Context, Result};
use log::{debug, info, trace, warn};

#[derive(Clone, Debug)]
pub struct Hash {
    value: String,
}

impl Hash {
    pub fn new(hash: impl ToString) -> Result<Self> {
        let value = hash.to_string();
        ensure!(
            value.chars().count() == 40,
            "attempted to chonstruct hash from less than 40 characters"
        );
        Ok(Self { value })
    }

    pub fn str(&self) -> &str {
        &self.value
    }

    pub fn short(&self) -> &str {
        todo!()
    }
}
