use crate::*;
use anyhow::Result;
use clap::Args;
use std::path::{absolute, PathBuf};

use super::{InstallInfo, ResultInfo};

#[derive(Debug, Args)]
pub struct ListCommand {
    /// Path to directory in which to search for installed versions of opensim.
    #[arg(long, short)]
    install: Option<PathBuf>,

    /// Filter by date.
    #[arg(long, short)]
    date: Option<String>,

    /// Filter by commit.
    #[arg(long, short)]
    commit: Option<String>,

    /// Path to results directory.
    #[arg(long, short)]
    results: Option<PathBuf>,

    /// Path to test cases directory.
    #[arg(long, short)]
    tests: Option<PathBuf>,

    #[arg(long, short)]
    opensim_log: bool,
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
            let mut arr = Vec::new();
            for path in find_file_by_name(install, "osimperf-install-info")
                .drain(..)
                .map(|path| absolute(path).expect("failed to create absolute path"))
            {
                let cmd = path.to_str().unwrap().to_owned();
                arr.push((
                    InstallInfo {
                        name: Command::parse(&format!("{cmd} name")).run_trim()?,
                        commit: Command::parse(&format!("{cmd} commit")).run_trim()?,
                        date: Command::parse(&format!("{cmd} date")).run_trim()?,
                        duration: 0,
                    },
                    cmd
                ));
            }
            arr.sort_by(|(a, _), (b, _)| a.date.cmp(&b.date));
            if let Some(date) = self.date.as_ref() {
                arr.retain(|(c, _)| &c.date == date);
            }
            if let Some(commit) = self.commit.as_ref() {
                arr.retain(|(c, _)| &c.commit == commit);
            }
            for path in arr.iter().rev().map(|(_, p)| p) {
                println!("{}", path);
            }
            return Ok(());
        }

        if let Some(tests) = self.tests.as_ref() {
            let mut arr = find_file_by_name(tests, "osimperf-test.conf");
            arr.sort_by(|a, b| a.to_str().unwrap().cmp(b.to_str().unwrap()));
            for file in arr.iter() {
                println!("{}", file.to_str().unwrap());
            }
        }

        if let Some(results) = self.results.as_ref() {
            let mut arr = find_file_by_name(results, super::ResultInfo::filename());
            arr.sort_by(|a, b| a.to_str().unwrap().cmp(b.to_str().unwrap()));
            if let Some(commit) = self.commit.as_ref() {
                arr.retain(|path| {
                    let result_info = read_json::<ResultInfo>(&path).expect(&format!(
                        "failed to read result info from {}",
                        path.display()
                    ));
                    &result_info.commit == commit
                });
            }
            for file in arr.iter() {
                if self.opensim_log {
                    let result_info = read_json::<ResultInfo>(&file).expect(&format!(
                        "failed to read result info from {}",
                        file.display()
                    ));
                    if let Some(log) = result_info.opensim_log {
                println!("{}", log.to_str().unwrap());
                    }
                } else {
                println!("{}", file.to_str().unwrap());
                }
            }
        }
        Ok(())
    }
}
