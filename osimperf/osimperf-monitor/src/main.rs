use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{debug, info, trace, warn};
use osimperf_lib::{
    bench_tests::{BenchTestSetup, TestNode},
    common::{duration_since_boot, read_config, write_default_config},
    Archive, BioLabRepositoryConfig, CMakeConfig, CMakeConfigReader, CompilationNode, Folder, Home,
    NodeFile, Repository, RepositoryConfig, CompilationTarget,
};
use rand::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,

    /// Write a default cmake config file to a specified path.
    #[arg(long)]
    pub write_default_cmake_config: Option<PathBuf>,

    /// The first version we will attempt to compile.
    #[arg(long, default_value = "2019-01-01")]
    pub start_date: String,

    /// Set this to enable daily compilation, after completing the monthly.
    #[arg(long)]
    pub daily: bool,

    /// Period in minutes between polling the repository for latest commits.
    #[arg(long)]
    pub pull_period: Option<u64>,

    /// Max consecutive compilation failures.
    #[arg(long, default_value_t = 5)]
    pub max_compile_fail: usize,

    /// Number of times to repeat succesful benchmark tests.
    #[arg(long, default_value_t = 50)]
    pub test_repeats: usize,

    /// Number of times to repeat a benchmark test before giving up.
    #[arg(long, default_value_t = 2)]
    pub max_test_fail: usize,

    /// Set this to write intermediate results to file during benchmarking.
    #[arg(long)]
    pub write_intermediate_results: bool,

    /// Number of test cycles that are ignored, before recording results.
    #[arg(long, default_value_t = 2)]
    pub warm_start_buffer: usize,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting OSimPerf-Monitor");

    let args = Args::parse();

    do_main(args).context("main exited with error")?;

    Ok(())
}

fn do_main(args: Args) -> Result<()> {
    if let Some(path) = args.write_default_cmake_config.as_ref() {
        write_default_config::<CMakeConfig>(path)?;
        return Ok(());
    }

    loop {
        info!("Start monitor loop");
        do_main_loop(&args)?;
    }
}

