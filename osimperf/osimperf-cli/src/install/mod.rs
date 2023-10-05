mod repo;
mod status;
mod cmake_cmds;

pub use cmake_cmds::CMakeCommands;

use crate::Ctxt;
use crate::FileBackedStruct;
use repo::{Repository, RepositoryState};
use status::{Progress, Status};

use crate::context::InstallId;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use osimperf_lib::common::collect_configs;
use osimperf_lib::common::git::Commit;

/// Stored at: `archive/ID/.compilation-node.osimperf`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompilationNode {
    pub repo: Repository,
    pub commit: Commit,
    /// Compilation status.
    pub status: Status,
    /// Used to detect changes in cmake config.
    pub config_hash: Option<u64>,
}

impl FileBackedStruct for CompilationNode {
    fn path_to_self(&self, context: &Ctxt) -> PathBuf {
        context
            .opensim_install_dir(self.id())
            .join(Self::magic_file())
    }
}

impl CompilationNode {
    pub const fn magic_file() -> &'static str {
        ".osimperf-compiler.node"
    }

    pub fn collect_archived(context: &Ctxt) -> Result<Vec<Self>> {
        let mut vec = collect_configs::<Self>(context.archive(), Self::magic_file())?;
        // vec.sort_by_key(|x| NaiveDate::parse_from_str(&x.repo.date, "%Y_%m_%d").unwrap());
        vec.sort_by(|a, b| {
            NaiveDate::parse_from_str(&b.commit.date, "%Y_%m_%d")
                .unwrap()
                .cmp(&NaiveDate::parse_from_str(&a.commit.date, "%Y_%m_%d").unwrap())
        });
        Ok(vec)
    }

    /// This id is used to create a file name that is discernable from the others.
    pub fn id<'a>(&'a self) -> InstallId<'a> {
        InstallId {
            name: self.repo.name(),
            branch: self.repo.branch(),
            hash: &self.commit.hash,
            date: &self.commit.date,
        }
    }
}
