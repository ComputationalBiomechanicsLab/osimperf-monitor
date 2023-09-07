use super::{
    run_cmds::{run_post_test_cmds, run_pre_test_cmds, run_test_bench_cmd, FileEnvVars},
    BenchTestResult, BenchTestSetup,
};
use crate::{CommandOutput, CompilationNode, Folder, Home, NodeFile, ResultsFolder};
use anyhow::Result;
use log::trace;
use std::hash::{Hash, Hasher};
use std::{collections::hash_map::DefaultHasher, path::PathBuf};

// TODO rename to TestNodeRunner
#[derive(Debug)]
pub struct TestNode<'a, 'b, 'c, 'd> {
    test: &'a BenchTestSetup,
    compiler: &'b CompilationNode,
    home: &'c Home,
    results: &'d ResultsFolder,
    result: BenchTestResult,
    last_command_output: Option<CommandOutput>,
    warm_start_buffer: usize,
}

impl<'a, 'b, 'c, 'd> TestNode<'a, 'b, 'c, 'd> {
    fn new_helper(
        test: &'a BenchTestSetup,
        compiler: &'b CompilationNode,
        home: &'c Home,
        results: &'d ResultsFolder,
        warm_start_buffer: usize,
    ) -> Result<Self> {
        Ok(Self {
            test,
            compiler,
            home,
            results,
            result: BenchTestResult::new(results, &compiler.id(), &test.name)?,
            last_command_output: None,
            warm_start_buffer,
        })
    }

    pub fn new(
        test: &'a BenchTestSetup,
        compiler: &'b CompilationNode,
        home: &'c Home,
        results: &'d ResultsFolder,
        warm_start_buffer: usize,
    ) -> Result<Option<Self>> {
        let mut out = Self::new_helper(test, compiler, home, results, warm_start_buffer)?;
        if !out.compiler.is_done() {
            return Ok(None);
        }
        out.pre_benchmark_setup()?;
        Ok(Some(out))
    }

    fn path_to_node(&self) -> PathBuf {
        self.result.path_to_self()
    }

    pub fn should_run(&self, max_iter: usize, max_failures: usize) -> bool {
        self.compiler.is_done() && self.result.should_run(max_iter, max_failures)
    }

    pub fn env_vars(&self) -> Result<FileEnvVars> {
        Ok(FileEnvVars {
            install: self.compiler.id().path(),
            output: self.path_to_node().parent().unwrap().to_path_buf(),
            root: self.path_to_node().parent().unwrap().join("context"),
            home: self.home.path()?.to_path_buf(),
        })
    }

    fn pre_benchmark_setup(&mut self) -> Result<()> {
        let env_vars = self.env_vars()?;

        let setup_dir = self.test.test_setup_file.parent().unwrap();
        let out = run_pre_test_cmds(
            &self.test.pre_benchmark_cmds,
            &env_vars,
            setup_dir,
            &self.test.model_files,
        );

        // Add the hash of the current bench config.
        let hash = compute_test_node_hash(&self.test, &self.compiler);
        self.result.update_hash(hash);

        if out.is_err() {
            self.result.update_result(None);
        }

        self.result.try_write()?;

        Ok(())
    }

    fn post_benchmark_teardown(&mut self) -> Result<()> {
        trace!("Run post benchmark teardown");
        let env_vars = self.env_vars()?;

        let out = run_post_test_cmds(
            &self.test.post_benchmark_cmds,
            &env_vars,
            &self.last_command_output,
        );

        // Set to failure if post commands failed.
        if out.is_err() {
            self.result.update_result(None);
        }

        self.result.try_write()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<&BenchTestResult> {
        let env_vars = self.env_vars()?;

        self.last_command_output = Some(run_test_bench_cmd(&self.test.benchmark_cmd, &env_vars)?);

        self.warm_start_buffer = self.warm_start_buffer.saturating_sub(1);
        if self.warm_start_buffer == 0 {
            self.result.update_result(
                self.last_command_output
                    .as_ref()
                    .filter(|x| x.success())
                    .map(|x| x.duration),
            );
        }

        Ok(&self.result)
    }

    pub fn try_write(&mut self) -> Result<()> {
        trace!("Writing test results to file: {:?}", &self.result);
        self.result.try_write()
    }
}

fn compute_test_node_hash(test: &BenchTestSetup, compiler: &CompilationNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    test.hash(&mut hasher);
    compiler.hash(&mut hasher);
    hasher.finish()
}

impl<'a, 'b, 'c, 'd> Drop for TestNode<'a, 'b, 'c, 'd> {
    fn drop(&mut self) {
        // Write to the backing file.
        self.post_benchmark_teardown()
            .expect("Failed to execute post-benchmark-teardown");
    }
}
