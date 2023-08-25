use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub static ARCHIVE_TOUCH_FILE: &str = ".osimperf-archive";
pub static RESULTS_TOUCH_FILE: &str = ".osimperf-results";

trait Folder: Sized {
    const TOUCH_FILE: &'static str;

    fn path(&self) -> &Path;

    fn new_unchecked(path: PathBuf) -> Self;

    fn new(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        let touch = path.join(Self::TOUCH_FILE);
        Some(Self::new_unchecked(path.clone()))
            .filter(|_| touch.exists())
            .with_context(|| format!("{:?} doesnt look like the correct directory", path))
    }
}

#[derive(Clone, Debug)]
struct Home {
    path: PathBuf,
}

impl Folder for Home {
    const TOUCH_FILE: &'static str = ".osimperf-home";

    fn path(&self) -> &Path {
        &self.path
    }

    fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

#[derive(Clone, Debug)]
struct Archive {
    path: PathBuf,
}

impl Folder for Archive {
    const TOUCH_FILE: &'static str = ".osimperf-archive";

    fn path(&self) -> &Path {
        &self.path
    }

    fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

#[derive(Clone, Debug)]
struct ResultsFolder {
    path: PathBuf,
}

impl Folder for ResultsFolder {
    const TOUCH_FILE: &'static str = ".osimperf-results";

    fn path(&self) -> &Path {
        &self.path
    }

    fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}
