use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::time::duration_since_boot;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct CommandOutput {
    pub duration: f64,
    pub output: std::process::Output,
}

impl CommandOutput {
    pub fn stdout_str_clone(&self) -> String {
        String::from_utf8(self.output.stdout.clone()).unwrap()
    }

    pub fn stderr_str_clone(&self) -> String {
        String::from_utf8(self.output.stdout.clone()).unwrap()
    }

    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    pub fn write_stdout(&self, path: &Path) -> Result<()> {
        File::open(path)
            .context(format!(
                "failed to open file for writing stdout logs at path = {:?}",
                path
            ))?
            .write_all(&self.output.stdout)?;
        Ok(())
    }

    pub fn write_stderr(&self, path: &Path) -> Result<()> {
        File::open(path)
            .context(format!(
                "failed to open file for writing stderr logs at path = {:?}",
                path
            ))?
            .write_all(&self.output.stderr)?;
        Ok(())
    }

    pub fn write_logs(&self, path: &Path) -> Result<()> {
        let mut file = File::open(path).context(format!(
            "failed to open file for writing stderr logs at path = {:?}",
            path
        ))?;
        file.write_all(&self.output.stderr)?;
        file.write_all(&self.output.stdout)?;
        file.write_all(format!("{:#?}", self.output.status).as_bytes())?;
        Ok(())
    }
}

pub trait CommandTrait {
    type Executor: CommandExecutorTrait + Debug;

    fn create_executor(&self) -> Self::Executor;

    fn run(&self) -> Result<CommandOutput> {
        let cmd = self.create_executor();
        let dbg_msg = format!("failed to execute command: {:#?}", &cmd);
        let start = duration_since_boot()?;
        let output = cmd.execute();
        let end = duration_since_boot()?;
        let duration = (end - start).as_secs_f64();
        Ok(CommandOutput {
            duration,
            output: output.context(dbg_msg)?,
        })
    }

    fn run_and_stream(&self, mut stream: impl Write) -> Result<CommandOutput> {
        let cmd = self.create_executor();
        let dbg_msg = format!("failed to execute command: {:#?}", &cmd);
        let start = duration_since_boot()?;
        let mut child = cmd.start_execute()?;

        let stdout = child.stdout.take().unwrap();
        let lines = BufReader::new(stdout).lines();
        for line in lines {
            let line = line?;
            stream.write_all(line.as_bytes())?;
            if child
                .try_wait()
                .context("error attempting to wait for command.")
                .context(dbg_msg.clone())?
                .is_some()
            {
                break;
            }
        }

        let output = child
            .wait_with_output()
            .context("Failed waiting for child output")
            .context(dbg_msg)?;
        let end = duration_since_boot()?;
        let duration = (end - start).as_nanos() as f64;
        Ok(CommandOutput { duration, output })
    }
}

pub trait CommandExecutorTrait {
    fn execute(self) -> Result<std::process::Output>;

    fn start_execute(self) -> Result<std::process::Child>;
}
