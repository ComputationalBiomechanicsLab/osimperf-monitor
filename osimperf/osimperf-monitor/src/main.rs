pub mod bench_tests;
mod cmake;
mod cmd;
mod commit;
mod config;
mod folders;
mod git;
pub mod time;

pub use cmake::{compile_opensim_core, run_cmake_cmd, OSimCoreCmakeConfig};
pub use cmd::Command;
pub use commit::{collect_last_daily_commit, Commit};
pub use config::*;
pub use folders::Folders;

use anyhow::{ensure, Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{debug, info, trace, warn};
use std::io;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub home: Option<String>,

    #[arg(long, default_value = ".osimperf.conf")]
    pub config: String,

    #[arg(long, default_value = "software/opensim-core")]
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

    #[arg(long, default_value = "2023/07/18")]
    pub start_date: String,

    #[arg(long)]
    pub write_default_config: bool,

    #[arg(long)]
    pub force_remove_archive: bool,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"));

    do_main().context("main exited with error")
}

fn do_main() -> Result<()> {
    info!("Starting OSimPerf-Monitor.");

    let args = Args::parse();
    debug!("Command line arguments:\n{:#?}", args);

    let folders = Folders::new(&args)?;
    debug!("Folder layout:\n{:#?}", folders);

    let compile_flags_path = folders.home.join(cmake::CMAKE_CONFIG_FILE);
    if args.write_default_config {
        write_default_config::<OSimCoreCmakeConfig>(&compile_flags_path)?; // TODO change all strings to Paths, and PathBuf to Path
        info!(
            "Default compilation flags written to: {:?}",
            compile_flags_path
        );
        return Ok(());
    }

    debug!("Reading compilation flags from = {:#?}", compile_flags_path);
    let compile_flags = read_config(&compile_flags_path)?;
    trace!("Compilation flags: {:#?}", compile_flags);

    let tests = bench_tests::read_perf_test_setup(&folders)?;
    info!("Found {} benchmark tests: ", tests.len());
    for t in tests.iter() {
        info!("    {:#?}", t.name);
    }

    // Switch to main branch on opensim-core repo, and pull.
    info!("Switching {:?} to main branch.", folders.opensim_core);
    git::switch_opensim_core_to_main(&folders)
        .context("Failed to switch opensim-core to main branch.")?;

    debug!("Start collecting opensim-core versions (commits) for compiling");
    let commits: Vec<Commit> = collect_last_daily_commit(&folders, &args.start_date)?;
    info!("Start compiling {} versions of opensim", commits.len());
    for c in commits.iter() {
        debug!("    {:#?}", c.date);
    }

    if args.force_remove_archive {
        warn!("Removing archive.");
        for c in commits.iter() {
            debug!("Removing: {:?}", c.get_archive_folder(&folders));
            c.remove_archive_dir(&folders)?;
            c.create_archive_dir(&folders)?;
        }
    }

    for c in commits.iter() {
        println!("Start installing opensim-core {:?}", c);

        if c.verify_compiled_version(&folders)? {
            println!("    Found previous installation.\n");
            continue;
        }

        println!("Start compilation of {:?}", c);

        // Switch opensim-core repo to correct commit.
        git::checkout_commit(&folders.opensim_core, c)?;

        println!("Preparing fresh build directory.");
        c.remove_archive_dir(&folders)?;
        c.create_archive_dir(&folders)?;

        match compile_opensim_core(&folders, c, &compile_flags) {
            Err(err) => println!(
                "Error:\n{:?}\nFailed to compile opensim core ( {:?} )",
                err, c
            ),
            Ok(compile_times) => {
                ensure!(c.verify_compiled_version(&folders)?,
                    format!("Post install check failed: Failed to verify version of installed opensim-cmd for {:?}", c));
                println!("Succesfully compiled opensim core ( {:?} )", c);
            }
        }
    }

    for c in commits.iter() {
        // Update all results --> should be swapped out instead.
        c.remove_results_dir(&folders)?;
        c.create_results_dir(&folders)?;

        for _ in 0..2 {
            for t in tests.iter() {
                let test_result = bench_tests::run_test(&folders, t, c)
                    .context("Failed to run test")?;
                bench_tests::update_test_result(&folders, t, c, test_result)
                    .context("failed to update test result")?;
                return Ok(());
            }
        }
    }

    let mut stdout = io::stdout().lock();
    bench_tests::table::print_results(&folders, &commits, &tests, &mut stdout)?;

    return Ok(());
}
