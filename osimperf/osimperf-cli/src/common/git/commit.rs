use serde::{Deserialize, Serialize};
use std::hash::Hash;
use super::Date;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
// Can be created from the [Repository]
pub struct Commit {
    /// The commit we are checking out.
    hash: String,
    /// The date is for ordering results.
    date: String,
}

impl Commit {
    pub fn new(hash: String, date: Date) -> Self {
        Self {
            hash,
            date: date.format("%Y_%m_%d").to_string(),
        }
    }

    pub fn hash(&self) -> &String {
        &self.hash
    }

    pub fn date(&self) -> Date {
        Date::parse_from_str(&self.date, "%Y_%m_%d").unwrap()
    }

    pub fn date_str(&self) -> &str {
        &self.date
    }
}
