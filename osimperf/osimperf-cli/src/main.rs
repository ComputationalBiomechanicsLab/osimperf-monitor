mod context;
mod install;
mod common;
mod file_backed_struct;

pub use context::*;
pub use common::*;
pub use install::*;
pub use file_backed_struct::*;

use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    Install {
        /// The remote to clone
        remote: String,
    },
    /// Record
    #[command(arg_required_else_help = true)]
    Record {
        /// Number of times to repeat succesful benchmark tests.
        #[arg(long, default_value_t = 50)]
        test_repeats: usize,
    },
}

impl Commands {
    fn get_context(&self) -> Result<Ctxt> {
        let mut context = Ctxt::default();
        match self {
            Commands::Ls { installed } => context.set_archive(installed.clone())?,
            Commands::Install { remote } => todo!(),
            Commands::Record { test_repeats } => todo!(),
        }
        Ok(context)
    }
}

fn main() -> Result<()> {
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
        Commands::Install { remote } => {
            println!("Cloning {remote}");
        }
        Commands::Record { test_repeats } => println!("record case {test_repeats} times"),
    }

    // Continued program logic goes here...
    Ok(())
}
