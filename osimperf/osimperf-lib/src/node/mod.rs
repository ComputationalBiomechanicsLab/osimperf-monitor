mod compile;
mod file;
mod focus;
mod repo;
mod status;

pub use compile::*;
pub use file::NodeFile;
pub use focus::Focus;
pub use repo::*;
pub use status::State;

use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, rename},
    path::PathBuf,
};

use crate::{Archive, BuildFolder, Folder};

use self::repo::Repository;
use log::{debug, info, trace};

///
///
/// Stored at:
/// archive/ID/.compilation-node.osimperf
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompilationNode {
    pub repo: Repository,
    /// Compilation status.
    pub state: State,
    /// Path to the archive.
    pub archive: PathBuf,
}

impl NodeFile for CompilationNode {
    fn path_to_self(&self) -> PathBuf {
        self.id().path().join(".osimperf-compiler.node")
    }
}

impl CompilationNode {
    pub fn new(input: Input, params: Params, archive: &Archive) -> anyhow::Result<Self> {
        let repo = Repository::new(input, params)?;
        let mut out = Self {
            archive: archive.path()?.to_path_buf(),
            repo,
            ..Default::default()
        };
        out.update()?;
        Ok(out)
    }

    fn update(&mut self) -> anyhow::Result<()> {
        let dir = self.id().path();
        if !dir.exists() {
            let temp_dir = dir.parent().unwrap().join("temp_dir");
            trace!("Creating temporary directory at {:?}", temp_dir);
            create_dir(&temp_dir)?;

            debug!("Creating new node directory at {:?}", dir);
            trace!("Moving temporary directory to {:?}", dir);
            rename(&temp_dir, &dir)?;
        }
        if let Ok(_) = self.try_read() {
            info!("found previous node: {:#?}", self);
        } else {
            info!("create new node at {:?}", self.path_to_self());
            self.try_write()?;
        }
        Ok(())
    }

    pub fn run(&mut self, build: &BuildFolder, config: &CMakeConfig) -> anyhow::Result<()> {
        let mut progress = ProgressStreamer::default();
        self.state = run_cmake_compilation(
            self.id(),
            self.repo.source(),
            build,
            config,
            &mut progress,
            &self.state,
        )?;
        self.try_write()?;
        Ok(())
    }

    pub fn id<'a>(&'a self) -> Id<'a> {
        Id {
            name: &self.repo.name,
            branch: &self.repo.branch,
            hash: &self.repo.hash,
            date: &self.repo.date,
            path: &self.archive,
        }
    }
}
