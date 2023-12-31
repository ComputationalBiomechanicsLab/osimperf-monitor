use std::{
    fs::{self, remove_dir_all, rename, OpenOptions},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::trace;

pub static ARCHIVE_TOUCH_FILE: &str = ".osimperf-archive";
pub static RESULTS_TOUCH_FILE: &str = ".osimperf-results";

pub trait Folder: Sized {
    const TOUCH_FILE: &'static str;

    unsafe fn path_unchecked(&self) -> &Path;

    unsafe fn new_unchecked(path: PathBuf) -> Self;

    fn path(&self) -> Result<&Path> {
        // TODO this always returns OK...
        Ok(unsafe { self.path_unchecked() })
    }

    fn path_str(&self) -> Result<&str> {
        Ok(self.path()?.to_str().unwrap())
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
    let temp_dir = path.parent().unwrap().join("osimperf-temporary");
    if path.exists() {
        rename(&path, &temp_dir).context("unable to move install dir to temporary")?;
        trace!("erase_folder fn: moving {:?} to {:?}", &path, &temp_dir);
        trace!("erase_folder fn: remove {:?}", &temp_dir);
        remove_dir_all(&temp_dir).with_context(|| format!("Failed to remove {:?}", temp_dir))?;
    }
    trace!(
        "erase_folder fn: Create new empty folder at: {:?}",
        &temp_dir
    );
    fs::create_dir(&temp_dir)
        .with_context(|| format!("Failed to create directory: {:?}", temp_dir))?;
    trace!("erase_folder fn: move empty folder to {:?}", path);
    rename(&temp_dir, &path).context("unable to move temporary to path")?;
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

impl Home {
    /// Construct from path if Some, or use current directory if None.
    pub fn new_or_current(path: Option<&str>) -> Result<Self> {
        if let Some(p) = path {
            Self::new(p)
        } else {
            let p = std::env::current_dir()?;
            Self::new(p.to_str().unwrap())
        }
    }

    pub fn default_build(&self) -> Result<BuildFolder> {
        BuildFolder::new(self.path.join("build").to_str().unwrap())
    }

    pub fn default_archive(&self) -> Result<Archive> {
        Archive::new(self.path.join("archive").to_str().unwrap())
    }

    pub fn default_results(&self) -> Result<ResultsFolder> {
        ResultsFolder::new(self.path.join("results").to_str().unwrap())
    }
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

impl ResultsFolder {
    pub fn test_context_dir(&self) -> Result<PathBuf> {
        Ok(self.path()?.join("scratch"))
    }
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
