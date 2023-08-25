mod commit;
pub mod git;
pub use commit::*;

use anyhow::{ensure, Result};
use std::path::PathBuf;

pub struct RepositoryPath {
    pub path: PathBuf,
    // owner: String,
    // project: String,
    pub url: String,
}

// impl RepositoryPath {
//     fn get_commits_since(&self, date: Date) -> Vec<Commit> {
//         todo!()
//     }

//     fn get_commits_between(&self, after: Date, until: Date) -> Vec<Commit> {
//         todo!()
//     }
// }

#[derive(Clone, Debug)]
pub struct Repository {
    path: PathBuf,
    url: String,
    branch: String,
    commit: String,
}

impl Repository {
    pub fn new(repo: RepositoryPath) -> Result<Self> {
        git::verify_repository(&repo.path, &repo.url)?;
        Ok(Self {
            branch: git::read_current_branch(&repo.path)?,
            commit: git::read_current_commit(&repo.path)?,
            path: repo.path,
            url: repo.url,
        })
    }

    pub fn verify(&self) -> Result<()> {
        git::verify_repository(&self.path, &self.url)?;
        ensure!(
            git::read_current_branch(&self.path)? == self.branch,
            "repository switched branch without us knowing"
        );
        ensure!(
            git::read_current_commit(&self.path)? == self.commit,
            "repository checked out commits without us knowing"
        );
        Ok(())
    }

    pub fn checkout(&mut self, commit: &Commit) -> Result<()> {
        self.verify()?;
        if &commit.hash() != &self.commit {
            git::switch_branch(&self.path, commit.branch())?;
            git::checkout_commit(&self.path, commit.hash())?;
            self.branch = String::from(commit.branch());
            self.commit = String::from(commit.hash());
            self.verify()?;
        }
        Ok(())
    }

    pub fn path(&self) -> Result<&PathBuf> {
        self.verify()?;
        Ok(&self.path)
    }

    pub fn branch(&self) -> Result<&str> {
        self.verify()?;
        Ok(self.branch.as_ref())
    }
}
