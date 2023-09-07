use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use osimperf_lib::{
    common::{collect_configs, read_config, write_default_config},
    git::Commit,
    *,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, thread::sleep, time::Duration};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct OldRepository {
    /// For nicer folder and results identifiers.
    name: String,
    /// Path to repository.
    path: PathBuf,
    /// For checking that path is correct.
    url: String,
    /// The branch the commit should belong to.
    branch: String,
    date: String,
    hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OldConfig {
    pub repo: OldRepository,
    pub commit: Commit,
    /// Compilation status.
    pub state: OldState,
    /// Path to the archive.
    pub archive: PathBuf,
    /// Used to detect changes in cmake config.
    pub config_hash: Option<u64>,
}

// TODO status improvements
// size of install
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OldStatus {
    Idle,
    Compiling(Progress),
    Error(String),
    Done(Complete),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OldState {
    status_dependencies: OldStatus,
    status_opensim_core: OldStatus,
    status_tests_source: OldStatus,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Complete {
    pub size: usize,
    pub duration: Duration,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub archive: String,
}

impl NodeFile for OldConfig {
    const SUBFOLDER_LEVEL: usize = 1;

    fn path_to_self(&self) -> PathBuf {
        self.id().path().join(Self::magic_file())
    }
}

impl OldConfig {
    pub const fn magic_file() -> &'static str {
        ".osimperf-compiler.node"
    }

    pub fn id<'a>(&'a self) -> Id<'a> {
        Id {
            name: &self.repo.name,
            branch: &self.repo.branch,
            hash: &self.commit.hash,
            date: &self.commit.date,
            path: &self.archive,
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting osimperf status converter");

    let args = Args::parse();

    do_main(args).context("main exited with error")?;

    Ok(())
}

fn do_main(args: Args) -> Result<()> {
    let archive = Archive::new(&args.archive)?;

    let mut old_nodes = collect_configs::<OldConfig>(archive.path()?, OldConfig::magic_file())?;

    info!("found old_nodes: {:#?}", old_nodes);

    for old in old_nodes.drain(..) {
        let t = convert_repo(old.repo);
        let new = CompilationNode {
            repo: t.0,
            commit: t.1,
            state: convert_state(old.state),
            archive: old.archive,
            config_hash: old.config_hash,
        };
        new.try_write()
            .context(format!("failed to write node {:#?}", new))?;
    }

    Ok(())
}

fn convert_state(old: OldState) -> State {
    State {
        status: [
            convert_status(old.status_dependencies),
            convert_status(old.status_opensim_core),
            convert_status(old.status_tests_source),
        ],
    }
}

fn convert_status(old: OldStatus) -> Status {
    match old {
        OldStatus::Idle => Status::Idle,
        OldStatus::Compiling(p) => Status::Compiling(p),
        OldStatus::Error(e) => Status::Error(e),
        OldStatus::Done(Complete { duration, .. }) => Status::Done(duration),
    }
}
fn convert_repo(old: OldRepository) -> (Repository, Commit) {
    (
        Repository {
            name: old.name,
            path: old.path,
            url: old.url,
            branch: old.branch,
        },
        Commit {
            hash: old.hash,
            date: old.date,
        },
    )
}
