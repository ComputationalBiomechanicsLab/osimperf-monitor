use serde::{Deserialize, Serialize};
use std::process::Stdio;

use super::*;

use anyhow::{anyhow, Context, Result};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Command {
    cmd: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct CommandExecutor {
    cmd: std::process::Command,
}

impl CommandExecutor {
    pub fn get_mut(&mut self) -> &mut std::process::Command {
        &mut self.cmd
    }
}

impl CommandExecutorTrait for CommandExecutor {
    fn execute(mut self) -> Result<std::process::Output> {
        Ok(self.get_mut().output()?)
    }

    fn start_execute(mut self) -> Result<std::process::Child> {
        Ok(self.get_mut().stdout(Stdio::piped()).spawn()?)
    }
}

impl Command {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: Vec::new(),
            envs: Vec::new(),
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
        self.envs.push((key, value));
    }

    pub fn parse(string: &str) -> Self {
        let mut split = string.split(' ');
        let mut cmd = Self::new(split.next().unwrap());
        for s in split {
            if !s.is_empty() {
                cmd.add_arg(s);
            }
        }
        cmd
    }
}

impl CommandTrait for Command {
    type Executor = CommandExecutor;

    fn create_executor(&self) -> CommandExecutor {
        let mut cmd = std::process::Command::new(substitute_all(&self.cmd, &self.envs));
        cmd.args(self.args.iter().map(|arg| substitute_all(arg, &self.envs)));
        CommandExecutor { cmd }
    }

    fn print_command_with_delim(&self, arg_delim: &str) -> String {
        let mut msg = substitute_all(&self.cmd, &self.envs);
        for arg in self.args.iter().map(|arg| substitute_all(arg, &self.envs)) {
            msg.push_str(arg_delim);
            msg.push_str(&arg);
        }
        msg
    }
}
