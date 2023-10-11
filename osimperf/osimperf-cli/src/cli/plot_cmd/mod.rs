use crate::{
    common::format_date,
    record::{BenchTestResult, BenchTestSetup, TestNode},
    write_json, CMakeCommands, Commit, Ctxt, Date, EnvVars, FileBackedStruct, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use rand::prelude::*;
use std::{
    fs::File,
    io::{self, LineWriter, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Path to results directory.
    #[arg(long)]
    results: Option<PathBuf>,

    /// Output path.
    #[arg(long)]
    out: PathBuf,
}

impl PlotCommand {
    pub fn run(&self) -> Result<()> {
        let file = File::create(&self.out).with_context(|| {
            format!(
                "failed to open file for writing stdout logs at path = {:?}",
                &self.out
            )
        })?;
        let mut file = LineWriter::new(file);
        let mut first_date = None;
        for line in io::stdin().lines() {
            let path = PathBuf::from_str(&line?)?;
            println!("path = {:?}", path);
            let result = crate::read_json::<BenchTestResult>(&path)?;
            println!("{:?}", result);

            let date = crate::common::parse_date(&result.date)?;
            let first_date = first_date.get_or_insert_with(|| date.clone());

            let days = (date - *first_date).num_days();
            // info!("{} - {} = {} days", date, first_date, days);
            if let Some(durations) = result.get_durations() {
                for duration in durations.get().iter() {
                    file.write_all(format!("{days},").as_bytes())
                        .with_context(|| format!("Failed to write number of days to {:?}", path))?;
                    file.write_all(format_date(&date).as_bytes())
                        .with_context(|| format!("Failed to write date to {:?}", path))?;
                    file.write_all(format!(",{}\n", duration.as_secs_f64()).as_bytes())
                        .with_context(|| format!("Failed to write duration to {:?}", path))?;
                }
            }
        }
        file.flush()
            .with_context(|| format!("Failed to flush {:?}", self.out))?;

        Ok(())
    }
}
