use crate::{
    read_json,
    record::{BenchTestResult, Durations, ReadBenchTestSetup, TestNode},
    write_json, CMakeCommands, Command, CommandTrait, Commit, Ctxt, Date, EnvVars,
    FileBackedStruct, InstallId, Repository, INSTALL_INFO_FILE_NAME, RESULT_INFO_FILE_NAME,
};
use super::InstallInfo;
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, info };
use rand::prelude::*;
use serde::{Deserialize, Serialize};
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
    #[arg(long, short, required_unless_present("grind"))]
    iter: usize,

    /// Use valgrind on test.
    #[arg(long, short)]
    grind: bool,
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
    /// Test config hash.
    pub config_hash: u64,
}

struct BenchTestCtxt {
    pub name: String,
    pub cmd: Command,
    pub dt: Durations,
    pub path: PathBuf,
}

fn absolute_path(relative_path: &PathBuf) -> Result<PathBuf> {
    std::fs::canonicalize(relative_path)
        .with_context(|| format!("failed to create absolute path to {:?}", relative_path))
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

            if let Ok(result) = read_json::<ResultInfo>(&path) {
                if result.commit == install_info.commit {
                    info!("{} previous result found", test.name);
                    continue;
                }
            }

            info!("Setting up context for {}", test.name);

            debug!("Create directory {}", test.name);
            std::fs::create_dir_all(&dir)?;

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

            for cmd in parse_commands(&test.pre_benchmark_cmds)
                .drain(..)
                .map(|c| c.set_envs(&env_vars).set_run_root(&dir))
            {
                cmd.run_trim()
                    .context("failed to run pre-benchmark-cmd")
                    .with_context(|| format!("failed to setup {}", test.name))?;
            }

            let benchmark_cmd = Command::parse(&test.benchmark_cmd)
                .set_envs(&env_vars)
                .set_run_root(&dir);

            tests.push(BenchTestCtxt {
                name: test.name,
                cmd: benchmark_cmd,
                dt: Default::default(),
                path,
            });
        }

        if tests.len() > 0 {
            let mut msg = format!("Prepare to run benchmarks ({}X):\n", self.iter);
            tests.iter().for_each(|t| {
                msg.push_str(&t.cmd.print_command());
                msg.push_str("\n")
            });
            info!("{msg}");

            // Prepare to run tests.
            let mut rng = rand::thread_rng();

            for _ in 0..self.iter {
                tests.shuffle(&mut rng);
                for test in tests.iter_mut() {
                    let output = test.cmd.run_and_time()?;
                    test.dt.add_sample(output.duration);
                }
            }

            // Store results.
            for test in tests.drain(..) {
                let result = ResultInfo {
                    name: test.name.clone(),
                    branch: install_info.branch.clone(),
                    commit: install_info.commit.clone(),
                    date: install_info.date.clone(),
                    durations: test.dt,
                    config_hash: 0,
                };
                write_json(&test.path, &result)?;
                info!("{} {:#?}", result.name, result.durations);
            }
        }

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
