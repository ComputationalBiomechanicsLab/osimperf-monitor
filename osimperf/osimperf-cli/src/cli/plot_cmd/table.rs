use super::ArgOrStdinIter;
use super::ResultInfo;
use crate::git::parse_date;
use crate::git::Date;
use crate::*;
use anyhow::Result;
use std::io::LineWriter;
use std::{io::Write, path::PathBuf};

pub fn print_table(arg_path: &Option<PathBuf>, buf: impl std::io::Write) -> Result<()> {
    let mut buf = LineWriter::new(buf);

    let mut results = Vec::new();
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    for path in ArgOrStdinIter::new(arg_path) {
        let result = crate::read_json::<ResultInfo>(&path)?;

        let row = Row {
            name: format!(
                "{} {} {}",
                result.opensim_name,
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
