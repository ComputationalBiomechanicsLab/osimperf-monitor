use crate::{git::format_date, *};
use anyhow::{ensure, Context, Result};
use clap::Args;
use log::{debug, info, log_enabled, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{absolute, PathBuf, Path}, io::Write,
};

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
    #[arg(long, short)]
    opensim: PathBuf,

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
        let source = absolute(self.opensim.clone())?;
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
            (None, None) => crate::common::git::read_current_commit(&source)?,
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
        let date = format_date(&crate::common::git::get_date(&source, &commit)?);

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
                info.install(info_path.parent().unwrap()).context("failed to install info")?;
                print_prefix_path(info_path.parent().unwrap());
                return Ok(());
            }
            warn!(
                "Overwriting previously installed commit {} ({}).",
                info.commit, info.date,
            );
        }

        // Verify url.
        debug!("Verify repository URL.");
        crate::common::git::verify_repository(&source, &self.url)?;

        // Verify commit part of branch.
        debug!("Verify branch.");
        ensure!(
            crate::common::git::was_commit_merged_to_branch(&source, &self.branch, &commit)?,
            format!("commit {} not part of branch {}", &commit, &self.branch)
        );

        // Checkout commit.
        warn!("Checkout {:?} to {}", source, commit);
        Command::parse(&format!(
            "git -C {} checkout {commit}",
            source.to_str().unwrap()
        ))
        .run_and_stream(&mut std::io::stdout())?;

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

        install_info.install(&info_path)?;

        info!(
            "Finished installing {} ({}) in {} minutes.",
            commit,
            date,
            duration.as_secs() / 60
        );

        debug!("Installation info written to {:?}", info_path);

        print_prefix_path(info_path.parent().unwrap());

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

fn print_prefix_path(path: &Path) {
    let dir = path.to_str().unwrap();
    println!("Don't forget to prefix the path:\n{dir}/bin:{dir}/include:$PATH");
}

impl InstallInfo {
    pub fn install(&self, path: &Path) -> Result<()> {

        let install_path = path.join("bin").join("osimperf-install-info");
        println!("install_path = {:?}", install_path);

        let mut line = Vec::<String>::new();

        line.push("#!/bin/bash".to_owned());
        // line.push("set -eo".to_owned());

        let dt_str = format!("{}",self.duration);

        let mut line_opt_a = Vec::<String>::new();
        let mut line_opt_b = Vec::<String>::new();

        line_opt_a.push(r#"if [ "$#" -eq 1 ] ; then"#.to_owned());
        for (key, value) in [
            ("name", &self.name),
            ("branch", &self.branch),
            ("commit", &self.commit),
            ("date", &self.date),
            ("path", &path.to_str().unwrap().to_owned()),
        ] {
            line_opt_a.push(format!("  if [ $1 == \"{}\" ] ; then", key));
            line_opt_a.push(format!("    echo {}", value));
            line_opt_a.push("    exit 0".to_owned());
            line_opt_a.push("  fi".to_owned());
            line_opt_b.push(format!("echo \"{},{}\"", key, value));
        }
        line_opt_a.push(r#"  echo "Unknown key passed.""#.to_owned());
        line_opt_a.push("  exit 1".to_owned());
        line_opt_a.push("fi".to_owned());

        line.extend(line_opt_a.drain(..));
        line.extend(line_opt_b.drain(..));

        let mut file = File::create(&install_path)?;
        for l in line.iter() {
            file.write_all(l.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }

        Command::parse(&format!("chmod +x {}", install_path.to_str().unwrap())).run_trim()?;

        Ok(())
    }
}
