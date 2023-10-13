mod install_cmd;
mod record_cmd;
mod list_cmd;
mod plot_cmd;
mod log_cmd;

pub use install_cmd::{
    InstallInfo,
    InstallCommand};
pub use record_cmd::RecordCommand;
pub use list_cmd::ListCommand;
pub use plot_cmd::PlotCommand;
pub use log_cmd::LogCommand;

use anyhow::Context;

pub fn absolute_path(relative_path: &std::path::PathBuf) -> anyhow::Result<std::path::PathBuf> {
    std::fs::canonicalize(relative_path)
        .with_context(|| format!("failed to create absolute path to {:?}", relative_path))
}
