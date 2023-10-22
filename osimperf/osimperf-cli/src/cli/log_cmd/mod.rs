use crate::*;
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct LogCommand {
    /// Path to repository directory.
    #[arg(long)]
    path: String,

    /// Date.
    #[arg(long)]
    date: Option<String>,

    /// Specify branch name (defaults to main).
    #[arg(long)]
    branch: String,
}

impl LogCommand {
    pub fn run(&self) -> Result<()> {
        let mut cmd = Command::parse(&format!(
            "git -C {} log {} --pretty=format:%H",
            self.path, self.branch,
        ));

        if let Some(date) = self.date.as_ref() {
            cmd.add_arg(format!("--before={}", date));
        }

        let output = cmd.run()?;
        if let Some(line) = output.lines().next() {
            println!("{line}");
        }

        Ok(())
    }
}
