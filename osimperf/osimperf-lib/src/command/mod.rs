mod piped_command;
mod single_command;

pub use piped_command::{PipedCommands, PipedCommandsExecutor};
pub use single_command::{Command, CommandExecutor};

use anyhow::{Context, Result};
use std::io::BufReader;
use std::thread;
use std::{
    fmt::Debug,
    fs::OpenOptions,
    io::{BufRead, Write},
    path::Path,
    time::Duration,
};
use crate::common::duration_since_boot;

#[derive(Debug)]
pub struct CommandOutput {
    pub duration: Duration,
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
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .context(format!(
                "failed to open file for writing stdout logs at path = {:?}",
                path
            ))?;
        file.write_all(&self.output.stdout)?;
        Ok(())
    }

    pub fn write_stderr(&self, path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .context(format!(
                "failed to open file for writing stderr logs at path = {:?}",
                path
            ))?;
        file.write_all(&self.output.stderr)?;
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
        let duration = end - start;
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
        // Construct command.
        let cmd = self.create_executor();

        // Exectute command and start timer.
        let start = duration_since_boot()?;
        let mut child = cmd.start_execute()?;

        // Access the stdout and stderr of the child process.
        let stderr = child.stderr.take().expect("Failed to capture stderr");
        let stdout = child.stdout.take().expect("Failed to capture stdout");

        // Create buffers to read the std-output.
        let mut stderr_buffer = Vec::<u8>::new();
        let mut stdout_buffer = Vec::<u8>::new();

        // Handle stderr in thread.
        let stderr_handle = thread::spawn(move || -> Result<Vec<u8>> {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                let bytes_read = reader.read_line(&mut line)?;
                if bytes_read == 0 {
                    break;
                }
                stderr_buffer.extend(line.as_bytes());
                line.clear();
            }
            Ok(stderr_buffer)
        });

        // Read stdout to the given argument, and store in buffer.
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }
            stream.write_all(line.as_bytes())?;
            stdout_buffer.extend(line.as_bytes());
            line.clear();
        }

        // Wait for thread to finish and handle any errors.
        let mut stderr_result = stderr_handle
            .join()
            .expect("Failed to join stderr thread")?;

        let mut output = child
            .wait_with_output()
            .context("error waiting for command output")
            .with_context(|| format!("failed to execute command: {}", self.print_command()))?;

        output.stdout.extend(stdout_buffer.drain(..));
        output.stderr.extend(stderr_result.drain(..));

        let end = duration_since_boot()?;
        let duration = end - start;
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
        let dollar_key = format!("${}", key);
        _ = substitute_if_present(&mut out, &dollar_key, value);
    }
    out
}
