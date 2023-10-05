use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    /// Clones repos
    #[command(arg_required_else_help = true)]
    Install {
        /// The remote to clone
        remote: String,
    },
    /// List stuff.
    #[command()]
    Ls {
        /// List all installed versions.
        #[arg(long, short)]
        installed: bool,
    },
    /// Record
    #[command(arg_required_else_help = true)]
    Record {
        /// Number of times to repeat succesful benchmark tests.
        #[arg(long, default_value_t = 50)]
        test_repeats: usize,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Ls{..} => {
            println!("List all test cases")
        },
        Commands::Install { remote } => {
            println!("Cloning {remote}");
        }
        Commands::Record { test_repeats } => println!("record case {test_repeats} times"),
    }

    // Continued program logic goes here...
}
