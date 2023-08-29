use crate::git;

use super::Input;
use log::{debug, info, trace};

#[derive(Clone, Debug)]
// Can be created from the [Input]
pub struct Params {
    /// The commit we are checking out.
    pub hash: String,
    /// The date is for ordering results.
    pub date: String,
}

impl Params {
    fn commits_between(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Self>> {
        Ok(
            git::get_commits_since(&input.repo, &input.branch, after_date, before_date)?
                .drain(..)
                .map(|hash| Self {
                    date: git::date_of_commit(&input.repo, &hash).unwrap(),
                    hash,
                })
                .collect::<Vec<Self>>(),
        )
    }

    pub fn collect_daily_commits(
        input: &Input,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Params>> {
        let mut commits = Vec::<Self>::new();
        for (i, c) in Self::commits_between(&input, after_date, before_date)?
            .drain(..)
            .enumerate()
        {
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