pub mod table;
mod config;
mod run_cmds;
mod node;
mod result;

use run_cmds::*;
pub use config::BenchTestSetup;
pub use node::TestNode;
pub use result::BenchTestResult;

// use std::{
//     fs::{self, create_dir, File},
//     path::{Path, PathBuf},
// };

// use crate::{find_file_by_name, read_config, write_config, Command, CompilationNode, ResultsFolder, Id, BuildFolder, erase_folder};
// use anyhow::{anyhow, Context, Result};
// use log::{debug, info};
// use serde::{Deserialize, Serialize};

//pub struct BenchTestNode {
//    test: BenchTestSetup,
//    compiler: CompilationNode,
//    path_to_node: PathBuf,
//}

//impl BenchTestNode {
//    pub fn id(&self, results: &ResultsFolder) -> TestId {
//        TestId::new(
//            self.compiler.id(),
//        &self.test.name,
//            results,
//        )
//    }

//    pub fn run(&self) -> BenchTestResult {
//        // File layout:
//        let id = self.id();
//        let root_dir = id.root_dir();
//        let setup_dir = id.setup_dir();
//        let output_dir = id.output_dir();
//        let install = self.compiler.id().path_str();

//        erase_folder(&root_dir);
//        erase_folder(&output_dir);

//        for i in 0..self.test.cmd.len() {
//            let mut cmd = self.test.cmd[i].clone();

//            // Replace environmental variables:
//            cmd.add_env(ENV_VAR_TEST_OUTPUT, output_dir.to_str().unwrap());
//            cmd.add_env(ENV_VAR_TEST_ROOT, root_dir.to_str().unwrap());
//            cmd.add_env(ENV_VAR_TEST_SETUP, setup_dir.to_str().unwrap());

//            cmd.add_env(ENV_VAR_INSTALL, self.compiler.id().path_str());
//            cmd.add_env(
//                "PATH",
//                format!(
//                    "/bin:{}:{}/lib:{}/include",
//                    install, install, install
//                ),
//            );
//            cmd.add_env(
//                "LD_LIBRARY_PATH",
//                format!(
//                    "/bin:{}:{}/lib:{}/include",
//                    install, install, install
//                ),
//            );

//            let last= i + 1 == self.test.cmd.len();
//            if last {
//                match cmd.run()? {
//                    Ok(duration) => {
//                        info!(
//                            "Test completed in {} seconds: {:?}, {:?}",
//                            duration, commit, setup
//                        );
//                        debug!("     Root folder: {:?}", test_root_dir);
//                        debug!("     intssll folder: {:?}", test_install_dir_str);
//                        return Ok(BenchTestResult {
//                            duration,
//                            status: true,
//                            iteration: 1,
//                        });
//                    }
//                    Err(err) => {
//                        info!("Test failed: {:?}, {:?}", commit, setup);
//                        info!("    with error: {:?}", err);
//                        return Ok(BenchTestResult {
//                            duration: f64::NAN,
//                            status: false,
//                            iteration: 0,
//                        });
//                    }
//                }
//            }

//            cmd.run_extend_log(&mut log_stdout, &mut log_stderr)?;
//        }
//        Err(anyhow!("Not possible to end up here!"))
//    }
//}

//#[derive(Deserialize, Serialize, Debug, Clone)]
//pub struct BenchTestResult {
//    pub duration: f64,
//    pub iteration: usize,
//    pub status: bool,
//}

//pub fn run_test(
//    node: &CompilationNode,
//    setup: &BenchTestSetup,
//    results_folder: &ResultsFolder,
//) -> Result<BenchTestResult> {

//    // Set the file layout:
//    let test_root_dir = results_folder.join(node.id().path());

//    commit
//        .get_results_folder(folders)
//        .join(Path::new(&setup.name))
//    let test_root_dir = test_environment_root_dir(folders, setup, commit);
//    let test_output_dir = test_root_dir.join("output");
//    let test_install_dir = commit.get_archive_folder(folders).join("install");
//    let mut log_stdout =
//        File::open(test_root_dir.join("log_stdout")).context("unable to open stdout log")?;
//    let mut log_stderr =
//        File::open(test_root_dir.join("log_stderr")).context("unable to open stderr log")?;
//    // let test_setup_dir = setup.path.clone();

//    if !test_root_dir.exists() {
//        create_dir(&test_root_dir)?;
//        create_dir(&test_output_dir)?;
//    }

