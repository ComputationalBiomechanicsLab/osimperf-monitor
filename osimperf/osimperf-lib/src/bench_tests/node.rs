use crate::{
    erase_folder, CompilationNode, ResultsFolder,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    run_cmds::{run_test_cmds, FileEnvVars},
    BenchTestResult, BenchTestSetup,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestNode {
    test: BenchTestSetup,
    compiler: CompilationNode,
    result: BenchTestResult,
}

impl TestNode {
    pub fn new(
        test: BenchTestSetup,
        compiler: CompilationNode,
        results: &ResultsFolder,
    ) -> Result<Option<Self>> {
        if compiler.is_done() {
            Ok(Some(Self {
                result: BenchTestResult::new(results, &compiler.id(), &test.name)?,
                test,
                compiler,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn env_vars(&self) -> FileEnvVars {
        FileEnvVars {
            install: self.compiler.id().path(),
            output: self.result.path_to_root.join("output"),
            root: self.result.path_to_root.clone(),
        }
    }

    pub fn run(&mut self) -> Result<&BenchTestResult> {
        let env_vars = self.env_vars();

        erase_folder(&env_vars.root)?;
        erase_folder(&env_vars.output)?;

        let out = run_test_cmds(&self.test.cmd, &env_vars)?;

        // Write logs.
        out.write_stdout(&env_vars.output.join("stdout.log"))?;
        out.write_stderr(&env_vars.output.join("stderr.log"))?;

        // Write output.
        self.result.process(out)?;

        Ok(&self.result)
    }
}
