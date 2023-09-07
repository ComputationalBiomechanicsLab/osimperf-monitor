mod build;
mod configure;

pub use build::CMakeBuilder;
pub use configure::CMakeConfigurerer;

use anyhow::{anyhow, ensure, Context};
use std::{io::Write, time::Duration};

use super::CMakeConfig;
use crate::node::{CompilationTarget, Id};
use crate::{
    path_to_build, path_to_install, path_to_source, BuildFolder, Command, CommandTrait, Home, RepositoryState,
};

pub struct CMakeCmds {
    configure: Command,
    build: Command,
}

impl CMakeCmds {
    pub fn new<'a>(
        id: &Id<'a>,
        repo: &RepositoryState,
        home: &Home,
        build: &BuildFolder,
        config: &CMakeConfig,
        cmp_target: CompilationTarget,
    ) -> anyhow::Result<Self> {
        let dependency = match cmp_target {
            CompilationTarget::Dependencies => None,
            CompilationTarget::OpenSimCore => Some(path_to_install(CompilationTarget::Dependencies, id)),
            CompilationTarget::TestsSource => Some(path_to_install(CompilationTarget::OpenSimCore, id)),
        };

        let source = path_to_source(cmp_target, home, repo)?;

        let target = match cmp_target {
            CompilationTarget::Dependencies => None,
            CompilationTarget::OpenSimCore => Some("install"),
            CompilationTarget::TestsSource => Some("install"),
        };

        let mut args = config.cmake_args(cmp_target);
        if let CompilationTarget::OpenSimCore = cmp_target {
            let add_arg = format!(
                "-DOPENSIM_DEPENDENCIES_DIR={}",
                dependency.clone().unwrap().to_str().unwrap()
            );
            args.push(add_arg);
        }

        Ok(Self {
            configure: CMakeConfigurerer {
                source,
                build: path_to_build(cmp_target, build)?,
                install: path_to_install(cmp_target, id),
                args: args.iter(),
                dependency,
            }
            .into_cmd(),
            build: CMakeBuilder {
                build: path_to_build(cmp_target, build)?,
                target,
                num_jobs: config.num_jobs,
            }
            .into_cmd(),
        })
    }

    pub fn run(&self, log: &mut impl Write) -> anyhow::Result<Duration> {
        let config_output = self.configure.run_and_stream(log)?;
        config_output.write_logs(log)?;
        if !config_output.success() {
            Err(anyhow!("configuration step failed"))
                .with_context(|| format!("output = {:?}", config_output.stdout_str_clone()))
                .with_context(|| format!("stderr = {:?}", config_output.stderr_str_clone()))?
        }

        let build_output = self.build.run_and_stream(log)?;
        build_output.write_logs(log)?;
        ensure!(build_output.success(), "build step failed");
        if !build_output.success() {
            Err(anyhow!("build step failed"))
                .with_context(|| format!("output = {:?}", build_output.stdout_str_clone()))
                .with_context(|| format!("stderr = {:?}", build_output.stderr_str_clone()))?
        }
        Ok(config_output.duration + build_output.duration)
    }

    pub fn print_pretty(&self) -> String {
        format!(
            "configure command: {:#}\nbuild command: {}",
            self.configure.print_command_with_delim(" \\\n    "),
            self.build.print_command_with_delim(" \\\n    ")
        )
    }
}
