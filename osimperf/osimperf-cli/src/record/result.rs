use super::{durations, Durations};
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::{CommandOutput, Ctxt, InstallId};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BenchTestResult {
    pub status: Option<Status>,
    // For detecting changed config.
    pub hash: Option<u64>,

    pub date: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Status {
    Failed(String),
    Success(Durations),
}

impl Status {
    fn add_sample(&mut self, duration: Duration) {
        if let Self::Failed(_) = self {
            *self = Self::Success(Durations::default());
        }
        if let Self::Success(durations) = self {
            durations.add_sample(duration);
        }
    }
}

impl BenchTestResult {
    pub(crate) const fn magic_file() -> &'static str {
        "osimperf-result.data"
    }

    pub fn default_path_to_file<'a>(context: &Ctxt, id: &InstallId<'a>, name: &str) -> PathBuf {
        context
            .results()
            .join(id.subfolder_name())
            .join(name)
            .join(Self::magic_file())
    }

    fn update_hash(&mut self, hash: u64) {
        if self.hash == Some(hash) {
            return;
        }
        debug!("Changed config detected! Reset test result");
        self.status = None;
        self.hash = Some(hash);
    }

    pub fn update_result(&mut self, cmd_output: CommandOutput) {
        if !cmd_output.success() {
            self.status = Some(Status::Failed(format!("{:#?}", cmd_output)));
            return;
        }

        self.status
            .get_or_insert(Status::Success(Durations::default()))
            .add_sample(cmd_output.duration);
    }

    pub fn get_durations(&self) -> Option<&Durations> {
        if let Some(Status::Success(durations)) = self.status.as_ref() {
            return Some(&durations);
        }
        None
    }
}
