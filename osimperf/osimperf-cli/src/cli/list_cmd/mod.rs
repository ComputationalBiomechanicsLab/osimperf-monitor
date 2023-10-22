use crate::*;
use anyhow::Result;
use clap::Args;
use osimperf_lib::common::find_file_by_name;
use std::path::{absolute, Path, PathBuf};

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
}

fn into_prefix_path(path: &Path) -> String {
    let path = path.to_str().unwrap().to_owned();
    let mut out = path.clone();
    out.push_str(":");
    out.push_str(&path);
    out.push_str("/bin");
    out.push_str(":");
    out.push_str(&path);
    out.push_str("/lib");
    out
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
                    .map(|path| absolute(path).expect("failed to create absolute path"))
                    .map(|path| {
                        (
                            read_json::<InstallInfo>(&path).expect(&format!(
                                "failed to read install info from {}",
                                path.display()
                            )),
                            into_prefix_path(path.parent().unwrap()),
                        )
                    }),
            );
            arr.sort_by(|(a, _), (b, _)| a.date.cmp(&b.date));
            let mut i = 0;
            loop {
                if i >= arr.len() {
                    break;
                }
                if let Some(j) = find_other(&arr, i) {
                    let append = arr[j].1.clone();
                    arr[i].1.push_str(":");
                    arr[i].1.push_str(&append);
                    // Remove duplicate version.
                    arr.remove(j);
                    continue;
                }
                i += 1;
            }
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
                println!("{}", file.to_str().unwrap());
            }
        }
        Ok(())
    }
}

fn find_other(arr: &[(InstallInfo, String)], index: usize) -> Option<usize> {
    for i in (0..arr.len())
        // Prevent matching on self.
        .filter(|&i| i != index)
        // Check if version matches.
        .filter(|&i| arr[i].0.commit == arr[index].0.commit)
    {
        return Some(i);
    }
    None
}
