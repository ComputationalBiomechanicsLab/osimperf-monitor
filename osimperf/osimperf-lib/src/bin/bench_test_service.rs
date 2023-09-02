use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use osimperf_lib::{
    bench_tests::{BenchTestSetup, TestNode},
    *,
};
use std::{thread::sleep, time::Duration};

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,

    #[arg(long, default_value = "2019-01-01")]
    pub start_date: String,

    /// Sleep time between loops in seconds.
    #[arg(long, default_value_t = 1)]
    pub sleep: u64,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    info!("Starting OSimPerf BenchTest Service.");

    let args = Args::parse();

    do_main(args).context("main exited with error")?;

    Ok(())
}

fn do_main(args: Args) -> Result<()> {
    // loop {
        info!("Enter test loop");
        if let Err(err) = do_main_loop(&args) {
            warn!("Loop exited with error: {:#}", err);
        }
        sleep(Duration::from_secs(args.sleep));
    // }
        Ok(())
}

fn do_main_loop(args: &Args) -> Result<()> {
    let home = Home::new_or_current(args.home.as_ref().map(|p| p.as_str()))?;
    let archive = home.default_archive()?;
    let results_dir = home.default_results()?;
    let tests_dir = home.path()?.join("tests");

    for node in CompilationNode::collect_archived(&archive)?.drain(..) {
        for setup in BenchTestSetup::find_all(&tests_dir)?.drain(..) {
            info!("Evaluating setup {:#?} at {:#?}", setup, node);
            if let Some(test) = TestNode::new(setup, node.clone(), &home, &results_dir)?.as_mut() {
                info!("Start bench test: {:#?}", test);
                let res = test.run()?;
                info!("{:?}", res);
                if res.failed_count > 0 {
                    warn!("Failed bench test: {:#?}", test);
                }
            } else {
                info!("-->Skipping test");
            }
        }
    }

    Ok(())
}
