use anyhow::{Context, Result};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str,
};

use crate::{Command, CommandTrait, Commit};

// Expexted to be in OSIMPERF_HOME directory.
pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

pub struct ProgressStreamer {}

impl Write for ProgressStreamer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > 0 {
            let string = str::from_utf8(buf).unwrap();
            println!("{}", string);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct CmakeDirs {
    pub source: PathBuf,
    pub build: PathBuf,
    pub install: PathBuf,
    pub dependency: Option<PathBuf>,
}

pub fn run_cmake_cmd<T: ToString>(
    cmake_dirs: &CmakeDirs,
    build_log: &Path,
    cmake_args: impl Iterator<Item = T>,
    num_jobs: usize,
    target: Option<String>,
    progress: &mut ProgressStreamer,
) -> Result<f64> {
    // Cmake configuration step.
    let mut cmake_confgure_cmd = Command::new("cmake");
    cmake_confgure_cmd.add_arg("-B");
    cmake_confgure_cmd.add_arg(cmake_dirs.build.to_str().unwrap());
    cmake_confgure_cmd.add_arg("-S");
    cmake_confgure_cmd.add_arg(cmake_dirs.source.to_str().unwrap());
    if let Some(dir) = cmake_dirs.dependency.as_ref() {
        cmake_confgure_cmd.add_arg(format!("-DCMAKE_PREFIX_PATH={}", dir.to_str().unwrap()));
    }
    cmake_confgure_cmd.add_arg(format!(
        "-DCMAKE_INSTALL_PREFIX={}",
        cmake_dirs.install.to_str().unwrap()
    ));
    cmake_confgure_cmd.add_args(cmake_args.map(|a| format!("-D{}", a.to_string())));
    debug!("cmake configure: {:#?}", cmake_confgure_cmd);
    let config_output = cmake_confgure_cmd
        .run_and_stream(progress)
        .context("failed to generate project configuration files")?;
    config_output.write_logs(build_log)?;

    // Cmake build step.
    let mut cmake_build_cmd = Command::new("cmake");
    cmake_build_cmd.add_arg("--build");
    cmake_build_cmd.add_arg(cmake_dirs.build.to_str().unwrap());
    if let Some(t) = target {
        cmake_build_cmd.add_arg("--target");
        cmake_build_cmd.add_arg(t);
    }
    cmake_build_cmd.add_arg(format!("-j{}", num_jobs));
    debug!("cmake build: {:#?}", cmake_build_cmd);
    let build_output = cmake_build_cmd
        .run_and_stream(progress)
        .context("failed to build project")?;
    build_output.write_logs(build_log)?;

    // TODO Test step.
    Ok(config_output.duration + build_output.duration)
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

pub struct CompileTimes {
    pub dependencies: f64,
    pub opensim_core: f64,
    pub tests_source: f64,
}

pub fn compile_opensim_core(
    folders: &Folders,
    commit: &Commit,
    config: &OSimCoreCmakeConfig,
) -> Result<CompileTimes> {
    let install = commit
        .get_archive_folder(folders)
        .join(Path::new("install"));
    debug!("Set archive to {:?}", &install);
    debug!("Start compilation of OpenSim dependencies.");

    let mut stdout_log = File::open(install.join("stdout.log"))?;
    let mut stderr_log = File::open(install.join("stderr.log"))?;
    let mut stream = std::io::stdout().lock();

    // Compile dependencies.
    let duration_deps = run_cmake_cmd(
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
        config.num_jobs,
        "",
        &mut stdout_log,
        &mut stderr_log,
        &mut stream,
    )
    .context("failed to compile opensim dependencies")?;
    debug!(
        "Opensim dependencies compilation completed in {} seconds",
        duration_deps
    );

    // Compile opensim-core.
    debug!("Start compilation of OpenSim-core.");
    let deps_arg = [format!(
        "OPENSIM_DEPENDENCIES_DIR={}",
        install.to_str().unwrap()
    )];
    let duration_opensim = run_cmake_cmd(
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
        config.num_jobs,
        "install",
        &mut stdout_log,
        &mut stderr_log,
        &mut stream,
    )
    .context("failed to compile opensim core")?;
    debug!(
        "Opensim-core compilation completed in {} seconds",
        duration_opensim
    );

    // Compile bench tests from source.
    debug!("Start compilation of tests from source.");
    let duration_tests = run_cmake_cmd(
        &CmakeInstallDirs {
            source: folders.source.clone(),
            build: folders.get_tests_build(),
            dependencies_install: Some(install.clone()),
            install,
        },
        config.common.iter(),
        config.num_jobs,
        "install",
        stdout_log,
        stderr_log,
        stream,
    )
    .context("failed to compile benchmark tests from source")?;
    debug!(
        "Tests from source compilation completed in {} seconds",
        duration_tests
    );

    Ok(CompileTimes {
        dependencies: duration_deps,
        opensim_core: duration_opensim,
        tests: duration_tests,
    })
}
