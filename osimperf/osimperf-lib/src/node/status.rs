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
    Done(Complete),
}

impl Status {
    pub fn should_compile(&self) -> bool {
        match self {
            // Do not recompile if already previously succeeded or failed.
            Status::Done(_) | Status::Error(_) => false,
            _ => true,
        }
    }

    pub fn is_done(&self) -> bool {
        if let Self::Done(_) = self {
            return true;
        }
        false
    }

    pub fn from_output(
        // TODO refine output and status construction
        output: anyhow::Result<Complete>,
    ) -> Self {
        match output {
            Ok(done) => Self::Done(done),
            Err(err) => Self::Error(format!("{:?}", err)),
        }
    }

    pub fn print_table_entry(&self) -> String {
        return match self {
            Status::Idle => "Queued".to_string(),
            Status::Compiling(Progress { percentage, .. }) => format!("{:.2}%", percentage),
            Status::Error(_) => "Error".to_string(),
            Status::Done(Complete { duration, size }) => format!(
                "{} [min], {} [Gb]",
                duration.as_secs() / 60,
                *size as f64 / 1000.
            ),
        };
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub percentage: f64,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Complete {
    pub duration: Duration,
    pub size: usize,
}

impl Hash for Progress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert the f64 value to its raw representation as a u64
        let bits = self.percentage.to_bits();
        bits.hash(state);
    }
}
