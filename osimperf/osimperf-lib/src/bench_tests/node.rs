use super::{
    run_cmds::{run_test_cmds, FileEnvVars},
    BenchTestResult, BenchTestSetup,
};
use crate::{CompilationNode, Folder, Home, NodeFile, ResultsFolder};
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::{collections::hash_map::DefaultHasher, path::PathBuf};

#[derive(Clone, Debug)]
pub struct TestNode<'a, 'b, 'c, 'd> {
    test: &'a BenchTestSetup,
    compiler: &'b CompilationNode,
    home: &'c Home,
    results: &'d ResultsFolder,
    result: BenchTestResult,
}

impl<'a, 'b, 'c, 'd> TestNode<'a, 'b, 'c, 'd> {
    fn new_helper(
        test: &'a BenchTestSetup,
        compiler: &'b CompilationNode,
        home: &'c Home,
        results: &'d ResultsFolder,
    ) -> Result<Self> {
        Ok(Self {
            test,
            compiler,
            home,
            results,
            result: BenchTestResult::new(results, &compiler.id(), &test.name)?,
        })
    }

    pub fn new(
        test: &'a BenchTestSetup,
        compiler: &'b CompilationNode,
        home: &'c Home,
        results: &'d ResultsFolder,
    ) -> Result<Option<Self>> {
        Ok(Some(Self::new_helper(test, compiler, home, results)?).filter(|x| x.compiler.is_done()))
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
            root: self.results.path()?.join("context"),
            home: self.home.path()?.to_path_buf(),
        })
    }

    pub fn run(&mut self) -> Result<&BenchTestResult> {
        let env_vars = self.env_vars()?;

        let setup_dir = self.test.test_setup_file.parent().unwrap();
        let out = run_test_cmds(
            &self.test.pre_benchmark_cmds,
            &self.test.benchmark_cmd,
            &self.test.post_benchmark_cmds,
            &env_vars,
            setup_dir,
            &self.test.model_files,
        )?;

        // Add the hash of the current bench config.
        let hash = compute_test_node_hash(&self.test, &self.compiler);
        self.result.update_hash(hash);

        // Set the command output result.
        self.result
            .update_result(Some(out.duration).filter(|_| out.success()));

        self.result.try_write()?;
        Ok(&self.result)
    }
}

fn compute_test_node_hash(test: &BenchTestSetup, compiler: &CompilationNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    test.hash(&mut hasher);
    compiler.hash(&mut hasher);
    hasher.finish()
}
