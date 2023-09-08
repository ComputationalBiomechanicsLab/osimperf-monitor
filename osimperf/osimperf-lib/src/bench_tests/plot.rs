use anyhow::{Context, Result};
use log::info;
use std::{
    fs::File,
    io::{LineWriter, Write}, path::PathBuf,
};

use crate::{CompilationNode, Folder, ResultsFolder};

use super::{BenchTestResult, BenchTestSetup};

pub fn print_csv(
    nodes: &[CompilationNode],
    test: &BenchTestSetup,
    results: &ResultsFolder,
) -> Result<PathBuf> {
    let path = results.path()?.join(format!("{}.csv", test.name));
    let file = File::create(&path).with_context(|| {
        format!(
            "failed to open file for writing stdout logs at path = {:?}",
            &path
        )
    })?;
    let mut file = LineWriter::new(file);

    // file with "date,duration" lines
    let first_date = nodes[nodes.len()-1].commit.date()?;
    info!("First date = {}", first_date);
    for node in nodes.iter() {
        if let Some(result) = BenchTestResult::read(results, &node.id(), &test.name)? {
            for duration in result.durations.get() {
                let days = (node.commit.date()? - first_date).num_days();
                info!("{} - {} = {} days", node.commit.date()?, first_date, days);
                file.write_all(format!("{days},").as_bytes())
                    .with_context(|| format!("Failed to write number of days to {:?}", path))?;
                file.write_all(node.commit.date.as_bytes())
                    .with_context(|| format!("Failed to write date to {:?}", path))?;
                file.write_all(format!(",{}\n", duration.as_secs_f64()).as_bytes())
                    .with_context(|| format!("Failed to write duration to {:?}", path))?;
            }
        }
    }
    file.flush()
        .with_context(|| format!("Failed to flush {:?}", path))?;

    Ok(path)
}
