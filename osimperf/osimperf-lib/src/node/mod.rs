mod compile;
mod file;
mod focus;
mod repo;
mod status;

use anyhow::Result;
pub use compile::*;
pub use file::NodeFile;
pub use focus::Focus;
pub use repo::*;
pub use status::State;

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, rename},
    path::PathBuf,
};

use crate::{collect_configs, find_file_by_name, Archive, BuildFolder, Folder, Home};

use self::repo::Repository;
use log::{debug, info, trace};

pub fn path_to_install<'a>(focus: Focus, id: &Id<'a>) -> PathBuf {
    id.path().join(focus.to_str())
}

pub fn path_to_source(focus: Focus, home: &Home, source: &Source) -> Result<PathBuf> {
    Ok(match focus {
        Focus::OpenSimCore => source.path()?.to_owned(),
        Focus::Dependencies => source.path()?.join("dependencies"),
        Focus::TestsSource => home.path()?.join("source"),
    })
}

pub fn path_to_build(focus: Focus, build: &BuildFolder) -> Result<PathBuf> {
    Ok(build.path()?.join(focus.to_str()))
}

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
    const SUBFOLDER_LEVEL: usize = 1;

    fn path_to_self(&self) -> PathBuf {
        self.id().path().join(Self::MAGIC_FILE())
    }
}

impl CompilationNode {
    pub const fn MAGIC_FILE() -> &'static str {
        ".osimperf-compiler.node"
    }

    pub fn is_done(&self) -> bool {
        self.state.get().iter().all(|s| s.is_done())
    }

    pub fn collect_archived(archive: &Archive) -> Result<Vec<Self>> {
        let mut vec = collect_configs::<Self>(archive.path()?, Self::MAGIC_FILE())?;
        // vec.sort_by_key(|x| NaiveDate::parse_from_str(&x.repo.date, "%Y_%m_%d").unwrap());
        vec.sort_by(|a, b| {
            NaiveDate::parse_from_str(&b.repo.date, "%Y_%m_%d")
                .unwrap()
                .cmp(&NaiveDate::parse_from_str(&a.repo.date, "%Y_%m_%d").unwrap())
        });
        Ok(vec)
    }

    pub fn new(input: Input, params: Params, archive: &Archive) -> Result<Self> {
        let repo = Repository::new(input, params)?;
        let mut out = Self {
            archive: archive.path()?.to_path_buf(),
            repo,
            ..Default::default()
        };
        out.read_or_write_new()?;
        Ok(out)
    }

    fn update(&mut self) -> Result<()> {
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

    pub fn run(&mut self,
        home: &Home,
        build: &BuildFolder,
        config: &CMakeConfig) -> Result<()> {
        let mut progress = ProgressStreamer::default();
        self.state = run_cmake_compilation(
            self.id(),
            self.repo.source(),
            build,
            home,
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
