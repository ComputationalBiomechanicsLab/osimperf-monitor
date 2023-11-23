use super::*;
use crate::git::parse_date;
use crate::git::Date;
use crate::*;
use anyhow::Result;
use std::io::LineWriter;
use std::{io::Write, path::PathBuf};

pub fn print_table(table: &Table, buf: impl std::io::Write) -> Result<()> {
    let mut buf = LineWriter::new(buf);

    let mut line = String::new();

    // Print column headers.
    line.push_str("| |");
    for row in table {
        for col in row {
            line.push_str(&col.col_name);
            line.push_str("|");
        }
        break;
    }
    line.push_str("\n");
    buf.write_all(line.as_bytes())?;

    line.clear();
    line.push_str("|---|");
    for row in table {
        for _ in row {
        line.push_str("---|");
        }
        break;
    }
    line.push_str("\n");
    buf.write_all(line.as_bytes())?;

    for row in table {
        line.clear();
        line.push_str("|");
        line.push_str(&row.row_name());
        line.push_str("|");
        for cell in row {
            cell.write_cell_str(&mut line);
            line.push_str(" |");
        }
        line.push_str("\n");
        buf.write_all(line.as_bytes())?;
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
    opensim_name: String,
    test_name: String,
    value: Durations,
}
