use crate::{CMakeCommands, Ctxt};
use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use osimperf_lib::git::{Commit, Date};
use std::path::PathBuf;

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
    /// Date %Y-%m-%d (defaults to today).
    #[arg(long)]
    date: Option<String>,
    /// Hash (defaults to last commit of the given date).
    #[arg(long)]
    hash: Option<String>,
}

// install --path PATH --hash HASH --date DATE

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

        let date = self
            .date
            .as_ref()
            .map(|d| Date::parse_from_str(d, "%Y-%m%d"))
            .transpose()
            .with_context(|| format!("failed to parse date {:?} to NaiveDate", self.date))?;

        let commit = match (date.as_ref(), self.hash.as_ref()) {
            (Some(d), Some(h)) => Commit {
                date: d.to_string(),
                hash: h.clone(),
            },
            (None, Some(h)) => Commit::new_from_hash(repo.path(), repo.branch(), h.clone())?,
            (None, None) => Commit::new_last_commit(repo.path(), repo.branch())?,
            (Some(d), None) => Commit::new_last_at_date(repo.path(), repo.branch(), d)?
                .with_context(|| format!("no commit at {:?}", self.date))?,
        };

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
