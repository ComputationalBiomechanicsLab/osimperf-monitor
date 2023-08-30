mod build;
mod configure;

use std::{io::Write, path::PathBuf, time::Duration};

use anyhow::{anyhow, ensure, Context};
pub use build::*;
pub use configure::*;
use log::info;

use super::CMakeConfig;
use crate::{
    node::{Focus, Id, Source},
    path_to_build, path_to_install, path_to_source, BuildFolder, Command, CommandTrait, Folder,
    Home,
};

pub struct CMakeCmds {
    configure: Command,
    build: Command,
}

impl CMakeCmds {
    pub fn new<'a>(
        id: &Id<'a>,
        source: &Source<'a>,
        home: &Home,
        build: &BuildFolder,
        config: &CMakeConfig,
        focus: Focus,
    ) -> anyhow::Result<Self> {
        let dependency = match focus {
            Focus::Dependencies => None,
            Focus::OpenSimCore => Some(path_to_install(Focus::Dependencies, id)),
            Focus::TestsSource => Some(path_to_install(Focus::OpenSimCore, id)),
        };

        let source = path_to_source(focus, home, source)?;

        let target = match focus {
            Focus::Dependencies => None,
            Focus::OpenSimCore => Some("install"),
            Focus::TestsSource => Some("install"),
        };

        let mut args = config.cmake_args(focus);
        if let Focus::OpenSimCore = focus {
            let add_arg = format!(
                "-DOPENSIM_DEPENDENCIES_DIR={}",
                dependency.clone().unwrap().to_str().unwrap()
            );
            args.push(add_arg);
        }

        Ok(Self {
            configure: CMakeConfigurerer {
                source,
                build: path_to_build(focus, build)?,
                install: path_to_install(focus, id),
                args: args.iter(),
                dependency,
            }
            .into_cmd(),
            build: CMakeBuilder {
                build: path_to_build(focus, build)?,
                target,
                num_jobs: config.num_jobs,
            }
            .into_cmd(),
        })
    }

    pub fn run(&self, log: &mut impl Write) -> anyhow::Result<Duration> {
        info!("start compilation!");
        println!("commands = {:#}", self.print_pretty());
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
        // if !config_output.success() {
        //     Err(anyhow!("configuration step failed"))
        //         .with_context(|| format!("output = {:?}", config_output.stdout_str_clone()))
        //         .with_context(|| format!("stderr = {:?}", config_output.stderr_str_clone()))?
        // }
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
