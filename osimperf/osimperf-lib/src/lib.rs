pub static CMAKE_CONFIG_FILE: &str = ".osimperf-cmake.conf";

// mod cmake;
mod command;
mod repo;
mod folders;
mod config;
mod cmake;

pub use command::*;
pub use repo::*;
pub use config::*;
pub use cmake::*;
pub use folders::*;
