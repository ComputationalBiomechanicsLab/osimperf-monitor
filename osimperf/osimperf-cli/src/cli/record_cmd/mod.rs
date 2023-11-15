use super::absolute_path;
use super::InstallInfo;

use crate::{read_json, write_json, Command, CommandTrait, Durations, RESULT_INFO_FILE_NAME};
use anyhow::{Context, Result};
use clap::Args;
use log::log_enabled;
use log::trace;
use log::warn;
use log::{debug, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{path::PathBuf, str::FromStr};

/// OSimPerf record command for running benchmark tests.
///
/// Takes path to benchmark configuration file.
/// Runs specified commands from that directory and creates `osimperf-result-ID/osimperf-result-info.json`
/// Uses PATH to find `osimperf-install-info`, which must match `opensim-cmd --version`
#[derive(Debug, Args)]
pub struct RecordCommand {
    /// Number of test iterations.
    #[arg(long, short)]
    iter: Option<usize>,

    /// Path to benchmark config file, or directory.
    #[arg(long, short)]
    config: Option<PathBuf>,

    /// Use valgrind on test.
    #[arg(long, short)]
    grind: bool,

    /// Force retesting.
    #[arg(long, short)]
    force: bool,

    /// Print the benchmark command.
    #[arg(long, short)]
    print: bool,

    /// Run visualization command (if present).
    #[arg(long, short)]
    visualize: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultInfo {
    /// Test case name.
    pub name: String,
    /// Cell name.
    pub cell_name: Option<String>,
    /// Opensim install name.
    pub opensim_name: String,
    /// Opensim-core commit hash.
    pub commit: String,
    /// Opensim-core commit date.
    pub date: String,
    /// Already ran the pre_benchmark_cmds.
    pub setup: bool,
    /// Benchmark durations.
    pub durations: Durations,
    /// Benchmark grind result.
    pub grind: Option<std::time::Duration>,
    /// Test config hash.
    pub config_hash: u64,
}

impl ResultInfo {
    pub fn filename() -> &'static str {
        RESULT_INFO_FILE_NAME
    }
}

#[derive(Debug)]
struct BenchTestCtxt {
    pub result_dir: PathBuf,
    pub pre_benchmark_cmds: Vec<Command>,
    pub benchmark_cmd: Command,
    pub grind_cmd: Command,
    pub visualize_cmd: Option<Command>,
    pub output: ResultInfo,
    pub repeats: usize,
}

impl RecordCommand {
    pub fn run(&self) -> Result<()> {
        info!("Start OSimPerf record command");

        // Prefix linker path.
        {
            let mut prefix_path = Command::parse("osimperf-install-info root").run_trim()?;
            prefix_path.push_str("/lib");
            super::prefix_path(&["LD_LIBRARY_PATH"], &prefix_path)?;

            debug!(
                "Using path env:\nPATH={}\nLD_LIBRARY_PATH={}",
                std::env::var("PATH")?,
                std::env::var("LD_LIBRARY_PATH")?
            );
        }

        let install_info = InstallInfo::try_read("osimperf-install-info")?;
        debug!("{:?}", install_info);

        let mut tests = Vec::new();

        // Read test paths from stdin if no --test arg was given.
        let mut lines_opt = None;

        loop {
            // Get path to test config file.
            let config_path = if let Some(path) = self.config.as_ref() {
                // Use path given as argument.
                absolute_path(path)
            } else {
                let lines = lines_opt.get_or_insert_with(|| std::io::stdin().lines());
                // Otherwise read paths from stdin.
                if let Some(l) = lines.next() {
                    let s = l?;
                    absolute_path(&PathBuf::from_str(&s)?)
                } else {
                    break;
                }
            }?;

            // Read test case setup file.
            let config = read_json::<ReadBenchTestSetup>(&config_path)?;

            // Directory containing the config is used as root for running the benchmark.
            let root_dir = config_path.parent().unwrap();

            // Create subdirectory for placing results from this record.
            let result_dir =
                PathBuf::from(Command::parse("osimperf-install-info root").run_trim()?)
                    .join("results")
                    .join(&config.name);

            // Path to result-info file, placed in results subdirectory.
            let result_info_path = result_dir.join(RESULT_INFO_FILE_NAME);

            // Detect changes in test configuration.
            let mut hasher = DefaultHasher::new();
            config.hash(&mut hasher);
            let config_hash = hasher.finish();

            // Read any previous result, if it exists.
            let result_info = read_json::<ResultInfo>(&result_info_path)
                .ok()
                .filter(|r| r.commit == install_info.commit)
                .filter(|r| r.config_hash == config_hash)
                .unwrap_or(ResultInfo {
                    name: config.name.clone(),
                    opensim_name: install_info.name.clone(),
                    commit: install_info.commit.clone(),
                    date: install_info.date.clone(),
                    durations: Default::default(),
                    grind: None,
                    config_hash,
                    setup: false,
                    cell_name: config.cell_name.clone(),
                });

            // Setup pre-benchmark, benchmark, grind, and visualize commands for this benchmark.

            let pre_benchmark_cmds = parse_commands(&config.pre_benchmark_cmds)
                .drain(..)
                .map(|c| c.set_run_root(&root_dir))
                .collect::<Vec<Command>>();

            let benchmark_cmd = Command::parse(&config.benchmark_cmd).set_run_root(&root_dir);

            let grind_cmd_base = "valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --cache-sim=yes --branch-sim=yes";
            let grind_cmd = Command::parse(&super::substitute_install_info(format!(
                "{grind_cmd_base} --callgrind-out-file={}/callgrind.out.%n_%H {}",
                result_dir.to_str().unwrap(),
                config.benchmark_cmd
            )))
            .set_run_root(&root_dir);

            let visualize_cmd = config
                .visualize_cmd
                .as_ref()
                .map(|s| Command::parse(s).set_run_root(&root_dir));

            // Collext benchmark info.
            tests.push(BenchTestCtxt {
                pre_benchmark_cmds,
                benchmark_cmd,
                grind_cmd,
                visualize_cmd,
                output: result_info,
                result_dir,
                repeats: self.iter.unwrap_or(config.repeats.unwrap_or(3)),
            });

            // Break if --test argument was used, otherwise continue reading from stdin.
            if self.config.is_some() {
                break;
            }
        }

        // Setup test context using pre-benchmark commands.
        run_all_pre_benchmark_commands(
            tests.iter().filter(|test| !test.output.setup | self.force),
        )?;
        // Update ResultInfo file such that setup is done once.
        for test in tests.iter_mut() {
            test.output.setup = true;
            write_json(&test.result_dir.join(RESULT_INFO_FILE_NAME), &test.output)?;
        }

        // If --print argument was set: Print relevant command to stdout.
        if self.print {
            for test in tests.iter() {
                let cmd = if self.visualize {
                    test.visualize_cmd
                        .as_ref()
                        .expect("no visualize command found")
                } else if self.grind {
                    &test.grind_cmd
                } else {
                    &test.benchmark_cmd
                };
                debug!("{} command:", test.output.name);
                println!(
                    "PATH={} LD_LIBRARY_PATH={} {}",
                    std::env::var("PATH")?,
                    std::env::var("LD_LIBRARY_PATH")?,
                    cmd.print_command()
                );
            }
            return Ok(());
        }

        // If --visualize argument was set: Run visualization.
        if self.visualize {
            if tests.len() == 0 {
                info!("Nothing to show.");
                return Ok(());
            }

            let mut msg = String::from("Prepare to visualize benchmarks:");
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.output.name);
            });
            info!("{msg}");

            for test in tests.iter() {
                let cmd = test
                    .visualize_cmd
                    .as_ref()
                    .expect("no visualize command found");

                cmd.run_trim()?;
            }

            return Ok(());
        }

        if self.grind {
            if !self.force {
                tests.retain(|t| t.output.grind.is_none());
            }

            if tests.len() == 0 {
                info!("Nothing to grind");
                return Ok(());
            }

            let mut msg = String::from("Prepare to grind benchmarks:");
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.output.name);
                if log_enabled!(log::Level::Debug) {
                    msg.push_str("\n");
                    msg.push_str(&t.grind_cmd.print_command());
                }
            });
            info!("{msg}");

            for test in tests.iter_mut() {
                let output = test.grind_cmd.run_and_time()?;
                let dt = *test.output.grind.insert(output.duration);

                // Store results.
                write_json(&test.result_dir.join(RESULT_INFO_FILE_NAME), &test.output)?;
                info!(
                    "Completed grinding {} in {}",
                    test.output.name,
                    dt.as_secs()
                );
            }

            info!("Grind complete.");
        }

        if true {
            // Filter tests that are complete.
            if !self.force {
                tests.retain(|t| t.output.durations.len() < t.repeats);
            }

            if tests.len() == 0 {
                info!("Nothing to test");
                return Ok(());
            }

            // Print list of tests that will be ran.
            let mut msg = format!("Prepare to run benchmarks:");
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.output.name);
                msg.push_str(&format!(" ({}X)", t.repeats));
            });
            info!("{msg}");

            // Reset any previous measurements.
            for test in tests.iter_mut() {
                test.output.durations = Default::default();
            }

            // Run tests repeatedly.
            let mut rng = rand::thread_rng();
            while tests.iter().filter(|t| t.output.durations.len() < t.repeats).count() > 0 {
                // Randomize test order.
                tests.shuffle(&mut rng);
                for test in tests.iter_mut().filter(|t| t.output.durations.len() < t.repeats) {
                    let output = if log_enabled!(log::Level::Trace) {
                        test.benchmark_cmd.run_and_stream(&mut std::io::stdout())?
                    } else {
                        test.benchmark_cmd.run_and_time()?
                    };
                    test.output.durations.add_sample(output.duration);
                    debug!("Completed {} in {} seconds.", test.output.name, output.duration.as_secs_f64());
                }
            }

            // Store results.
            for test in tests.drain(..) {
                info!(
                    "Benchmark result {}: {} ({})",
                    test.output.name,
                    test.output.durations.get_mean().unwrap_or(f64::NAN),
                    test.output.durations.get_stddev().unwrap_or(f64::NAN)
                );
                let min_dt = 0.1;
                if test.output.durations.get_mean().unwrap_or(0.) > min_dt {
                    write_json(&test.result_dir.join(RESULT_INFO_FILE_NAME), &test.output)?;
                } else {
                    warn!(
                        "{} executed in less than {} secs, it probably failed...",
                        test.output.name, min_dt
                    );
                    trace!("command:\n{}", test.benchmark_cmd.print_command());
                }
            }

            info!("Benchmark complete");

            return Ok(());
        }

        info!("Record command complete: exiting.");

        Ok(())
    }
}

