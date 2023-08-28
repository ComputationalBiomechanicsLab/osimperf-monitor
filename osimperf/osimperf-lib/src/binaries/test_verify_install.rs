use anyhow::Result;
use env_logger::Env;
use log::{info, trace, debug};
use osimperf_lib::{OPENSIM_CORE_URL, git};
use osimperf_lib::{Archive, Folder, Home, OSimCoreCmakeConfig, RepositoryPath};

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting OSimPerf-compiler.");

    let home = Home::new("/home/pep/opensim/osimperf-monitor")?;
    let archive = Archive::new("/home/pep/opensim/osimperf-monitor/archive")?;

    let compile_flags = OSimCoreCmakeConfig::default();

    info!("compile flags = {:#?}", compile_flags);

    let mut repo = RepositoryPath {
        path: home.path()?.join("software/opensim-core"),
        url: OPENSIM_CORE_URL.to_string(),
        branch: "main".to_string(),
        name: "opensim-core".to_string(),
    }
    .to_repo()?;

    let after = Some("2020-01-01");
    let before = Some("2023-05-20").filter(|_| false);

    info!("{:#?}", repo);

    let commits = repo.collect_daily_commits(after, before)?;

    for c in commits.iter() {
        repo.checkout(c)?;
        trace!("Checking {:?}", repo.install_folder(&archive)?);
        if repo.verify_installation(&archive, Some(c))? {
            info!("Verified {:?}", git::date_of_commit(&repo.path, c));
        }
        debug!("FAILED {:?}", git::date_of_commit(&repo.path, c));
    }

    Ok(())
}
