pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";
pub static BIO_LAB_URL: &str = "git@github.com:ComputationalBiomechanicsLab/opensim-core.git";

pub mod bench_tests;
pub mod common;

mod command;
mod folders;
mod node;

pub use command::{
    Command, CommandExecutor, CommandExecutorTrait, CommandOutput, CommandTrait, PipedCommands,
    PipedCommandsExecutor,
};
pub use folders::*;
pub use node::*;

pub use common::git;
