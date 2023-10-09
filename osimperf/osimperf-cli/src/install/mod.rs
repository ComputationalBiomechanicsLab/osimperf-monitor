mod cmake_cmds;
mod progress;
mod status;

use anyhow::anyhow;
use anyhow::ensure;
pub use cmake_cmds::CMakeCommands;
use progress::CMakeProgressStreamer;

use crate::env_vars;
use crate::Commit;
use crate::Ctxt;
use crate::EnvVar;
use crate::FileBackedStruct;
pub use crate::{Repository, RepositoryState};
use status::{Progress, Status};

use crate::context::InstallId;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::path::PathBuf;

use crate::Command;
use crate::CommandTrait;
use osimperf_lib::common::collect_configs;

use std::time::Duration;

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
        vec.sort_by(|a, b| b.commit.date().cmp(&a.commit.date()));
        Ok(vec)
    }

    /// This id is used to create a file name that is discernable from the others.
    pub fn id<'a>(&'a self) -> InstallId<'a> {
        InstallId {
            name: self.repo.name(),
            branch: self.repo.branch(),
            hash: self.commit.hash(),
            date: self.commit.date_str(),
        }
    }

    pub fn new(context: &Ctxt, repo: Repository, commit: Commit) -> Result<Self> {
        let mut out = Self {
            repo,
            commit,
            status: Status::default(),
            config_hash: None,
        };
        out.read_or_write_new(context)?;
        Ok(out)
    }

    pub fn install(
        &mut self,
        context: &Ctxt,
        cmake_cmds: &CMakeCommands,
        force: bool,
    ) -> Result<bool> {
        // Returns whether there was any compilation attempted.
        if self.status.done() && !force {
            return Ok(false);
        }

        // Check if the config changed since last time we compiled.
        let mut hasher = DefaultHasher::new();
        cmake_cmds.hash(&mut hasher);
        let hash = hasher.finish();
        let changed = hash != *self.config_hash.replace(hash).get_or_insert(hash);

        // If config changed, and compilation not yet succesful, we recompile.
        if changed {
            warn!("Cmake config changed -- retry compilation");
            self.status = Status::default();
        }

        // If we already failed compiling, no need to try again.
        if self.status.failed().is_some() && !force {
            return Ok(false);
        }

        // Check-out the Repository to the correct commit.
        let checked_out_token = self.repo.checkout(&self.commit)?;

        // Set environmental variables.
        let env_vars = env_vars(
            context,
            self.id(),
            Some(checked_out_token.path().to_owned()),
        );
        let cmake_cmds = cmake_cmds.with_env_vars(&env_vars);

        let mut dt = Duration::from_secs(0);
        for (task, cmd) in cmake_cmds.0.iter() {
            // First update the status.
            self.status = Status::Compiling(Progress {
                percentage: 0.,
                task: task.clone(),
            });
            self.try_write(context)?;

            // Setup something to keep track of the progres.
            let mut progress = CMakeProgressStreamer::new(self, context, task.clone());

            // Start compilation.
            debug!("run cmake command: {:#?}", cmd.print_command());
            let output = cmd
                .run_and_stream(&mut progress)
                .with_context(|| format!("cmake command failed: {:#?}", cmd.print_command()))?;
            dt += output.duration;

            // We failed to compile, so we stop.
            if !output.success() {
                // Update the status.
                self.status = Status::Error(output.stderr_str_clone());
                // Update the file backing this struct.
                self.try_write(context)?;
                return Err(anyhow!("Failed to compile"));
            }
        }

        // Update the status.
        self.status = Status::Done(dt);
        // Update the file backing this struct.
        self.try_write(context)?;

        // Return that we compiled something.
        Ok(true)
    }

    /// Returns the size of the installed targets.
    pub fn read_disk_size(&self, context: &Ctxt) -> Result<usize> {
        crate::common::folder_size(&self.path_to_self(context))
    }
}
