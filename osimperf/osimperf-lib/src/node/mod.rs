mod cmake;
mod file;
mod focus;
mod installed_size;
mod repo;
mod status;

use anyhow::{Context, Result};
pub use cmake::*;
pub use file::NodeFile;
pub use focus::Focus;
pub use repo::*;
pub use status::{Complete, Progress, State, Status};

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::common::collect_configs;
use crate::git::Commit;
use crate::{erase_folder, Archive, BuildFolder, Folder, Home};

use self::installed_size::folder_size;
use log::trace;

pub fn path_to_install<'a>(focus: Focus, id: &Id<'a>) -> PathBuf {
    id.path().join(focus.to_str())
}

pub fn path_to_source(focus: Focus, home: &Home, repo: &RepositoryState) -> Result<PathBuf> {
    Ok(match focus {
        Focus::OpenSimCore => repo.path().to_owned(),
        Focus::Dependencies => repo.path().join("dependencies"),
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

    pub fn run(&mut self, home: &Home, build: &BuildFolder, config: &CMakeConfig) -> Result<bool> {
        // Check if the config changed since last time we compiled.
        let mut hasher = DefaultHasher::new();
        config.hash(&mut hasher);
        let hash = hasher.finish();
        let changed = hash != *self.config_hash.replace(hash).get_or_insert(hash);

        // If config changed, we need to recompile.
        if changed {
            self.state.reset();
        }

        // Returns true if there was any compilation being done.
        let mut ret = false;

        let checked_out = self.repo.checkout(&self.commit)?;

        // Go over compile targets: [dependencies, opensim-core, tests].
        for i in 0..3 {
            // Start compiling project.
            let focus = Focus::from(i);
            let install_dir = self.id().path().join(focus.to_str());

            if self.state.get()[i].should_compile() {
                ret = true;
                // || i == 2 { // TODO this will always recompile tests from source...

                // First update the status.
                self.state
                    .set(focus, Status::Compiling(Progress { percentage: 0. }));
                self.try_write()?;

                // Setup cmake commands.
                let cmd = CMakeCmds::new(&self.id(), &checked_out, home, build, config, focus)?;
                trace!("CMAKE COMMAND:\n{}", cmd.print_pretty());

                // Erase the install dir.
                erase_folder(&install_dir)
                    .with_context(|| format!("failed to erase install dir: {:?}", install_dir))?;

                // Erase the build dir.
                erase_folder(&build.path()?.join(focus.to_str()))
                    .with_context(|| format!("failed to erase build dir"))?;

                let mut progress = CMakeProgressStreamer::new(self, focus);

                // Start compilation.
                let output = cmd
                    .run(&mut progress)
                    .with_context(|| format!("cmake failed: {:#?}", cmd.print_pretty()));

                // Strange crashes happen if we evaluate the size here...
                let output = output.map(|duration| Complete { duration, size: 0 });

                // Update the status.
                self.state.set(focus, Status::from_output(output));
                self.try_write()?;
            }

            // Update install size... this was causing bugs if done right after install.... weird.
            if let Status::Done(Complete { duration, size: 0 }) = self.state.get()[i] {
                let size = folder_size(&install_dir).context("failed to get size of install")?;

                let updated = Ok(Complete {
                    duration: *duration,
                    size,
                });

                // Update the status.
                self.state.set(focus, Status::from_output(updated));
                self.try_write()?;
            }

            if !self.state.get()[i].is_done() {
                break;
            }
        }
        Ok(ret)
    }

    pub fn id<'a>(&'a self) -> Id<'a> {
        Id {
            name: self.repo.name(),
            branch: self.repo.branch(),
            hash: &self.commit.hash,
            date: &self.commit.date,
            path: &self.archive,
        }
    }
}
