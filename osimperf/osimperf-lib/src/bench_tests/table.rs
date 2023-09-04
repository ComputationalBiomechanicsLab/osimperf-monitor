use anyhow::Result;
use std::io::Write;

use crate::{CompilationNode, ResultsFolder};

use super::{BenchTestResult, BenchTestSetup};

//// Name, date, status, time,
const NODE_COL_HEADERS: [&str; 2] = ["Date", "Status"];

pub fn print_results(
    compiler_nodes: &[CompilationNode],
    tests: &[BenchTestSetup],
    results: &ResultsFolder,
    mut buffer: impl Write,
) -> Result<()> {
    // Header:
    buffer.write_all(b"|")?;
    for b in NODE_COL_HEADERS
        .iter()
        .map(|c| c.as_bytes())
        .chain(tests.iter().map(|t| t.name.as_bytes()))
    {
        buffer.write_all(b" ")?;
        buffer.write_all(b)?;
        buffer.write_all(b" |")?;
    }

    // Header seperated:  | --- | --- | ...
    buffer.write_all(b"\n|")?;
    for _ in 0..NODE_COL_HEADERS.len() + tests.len() {
        buffer.write_all(b" --- |")?;
    }

    // Print a row for each commit.
    for c in compiler_nodes.iter() {
        buffer.write_all(b"\n| ")?;
        let name = format!("{}-{}", c.repo.name, c.repo.date);
        let status = c.state.get()[1].print_table_entry();
        for b in [name.as_bytes(), status.as_bytes()].iter() {
            buffer.write_all(b" ")?;
            buffer.write_all(b)?;
            buffer.write_all(b" |")?;
        }
        if c.is_done() {
            // Print a column for each test.
            for t in tests.iter() {
                buffer.write_all(b" ")?;
                let result = BenchTestResult::read(results, &c.id(), &t.name)?;
                if let (Some(dt), stddev, Some(iter)) = (
                    result.as_ref().and_then(|x| x.duration),
                    result
                        .as_ref()
                        .and_then(|x| x.duration_stddev)
                        .unwrap_or(f64::NAN),
                    result.as_ref().map(|x| x.iteration),
                ) {
                    if stddev < 1e-2 {
                        buffer.write_all(format!("{:.2}", dt).as_bytes())?;
                    } else {
                        buffer.write_all(format!("{:.2} ({:.3}, {iter}X)", dt, stddev).as_bytes())?;
                    }
                } else {
                    buffer.write_all(b"Failed")?;
                }
                buffer.write_all(b" |")?;
            }
        } else {
            for _ in 0..tests.len() {
                buffer.write_all(b" |")?;
            }
        }
    }
    buffer.flush()?;

    Ok(())
}
