use crate::git;
use crate::Repository;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Commit {
    hash: String,
    date: String,
    branch: String,
}

impl Commit {
    /// Private interface.
    fn from_tuple(tuple: (String, String, String)) -> Self {
        Self {
            hash: tuple.0,
            date: tuple.1,
            branch: tuple.2,
        }
    }

    pub fn commits_between(
        repo: &Repository,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> Result<Vec<Self>> {
        Ok(
            git::get_commits_since(&repo.path()?, &repo.branch()?, after_date, before_date)?
                .drain(..)
                .map(|tuple| Self::from_tuple(tuple))
                .collect::<Vec<Self>>(),
        )
    }

    pub fn last_commit(repo: &Repository) -> Result<Self> {
        Ok(Self::commits_between(repo, None, None)?
            .drain(..)
            .next()
            .unwrap())
    }

    pub fn hash(&self) -> &str {
        self.hash.as_ref()
    }

    pub fn date(&self) -> &str {
        self.date.as_ref()
    }

    pub fn branch(&self) -> &str {
        self.branch.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct Date {
    yyyy_mm_dd: String,
}

impl Date {
    pub fn new(yyyy_mm_dd: &str) -> Self {
        Self {
            yyyy_mm_dd: String::from(yyyy_mm_dd),
        }
    }
    pub fn to_str(&self) -> &str {
        &self.yyyy_mm_dd
    }
}

#[derive(Clone, Debug)]
pub struct Hash {
    value: String,
}

impl Hash {
    pub fn new(hash: &str) -> Self {
        todo!()
    }

    pub fn str(&self) -> &str {
        todo!()
    }

    pub fn short(&self) -> &str {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    value: String,
}

impl Branch {
    pub fn new(branch: String) {
        todo!()
    }

    pub fn get_str(&self) -> &str {
        todo!()
    }
}
