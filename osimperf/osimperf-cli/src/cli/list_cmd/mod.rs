use crate::{
    read_json,
    record::{BenchTestResult, BenchTestSetup, TestNode, TEST_SETUP_FILE_NAME},
    CMakeCommands, Commit, CompilationNode, Ctxt, Date, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use osimperf_lib::common::find_file_by_name;
use rand::prelude::*;
use std::path::PathBuf;

use super::InstallInfo;

#[derive(Debug, Args)]
pub struct ListCommand {
    /// Path to directory in which to search for installed versions of opensim.
    #[arg(long, short)]
    install: Option<PathBuf>,

    /// Show date of install.
    #[arg(long, short)]
    date: bool,

    /// Show hash of install.
    #[arg(long, short)]
    commit: bool,

    /// Show path to install.
    #[arg(long, short)]
    path: bool,

    /// Path to results directory.
    #[arg(long, short)]
    results: Option<PathBuf>,

    /// Path to test cases directory.
    #[arg(long, short)]
    tests: Option<PathBuf>,
}

impl ListCommand {
    pub fn run(&mut self) -> Result<()> {
        if (self.install.is_none() && self.results.is_none()) && self.tests.is_none() {
            let dir = std::env::current_dir()?;
            self.install = Some(dir.clone());
            self.results = Some(dir.clone());
            self.tests = Some(dir.clone());
        }

        if let Some(install) = self.install.as_ref() {
            let mut arr = Vec::from_iter(
                find_file_by_name(install, crate::INSTALL_INFO_FILE_NAME)
                    .drain(..)
                    .map(|path| {
                        (
                            read_json::<InstallInfo>(&path).expect(&format!(
                                "failed to read install info from {}",
                                path.display()
                            )),
                            path,
                        )
                    }),
            );
            arr.sort_by(|(a, _), (b, _)| a.date.cmp(&b.date));
            for (config, file) in arr.iter().rev() {
                let file = file.parent().unwrap().to_str().unwrap();
                let mut msg = String::new();
                if self.path {
                    msg.push_str(&format!("{}", file));
                }
                if self.date {
                    msg.push_str(&format!("{}", config.date));
                }
                if self.commit {
                    msg.push_str(&format!("{}", config.commit));
                }
                if msg.len() == 0 {
                    msg.push_str(&format!("{}", file));
                }
                println!("{msg}");
            }
        }

        if let Some(tests) = self.tests.as_ref() {
            let mut arr = find_file_by_name(tests, TEST_SETUP_FILE_NAME);
            arr.sort_by(|a, b| a.to_str().unwrap().cmp(b.to_str().unwrap()));
            for file in arr.iter() {
                println!("{}", file.to_str().unwrap());
            }
        }

        if let Some(results) = self.results.as_ref() {
            let mut arr = find_file_by_name(results, super::ResultInfo::filename());
            arr.sort_by(|a, b| a.to_str().unwrap().cmp(b.to_str().unwrap()));
            for file in arr.iter() {
                println!("{}", file.to_str().unwrap());
            }
        }
        Ok(())
    }
}