fn parse_commands(cmds: &Option<Vec<String>>) -> Vec<Command> {
    if let Some(c) = cmds {
        c.iter().map(|cmd| Command::parse(cmd)).collect()
    } else {
        Vec::new()
    }
}

fn run_all_pre_benchmark_commands<'a>(
    tests: impl Iterator<Item = &'a BenchTestCtxt>,
) -> Result<()> {
    for test in tests {
        info!("Setup context for {}", test.output.name);
        run_pre_benchmark_commands(&test.result_dir, &test.pre_benchmark_cmds)
            .context("failed to run pre-benchmark-cmd")
            .with_context(|| format!("failed to setup {}", test.output.name))?;
    }
    Ok(())
}

fn run_pre_benchmark_commands(path: &PathBuf, cmds: &[Command]) -> Result<()> {
    debug!("Create directory {:?}", path);
    std::fs::create_dir_all(path)?;
    for cmd in cmds {
        debug!("Run cmd: {}", cmd.print_command());
        if log_enabled!(log::Level::Trace) {
            cmd.run_and_stream(&mut std::io::stdout())?
                .into_duration()?;
        } else {
            cmd.run_trim()?;
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct ReadBenchTestSetup {
    pub name: String,
    pub cell_name: Option<String>,
    /// Will be run before executing the benchmark.
    pub pre_benchmark_cmds: Option<Vec<String>>,
    /// The benchmark test command.
    pub benchmark_cmd: String,
    /// Will be run after executing the benchmark.
    pub post_benchmark_cmds: Option<Vec<String>>,
    /// Optional visualization cmd.
    pub visualize_cmd: Option<String>,
    /// Number of repeats for this test.
    pub repeats: Option<usize>,
}

impl Default for ReadBenchTestSetup {
    fn default() -> ReadBenchTestSetup {
        ReadBenchTestSetup {
            name: "foobar".to_owned(),
            benchmark_cmd: format!("ls ${}", crate::CONTEXT_ENV_VAR),
            pre_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::OPENSIM_BUILD_ENV_VAR),
                format!("ls ${}", crate::OPENSIM_SRC_ENV_VAR),
                format!("ls ${}", crate::MODELS_ENV_VAR),
                format!("ls ${}", crate::SETUP_ENV_VAR),
            ]),
            post_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::MODELS_ENV_VAR),
                format!("ls ${}", crate::SETUP_ENV_VAR),
            ]),
            visualize_cmd: Some(format!("ls ${}", crate::OPENSIM_INSTALL_ENV_VAR)),
            cell_name: None,
            repeats: None,
        }
    }
}
