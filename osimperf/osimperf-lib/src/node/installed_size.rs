use super::Focus;
use crate::{path_to_install, Command, CommandTrait, Id};
use anyhow::{Context, Result};
use log::info;

pub fn get_installed_size_mbytes<'a>(focus: Focus, id: &Id<'a>) -> Result<usize> {
    let dir = path_to_install(focus, id).to_str().unwrap().to_string();
    let cmd = Command::parse(&format!("du -sm {dir}"));
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
