use crate::{git::format_date, *};
use anyhow::{ensure, Context, Result};
use clap::Args;
use log::{debug, info, log_enabled, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    env::current_dir,
    fs::{create_dir_all, File},
    io::Write,
    path::{absolute, Path, PathBuf},
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
    installer: Option<PathBuf>,

    /// Path to opensim-core repo.
    #[arg(long, short)]
    opensim: Option<PathBuf>,

    /// Path to install, defaults to current directory.
    #[arg(long, short)]
    root: Option<PathBuf>,

    /// Path to build dir.
    #[arg(long, short)]
    build: Option<PathBuf>,

    /// Force reinstalling.
    #[arg(long, short)]
    force: bool,
}

impl InstallCommand {
    pub fn run(&self) -> Result<()> {
        // Get path to opensim-core source from argument or environmental variable.
        let source = arg_or_env_var(self.opensim.clone(), "OSPC_OPENSIM_SRC")?
            .context("failed to get path to opensim-source")?;
        trace!("Path to OpenSim-core source = {:?}", source);

        let commit = crate::common::git::read_current_commit(&source)?;
        let date = format_date(&crate::common::git::get_date(&source, &commit)?);

        info!(
            "OSimPerf: Start install command:\npath to opensim: {:?}\ncommit: {}\ndate: {}",
            source, commit, date
        );

        // Use directory of config file as root for installer.
        let install_root = if let Some(root) = self.root.clone() {
            root
        } else {
            current_dir()?.join(format!("install_{}_{}_{}", self.name, date, commit))
        };

        trace!("Installer root = {:?}", install_root);

        // Check if already installed.
        if let Ok(prev_version) = Command::parse(&format!(
            "{}/bin/osimperf-install-info commit",
            install_root.to_str().unwrap()
        ))
        .run_trim()
        {
            if !self.force && commit == prev_version {
                info!("Found installed commit {} ({}).", commit, date);
                print_prefix_path(&install_root);
                return Ok(());
            }
            warn!(
                "Overwriting previously installed commit {} ({}).",
                prev_version, date
            );
        }

        // Set environmental variables.
        let mut env_vars = vec![EnvVar::new("OSPC_OPENSIM_SRC", &source)];
        if let Some(build) = self.build.as_ref() {
            env_vars.push(EnvVar::new("OSPC_BUILD_DIR", build));
        }

        create_dir_all(&install_root)?;
        debug!("Created install directory {:?}", install_root);

        let installer = self
            .installer
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("osimperf-default-install-opensim");
        let cmd = Command::new(installer)
            .set_envs(&env_vars)
            .set_run_root(&install_root);
        debug!("Run installer:\n{}", cmd.print_command());

        let duration = if log_enabled!(log::Level::Trace) {
            cmd.run_and_stream(&mut std::io::stdout())
        } else {
            cmd.run_and_time()
        }
        .and_then(|output| output.into_duration())
        .with_context(|| format!("installer failed: {:#?}", cmd.print_command()))?;

        debug!("Installer finished");

        let install_info = InstallInfo {
            name: self.name.clone(),
            commit: commit.clone(),
            date: date.clone(),
            duration: duration.as_secs(),
        };

        install_info.install(&install_root)?;

        info!(
            "Finished installing {} ({}) in {} minutes.",
            commit,
            date,
            duration.as_secs() / 60
        );

        print_prefix_path(&install_root);

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallInfo {
    pub name: String,
    pub commit: String,
    pub date: String,
    pub duration: u64,
}

fn print_prefix_path(path: &Path) {
    let dir = path.to_str().unwrap();
    println!("Don't forget to prefix the path:\n{dir}/bin:{dir}/include:$PATH");
}

impl InstallInfo {
    pub fn try_read(cmd: &str) -> Result<Self> {
        Ok(Self {
            name: Command::parse(&format!("{cmd} name")).run_trim()?,
            commit: Command::parse(&format!("{cmd} commit")).run_trim()?,
            date: Command::parse(&format!("{cmd} date")).run_trim()?,
            duration: Command::parse(&format!("{cmd} duration"))
                .run_trim()?
                .parse::<u64>()?,
        })
    }

    pub fn install(&self, path: &Path) -> Result<()> {
        let install_path = path.join("bin").join("osimperf-install-info");
        println!("install_path = {:?}", install_path);

        let mut line = Vec::<String>::new();

        line.push("#!/bin/bash".to_owned());
        // line.push("set -eo".to_owned());

        let mut line_opt_a = Vec::<String>::new();
        let mut line_opt_b = Vec::<String>::new();

        line_opt_a.push(r#"if [ "$#" -eq 1 ] ; then"#.to_owned());
        let duration = format!("{}", self.duration);
        for (key, value) in [
            ("name", &self.name),
            ("commit", &self.commit),
            ("date", &self.date),
            ("path", &path.to_str().unwrap().to_owned()),
            ("duration", &duration),
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
        debug!("osimperf-install-info written to {:?}", install_path);

        Ok(())
    }
}
