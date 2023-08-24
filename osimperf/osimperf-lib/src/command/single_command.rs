use super::*;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, Default)]
pub struct Command {
    cmd: String,
    args: Vec<String>,
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
        Ok(self.get_mut().spawn()?)
    }
}

impl Command {
    pub fn new(cmd: impl ToString) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: Vec::new(),
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
}

impl CommandTrait for Command {
    type Executor = CommandExecutor;

    fn create_executor(&self) -> CommandExecutor {
        let mut cmd = std::process::Command::new(&self.cmd);
        cmd.args(&self.args);
        CommandExecutor { cmd }
    }

    fn print_command(&self) -> String {
        let mut msg = self.cmd.clone();
        for arg in self.args.iter() {
            msg.push_str(" ");
            msg.push_str(arg);
        }
        msg
    }
}
