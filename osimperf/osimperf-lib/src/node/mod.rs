mod cmake;
mod file;
mod installed_size;
mod repo;
mod status;
mod target;

use anyhow::{Context, Result};
pub use cmake::*;
pub use file::NodeFile;
pub use repo::*;
pub use status::{Progress, State, Status};
pub use target::CompilationTarget;

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::common::collect_configs;
use crate::git::Commit;
use crate::{erase_folder, Archive, BuildFolder, Folder, Home};

use self::installed_size::folder_size;
use log::{trace, warn};

pub fn path_to_install<'a>(target: CompilationTarget, id: &Id<'a>) -> PathBuf {
    id.path().join(target.to_str())
}

pub fn path_to_source(
    target: CompilationTarget,
    home: &Home,
    repo: &RepositoryState,
) -> Result<PathBuf> {
    Ok(match target {
        CompilationTarget::OpenSimCore => repo.path().to_owned(),
        CompilationTarget::Dependencies => repo.path().join("dependencies"),
        CompilationTarget::TestsSource => home.path()?.join("source"),
    })
}

pub fn path_to_build(target: CompilationTarget, build: &BuildFolder) -> Result<PathBuf> {
    Ok(build.path()?.join(target.to_str()))
}

///
///
/// Stored at:
/// archive/ID/.compilation-node.osimperf
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct CompilationNode {
    pub repo: Repository,
    pub commit: Commit,
    /// Compilation status.
    pub state: State,
    /// Path to the archive.
    pub archive: PathBuf,
    /// Used to detect changes in cmake config.
    pub config_hash: Option<u64>,
}

impl NodeFile for CompilationNode {
    const SUBFOLDER_LEVEL: usize = 1;

    fn path_to_self(&self) -> PathBuf {
        self.id().path().join(Self::magic_file())
    }
}

impl CompilationNode {
    pub const fn magic_file() -> &'static str {
        ".osimperf-compiler.node"
    }

    /// Returns true if everything compiled succesfully.
    pub fn is_done(&self) -> bool {
        self.state.get().iter().all(|s| s.is_done())
    }

    pub fn collect_archived(archive: &Archive) -> Result<Vec<Self>> {
        let mut vec = collect_configs::<Self>(archive.path()?, Self::magic_file())?;
        // vec.sort_by_key(|x| NaiveDate::parse_from_str(&x.repo.date, "%Y_%m_%d").unwrap());
        vec.sort_by(|a, b| {
            NaiveDate::parse_from_str(&b.commit.date, "%Y_%m_%d")
                .unwrap()
                .cmp(&NaiveDate::parse_from_str(&a.commit.date, "%Y_%m_%d").unwrap())
        });
        Ok(vec)
    }

    pub fn new(repo: Repository, commit: Commit, archive: &Archive) -> Result<Self> {
        let mut out = Self {
            archive: archive.path()?.to_path_buf(),
            repo,
            commit,
            state: State::default(),
            config_hash: None,
        };
        out.read_or_write_new()?;
        Ok(out)
    }

    fn install_dir(&self, target: CompilationTarget) -> PathBuf {
        self.id().path().join(target.to_str())
    }

    pub fn run(&mut self, home: &Home, build: &BuildFolder, config: &CMakeConfig) -> Result<bool> {
        // Returns whether there was any compilation attempted.
        let already_compiled = self.state.get().iter().all(|x| x.is_done());
        if already_compiled {
            return Ok(false);
        }

        // Check if the config changed since last time we compiled.
        let mut hasher = DefaultHasher::new();
        config.hash(&mut hasher);
        let hash = hasher.finish();
        let changed = hash != *self.config_hash.replace(hash).get_or_insert(hash);

        // If config changed, and compilation not yet succesful, we recompile.
        if changed {
            warn!("Cmake config changed -- retry compilation");
            self.state.reset();
        }

        // If we already failed compiling, no need to try again.
        let already_failed = self.state.get().iter().any(|x| x.has_failed());
        if already_failed {
            return Ok(false);
        }

        // Otherwise we start compiling.

        // Go over compile targets: [dependencies, opensim-core, tests].
        for target in CompilationTarget::list_all() {
            // Start compiling project.
            let install_dir = self.install_dir(target);

            // Check the status of this target, and if we should attempt compilation.
            if self.state.status(target).should_compile() {

                // Check-out the Repository to the correct commit.
                let checked_out_token = self.repo.checkout(&self.commit)?;

                // First update the status.
                self.state
                    .set(target, Status::Compiling(Progress { percentage: 0. }));
                self.try_write()?;

                // Setup cmake commands.
                let cmd = CMakeCmds::new(&self.id(), &checked_out_token, home, build, config, target)?;
                trace!("CMAKE COMMAND:\n{}", cmd.print_pretty());

                // Erase the install dir.
                erase_folder(&install_dir)
                    .with_context(|| format!("failed to erase install dir: {:?}", install_dir))?;

                // Erase the build dir.
                erase_folder(&build.path()?.join(target.to_str()))
                    .with_context(|| format!("failed to erase build dir"))?;

                // Setup something to keep track of the progres (for the UI).
                let mut progress = CMakeProgressStreamer::new(self, target);

                // Start compilation.
                let output = cmd
                    .run(&mut progress, &install_dir)
                    .with_context(|| format!("cmake failed: {:#?}", cmd.print_pretty()));

                // Update the status.
                self.state.set(target, Status::from_output(output));

                // Update the file backing this struct.
                self.try_write()?;
            }

            // We failed to compile, so we stop.
            if !self.state.status(target).is_done() {
                break;
            }
        }

        // Return that we at least attempted to compile something.
        Ok(true)
    }

    /// This id is used to create a file name that is discernable from the others.
    pub fn id<'a>(&'a self) -> Id<'a> {
        Id {
            name: self.repo.name(),
            branch: self.repo.branch(),
            hash: &self.commit.hash,
            date: &self.commit.date,
            path: &self.archive,
        }
    }

    /// Returns the size of the installed targets.
    pub fn read_disk_size(&self) -> [usize; 3] {
        [0, 1, 2].map(|i| CompilationTarget::from(i)).map(|target| {
            folder_size(&self.install_dir(target)).expect("failed to get size of install")
        })
    }
}
