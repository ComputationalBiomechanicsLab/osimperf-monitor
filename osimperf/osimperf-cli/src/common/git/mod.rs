mod commit;
mod git;
mod repo;

pub use commit::Commit;
pub use repo::{Repository, RepositoryState};
pub use git::verify_repository;

use anyhow::{Context, Result};

pub type Date = chrono::NaiveDate;

pub fn parse_date(s: &str) -> Result<Date> {
    let out =
    Date::parse_from_str(s, "%Y-%m-%d").with_context(|| format!("failed to parse date string: {s}"));
    if out.is_err() {
        let alternative = Date::parse_from_str(s, "%Y_%m_%d").with_context(|| format!("failed to parse date string: {s}"));
        if alternative.is_ok() {
            return alternative;
        }
    } else {
        return out;
    }
    out
}

pub fn format_date(d: &Date) -> String {
    d.format("%Y-%m-%d").to_string()
}
