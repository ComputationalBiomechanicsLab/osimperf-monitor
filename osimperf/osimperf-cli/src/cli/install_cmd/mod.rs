use crate::FileBackedStruct;
use crate::{
    read_json, write_json, CMakeCommands, CommandTrait, Commit, Ctxt, Date, Repository,
    INSTALL_INFO_FILE_NAME,
};

use crate::context::OPENSIM_BUILD_ENV_VAR;
use crate::context::OPENSIM_INSTALL_ENV_VAR;
use crate::context::OPENSIM_SRC_ENV_VAR;

use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, info, log_enabled, trace};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Path to opensim-core repo (or set OSIMPERF_OPENSIM_SRC env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == OPENSIM_SRC_ENV_VAR).is_none()))]
    opensim: Option<PathBuf>,
    /// Path to install directory (or set OSIMPERF_OPENSIM_INSTALL env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == OPENSIM_INSTALL_ENV_VAR).is_none()))]
    install: Option<PathBuf>,
    /// Path to build directory (or set OSIMPERF_OPENSIM_BUILD env variable).
    #[arg(long, required(std::env::vars().find(|(key,_)| key == OPENSIM_BUILD_ENV_VAR).is_none()))]
    build: Option<PathBuf>,
    /// Branch.
    #[arg(long, default_value = "main")]
    branch: String,
    /// Url.
    #[arg(
        long,
        default_value = "https://github.com/opensim-org/opensim-core.git"
    )]
    url: String,
    /// Commit hash (defaults to currently checked out).
    #[arg(long)]
    commit: Option<String>,
    /// Path to cmake config file.
    #[arg(long)]
    cmake: Option<PathBuf>,
}

/// Returns the absolute path of the arg or checks the environmental variables.
fn arg_or_env_var(arg: Option<PathBuf>, key: &str) -> Result<Option<PathBuf>> {
    arg.or_else(|| {
        std::env::vars()
            .find(|(k, _)| k == key)
            .map(|(_, value)| PathBuf::from(value))
    })
    .map(|relative| super::absolute_path(&relative))
    .transpose()
}

impl InstallCommand {
    pub fn run(&self) -> Result<()> {
        let source =
            arg_or_env_var(self.opensim.clone(), crate::context::OPENSIM_SRC_ENV_VAR)?.unwrap();
        let install = arg_or_env_var(
            self.install.clone(),
            crate::context::OPENSIM_INSTALL_ENV_VAR,
        )?
        .unwrap();
        let build =
            arg_or_env_var(self.build.clone(), crate::context::OPENSIM_BUILD_ENV_VAR)?.unwrap();

        let commit = if let Some(c) = self.commit.clone() {
            c
        } else {
            osimperf_lib::git::read_current_commit(&source)?
        };

        // Check if already installed.
        let info_path = install.join(INSTALL_INFO_FILE_NAME);
        if let Ok(info) = read_json::<InstallInfo>(&info_path) {
            if info.commit == commit {
                info!("Found installed commit {} ({}).", info.commit, info.date,);
                return Ok(());
            }
        }

        // Find date of commit.
        let date = osimperf_lib::git::get_date(&source, &commit)?;

        info!("Start installing commit {} ({}).", commit, date,);

        // Verify url.
        debug!("Verify repository URL.");
        crate::common::verify_repository(&source, &self.url)?;

        // Verify commit part of branch.
        debug!("Verify branch.");
        ensure!(
            osimperf_lib::git::was_commit_merged_to_branch(&source, &self.branch, &commit)?,
            format!("commit {} not part of branch {}", &commit, &self.branch)
        );

        // Checkout commit.
        info!("Checkout {:?} to {}", source, commit);
        osimperf_lib::git::checkout_commit(&source, &commit)?;

        // Set environmental variables.
        let env_vars = crate::EnvVars {
            opensim_build: Some(build),
            opensim_source: Some(source.clone()),
            opensim_install: Some(install),
            ..Default::default()
        }
        .make();
        debug!("{:#?}", env_vars);

        let cmake_cmds = self
            .cmake
            .as_ref()
            .map(|path| read_json::<CMakeCommands>(path))
            .unwrap_or(Ok(CMakeCommands::default()))?
            .with_env_vars(&env_vars);

        let mut dt = Duration::from_secs(0);
        for (task, cmd) in cmake_cmds.0.iter() {
            debug!("run cmake command: {:#?}", cmd.print_command());
            let output = if log_enabled!(log::Level::Trace) {
                cmd.run_and_stream(&mut std::io::stdout())
            } else {
                cmd.run_and_time()
            }
            .with_context(|| format!("cmake command failed: {:#?}", cmd.print_command()))?;
            dt += output.duration;

            // We failed to compile, so we stop.
            if !output.success() {
                Err(anyhow!("Failed to compile"))
                    .with_context(|| format!("command failed: {:#?}", cmd.print_command()))
                    .with_context(|| format!("command output: {:#?}", cmd))?;
            }
        }

        let install_info = InstallInfo {
            commit: commit.clone(),
            date: date.clone(),
            duration: dt.as_secs(),
            branch: self.branch.clone(),
        };
        crate::write_json(&info_path, &install_info)?;

        info!(
            "Finished installing {} ({}) in {} minutes.",
            commit,
            date,
            dt.as_secs() / 60
        );

        info!("Installation info written to {:?}", info_path);

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallInfo {
    pub branch: String,
    pub commit: String,
    pub date: String,
    pub duration: u64,
}
