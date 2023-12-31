use super::{substitute_all, CommandExecutorTrait, CommandTrait};
use super::EnvVar;
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

    pub fn set_arg_if(mut self, key: &str, arg: &Option<String>) -> Command {
        if let Some(value) = arg {
            self.add_arg(key);
            self.add_arg(value);
        }
        self
    }

    pub fn add_args<T: ToString>(&mut self, args: impl Iterator<Item = T>) {
        for a in args {
            self.args.push(a.to_string());
        }
    }

    pub fn add_env(&mut self, env: EnvVar) {
        self.envs.get_or_insert(Vec::new()).push(env);
    }

    pub fn add_envs(&mut self, envs: &[EnvVar]) {
        for env in envs {
            self.add_env(env.clone());
        }
    }

    pub fn set_envs(mut self, envs: &[EnvVar]) -> Self {
        self.add_envs(envs);
        self
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

    #[must_use]
    pub fn set_run_root(mut self, root: &Path) -> Self {
        self.root = Some(String::from(root.to_str().unwrap()));
        self
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
            for env in envs.iter() {
                cmd.env(&env.key, &env.value);
            }
        }
        CommandExecutor { cmd }
    }

    fn print_command_with_delim(&self, arg_delim: &str) -> String {
        let mut msg = String::new();
        if let Some(envs) = self.envs.as_ref() {
            for env in envs.iter() {
                msg.push_str(&format!("{}={} ", env.key, env.value));
            }
        }
        if let Some(root) = self.root.as_ref() {
            msg.push_str(&format!("env -C {}{}", root, arg_delim));
        }
        msg.push_str(&self.cmd);
        for arg in self.args.iter() {
            msg.push_str(arg_delim);
            msg.push_str(&arg);
        }
        then_substitute_all(&msg, &self.envs)
    }
}

pub fn then_substitute_all(string: &str, key_value: &Option<Vec<EnvVar>>) -> String {
    if let Some(env) = key_value {
        return substitute_all(string, env);
    }
    String::from(string)
}
