use crate::{Args, Command};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Folders {
    /// Home of osimperf service.
    pub home: PathBuf,
    /// Location of helper scripts.
    pub scripts: PathBuf,
    /// Location of all aggregates.
    ///
    /// Default layout:
    /// osimperf_home/archive/.osimperf-archive.conf
    ///                      /opensim-core-DATE0-HASH0/install/...
    ///                      /opensim-core-DATE1-HASH1/install/...
    pub archive: PathBuf,
    /// Location of source code to be compiled next to opensim_core.
    pub source: PathBuf,
    /// Location of opensim-core source code repository.
    pub opensim_core: PathBuf,
    /// Folder for all temporary files (build files, logs, etc.).
    ///
    /// Default layout:
    /// osimperf_home/throwaway/.osimperf-throwaway.conf
    ///                        /build/opensim-core
    ///                              /opensim-core-dependencies
    ///                              /tests
    pub throwaway: PathBuf,
    /// Folder with all bench tests definitions.
    ///
    /// Subfolders of tests/ are expected to be testcases. Simply put a file `osimperf-test.conf`
    /// in each folder that defines a test.
    ///
    /// Default layout:
    /// osimperf_home/tests/Hopper/osimperf-test.conf
    ///                           /Hopper_model.osim
    ///                           /Hopper_setup.osim
    ///                     InverseKinematics/osimperf-test.conf
    ///                                      /IK_model.osim
    ///                                      /IK_setup.osim
    pub tests: PathBuf,
    /// Folder for collecting bench test results.
    ///
    /// Default layout:
    /// osimperf_home/results/.osimperf-results.conf
    ///                      /results-DATE-HASH/Hopper/osimperf-results.data
    ///                                        /InverseKinematics/osimperf-resuls.data
    pub results: PathBuf,
}

impl Folders {
    pub fn new(args: &Args) -> Result<Self> {
        let home = if let Some(h) = args.home.as_ref() {
            PathBuf::from(h)
        } else {
            get_current_dir()?
        };
        println!("Perf home = {:?}", home);
        Ok(Self {
            scripts: parse_folder_arg(&home, &args.scripts)?,
            archive: parse_folder_arg(&home, &args.archive)?,
            source: parse_folder_arg(&home, &args.source)?,
            opensim_core: parse_folder_arg(&home, &args.opensim_core)?,
            throwaway: parse_folder_arg(&home, &args.throwaway)?,
            tests: parse_folder_arg(&home, &args.tests)?,
            results: parse_folder_arg(&home, &args.results)?,
            home,
        })
    }

    pub fn get_opensim_dependencies_source(&self) -> PathBuf {
        self.opensim_core.join("dependencies")
    }

    pub fn get_opensim_dependencies_build(&self) -> PathBuf {
        self.get_build_dir().join("opensim-core-dependencies")
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.throwaway.join(Path::new("build"))
    }

    pub fn get_opensim_core_build_dir(&self) -> PathBuf {
        self.get_build_dir().join("opensim_core")
    }

    pub fn get_tests_build(&self) -> PathBuf {
        self.get_build_dir().join(Path::new("tests"))
    }
}

fn get_current_dir() -> Result<PathBuf> {
    println!("Using current directory as home");
    Ok(PathBuf::from(Command::new("pwd").run()?))
}

fn parse_folder_arg(home: &PathBuf, folder: &String) -> Result<PathBuf> {
    let ch = folder.chars().next().context("empty string for folder")?;
    let path = if ch != '/' {
        home.join(Path::new(&folder))
    } else {
        PathBuf::from(folder)
    };
    Ok(path)
}
