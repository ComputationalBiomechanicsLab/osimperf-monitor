use anyhow::{anyhow, ensure, Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::PathBuf,
    str,
};

use crate::{
    erase_folder, Archive, BuildFolder, Command, CommandTrait, Folder, Home, PipedCommands,
    Repository,
};

// Expexted to be in OSIMPERF_HOME directory.
pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

#[derive(Clone, Debug, Default)]
pub struct ProgressStreamer {
    process_name: String,
    process_step: String,
    buffer: String,
    percentage: Option<f64>,
}

impl ProgressStreamer {
    fn set_process_name(&mut self, name: &str) {
        self.percentage = None;
        self.process_name = name.to_string();
    }

    fn set_process_step(&mut self, name: &str) {
        self.percentage = None;
        self.process_step = name.to_string();
        // println!("Start {} {}", self.process_step, self.process_name);
    }

    fn pop_line(&mut self) -> Result<()> {
        // Check if a complete line is present in the buffer
        if self.buffer.contains('\n') {
            // Split the buffer into lines and process each complete line
            let lines: Vec<&str> = self.buffer.split('\n').collect();
            let num_lines = lines.len();

            // Print and remove all complete lines except the last one (if it's incomplete)
            for line in lines.iter().take(num_lines - 1) {
                let mut cmd_echo = Command::parse("echo");
                cmd_echo.add_arg(format!(r#""{}""#, line));

                let mut cmd_grep = Command::new("grep");
                cmd_grep.add_arg("-o");
                cmd_grep.add_arg("\\[ [0-9]*%");

                let mut cmd_sed = Command::new("sed");
                cmd_sed.add_arg("s/[^0-9]//g");

                let cmd = PipedCommands::new(vec![cmd_echo, cmd_grep, cmd_sed]);
                if let Some(perc_str) = Some(cmd.run_trim()?).filter(|s| s.len() > 0) {
                    let parsed_percentage = perc_str
                        .parse::<f64>()
                        .with_context(|| format!("failed to parse percentage {perc_str}"))?;
                    if self.percentage.is_some() {
                        print!("\r");
                    }
                    self.percentage = Some(parsed_percentage);
                    print!(
                        "{} {}: {}% -- {}",
                        self.process_step, self.process_name, perc_str, line
                    );
                    io::stdout().flush().context("Failed to flush stdout")?; // Flush the buffer
                }
                // println!(
                //     "{} {}: {:?}% -- {}",
                //     self.process_step, self.process_name, self.percentage, line
                // );
            }

            // Keep the last incomplete line in the buffer
            self.buffer = lines[num_lines - 1].to_string();
        }
        Ok(())
    }
}

impl Write for ProgressStreamer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > 0 {
            self.buffer.push_str(str::from_utf8(buf).unwrap());
        }
        self.pop_line().map_err(|_| std::io::ErrorKind::NotFound)?; // TODO different error kind.
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.pop_line().map_err(|_| std::io::ErrorKind::NotFound)?; // TODO different error kind.
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
    cmake_args: impl Iterator<Item = T>,
    num_jobs: usize,
    target: Option<&str>,
    log: &mut impl Write,
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
    progress.set_process_step("configuring");
    let config_output = cmake_confgure_cmd
        .run_and_stream(progress)
        .context("failed to generate project configuration files")?;
    config_output.write_logs(log)?;
    if !config_output.success() {
        Err(anyhow!(format!(
            "command = {}",
            cmake_confgure_cmd.print_command_with_delim(" \\\n")
        )))
        .context("cmake configuration step failed")?;
    }

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
    progress.set_process_step("building");
    let build_output = cmake_build_cmd
        .run_and_stream(progress)
        .context("failed to build project")?;
    build_output.write_logs(log)?;

    if !build_output.success() {
        Err(anyhow!(format!(
            "configure-cmd = {}",
            cmake_confgure_cmd.print_command_with_delim(" \\\n")
        )))
        .with_context(|| {
            format!(
                "build-cmd = {}",
                cmake_build_cmd.print_command_with_delim(" \\\n")
            )
        })
        .context("cmake build step failed")?;
    }
    ensure!(build_output.success(), "cmake build step failed");

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
            num_jobs: 4,
        }
    }
}

