mod cli;
mod command;
mod common;
mod context;
mod file_backed_struct;
mod install;
mod record;

pub use command::*;
pub use common::*;
pub use context::*;
pub use file_backed_struct::*;
pub use install::*;

use cli::{InstallCommand, RecordCommand, ListCommand};

use log::info;
use record::ReadBenchTestSetup;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use env_logger::Env;

use anyhow::Result;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "git")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List relevant objects.
    Ls(ListCommand),
    /// Install dir.
    Install(InstallCommand),
    /// Record
    Record(RecordCommand),
    /// Write default cmake config file.
    #[command(arg_required_else_help = true)]
    WriteDefaultCmakeConfig { path: PathBuf },
    /// Write default test config file.
    #[command(arg_required_else_help = true)]
    WriteDefaultTestConfig { path: PathBuf },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let args = Cli::parse();

    match args.command {
        Commands::Ls(mut args) => args.run()?,
        Commands::Install(args) => args.run()?,
        Commands::Record(args) => args.run()?,
        Commands::WriteDefaultCmakeConfig { path } => write_default_json::<CMakeCommands>(&path)?,
        Commands::WriteDefaultTestConfig { path } => write_default_json::<ReadBenchTestSetup>(&path)?,
    }

    Ok(())
}
