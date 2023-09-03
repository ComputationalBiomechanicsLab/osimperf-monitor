use crate::common::{find_file_by_name, read_config};
use crate::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::{Path, PathBuf};

// Go over subfolders of tests/ to find "osimperf-test.conf"
static TEST_SETUP_FILE_NAME: &str = "osimperf-test.conf";

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReadBenchTestSetup {
    name: String,
    /// Will be run before executing the benchmark.
    pre_benchmark_cmds: Option<Vec<String>>,
    /// The benchmark test command.
    benchmark_cmd: String,
    /// Will be run after executing the benchmark.
    post_benchmark_cmds: Option<Vec<String>>,
    /// Will search in OSIMPERF_HOME/tests/opensim-models/* for files with the same name.
    files: Option<Vec<String>>,
}

#[derive(Clone, Debug, Hash)]
pub struct BenchTestSetup {
    pub name: String,
    pub benchmark_cmd: Command,
    pub pre_benchmark_cmds: Vec<Command>,
    pub post_benchmark_cmds: Vec<Command>,
    /// Path to the test config file.
    ///
    /// Used to subsitutute [ENV_VAR_TEST_SETUP].
    pub test_setup_file: PathBuf,
    pub model_files: Vec<String>,
}

fn parse_commands(cmds: &Option<Vec<String>>) -> Vec<Command> {
    if let Some(c) = cmds {
        c.iter().map(|cmd| Command::parse(cmd)).collect()
    } else {
        Vec::new()
    }
}

impl BenchTestSetup {
    fn new(config: ReadBenchTestSetup, path: PathBuf) -> Self {
        Self {
            test_setup_file: path,
            name: config.name,
            benchmark_cmd: Command::parse(&config.benchmark_cmd),
            pre_benchmark_cmds: parse_commands(&config.pre_benchmark_cmds),
            post_benchmark_cmds: parse_commands(&config.post_benchmark_cmds),
            model_files: config.files.unwrap_or_default(),
        }
    }

    pub fn find_all(path: &Path) -> Result<Vec<Self>> {
        let mut tests = Vec::new();
        for p in find_file_by_name(path, TEST_SETUP_FILE_NAME) {
            let c = read_config::<ReadBenchTestSetup>(&p)?;
            tests.push(BenchTestSetup::new(c, p));
        }
        Ok(tests)
    }
}
