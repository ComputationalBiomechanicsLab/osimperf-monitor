use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::{CommandOutput, Folder, Id, NodeFile, ResultsFolder};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BenchTestResult {
    pub duration: Option<f64>,
    pub iteration: usize,
    pub failed_count: usize,
    pub path_to_root: PathBuf,
}

impl NodeFile for BenchTestResult {
    const SUBFOLDER_LEVEL: usize = 2;

    fn path_to_self(&self) -> PathBuf {
        self.path_to_root.join(Self::MAGIC_FILE())
    }
}

impl BenchTestResult {
    pub(crate) const fn MAGIC_FILE() -> &'static str {
        ".osimperf-result.node"
    }

    fn new_helper<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Self> {
        let path_to_root = results.path()?.join(id.subfolder_name()).join(name);
        Ok(Self {
            duration: None,
            iteration: 0,
            failed_count: 0,
            path_to_root,
        })
    }

    pub(crate) fn new<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Self> {
        let mut out = Self::new_helper(results, id, name)?;
        out.read_or_write_new()?;
        Ok(out)
    }

    pub fn read<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Option<Self>> {
        let mut out = Self::new_helper(results, id, name)?;
        let success = out.try_read().is_ok();
        Ok(Some(out).filter(|_| success))
    }

    pub(crate) fn process(&mut self, cmd_output: CommandOutput) -> Result<()> {
        if !cmd_output.success() {
            self.failed_count += 1;
            self.duration = None;
            self.iteration = 0;
        } else {
            let dt = self.duration.get_or_insert(0.);
            let count = self.iteration.min(99) as f64;
            *dt = (*dt * count + cmd_output.duration.as_secs_f64()) / (count + 1.);
            self.iteration += 1;
        }
        info!("Updating result: {:#?}", &self);
        self.try_write()?;
        Ok(())
    }
}
