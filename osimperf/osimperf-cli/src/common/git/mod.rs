mod commit;
mod git;
mod repo;

pub use commit::Commit;
pub use repo::{Repository, RepositoryState};

use anyhow::{Context, Result};

pub type Date = chrono::NaiveDate;

fn parse_date(s: &str) -> Result<Date> {
    Date::parse_from_str(s, "%Y-%m-%d").with_context(|| format!("failed to parse date string: {s}"))
}

fn format_date(d: &Date) -> String {
    d.format("%Y-%m-%d").to_string()
}
