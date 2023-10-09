use std::path::PathBuf;

use anyhow::ensure;
use log::{debug, info, trace};
use osimperf_lib::git::Date;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use anyhow::Result;
use osimperf_lib::common::git;
use osimperf_lib::common::git::Commit;

use crate::context::Ctxt;

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

    fn commits_between(
        &self,
        after_date: Option<&Date>,
        before_date: Option<&Date>,
    ) -> anyhow::Result<Vec<Commit>> {
        println!("after = {:?}", after_date);
        let after = after_date
            .map(|d| d.format("%Y_%m_%d").to_string())
            .unwrap();
        println!("after = {:?}", after);
        let before = before_date
            .map(|d| d.format("%Y_%m_%d").to_string())
            .unwrap();
        git::get_commits_since(&self.path, &self.branch, Some(&after), Some(&before))
    }

    pub fn collect_monthly_commits(
        &self,
        after_date: Option<&Date>,
        before_date: Option<&Date>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Commit>::new();
        for c in Self::commits_between(&self, after_date, before_date)?.drain(..) {
            info!("c = {:#?}", c);
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

    pub fn checkout(&self, commit: &Commit) -> Result<RepositoryState> {
        git::was_commit_merged_to_branch(&self.path, &self.branch, &commit.hash)?;
        let hash = git::read_current_commit(&self.path)?;
        if &hash != &commit.hash {
            git::checkout_commit(&self.path, &commit.hash)?;
            ensure!(
                git::read_current_commit(&self.path)? == commit.hash,
                "checkout failed"
            );
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
