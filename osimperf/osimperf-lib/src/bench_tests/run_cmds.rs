use crate::{Command, CommandOutput, CommandTrait, ResultsFolder, Folder};
use anyhow::{anyhow, Context, Result};
use log::{trace, info };
use std::{path::{Path, PathBuf}, fs::DirEntry};

use super::setup_context;

// Environmental variables to be used when defining the tests.
static ENV_VAR_TEST_INSTALL: &str = "$OSIMPERF_INSTALL";
static ENV_VAR_TEST_OUTPUT: &str = "$OSIMPERF_OUTPUT";
/// The root of the results folder:
/// results/scratch
static ENV_VAR_TEST_ROOT: &str = "$OSIMPERF_ROOT";
/// Home directory of osimperf repo. Can be used to locate setup files.
static ENV_VAR_TEST_HOME: &str = "$OSIMPERF_HOME";

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

pub fn run_test_cmds(cmds: &[Command], env: &FileEnvVars, setup_dir: &Path) -> Result<CommandOutput> {
    info!("Setting up context at {:?}", env.root);

    // Copy all files to context dir.
    setup_context(setup_dir, &env.root)?;

    for i in 0..cmds.len() {
        // Add environmental variables:
        let mut cmd = cmds[i].clone();
        env.add_env(&mut cmd);

        trace!("running test command: {}", cmd.print_command());

        let is_last = i + 1 == cmds.len();
        if is_last {
            return cmd.run_and_time();
        }

        cmd.run()
            .with_context(|| format!("Failed at {i}-th command"))?;
    }
    Err(anyhow!("Not possible to end up here!"))
}
