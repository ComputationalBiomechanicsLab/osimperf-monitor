use super::*;

use anyhow::{anyhow, Context, Result};
use std::hash::Hash;

#[derive(Debug, Clone, Hash)]
pub struct PipedCommands {
    cmds: Vec<Command>,
}

#[derive(Debug)]
pub struct PipedCommandsExecutor {
    cmds: Vec<CommandExecutor>,
}

impl PipedCommandsExecutor {
    fn get_mut(&mut self, index: usize) -> &mut std::process::Command {
        self.cmds[index].get_mut()
    }

    fn len(&self) -> usize {
        self.cmds.len()
    }
}

impl CommandExecutorTrait for PipedCommandsExecutor {
    fn start_execute(mut self) -> Result<std::process::Child> {
        // Start spawning first command.
        let child = self
            .get_mut(0)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .context(format!(
                "Failed to start child process: cmd = {:?}",
                self.get_mut(0)
            ))?;

        // Grad the stdout handle.
        let mut last_child = child.stdout.context(format!(
            "Failed to open child stdout, of cmd={:?}",
            self.get_mut(0)
        ))?;

        for i in 1..self.len() {
            // Start spawning the next command.
            let parent = self
                .get_mut(i)
                .stdin(std::process::Stdio::from(last_child))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context(format!(
                    "Failed to start parent process: cmd = {:?}",
                    self.get_mut(i)
                ))?;

            // Exit if this was the last command.
            if i == self.len() - 1 {
                return Ok(parent);
            }

            // Grab handle of current command.
            last_child = parent.stdout.context(format!(
                "Failed to open parent stdout, of cmd={:?}",
                self.get_mut(i - 1)
            ))?;
        }

        Err(anyhow!("How did we end up here?"))
    }

    fn execute(self) -> Result<std::process::Output> {
        Ok(self.start_execute()?.wait_with_output()?)
    }
}

impl PipedCommands {
    pub fn new(cmds: Vec<Command>) -> Self {
        Self { cmds }
    }

    pub fn push(&mut self, cmd: Command) {
        self.cmds.push(cmd);
    }

    pub fn parse(string: &str) -> Self {
        let mut pipe = Self { cmds: Vec::new() };
        let split = string.split('|');
        for s in split {
            if !s.is_empty() {
                pipe.cmds.push(Command::parse(s));
            }
        }
        pipe
    }
}

impl CommandTrait for PipedCommands {
    type Executor = PipedCommandsExecutor;
    fn create_executor(&self) -> Self::Executor {
        if self.cmds.len() == 0 {
            panic!();
        }

        let mut cmds = Vec::new();
        for c in self.cmds.iter() {
            cmds.push(c.create_executor());
        }
        Self::Executor { cmds }
    }

    fn print_command_with_delim(&self, arg_delim: &str) -> String {
        if self.cmds.len() == 0 {
            return String::from("empty command");
        }
        let mut iter = self.cmds.iter();
        let mut msg = iter.next().unwrap().print_command_with_delim(arg_delim);
        for arg in iter {
            msg.push_str(" | ");
            msg.push_str(&arg.print_command_with_delim(arg_delim));
        }
        msg
    }
}
