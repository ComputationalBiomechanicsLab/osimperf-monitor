use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::hash::{Hash, Hasher};

use super::Focus;

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
        // TODO refine this?
        !self.is_done()
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
            Status::Idle => "idle".to_string(),
            Status::Compiling(Progress { percentage, .. }) => format!("{:.2}%", percentage),
            Status::Error(_) => "error".to_string(),
            Status::Done(Complete { duration, .. }) => format!("{}", duration.as_secs() / 60),
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
    pub status_dependencies: Status,
    pub status_opensim_core: Status,
    pub status_tests_source: Status,
}

impl State {
    pub fn set(&mut self, focus: Focus, status: Status) {
        match focus {
            Focus::Dependencies => self.status_dependencies = status,
            Focus::OpenSimCore => self.status_opensim_core = status,
            Focus::TestsSource => self.status_tests_source = status,
        }
    }

    pub fn get(&self) -> [&Status; 3] {
        return [
            &self.status_dependencies,
            &self.status_opensim_core,
            &self.status_tests_source,
        ];
    }

    pub fn get_compiler_list(&self) -> [Option<Focus>; 3] {
        [
            Some(Focus::Dependencies),
            Some(Focus::OpenSimCore),
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
            Focus::OpenSimCore => match self.status_opensim_core {
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

        self.process.hash(state);
    }
}
