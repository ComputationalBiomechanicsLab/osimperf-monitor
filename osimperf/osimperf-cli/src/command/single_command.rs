use crate::EnvVar;

use super::{CommandExecutorTrait, CommandTrait};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{path::Path, process::Stdio};

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Command {
    cmd: String,
    args: Vec<String>,
    envs: Option<Vec<EnvVar>>,
    root: Option<String>,
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
        self.cmd.stdout(Stdio::piped());
        self.cmd.stderr(Stdio::piped());

        Ok(self.cmd.spawn()?)
    }
}

impl Command {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: Vec::new(),
            envs: None,
            root: None,
        }
    }

    pub fn add_arg(&mut self, arg: impl ToString) {
        self.args.push(arg.to_string());
    }

    pub fn add_args<T: ToString>(&mut self, args: impl Iterator<Item = T>) {
        for a in args {
            self.add_arg(a);
        }
    }

    pub fn add_env(&mut self, env: EnvVar) {
        self.envs.get_or_insert(Vec::new()).push(env);
    }

    pub fn add_envs(&mut self, envs: impl Iterator<Item = EnvVar>) {
        for e in envs {
            self.add_env(e);
        }
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

    pub fn set_run_root(&mut self, root: &Path) {
        self.root = Some(String::from(root.to_str().unwrap()));
    }
}

impl CommandTrait for Command {
    type Executor = CommandExecutor;

    fn create_executor(&self) -> CommandExecutor {
        let mut cmd = if let Some(root) = self.root.as_ref() {
            let mut cmd = std::process::Command::new("env");
            cmd.arg("-C");
            cmd.arg(root);
            cmd.arg(&self.cmd);
            cmd
        } else {
            std::process::Command::new(&self.cmd)
        };
        cmd.args(self.args.iter());
        if let Some(envs) = self.envs.as_ref() {
            for e in envs.iter() {
                cmd.env(&e.key, &e.value);
            }
        }
        CommandExecutor { cmd }
    }

    fn print_command_with_delim(&self, arg_delim: &str) -> String {
        let mut msg = String::new();
        if let Some(envs) = self.envs.as_ref() {
            for e in envs.iter() {
                msg.push_str(&format!("{}={}{}", e.key, e.value, arg_delim));
            }
        }
        msg.push_str(&self.cmd);
        for arg in self.args.iter() {
            msg.push_str(arg_delim);
            msg.push_str(&arg);
        }
        msg
    }
}
