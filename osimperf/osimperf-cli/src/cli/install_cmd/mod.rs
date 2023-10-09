use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{anyhow, Context, Result };
use log::info;
use crate::{Ctxt, CMakeCommands};

#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Path to opensim-core repo.
    #[arg(long)]
    opensim_core: Option<PathBuf>,
    /// Path to archive directory.
    #[arg(long)]
    archive: Option<PathBuf>,
    /// Path to build directory.
    #[arg(long)]
    build: Option<PathBuf>,
}

impl InstallCommand {
    fn get_context(&self) -> Result<Ctxt> {
        let mut context = Ctxt::default();
        context.set_opensim_core(self.opensim_core.clone())?;
        context.set_archive(self.archive.clone())?;
        context.set_build(self.build.clone())?;
        Ok(context)
    }

    pub fn run(&self) -> Result<()> {
        info!("Starting OSimPerf install command.");
        let context = self.get_context()?;

        let repo = crate::install::Repository::new_opensim_core(context.opensim_core().clone())?;

        let cmake_config = CMakeCommands::default();

        let commit = repo.last_commit()?;

        let mut node = crate::install::CompilationNode::new(&context, repo, commit)?;
        info!("Installing node {:#?}", node);
        if node.install(&context, &cmake_config, true)? {
            info!("Install complete.");
        } else {
            info!("Nothing to do.");
        }

        Ok(())
    }
}
