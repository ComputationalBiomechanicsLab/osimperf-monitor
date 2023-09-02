use std::fs::File;

use anyhow::Result;
use env_logger::Env;
use log::info;
use osimperf_lib::bench_tests::table::print_results;
use osimperf_lib::bench_tests::BenchTestSetup;
use osimperf_lib::*;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting OSimPerf-compiler.");

    do_main()?;

    Ok(())
}

fn do_main() -> Result<()> {
    let home = Home::new("/home/pep/opensim/osimperf-monitor")?;
    let archive = Archive::new("/home/pep/opensim/osimperf-monitor/archive")?;

    let tests_dir = home.path()?.join("tests");
    let results_dir = ResultsFolder::new("/home/pep/opensim/osimperf-monitor/results")?;

    let nodes = CompilationNode::collect_archived(&archive)?;
    let tests = BenchTestSetup::find_all(&tests_dir)?;

    let mut file =
        File::create(home.path()?.join("results_table.data"))?;

    print_results(&nodes, &tests, &results_dir, &mut file)?;

    Ok(())
}