//    let test_install_dir_str = test_install_dir.to_str().unwrap();

//    for i in 0..setup.cmd.len() {
//        // let mut cmd = Command::new("env");
//        // cmd.add_arg("-C");
//        // cmd.add_arg(test_root_dir.to_str().unwrap());
//        // cmd.add_arg(&setup.cmd[i].cmd);
//        // cmd.add_args(setup.cmd[i].args.iter());

//        // TODO copy original env vars over as well.
//        let mut cmd = setup.cmd[i].clone();

//        // Replace environmental variables:
//        cmd.add_env(ENV_VAR_TEST_OUTPUT, test_output_dir.to_str().unwrap());
//        cmd.add_env(ENV_VAR_TEST_ROOT, test_root_dir.to_str().unwrap());
//        cmd.add_env(ENV_VAR_TEST_SETUP, test_setup_dir.to_str().unwrap());
//        cmd.add_env(ENV_VAR_INSTALL, test_install_dir_str);
//        cmd.add_env(
//            "PATH",
//            format!(
//                "/bin:{}:{}/lib:{}/include",
//                test_install_dir_str, test_install_dir_str, test_install_dir_str
//            ),
//        );
//        cmd.add_env(
//            "LD_LIBRARY_PATH",
//            format!(
//                "/bin:{}:{}/lib:{}/include",
//                test_install_dir_str, test_install_dir_str, test_install_dir_str
//            ),
//        );

//        if i + 1 == setup.cmd.len() {
//            match cmd.run_extend_log(&mut log_stdout, &mut log_stderr) {
//                Ok(duration) => {
//                    info!(
//                        "Test completed in {} seconds: {:?}, {:?}",
//                        duration, commit, setup
//                    );
//                    debug!("     Root folder: {:?}", test_root_dir);
//                    debug!("     intssll folder: {:?}", test_install_dir_str);
//                    return Ok(BenchTestResult {
//                        duration,
//                        status: true,
//                        iteration: 1,
//                    });
//                }
//                Err(err) => {
//                    info!("Test failed: {:?}, {:?}", commit, setup);
//                    info!("    with error: {:?}", err);
//                    return Ok(BenchTestResult {
//                        duration: f64::NAN,
//                        status: false,
//                        iteration: 0,
//                    });
//                }
//            }
//        }

//        cmd.run_extend_log(&mut log_stdout, &mut log_stderr)?;
//    }
//    Err(anyhow!("Not possible to end up here!"))
//}

///// Returns the directory that is used as root during test execution.
///// Something like:
///// osimperf-home/results/results-DATE-COMMIT/TEST_NAME
//pub fn test_environment_root_dir(
//    folders: &Folders,
//    setup: &BenchTestSetup,
//    commit: &Commit,
//) -> PathBuf {
//    commit
//        .get_results_folder(folders)
//        .join(Path::new(&setup.name))
//}

///// Path to final [BenchTestResult] of this test.
/////
///// Something like
///// `SIMPERF_HOME/results/results-DATE-HASH/TEST_NAME/osimperf-result.data`
//fn test_result_output_file_path(
//    folders: &Folders,
//    setup: &BenchTestSetup,
//    commit: &Commit,
//) -> PathBuf {
//    test_environment_root_dir(folders, setup, commit).join("osimperf-results.data")
//}

//pub fn read_test_result(
//    folders: &Folders,
//    setup: &BenchTestSetup,
//    commit: &Commit,
//) -> Result<Option<BenchTestResult>> {
//    let path = test_result_output_file_path(folders, setup, commit);
//    if path.exists() {
//        return Some(read_config::<BenchTestResult>(Path::new(&path))).transpose();
//    }
//    Ok(None)
//}

///// Write test result to file.
//pub fn update_test_result(
//    folders: &Folders,
//    setup: &BenchTestSetup,
//    commit: &Commit,
//    mut result: BenchTestResult,
//) -> Result<()> {
//    let path = test_result_output_file_path(folders, setup, commit);
//    if let Some(prev_result) = read_test_result(folders, setup, commit)? {
//        if prev_result.status {
//            result.iteration += prev_result.iteration;
//            result.duration += prev_result.duration;
//            result.duration *= 0.5;
//        }
//    }
//    write_config::<BenchTestResult>(Path::new(&path), &result)?;
//    Ok(())
//}

//pub struct ResultsNode {
//    compile_node: PathBuf,
//}
