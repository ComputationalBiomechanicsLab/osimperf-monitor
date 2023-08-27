pub mod git;
mod hash;
pub use hash::Hash;

use anyhow::{ensure, Context, Result};
use log::{debug, info, trace, warn};
use std::path::PathBuf;

use crate::{git::date_of_commit, Archive, Folder, ResultsFolder, Command, CommandTrait};

pub static OPENSIM_CORE_URL: &str = "https://github.com/opensim-org/opensim-core.git";
pub static BIO_LAB_URL: &str = "git@github.com:ComputationalBiomechanicsLab/osimperf-monitor.git";

pub struct RepositoryPath {
    pub name: String, // e.g. opensim-core, biolab, testbranch,
    pub path: PathBuf,
    pub url: String,    // e.g. OPENSIM_CORE_URL
    pub branch: String, // e.g. main
}

impl RepositoryPath {
    pub fn to_repo(self) -> Result<Repository> {
        Repository::new(self)
    }
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
    name: String,
    path: PathBuf,
    url: String,
    branch: String,
    hash: String,
}

impl Repository {
    pub fn new(repo: RepositoryPath) -> Result<Self> {
        let read_url = git::read_repo_url(&repo.path)?;
        ensure!(
            read_url == repo.url,
            format!(
                "path to repository reads-url {} does not math given url = {}",
                read_url, repo.url
            )
        );
        warn!("Switching {:?} to branch {}", repo.path, repo.branch);
        git::switch_branch(&repo.path, &repo.branch)?;
        let out = Self {
            hash: git::read_current_commit(&repo.path)?,
            name: repo.name,
            branch: repo.branch,
            path: repo.path,
            url: repo.url,
        };
        out.verify()?;
        Ok(out)
    }

    pub fn verify(&self) -> Result<()> {
        let read_url = git::read_repo_url(&self.path)?;
        ensure!(
            read_url == self.url,
            format!(
                "path to repository reads-url {} does not math given url = {}",
                read_url, self.url
            )
        );
        let read_hash = git::read_current_commit(&self.path)?;
        ensure!(
            read_hash == self.hash,
            format!(
                "repository checked out commit {} does not match expected {}",
                read_hash, self.hash
            )
        );
        let was_merged_to = git::commit_merged_to(&self.path, &self.hash)?;
        ensure!(
            was_merged_to.lines().any(|line| line.trim() == self.branch),
            format!(
                "repository switched branch without us knowing: {:?}, not equal to {:?}",
                was_merged_to, self.branch
            )
        );
        Ok(())
    }

    pub fn checkout(&mut self, hash: impl ToString) -> Result<()> {
        self.verify()?;
        let hash = hash.to_string();
        if &hash != &self.hash {
            git::checkout_commit(&self.path, &hash)?;
            self.hash = hash;
            self.verify().context("checkout failed");
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

    fn commits_between(
        &self,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> Result<Vec<String>> {
        self.verify()?;
        git::get_commits_since(&self.path, &self.branch, after_date, before_date)
    }

    pub fn collect_daily_commits(
        &self,
        after_date: Option<&str>,
        before_date: Option<&str>,
    ) -> Result<Vec<String>> {
        self.verify()?;
        let mut commits = Vec::<String>::new();
        for (i, c) in self
            .commits_between(after_date, before_date)?
            .drain(..)
            .enumerate()
        {
            if let Some(previous) = commits
                .last()
                .map(|c| git::date_of_commit(&self.path, c))
                .transpose()?
            {
                let date = git::date_of_commit(&self.path, &c)?;
                trace!("comparing date {:?} to previous date {:?}", date, previous);
                if date == previous {
                    debug!("Skipping duplicate {:?}", c);
                    continue;
                }
            }
            info!("Last commit of the day: {:#?}", c);
            commits.push(c);
        }
        Ok(commits)
    }

    fn subfolder_name(&self) -> Result<String> {
        self.verify()?;
        Ok(format!(
            "{}-{}-{}-{}",
            self.name,
            self.branch,
            git::date_of_commit(&self.path, &self.hash)?,
            self.hash,
        ))
    }

    // Install folder: archive/name-branch-date-hash
    pub fn install_folder(&self, archive: &Archive) -> Result<PathBuf> {
        Ok(archive.path()?.join(self.subfolder_name()?))
    }

    pub fn results_folder(&self, folder: &ResultsFolder) -> Result<PathBuf> {
        Ok(folder.path()?.join(self.subfolder_name()?))
    }

    fn short_hash(&self) -> Result<&str> {
        const short_hash_len: usize = 9;
        let short = self.hash.split_at(short_hash_len).0;
        Some(short)
            .filter(|s| s.len() == short_hash_len)
            .with_context(|| format!("error taking short hash of {:?}", &self))
    }

    pub fn verify_installation(&self, archive: &Archive) -> Result<bool> {
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
