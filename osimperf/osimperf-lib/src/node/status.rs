use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Idle,
    Queued,
    Compiling(Progress),
    BuildFailed,
    CmdError,
    Done,
}

impl Default for Status {
    fn default() -> Self {
        Self::Idle
    }
}

/// The three compilation targets:
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub status_opensim_core: Status,
    pub status_dependencies: Status,
    pub status_tests_source: Status,
}

impl State {
    pub fn set(&mut self, focus: &Focus, status: &Status) {
        todo!()
    }

    pub fn get(&mut self) -> &Status {
        todo!()
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Error {}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Focus {
    OpenSimCore,
    Dependencies,
    TestsSource,
}
