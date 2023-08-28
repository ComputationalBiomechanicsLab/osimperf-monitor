use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use env_logger::Env;
use log::info;
use osimperf_lib::{
    compile_opensim_core, Archive, BuildFolder, Command, CommandTrait, Folder, Home,
    OSimCoreCmakeConfig, RepositoryPath,
};
use osimperf_lib::{EraseableFolder, OPENSIM_CORE_URL};

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("Starting OSimPerf-compiler.");

    for i in 0..10 {
        let res = do_main();
        sleep(Duration::from_secs(60));
    }
    do_main()?;

    Ok(())
}

fn do_main() -> Result<()> {
    let home = Home::new("/home/pep/opensim/osimperf-monitor")?;
    let build = BuildFolder::new("/home/pep/opensim/osimperf-monitor/build")?;
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

    let dates = [
        "2020-01-01",
        "2020-02-01",
        "2020-03-01",
        "2020-04-01",
        "2020-05-01",
        "2020-06-01",
        "2020-07-01",
        "2020-08-01",
        "2020-09-01",
        "2020-10-01",
        "2020-11-01",
        "2020-12-01",
        "2021-01-01",
        "2021-02-01",
        "2021-03-01",
        "2021-04-01",
        "2021-05-01",
        "2021-06-01",
        "2021-07-01",
        "2021-08-01",
        "2021-09-01",
        "2021-10-01",
        "2021-11-01",
        "2021-12-01",
        "2022-01-01",
        "2022-02-01",
        "2022-03-01",
        "2022-04-01",
        "2022-05-01",
        "2022-06-01",
        "2022-07-01",
        "2022-08-01",
        "2022-09-01",
        "2022-10-01",
        "2022-11-01",
        "2022-12-01",
        "2023-01-01",
        "2023-02-01",
        "2023-03-01",
        "2023-04-01",
        "2023-05-01",
        "2023-06-01",
        "2023-07-01",
        "2023-08-01",
        "2023-09-01",
    ];

    let mut commits = Vec::new();
    for i in 1..dates.len() {
        let cs = repo.collect_daily_commits(Some(dates[i - 1]), Some(dates[i]))?;
        if let Some(c) = cs.first() {
            commits.push(c.clone());
        }
    }

    let mut result = Vec::new();
    for c in commits.iter() {
        repo.checkout(c)?;
        if repo.verify_installation(&archive, Some(c))? {
            info!("VERIFIED INSTALLATION of {:?}", c);
            continue;
        }
        info!("start compiling {:#?}", c);
        build.erase_folder()?;
        let res = if compile_opensim_core(&repo, &home, &archive, &build, &compile_flags).is_ok() {
            "success"
        } else {
            let mut cmd = Command::new("mv");
            let mut install_folder = repo.install_folder(&archive)?.to_str().unwrap().to_string();
            cmd.add_arg(format!("{}", install_folder));
            install_folder.push_str("-failed");
            cmd.add_arg(format!("{}", install_folder));
            cmd.run()?;
            "failed"
        };
        result.push(format!(
            "{},{}",
            repo.install_folder(&archive)?.to_str().unwrap(),
            res
        ));
        info!("{:?}", result);
    }

    // Check if file gets overwrittern.
    // let mut file = File::open(path).context(format!(
    //     "failed to open file for writing stderr logs at path = {:?}",
    //     path
    // ))?;
    // file.write_all(&self.output.stderr)?;

    // let OSimCoreCmakeConfig {
    // };

    Ok(())
}
