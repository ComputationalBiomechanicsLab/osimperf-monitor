use std::path::PathBuf;

use crate::Command;

pub struct CMakeConfigurerer<I> {
    pub source: PathBuf,
    pub build: PathBuf,
    pub install: PathBuf,
    pub dependency: Option<PathBuf>,
    pub args: I,
}

impl<I, S> CMakeConfigurerer<I>
where
    I: Iterator<Item = S>,
    S: ToString,
{
    pub fn into_cmd(self) -> Command {
        // Cmake configuration step.
        let mut cmd = Command::new("cmake");
        cmd.add_arg("-B");
        cmd.add_arg(self.build.to_str().unwrap());
        cmd.add_arg("-S");
        cmd.add_arg(self.source.to_str().unwrap());
        if let Some(dir) = self.dependency.as_ref() {
            cmd.add_arg(format!("-DCMAKE_PREFIX_PATH={}", dir.to_str().unwrap()));
        }
        cmd.add_arg(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            self.install.to_str().unwrap()
        ));
        cmd.add_args(self.args);
        cmd
    }
}
