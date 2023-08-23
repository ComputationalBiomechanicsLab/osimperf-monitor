pub mod bench_tests;
pub mod time;
// mod cleanup;
mod cmake;
mod cmd;
mod commit;
mod config;
mod folders;
mod git;

use std::path::{Path, PathBuf};

// pub use cleanup::{cleanup, create_build_dir};
pub use cmake::{compile_opensim_core, run_cmake_cmd, OSimCoreCmakeConfig};
pub use cmd::Command;
pub use commit::{collect_commits_to_test, Commit};
pub use config::*;
pub use folders::Folders;
pub use git::{checkout_commit, switch_main};

use anyhow::{ensure, Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub home: Option<String>,

    #[arg(long, default_value = ".osimperf.conf")]
    pub config: String,

    #[arg(long, default_value = "software/opensim-core-main")]
    pub opensim_core: String,

    #[arg(long, default_value = "throwaway")]
    pub throwaway: String,

    #[arg(long, default_value = "source")]
    pub source: String,

    #[arg(long, default_value = "scripts")]
    pub scripts: String,

    #[arg(long, default_value = "archive")]
    pub archive: String,

    #[arg(long, default_value = "tests")]
    pub tests: String,

    #[arg(long, default_value = "results")]
    pub results: String,

    #[arg(long, default_value = "2023/07/01")]
    pub start_date: String,

    #[arg(long)]
    pub write_default_config: bool,

    #[arg(long)]
    pub force_remove_archive: bool,
}

fn main() -> Result<()> {
    println!("Run simperf.");
    do_main().context("main exited with error")
}

fn do_main() -> Result<()> {
    let args = Args::parse();
    println!("args = {:#?}", args);

    let folders = Folders::new(&args)?;

    let path_to_cmake_config = folders.home.join(Path::new(cmake::CMAKE_CONFIG_FILE));
    if args.write_default_config {
        write_default_config::<OSimCoreCmakeConfig>(&path_to_cmake_config)?; // TODO change all strings to Paths, and PathBuf to Path
        println!("Default config written to: {:?}", path_to_cmake_config);
        return Ok(());
    }

    let tests = bench_tests::read_perf_test_setup(&folders)?;
    println!("Found {} benchmark tests: ", tests.len());
    for t in tests.iter() {
        println!("    {:#?}", t.name);
    }

    // Switch to main branch on opensim-core repo, and pull.
    git::switch_main(Path::new(&folders.opensim_core))?;
    let commits: Vec<Commit> = collect_commits_to_test(&folders, &args.start_date)?;

    println!("Start compiling {} versions of opensim", commits.len());
    for c in commits.iter() {
        println!("    {:#?}", c.date);
    }

    if args.force_remove_archive {
        for c in commits.iter() {
            println!("!!!REMOVING ARCHIVE!!!");
            panic!();
            c.remove_archive_dir(&folders)?;
            c.create_archive_dir(&folders)?;
        }
    }

    let opensim_config = read_config(&path_to_cmake_config)?;
    println!("config = {:#?}", opensim_config);

    println!("\nStart compilation step");
    for c in commits.iter() {
        println!("Start installing opensim-core {:?}", c);

        if c.verify_compiled_version(&folders)? {
            println!("    Found previous installation.\n");
            continue;
        }

        println!("Start compilation of {:?}", c);
        panic!();

        // Switch opensim-core repo to correct commit.
        git::checkout_commit(&folders.opensim_core, c)?;

        println!("Preparing fresh build directory.");
        c.remove_archive_dir(&folders)?;
        c.create_archive_dir(&folders)?;

        let mut log = String::new();
        match compile_opensim_core(&folders, c, &opensim_config, &mut log) {
            Err(err) => println!(
                "Error:\n{:?}\nFailed to compile opensim core ( {:?} )",
                err, c
            ),
            Ok(()) => {
                ensure!(c.verify_compiled_version(&folders)?,
                    format!("Post install check failed: Failed to verify version of installed opensim-cmd for {:?}", c));
                println!("Succesfully compiled opensim core ( {:?} )", c);
            }
        }
    }

    let mut log = String::new();
    for c in commits.iter() {
        // Update all results --> should be swapped out instead.
        c.remove_results_dir(&folders)?;
        c.create_results_dir(&folders)?;

        for i in 0..1 {
            for t in tests.iter() {
                let test_result = bench_tests::run_test(&folders, t, c, &mut log)
                    .context("Failed to run test")?;
                bench_tests::update_test_result(&folders, t, c, test_result)
                    .context("failed to update test result")?;
                return Ok(());
            }
        }
    }

    return Ok(());
}

// TODO compilation:
// - parse percentage to a bar [====>***]
// - store build logs somewhere

// TODO Dashboard: table
// - status of versions (compiling, broken, etc)

// TODO Add opensim lab as thing to test against.

// Compile opensim-core versions
// Compile benchtests-source

// Run bench tests -> means running profiler!
// Generate table with compilation overview

// Add install script: folder layout, opensim-core submodule,

// Add bench tests: Raja, IK, Basic millard,

// Add web-interface
// - status (compiling)
// - blacklisted
//
// version | status | hopper | rajagopal | ...

// Add ubuntu package
