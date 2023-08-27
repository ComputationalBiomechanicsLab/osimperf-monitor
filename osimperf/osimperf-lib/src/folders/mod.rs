use std::{
    fs::{self, remove_dir_all, OpenOptions},
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context, Result};
use log::warn;

pub static ARCHIVE_TOUCH_FILE: &str = ".osimperf-archive";
pub static RESULTS_TOUCH_FILE: &str = ".osimperf-results";

pub trait Folder: Sized {
    const TOUCH_FILE: &'static str;

    unsafe fn path_unchecked(&self) -> &Path;

    unsafe fn new_unchecked(path: PathBuf) -> Self;

    fn path(&self) -> Result<&Path> {
        self.verify()?;
        Ok(unsafe { self.path_unchecked() })
    }

    fn new(path: &str) -> Result<Self> {
        let out = unsafe { Self::new_unchecked(PathBuf::from(path)) };
        out.verify()?;
        Ok(out)
    }

    fn magic_file(&self) -> PathBuf {
        let path = unsafe { PathBuf::from(self.path_unchecked()) };
        path.join(Self::TOUCH_FILE)
    }

    fn verify(&self) -> Result<()> {
        Some(())
            .filter(|_| self.magic_file().exists())
            .with_context(|| format!("unable to find magic file {:?}", self.magic_file()))
            .with_context(|| {
                format!("{:?} doesnt look like the correct directory", unsafe {
                    self.path_unchecked()
                })
            })
    }
}

// A simple implementation of `% touch path` (ignores existing files)
unsafe fn create_magic_file(folder: &impl Folder) -> Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(folder.magic_file())
        .with_context(|| format!("failed to create magic file at {:?}", folder.magic_file()))?;
    Ok(())
}

pub fn erase_folder(path: &Path) -> Result<()> {
    if path.exists() {
        warn!("removing directory {:?}", path);
        remove_dir_all(&path).context(format!("Failed to remove {:?}", path))?;
    }
    fs::create_dir(&path).with_context(|| format!("Failed to create directory: {:?}", path))?;
    Ok(())
}

pub trait EraseableFolder: Folder {
    fn erase_folder(&self) -> Result<()> {
        let dir = PathBuf::from(self.path()?);
        remove_dir_all(&dir).context(format!("Failed to remove directory: {:?}", dir))?;
        fs::create_dir(&dir).with_context(|| format!("Failed to create directory: {:?}", dir))?;
        unsafe { create_magic_file(self)? };
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Home {
    path: PathBuf,
}

impl Folder for Home {
    const TOUCH_FILE: &'static str = ".osimperf-home";

    unsafe fn path_unchecked(&self) -> &Path {
        &self.path
    }

    unsafe fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

/// This is where all the installs are stored.
#[derive(Clone, Debug)]
pub struct BuildFolder {
    path: PathBuf,
}

impl Folder for BuildFolder {
    const TOUCH_FILE: &'static str = ".osimperf-build";

    unsafe fn path_unchecked(&self) -> &Path {
        &self.path
    }

    unsafe fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

impl EraseableFolder for BuildFolder {}

/// This is where all the installs are stored.
#[derive(Clone, Debug)]
pub struct Archive {
    path: PathBuf,
}

impl Folder for Archive {
    const TOUCH_FILE: &'static str = ".osimperf-archive";

    unsafe fn path_unchecked(&self) -> &Path {
        &self.path
    }

    unsafe fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

impl EraseableFolder for Archive {}

#[derive(Clone, Debug)]
pub struct ResultsFolder {
    path: PathBuf,
}

impl Folder for ResultsFolder {
    const TOUCH_FILE: &'static str = ".osimperf-results";

    unsafe fn path_unchecked(&self) -> &Path {
        &self.path
    }

    unsafe fn new_unchecked(path: PathBuf) -> Self {
        Self { path }
    }
}

impl EraseableFolder for ResultsFolder {}
