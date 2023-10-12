use crate::{Command, read_json};
use osimperf_lib::common::{find_file_by_name, read_config};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::{Path, PathBuf};

// Go over subfolders of tests/ to find "osimperf-test.conf"
pub static TEST_SETUP_FILE_NAME: &str = "osimperf-test.conf";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadBenchTestSetup {
    pub name: String,
    /// Will be run before executing the benchmark.
    pub pre_benchmark_cmds: Option<Vec<String>>,
    /// The benchmark test command.
    pub benchmark_cmd: String,
    /// Will be run after executing the benchmark.
    pub post_benchmark_cmds: Option<Vec<String>>,
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
}

impl Default for ReadBenchTestSetup {
    fn default() -> ReadBenchTestSetup {
        ReadBenchTestSetup {
            name: "foobar".to_owned(),
            benchmark_cmd: 
                format!("ls ${}", crate::context::CONTEXT_ENV_VAR),
            pre_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::context::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::context::MODELS_ENV_VAR),
                format!("ls ${}", crate::context::SETUP_ENV_VAR),
            ]),
            post_benchmark_cmds: Some(vec![
                format!("ls ${}", crate::context::OPENSIM_INSTALL_ENV_VAR),
                format!("ls ${}", crate::context::MODELS_ENV_VAR),
                format!("ls ${}", crate::context::SETUP_ENV_VAR),
            ]),
        }
    }
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
