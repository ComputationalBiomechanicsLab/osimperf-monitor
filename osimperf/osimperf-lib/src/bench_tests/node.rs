use super::{
    run_cmds::{run_test_cmds, FileEnvVars},
    BenchTestResult, BenchTestSetup,
};
use crate::{erase_folder, CompilationNode, Folder, Home, ResultsFolder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct TestNode<'a, 'b> {
    test: BenchTestSetup,
    compiler: CompilationNode,
    result: BenchTestResult,
    home: &'a Home,
    results: &'b ResultsFolder,
}

impl<'a, 'b> TestNode<'a, 'b> {
    pub fn new(
        test: BenchTestSetup,
        compiler: CompilationNode,
        home: &'a Home,
        results: &'b ResultsFolder,
    ) -> Result<Option<Self>> {
        if compiler.is_done() {
            Ok(Some(Self {
                result: BenchTestResult::new(results, &compiler.id(), &test.name)?,
                test,
                compiler,
                home,
                results,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn env_vars(&self) -> Result<FileEnvVars> {
        Ok(FileEnvVars {
            install: self.compiler.id().path(),
            output: self.result.path_to_node.join("output"),
            // root: self.results.test_context_dir()?,
            root: self.result.path_to_node.join("context"),
            home: self.home.path()?.to_path_buf(),
        })
    }

    pub fn run(&mut self) -> Result<&BenchTestResult> {
        let env_vars = self.env_vars()?;

        erase_folder(&env_vars.root)?;
        erase_folder(&env_vars.output)?;

        let setup_dir = self.test.test_setup_file.parent().unwrap();
        let out = run_test_cmds(&self.test.cmd, &env_vars, &setup_dir)?;

        // Write logs.
        out.write_stdout(&env_vars.output.join("stdout.log"))?;
        out.write_stderr(&env_vars.output.join("stderr.log"))?;


        // Add the hash of the current bench config.
        let hash = compute_test_config_hash(&self.test, &self.compiler);

        // Write output.
        self.result.process(out, hash)?;

        Ok(&self.result)
    }
}

fn compute_test_config_hash(test: &BenchTestSetup, compiler: &CompilationNode) -> u64 {
    combine_hashes(compute_hash(test), compute_hash(compiler))
}

fn compute_hash(x: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

fn combine_hashes(hash1: u64, hash2: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash1.hash(&mut hasher);
    hash2.hash(&mut hasher);
    hasher.finish()
}
