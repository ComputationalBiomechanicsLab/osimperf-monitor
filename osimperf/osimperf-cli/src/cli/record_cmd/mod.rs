use super::InstallInfo;
use crate::{
    read_json,
    record::{BenchTestResult, Durations, ReadBenchTestSetup, TestNode},
    write_json, CMakeCommands, Command, CommandTrait, Commit, Ctxt, Date, EnvVars,
    FileBackedStruct, InstallId, Repository, INSTALL_INFO_FILE_NAME, RESULT_INFO_FILE_NAME,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, info};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Args)]
pub struct RecordCommand {
    /// Path to install directory (looks for osimperf-install-info.data).
    #[arg(long)]
    install: PathBuf,

    /// Path to results directory.
    #[arg(long)]
    results: PathBuf,

    /// Path to models directory.
    #[arg(long)]
    models: PathBuf,

    /// Number of test iterations.
    #[arg(long, short, default_value_t = 0, required_unless_present("grind"))]
    iter: usize,

    /// Use valgrind on test.
    #[arg(long, short)]
    grind: bool,

    /// Force retesting.
    #[arg(long, short)]
    force: bool,
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

struct BenchTestCtxt {
    pub dir: PathBuf,
    pub pre_benchmark_cmds: Vec<Command>,
    pub benchmark_cmd: Command,
    pub grind_cmd: Command,
    pub output: ResultInfo,
}

struct GrindTestCtxt {
    pub name: String,
    pub cmd: Command,
}

fn absolute_path(relative_path: &PathBuf) -> Result<PathBuf> {
    std::fs::canonicalize(relative_path)
        .with_context(|| format!("failed to create absolute path to {:?}", relative_path))
}

struct RecordCommandInput {
    install: PathBuf,
    results: PathBuf,
    models: PathBuf,
    cmds: Vec<Command>,
}

impl RecordCommand {
    pub fn run(&self) -> Result<()> {
        let install = absolute_path(&self.install)?;
        let results_dir = absolute_path(&self.results)?;
        let models = absolute_path(&self.models)?;

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

            tests.push(BenchTestCtxt {
                dir,
                pre_benchmark_cmds,
                benchmark_cmd,
                grind_cmd,
                output: result_info,
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

            info!("Grind complete: exiting.");

            return Ok(());
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
                    let output = test.benchmark_cmd.run_and_time()?;
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

            info!("Benchmark complete: exiting.");

            return Ok(());
        }

        info!("Nothing to do.");

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
        run_pre_benchmark_commands(&test.dir, &test.pre_benchmark_cmds)
            .context("failed to run pre-benchmark-cmd")
            .with_context(|| format!("failed to setup {}", test.output.name))?;
    }
    Ok(())
}

fn run_pre_benchmark_commands(path: &PathBuf, cmds: &[Command]) -> Result<()> {
    debug!("Create directory {:?}", path);
    std::fs::create_dir_all(path)?;
    for cmd in cmds {
        cmd.run_trim()?;
    }
    Ok(())
}
