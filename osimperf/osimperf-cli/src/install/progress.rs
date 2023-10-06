use regex::Regex;
use std::io::Write;

use super::status::Progress;
use super::status::Status;
use crate::FileBackedStruct;
use crate::Ctxt;
use super::CompilationNode;
use anyhow::Context;
use std::str;

#[derive(Debug)]
pub struct CMakeProgressStreamer<'a> {
    task: String,
    buffer: String,
    percentage: Option<f64>,
    parent: &'a mut CompilationNode,
    context: &'a Ctxt,
    re: Regex,
}

impl<'a> CMakeProgressStreamer<'a> {
    pub fn new(parent: &'a mut CompilationNode, context: &'a Ctxt, task: String) -> Self {
        Self {
            task,
            buffer: String::new(),
            percentage: None,
            parent,
            re: Regex::new(r"\[\s*(\d+)%\]").unwrap(),
            context,
        }
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
                if let Some(captures) = self.re.captures(line) {
                    if let Some(percentage_str) = captures.get(1) {
                        self.percentage =
                            Some(percentage_str.as_str().parse::<f64>().with_context(|| {
                                format!("failed to parse percentage {}", percentage_str.as_str())
                            })?);

                        self.parent.status =
                            Status::Compiling(Progress {
                                percentage: self.percentage.unwrap_or(0.),
                                task: self.task.clone(),
                            });
                        self.parent.try_write(self.context)?;
                    }
                }
                println!("{line}");
            }

            // Keep the last incomplete line in the buffer
            self.buffer = lines[num_lines - 1].to_string();
        }
        Ok(())
    }
}

impl<'a> Write for CMakeProgressStreamer<'a> {
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
