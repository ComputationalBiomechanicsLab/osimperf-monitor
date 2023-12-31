use super::setup_context;
use crate::{erase_folder, Command, CommandOutput, CommandTrait};
use anyhow::{Context, Result};
use log::trace;
use std::path::{Path, PathBuf};

// Environmental variables to be used when defining the tests.
static ENV_VAR_TEST_INSTALL: &str = "OSIMPERF_INSTALL";
static ENV_VAR_TEST_OUTPUT: &str = "OSIMPERF_OUTPUT";
/// The root of the results folder:
/// results/scratch
static ENV_VAR_TEST_ROOT: &str = "OSIMPERF_ROOT";
/// Home directory of osimperf repo. Can be used to locate setup files.
static ENV_VAR_TEST_HOME: &str = "OSIMPERF_HOME";

pub struct FileEnvVars {
    /// Contains opensim-core, simbody, and test binary installs.
    pub install: PathBuf,
    /// Folder for collecting simulation output files.
    pub output: PathBuf,
    /// Directory from which this command is run.
    pub root: PathBuf,
    /// Absolute path to home directory of this project.
    pub home: PathBuf,
}

impl FileEnvVars {
    /// Adds all environmental variables and sets working directory to SCRATCH_DIR.
    pub fn add_env(&self, cmd: &mut Command) {
        cmd.add_env_path(ENV_VAR_TEST_OUTPUT, &self.output);
        cmd.add_env_path(ENV_VAR_TEST_ROOT, &self.root);
        cmd.add_env_path(ENV_VAR_TEST_INSTALL, &self.install);
        cmd.add_env_path(ENV_VAR_TEST_HOME, &self.home);

        let install = String::from(self.install.join("opensim-core").to_str().unwrap());
        cmd.add_env(
            "PATH",
            format!("/bin:{}:{}/lib:{}/include", install, install, install),
        );
        cmd.add_env(
            "LD_LIBRARY_PATH",
            format!("/bin:{}:{}/lib:{}/include", install, install, install),
        );

        // Set command working directory.
        cmd.set_run_root(&self.root);
    }

    pub fn with_env(&self, mut cmd: Command) -> Command {
        self.add_env(&mut cmd);
        cmd
    }
}

pub fn run_pre_test_cmds(
    cmds: &[Command],
    env: &FileEnvVars,
    setup_dir: &Path,
    required_files: &[String],
) -> Result<()> {
    // Erase contents in output directories.
    erase_folder(&env.output)?;

    // Copy all files to context dir.
    let models_dir = env.home.join("tests").join("opensim-models");
    trace!("Setting up context at {:?}", env.root);
    setup_context(setup_dir, &env.root, required_files, &models_dir)?;

    // Run all pre-benchmark commands.
    for c in cmds.iter().map(|c| env.with_env(c.clone())) {
        // Add environmental variables:
        trace!("Running pre-benchmark command: {}", c.print_command());
        let _ = c.run()?;
    }
    Ok(())
}

pub fn run_post_test_cmds(
    cmds: &[Command],
    env: &FileEnvVars,
    output: &Option<CommandOutput>,
) -> Result<()> {
    // Write logs.
    if let Some(out) = output.as_ref() {
        out.write_stdout(&env.output.join("stdout.log"))?;
        out.write_stderr(&env.output.join("stderr.log"))?;
    }

    // Run all post-benchmark commands.
    for c in cmds.iter().map(|c| env.with_env(c.clone())) {
        trace!("Running post-benchmark command: {}", c.print_command());
        let _ = c.run()?;
    }

    Ok(())
}

pub fn run_test_bench_cmd(
    cmd: &Command,
    env: &FileEnvVars,
) -> Result<CommandOutput> {
    // Run benchmark command.
    let benchmark_cmd = env.with_env(cmd.clone());
    trace!(
        "Running benchmark command: {}",
        benchmark_cmd.print_command()
    );
    let output = benchmark_cmd
        .run_and_time()
        .context("failed to run benchmark command")?;
    Ok(output)
}
