pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";
pub static BIO_LAB_URL: &str = "git@github.com:ComputationalBiomechanicsLab/osimperf-monitor.git";

mod command;
mod config;
mod folders;
mod node;

pub mod git;

pub use command::*;
pub use config::*;
pub use folders::*;
pub use node::*;

