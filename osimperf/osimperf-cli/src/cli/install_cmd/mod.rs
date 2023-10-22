use crate::{read_json, Command, CommandTrait, INSTALL_INFO_FILE_NAME};

use anyhow::{ensure, Context, Result};
use clap::Args;
use log::{debug, info, log_enabled, trace, warn};
use serde::{Deserialize, Serialize};
use std::path::{absolute, PathBuf};

use super::arg_or_env_var;

/// OSimPerf install command.
///
/// Executes installer script with `OSIMPERF_INSTALL`, `OSIMPERF_OPENSIM_SRC` set.
#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Name of project.
    #[arg(long, short, default_value = "opensim")]
    name: String,

    /// Path to install script.
    #[arg(long, short)]
    installer: PathBuf,

    /// Path to opensim-core repo.
    #[arg(long, short, required_unless_present("prefix_path"))]
    opensim: Option<PathBuf>,

    /// Commit hash (defaults to currently checked out) of opensim-core if set.
    #[arg(long, short)]
    commit: Option<String>,

    /// Prefix path for finding previously installed (osimperf) software.
    #[arg(long, short)]
    prefix_path: Option<String>,

    /// Branch of opensim-core.
    #[arg(long, short, default_value = "main")]
    branch: String,

    /// Url to opensim-core remote repository.
    #[arg(
        long,
        short,
        default_value = "https://github.com/opensim-org/opensim-core.git"
    )]
    url: String,

    /// Force reinstalling.
    #[arg(long, short)]
    force: bool,
}

impl InstallCommand {
    pub fn run(&self) -> Result<()> {
        info!("Start OSimPerf install command");

        // Get path to config file.
        let config = absolute(&self.installer).context("failed to setup config dir")?;
        debug!("Read install script from {:?}", config);

        // Use directory of config file as root for installer.
        let installer_root = config
            .parent()
            .context("failed to get installer parent dir")?;
        trace!("Installer root = {:?}", installer_root);

        // Get installer filename for abbreviated commands.
        let installer_filename = config
            .file_name()
            .context("failed to get installer filename")?;
        trace!("Installer filename = {:?}", installer_filename);

        // Get path to opensim-core source from argument or environmental variable.
        let source = arg_or_env_var(self.opensim.clone(), crate::context::OPENSIM_SRC_ENV_VAR)?
            .context("failed to setup source dir")?;
        trace!("Path to OpenSim-core source = {:?}", source);

        // If prefix path is set, get version from PATH.
        let prefix_install_info = self.prefix_path.as_ref().map(|prefix| {
            super::prefix_path(&["PATH", "LD_LIBRARY_PATH"], prefix)
            .expect("Failed to prefix path");
            super::find_install_info_on_path()
                .expect("Failed to find InstallInfo on prefixed path.")
        });
        let commit = prefix_install_info.as_ref().map(|info| &info.commit);

        // Get version from either prefix-path, commit argument, or both:
        let commit = match (commit, self.commit.as_ref()) {
            // Read currently checked out commit.
            (None, None) => osimperf_lib::git::read_current_commit(&source)?,
            // Use commit obtained from PATH.
            (Some(a), None) => a.to_owned(),
            // Use commit from argument --commit.
            (None, Some(b)) => b.to_owned(),
            // Verify that commit from PATH and argument matches, and use it.
            (Some(a), Some(b)) => Some(a.clone())
                .filter(|a| a == b)
                .context("Failed to match opensim version on PATH")
                .with_context(|| format!("Got version {a}\nExpected version {b}"))?,
        };

        // Find date of commit.
        let date = osimperf_lib::git::get_date(&source, &commit)?;

        // Setup install folder.
        let install = PathBuf::from(format!(
            "osimperf-install_{}_{}_{}",
            self.name, date, commit
        ));

        // Check if already installed.
        let info_path = installer_root.join(install.join(INSTALL_INFO_FILE_NAME));
        if let Some(info) = read_json::<InstallInfo>(&info_path)
            .ok()
            .filter(|info| info.name == self.name)
            .filter(|info| info.commit == commit)
        {
            if !self.force {
                info!("Found installed commit {} ({}).", info.commit, info.date,);
                print_include_dir(&install);
                return Ok(());
            }
            warn!(
                "Overwriting previously installed commit {} ({}).",
                info.commit, info.date,
            );
        }

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
            opensim_source: Some(source.clone()),
            install: Some(install.clone()),
            ..Default::default()
        }
        .make();

        let cmd = Command::new(config.to_str().unwrap())
            .set_envs(&env_vars)
            .set_run_root(&installer_root);
        debug!("Run installer:\n{}", cmd.print_command());

        let duration = if log_enabled!(log::Level::Trace) {
            cmd.run_and_stream(&mut std::io::stdout())
        } else {
            cmd.run_and_time()
        }
        .and_then(|output| output.into_duration())
        .with_context(|| format!("cmake command failed: {:#?}", cmd.print_command()))?;

        debug!("Installer finished");

        let install_info = InstallInfo {
            name: self.name.clone(),
            commit: commit.clone(),
            date: date.clone(),
            duration: duration.as_secs(),
            branch: self.branch.clone(),
        };
        debug!("Writing installer info to {:?}", info_path);
        crate::write_json(&info_path, &install_info)?;

        info!(
            "Finished installing {} ({}) in {} minutes.",
            commit,
            date,
            duration.as_secs() / 60
        );

        debug!("Installation info written to {:?}", info_path);

        print_include_dir(&install);

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallInfo {
    pub name: String,
    pub branch: String,
    pub commit: String,
    pub date: String,
    pub duration: u64,
}

fn print_include_dir(path: &PathBuf) {
    let dir = path.to_str().unwrap();
    println!("Include path using:\nexport PATH={dir}:{dir}/bin:{dir}/include:$PATH; export LD_LIBRARY_PATH={dir}:{dir}:{dir}");
}
