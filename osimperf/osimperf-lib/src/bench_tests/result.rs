use super::Durations;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::{Folder, Id, NodeFile, ResultsFolder};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BenchTestResult {
    pub hash: Option<u64>,
    pub failed_count: usize,
    pub durations: Durations,
    pub path_to_self: PathBuf,
}

impl NodeFile for BenchTestResult {
    const SUBFOLDER_LEVEL: usize = 2;

    fn path_to_self(&self) -> PathBuf {
        self.path_to_self.clone()
    }
}

impl BenchTestResult {
    pub(crate) const fn magic_file() -> &'static str {
        ".osimperf-result.node"
    }

    /// Returns the path that this result would be stored at.
    pub fn path_to_node<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> PathBuf {
        results
            .path()
            .unwrap()
            .join(id.subfolder_name())
            .join(name)
            .join(Self::magic_file())
    }

    fn new_helper<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Self {
        Self {
            path_to_self: Self::path_to_node(results, id, name),
            ..Default::default()
        }
    }

    pub(crate) fn new<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Self> {
        let mut out = Self::new_helper(results, id, name);
        out.read_or_write_new()?;
        Ok(out)
    }

    pub fn read<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Option<Self>> {
        let mut out = Self::new_helper(results, id, name);
        let success = out.try_read().is_ok();
        Ok(Some(out).filter(|_| success))
    }

    pub(crate) fn should_run(&self, max_iter: usize, max_failures: usize) -> bool {
        self.durations.len() < max_iter && self.failed_count < max_failures
    }

    pub(crate) fn update_hash(&mut self, hash: u64) {
        if self.hash == Some(hash) {
            return;
        }
        debug!("Changed config detected! Reset test result");
        self.failed_count = 0;
        self.durations.clear();
        self.hash = Some(hash);
    }

    pub(crate) fn update_result(&mut self, cmd_output: Option<Duration>) {
        if let Some(duration) = cmd_output {
            // If the command was succesfully executed:
            self.durations.add_sample(duration);
        } else {
            // If the command was failed.
            self.failed_count += 1;
            self.durations.clear();
        }
    }
}
