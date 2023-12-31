use super::{substitute_all, CommandExecutorTrait, CommandTrait};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{path::Path, process::Stdio};

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Command {
    cmd: String,
    args: Vec<String>,
    envs: Option<Vec<(String, String)>>,
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
            self.args.push(a.to_string());
        }
    }

    pub fn add_env(&mut self, key: impl ToString, value: impl ToString) {
        let key = key.to_string();
        let value = value.to_string();
        self.envs.get_or_insert(Vec::new()).push((key, value));
    }

    pub fn add_env_path(&mut self, key: impl ToString, value: &Path) {
        self.add_env(key, value.to_str().unwrap());
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
        let cmd_str = then_substitute_all(&self.cmd, &self.envs);
        let root_str = self
            .root
            .as_ref()
            .map(|path| then_substitute_all(path, &self.envs));
        let mut cmd = if let Some(root) = root_str {
            let mut cmd = std::process::Command::new("env");
            cmd.arg("-C");
            cmd.arg(root);
            cmd.arg(cmd_str);
            cmd
        } else {
            std::process::Command::new(cmd_str)
        };
        cmd.args(
            self.args
                .iter()
                .map(|arg| then_substitute_all(arg, &self.envs)),
        );
        if let Some(envs) = self.envs.as_ref() {
            for (key, value) in envs.iter() {
                cmd.env(key, value);
            }
        }
        CommandExecutor { cmd }
    }

    fn print_command_with_delim(&self, arg_delim: &str) -> String {
        let mut msg = then_substitute_all(&self.cmd, &self.envs);
        for arg in self
            .args
            .iter()
            .map(|arg| then_substitute_all(arg, &self.envs))
        {
            msg.push_str(arg_delim);
            msg.push_str(&arg);
        }
        msg
    }
}

pub fn then_substitute_all(string: &str, key_value: &Option<Vec<(String, String)>>) -> String {
    if let Some(env) = key_value {
        return substitute_all(string, env);
    }
    String::from(string)
}
