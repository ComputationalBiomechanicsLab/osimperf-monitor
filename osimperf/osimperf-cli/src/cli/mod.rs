mod install_cmd;
mod list_cmd;
mod log_cmd;
mod plot_cmd;
mod record_cmd;

use anyhow::ensure;
pub use install_cmd::{InstallCommand, InstallInfo};
pub use list_cmd::ListCommand;
pub use log_cmd::LogCommand;
pub use plot_cmd::PlotCommand;
pub use record_cmd::{ReadBenchTestSetup, RecordCommand, ResultInfo};

use crate::read_json;
use crate::Command;
use crate::CommandTrait;
use crate::INSTALL_INFO_FILE_NAME;
use anyhow::{anyhow, Context, Result};
use log::debug;
use std::path::PathBuf;

pub fn absolute_path(relative_path: &PathBuf) -> Result<PathBuf> {
    std::fs::canonicalize(relative_path)
        .with_context(|| format!("failed to create absolute path to {:?}", relative_path))
}

/// Returns the absolute path of the arg or checks the env vars.
pub fn arg_or_env_var(arg: Option<PathBuf>, key: &str) -> Result<Option<PathBuf>> {
    arg.or_else(|| {
        std::env::vars()
            .find(|(k, _)| k == key)
            .map(|(_, value)| PathBuf::from(value))
    })
    .map(|relative| absolute_path(&relative))
    .transpose()
}

/// Prefixes selected env vars with path.
pub fn prefix_path(keys: &[&str], prefix_path: &String) -> Result<()> {
    ensure!(prefix_path.len() > 0, "Prefix path is empty string.");
    for key in keys {
        let mut value = prefix_path.to_owned();
        if let Ok(e) = std::env::var(key) {
        value.push_str(":");
        value.push_str(&e);
        }
        std::env::set_var(key, value);
    }
    Ok(())
}

/// Finds the opensim-core installation using the PATH env var.
pub fn find_install_info_on_path() -> Result<InstallInfo> {
    let path_env = std::env::var("PATH")?;
    debug!("PATH={path_env}");

    // Split on ':' and search for first `osimperf-install-info.json` on PATH.
    let install_info_path = path_env
        .split(":")
        .map(|path| PathBuf::from(path))
        .map(|path| PathBuf::from(path).join(INSTALL_INFO_FILE_NAME))
        .find(|path| path.exists())
        .with_context(|| format!("{INSTALL_INFO_FILE_NAME} not found in path\nPATH={path_env}"))?;
    debug!("Path to InstallInfo: {:?}", install_info_path);

    // Parse InstallInfo.
    let install_info = read_json::<InstallInfo>(&install_info_path)
        .context("failed to find opensim installation")
        .with_context(|| format!("failed to locate {INSTALL_INFO_FILE_NAME}"))?;
    debug!("InstallInfo: {:?}", install_info);

    // Verify version against `opensim-cmd --version`.
    let version_output = Command::parse("opensim-cmd --version").run_trim()?;
    if !version_output.contains(install_info.commit.split_at(9).0) {
        Err(anyhow!(
            "Expected version {}\nGot version {}",
            install_info.commit,
            version_output
        ))
        .with_context(|| format!("PATH={path_env}"))
        .context("Failed to verify opensim version.")?;
    }

    // Return the active opensim-core version.
    Ok(install_info)
}
