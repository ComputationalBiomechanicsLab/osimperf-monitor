mod install_cmd;
mod list_cmd;
mod log_cmd;
mod plot_cmd;
mod record_cmd;

pub use install_cmd::{InstallCommand, InstallInfo};
pub use list_cmd::ListCommand;
pub use log_cmd::LogCommand;
pub use plot_cmd::PlotCommand;
pub use record_cmd::RecordCommand;

use std::path::PathBuf;
use anyhow::{Context, Result};

pub fn absolute_path(relative_path: &PathBuf) -> Result<PathBuf> {
    std::fs::canonicalize(relative_path)
        .with_context(|| format!("failed to create absolute path to {:?}", relative_path))
}

/// Returns the absolute path of the arg or checks the environmental variables.
pub fn arg_or_env_var(arg: Option<PathBuf>, key: &str) -> Result<Option<PathBuf>> {
    arg.or_else(|| {
        std::env::vars()
            .find(|(k, _)| k == key)
            .map(|(_, value)| PathBuf::from(value))
    })
    .map(|relative| absolute_path(&relative))
    .transpose()
}
