use super::absolute_path;
use super::arg_or_env_var;
use super::InstallInfo;

use crate::context::MODELS_ENV_VAR;
use crate::context::OPENSIM_INSTALL_ENV_VAR;

use crate::{
    read_json,
    record::{BenchTestResult, Durations, TestNode},
    write_json, CMakeCommands, Command, CommandTrait, Commit, Ctxt, Date, EnvVars,
    FileBackedStruct, InstallId, Repository, INSTALL_INFO_FILE_NAME, RESULT_INFO_FILE_NAME,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::log_enabled;
use log::{debug, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{path::PathBuf, str::FromStr};

static RESULTS_ENV_VAR: &str = "OSIMPERF_RESULTS";

#[derive(Debug, Args)]
pub struct RecordCommand {
    /// Path to install directory (or set OSIMPERF_OPENSIM_INSTALL env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == OPENSIM_INSTALL_ENV_VAR).is_none()))]
    install: Option<PathBuf>,

    /// Path to results directory (or set OSIMPERF_RESULTS env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == RESULTS_ENV_VAR).is_none()))]
    results: Option<PathBuf>,

    /// Path to models directory (or set OSIMPERF_MODELS env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == MODELS_ENV_VAR).is_none()))]
    models: Option<PathBuf>,

    /// Number of test iterations.
    #[arg(long, short, default_value_t = 0)]
    iter: usize,

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
struct ResultInfo {
    /// Test case name.
    pub name: String,
    /// Opensim-core branch name.
    pub branch: String,
    /// Opensim-core commit hash.
    pub commit: String,
    /// Opensim-core commit date.
    pub date: String,
    /// Benchmark durations.
    pub durations: Durations,
    /// Benchmark grind result.
    pub grind: Option<std::time::Duration>,
    /// Test config hash.
    pub config_hash: u64,
}

#[derive(Debug)]
struct BenchTestCtxt {
    pub dir: PathBuf,
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
        let install = arg_or_env_var(self.install.clone(), OPENSIM_INSTALL_ENV_VAR)?.unwrap();
        let results_dir = arg_or_env_var(self.results.clone(), RESULTS_ENV_VAR)?.unwrap();
        let models = arg_or_env_var(self.models.clone(), MODELS_ENV_VAR)?.unwrap();

        let install_info = read_json::<InstallInfo>(&install.join(INSTALL_INFO_FILE_NAME))
            .context("failed to find opensim installation")
            .with_context(|| format!("failed to locate {INSTALL_INFO_FILE_NAME}"))?;

        let mut tests = Vec::new();

        for line in std::io::stdin().lines() {
            let config_path = absolute_path(&PathBuf::from_str(&line?)?)?;

            // Read test case setup file.
            let test = read_json::<ReadBenchTestSetup>(&config_path)?;

            // Check if previous result exists.
            let dir = results_dir.join(&test.name);
            let path = dir.join(RESULT_INFO_FILE_NAME);

            // Detect changes in test configuration.
            let mut hasher = DefaultHasher::new();
            test.hash(&mut hasher);
            let config_hash = hasher.finish();

            // Read any previous result.
            let result_info = read_json::<ResultInfo>(&path)
                .ok()
                .filter(|r| r.commit == install_info.commit)
                .filter(|r| r.config_hash == config_hash)
                .unwrap_or(ResultInfo {
                    name: test.name.clone(),
                    branch: install_info.branch.clone(),
                    commit: install_info.commit.clone(),
                    date: install_info.date.clone(),
                    durations: Default::default(),
                    grind: None,
                    config_hash,
                });

            let env_vars = EnvVars {
                opensim_install: Some(install.clone()),
                models: Some(models.clone()),
                test_setup: Some(config_path.parent().unwrap().to_path_buf()),
                test_context: Some(dir.clone()),
                ..Default::default()
            }
            .make();
            for env in env_vars.iter() {
                debug!("set {}={}", env.key, env.value);
            }

            let pre_benchmark_cmds = parse_commands(&test.pre_benchmark_cmds)
                .drain(..)
                .map(|c| c.set_envs(&env_vars).set_run_root(&dir))
                .collect::<Vec<Command>>();

            let benchmark_cmd = Command::parse(&test.benchmark_cmd)
                .set_envs(&env_vars)
                .set_run_root(&dir);

            let grind_cmd_base = "valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --cache-sim=yes --branch-sim=yes";
            let grind_cmd = Command::parse(&format!(
                "{grind_cmd_base} --callgrind-out-file={}/callgrind.out {}",
                dir.to_str().unwrap(),
                test.benchmark_cmd
            ))
            .set_envs(&env_vars)
            .set_run_root(&dir);

            let visualize_cmd = test
                .visualize_cmd
                .as_ref()
                .map(|s| Command::parse(s).set_envs(&env_vars).set_run_root(&dir));

            tests.push(BenchTestCtxt {
                pre_benchmark_cmds,
                benchmark_cmd,
                grind_cmd,
                visualize_cmd,
                output: result_info,
                dir,
            });
        }

        if self.grind {
            tests.retain(|t| t.output.grind.is_none());

            if tests.len() == 0 {
                info!("Nothing to grind");
                return Ok(());
            }

            run_all_pre_benchmark_commands(&tests)?;

            let mut msg = String::from("Prepare to grind benchmarks:");
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.grind_cmd.print_command())
            });
            info!("{msg}");

            for test in tests.iter_mut() {
                let output = test.grind_cmd.run_and_time()?;
                let dt = *test.output.grind.insert(output.duration);

                // Store results.
                write_json(&test.dir.join(RESULT_INFO_FILE_NAME), &test.output)?;
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

            // Setup test context.
            run_all_pre_benchmark_commands(&tests)?;

            // Print list of tests that will be ran.
            let mut msg = format!("Prepare to run benchmarks ({}X):", self.iter);
            tests.iter().for_each(|t| {
                msg.push_str("\n");
                msg.push_str(&t.benchmark_cmd.print_command())
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
                write_json(&test.dir.join(RESULT_INFO_FILE_NAME), &test.output)?;
            }

            info!("Benchmark complete");

            return Ok(());
        } else {
            if self.visualize {
                tests.retain(|t| t.visualize_cmd.is_some());
            }

            // Setup test context.
            for test in tests.iter() {
                if self.visualize {
                    println!("{}", test.benchmark_cmd.print_command());
                } else {
                    println!("{}", test.visualize_cmd.as_ref().unwrap().print_command());
                }
            }
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

fn run_all_pre_benchmark_commands(tests: &[BenchTestCtxt]) -> Result<()> {
    for test in tests {
        info!("Setup context for {}", test.output.name);
        run_pre_benchmark_commands(&test.dir, &test.pre_benchmark_cmds)
            .context("failed to run pre-benchmark-cmd")
            .with_context(|| format!("failed to setup {}", test.output.name))?;
        info!("Setup complete");
    }
    Ok(())
}

fn run_pre_benchmark_commands(path: &PathBuf, cmds: &[Command]) -> Result<()> {
    debug!("Create directory {:?}", path);
    std::fs::create_dir_all(path)?;
    for cmd in cmds {
        info!("Run cmd: {}", cmd.print_command());
        if log_enabled!(log::Level::Trace) {
            cmd.run_and_stream(&mut std::io::stdout())?;
        } else {
            cmd.run_trim()?;
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct ReadBenchTestSetup {
    pub name: String,
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
        }
    }
}
