use crate::{
    record::{BenchTestResult, BenchTestSetup, TestNode, TEST_SETUP_FILE_NAME},
    CMakeCommands, Commit, CompilationNode, Ctxt, Date, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use osimperf_lib::common::find_file_by_name;
use rand::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ListCommand {
    /// Path to archive directory.
    #[arg(long)]
    archive: Option<PathBuf>,

    /// Path to results directory.
    #[arg(long)]
    results: Option<PathBuf>,

    /// Path to test cases directory.
    #[arg(long)]
    tests: Option<PathBuf>,
}

impl ListCommand {
    pub fn run(&mut self) -> Result<()> {
        if (self.archive.is_none() && self.results.is_none()) && self.tests.is_none() {
            let dir = std::env::current_dir()?;
            self.archive = Some(dir.clone());
            self.results = Some(dir.clone());
            self.tests = Some(dir.clone());
        }

        if let Some(archive) = self.archive.as_ref() {
            for file in find_file_by_name(archive, CompilationNode::magic_file()) {
                println!("{}", file.to_str().unwrap());
            }
        }

        if let Some(tests) = self.tests.as_ref() {
            for file in find_file_by_name(tests, TEST_SETUP_FILE_NAME) {
                println!("{}", file.to_str().unwrap());
            }
        }

        if let Some(results) = self.results.as_ref() {
            for file in find_file_by_name(results, BenchTestResult::magic_file()) {
                println!("{}", file.to_str().unwrap());
            }
        }
        Ok(())
    }
}
