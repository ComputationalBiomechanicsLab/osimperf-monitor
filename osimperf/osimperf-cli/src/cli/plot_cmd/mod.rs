use crate::{
    record::{BenchTestResult, BenchTestSetup, TestNode},
    write_json, CMakeCommands, Commit, Ctxt, Date, EnvVars, FileBackedStruct, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use rand::prelude::*;
use std::{
    fs::File,
    io::{LineWriter, Write, self},
    path::PathBuf, str::FromStr,
};

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Path to results directory.
    #[arg(long)]
    results: Option<PathBuf>,

    /// Output path.
    #[arg(long)]
    out: Option<PathBuf>,
}

impl PlotCommand {
    pub fn run(&self) -> Result<()> {

        for line in io::stdin().lines() {
            let path = PathBuf::from_str(&line?)?;
            println!("path = {:?}", path);
            let result = crate::read_json::<BenchTestResult>(&path)?;
            println!("{:?}", result);
        }

        Ok(())
    }
}
