use crate::{read_json, CMakeCommands, Commit, Ctxt, Date, Repository};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
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
    /// Path to cmake config file.
    #[arg(long)]
    cmake: Option<PathBuf>,
    /// Commit date (in %Y-%m-%d format), or hash.
    #[arg(long, default_value = "2019-08-01")]
    commit: String,
    /// Compile last commits of the month since specified commit.
    #[arg(long)]
    monthly: bool,
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

        let cmake_config = self
            .cmake
            .as_ref()
            .map(|path| read_json::<CMakeCommands>(path))
            .unwrap_or(Ok(CMakeCommands::default()))?;

        let commit_arg = CommitArg::parse_arg(&self.commit)?;

        let mut commits = if self.monthly {
            repo.collect_monthly_commits(Some(&commit_arg.to_date(&repo)?), None)?
        } else {
            Vec::new()
        };

        if let Some(commit) = commit_arg.to_commit(&repo)? {
            commits.push(commit);
        }

        info!("Preparing to install commits: {:?}", commits);
        for commit in commits.drain(..) {
            let mut node = crate::install::CompilationNode::new(&context, repo.clone(), commit)?;
            info!("Installing node {:#?}", node);
            if node.install(&context, &cmake_config, true)? {
                info!("Install complete.");
            } else {
                info!("Already installed: Nothing to do.");
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum CommitArg<'a> {
    Hash(&'a str),
    Date(Date),
}

impl<'a> CommitArg<'a> {
    fn parse_arg(str_arg: &'a str) -> Result<Self> {
        Ok(if str_arg.chars().count() == 40 {
            // TODO check if valid.
            Self::Hash(str_arg)
        } else {
            Self::Date(Date::parse_from_str(str_arg, "%Y-%m-%d")?)
        })
    }

    fn to_date(&self, repo: &Repository) -> Result<Date> {
        Ok(match self {
            Self::Hash(hash) => repo.read_commit_from_hash(hash)?.date(),
            Self::Date(date) => date.clone(),
        })
    }

    fn to_commit(&self, repo: &Repository) -> Result<Option<Commit>> {
        Ok(match self {
            Self::Hash(hash) => Some(repo.read_commit_from_hash(hash)?),
            Self::Date(date) => repo.last_commit_at_date(date)?,
        })
    }
}
