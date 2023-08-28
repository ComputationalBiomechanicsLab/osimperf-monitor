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
    BuildFolder, Command, CommandTrait, Folder,
};

pub struct CMakeCmds {
    configure: Command,
    build: Command,
}

impl CMakeCmds {
    pub fn new<'a>(
        id: &Id<'a>,
        source: &Source<'a>,
        build: &BuildFolder,
        config: &CMakeConfig,
        focus: Focus,
    ) -> anyhow::Result<Self> {
        let dependency = match focus {
            Focus::OpenCimCore => Some(id.path().join("dependencies")),
            Focus::Dependencies => None,
            Focus::TestsSource => Some(id.path().join("opensim-core")),
        };

        let focus_str = focus.to_str();

        let source = match focus {
            Focus::OpenCimCore => source.path()?.to_owned(),
            Focus::Dependencies => source.path()?.join("dependencies"),
            Focus::TestsSource => return Err(anyhow!("not yet tests source implemented")),
        };

        let target = match focus {
            Focus::OpenCimCore => Some("install"),
            Focus::Dependencies => None,
            Focus::TestsSource => return Err(anyhow!("not yet tests source implemented")),
        };

        let mut args = config.cmake_args(focus);
        if let Focus::OpenCimCore = focus {
            let add_arg = format!(
                "-DOPENSIM_DEPENDENCIES_DIR={}",
                dependency.clone().unwrap().to_str().unwrap()
            );
            args.push(add_arg);
        }

        Ok(Self {
            configure: CMakeConfigurerer {
                source: source,
                build: build.path()?.join(focus_str),
                install: id.path().join(focus_str),
                args: args.iter(),
                dependency,
            }
            .into_cmd(),
            build: CMakeBuilder {
                build: build.path()?.join(focus_str),
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
