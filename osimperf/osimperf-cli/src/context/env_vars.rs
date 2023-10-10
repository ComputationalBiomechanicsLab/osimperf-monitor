use super::Ctxt;
use crate::InstallId;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::PathBuf;

// OSimPerf environmental variables.
pub const OPENSIM_BUILD_ENV_VAR: &str = "OSIMPERF_OPENSIM_BUILD";
pub const OPENSIM_SRC_ENV_VAR: &str = "OSIMPERF_OPENSIM_SRC";
pub const OPENSIM_INSTALL_ENV_VAR: &str = "OSIMPERF_OPENSIM_INSTALL";
pub const MODELS_ENV_VAR: &str = "OSIMPERF_MODELS";
pub const SETUP_ENV_VAR: &str = "OSIMPERF_SETUP";
pub const CONTEXT_ENV_VAR: &str = "OSIMPERF_CONTEXT";

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

pub fn env_vars<'a>(
    context: &Ctxt,
    id: InstallId<'a>,
    repo: Option<PathBuf>,
    ) -> Vec<EnvVar> {
    let mut vars = vec![
        EnvVar::opensim_build_dir(context),
        EnvVar::opensim_install_dir(context, id),
    ];
    if let Some(p) = repo {
        vars.push(EnvVar::opensim_src_dir(&p));
    }
    vars
}

pub fn bench_env_vars(
    mut env_vars: Vec<EnvVar>,
    test_context_dir: PathBuf,
    test_setup_dir: PathBuf,
) -> Vec<EnvVar> {
    todo!()
}

impl EnvVar {
    pub fn opensim_build_dir(context: &Ctxt) -> Self {
        Self {
            key: String::from(OPENSIM_BUILD_ENV_VAR),
            value: context.opensim_build_dir().to_string_lossy().to_string(),
        }
    }

    pub fn opensim_src_dir(repo: &PathBuf) -> Self {
        Self {
            key: String::from(OPENSIM_SRC_ENV_VAR),
            value: repo.to_string_lossy().to_string(),
        }
    }

    pub fn opensim_install_dir<'a>(context: &Ctxt, id: InstallId<'a>) -> Self {
        Self {
            key: String::from(OPENSIM_INSTALL_ENV_VAR),
            value: context
                .opensim_install_dir(id)
                .to_string_lossy()
                .to_string(),
        }
    }
}
