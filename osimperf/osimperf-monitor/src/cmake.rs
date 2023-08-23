use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str,
};

use crate::{Command, Commit, Folders};

// Expexted to be in OSIMPERF_HOME directory.
pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

pub struct CmakeInstallDirs {
    pub source: PathBuf,
    pub build: PathBuf,
    pub install: PathBuf,
    pub dependencies_install: Option<PathBuf>, // Skipped if empty.
}

pub fn run_cmake_cmd<T: ToString>(
    cmake_dirs: &CmakeInstallDirs,
    cmake_args: impl Iterator<Item = T>,
    log: &mut String,
    num_jobs: usize,
    target: &str,
) -> Result<()> {
    // Cmake configuration step.
    let mut cmake_confgure_cmd = Command::new("cmake");
    cmake_confgure_cmd.add_arg("-B");
    cmake_confgure_cmd.add_arg(cmake_dirs.build.to_str().unwrap());
    cmake_confgure_cmd.add_arg("-S");
    cmake_confgure_cmd.add_arg(cmake_dirs.source.to_str().unwrap());
    if let Some(dir) = cmake_dirs.dependencies_install.as_ref() {
        cmake_confgure_cmd.add_arg(format!("-DCMAKE_PREFIX_PATH={}", dir.to_str().unwrap()));
    }
    cmake_confgure_cmd.add_arg(format!(
        "-DCMAKE_INSTALL_PREFIX={}",
        cmake_dirs.install.to_str().unwrap()
    ));
    // cmake_confgure_cmd.add_arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON");
    cmake_confgure_cmd.add_args(cmake_args.map(|a| format!("-D{}", a.to_string())));
    println!("cmake configure: {:#?}", cmake_confgure_cmd);
    cmake_confgure_cmd
        .run_print(log, true)
        .context("cmake failed")
        .context("failed to generate project configuration files")?;

    // Cmake build step.
    let mut cmake_build_cmd = Command::new("cmake");
    cmake_build_cmd.add_arg("--build");
    cmake_build_cmd.add_arg(cmake_dirs.build.to_str().unwrap());
    if target.len() > 0 {
        cmake_build_cmd.add_arg("--target");
        cmake_build_cmd.add_arg(target);
    }
    cmake_build_cmd.add_arg(format!("-j{}", num_jobs));
    println!("cmake build: {:#?}", cmake_build_cmd);
    cmake_build_cmd
        .run_print(log, true)
        .context("cmake failed")
        .context("failed to build project")?;

    // TODO Test step.
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OSimCoreCmakeConfig {
    common: Vec<String>,
    common_opensim: Vec<String>,
    opensim: Vec<String>,
    dependencies: Vec<String>,
    num_jobs: usize,
}

impl Default for OSimCoreCmakeConfig {
    fn default() -> Self {
        Self {
            common: vec!["CMAKE_BUILD_TYPE=RelWithDebInfo".to_string()],
            common_opensim: vec![
                "OPENSIM_WITH_CASADI=OFF".to_string(),
                "OPENSIM_WITH_TROPTER=OFF".to_string(),
            ],
            opensim: vec![
                "OPENSIM_BUILD_INDIVIDUAL_APPS=OFF".to_string(),
                "OPENSIM_INSTALL_UNIX_FHS=ON".to_string(),
                "BUILD_API_ONLY=OFF".to_string(),
                "BUILD_API_EXAMPLES=OFF".to_string(),
                "BUILD_JAVA_WRAPPING=OFF".to_string(),
                "BUILD_PYTHON_WRAPPING=OFF".to_string(),
                "BUILD_TESTING=ON".to_string(),
            ],
            dependencies: Vec::new(),
            num_jobs: 1,
        }
    }
}

pub fn compile_opensim_core(
    folders: &Folders,
    commit: &Commit,
    config: &OSimCoreCmakeConfig,
    log: &mut String,
) -> Result<()> {
    let install = commit
        .get_archive_folder(folders)
        .join(Path::new("install"));
    // Compile dependencies.
    run_cmake_cmd(
        &CmakeInstallDirs {
            source: folders.get_opensim_dependencies_source(),
            build: folders.get_opensim_dependencies_build(),
            install: install.clone(),
            dependencies_install: None,
        },
        config
            .common
            .iter()
            .chain(config.dependencies.iter())
            .chain(config.common_opensim.iter()),
        log,
        config.num_jobs,
        "",
    )
    .context("failed to compile opensim dependencies")?;

    // Compile opensim-core.
    let deps_arg = [format!(
        "OPENSIM_DEPENDENCIES_DIR={}",
        install.to_str().unwrap()
    )];
    run_cmake_cmd(
        &CmakeInstallDirs {
            source: folders.opensim_core.clone(),
            build: folders.get_opensim_core_build_dir(),
            dependencies_install: Some(install.clone()),
            install: install.clone(),
        },
        config
            .common
            .iter()
            .chain(config.opensim.iter())
            .chain(config.common_opensim.iter())
            .chain(deps_arg.iter()),
        log,
        config.num_jobs,
        "install",
    )
    .context("failed to compile opensim core")?;

    // Compile bench tests from source.
    run_cmake_cmd(
        &CmakeInstallDirs {
            source: folders.source.clone(),
            build: folders.get_tests_build(),
            dependencies_install: Some(install.clone()),
            install,
        },
        config.common.iter(),
        log,
        config.num_jobs,
        "install",
    )
    .context("failed to compile benchmark tests from source")?;

    Ok(())
}
