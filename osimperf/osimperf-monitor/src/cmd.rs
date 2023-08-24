use anyhow::{ensure, Context, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::str;
use std::thread::sleep;
use std::time::Duration;

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

    pub fn run_print(&self, log: &mut String) -> Result<()> {
        let mut cmd = std::process::Command::new(&self.cmd);
        cmd.args(&self.args);
        let mut child = cmd
            .stdout(Stdio::piped())
            .spawn()
            .context(format!("Command {:?} failed", cmd))?;

        // Stream output.
        let stdout = child.stdout.take().unwrap();
        let lines = BufReader::new(stdout).lines();
        for line in lines {
            let line = line?;
            log.push_str(line.trim());

            println!("{:?}", line);
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

        ensure!(
            output.stderr.len() == 0 && output.status.success(),
            format!(
                "Command {:?} exited with error:\n{}",
                cmd,
                String::from(str::from_utf8(&output.stderr)?.trim()),
            )
        );
        Ok(())
    }

    pub fn run_extend_log(&self, log: &mut String) -> Result<f64> {
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
        log.push_str(str::from_utf8(&output.stdout)?.trim());
        Ok(duration)
    }

    pub fn run(&self) -> Result<String> {
        let mut log = String::new();
        self.run_extend_log(&mut log)?;
        Ok(log)
    }

    // Substitutes environmental variable
    pub fn sub_env_var(&mut self, key: &str, value: &str) {
        let key = format!("${}", key);
        substitute_if_present(&mut self.cmd, &key, value);
        for arg in self.args.iter_mut() {
            substitute_if_present(arg, &key, value);
        }
    }
}

fn substitute_if_present(string: &mut String, key: &str, value: &str) -> Option<()> {
    let start = string.find(key)?;
    let end = start + key.len();
    let start = string.find(key)?;
    string.replace_range(start..end, value);
    Some(())
}

pub fn pipe_commands(cmds: &[Command]) -> Result<String> {
    if cmds.len() == 0 {
        return Ok(String::new());
    }

    let child = std::process::Command::new(&cmds[0].cmd)
        .args(&cmds[0].args)
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
        let parent = std::process::Command::new(&cmds[i].cmd)
            .args(&cmds[i].args)
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
            let log = String::from(str::from_utf8(&output.stdout)?.trim());
            return Ok(log);
        }

        last_child = parent.stdout.context(format!(
            "Failed to open parent stdout, of cmd={:?}",
            &cmds[i - 1]
        ))?;
    }

    Ok(String::new())
}
