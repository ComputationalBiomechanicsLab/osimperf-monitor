use std::path::Path;
use anyhow::Result;
use anyhow::Context;
use osimperf_lib::Command;
use osimperf_lib::CommandTrait;

pub fn folder_size(dir: &Path) -> Result<usize> {
    let cmd = Command::parse(&format!("du -sm {}", dir.to_str().unwrap()));
    let output = cmd.run_trim()?;
    let word = output
        .split('\t')
        .next()
        .with_context(|| format!("{}", cmd.print_command()))
        .context("failed to get first argument from command output")?;
    Ok(word
        .parse::<usize>()
        .with_context(|| format!("Failed to parse word: {}", word))?)
}
