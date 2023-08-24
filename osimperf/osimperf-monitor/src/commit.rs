use crate::folders::{ARCHIVE_TOUCH_FILE, RESULTS_TOUCH_FILE};
use crate::{Command, Folders};
use anyhow::{ensure, Context, Result};
use log::debug;
use log::trace;
use std::{
    fs::{self, remove_dir_all, OpenOptions},
    path::{Path, PathBuf},
};

pub fn collect_last_daily_commit(folders: &Folders, start_date: &String) -> Result<Vec<Commit>> {
    let mut cmd = Command::new(format!(
        "{}/{}",
        folders.scripts.to_str().unwrap(),
        "osimperf_get_commits_since.sh"
    ));
    cmd.add_arg(start_date);
    cmd.add_arg(folders.opensim_core.to_str().unwrap());
    trace!("Setting up command for collecting commits, cmd=\n{:?}", cmd);

    let mut commits = Vec::<Commit>::new();
    for (i, c) in Commit::parse(
        cmd.run()
            .context("failed to run command for collecting commits")?,
    )
    .context("Failed to parse output of command that collected the commits.")?
    .iter()
    .enumerate()
    {
        if i > 0 {
            if commits.last().unwrap().date == c.date {
                trace!("Duplicate date: skipping {:?}", c);
                continue;
            }
        }
        trace!("Keeping: {:?}", c);
        commits.push(c.clone());
    }
    Ok(commits)
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub date: String,
}

impl Commit {
    fn parse(string: String) -> Result<Vec<Self>> {
        let mut commits = Vec::new();
        for (i, line) in string.lines().enumerate() {
            let mut it = line.split(",");
            commits.push(Commit {
                date: String::from(
                    it.next()
                        .context("failed to parse line {i}")
                        .context("failed to parse date")?,
                ),
                hash: String::from(
                    it.next()
                        .context("failed to parse line {i}")
                        .context("failed to parse hash")?,
                ),
            });
        }

        Ok(commits)
    }

    /// The folder containing the install of this commit.
    ///
    /// Something like
    /// SIMPERF_HOME/archive/opensim-core-DATE-HASH
    pub fn get_archive_folder(&self, folders: &Folders) -> PathBuf {
        folders.archive.join(Path::new(&format!(
            "opensim-core-{}-{}",
            self.date, self.hash
        )))
    }

    /// The folder containing the install of this commit.
    ///
    /// Something like
    /// SIMPERF_HOME/results/results-DATE-HASH
    pub fn get_results_folder(&self, folders: &Folders) -> PathBuf {
        folders
            .results
            .join(Path::new(&format!("results-{}-{}", self.date, self.hash)))
    }

    pub fn create_results_dir(&self, folders: &Folders) -> Result<()> {
        let dir = self.get_results_folder(folders);
        fs::create_dir(&dir).context(format!("Failed to create results directory: {:?}", dir))?;
        touch(dir.join(Path::new(&RESULTS_TOUCH_FILE)).as_path())?;
        Ok(())
    }

    pub fn create_archive_dir(&self, folders: &Folders) -> Result<()> {
        let dir = self.get_archive_folder(folders);
        fs::create_dir(&dir).context(format!("Failed to create archive directory: {:?}", dir))?;
        touch(dir.join(Path::new(&ARCHIVE_TOUCH_FILE)).as_path())?;
        Ok(())
    }

    pub fn remove_archive_dir(&self, folders: &Folders) -> Result<()> {
        let dir = self.get_archive_folder(folders);
        debug!("Removing archive: {:?}", dir);
        if !dir.exists() {
            trace!("Nothing to do: archive did not exist");
            return Ok(());
        }
        let verification_file = dir.join(ARCHIVE_TOUCH_FILE);
        ensure!(
            verification_file.exists(),
            format!(
                "Tried to remove directory {:?}, but doesnt look like an archive directory",
                dir
            )
        );
        trace!(
            "Verified that {:?} exists -> ok to remove archive.",
            verification_file
        );
        remove_dir_all(&dir).context(format!("Failed to remove archive directory: {:?}", dir))?;
        Ok(())
    }

    pub fn remove_results_dir(&self, folders: &Folders) -> Result<()> {
        let dir = self.get_results_folder(folders);
        if !dir.exists() {
            return Ok(());
        }
        ensure!(
            dir.join(Path::new(&RESULTS_TOUCH_FILE)).as_path().exists(),
            format!(
                "Tried to remove directory {:?}, but doesnt look like a results directory",
                dir
            )
        );
        remove_dir_all(&dir).context(format!("Failed to remove results directory: {:?}", dir))?;
        Ok(())
    }

    pub fn archive_exists(&self, folders: &Folders) -> bool {
        Path::new(&self.get_archive_folder(folders)).exists()
    }

    pub fn verify_compiled_version(&self, folders: &Folders) -> Result<bool> {
        debug!("Start verification of archived opensim-cmd version.");
        if !self.archive_exists(folders) {
            debug!("Failed to verify: archive does not exist: {:?}", &self);
            return Ok(false);
        }

        let mut cmd = Command::new(format!(
            "{}/install/bin/opensim-cmd",
            self.get_archive_folder(folders).to_str().unwrap()
        ));
        cmd.add_arg("--version");
        let output = if let Ok(res) = cmd.run() {
            res
        } else {
            debug!("Failed to execute command: {:?} of {:?}", cmd, &self);
            return Ok(false);
        };

        trace!("Compare hash to output of 'opensim-cmd --version'");
        let short_hash_len = 9;
        let short = self.hash.split_at(short_hash_len).0;
        trace!("    Short hash = {:?}", short);
        trace!("    Cmd output = {:?}", output);
        ensure!(short.len() == short_hash_len, "error taking short hash");
        debug!("Successfully verified compiled_version");

        Ok(output.as_str().contains(short))
    }
}

// A simple implementation of `% touch path` (ignores existing files)
fn touch(path: &Path) -> Result<()> {
    OpenOptions::new().create(true).write(true).open(path)?;
    Ok(())
}
