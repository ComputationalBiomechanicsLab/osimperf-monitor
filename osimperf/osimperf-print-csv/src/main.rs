/// Script for generating csv files containing all benchmark results.
///
/// Run from osimperf-home folder, or specify osimperf-home using argument --home.
/// CSVs will be written to the results folder.

use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::info;
use osimperf_lib::{
    bench_tests::{print_csv, BenchTestSetup},
    CompilationNode, Folder, Home,
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting tool to print results to csv.");

    let args = Args::parse();

    do_main(args).context("main exited with error")?;

    Ok(())
}

fn do_main(args: Args) -> Result<()> {
    // Setup folders, read configs etc.
    let home = Home::new_or_current(args.home.as_ref().map(|p| p.as_str()))?;
    let archive = home.default_archive()?;
    let results_dir = home.default_results()?;
    let tests_dir = home.path()?.join("tests");

    let nodes = CompilationNode::collect_archived(&archive)?;
    info!("Found {} nodes:", nodes.len());
    for node in nodes.iter() {
        info!(
            "{}, {}, {}",
            node.repo.name(), node.commit.date, node.commit.hash
        );
    }

    let tests = BenchTestSetup::find_all(&tests_dir)?;
    info!("Found {} tests:", tests.len());
    for test in tests.iter() {
        info!("{}", test.name);
    }

    for test in tests.iter() {
        let path = print_csv(&nodes, test, &results_dir)?;
        info!("{} results printed to {:?}", test.name, path);
    }

    Ok(())
}
