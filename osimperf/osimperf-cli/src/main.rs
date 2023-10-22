#![feature(absolute_path)]

mod cli;
mod command;
mod common;
mod context;

pub use command::*;
pub use common::*;
pub use context::*;

use cli::{InstallCommand, ListCommand, RecordCommand, PlotCommand, LogCommand, ReadBenchTestSetup};

use log::info;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use env_logger::Env;

use anyhow::{Result, Context};

pub static INSTALL_INFO_FILE_NAME: &'static str = "osimperf-install-info.json";
pub static RESULT_INFO_FILE_NAME: &'static str = "osimperf-result-info.json";
pub static TEST_CONFIG_FILE_NAME: &'static str = "osimperf-test.config";

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "osimperf-cli")]
#[command(about = "OpenSim performance collector CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Helper for git log commands.
    Log(LogCommand),
    /// List relevant objects.
    Ls(ListCommand),
    /// Install dir.
    Install(InstallCommand),
    /// Record test result.
    ///
    /// Description: Reads path to test config from stdin.
    Record(RecordCommand),
    /// Plot
    Plot(PlotCommand),
    /// Write default test config file.
    #[command(arg_required_else_help = true)]
    WriteDefaultTestConfig { path: PathBuf },
}

fn main() -> Result<()> {
    do_main().context("main exited with error")?;
    Ok(())
}

fn do_main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let args = Cli::parse();

    match args.command {
        Commands::Log(mut args) => args.run()?,
        Commands::Ls(mut args) => args.run()?,
        Commands::Install(args) => args.run()?,
        Commands::Record(args) => args.run()?,
        Commands::Plot(args) => args.run()?,
        Commands::WriteDefaultTestConfig { path } => {
            write_default_json::<ReadBenchTestSetup>(&path)?
        }
    }

    Ok(())
}
