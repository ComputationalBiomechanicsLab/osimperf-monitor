use std::path::PathBuf;

use anyhow::ensure;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::common::git;
use crate::common::git::Commit;
use crate::{Folder, Home, BIO_LAB_URL, OPENSIM_CORE_URL};
use anyhow::Result;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioLabRepositoryConfig {
    repositories: Vec<RepositoryConfig>,
}

impl BioLabRepositoryConfig {
    pub fn take(mut self, home: &Home) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        for r in self.repositories.drain(..) {
            repos.push(Repository::new_default_biolab(r, home)?);
        }
        Ok(repos)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub name: String,
    pub branch: String,
    /// Path to repository: defaults to biolab/opensimcore submodule.
    pub path: Option<PathBuf>,
    /// Defaults to CompurationalBiomechanics or opensim-core url.
    pub url: Option<String>,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            name: "opensim".to_string(),
            branch: "main".to_string(),
            path: None,
            url: None,
        }
    }
}

impl RepositoryConfig {
    pub fn take(self, home: &Home) -> Result<Repository> {
        Repository::new_default_opensim_core(self, home)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
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

    fn new_default_biolab(config: RepositoryConfig, home: &Home) -> Result<Self> {
        let out = Self {
            name: config.name,
            path: config.path.unwrap_or(
                home.path()
                    .unwrap()
                    .join("software")
                    .join("computational-biomechanics-lab"),
            ),
            url: BIO_LAB_URL.to_string(),
            branch: config.branch,
        };
        out.verify_url()?;
        Ok(out)
    }

    fn new_default_opensim_core(config: RepositoryConfig, home: &Home) -> Result<Self> {
        let out = Self {
            name: config.name,
            path: config
                .path
                .unwrap_or(home.path().unwrap().join("software").join("opensim-core")),
            url: OPENSIM_CORE_URL.to_string(),
            branch: config.branch,
        };
        out.verify_url()?;
        Ok(out)
    }

    pub fn last_commit(&self) -> anyhow::Result<Commit> {
        git::get_last_commit(&self.path, &self.branch)
    }

    fn commits_between(
        &self,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Commit>> {
        git::get_commits_since(&self.path, &self.branch, after_date, before_date)
    }

    pub fn collect_monthly_commits(
        &self,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Commit>::new();
        for c in Self::commits_between(&self, after_date, before_date)?.drain(..) {
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
        &self,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::<Commit>::new();
        for c in Self::commits_between(&self, after_date, before_date)?.drain(..) {
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
