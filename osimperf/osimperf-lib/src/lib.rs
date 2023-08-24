mod cmd_trait;
mod time;

pub use cmd_trait::*;

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
}

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
    pub fn new() -> Self {
        Self { cmds: Vec::new() }
    }

    pub fn push(&mut self, cmd: Command) {
        self.cmds.push(cmd);
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
}
