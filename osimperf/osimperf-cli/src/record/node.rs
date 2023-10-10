use super::{BenchTestResult, BenchTestSetup};
use crate::{
    write_json, CommandOutput, CommandTrait, CompilationNode, Ctxt,
    EnvVar, EnvVars,
};
use anyhow::{ensure, Context, Result};
use log::{info, trace, warn};
use std::hash::{Hash, Hasher};
use std::{collections::hash_map::DefaultHasher, path::PathBuf};

// TODO rename to TestNodeRunner
#[derive(Debug)]
pub struct TestNode<'a, 'b, 'c> {
    pub config: &'a BenchTestSetup,
    compiler: &'b CompilationNode,
    context: &'c Ctxt,
    result: BenchTestResult,
    path_to_result: PathBuf,
    env_var: Vec<EnvVar>,
}

impl<'a, 'b, 'c> TestNode<'a, 'b, 'c> {
    pub fn new(
        config: &'a BenchTestSetup,
        compiler: &'b CompilationNode,
        context: &'c Ctxt,
        path_to_result: PathBuf,
        env_vars: EnvVars,
    ) -> Result<Option<Self>> {
        ensure!(
            compiler.status.done(),
            "Incomplete installation of opensim-core"
        );

        // TODO Check if previous result present: load or create new.
        let result = BenchTestResult::default();

        // TODO Check hash.
        // let hash = compute_hash(&self.test, &self.compiler);
        // self.result.update_hash(hash);

        // TODO Check if max iterations exceeded.

        // Complete env vars.
        let env_var = env_vars.make();

        // Run setup commands.
        Ok(Some(Self {
            config,
            compiler,
            context,
            result,
            path_to_result,
            env_var,
        }))
    }

    pub fn pre_benchmark_setup(&self) -> Result<()> {
        let mut cmds = self.config.pre_benchmark_cmds.clone();
        for c in cmds.iter_mut() {
            c.add_envs(&self.env_var);
            c.run()?;
        }
        Ok(())
    }

    pub fn post_benchmark_teardown(self) -> Result<(PathBuf, BenchTestResult)> {
        let mut cmds = self.config.post_benchmark_cmds.clone();
        for c in cmds.iter_mut() {
            c.add_envs(&self.env_var);
            c.run()?;
        }
        Ok((self.path_to_result, self.result))
    }

    pub fn run(&mut self) -> Result<&BenchTestResult> {
        let mut benchmark_cmd = self.config.benchmark_cmd.clone();
        benchmark_cmd.add_envs(&self.env_var);

        let output = benchmark_cmd
            .run_and_time()
            .context("failed to run benchmark command")?;

        self.result.update_result(output);

        Ok(&self.result)
    }
}

fn compute_hash(config: &BenchTestSetup) -> u64 {
    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}