pub struct CompileTimes {
    pub dependencies: f64,
    pub opensim_core: f64,
    pub tests_source: f64,
}

pub fn compile_opensim_core(
    repo: &Repository,
    home: &Home,
    archive: &Archive,
    build: &BuildFolder,
    config: &OSimCoreCmakeConfig,
) -> Result<CompileTimes> {
    let install = repo
        .install_folder(archive)
        .context("install folder error")?;
    info!("Set archive to {:?}", &install);
    info!("Start compilation of OpenSim dependencies.");
    erase_folder(&install)?;

    let install_opensim_core = install.join("opensim-core");
    let install_dependencies = install.join("dependencies");
    let install_tests_source = install.join("tests_source");

    let mut deps_log = OpenOptions::new()
        .write(true)
        .create(true)
        .open(install.join("simbody-build.log"))
        .with_context(|| format!("failed to create dependencies log at {:?}", install))?;

    let mut stream = ProgressStreamer::default();

    // Compile dependencies.
    stream.set_process_name("dependencies");
    let duration_deps = run_cmake_cmd(
        &&CmakeDirs {
            source: repo.path()?.join("dependencies"),
            build: build.path()?.join("opensim-core-dependencies"),
            install: install_dependencies.clone(),
            dependency: None,
        },
        config
            .common
            .iter()
            .chain(config.dependencies.iter())
            .chain(config.common_opensim.iter()),
        config.num_jobs,
        None,
        &mut deps_log,
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
        install_dependencies.to_str().unwrap()
    )];
    let mut opensim_log = OpenOptions::new()
        .write(true)
        .create(true)
        .open(install.join("opensim-build.log"))
        .with_context(|| format!("failed to create opensim log at {:?}", install))?;
    let target = Some("install");
    stream.set_process_name("opensim core");
    let duration_opensim = run_cmake_cmd(
        &CmakeDirs {
            source: repo.path()?.to_path_buf(),
            build: build.path()?.join("opensim-core"),
            dependency: Some(install_dependencies.clone()),
            install: install_opensim_core.clone(),
        },
        config
            .common
            .iter()
            .chain(config.opensim.iter())
            .chain(config.common_opensim.iter())
            .chain(deps_arg.iter()),
        config.num_jobs,
        target,
        &mut opensim_log,
        &mut stream,
    )
    .context("failed to compile opensim core")?;
    debug!(
        "Opensim-core compilation completed in {} seconds",
        duration_opensim
    );

    // Compile bench tests from source.
    debug!("Start compilation of tests from source.");
    let mut source_log = OpenOptions::new()
        .write(true)
        .create(true)
        .open(install.join("tests-build.log"))
        .with_context(|| format!("failed to create tests log at {:?}", install))?;
    stream.set_process_name("tests source");
    let instal_prefix = format!(
        "CMAKE_INSTALL_PREFIX={}",
        install_opensim_core.to_str().unwrap(),
    );
    let mut duration_tests = f64::NAN;
    if (false) {
        duration_tests = run_cmake_cmd(
            &CmakeDirs {
                source: home.path()?.join("source"),
                build: build.path()?.join("test-source"),
                install: install_tests_source,
                dependency: None,
            },
            config
                .common
                .iter()
                .chain(&[instal_prefix])
                .chain(config.common_opensim.iter()),
            config.num_jobs,
            target,
            &mut source_log,
            &mut stream,
        )
        .context("failed to compile benchmark tests from source")?;
        debug!(
            "Tests from source compilation completed in {} seconds",
            duration_tests
        );
    } else {
        warn!("Manually skipping building tests from source");
    }

    Ok(CompileTimes {
        dependencies: duration_deps,
        opensim_core: duration_opensim,
        tests_source: duration_tests,
    })
}
