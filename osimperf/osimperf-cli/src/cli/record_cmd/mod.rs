use super::absolute_path;
use super::arg_or_env_var;
use super::InstallInfo;

use crate::context::MODELS_ENV_VAR;
use crate::context::OPENSIM_INSTALL_ENV_VAR;

use crate::{
    read_json, Durations, write_json, Command, CommandTrait, EnvVars,
    INSTALL_INFO_FILE_NAME, RESULT_INFO_FILE_NAME,
};
use anyhow::anyhow;
use anyhow::{Context, Result};
use clap::Args;
use log::log_enabled;
use log::warn;
use log::{debug, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::env::current_dir;
use std::hash::{Hash, Hasher};
use std::path::absolute;
use std::{path::PathBuf, str::FromStr};

static RESULTS_ENV_VAR: &str = "OSIMPERF_RESULTS";

/// OSimPerf record command for running benchmark tests.
///
/// Takes path to benchmark configuration file.
/// Runs specified commands from that directory and creates `osimperf-result-ID/osimperf-result-info.json`
/// Uses PATH to find `osimperf-install-info`, which must match `opensim-cmd --version`
#[derive(Debug, Args)]
pub struct RecordCommand {
    /// Number of test iterations.
    #[arg(long, short, default_value_t = 0)]
    iter: usize,

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

    /// Prefix PATH env var.
    #[arg(long, short)]
    prefix_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultInfo {
    /// Test case name.
    pub name: String,
    /// Cell name.
    pub cell_name: Option<String>,
    /// Opensim-core branch name.
    pub branch: String,
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
}

struct GrindTestCtxt {
    pub name: String,
    pub cmd: Command,
}

struct RecordCommandInput {
    install: PathBuf,
    results: PathBuf,
    models: PathBuf,
    cmds: Vec<Command>,
}

impl RecordCommand {
    pub fn run(&self) -> Result<()> {
        info!("Start OSimPerf record command");

        // If prefix path is set, get version from PATH.
        if let Some(prefix_path) = self.prefix_path.as_ref() {
            super::prefix_path(&["PATH", "LD_LIBRARY_PATH"], prefix_path)?;
        }
        let install_info =
            super::find_install_info_on_path().context("failed to find install-info")?;
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
            let result_dir = root_dir.join(&format!(
                "osimperf-results_{}_{}_{}",
                config.name, install_info.date, install_info.commit
            ));

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
                    branch: install_info.branch.clone(),
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
            let grind_cmd = Command::parse(&format!(
                "{grind_cmd_base} --callgrind-out-file={}/callgrind.out {}",
                result_dir.to_str().unwrap(),
                config.benchmark_cmd
            ))
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
                println!("PATH={} LD_LIBRARY_PATH={} {}",
                    std::env::var("PATH")?,
                    std::env::var("LD_LIBRARY_PATH")?,
                    cmd.print_command());
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
        }

        if self.grind {
            tests.retain(|t| t.output.grind.is_none());

            if tests.len() == 0 {
                info!("Nothing to grind");
                return Ok(());
            }

            let mut msg = String::from("Prepare to grind benchmarks:");
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.output.name);
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

        if self.iter > 0 {
            // Filter tests that are complete.
            if !self.force {
                tests.retain(|t| t.output.durations.len() != self.iter);
            }

            if tests.len() == 0 {
                info!("Nothing to test");
                return Ok(());
            }

            // Print list of tests that will be ran.
            let mut msg = format!("Prepare to run benchmarks ({}X):", self.iter);
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.output.name);
            });
            info!("{msg}");

            // Reset any previous measurements.
            for test in tests.iter_mut() {
                test.output.durations = Default::default();
            }

            // Run tests repeatedly.
            let mut rng = rand::thread_rng();
            for _ in 0..self.iter {
                // Randomize test order.
                tests.shuffle(&mut rng);
                for test in tests.iter_mut() {
                    debug!("Running {}", test.output.name);
                    let output = if log_enabled!(log::Level::Trace) {
                        test.benchmark_cmd.run_and_stream(&mut std::io::stdout())?
                    } else {
                        test.benchmark_cmd.run_and_time()?
                    };
                    test.output.durations.add_sample(output.duration);
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
                    warn!("{} executed in less than {} secs, it probably failed...", test.output.name, min_dt);
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
}

impl Default for ReadBenchTestSetup {
    fn default() -> ReadBenchTestSetup {
        ReadBenchTestSetup {
            name: "foobar".to_owned(),
            benchmark_cmd: format!("ls ${}", crate::context::CONTEXT_ENV_VAR),
            pre_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::context::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::context::OPENSIM_BUILD_ENV_VAR),
                format!("ls ${}", crate::context::OPENSIM_SRC_ENV_VAR),
                format!("ls ${}", crate::context::MODELS_ENV_VAR),
                format!("ls ${}", crate::context::SETUP_ENV_VAR),
            ]),
            post_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::context::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::context::MODELS_ENV_VAR),
                format!("ls ${}", crate::context::SETUP_ENV_VAR),
            ]),
            visualize_cmd: Some(format!("ls ${}", crate::context::OPENSIM_INSTALL_ENV_VAR)),
            cell_name: None,
        }
    }
}
