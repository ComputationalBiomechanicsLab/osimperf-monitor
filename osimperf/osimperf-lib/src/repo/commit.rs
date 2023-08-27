use std::path::PathBuf;

use crate::{git, Archive, Command, CommandTrait, Folder, Repository, ResultsFolder};
use anyhow::{ensure, Context, Result};
use log::{debug, info, trace, warn};

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

    fn commits_between(
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

    pub fn collect_daily_commits(
        repo: &Repository,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> Result<Vec<Self>> {
        let mut commits = Vec::<Self>::new();
        for (i, c) in Self::commits_between(repo, after_date, before_date)?
            .drain(..)
            .enumerate()
        {
            if let Some(last) = commits.last() {
                trace!(
                    "comparing {:?} to {:?}",
                    c.date(),
                    last.date(),
                );
                if c.date() == last.date() {
                    debug!("Skipping duplicate {:?}", c);
                    continue;
                }
            }
            info!("Last commit of the day: {:#?}", c);
            commits.push(c);
        }
        Ok(commits)
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

    pub fn short_hash(&self) -> Result<&str> {
        const short_hash_len: usize = 9;
        let short = self.hash.split_at(short_hash_len).0;
        Some(short)
            .filter(|s| s.len() == short_hash_len)
            .with_context(|| format!("error taking short hash of {:?}", &self))
    }

    pub fn date(&self) -> &str {
        self.date.as_ref()
    }

    pub fn branch(&self) -> &str {
        self.branch.as_ref()
    }

    pub fn install_folder(&self, archive: &Archive) -> Result<PathBuf> {
        Ok(archive.path()?.join(format!(
            "opensim-core-{}-{}-{}",
            self.branch, self.date, self.hash
        )))
    }

    pub fn results_folder(&self, folder: &ResultsFolder) -> Result<PathBuf> {
        Ok(folder.path()?.join(format!(
            "results-{}-{}-{}",
            self.branch, self.date, self.hash
        )))
    }

    pub fn verify_compiled_version(&self, archive: &Archive) -> Result<bool> {
        debug!(
            "Start verification of archived opensim-cmd install: {:?}",
            &self
        );
        let opensim_cmd_path = self
            .install_folder(archive)?
            .join("opensim-core/bin/opensim-cmd");

        if !opensim_cmd_path.exists() {
            debug!("could not verify: {:?} does not exist.", opensim_cmd_path);
            return Ok(false);
        }

        let mut cmd = Command::new(opensim_cmd_path.to_str().unwrap());
        cmd.add_arg("--version");
        let output = if let Ok(res) = cmd.run_trim() {
            res
        } else {
            warn!(
                "Failed to execute {:?}:\ncmd = {:?}",
                opensim_cmd_path,
                cmd.print_command()
            );
            return Ok(false);
        };
        if output.as_str().contains(self.short_hash()?) {
            debug!("Successfully verified opensim-cmd install {:?}", &self);
            return Ok(true);
        } else {
            warn!(
                "Previously installed opensim-cmd --version {:?} does not match expected {:?}",
                output.as_str(),
                &self
            );
            return Ok(false);
        }
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
