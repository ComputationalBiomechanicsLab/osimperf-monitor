use crate::common::{read_config, visit_dirs};
use crate::Folder;
use crate::{git::Date, Home};

use super::super::CompilationTarget;
use anyhow::{Context, Result};
use log::{info, debug};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct CMakeConfig {
    common: Vec<String>,
    opensim: Vec<String>,
    dependencies: Vec<String>,
    opensim_and_dependencies: Vec<String>,
    tests: Vec<String>,
    pub num_jobs: usize,
}

impl CMakeConfig {
    pub fn cmake_args(&self, target: CompilationTarget) -> Vec<String> {
        match target {
            CompilationTarget::OpenSimCore => self
                .common
                .iter()
                .chain(self.opensim_and_dependencies.iter())
                .chain(self.opensim.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            CompilationTarget::Dependencies => self
                .common
                .iter()
                .chain(self.opensim_and_dependencies.iter())
                .chain(self.dependencies.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            CompilationTarget::TestsSource => self
                .common
                .iter()
                .chain(self.tests.iter())
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        }
    }
}

impl Default for CMakeConfig {
    fn default() -> Self {
        Self {
            common: vec!["-DCMAKE_BUILD_TYPE=RelWithDebInfo".to_string()],
            opensim: vec![
                "-DOPENSIM_BUILD_INDIVIDUAL_APPS=OFF".to_string(),
                "-DOPENSIM_INSTALL_UNIX_FHS=ON".to_string(),
                "-DBUILD_API_ONLY=OFF".to_string(),
                "-DBUILD_API_EXAMPLES=OFF".to_string(),
                "-DBUILD_JAVA_WRAPPING=OFF".to_string(),
                "-DBUILD_PYTHON_WRAPPING=OFF".to_string(),
                "-DBUILD_TESTING=OFF".to_string(),
                "-DOPENSIM_DOXYGEN_USE_MATHJAX=OFF".to_string(),
            ],
            opensim_and_dependencies: vec![
                "-DOPENSIM_WITH_CASADI=OFF".to_string(),
                "-DOPENSIM_WITH_TROPTER=OFF".to_string(),
            ],
            num_jobs: 3,
            dependencies: vec![],
            tests: vec![],
        }
    }
}

#[derive(Debug)]
pub struct CMakeConfigReader {
    default_config: CMakeConfig,
    dated_configs: Vec<(Date, CMakeConfig)>,
}

/// The default file name containing the cmake compiler flags.
static DEFAULT_CMAKE_CONFIG_FILE_NAME: &'static str = "osimperf-cmake.conf";

static DATED_CMAKE_CONFIG_FILE_NAME_PREFIX: &'static str = "osimperf-cmake-before-";

/// The subfolder that will be searched for all cmake compile flags.
static DEFAULT_CMAKE_CONFIG_DIR: &'static str = "compile-flags";

impl CMakeConfigReader {
    fn read_default(home: &Home) -> Result<Self> {
        let dir = home
            .path()?
            .join(DEFAULT_CMAKE_CONFIG_DIR)
            .join(DEFAULT_CMAKE_CONFIG_FILE_NAME);
        Ok(Self {
            default_config: read_config(&dir)?,
            dated_configs: Vec::new(),
        })
    }

    pub fn read(home: &Home) -> Result<Self> {
        // Must read atleast the default cmake config file.
        let mut out = Self::read_default(home)?;

        // Proceed by checking if there are any config files with a date in the title.
        let dir = home.path()?.join(DEFAULT_CMAKE_CONFIG_DIR);

        let mut files = Vec::<PathBuf>::new();
        visit_dirs(&dir, &mut |entry| {
            files.push(entry.path());
        })?;

        for file in files.iter() {
            if let Some(config) = try_read_as_dated_cmake_config(file)? {
                out.dated_configs.push(config);
            }
        }
        out.dated_configs.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    }

    /// Returns the config that is dated to be after the given date, or the default config is there
    /// is none.
    pub fn get(&self, date: &Date) -> &CMakeConfig {
        for (config_date_stamp, config) in self.dated_configs.iter() {
            if date < config_date_stamp {
                return config;
            }
        }
        &self.default_config
    }
}

fn try_read_as_dated_cmake_config(path: &Path) -> Result<Option<(Date, CMakeConfig)>> {
    let file_name = if let Some(file_name) = path.file_name().map(|x| x.to_str()).flatten() {
        file_name
    } else {
        return Ok(None);
    };

    if !file_name.contains(DATED_CMAKE_CONFIG_FILE_NAME_PREFIX) {
        return Ok(None);
    }
    debug!("Found cmake config {}", file_name);
    let s = file_name
        .split_at(DATED_CMAKE_CONFIG_FILE_NAME_PREFIX.len())
        .1
        .split_at(10)
        .0;
    let date =
        Date::parse_from_str(s, "%Y_%m_%d").context("failed to parse date of cmake config file")?;
    let config = read_config(path).context("failed to read cmake config file")?;
    Ok(Some((date, config)))
}
