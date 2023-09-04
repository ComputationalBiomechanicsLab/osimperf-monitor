use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use osimperf_lib::*;
use std::{thread::sleep, time::Duration};

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,

    #[arg(long, default_value = "2019-01-01")]
    pub start_date: String,

    #[arg(long)]
    pub daily: bool,

    /// Sleep time between loops in seconds.
    #[arg(long, default_value_t = 900)]
    pub sleep: u64,

    /// Max consecutive compilation failures.
    #[arg(long, default_value_t = 4)]
    pub max_fail: usize,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting OSimPerf-Monitor.");

    let args = Args::parse();

    do_main(args).context("main exited with error")?;

    Ok(())
}

fn do_main(args: Args) -> Result<()> {
    loop {
        info!("Start monitor loop");
        if let Err(err) = do_main_loop(&args) {
            warn!("Monitor loop exited with error: {:#}", err);
        }
        sleep(Duration::from_secs(args.sleep));
    }
}

fn do_main_loop(args: &Args) -> Result<()> {
    let home = Home::new_or_current(args.home.as_ref().map(|p| p.as_str()))?;
    let build = home.default_build()?;
    let archive = home.default_archive()?;

    let cmake_config = CMakeConfig::default();
    info!("compile flags = {:#?}", cmake_config);

    let input = Input {
        repo: home.path()?.join("software/opensim-core"),
        url: OPENSIM_CORE_URL.to_string(),
        branch: "main".to_string(),
        name: "opensim-core".to_string(),
    };

    // Use this list to check for any stray nodes that ended up in the archive. We can then delete
    // these later.
    let mut stray_nodes: Vec<CompilationNode> = CompilationNode::collect_archived(&archive)?
        .drain(..)
        .collect();

    // Keep going back in time until failing to compile for a number of consecutive times.
    let mut failed_count = 0;
    let mut ok_start_date = None;
    for param in Params::collect_monthly_commits(&input, Some(&args.start_date), None)?.iter() {
        let mut node = CompilationNode::new(input.clone(), param.clone(), &archive)?;

        // Maintain the list of stray nodes.
        remove_duplicates_from_vec(&node, &mut stray_nodes, |x, y| x.repo.hash == y.repo.hash);

        info!("Start compiling monthly {:#?}", node);

        let res = node.run(&home, &build, &cmake_config)?;
        info!("node = {:#?}", node);
        if !res {
            failed_count += 1;
        } else {
            // Reset counter.
            failed_count = 0;
            ok_start_date = Some(param.date.clone());
        }
        if failed_count > args.max_fail {
            info!("Failed {failed_count} times in a row, stopping");
            break;
        }
    }

    if !args.daily {
        return Ok(());
    }

    let fine_start_date = if let Some(date) = ok_start_date {
        date
    } else {
        info!("Not one compilation succeeded. Skipping daily compilation.");
        return Ok(());
    };

    // Now do another finer Daily commits compilation.
    for param in Params::collect_monthly_commits(&input, Some(&fine_start_date), None)?.iter() {
        let mut node = CompilationNode::new(input.clone(), param.clone(), &archive)?;

        // Maintain the list of stray nodes.
        remove_duplicates_from_vec(&node, &mut stray_nodes, |x, y| x.repo.hash == y.repo.hash);

        info!("Start compiling daily {:#?}", node);
        if !node.run(&home, &build, &cmake_config)? {
            warn!("Failed to compile daily {:#?}", node);
        }
    }

    // Finally, remove the stray nodes.
    for node in stray_nodes.iter() {
        warn!("Found stray install {:#?}", node);
        let delete_dir = node.path_to_self();
        warn!("Removing directory {:?}", delete_dir);
        // remove_dir_all(&delete_dir); TODO uncomment this to delete stray nodes.
    }

    Ok(())
}

fn remove_duplicates_from_vec<T, P>(x: &T, vec: &mut Vec<T>, pred: P)
where
    P: Fn(&T, &T) -> bool,
{
    // Maintain the list of stray nodes.
    while let Some((i, _)) = vec.iter().enumerate().find(|(_, &ref y)| pred(x, y)) {
        vec.remove(i);
    }
}
