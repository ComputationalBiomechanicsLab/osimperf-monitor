use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Idle,
    Compiling(Progress),
    Error(String),
    Done(Duration),
}

impl Status {
    pub fn failed(&self) -> Option<&String> {
        if let Self::Error(s) = self {
            return Some(s);
        }
        None
    }

    pub fn done(&self) -> bool {
        if let Self::Done(_) = self {
            return true;
        }
        false
    }

    pub fn from_output(
        output: anyhow::Result<Duration>,
    ) -> Self {
        match output {
            Ok(duration) => Self::Done(duration),
            Err(err) => Self::Error(format!("{:?}", err)),
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub percentage: f64,
    pub task: String,
}
