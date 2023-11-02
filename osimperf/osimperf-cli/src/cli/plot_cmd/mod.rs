mod plot;
mod table;

use plot::print_csv_plot;
use table::print_table;

use super::ArgOrStdinIter;
use super::ResultInfo;
use anyhow::{Context, Result};
use clap::Args;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Path to result file (or reads from stdin).
    #[arg(long)]
    results: Option<PathBuf>,

    /// Output path.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Table.
    #[arg(long, short)]
    table: bool,
}

impl PlotCommand {
    pub fn run(&self) -> Result<()> {
        if let Some(path) = self.out.as_ref() {
            let mut file = File::create(path).with_context(|| {
                format!(
                    "failed to open file for writing stdout logs at path = {:?}",
                    &self.out
                )
            })?;
            if self.table {
                print_table(&self.results, &mut file)?;
            } else {
                print_csv_plot(&self.results, &mut file)?;
            }
        } else {
            if self.table {
                print_table(&self.results, std::io::stdout())?;
            } else {
                print_csv_plot(&self.results, std::io::stdout())?;
            }
        }

        Ok(())
    }
}
