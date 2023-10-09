use std::path::PathBuf;

use anyhow::ensure;
use chrono::Days;
use log::{debug, info, trace};
use osimperf_lib::git::Date;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use super::{format_date, git, Commit};
use anyhow::Result;

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Repository {
    /// For nicer folder and results identifiers.
    name: String,
    /// Path to repository.
    path: PathBuf,
    /// For checking that path is correct.
    url: String,
    /// The branch the commit should belong to.
    branch: String,
}

impl Repository {
    pub fn new_opensim_core(path: PathBuf) -> Result<Self> {
        let out = Self {
            name: "opensim-core".to_string(),
            branch: "main".to_string(),
            path,
            url: String::from(OPENSIM_CORE_URL),
        };
        out.verify_url()?;
        Ok(out)
    }

    fn verify_url(&self) -> Result<()> {
        let read_url = git::read_repo_url(&self.path)?;
        ensure!(
            read_url.contains(&self.url),
            format!(
                "path to repository reads-url {} does not math given url = {}",
                read_url, self.url
            )
        );
        Ok(())
    }

    pub fn last_commit(&self) -> anyhow::Result<Commit> {
        git::get_last_commit(&self.path, &self.branch)
    }

    pub fn read_commit_from_hash(&self, hash: &str) -> Result<Commit> {
        git::complete_commit_from_hash(&self.path, hash.to_string())
    }

    fn commits_between(
        &self,
        after_date: Option<&Date>,
        before_date: Option<&Date>,
    ) -> anyhow::Result<Vec<Commit>> {
        git::get_commits_since(&self.path, &self.branch, after_date, before_date)
    }

    pub fn last_commit_at_date(&self, date: &Date) -> anyhow::Result<Option<Commit>> {
        let after = date.clone() - Days::new(1);
        let commits = self.commits_between(Some(&after), Some(&date))?;
        if commits.len() == 0 {
            return Ok(None);
        }
        Ok(Some(commits.iter().next().unwrap().clone()))
    }

    pub fn collect_monthly_commits(
        &self,
        after_date: Option<&Date>,
        before_date: Option<&Date>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Commit>::new();
        for c in Self::commits_between(&self, after_date, before_date)?.drain(..) {
            if let Some(last) = commits.last() {
                let d0 = format_date(&c.date()).split_at(7).0.to_string();
                let d1 = format_date(&last.date()).split_at(7).0.to_string();
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

    pub fn checkout(&self, commit: &Commit) -> Result<RepositoryState> {
        git::was_commit_merged_to_branch(&self.path, &self.branch, commit.hash())?;
        let hash = git::read_current_commit(&self.path)?;
        if &hash != commit.hash() {
            git::checkout_commit(&self.path, commit.hash())?;
            let current = git::read_current_commit(&self.path)?;
            ensure!(&current == commit.hash(), "checkout failed");
        }
        Ok(RepositoryState {
            path: self.path.clone(),
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn branch(&self) -> &str {
        self.branch.as_ref()
    }

    pub fn pull(&mut self) -> Result<String> {
        git::pull(&self.path)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// This is the repository at the current checked out commit.
pub struct RepositoryState {
    /// For nicer folder and results identifiers.
    path: PathBuf,
}

impl RepositoryState {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
