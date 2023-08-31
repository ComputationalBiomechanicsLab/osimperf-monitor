use std::{
    fs::{self, remove_dir_all, rename, OpenOptions},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::{trace, warn};

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
    let temp_dir = path.parent().unwrap().join("osimperf-temporary");
    if path.exists() {
        warn!("Removing directory {:?}", &path);
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
        warn!("Erasing folder {}", dir.to_str().unwrap());
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

impl Default for Home {
    fn default() -> Self {
        Self {
            path: std::env::current_dir().unwrap(),
        }
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

// impl BuildFolder {
//     pub fn join(&self, focus: Focus) -> Result<PathBuf> {
//         Ok(self.path()?.join(match focus {
//             Focus::OpenCimCore => "opensim-core",
//             Focus::Dependencies => "dependencies",
//             Focus::TestsSource => "tests",
//         }))
//     }
// }

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
