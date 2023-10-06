mod command;
mod common;
mod context;
mod file_backed_struct;
mod install;

pub use command::*;
pub use common::*;
pub use context::*;
pub use file_backed_struct::*;
pub use install::*;

use log::info;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use env_logger::Env;

use anyhow::Result;

pub struct Context {}

impl Context {
    pub fn set_home(&mut self, home: Option<PathBuf>) -> Result<()> {
        todo!();
    }

    pub fn set_archive(&mut self, archive: Option<PathBuf>) -> Result<()> {
        todo!()
    }

    pub fn home(&self) -> Result<PathBuf> {
        todo!()
    }

    pub fn archive(&self) -> Result<PathBuf> {
        todo!()
    }
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "git")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<PathBuf>,

    /// Build dir.
    #[arg(long)]
    pub build: Option<PathBuf>,

    /// Archive dir.
    #[arg(long)]
    pub archive: Option<PathBuf>,

    /// Results dir.
    #[arg(long)]
    pub results: Option<PathBuf>,

    /// Test cases dir.
    #[arg(long)]
    pub test_cases: Option<PathBuf>,

    /// CMake config.
    #[arg(long)]
    pub cmake_config: Option<PathBuf>,

    /// Repo config.
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List stuff.
    #[command()]
    Ls {
        /// List all installed versions.
        #[arg(long, short)]
        installed: Option<PathBuf>,
    },
    /// Install dir.
    #[command(arg_required_else_help = true)]
    Install(InstallCommand),
    /// Write default cmake config.
    #[command(arg_required_else_help = true)]
    WriteDefaultCmakeConfig { path: PathBuf },
    /// Record
    #[command(arg_required_else_help = true)]
    Record {
        /// Number of times to repeat succesful benchmark tests.
        #[arg(long, default_value_t = 50)]
        test_repeats: usize,
    },
}

#[derive(Debug, Args)]
struct InstallCommand {
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
}

impl Commands {
    fn get_context(&self) -> Result<Ctxt> {
        let mut context = Ctxt::default();
        match self {
            Commands::Ls { installed } => context.set_archive(installed.clone())?,
            Commands::Record { test_repeats } => todo!(),
            _ => {}
        }
        Ok(context)
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Cli::parse();

    let context = args.command.get_context()?;

    match args.command {
        Commands::Ls { .. } => {
            let nodes = CompilationNode::collect_archived(&context)?;
            println!("Found {} compiled versions", nodes.len());
            for node in nodes {
                println!("node = {:#?}", node);
            }
        }
        Commands::WriteDefaultCmakeConfig { path } => write_default_json::<CMakeCommands>(&path)?,
        Commands::Install(args) => {
            run_install_cmd(&args)?;
        }
        Commands::Record { test_repeats } => println!("record case {test_repeats} times"),
    }

    // Continued program logic goes here...
    Ok(())
}

fn run_install_cmd(args: &InstallCommand) -> Result<()> {
    info!("Starting OSimPerf install command.");
    let context = args.get_context()?;

    let repo = crate::install::Repository::new_opensim_core(context.opensim_core().clone())?;

    let cmake_config = CMakeCommands::default();

    let commit = repo.last_commit()?;

    let mut node = crate::install::CompilationNode::new(&context, repo, commit)?;
    info!("Installing node {:#?}", node);
    if node.install(&context, &cmake_config)? {
        info!("Install complete.");
    } else {
        info!("Nothing to do.");
    }

    Ok(())
}
