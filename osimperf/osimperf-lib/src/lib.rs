pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";
pub static BIO_LAB_URL: &str = "git@github.com:ComputationalBiomechanicsLab/osimperf-monitor.git";

mod command;
mod common;
mod folders;
mod node;
pub mod bench_tests;

pub use command::*;
pub use common::*;
pub use folders::*;
pub use node::*;

pub use common::git;