fn do_main_loop(args: &Args) -> Result<()> {
    // Setup folders, read configs etc.
    let home = Home::new_or_current(args.home.as_ref().map(|p| p.as_str()))?;
    let build = home.default_build()?;
    let archive = home.default_archive()?;
    let results_dir = home.default_results()?;
    let tests_dir = home.path()?.join("tests");

    let cmake_config = CMakeConfigReader::read(&home)?;
    info!("compile flags = {:#?}", cmake_config);

    let mut repo = RepositoryConfig::default().take(&home)?;
    debug!("OpenSim repo = {:#?}", repo);

    // Check if there are any other repositories to follow.
    let mut biolab = read_config::<BioLabRepositoryConfig>(
        &home
            .path()?
            .join("compile-flags")
            .join("osimperf-biolab-targets.conf"),
    )
    .map(|x| x.take(&home).expect("failed to verify repository"))
    .unwrap_or(Vec::new());

    // Loop:
    // 1. Warm start.
    // 2. Do X benchmark tests.
    // 3. Do one compilation.
    // 4. Goto step 1.
    let mut last_pull = None;
    let mut rng = rand::thread_rng();
    loop {
        // Run the benchmark tests.
        let nodes = CompilationNode::collect_archived(&archive)?;
        let test_setups = BenchTestSetup::find_all(&tests_dir)?;
        for node in nodes.iter() {
            let mut tests = Vec::new();
            for setup in test_setups.iter() {
                // Creating the test node also sets up the context.
                if let Some(test) =
                    TestNode::new(&setup, &node, &home, &results_dir, args.warm_start_buffer)?
                {
                    tests.push(test);
                }
            }

            let mut count = 0;
            while tests.len() > 0 {
                // Dropping tests triggers post benchmark cmds.
                tests.retain(|t| t.should_run(args.test_repeats, args.max_test_fail));
                tests.shuffle(&mut rng);
                count +=1;
                info!("count = {count}");

                for test in tests.iter_mut() {
                    info!("running = {}", test.name());
                    trace!("Start bench test: {:#?}", test);
                    let res = test.run()?;
                    if res.failed_count > 0 {
                        trace!("Failed bench test: {:#?}", test);
                    }
                    if args.write_intermediate_results {
                        test.try_write()?;
                    }
                }
            }
        }

        // Pull latest changes to opensim.
        if let Some(pull_period) = args.pull_period {
            let dt = duration_since_boot().context("Failed to read system clock")?;
            let prev_dt = last_pull.get_or_insert(dt);
            if (dt - *prev_dt).as_secs() / 60 > pull_period {
                *prev_dt = dt;
                info!("Pull latest OpenSim-Core");
                info!("{}", repo.pull()?);
                for r in biolab.iter_mut() {
                    info!("Pull latest Computational Biomechanics");
                    info!("{}", r.pull()?);
                }

                garbage_collector(&archive, &repo)?;
            }
        }

        // Start compiling opensim.

        // Continue from the top after compiling a single node.
        let mut compiled_a_node = false;

        // First consider any external biolab repo.
        for i in 0..biolab.len() {
            let commit = biolab[i].last_commit()?;
            let mut node = CompilationNode::new(biolab[i].clone(), commit, &archive)?;

            let config = cmake_config.get(&node.commit.date()?);
            compiled_a_node |= node.run(&home, &build, config)?;

            // Stop after a single compilation.
            if compiled_a_node {
                break;
            }
        }

        // Go back to the benchmarking after a succesful compilation.
        if compiled_a_node {
            continue;
        }

        // Keep going back in time until failing to compile for a number of consecutive times.
        let mut failed_count = 0;
        // Take larger monthly versions, and record the date from which we can still compile.
        let mut ok_start_date = None;

        for commit in repo
            .collect_monthly_commits(Some(&args.start_date), None)?
            .drain(..)
        {
            let mut node = CompilationNode::new(repo.clone(), commit, &archive)?;

            debug!("Start compiling monthly {:#?}", node);
            let config = cmake_config.get(&node.commit.date()?);
            compiled_a_node |= node.run(&home, &build, &config)?;

            // Stop compiling if we failed compiling opensim-core X times in a row.
            if !node.state.status(CompilationTarget::OpenSimCore).is_done() {
                failed_count += 1;
            } else {
                // Reset counter.
                failed_count = 0;
                ok_start_date = Some(node.commit.date.clone());
            }
            if failed_count > args.max_compile_fail {
                debug!("Failed {failed_count} times in a row, stopping");
                break;
            }

            // Stop after having compiled a node.
            if compiled_a_node {
                break;
            }
        }

        if !args.daily || compiled_a_node {
            continue;
        }

        let fine_start_date = if let Some(date) = ok_start_date {
            date
        } else {
            warn!("Not one compilation succeeded. Skipping daily compilation.");
            continue;
        };

        // Now do another finer Daily commits compilation.
        for commit in repo
            .collect_daily_commits(Some(&fine_start_date), None)?
            .drain(..)
        {
            let mut node = CompilationNode::new(repo.clone(), commit, &archive)?;

            debug!("Start compiling daily {:#?}", node);
            let config = cmake_config.get(&node.commit.date()?);
            compiled_a_node |= node.run(&home, &build, &config)?;
            if compiled_a_node {
                break;
            }
        }
    }
}

fn garbage_collector(archive: &Archive, repo: &Repository) -> Result<()> {
    // Cleanup archive:
    // We want to have a daily version installed: but we pull periodically per day.
    // This means that we might install different versions
    let mut stray_nodes = CompilationNode::collect_archived(&archive)?;
    let daily_commits = repo.collect_daily_commits(None, None)?;
    // Look for nodes that have a duplicate day, and are not part of the daily commits
    // list. We do not want to accidentally delete stuff from the daily commit list,
    // nor do we want to remove something unique.
    while stray_nodes.len() > 0 {
        let check_node = stray_nodes.remove(0);
        // Check if there are multiple nodes on the same day.
        let mut duplicate_day = stray_nodes
            .iter()
            .filter(|n| n.commit.date == check_node.commit.date)
            .count()
            > 1;
        // Check if it is not part of the daily list.
        duplicate_day &= daily_commits
            .iter()
            .filter(|n| n.date == check_node.commit.date)
            .count()
            != 1;
        // This node is OK, we keep it.
        if !duplicate_day {
            continue;
        }
        // Delete the stray node.
        check_node.delete_folder()?;
        // TODO delete stray test results.
    }
    Ok(())
}
