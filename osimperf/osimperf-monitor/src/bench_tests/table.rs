use crate::{Commit, Folders};
use anyhow::Result;
use std::io::Write;

use super::BenchTestSetup;

pub fn print_results(
    folders: &Folders,
    commits: &[Commit],
    tests: &[BenchTestSetup],
    mut buffer: impl Write,
) -> Result<()> {
    // Header:
    buffer.write_all(b"| |");
    for t in tests.iter() {
        buffer.write_all(t.name.as_bytes());
        buffer.write_all(b"| |");
    }

    // Header seperated:  | --- | --- | ...
    buffer.write_all(b"\n| |");
    for t in tests.iter() {
        buffer.write_all(b"| --- |");
    }

    // Print a row for each commit.
    for c in commits.iter() {
        buffer.write_all(b"\n| ");
        buffer.write_all(c.date.as_bytes());
        buffer.write_all(b" |");
        // Print a column for each test.
        for t in tests.iter() {
            if let Some(result) = super::read_test_result(folders, t, c)? {
                buffer.write_all(format!("{:.2}", result.duration).as_bytes());
            } else {
                buffer.write_all(b"X");
            }
            buffer.write_all(b" |");
        }
    }
    buffer.flush()?;

    Ok(())
}
