use crate::{
    cli::ResultInfo,
    git::{format_date, parse_date},
};
use anyhow::{Context, Result};
use log::{debug, info};
use std::{
    io::{LineWriter, Write},
    path::PathBuf,
};

pub fn print_csv_plot(arg_path: &Option<PathBuf>, buf: impl std::io::Write) -> Result<()> {
    info!("Print results to csv plot.");
    let mut file = LineWriter::new(buf);

    let mut first_date = None;
    for path in super::ArgOrStdinIter::new(arg_path) {
        debug!("path = {:?}", path);
        let result = crate::read_json::<ResultInfo>(&path)?;
        debug!("{:?}", result);

        let date = parse_date(&result.date)?;
        let first_date = first_date.get_or_insert_with(|| date.clone());

        let days = (date - *first_date).num_days();
        let name = result.name;
        for duration in result.durations.get().iter() {
            file.write_all(format!("{name},").as_bytes())
                .with_context(|| format!("Failed to write name {:?}", path))?;
            file.write_all(format!("{days},").as_bytes())
                .with_context(|| format!("Failed to write number of days to {:?}", path))?;
            file.write_all(format_date(&date).as_bytes())
                .with_context(|| format!("Failed to write date to {:?}", path))?;
            file.write_all(format!(",{}\n", duration.as_secs_f64()).as_bytes())
                .with_context(|| format!("Failed to write duration to {:?}", path))?;
        }
    }
    file.flush()?;

    Ok(())
}
