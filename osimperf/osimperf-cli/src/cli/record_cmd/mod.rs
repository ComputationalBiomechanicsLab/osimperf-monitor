use crate::{
    record::{BenchTestResult, BenchTestSetup, TestNode},
    CMakeCommands, Commit, Ctxt, Date, EnvVars, FileBackedStruct, Repository,
};
use anyhow::{anyhow, ensure, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use rand::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct RecordCommand {
    /// Path to archive directory.
    #[arg(long)]
    archive: Option<PathBuf>,

    /// Path to results directory.
    #[arg(long)]
    results: Option<PathBuf>,

    /// Path to test cases directory.
    #[arg(long)]
    tests: Option<PathBuf>,

    /// Path to models directory.
    #[arg(long)]
    models: Option<PathBuf>,

    /// Number of test iterations.
    #[arg(long, short, default_value_t = 1)]
    iter: usize,
}

impl RecordCommand {
    fn get_context(&self) -> Result<Ctxt> {
        let mut context = Ctxt::default();
        context.set_archive(self.archive.clone())?;
        context.set_results(self.results.clone())?;
        context.set_tests(self.tests.clone())?;
        context.set_models(self.models.clone())?;
        Ok(context)
    }

    pub fn run(&self) -> Result<()> {
        info!("Starting OSimPerf record command.");
        let context = self.get_context()?;

        let opensim_installs = crate::install::CompilationNode::collect_archived(&context)?;
        let test_setups = BenchTestSetup::find_all(context.tests())?;

        let mut rng = rand::thread_rng();
        for node in opensim_installs.iter() {
            let mut tests = Vec::new();
            for setup in test_setups.iter() {
                // Creating the test node also sets up the context.
                let id = node.id();
                let path_to_result =
                    BenchTestResult::default_path_to_file(&context, &id, &setup.name);
                let env_vars = EnvVars {
                    opensim_install: Some(node.path_to_self(&context).parent().unwrap().to_owned()),
                    models: Some(context.models().to_owned()),
                    test_setup: Some(setup.test_setup_file.parent().unwrap().to_owned()),
                    test_context: Some(path_to_result.parent().unwrap().to_owned()),
                    ..Default::default()
                };
                if let Some(test) =
                    TestNode::new(&setup, &node, &context, path_to_result, env_vars.clone())?
                {
                    tests.push(test);
                }
            }

            for test in tests.iter_mut() {
                test.pre_benchmark_setup()?;
            }

            for _ in 0..self.iter {
                tests.shuffle(&mut rng);
                for test in tests.iter_mut() {
                    info!("running = {}", test.config.name);
                    test.run()?;
                }
            }

            for test in tests.iter_mut() {
                info!("grinding = {}", test.config.name);
                test.grind()?;
            }

            for test in tests.drain(..) {
                let name: String = test.config.name.clone();
                let result = test.post_benchmark_teardown()?;
                info!("{} {:#?}", name, result);
            }
        }
        Ok(())
    }
}
