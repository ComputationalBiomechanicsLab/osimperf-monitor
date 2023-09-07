use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{info, warn};
use osimperf_lib::{
    common::{read_config, write_default_config},
    *,
};
use std::{path::PathBuf, thread::sleep, time::Duration};

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,

    /// Specify path to cmake config. Defaults to compiler-flags/osimperf-cmake.conf
    #[arg(long)]
    pub cmake: Option<PathBuf>,

    /// Write a default cmake config file to a specified path.
    #[arg(long)]
    pub write_default_cmake_config: Option<PathBuf>,

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
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting OSimPerf-Monitor.");

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

    let cmake_config_path = args.cmake.clone().unwrap_or(
        home.path()?
            .join("compile-flags")
            .join("osimperf-cmake.conf"),
    );
    let cmake_config = read_config(&cmake_config_path)?;

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
    for param in Commit::collect_monthly_commits(&input, Some(&args.start_date), None)?.iter() {
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
    for param in Commit::collect_monthly_commits(&input, Some(&fine_start_date), None)?.iter() {
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
