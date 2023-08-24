use std::{io::Read, sync::{Arc, RwLock, Mutex}};

use crate::{Commit, Date, Hash, Repository};

pub struct Nodes {}

pub struct CompilationNode {
    repo: Arc<Mutex<Repository>>,
    commit: Commit,
    status: CompilationStatus,
}

pub enum CompilationStatus {
    Queued,
    Failed { exitCode: usize },
    CompilingSimbody { percentage: f64 },
    CompilingOpenSim { percentage: f64 },
    CompilingSource { percentage: f64 },
    Complete { time: f64, size: f64 },
}

pub struct CompilationStatusReader {}

impl CompilationNode {
    pub fn status(&self) -> &CompilationStatus {
        &self.status
    }

    pub fn date(&self) -> Date {
        todo!()
    }

    pub fn hash(&self) -> Hash {
        todo!()
    }

    pub fn read_log(&self, sink: impl Read) -> Date {
        todo!()
    }

    pub fn compile(&mut self) {
        todo!()
    }

    pub fn get_status_reader(&self) -> CompilationStatusReader {
        todo!()
    }
}
