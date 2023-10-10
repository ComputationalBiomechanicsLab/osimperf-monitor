use super::Durations;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::{CommandOutput, Ctxt, InstallId};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct BenchTestResult {
    status: Option<Status>,
    // For detecting changed config.
    hash: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Status {
    Failed(String),
    Success(Durations),
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

    pub fn status(&self) -> Result<Durations> {
        todo!()
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
        todo!()
    }
}
