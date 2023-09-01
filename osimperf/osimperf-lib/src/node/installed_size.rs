use anyhow::{Context, Result};
use crate::{Command, CommandTrait};
use log::info;
use std::path::Path;

pub fn folder_size(dir: &Path) -> Result<usize> {
    let cmd = Command::parse(&format!("du -sm {}", dir.to_str().unwrap()));
    let output = cmd.run_trim()?;
    let word = output
        .split('\t')
        .next()
        .with_context(|| format!("{}", cmd.print_command()))
        .context("failed to get first argument from command output")?;
    info!("Running command to get size: {}", cmd.print_command());
    Ok(word
        .parse::<usize>()
        .with_context(|| format!("Failed to parse word: {}", word))?)
}
