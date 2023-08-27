mod piped_command;
mod single_command;
mod time;

pub use piped_command::{PipedCommands, PipedCommandsExecutor};
pub use single_command::{Command, CommandExecutor};
pub use time::duration_since_boot;

use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use anyhow::{ensure, Context, Result};

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

    pub fn write_logs(&self, buffer: &mut impl Write) -> Result<()> {
        buffer.write_all(&self.output.stderr)?;
        buffer.write_all(&self.output.stdout)?;
        buffer.write_all(format!("{:#?}", self.output.status).as_bytes())?;
        Ok(())
    }
}

pub trait CommandTrait {
    type Executor: CommandExecutorTrait + Debug;

    fn print_command_with_delim(&self, arg_delim: &str) -> String;

    fn print_command(&self) -> String {
        self.print_command_with_delim(" ")
    }

    fn create_executor(&self) -> Self::Executor;

    fn run_and_time(&self) -> Result<CommandOutput> {
        let cmd = self.create_executor();
        let start = duration_since_boot()?;
        let output = cmd.execute();
        let end = duration_since_boot()?;
        let duration = (end - start).as_secs_f64();
        Ok(CommandOutput {
            duration,
            output: output
                .with_context(|| format!("failed to execute command: {}", self.print_command()))?,
        })
    }

    fn run_stdout(&self) -> Result<Vec<u8>> {
        let output = self.run_and_time()?;
        Some(())
            .filter(|_| output.success())
            .with_context(|| format!("stdout: {:#?}", output.stdout_str_clone()))
            .with_context(|| format!("stderr: {:#?}", output.stderr_str_clone()))
            .with_context(|| format!("returned exit code: {:#?}", output.output.status))
            .with_context(|| format!("failed to execute command: {}", self.print_command()))?;
        Ok(output.output.stdout)
    }

    fn run(&self) -> Result<String> {
        Ok(String::from_utf8(self.run_stdout()?).unwrap())
    }

    fn run_trim(&self) -> Result<String> {
        Ok(String::from(self.run()?.trim()))
    }

    fn run_and_stream(&self, stream: &mut impl Write) -> Result<CommandOutput> {
        let cmd = self.create_executor();
        let start = duration_since_boot()?;
        let mut child = cmd.start_execute()?;

        let stdout = child.stdout.take().unwrap();
        let lines = BufReader::new(stdout).lines();
        for line in lines {
            let line = line?;
            stream.write_all(line.as_bytes())?;
            stream.write_all('\n'.to_string().as_bytes())?;
            if child
                .try_wait()
                .with_context(|| format!("failed to execute command: {}", self.print_command()))
                .context("error while waiting for child")?
                .is_some()
            {
                break;
            }
        }

        let output = child
            .wait_with_output()
            .context("error waiting for command output")
            .with_context(|| format!("failed to execute command: {}", self.print_command()))?;
        let end = duration_since_boot()?;
        let duration = (end - start).as_nanos() as f64;
        Ok(CommandOutput { duration, output })
    }
}

pub trait CommandExecutorTrait {
    fn execute(self) -> Result<std::process::Output>;

    fn start_execute(self) -> Result<std::process::Child>;
}

pub(crate) fn substitute_if_present(string: &mut String, key: &str, value: &str) -> Option<()> {
    let start = string.find(key)?;
    let end = start + key.len();
    let start = string.find(key)?;
    string.replace_range(start..end, value);
    Some(())
}

pub(crate) fn substitute_all(string: &str, key_value: &[(String, String)]) -> String {
    let mut out = String::from(string);
    for (key, value) in key_value.iter() {
        _ = substitute_if_present(&mut out, key, value);
    }
    out
}
