use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{CommandOutput, Folder, Id, NodeFile, ResultsFolder};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BenchTestResult {
    pub hash: Option<u64>,
    pub duration: Option<f64>,
    pub duration_stddev: Option<f64>,
    pub iteration: usize,
    pub failed_count: usize,
    pub path_to_node: PathBuf,
}

impl NodeFile for BenchTestResult {
    const SUBFOLDER_LEVEL: usize = 2;

    fn path_to_self(&self) -> PathBuf {
        self.path_to_node.join(Self::MAGIC_FILE())
    }
}

impl BenchTestResult {
    pub(crate) const fn MAGIC_FILE() -> &'static str {
        ".osimperf-result.node"
    }

    fn new_helper<'a>(results: &ResultsFolder, id: &Id<'a>, name: &str) -> Result<Self> {
        let path_to_root = results.path()?.join(id.subfolder_name()).join(name);
        Ok(Self {
            hash: None,
            duration: None,
            duration_stddev: None,
            iteration: 0,
            failed_count: 0,
            path_to_node: path_to_root,
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

    fn reset(&mut self) {
        *self = Self {
            hash: None,
            duration: None,
            duration_stddev: None,
            iteration: 0,
            failed_count: 0,
            path_to_node: self.path_to_node.clone(),
        };
    }

    pub(crate) fn process(&mut self, cmd_output: CommandOutput, hash: u64) -> Result<()> {
        // Check hash for config changes.
        if self.hash != Some(hash) {
            warn!("Changed config detected! Reset test result");
            self.reset();
        }
        self.hash = Some(hash);
        if !cmd_output.success() {
            self.failed_count += 1;
            self.duration = None;
            self.iteration = 0;
        } else {
            let count = self.iteration.min(99) as f64;
            let measured_dt = cmd_output.duration.as_secs_f64();
            let dt = self.duration.get_or_insert(measured_dt);
            // Update stddev estimate duration.
            if self.iteration == 1 {
                self.duration_stddev =  Some((*dt - measured_dt).abs());
            }
            if self.iteration > 1 {
                let stddev = self.duration_stddev.get_or_insert(f64::NAN);
                let measured_var = (measured_dt - *dt).powi(2);
                let filtered_var = stddev.powi(2);
                let var = (filtered_var * count + measured_var) / (count + 1.);
                *stddev = var.sqrt();
            }
            // Update mean estimate duration.
            *dt = (*dt * count + measured_dt) / (count + 1.);
            self.iteration += 1;
        }
        info!("Updating result: {:#?}", &self);
        self.try_write()?;
        Ok(())
    }
}
