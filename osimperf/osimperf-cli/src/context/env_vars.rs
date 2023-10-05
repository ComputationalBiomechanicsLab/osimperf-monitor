use super::Ctxt;
use crate::InstallId;
use std::path::PathBuf;

// OSimPerf environmental variables.
pub const OPENSIM_BUILD_ENV_VAR: &str = "OSIMPERF_OPENSIM_BUILD";
pub const OPENSIM_SRC_ENV_VAR: &str = "OSIMPERF_OPENSIM_SRC";
pub const OPENSIM_INSTALL_ENV_VAR: &str = "OSIMPERF_OPENSIM_INSTALL";

/// OSIMPERF_RESULTS
/// OSIMPERF_CONFIG

pub struct EnvVar {
    pub key: String,
    pub value: String,
}

pub fn env_vars<'a>(context: &Ctxt, id: InstallId<'a>, repo: Option<PathBuf>) -> Vec<EnvVar> {
    let mut vars = vec![
        EnvVar::opensim_build_dir(context),
        EnvVar::opensim_install_dir(context, id),
    ];
    if let Some(p) = repo {
        vars.push(EnvVar::opensim_src_dir(&p));
    }
    vars
}

impl EnvVar {
    pub fn opensim_build_dir(context: &Ctxt) -> Self {
        todo!()
    }

    pub fn opensim_src_dir(repo: &PathBuf) -> Self {
        todo!()
    }

    pub fn opensim_install_dir<'a>(context: &Ctxt, id: InstallId<'a>) -> Self {
        todo!()
    }
}
