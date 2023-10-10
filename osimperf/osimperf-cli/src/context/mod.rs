mod env_vars;
mod install_id;

pub use env_vars::*;
pub use install_id::InstallId;

use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Ctxt {
    archive: Option<PathBuf>,
    build: Option<PathBuf>,
    results: Option<PathBuf>,
    tests: Option<PathBuf>,
    models: Option<PathBuf>,
    /// Path to opensim-core repository.
    opensim_core: Option<PathBuf>,
}

/// Returns current working directory, and checks if it is the home of osimperf.
fn checked_working_dir() -> Result<PathBuf> {
    // Get current directory.
    let p = std::env::current_dir()?;
    // Verify that it is the osimperf-home dir.
    let magic_file = p.join(".osimperf-home");
    Some(p.clone())
        .filter(|_| magic_file.exists())
        .with_context(|| format!("unable to find magic file {:?}", magic_file))
        .with_context(|| format!("{:?} doesnt look like the correct directory", p))
}

impl Ctxt {
    pub fn set_archive(&mut self, archive: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = archive {
            self.archive = Some(fs::canonicalize(dir)?);
        } else {
            self.archive = Some(checked_working_dir()?.join("archive"));
        }
        Ok(())
    }

    pub fn set_build(&mut self, build: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = build {
            self.build = Some(fs::canonicalize(dir)?);
        } else {
            self.build = Some(checked_working_dir()?.join("build"));
        }
        Ok(())
    }

    pub fn set_opensim_core(&mut self, opensim_core: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = opensim_core {
            self.opensim_core = Some(fs::canonicalize(dir)?);
        } else {
            self.opensim_core = Some(checked_working_dir()?.join("software").join("opensim-core"));
        }
        Ok(())
    }

    pub fn set_tests(&mut self, tests: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = tests {
            self.tests = Some(fs::canonicalize(dir)?);
        } else {
            self.tests = Some(checked_working_dir()?.join("tests"));
        }
        Ok(())
    }

    pub fn tests(&self) -> &PathBuf {
        self.tests
            .as_ref()
            .expect("tests directory was not set")
    }

    pub fn set_models(&mut self, models: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = models {
            self.models = Some(fs::canonicalize(dir)?);
        } else {
            self.models = Some(checked_working_dir()?.join("tests/models"));
        }
        Ok(())
    }

    pub fn models(&self) -> &PathBuf {
        self.models
            .as_ref()
            .expect("models directory was not set")
    }

    pub fn set_results(&mut self, results: Option<PathBuf>) -> Result<()> {
        if let Some(dir) = results {
            self.results = Some(fs::canonicalize(dir)?);
        } else {
            self.results = Some(checked_working_dir()?.join("results"));
        }
        Ok(())
    }

    pub fn results(&self) -> &PathBuf {
        self.results
            .as_ref()
            .expect("results directory was not set")
    }

    pub fn archive(&self) -> &PathBuf {
        self.archive
            .as_ref()
            .expect("archive directory was not set")
    }

    pub fn opensim_install_dir<'a>(&self, id: InstallId<'a>) -> PathBuf {
        self.archive().join(id.subfolder_name())
    }

    pub fn opensim_build_dir(&self) -> &PathBuf {
        self.build.as_ref().expect("build directory was not set")
    }

    pub fn opensim_core(&self) -> &PathBuf {
        self.opensim_core
            .as_ref()
            .expect("path to opensim-core was not set")
    }
}
