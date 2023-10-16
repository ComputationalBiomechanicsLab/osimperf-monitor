use super::absolute_path;
use super::ResultInfo;
use crate::{
    common::format_date,
    common::parse_date,
    record::{BenchTestResult, BenchTestSetup, Durations, TestNode},
    write_json, CMakeCommands, Commit, Ctxt, Date, EnvVars, FileBackedStruct, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::debug;
use log::info;
use rand::prelude::*;
use std::io::Lines;
use std::io::StdinLock;
use std::{
    fs::File,
    io::{self, LineWriter, Write},
    path::PathBuf,
    str::FromStr,
};

// osimperf-cli results --table => | | test | ...
// osimperf-cli results --table

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Path to result file (or reads from stdin).
    #[arg(long)]
    results: Option<PathBuf>,

    /// Output path.
    #[arg(long)]
    out: Option<PathBuf>,
}

impl PlotCommand {
    pub fn run(&self) -> Result<()> {
        // let mut file = self.out.as_ref().map(|path| {
        //     LineWriter::new(File::create(path).expect(&format!(
        //         "failed to open file for writing stdout logs at path = {:?}",
        //         &self.out
        //     )))
        // });

        let mut results = Vec::new();
        let mut rows = Vec::new();
        let mut cols = Vec::new();
        for path in ArgOrStdinIter::new(&self.results) {
            let result = crate::read_json::<ResultInfo>(&path)?;

            let row = Row {
                name: format!("{} {} {}", result.branch, result.date, result.commit.as_str().split_at(6).0),
                date: parse_date(&result.date)?,
            };

            if rows.iter().find(|&r| r == &row).is_none() {
                rows.push(row.clone());
            }

            if cols.iter().find(|&c| c == &result.name).is_none() {
                cols.push(result.name.clone());
            }

            results.push(TableCell {
                col: result.name,
                row,
                value: result.durations,
            });
        }

        rows.sort_by(|a,b| a.date.cmp(&b.date));
        rows.reverse();
        cols.sort_by(|a,b| a.cmp(b));

        let mut line = String::new();
        line.push_str("| |");
        for col in cols.iter() {
            line.push_str(col);
            line.push_str("|");
        }
        println!("{line}");

        line.clear();
        line.push_str("|---|");
        for _ in 0..cols.len() {
            line.push_str("---|");
        }
        println!("{line}");

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
                    line.push_str(&format!(" {:.3} ({:.3}) |",
                    cell.value.get_mean().unwrap_or(f64::NAN),
                    cell.value.get_stddev().unwrap_or(f64::NAN),
                    ));
                } else {
                    line.push_str(" |");
                }
            }
            println!("{line}");
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
