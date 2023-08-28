pub mod table;

use std::{
    fs::{self, create_dir, File},
    path::{Path, PathBuf},
};

use crate::{read_config, write_config, Command, Commit, Folders};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use log::{info, debug };

// Go over subfolders of tests/ to find "osimperf-test.conf"
static TEST_SETUP_FILE_NAME: &str = "osimperf-test.conf";

// Env vars that will be subsitututed per commit case.
static ENV_VAR_INSTALL: &str = "OSIMPERF_INSTALL";
static ENV_VAR_TEST_OUTPUT: &str = "OSIMPERF_OUTPUT";
// static ENV_VAR_TEST_SETUP: &str = "OSIMPERF_SETUP"; // TODO broken
static ENV_VAR_TEST_ROOT: &str = "OSIMPERF_ROOT";

// Search for "TEST_SETUP_FILE_NAME" in directory and subdirectories.
fn find_perf_test_setup_files(root_dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(root_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                result.extend(find_perf_test_setup_files(&entry_path));
            } else if entry_path.file_name() == Some(TEST_SETUP_FILE_NAME.as_ref()) {
                result.push(entry_path);
            }
        }
    }

    result
}

pub fn read_perf_test_setup(folders: &Folders) -> Result<Vec<BenchTestSetup>> {
    let paths = find_perf_test_setup_files(&folders.tests);
    let mut setups = Vec::new();
    for path in paths {
        let setup = read_config::<ReadBenchTestSetup>(path.as_path())
            .context("failed to parse perf test setup file")
            .context(format!("setup file path: {:?}", path))?;
        setups.push(BenchTestSetup {
            name: setup.name.clone(),
            cmd: setup.cmd.clone(),
            path: path.clone(),
        });
    }
    Ok(setups)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReadBenchTestSetup {
    pub name: String,
    pub cmd: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct BenchTestSetup {
    pub name: String,
    pub cmd: Vec<Command>,
    /// Path to the test config file.
    ///
    /// Used to subsitutute [ENV_VAR_TEST_SETUP].
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BenchTestResult {
    pub duration: f64,
    pub iteration: usize,
    pub status: bool,
}

pub fn run_test(
    folders: &Folders,
    setup: &BenchTestSetup,
    commit: &Commit,
) -> Result<BenchTestResult> {
    let test_root_dir = test_environment_root_dir(folders, setup, commit);
    let test_output_dir = test_root_dir.join("output");
    let test_install_dir = commit.get_archive_folder(folders).join("install");
    let mut log_stdout = File::open(test_root_dir.join("log_stdout")).context("unable to open stdout log")?;
    let mut log_stderr = File::open(test_root_dir.join("log_stderr")).context("unable to open stderr log")?;
    // let test_setup_dir = setup.path.clone();

    if !test_root_dir.exists() {
        create_dir(&test_root_dir)?;
        create_dir(&test_output_dir)?;
    }

    let test_install_dir_str = test_install_dir.to_str().unwrap();

    for i in 0..setup.cmd.len() {
        // let mut cmd = Command::new("env");
        // cmd.add_arg("-C");
        // cmd.add_arg(test_root_dir.to_str().unwrap());
        // cmd.add_arg(&setup.cmd[i].cmd);
        // cmd.add_args(setup.cmd[i].args.iter());

        // TODO copy original env vars over as well.
        let mut cmd = setup.cmd[i].clone();

        // Replace environmental variables:
        cmd.add_env(ENV_VAR_TEST_OUTPUT, test_output_dir.to_str().unwrap());
        cmd.add_env(ENV_VAR_TEST_ROOT, test_root_dir.to_str().unwrap());
        // cmd.add_env(ENV_VAR_TEST_SETUP, test_setup_dir.to_str().unwrap());
        cmd.add_env(ENV_VAR_INSTALL, test_install_dir_str);
        cmd.add_env(
            "PATH",
            format!(
                "/bin:{}:{}/lib:{}/include",
                test_install_dir_str, test_install_dir_str, test_install_dir_str
            ),
        );
        cmd.add_env(
            "LD_LIBRARY_PATH",
            format!(
                "/bin:{}:{}/lib:{}/include",
                test_install_dir_str, test_install_dir_str, test_install_dir_str
            ),
        );

        if i + 1 == setup.cmd.len() {
            match cmd.run_extend_log(&mut log_stdout, &mut log_stderr) {
                Ok(duration) => {
                    info!(
                        "Test completed in {} seconds: {:?}, {:?}",
                        duration, commit, setup
                    );
                    debug!("     Root folder: {:?}", test_root_dir);
                    debug!("     intssll folder: {:?}", test_install_dir_str);
                    return Ok(BenchTestResult {
                        duration,
                        status: true,
                        iteration: 1,
                    });
                }
                Err(err) => {
                    info!("Test failed: {:?}, {:?}", commit, setup);
                    info!("    with error: {:?}", err);
                    return Ok(BenchTestResult {
                        duration: f64::NAN,
                        status: false,
                        iteration: 0,
                    });
                }
            }
        }

        cmd.run_extend_log(&mut log_stdout, &mut log_stderr)?;
    }
    Err(anyhow!("Not possible to end up here!"))
}

/// Returns the directory that is used as root during test execution.
/// Something like:
/// osimperf-home/results/results-DATE-COMMIT/TEST_NAME
pub fn test_environment_root_dir(
    folders: &Folders,
    setup: &BenchTestSetup,
    commit: &Commit,
) -> PathBuf {
    commit
        .get_results_folder(folders)
        .join(Path::new(&setup.name))
}

/// Path to final [BenchTestResult] of this test.
///
/// Something like
/// `SIMPERF_HOME/results/results-DATE-HASH/TEST_NAME/osimperf-result.data`
fn test_result_output_file_path(
    folders: &Folders,
    setup: &BenchTestSetup,
    commit: &Commit,
) -> PathBuf {
    test_environment_root_dir(folders, setup, commit).join("osimperf-results.data")
}

pub fn read_test_result(
    folders: &Folders,
    setup: &BenchTestSetup,
    commit: &Commit,
) -> Result<Option<BenchTestResult>> {
    let path = test_result_output_file_path(folders, setup, commit);
    if path.exists() {
        return Some(read_config::<BenchTestResult>(Path::new(&path))).transpose();
    }
    Ok(None)
}

/// Write test result to file.
pub fn update_test_result(
    folders: &Folders,
    setup: &BenchTestSetup,
    commit: &Commit,
    mut result: BenchTestResult,
) -> Result<()> {
    let path = test_result_output_file_path(folders, setup, commit);
    if let Some(prev_result) = read_test_result(folders, setup, commit)? {
        if prev_result.status {
            result.iteration += prev_result.iteration;
            result.duration += prev_result.duration;
            result.duration *= 0.5;
        }
    }
    write_config::<BenchTestResult>(Path::new(&path), &result)?;
    Ok(())
}
