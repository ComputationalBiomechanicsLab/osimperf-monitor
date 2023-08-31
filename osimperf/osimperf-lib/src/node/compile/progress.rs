use std::io::{self, Write};

use anyhow::Context;
use std::str;

use crate::{Command, PipedCommands, CommandTrait};

#[derive(Clone, Debug, Default)]
pub struct ProgressStreamer {
    process_name: String,
    process_step: String,
    buffer: String,
    percentage: Option<f64>,
}

impl ProgressStreamer {
    pub fn set_process_name(&mut self, name: &str) {
        self.percentage = None;
        self.process_name = name.to_string();
    }

    pub fn set_process_step(&mut self, name: &str) {
        self.percentage = None;
        self.process_step = name.to_string();
    }

    fn pop_line(&mut self) -> anyhow::Result<()> {
        // Check if a complete line is present in the buffer
        // println!("line = {:?}", self.buffer);
        if self.buffer.contains('\n') {
            // Split the buffer into lines and process each complete line
            let lines: Vec<&str> = self.buffer.split('\n').collect();
            let num_lines = lines.len();

            // Print and remove all complete lines except the last one (if it's incomplete)
            for line in lines.iter().take(num_lines - 1) {
                let mut cmd_echo = Command::parse("echo");
                cmd_echo.add_arg(format!(r#""{}""#, line));

                let mut cmd_grep = Command::new("grep");
                cmd_grep.add_arg("-o");
                cmd_grep.add_arg("\\[ [0-9]*%");

                let mut cmd_sed = Command::new("sed");
                cmd_sed.add_arg("s/[^0-9]//g");

                let cmd = PipedCommands::new(vec![cmd_echo, cmd_grep, cmd_sed]);
                if let Some(perc_str) = Some(cmd.run_trim()?).filter(|s| s.len() > 0) {
                    let parsed_percentage = perc_str
                        .parse::<f64>()
                        .with_context(|| format!("failed to parse percentage {perc_str}"))?;
                    if self.percentage.is_some() {
                        print!("\r");
                    }
                    self.percentage = Some(parsed_percentage);
                    print!(
                        "{} {}: {}% -- {}",
                        self.process_step, self.process_name, perc_str, line
                    );
                    io::stdout().flush().context("Failed to flush stdout")?; // Flush the buffer
                }
                // println!(
                //     "{} {}: {:?}% -- {}",
                //     self.process_step, self.process_name, self.percentage, line
                // );
            }

            // Keep the last incomplete line in the buffer
            self.buffer = lines[num_lines - 1].to_string();
        }
        Ok(())
    }
}

impl Write for ProgressStreamer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() > 0 {
            self.buffer.push_str(str::from_utf8(buf).unwrap());
        }
        self.pop_line().map_err(|_| std::io::ErrorKind::NotFound)?; // TODO different error kind.
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.pop_line().map_err(|_| std::io::ErrorKind::NotFound)?; // TODO different error kind.
        Ok(())
    }
}
