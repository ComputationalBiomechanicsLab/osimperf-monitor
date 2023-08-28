mod file;
mod status;
mod repo;

pub use file::NodeFile;
pub use status::State;
pub use repo::*;

use std::time::Duration;
use std::{io::Write, path::PathBuf};
use serde::{Serialize, Deserialize};

use crate::{erase_folder, git, Archive, BuildFolder, Command, CommandTrait, Folder, Repository};

///
///
/// Stored at:
/// archive/ID/.compilation-node.osimperf
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompilationNode {
    /// For nicer folder and results identifiers.
    pub name: String,
    /// Path to repository.
    pub repo: PathBuf,
    /// For checking that path is correct.
    pub url: String,
    /// The branch the commit should belong to.
    pub branch: String,
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
    /// Compilation status.
    pub state: State,
    /// Path to this node (in the archive-subfolder).
    pub path_to_node: PathBuf,
}

impl NodeFile for CompilationNode {}

#[derive(Debug)]
pub struct CompilationNodeHandle<'a> {
    focus: CurrentlyCompiling,
    handle: &'a mut CompilationNode,
}

impl<'a> CompilationNodeHandle<'a> {
    pub fn set_percentage(&mut self, percentage: usize) {
        todo!()
    }

    pub fn set_complete(self, size: usize, duration: f64) 
    {
        todo!()
    }

    pub fn set_failed(self)
    {
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CMakeConfig {

}

impl CMakeConfig {
    pub fn cmake_args(&self) -> &[&str] {todo!()}
    pub fn raw_args(&self) -> &[&str] {todo!()}
}

impl CompilationNode {
    pub fn new(
        repo: RepositoryInput,
        hash: impl ToString,
        archive: &Archive,
    ) -> anyhow::Result<Self> {
        let hash = hash.to_string();
        Ok(Self {
            date: git::date_of_commit(&repo.path, &hash)?,
            name: repo.name,
            repo: repo.path,
            url: repo.url,
            branch: repo.branch,
            hash: hash.to_string(),
            install_folder: archive.path()?.join(format!("")),
            ..Default::default()
        })
    }

    pub fn read_update(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn write_update(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn find_nodes(archive: &Archive) -> anyhow::Result<Vec<Self>> {
        todo!()
    }

    pub fn ready_for_compilation(&self) -> anyhow::Result<bool> {
        todo!()
    }

    pub fn start_compilation(&mut self, build: &BuildFolder, config: &CMakeConfig) -> anyhow::Result<()> {
        if self.ready_for_compilation() {

        }
        // Checkout commit.

        // Verify checkout.

        // Erase the folder contents.

        // Start compiling each part.
        self.compile_opensim_core(build, config)?;
        todo!()
    }

    pub fn compile_opensim_core(
        &mut self,
        build: &BuildFolder,
        config: &CMakeConfig,
    ) -> anyhow::Result<()> {
        // Try to take the lock.

        CmakeRunner {
            source: self.repo.clone(),
            install: self.install_folder.join("opensim-core"),
            build: build.path()?.join("opensim-core"),
            cmake_args: config.cmake_args(),
            raw_args: config.raw_args(),
            target: Some("install"),
            progress: ProgressStreamer { status: todo!() },
        }
        .start_compilation()?;

        Ok(())
    }

    pub fn update_status(&mut self) -> anyhow::Result<()> {
        // Verifiy fields are integer.
        // Check repo url
        //

        // Verify opensim-cmd version, if status = installed.

        //

        todo!()
    }

    pub fn write_to_file(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub fn id<'a>(&'a self) -> Id<'a> {
        todo!()
    }

}
