use crate::FileBackedStruct;
use crate::{INSTALL_INFO_FILE_NAME,
    read_json, write_json, CMakeCommands, CommandTrait, Commit, Ctxt, Date, Repository};

use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, info, log_enabled, trace};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Path to opensim-core repo.
    #[arg(long)]
    opensim: PathBuf,
    /// Path to install directory.
    #[arg(long)]
    install: PathBuf,
    /// Path to build directory.
    #[arg(long)]
    build: PathBuf,
    /// Branch.
    #[arg(long, default_value = "main")]
    branch: String,
    /// Url.
    #[arg(
        long,
        default_value = "https://github.com/opensim-org/opensim-core.git"
    )]
    url: String,
    /// Commit.
    #[arg(long)]
    commit: String,
    /// Path to cmake config file.
    #[arg(long)]
    cmake: Option<PathBuf>,
}

impl InstallCommand {
    pub fn run(&self) -> Result<()> {
        let source = std::fs::canonicalize(&self.opensim)
            .with_context(|| format!("failed to setup path to {:?}", self.opensim))?;
        let install = std::fs::canonicalize(&self.install)
            .with_context(|| format!("failed to setup path to {:?}", self.install))?;
        let build = std::fs::canonicalize(&self.build)
            .with_context(|| format!("failed to setup path to {:?}", self.build))?;

        // Check if already installed.
        let info_path = install.join(INSTALL_INFO_FILE_NAME);
        if let Ok(info) = read_json::<InstallInfo>(&info_path) {
            if info.commit == self.commit {
                info!("Found installed commit {} ({}).", info.commit, info.date,);
                return Ok(());
            }
        }

        // Find date of commit.
        let date = osimperf_lib::git::get_date(&source, &self.commit)?;

        info!("Start installing commit {} ({}).", self.commit, date,);

        // Verify url.
        debug!("Verify repository URL.");
        crate::common::verify_repository(&source, &self.url)?;

        // Verify commit part of branch.
        debug!("Verify branch.");
        ensure!(
            osimperf_lib::git::was_commit_merged_to_branch(&source, &self.branch, &self.commit)?,
            format!(
                "commit {} not part of branch {}",
                &self.commit, &self.branch
            )
        );

        // Checkout commit.
        info!("Checkout {:?} to {}", source, self.commit);
        osimperf_lib::git::checkout_commit(&source, &self.commit)?;

        // Set environmental variables.
        let env_vars = crate::EnvVars {
            opensim_build: Some(build),
            opensim_source: Some(source.clone()),
            opensim_install: Some(install),
            ..Default::default()
        }
        .make();
        trace!("{:#?}", env_vars);

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
            commit: self.commit.clone(),
            date: date.clone(),
            duration: dt.as_secs(),
            branch: self.branch.clone(),
        };
        crate::write_json(&info_path, &install_info)?;

        info!(
            "Finished installing {} ({}) in {} minutes.",
            self.commit,
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
