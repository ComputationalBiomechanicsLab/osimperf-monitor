use log::warn;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::Focus;

// TODO status improvements
// size of install
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Idle,
    Queued,
    Compiling(Progress),
    BuildFailed,
    CmdError,
    Done(Complete),
}

impl Status {
    pub fn update(&mut self, res: &anyhow::Result<Duration>) {
        match (&self, res) {
            (_, Ok(duration)) => {
                *self = Status::Done(Complete {
                    duration: *duration,
                    size: 0,
                })
            }
            _ => *self = Status::BuildFailed,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::Idle
    }
}

/// The three compilation targets:
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub status_dependencies: Status,
    pub status_opensim_core: Status,
    pub status_tests_source: Status,
}

impl State {
    pub fn update(&mut self, res: [anyhow::Result<Duration>; 3]) {
        self.status_dependencies.update(&res[0]);
        self.status_opensim_core.update(&res[1]);
        self.status_tests_source.update(&res[2]);
    }

    pub fn set(&mut self, focus: &Focus, status: Status) {
        match focus {
            Focus::Dependencies => self.status_dependencies = status,
            Focus::OpenCimCore => self.status_opensim_core = status,
            Focus::TestsSource => self.status_tests_source = status,
        }
    }

    pub fn get_compiler_list(&self) -> [Option<Focus>; 3] {
        [
            Some(Focus::Dependencies),
            Some(Focus::OpenCimCore),
            Some(Focus::TestsSource),
        ]
        .map(|f| f.filter(|x| self.get_compiler_job(x)))
    }

    fn get_compiler_job(&self, focus: &Focus) -> bool {
        match focus {
            Focus::Dependencies => match self.status_dependencies {
                Status::Done(_) => return false,
                _ => return true,
            },
            Focus::OpenCimCore => match self.status_opensim_core {
                Status::Done(_) => return false,
                _ => return true,
            },
            Focus::TestsSource => match self.status_tests_source {
                Status::Done(_) => return false,
                _ => return true,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub percentage: f64,
    pub process: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Complete {
    pub duration: Duration,
    pub size: u64,
}
