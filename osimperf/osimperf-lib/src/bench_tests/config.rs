use crate::common::{find_file_by_name, read_config};
use crate::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::hash::Hash;

// Go over subfolders of tests/ to find "osimperf-test.conf"
static TEST_SETUP_FILE_NAME: &str = "osimperf-test.conf";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadBenchTestSetup {
    pub name: String,
    pub cmd: Vec<Command>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct BenchTestSetup {
    pub name: String,
    pub cmd: Vec<Command>,
    /// Path to the test config file.
    ///
    /// Used to subsitutute [ENV_VAR_TEST_SETUP].
    pub test_setup_file: PathBuf,
}

impl BenchTestSetup {
    fn new(config: ReadBenchTestSetup, path: PathBuf) -> Self {
        Self {
            test_setup_file: path,
            name: config.name,
            cmd: config.cmd,
        }
    }

    pub fn find_all(path: &Path) -> Result<Vec<BenchTestSetup>> {
        let mut tests = Vec::new();
        for p in find_file_by_name(path, TEST_SETUP_FILE_NAME) {
            let c = read_config::<ReadBenchTestSetup>(&p)?;
            tests.push(BenchTestSetup::new(c, p));
        }
        Ok(tests)
    }
}
