use anyhow::{ensure, Context, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Output, Stdio};
use std::str;
use std::thread::sleep;
use std::time::Duration;

use crate::time::duration_since_boot;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Command {
    pub cmd: String,
    pub args: Vec<String>,
    // Key, Value pair environmental variables.
    pub env: Option<Vec<(String, String)>>,
}

impl Command {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: Vec::new(),
            env: None,
        }
    }

    pub fn add_arg(&mut self, arg: impl ToString) {
        self.args.push(arg.to_string());
    }

    pub fn add_args<T: ToString>(&mut self, args: impl Iterator<Item = T>) {
        for a in args {
            self.args.push(a.to_string());
        }
    }

    pub fn add_env(&mut self, key: impl ToString, value: impl ToString) {
        let key = key.to_string();
        let value = value.to_string();
        self.sub_env_var(&key, &value);
        self.env.get_or_insert(Vec::new()).push((key, value));
    }

    // Substitutes environmental variable TODO weird method.
    pub fn sub_env_var(&mut self, key: &str, value: &str) {
        let key = format!("${}", key);
        substitute_if_present(&mut self.cmd, &key, value);
        for arg in self.args.iter_mut() {
            substitute_if_present(arg, &key, value);
        }
    }

    /// TODO create thread for collecting stdin, stdout simultaneously.
    pub fn run_stream_output(
        &self,
        mut stdout_log: impl Write,
        mut stderr_log: impl Write,
        mut stream: impl Write,
    ) -> Result<f64> {
        let mut cmd = std::process::Command::new(&self.cmd);
        cmd.args(&self.args);
        if let Some(envs) = self.env.as_ref() {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        let start = duration_since_boot()?;
        let mut child = cmd
            .stdout(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn command: {:?}", cmd))?;

        // Stream output.
        let stdout = child.stdout.take().unwrap();

        let lines = BufReader::new(stdout).lines();
        for line in lines {
            let line = line?;
            stream.write_all(line.as_bytes());
            if child
                .try_wait()
                .context("error attempting to wait for command.")?
                .is_some()
            {
                break;
            }
        }

        let output = child
            .wait_with_output()
            .expect("Failed to execute piped commands.");
        let end = duration_since_boot()?;
        let duration = (end - start).as_nanos() as f64;

        stdout_log.write_all(&output.stdout);
        stderr_log.write_all(&output.stderr);

        ensure!(
            output.status.success(),
            format!("Exit status {:?} of command {:?}", output.status, cmd)
        );
        Ok(duration)
    }

    pub fn run_to_string(&self) -> Result<String> {
        let mut x = Vec::<u8>::new();
        let mut y = Vec::<u8>::new();
        self.run_extend_log(&mut x, &mut y)?;
        let mut output = String::from(
            String::from_utf8(x)?.trim());
        // output.push_str(&String::from_utf8(y)?.trim());
        Ok(output)
    }

    pub fn run_extend_log(&self, mut stdout_log: impl Write, mut stderr_log: impl Write) -> Result<f64> {
        sleep(Duration::from_secs_f64(0.1));
        let mut cmd = std::process::Command::new(&self.cmd);
        cmd.args(&self.args);
        if let Some(envs) = self.env.as_ref() {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }

        trace!("Preparing to run command:");
        trace!("CMD: {}", self.cmd);
        trace!("with args:");

        for arg in self.args.iter() {
            println!("        {}", arg);
        }

        let start = crate::time::duration_since_boot()
            .context("Failed to read system clock before running command")?;

        let output = cmd.output();

        let end = crate::time::duration_since_boot()
            .context("Failed to read system clock after running command")?;
        let duration = (end - start).as_nanos() as f64;

        trace!("completed cmd in {duration} seconds");
        trace!("cmd output = {:#?}", output);

        let output = match output {
            Err(err) => return Err(err).context("dude")?,
            Ok(output) => output,
        };

        ensure!(
            output.stderr.len() == 0 && output.status.success(),
            format!(
                "Command {:?} exited with error:\n    stderr: {},\n    stdout: {}\n    status: {}",
                cmd,
                String::from(str::from_utf8(&output.stderr)?.trim()),
                String::from(str::from_utf8(&output.stdout)?.trim()),
                output.status,
            )
        );
        stdout_log.write_all(&output.stdout);
        stderr_log.write_all(&output.stderr);
        Ok(duration)
    }
}

fn substitute_if_present(string: &mut String, key: &str, value: &str) -> Option<()> {
    let start = string.find(key)?;
    let end = start + key.len();
    let start = string.find(key)?;
    string.replace_range(start..end, value);
    Some(())
}

pub fn pipe_commands(
    cmds: &[Command],
    mut stdout_log: impl Write,
    mut stderr_log: impl Write,
) -> Result<f64> {
    if cmds.len() == 0 {
        return Ok(f64::NAN);
    }

    let start = duration_since_boot()?;
    let mut cmd = std::process::Command::new(&cmds[0].cmd);
    if let Some(envs) = cmds[0].env.as_ref() {
        for (key, value) in envs {
            cmd.env(key, value);
        }
    }
    let child = cmd
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context(format!(
            "Failed to start child process: cmd = {:?}",
            cmds[0]
        ))?;

    let mut last_child = child.stdout.context(format!(
        "Failed to open child stdout, of cmd={:?}",
        &cmds[0]
    ))?;

    for i in 1..cmds.len() {
        let mut cmd = std::process::Command::new(&cmds[i].cmd);
        if let Some(envs) = cmds[i].env.as_ref() {
            for (key, value) in envs {
                cmd.env(key, value);
            }
        }
        let parent = cmd
            .stdin(std::process::Stdio::from(last_child))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .context(format!(
                "Failed to start parent process: cmd = {:?}",
                cmds[i]
            ))?;

        if i == cmds.len() - 1 {
            let output = parent
                .wait_with_output()
                .expect("Failed to execute piped commands.");
            let end = duration_since_boot()?;
            let duration = (end - start).as_nanos() as f64;

            stdout_log.write_all(&output.stdout);
            stderr_log.write_all(&output.stderr);

            return Ok(duration);
        }

        last_child = parent.stdout.context(format!(
            "Failed to open parent stdout, of cmd={:?}",
            &cmds[i - 1]
        ))?;
    }

    Ok(f64::NAN)
}
