use std::path::PathBuf;

use crate::Command;

pub struct CMakeBuilder<S> {
    pub build: PathBuf,
    pub target: Option<S>,
    pub num_jobs: usize,
}

impl<S: ToString> CMakeBuilder<S> {
    pub fn into_cmd(self) -> Command {
        // Cmake build step.
        let mut cmd = Command::new("cmake");
        cmd.add_arg("--build");
        cmd.add_arg(self.build.to_str().unwrap());
        if let Some(t) = self.target {
            cmd.add_arg("--target");
            cmd.add_arg(t.to_string());
        }
        cmd.add_arg(format!("-j{}", self.num_jobs));
        cmd
    }
}
