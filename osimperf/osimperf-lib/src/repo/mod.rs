pub mod git;
mod commit;
pub use commit::*;

use std::path::PathBuf;

pub struct RepositoryPath {
    path: PathBuf,
    owner: String,
    project: String,
}

impl RepositoryPath {
    fn get_commits_since(&self, date: Date) -> Vec<Commit> {
        todo!()
    }

    fn get_commits_between(&self, after: Date, until: Date) -> Vec<Commit> {
        todo!()
    }
}

pub struct Repository {
    path: PathBuf,
    owner: String,
    project: String,
    branch: String,
    commit: Commit,
}

impl  Repository {

    pub fn switch(&mut self, branch: &str) {todo!()}

    pub fn checkout(&mut self, commit: &Commit) {todo!()}

}
