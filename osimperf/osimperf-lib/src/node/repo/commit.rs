use crate::common::git;

use super::Input;
use log::{debug, info, trace};

#[derive(Clone, Debug)]
// Can be created from the [Input]
pub struct Commit {
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

impl Commit {
    pub fn last_commit(input: &Input) -> anyhow::Result<Self> {
        git::get_last_commit(&input.repo, &input.branch).map(|c| Self {
            hash: c.0,
            date: c.1,
        })
    }

    fn commits_between(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Self>> {
        Ok(
            git::get_commits_since(&input.repo, &input.branch, after_date, before_date)?
                .drain(..)
                .map(|c| Self {
                    hash: c.0,
                    date: c.1,
                })
                .collect::<Vec<Self>>(),
        )
    }

    pub fn collect_monthly_commits(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Self>::new();
        for c in Self::commits_between(&input, after_date, before_date)?.drain(..) {
            if let Some(last) = commits.last() {
                let d0 = c.date.as_str().split_at(7).0;
                let d1 = last.date.as_str().split_at(7).0;
                trace!("comparing {:?} to {:?}, same = {}", d0, d1, d0 == d1);
                if d0 == d1 {
                    debug!("Skipping duplicate {:?}", c);
                    continue;
                }
            }
            info!("Last commit of the month: {:#?}", c);
            commits.push(c);
        }
        Ok(commits)
    }

    pub fn collect_daily_commits(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Self>::new();
        for c in Self::commits_between(&input, after_date, before_date)?.drain(..) {
            if let Some(last) = commits.last() {
                trace!("comparing {:?} to {:?}", c.date, last.date,);
                if c.date == last.date {
                    debug!("Skipping duplicate {:?}", c);
                    continue;
                }
            }
            info!("Last commit of the day: {:#?}", c);
            commits.push(c);
        }
        Ok(commits)
    }
}
