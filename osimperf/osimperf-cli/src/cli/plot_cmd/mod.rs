use super::absolute_path;
use super::ResultInfo;
use crate::*;
use crate::git::Date;
use crate::git::parse_date;
use anyhow::{Context, Result};
use clap::Args;
use std::io::Lines;
use std::io::StdinLock;
use std::{fs::File, io::Write, path::PathBuf, str::FromStr};

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Path to result file (or reads from stdin).
    #[arg(long)]
    results: Option<PathBuf>,

    /// Output path.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn print_table(arg_path: &Option<PathBuf>, mut buf: impl std::io::Write) -> Result<()> {
    let mut results = Vec::new();
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    for path in ArgOrStdinIter::new(arg_path) {
        let result = crate::read_json::<ResultInfo>(&path)?;

        let row = Row {
            name: format!(
                "{} {} {}",
                result.branch,
                result.date,
                result.commit.as_str().split_at(6).0
            ),
            date: parse_date(&result.date)?,
        };

        let name = result.cell_name.clone().unwrap_or(result.name.clone());

        if rows.iter().find(|&r| r == &row).is_none() {
            rows.push(row.clone());
        }

        if cols.iter().find(|&c| c == &name).is_none() {
            cols.push(name.clone());
        }

        results.push(TableCell {
            col: name,
            row,
            value: result.durations,
        });
    }

    rows.sort_by(|a, b| a.date.cmp(&b.date));
    rows.reverse();
    cols.sort_by(|a, b| a.cmp(b));

    let mut line = String::new();

    if rows.len() > 1 {
        line.push_str("| |");
        for col in cols.iter() {
            line.push_str(col);
            line.push_str("|");
        }
        line.push_str("\n");
        buf.write_all(line.as_bytes())?;

        line.clear();
        line.push_str("|---|");
        for _ in 0..cols.len() {
            line.push_str("---|");
        }
        line.push_str("\n");
        buf.write_all(line.as_bytes())?;

        for row in rows.iter() {
            line.clear();
            line.push_str("|");
            line.push_str(&row.name);
            line.push_str("|");
            for col in cols.iter() {
                if let Some(cell) = results
                    .iter()
                    .find(|result| &result.row == row && &result.col == col)
                {
                    line.push_str(&format!(
                        " {:.3} ({:.3}) |",
                        cell.value.get_mean().unwrap_or(f64::NAN),
                        cell.value.get_stddev().unwrap_or(f64::NAN),
                    ));
                } else {
                    line.push_str(" |");
                }
            }
            line.push_str("\n");
            buf.write_all(line.as_bytes())?;
        }
    } else {
        let tmp = cols.clone();
        let cols = rows.clone();
        let rows = tmp.clone();

        line.push_str("| |");
        for col in cols.iter() {
            line.push_str(&col.name);
            line.push_str("|");
        }
        line.push_str("\n");
        buf.write_all(line.as_bytes())?;

        line.clear();
        line.push_str("|---|");
        for _ in 0..cols.len() {
            line.push_str("---|");
        }
        line.push_str("\n");
        buf.write_all(line.as_bytes())?;

        for row in rows.iter() {
            line.clear();
            line.push_str("|");
            line.push_str(&row);
            line.push_str("|");
            for col in cols.iter() {
                if let Some(cell) = results
                    .iter()
                    .find(|result| &result.col == row && &result.row == col)
                {
                    line.push_str(&format!(
                        " {:.3} ({:.3}) |",
                        cell.value.get_mean().unwrap_or(f64::NAN),
                        cell.value.get_stddev().unwrap_or(f64::NAN),
                    ));
                } else {
                    line.push_str(" |");
                }
            }
            line.push_str("\n");
            buf.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

impl PlotCommand {
    pub fn run(&self) -> Result<()> {
        if let Some(path) = self.out.as_ref() {
            let mut file = File::create(path).with_context(|| {
                format!(
                    "failed to open file for writing stdout logs at path = {:?}",
                    &self.out
                )
            })?;
            print_table(&self.results, &mut file)?;
            file.flush()
                .with_context(|| format!("Failed to flush {:?}", self.out))?;
        } else {
            print_table(&self.results, std::io::stdout())?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Ord, Eq, PartialOrd, PartialEq)]
struct Row {
    name: String,
    date: Date,
}

struct TableCell {
    row: Row,
    col: String,
    value: Durations,
}

pub struct ArgOrStdinIter {
    arg: Option<PathBuf>,
    stdin: Option<Lines<StdinLock<'static>>>,
}

impl ArgOrStdinIter {
    pub fn new(arg: &Option<PathBuf>) -> Self {
        Self {
            arg: arg.clone(),
            stdin: if arg.is_none() {
                Some(std::io::stdin().lines())
            } else {
                None
            },
        }
    }
}

impl Iterator for ArgOrStdinIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stdin) = self.stdin.as_mut() {
            stdin
                .next()
                .map(|s| s.expect("failed to read stdin"))
                .map(|s| PathBuf::from_str(&s).expect("failed to create PathBuf from str"))
        } else {
            return self.arg.take();
        }
        .map(|path| absolute_path(&path).expect("failed to create absolute_path"))
    }
}
