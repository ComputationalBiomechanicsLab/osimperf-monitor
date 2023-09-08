use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::time::Duration;

use super::CompilationTarget;

// TODO status improvements
// size of install
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub enum Status {
    Idle,
    Compiling(Progress),
    Error(String),
    Done(Duration),
}

impl Status {
    pub fn should_compile(&self) -> bool {
        match self {
            // Do not recompile if already previously succeeded or failed.
            Status::Done(_) | Status::Error(_) => false,
            _ => true,
        }
    }

    pub fn has_failed(&self) -> bool {
        if let Self::Error(_) = self {
            return true;
        }
        false
    }

    pub fn is_done(&self) -> bool {
        if let Self::Done(_) = self {
            return true;
        }
        false
    }

    pub fn from_output(
        // TODO refine output and status construction
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

/// The three compilation targets:
#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct State {
    /// One status for each target.
    status: [Status; 3],
}

impl State {
    pub fn set(&mut self, target: CompilationTarget, status: Status) {
        self.status[target as usize] = status;
    }

    pub fn reset(&mut self) {
        self.status.iter_mut().for_each(|s| *s = Status::Idle);
    }

    pub fn get(&self) -> &[Status; 3] {
        &self.status
    }

    pub fn status(&self, target: CompilationTarget) -> &Status {
        &self.status[target as usize]
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub percentage: f64,
}

impl Hash for Progress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert the f64 value to its raw representation as a u64
        let bits = self.percentage.to_bits();
        bits.hash(state);
    }
}
