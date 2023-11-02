// mod run_cmd;
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

use anyhow::{Context, Result};
use std::io::Lines;
use std::io::StdinLock;
use std::path::PathBuf;
use std::str::FromStr;

use crate::Command;
use crate::CommandTrait;

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

pub struct ArgOrStdinIter {
    arg: Option<PathBuf>,
    stdin: Option<Lines<StdinLock<'static>>>,
}

impl ArgOrStdinIter {
    pub fn new(arg: &Option<PathBuf>) -> Self {
        Self {
            arg: arg.clone(),
            stdin: if arg.is_none() {
                Some(std::io::stdin().lines())
            } else {
                None
            },
        }
    }
}

impl Iterator for ArgOrStdinIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stdin) = self.stdin.as_mut() {
            stdin
                .next()
                .map(|s| s.expect("failed to read stdin"))
                .map(|s| PathBuf::from_str(&s).expect("failed to create PathBuf from str"))
        } else {
            return self.arg.take();
        }
        .map(|path| absolute_path(&path).expect("failed to create absolute_path"))
    }
}

/// Substitute occurances of `%H`, and `%n`.
pub fn substitute_install_info(mut s: String) -> String {
    for (key, value) in [
        (
            "%H",
            Command::parse("osimperf-install-info commit")
                .run_trim()
                .expect("failed to read commit"),
        ),
        (
            "%n",
            Command::parse("osimperf-install-info name")
                .run_trim()
                .expect("failed to read name"),
        ),
    ] {
        s = s.replace(key, &value);
    }
    s
}
