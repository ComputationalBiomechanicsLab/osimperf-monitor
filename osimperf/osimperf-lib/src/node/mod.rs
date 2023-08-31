mod compile;
mod file;
mod focus;
mod repo;
mod status;

use anyhow::{Context, Result};
pub use compile::*;
pub use file::NodeFile;
pub use focus::Focus;
pub use repo::*;
pub use status::State;

use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    collect_configs, erase_folder, node::status::Status, Archive, BuildFolder, Folder, Home,
};

use self::status::Progress;
use log::debug;

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

    pub fn run(&mut self, home: &Home, build: &BuildFolder, config: &CMakeConfig) -> Result<bool> {
        let mut progress = ProgressStreamer::default();

        // Go over compile targets: [dependencies, opensim-core, tests].
        for i in 0..3 {
            if self.state.get()[i].should_compile()
                || i == 2 { // TODO this will always recompile tests from source...

                // Start compiling project.
                let focus = Focus::from(i);

                // First update the status.
                self.state.set(
                    focus,
                    Status::Compiling(Progress {
                        percentage: 0.,
                        process: "start compiling".to_string(),
                    }),
                );
                self.try_write()?;

                // Setup cmake commands.
                let cmd =
                    CMakeCmds::new(&self.id(), &self.repo.source(), home, build, config, focus)?;
                debug!("CMAKE COMMAND:\n{}", cmd.print_pretty());

                // Switch to correct commit (only switches if not there already).
                self.repo.source().checkout()?;

                // Erase the install dir.
                let install_dir = self.id().path().join(focus.to_str());
                erase_folder(&install_dir)
                    .with_context(|| format!("failed to erase install dir: {:?}", install_dir))?;

                // Erase the build dir.
                erase_folder(&build.path()?.join(focus.to_str()))
                    .with_context(|| format!("failed to erase build dir"))?;

                // Start compilation.
                let output = cmd
                    .run(&mut progress)
                    .with_context(|| format!("cmake failed: {:#?}", cmd.print_pretty()));

                // Update the status.
                self.state.set(focus, Status::from_output(output));
                self.try_write()?;
            }

            if !self.state.get()[i].is_done() {
                break;
            }
        }
        Ok(self.state.get().iter().all(|x| x.is_done()))
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
